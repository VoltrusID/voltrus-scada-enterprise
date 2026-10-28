//! The replication sessions: the primary shipping changes to its standby, the
//! standby applying them, full synchronisation, and the controlled hand-over.

use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, timeout};
use voltrus::storage::Storage;

use super::link::Peer;
use super::node::{Node, Promote, PrimaryCmd, StandbyCmd, SwitchoverReport, can_resume, lock, now_ms};
use super::store::{self, Position, SnapshotTable, TableChanges, TableSchema};
use super::wire::{Hello, Msg};
use super::{FullSyncReport, HaRole, SyncState};

/// Batches in flight before the primary waits for acknowledgements.
const WINDOW: usize = 8;
const BATCH_ENTRIES: usize = 5000;
const CHUNK_ROWS: usize = 2000;
const CHUNK_BYTES: usize = 1 << 20;
const SCHEMA_REFRESH: Duration = Duration::from_secs(30);

// ── a second, read-only connection for building batches and snapshots ──

struct ReaderInner {
    conn: rusqlite::Connection,
    schemas: Option<(Instant, Arc<BTreeMap<String, TableSchema>>)>,
}

impl ReaderInner {
    fn schemas(&mut self, force: bool) -> Result<Arc<BTreeMap<String, TableSchema>>> {
        if let Some((at, s)) = &self.schemas
            && !force
            && at.elapsed() < SCHEMA_REFRESH
        {
            return Ok(s.clone());
        }
        let s = Arc::new(store::table_schemas(&self.conn)?);
        self.schemas = Some((Instant::now(), s.clone()));
        Ok(s)
    }
}

struct Reader(Arc<Mutex<ReaderInner>>);

impl Reader {
    fn open(storage: &Storage) -> Result<Self> {
        Ok(Self(Arc::new(Mutex::new(ReaderInner {
            conn: storage.open_reader()?,
            schemas: None,
        }))))
    }

    async fn run<T, F>(&self, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut ReaderInner) -> Result<T> + Send + 'static,
    {
        let inner = self.0.clone();
        tokio::task::spawn_blocking(move || f(&mut lock(&inner)))
            .await
            .context("reader task failed")?
    }
}

// ── the primary's session state ──

struct Sess {
    /// Highest sequence sent.
    sent: u64,
    /// Highest sequence the standby acknowledged. Zero until known.
    acked: u64,
    inflight: VecDeque<(u64, Instant)>,
    pending_full: Option<PendingFull>,
    resync_waiters: Vec<oneshot::Sender<Result<()>>>,
}

struct PendingFull {
    snapshot_seq: u64,
    report: FullSyncReport,
}

impl Node {
    // ───────────────────────── primary side ─────────────────────────

    pub(super) async fn serve_standby(self: &Arc<Self>, peer: Peer, ph: Hello) -> Result<()> {
        let gen_id = self.session_gen.fetch_add(1, Ordering::SeqCst) + 1;
        let (cmd_tx, cmd_rx) = mpsc::channel(8);
        *lock(&self.primary_cmd) = Some(cmd_tx);
        {
            let mut d = lock(&self.dynamic);
            d.session_up = true;
            d.sync_error = None;
        }
        tracing::info!("HA: standby {} attached (epoch {}, position {})", ph.node_id, ph.epoch, ph.applied_seq);
        let result = self.serve_loop(peer, ph, gen_id, cmd_rx).await;
        if self.session_gen.load(Ordering::SeqCst) == gen_id {
            *lock(&self.primary_cmd) = None;
            let mut d = lock(&self.dynamic);
            d.session_up = false;
            d.lag_ms = None;
            if matches!(d.sync, SyncState::Streaming | SyncState::FullSync { .. }) {
                d.sync = SyncState::Waiting;
            }
        }
        if let Err(e) = &result {
            tracing::warn!("HA: standby session ended: {e:#}");
        }
        result
    }

    fn session_is_current(&self, gen_id: u64) -> bool {
        self.session_gen.load(Ordering::SeqCst) == gen_id
            && (self.role() == HaRole::Primary || self.switching.load(Ordering::SeqCst))
    }

    async fn serve_loop(
        self: &Arc<Self>,
        mut peer: Peer,
        ph: Hello,
        gen_id: u64,
        mut cmd_rx: mpsc::Receiver<PrimaryCmd>,
    ) -> Result<()> {
        let reader = Reader::open(&self.storage)?;
        let (floor, max_seq) = self
            .storage
            .with_conn(|c| Ok((store::log_floor(c)?, store::max_seq(c)?)))
            .await?;
        let mut s = Sess {
            sent: 0,
            acked: 0,
            inflight: VecDeque::new(),
            pending_full: None,
            resync_waiters: Vec::new(),
        };
        if can_resume(&ph, self.epoch(), floor, max_seq) {
            peer.send(&Msg::PlanIncremental { from_seq: ph.applied_seq }).await?;
            s.sent = ph.applied_seq;
            s.acked = ph.applied_seq;
            lock(&self.dynamic).sync = SyncState::Streaming;
        } else {
            tracing::info!(
                "HA: standby needs a full sync (its epoch {} / position {} vs ours {} / log {floor}..{max_seq}, needs_full_sync={})",
                ph.epoch,
                ph.applied_seq,
                self.epoch(),
                ph.needs_full_sync
            );
            self.send_full_sync(&mut peer, &reader, &mut s).await?;
        }

        let mut ship = tokio::time::interval(self.cfg.sync_interval);
        ship.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut beat = tokio::time::interval(self.cfg.heartbeat);
        let mut since_prune = Instant::now();
        loop {
            if !self.session_is_current(gen_id) {
                bail!("this node is no longer the primary of this session");
            }
            tokio::select! {
                _ = ship.tick() => {
                    if self.role() == HaRole::Primary {
                        self.ship_pending(&mut peer, &reader, &mut s).await?;
                    }
                }
                _ = beat.tick() => {
                    if self.role() == HaRole::Primary {
                        self.send_heartbeat(&mut peer, &reader, &s).await?;
                    }
                    if since_prune.elapsed() >= Duration::from_secs(1) && s.acked > 0 {
                        since_prune = Instant::now();
                        let through = s.acked;
                        self.storage.with_conn(move |c| store::prune_log(c, through)).await?;
                    }
                }
                msg = peer.recv() => {
                    match msg {
                        None => bail!("standby closed the link"),
                        Some(Err(e)) => bail!("link error: {e}"),
                        Some(Ok(m)) => {
                            if self.on_standby_msg(&mut peer, &reader, &mut s, m).await? {
                                return Ok(());
                            }
                        }
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        None => bail!("session control closed"),
                        Some(PrimaryCmd::Resync(reply)) => {
                            s.resync_waiters.push(reply);
                            self.send_full_sync(&mut peer, &reader, &mut s).await?;
                        }
                        Some(PrimaryCmd::Switchover(reply)) => {
                            let r = self.handoff(&mut peer, &reader, &mut s).await;
                            let done = r.is_ok();
                            let _ = reply.send(r);
                            if done {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    /// Handle one message of the standby. Returns true when the session is over.
    async fn on_standby_msg(
        self: &Arc<Self>,
        peer: &mut Peer,
        reader: &Reader,
        s: &mut Sess,
        msg: Msg,
    ) -> Result<bool> {
        lock(&self.dynamic).peer_seen = Some(Instant::now());
        match msg {
            Msg::Ack { seq } => self.on_ack(reader, s, seq).await,
            Msg::HeartbeatAck { applied_seq, .. } => {
                lock(&self.dynamic).last_seq = Some(applied_seq);
            }
            Msg::ResyncRequest => {
                self.send_full_sync(peer, reader, s).await?;
            }
            Msg::SwitchoverRequest => {
                // Run it like an operator call on this node; nobody awaits the report.
                let node = self.clone();
                let tx = lock(&self.primary_cmd).clone();
                if let Some(tx) = tx {
                    tokio::spawn(async move {
                        let (reply_tx, reply_rx) = oneshot::channel();
                        if tx.send(PrimaryCmd::Switchover(reply_tx)).await.is_ok() {
                            if let Ok(Err(e)) = reply_rx.await {
                                tracing::warn!("HA: requested switchover failed: {e:#}");
                            }
                        }
                        drop(node);
                    });
                }
            }
            Msg::Reject { reason } => bail!("standby refused the session: {reason}"),
            other => tracing::debug!("HA: unexpected message from the standby: {other:?}"),
        }
        Ok(false)
    }

    async fn on_ack(&self, reader: &Reader, s: &mut Sess, seq: u64) {
        s.acked = s.acked.max(seq);
        while s.inflight.front().is_some_and(|(to, _)| *to <= seq) {
            s.inflight.pop_front();
        }
        let lag = s.inflight.front().map_or(0, |(_, at)| at.elapsed().as_millis() as u64);
        let max = reader.run(|r| store::max_seq(&r.conn)).await.unwrap_or(s.sent);
        {
            let mut d = lock(&self.dynamic);
            d.lag_ms = Some(lag);
            d.pending = Some(max.saturating_sub(s.acked));
            d.last_seq = Some(s.acked);
            d.last_sync_at = Some(voltrus::timeutil::now_secs());
            if let Some(p) = s.pending_full.take_if(|p| seq >= p.snapshot_seq) {
                d.last_full_sync = Some(p.report);
                d.sync = SyncState::Streaming;
                for w in s.resync_waiters.drain(..) {
                    let _ = w.send(Ok(()));
                }
                self.sync_tx.send_modify(|g| *g += 1);
            }
        }
    }

    async fn send_heartbeat(&self, peer: &mut Peer, reader: &Reader, s: &Sess) -> Result<()> {
        let max_seq = reader.run(|r| store::max_seq(&r.conn)).await.unwrap_or(s.sent);
        let lag = s.inflight.front().map_or(0, |(_, at)| at.elapsed().as_millis() as u64);
        {
            let mut d = lock(&self.dynamic);
            d.lag_ms = Some(lag);
            d.pending = Some(max_seq.saturating_sub(s.acked));
        }
        peer.send(&Msg::Heartbeat {
            epoch: self.epoch(),
            max_seq,
            lag_ms: lag,
            pollers: self.pollers.running_ids().await,
            now_ms: now_ms(),
        })
        .await
    }

    async fn ship_pending(&self, peer: &mut Peer, reader: &Reader, s: &mut Sess) -> Result<()> {
        // Nothing is shipped while a full sync waits for its acknowledgement
        // only in the sense of window size: batches after the snapshot are valid.
        while s.inflight.len() < WINDOW {
            let from = s.sent;
            let batch = reader
                .run(move |r| {
                    r.conn.execute_batch("BEGIN")?;
                    let built = (|| {
                        let entries = store::read_log(&r.conn, from, BATCH_ENTRIES)?;
                        if entries.is_empty() {
                            return Ok(None);
                        }
                        let mut schemas = r.schemas(false)?;
                        if entries
                            .iter()
                            .any(|e| store::is_replicated(&e.tbl) && !schemas.contains_key(&e.tbl))
                        {
                            schemas = r.schemas(true)?;
                        }
                        Ok::<_, anyhow::Error>(Some(store::build_batch(&r.conn, &schemas, &entries)?))
                    })();
                    let _ = r.conn.execute_batch("COMMIT");
                    built
                })
                .await?;
            let Some(batch) = batch else { break };
            let seq_to = batch.seq_to;
            peer.send(&Msg::Batch(batch)).await?;
            s.sent = seq_to;
            s.inflight.push_back((seq_to, Instant::now()));
        }
        Ok(())
    }

    /// Send a consistent snapshot of every replicated table, then continue
    /// incrementally after it.
    async fn send_full_sync(&self, peer: &mut Peer, reader: &Reader, s: &mut Sess) -> Result<()> {
        let started = Instant::now();
        let (snapshot_seq, tables) = reader
            .run(|r| {
                r.conn.execute_batch("BEGIN")?;
                let seq = store::max_seq(&r.conn)?;
                let tables = r.schemas(true)?;
                Ok((seq, tables))
            })
            .await?;
        let result = self.send_snapshot(peer, reader, snapshot_seq, &tables).await;
        let _ = reader
            .run(|r| {
                let _ = r.conn.execute_batch("ROLLBACK");
                Ok(())
            })
            .await;
        let rows = result?;
        s.sent = snapshot_seq;
        s.acked = 0;
        s.inflight.clear();
        s.pending_full = Some(PendingFull {
            snapshot_seq,
            report: FullSyncReport {
                at: super::rfc3339(voltrus::timeutil::now_secs()).unwrap_or_default(),
                tables: tables.len(),
                rows,
                duration_ms: started.elapsed().as_millis() as u64,
            },
        });
        tracing::info!(
            "HA: full sync sent: {} tables, {rows} rows in {} ms, continuing after change {snapshot_seq}",
            tables.len(),
            started.elapsed().as_millis()
        );
        Ok(())
    }

    async fn send_snapshot(
        &self,
        peer: &mut Peer,
        reader: &Reader,
        snapshot_seq: u64,
        tables: &Arc<BTreeMap<String, TableSchema>>,
    ) -> Result<u64> {
        peer.send(&Msg::PlanFull {
            snapshot_seq,
            tables: tables.values().map(SnapshotTable::from).collect(),
        })
        .await?;
        let total = tables.len();
        let mut rows_sent = 0u64;
        for (done, schema) in tables.values().enumerate() {
            lock(&self.dynamic).sync = SyncState::FullSync {
                tables_done: done,
                tables_total: total,
                rows: rows_sent,
            };
            let mut cursor = 0i64;
            loop {
                let sch = schema.clone();
                let (rows, next) = reader
                    .run(move |r| store::snapshot_rows(&r.conn, &sch, cursor, CHUNK_ROWS, CHUNK_BYTES))
                    .await?;
                if !rows.is_empty() {
                    rows_sent += rows.len() as u64;
                    peer.send(&Msg::Chunk {
                        table: schema.name.clone(),
                        cols: schema.cols.clone(),
                        key: schema.key.clone(),
                        rows,
                    })
                    .await?;
                }
                match next {
                    Some(c) => cursor = c,
                    None => break,
                }
            }
        }
        peer.send(&Msg::FullEnd { snapshot_seq }).await?;
        Ok(rows_sent)
    }

    // ── the controlled hand-over, primary side ──

    async fn handoff(
        self: &Arc<Self>,
        peer: &mut Peer,
        reader: &Reader,
        s: &mut Sess,
    ) -> Result<SwitchoverReport> {
        let started = Instant::now();
        if s.pending_full.is_some() {
            bail!("the standby is still taking a full sync; try again when it has finished");
        }
        self.switching.store(true, Ordering::SeqCst);
        let result = self.handoff_inner(peer, reader, s, started).await;
        self.switching.store(false, Ordering::SeqCst);
        if let Err(e) = &result {
            tracing::warn!("HA: switchover failed: {e:#}");
            if self.role() == HaRole::Fenced {
                if let Err(e2) = self.resume_primary().await {
                    tracing::error!("HA: could not resume as primary after a failed switchover: {e2:#}");
                }
            }
        }
        result
    }

    async fn handoff_inner(
        self: &Arc<Self>,
        peer: &mut Peer,
        reader: &Reader,
        s: &mut Sess,
        started: Instant,
    ) -> Result<SwitchoverReport> {
        // 1. Stop acting. From here no device is written and no sample stored.
        {
            let _g = self.transition.lock().await;
            if self.role() != HaRole::Primary {
                bail!("this node is not the primary");
            }
            self.go_passive_for_handoff().await;
        }
        // 2. Let requests that were already running finish, then ship everything
        //    until the log stops growing and the standby has all of it.
        sleep(Duration::from_millis(300)).await;
        let deadline = Instant::now() + Duration::from_secs(15);
        let final_seq = loop {
            let target = self.storage.with_conn(|c| store::max_seq(c)).await?;
            while s.acked < target {
                if Instant::now() > deadline {
                    bail!("the standby did not catch up in time ({} of {target} changes)", s.acked);
                }
                self.ship_pending(peer, reader, s).await?;
                match timeout(Duration::from_millis(100), peer.recv()).await {
                    Ok(Some(Ok(Msg::Ack { seq }))) => self.on_ack(reader, s, seq).await,
                    Ok(Some(Ok(Msg::HeartbeatAck { .. }))) => {}
                    Ok(Some(Ok(_))) => {}
                    Ok(Some(Err(e))) => bail!("link error: {e}"),
                    Ok(None) => bail!("standby closed the link"),
                    Err(_) => {}
                }
            }
            if self.storage.with_conn(|c| store::max_seq(c)).await? == target {
                break target;
            }
        };
        // 3. Give the lease up so the standby can take it, then tell it to go.
        if let Err(e) = self.witness.release(&self.cfg.node_id, self.epoch()).await {
            bail!("could not release the witness lease: {e:#}");
        }
        peer.send(&Msg::Handoff { final_seq }).await?;
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match timeout(left.max(Duration::from_millis(1)), peer.recv()).await {
                Ok(Some(Ok(Msg::Promoted { epoch }))) => {
                    // The standby has every change through final_seq and is primary
                    // in `epoch`: this node follows it from exactly that point.
                    self.finish_handoff(epoch, final_seq).await?;
                    return Ok(SwitchoverReport {
                        new_primary: peer.node_id.clone(),
                        epoch,
                        duration_ms: started.elapsed().as_millis() as u64,
                    });
                }
                Ok(Some(Ok(Msg::HandoffRefused { reason }))) => bail!("the standby refused to take over: {reason}"),
                Ok(Some(Ok(_))) => {}
                Ok(Some(Err(e))) => bail!("link error: {e}"),
                Ok(None) => bail!("the standby closed the link before confirming"),
                Err(_) => bail!("the standby did not confirm the takeover in time"),
            }
        }
    }

    // ───────────────────────── standby side ─────────────────────────

    pub(super) async fn follow_primary(self: &Arc<Self>, mut peer: Peer, ph: Hello) -> Result<()> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(8);
        *lock(&self.standby_cmd) = Some(cmd_tx);
        {
            let mut d = lock(&self.dynamic);
            d.session_up = true;
            d.sync_error = None;
            d.primary_contact = Some(Instant::now());
        }
        tracing::info!("HA: following primary {} (epoch {})", ph.node_id, ph.epoch);
        let silence = self.cfg.failover_timeout();
        let mut full: Option<FullApply> = None;
        let result: Result<()> = async {
            loop {
                tokio::select! {
                    msg = timeout(silence, peer.recv()) => {
                        let msg = match msg {
                            Err(_) => bail!("the primary has been silent for {} ms", silence.as_millis()),
                            Ok(None) => bail!("the primary closed the link"),
                            Ok(Some(Err(e))) => bail!("link error: {e}"),
                            Ok(Some(Ok(m))) => m,
                        };
                        {
                            let mut d = lock(&self.dynamic);
                            d.primary_contact = Some(Instant::now());
                            d.peer_seen = Some(Instant::now());
                        }
                        if self.on_primary_msg(&mut peer, &ph, &mut full, msg).await? {
                            return Ok(());
                        }
                    }
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(StandbyCmd::Resync) => peer.send(&Msg::ResyncRequest).await?,
                            Some(StandbyCmd::Switchover) => peer.send(&Msg::SwitchoverRequest).await?,
                            None => {}
                        }
                    }
                }
            }
        }
        .await;
        *lock(&self.standby_cmd) = None;
        {
            let mut d = lock(&self.dynamic);
            d.session_up = false;
            if let Err(e) = &result {
                if matches!(d.sync, SyncState::FullSync { .. }) {
                    d.sync = SyncState::Error(format!("full sync interrupted: {e:#}"));
                } else if matches!(d.sync, SyncState::Streaming) {
                    d.sync = SyncState::Waiting;
                }
            }
        }
        if let Err(e) = &result {
            tracing::warn!("HA: lost the primary: {e:#}");
        }
        result
    }

    /// Handle one message of the primary. Returns true when the session is over
    /// because this node became primary.
    async fn on_primary_msg(
        self: &Arc<Self>,
        peer: &mut Peer,
        ph: &Hello,
        full: &mut Option<FullApply>,
        msg: Msg,
    ) -> Result<bool> {
        match msg {
            Msg::PlanIncremental { from_seq } => {
                lock(&self.dynamic).sync = SyncState::Streaming;
                tracing::info!("HA: incremental sync from change {from_seq}");
            }
            Msg::PlanFull { snapshot_seq, tables } => {
                tracing::info!("HA: full sync started ({} tables, snapshot at change {snapshot_seq})", tables.len());
                let tables2 = tables.clone();
                self.storage.with_conn(move |c| store::begin_full_sync(c, &tables2)).await?;
                lock(&self.dynamic).needs_full_sync = true;
                lock(&self.dynamic).sync = SyncState::FullSync { tables_done: 0, tables_total: tables.len(), rows: 0 };
                *full = Some(FullApply { total: tables.len(), done: 0, current: None, rows: 0 });
            }
            Msg::Chunk { table, cols, key, rows } => {
                let Some(f) = full.as_mut() else { bail!("a chunk arrived outside a full sync") };
                if f.current.as_deref() != Some(table.as_str()) {
                    if f.current.is_some() {
                        f.done += 1;
                    }
                    f.current = Some(table.clone());
                }
                f.rows += rows.len() as u64;
                let n = rows.len() as u64;
                let changes = TableChanges { name: table, cols, key, rows, ..Default::default() };
                self.storage
                    .with_conn(move |c| store::apply_changes(c, &[changes], None))
                    .await
                    .context("applying a full sync chunk")?;
                let _ = n;
                lock(&self.dynamic).sync = SyncState::FullSync {
                    tables_done: f.done,
                    tables_total: f.total,
                    rows: f.rows,
                };
            }
            Msg::FullEnd { snapshot_seq } => {
                let f = full.take().context("a full sync end arrived outside a full sync")?;
                let epoch = ph.epoch;
                self.storage
                    .with_conn(move |c| store::finish_full_sync(c, epoch, snapshot_seq))
                    .await?;
                self.adopt_epoch(epoch).await?;
                {
                    let mut d = lock(&self.dynamic);
                    d.needs_full_sync = false;
                    d.sync = SyncState::Streaming;
                    d.sync_error = None;
                    d.last_seq = Some(snapshot_seq);
                    d.last_sync_at = Some(voltrus::timeutil::now_secs());
                    d.last_full_sync = Some(FullSyncReport {
                        at: super::rfc3339(voltrus::timeutil::now_secs()).unwrap_or_default(),
                        tables: f.total,
                        rows: f.rows,
                        duration_ms: 0,
                    });
                }
                self.sync_tx.send_modify(|g| *g += 1);
                tracing::info!("HA: full sync complete at change {snapshot_seq} ({} rows)", f.rows);
                peer.send(&Msg::Ack { seq: snapshot_seq }).await?;
            }
            Msg::Batch(batch) => {
                let (epoch, seq_to) = (self.epoch(), batch.seq_to);
                let tables = batch.tables;
                let res = self
                    .storage
                    .with_conn(move |c| {
                        store::apply_changes(
                            c,
                            &tables,
                            Some(Position { epoch, applied_seq: seq_to, needs_full_sync: false }),
                        )
                    })
                    .await;
                match res {
                    Ok(_) => {
                        {
                            let mut d = lock(&self.dynamic);
                            d.last_seq = Some(seq_to);
                            d.last_sync_at = Some(voltrus::timeutil::now_secs());
                            d.sync = SyncState::Streaming;
                            d.sync_error = None;
                            d.needs_full_sync = false;
                        }
                        peer.send(&Msg::Ack { seq: seq_to }).await?;
                    }
                    Err(e) => {
                        let msg = format!("{e:#}");
                        lock(&self.dynamic).sync = SyncState::Error(msg.clone());
                        // A change that cannot be applied leaves the copy behind: take a full sync.
                        let _ = self.update_record(|r| r.needs_full_sync = true).await;
                        return Err(e.context("applying a change batch"));
                    }
                }
            }
            Msg::Heartbeat { epoch, max_seq, lag_ms, pollers, .. } => {
                if epoch > self.epoch() {
                    // The same primary re-numbered its term (its witness did): follow it.
                    self.adopt_epoch(epoch).await?;
                }
                let applied = lock(&self.dynamic).last_seq.unwrap_or(0);
                let changed = {
                    let mut d = lock(&self.dynamic);
                    d.lag_ms = Some(lag_ms);
                    d.pending = Some(max_seq.saturating_sub(applied));
                    let changed = d.peer_running != pollers;
                    d.peer_running = pollers.clone();
                    d.peer_epoch = Some(epoch);
                    changed
                };
                if changed {
                    let _ = self.update_record(move |r| r.peer_running = pollers).await;
                }
                peer.send(&Msg::HeartbeatAck { applied_seq: applied, now_ms: now_ms() }).await?;
            }
            Msg::Handoff { final_seq } => return self.accept_handoff(peer, final_seq).await,
            Msg::Reject { reason } => bail!("the primary refused the session: {reason}"),
            other => tracing::debug!("HA: unexpected message from the primary: {other:?}"),
        }
        Ok(false)
    }

    /// The primary has stopped and has shipped everything through `final_seq`.
    async fn accept_handoff(self: &Arc<Self>, peer: &mut Peer, final_seq: u64) -> Result<bool> {
        let rec = self.storage.with_conn(|c| store::load_node(c)).await?;
        let applied = rec.as_ref().map_or(0, |r| r.applied_seq);
        if applied < final_seq {
            let reason = format!("this standby has applied change {applied} but the primary stopped at {final_seq}");
            peer.send(&Msg::HandoffRefused { reason: reason.clone() }).await?;
            bail!(reason);
        }
        match self.promote(Promote::Handoff { final_seq }).await {
            Ok(t) => {
                peer.send(&Msg::Promoted { epoch: t.epoch }).await?;
                Ok(true)
            }
            Err(e) => {
                let _ = peer.send(&Msg::HandoffRefused { reason: format!("{e:#}") }).await;
                Err(e)
            }
        }
    }
}

/// Progress of a full sync on the standby.
pub(super) struct FullApply {
    total: usize,
    done: usize,
    current: Option<String>,
    rows: u64,
}
