//! The state of this node in a pair, its role transitions and its background
//! tasks. The replication sessions live in `session.rs`.
//!
//! # Roles
//!
//! * `Primary`: acts. Polls devices, writes to them, stores history, evaluates
//!   alarms, ships its changes to the standby.
//! * `Standby`: passive. Follows the primary, never writes to devices, never
//!   stores samples. Takes over when the primary is gone.
//! * `Fenced`: wants to be primary but may not act, because it lost its lease
//!   (or both its peer and its witness). Passive until the doubt is resolved.
//!
//! # Terms (epochs)
//!
//! Every role change that creates a new primary starts a new epoch, persisted
//! with the role before the node acts. A node that learns of a higher epoch
//! than its own stops acting at once. Two primaries at the same epoch (both
//! started as primary, or both promoted in the same instant) are ordered by
//! node id: the lower id keeps the role. A demoted node's data may contain
//! changes the new primary never received, so it takes a full sync before it
//! follows again.

use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, mpsc, oneshot, watch};
use tokio::time::{sleep, timeout};
use voltrus::poller_manager::PollerManager;
use voltrus::storage::Storage;

use super::link::{LinkSecurity, Peer};
use super::settings::HaSettings;
use super::store::{self, NodeRecord, StoredRole};
use super::wire::{Hello, Msg, PROTOCOL_VERSION, WireRole};
use super::witness::{Reply, Witness};
use super::{FullSyncReport, HaRole, HaStatus, SyncState, WitnessStatus};

/// Largest log the primary keeps while no standby acknowledges. Beyond it the
/// oldest entries go and a standby that far behind takes a full sync.
pub(super) const LOG_CAP: u64 = 4_000_000;
const HELLO_TIMEOUT: Duration = Duration::from_secs(5);

/// A message to the active session of the primary.
pub(super) enum PrimaryCmd {
    Resync(oneshot::Sender<Result<()>>),
    Switchover(oneshot::Sender<Result<SwitchoverReport>>),
}

/// A message to the active session of the standby.
pub(super) enum StandbyCmd {
    Resync,
    Switchover,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SwitchoverReport {
    pub new_primary: String,
    pub epoch: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Transition {
    pub role: HaRole,
    pub epoch: u64,
    pub duration_ms: u64,
}

pub(super) enum Promote {
    /// The primary has been silent too long.
    Auto(String),
    /// An operator asked for it. `force` skips the data and reachability checks.
    Operator { force: bool },
    /// The primary handed over and has stopped acting. The standby has every
    /// change through `final_seq`.
    Handoff { final_seq: u64 },
}

#[derive(Default)]
pub(super) struct Dyn {
    pub peer_id: Option<String>,
    pub peer_role: Option<HaRole>,
    pub peer_epoch: Option<u64>,
    /// Last time the peer answered, in any role.
    pub peer_seen: Option<Instant>,
    /// Last time a primary answered (standby side).
    pub primary_contact: Option<Instant>,
    pub session_up: bool,
    pub sync: SyncState,
    pub lag_ms: Option<u64>,
    pub pending: Option<u64>,
    pub last_seq: Option<u64>,
    pub last_sync_at: Option<i64>,
    pub sync_error: Option<String>,
    pub failover_at: Option<i64>,
    pub failover_reason: Option<String>,
    pub role_since: Option<Instant>,
    pub needs_full_sync: bool,
    pub peer_running: Vec<String>,
    pub witness_ok: Option<bool>,
    pub witness_detail: Option<String>,
    pub last_full_sync: Option<FullSyncReport>,
    pub promotion_blocked: Option<String>,
    pub peer_lost_since: Option<Instant>,
}

pub struct Node {
    pub(super) cfg: HaSettings,
    pub(super) security: LinkSecurity,
    pub(super) witness: Witness,
    pub(super) storage: Storage,
    pub(super) pollers: PollerManager,
    role: AtomicU8,
    epoch: AtomicU64,
    started: Instant,
    pub(super) transition: tokio::sync::Mutex<()>,
    pub(super) dynamic: Mutex<Dyn>,
    pub(super) primary_cmd: Mutex<Option<mpsc::Sender<PrimaryCmd>>>,
    pub(super) standby_cmd: Mutex<Option<mpsc::Sender<StandbyCmd>>>,
    pub(super) session_gen: AtomicU64,
    /// Bumped on every role change.
    pub(super) role_tx: watch::Sender<u64>,
    /// Bumped each time a full sync completes.
    pub(super) sync_tx: watch::Sender<u64>,
    lease_until: Mutex<Option<Instant>>,
    inbound: Arc<Semaphore>,
    /// A controlled hand-over is running: the node is passive on purpose.
    pub(super) switching: AtomicBool,
}

/// Where a node stands when it starts.
#[derive(Debug, Clone)]
pub struct Boot {
    pub role: HaRole,
    pub epoch: u64,
}

fn role_to_u8(r: HaRole) -> u8 {
    match r {
        HaRole::Primary => 0,
        HaRole::Standby => 1,
        HaRole::Fenced => 2,
    }
}

fn role_from_u8(v: u8) -> HaRole {
    match v {
        0 => HaRole::Primary,
        1 => HaRole::Standby,
        _ => HaRole::Fenced,
    }
}

pub(super) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub(super) fn now_ms() -> i64 {
    voltrus::timeutil::now_millis()
}

impl Node {
    pub fn new(
        cfg: HaSettings,
        security: LinkSecurity,
        storage: Storage,
        pollers: PollerManager,
        boot: Boot,
        record: &NodeRecord,
    ) -> Result<Arc<Self>> {
        let witness = Witness::from_config(cfg.witness.as_ref(), &security)?;
        let now = Instant::now();
        let dynamic = Dyn {
            role_since: Some(now),
            needs_full_sync: record.needs_full_sync,
            last_seq: Some(record.applied_seq).filter(|_| boot.role == HaRole::Standby),
            failover_at: record.last_failover_at,
            failover_reason: record.last_failover_reason.clone(),
            peer_running: record.peer_running.clone(),
            sync: SyncState::Waiting,
            ..Default::default()
        };
        Ok(Arc::new(Self {
            cfg,
            security,
            witness,
            storage,
            pollers,
            role: AtomicU8::new(role_to_u8(boot.role)),
            epoch: AtomicU64::new(boot.epoch),
            started: now,
            transition: tokio::sync::Mutex::new(()),
            dynamic: Mutex::new(dynamic),
            primary_cmd: Mutex::new(None),
            standby_cmd: Mutex::new(None),
            session_gen: AtomicU64::new(0),
            role_tx: watch::channel(0).0,
            sync_tx: watch::channel(0).0,
            lease_until: Mutex::new(None),
            inbound: Arc::new(Semaphore::new(16)),
            switching: AtomicBool::new(false),
        }))
    }

    pub fn role(&self) -> HaRole {
        role_from_u8(self.role.load(Ordering::SeqCst))
    }

    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::SeqCst)
    }

    pub(super) fn set_role(&self, role: HaRole) {
        self.role.store(role_to_u8(role), Ordering::SeqCst);
        lock(&self.dynamic).role_since = Some(Instant::now());
        self.role_tx.send_modify(|g| *g += 1);
    }

    pub(super) fn write_timeout(&self) -> Duration {
        (self.cfg.failover_timeout() * 3).max(Duration::from_secs(15))
    }

    pub(super) fn lease_ttl(&self) -> Duration {
        self.cfg.failover_timeout().max(Duration::from_secs(1))
    }

    /// Spawn the listener, the connector and the monitor.
    pub fn start(self: &Arc<Self>) {
        tokio::spawn(self.clone().run_listener());
        tokio::spawn(self.clone().run_connector());
        tokio::spawn(self.clone().run_monitor());
        tracing::info!(
            "HA node {} started as {:?} at epoch {} (peer {}, failover after {} ms)",
            self.cfg.node_id,
            self.role(),
            self.epoch(),
            self.cfg.peer_address,
            self.cfg.failover_timeout().as_millis()
        );
    }

    // ── the persistent record ──

    pub(super) async fn update_record<F>(&self, f: F) -> Result<NodeRecord>
    where
        F: FnOnce(&mut NodeRecord) + Send + 'static,
    {
        let node_id = self.cfg.node_id.clone();
        let rec = self
            .storage
            .with_conn(move |c| {
                let mut rec = store::load_node(c)?.unwrap_or(NodeRecord {
                    node_id,
                    role: StoredRole::Standby,
                    epoch: 0,
                    applied_seq: 0,
                    needs_full_sync: true,
                    peer_running: vec![],
                    last_failover_at: None,
                    last_failover_reason: None,
                });
                f(&mut rec);
                store::save_node(c, &rec)?;
                Ok(rec)
            })
            .await?;
        lock(&self.dynamic).needs_full_sync = rec.needs_full_sync;
        Ok(rec)
    }

    /// Adopt a higher epoch of the primary this node follows, or of the witness.
    pub(super) async fn adopt_epoch(&self, epoch: u64) -> Result<()> {
        if epoch <= self.epoch() {
            return Ok(());
        }
        self.update_record(move |r| r.epoch = epoch.max(r.epoch)).await?;
        self.epoch.fetch_max(epoch, Ordering::SeqCst);
        Ok(())
    }

    pub(super) async fn hello(&self) -> Hello {
        let (applied_seq, needs_full_sync, max_seq) = self
            .storage
            .with_conn(|c| {
                let rec = store::load_node(c)?;
                Ok((
                    rec.as_ref().map_or(0, |r| r.applied_seq),
                    rec.as_ref().is_none_or(|r| r.needs_full_sync),
                    store::max_seq(c).unwrap_or(0),
                ))
            })
            .await
            .unwrap_or((0, true, 0));
        Hello {
            proto: PROTOCOL_VERSION,
            node_id: self.cfg.node_id.clone(),
            role: self.role().to_wire(),
            epoch: self.epoch(),
            applied_seq,
            needs_full_sync,
            max_seq,
            version: env!("CARGO_PKG_VERSION").to_string(),
            now_ms: now_ms(),
        }
    }

    // ── tasks ──

    async fn run_listener(self: Arc<Self>) {
        let listener = loop {
            match TcpListener::bind(&self.cfg.bind_address).await {
                Ok(l) => break l,
                Err(e) => {
                    tracing::error!("HA: cannot listen on {}: {e}", self.cfg.bind_address);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        };
        tracing::info!("HA link listening on {}", self.cfg.bind_address);
        loop {
            match listener.accept().await {
                Ok((tcp, _)) => {
                    // Unauthenticated connections are cheap to open; cap how many are in flight.
                    let Ok(permit) = self.inbound.clone().try_acquire_owned() else {
                        continue;
                    };
                    let node = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = node.handle_inbound(tcp, permit).await {
                            tracing::debug!("HA inbound link ended: {e:#}");
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("HA accept failed: {e}");
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }
    }

    async fn handle_inbound(
        self: &Arc<Self>,
        tcp: TcpStream,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) -> Result<()> {
        let link = match self.security.accept(tcp).await {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!("HA: rejected an inbound link: {e:#}");
                return Ok(());
            }
        };
        drop(permit);
        let mut peer = link.into_peer(self.write_timeout());
        let ph = match timeout(HELLO_TIMEOUT, peer.recv()).await {
            Ok(Some(Ok(Msg::Hello(h)))) => h,
            _ => bail!("peer sent no hello"),
        };
        if let Err(reason) = self.check_peer(&ph, &peer.node_id) {
            let _ = peer.send(&Msg::Reject { reason: reason.clone() }).await;
            bail!(reason);
        }
        peer.send(&Msg::Hello(self.hello().await)).await?;
        self.observe_peer(&ph).await;
        if self.role() == HaRole::Primary && ph.role == WireRole::Standby {
            self.serve_standby(peer, ph).await
        } else {
            Ok(())
        }
    }

    fn check_peer(&self, ph: &Hello, authenticated_id: &str) -> std::result::Result<(), String> {
        if ph.proto != PROTOCOL_VERSION {
            let msg = format!(
                "peer speaks link protocol {} but this node speaks {PROTOCOL_VERSION}: run the same version on both nodes",
                ph.proto
            );
            lock(&self.dynamic).sync_error = Some(msg.clone());
            return Err(msg);
        }
        if ph.node_id != authenticated_id {
            return Err("hello does not match the authenticated node id".into());
        }
        if ph.node_id == self.cfg.node_id {
            let msg = format!("both nodes are called '{}': give each node its own ha.node_id", ph.node_id);
            lock(&self.dynamic).sync_error = Some(msg.clone());
            return Err(msg);
        }
        Ok(())
    }

    /// Dial the peer, exchange hellos, and follow it if it is the primary.
    /// Returns when the link ends. A primary uses this only to probe.
    async fn connect_once(self: &Arc<Self>) -> Result<()> {
        let link = self
            .security
            .dial(&self.cfg.peer_address, self.cfg.heartbeat.max(Duration::from_secs(1)))
            .await?;
        let mut peer = link.into_peer(self.write_timeout());
        peer.send(&Msg::Hello(self.hello().await)).await?;
        let ph = match timeout(HELLO_TIMEOUT, peer.recv()).await {
            Ok(Some(Ok(Msg::Hello(h)))) => h,
            Ok(Some(Ok(Msg::Reject { reason }))) => {
                lock(&self.dynamic).sync_error = Some(format!("peer refused the link: {reason}"));
                bail!("peer refused the link: {reason}");
            }
            _ => bail!("peer sent no hello"),
        };
        if let Err(reason) = self.check_peer(&ph, &peer.node_id) {
            bail!(reason);
        }
        self.observe_peer(&ph).await;
        if self.role() == HaRole::Standby && ph.role == WireRole::Primary {
            self.follow_primary(peer, ph).await
        } else {
            Ok(())
        }
    }

    async fn run_connector(self: Arc<Self>) {
        let probe_every = (self.cfg.heartbeat * 2).max(Duration::from_secs(2));
        let mut quiet_since: Option<Instant> = None;
        loop {
            if self.role() == HaRole::Primary {
                // Nobody dials a primary that has its standby; probe only when
                // none is attached, to find a primary that outranks this node.
                sleep(probe_every).await;
                if lock(&self.primary_cmd).is_none() {
                    let _ = self.connect_once().await;
                }
                continue;
            }
            match self.connect_once().await {
                Ok(()) => quiet_since = None,
                Err(e) => {
                    let since = *quiet_since.get_or_insert_with(Instant::now);
                    if since.elapsed() < Duration::from_secs(1) {
                        tracing::info!("HA: peer {} not reachable: {e:#}", self.cfg.peer_address);
                    } else {
                        tracing::debug!("HA: peer {} not reachable: {e:#}", self.cfg.peer_address);
                    }
                    let mut d = lock(&self.dynamic);
                    d.session_up = false;
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    // ── what a hello teaches ──

    pub(super) async fn observe_peer(self: &Arc<Self>, ph: &Hello) {
        let role = HaRole::from_wire(ph.role);
        {
            let mut d = lock(&self.dynamic);
            d.peer_id = Some(ph.node_id.clone());
            d.peer_role = Some(role);
            d.peer_epoch = Some(ph.epoch);
            d.peer_seen = Some(Instant::now());
            d.peer_lost_since = None;
            if role == HaRole::Primary {
                d.primary_contact = Some(Instant::now());
            }
        }
        let (mine, my_epoch) = (self.role(), self.epoch());
        match mine {
            HaRole::Primary | HaRole::Fenced if ph.epoch > my_epoch => {
                self.demote(ph.epoch, &format!("{} is at epoch {} (this node: {my_epoch})", ph.node_id, ph.epoch))
                    .await;
            }
            HaRole::Primary if role == HaRole::Primary && ph.epoch == my_epoch => {
                // Two primaries in one term: the lower node id keeps the role.
                if ph.node_id < self.cfg.node_id {
                    self.demote(ph.epoch, &format!("{} is also primary at epoch {} and ranks first", ph.node_id, ph.epoch))
                        .await;
                }
            }
            HaRole::Fenced if role == HaRole::Primary => {
                self.demote(ph.epoch.max(my_epoch), &format!("{} is primary", ph.node_id)).await;
            }
            _ => {}
        }
    }

    // ── transitions ──

    /// Make this node passive and a follower. `epoch` is the term of the node it follows.
    pub(super) async fn demote(self: &Arc<Self>, epoch: u64, reason: &str) {
        let _g = self.transition.lock().await;
        let was = self.role();
        if was == HaRole::Standby {
            let _ = self.adopt_epoch(epoch).await;
            return;
        }
        tracing::warn!("HA: stepping down ({reason}); epoch {epoch}");
        self.go_passive().await;
        self.set_role(HaRole::Standby);
        self.epoch.fetch_max(epoch, Ordering::SeqCst);
        let epoch = self.epoch();
        // Whatever this node wrote since it lost its lease may not exist on the
        // new primary: it must take a full sync before it follows.
        if let Err(e) = self
            .update_record(move |r| {
                r.role = StoredRole::Standby;
                r.epoch = epoch;
                r.applied_seq = 0;
                r.needs_full_sync = true;
            })
            .await
        {
            tracing::error!("HA: cannot record the demotion: {e:#}");
        }
        let _ = self.storage.with_conn(|c| store::clear_log(c)).await;
        *lock(&self.lease_until) = None;
        self.audit("ha_demoted", reason).await;
    }

    /// Stop acting: no writes, no device owner services. Pollers stay only when
    /// the standby is configured to read.
    async fn go_passive(&self) {
        voltrus::node_role::set(false, self.cfg.standby_polling);
        if !self.cfg.standby_polling {
            self.pollers.release_all().await;
        }
        crate::rt().pause_active_services().await;
    }

    /// Start acting: device owner services on.
    async fn go_active(self: &Arc<Self>) {
        voltrus::node_role::set(true, false);
        let running: HashSet<String> = lock(&self.dynamic).peer_running.iter().cloned().collect();
        match self.pollers.start_configured(&running).await {
            Ok(n) => tracing::info!("HA: started {n} device poller(s)"),
            Err(e) => tracing::error!("HA: failed to start device pollers: {e:#}"),
        }
        crate::rt().apply_license_effects().await;
    }

    pub(super) async fn fence(self: &Arc<Self>, reason: &str) {
        let _g = self.transition.lock().await;
        if self.role() != HaRole::Primary {
            return;
        }
        tracing::error!("HA: FENCED, this node stops acting: {reason}");
        self.go_passive().await;
        self.set_role(HaRole::Fenced);
        self.audit("ha_fenced", reason).await;
    }

    /// Step out of the active role for a controlled hand-over. The node is held
    /// as `Fenced`: passive, but not following anyone yet.
    pub(super) async fn go_passive_for_handoff(&self) {
        self.go_passive().await;
        self.set_role(HaRole::Fenced);
    }

    /// The standby confirmed it is primary in `epoch` and has every change
    /// through `final_seq`: follow it from exactly there.
    pub(super) async fn finish_handoff(self: &Arc<Self>, epoch: u64, final_seq: u64) -> Result<()> {
        let _g = self.transition.lock().await;
        self.epoch.fetch_max(epoch, Ordering::SeqCst);
        let epoch = self.epoch();
        self.update_record(move |r| {
            r.role = StoredRole::Standby;
            r.epoch = epoch;
            r.applied_seq = final_seq;
            r.needs_full_sync = false;
        })
        .await?;
        self.storage.with_conn(|c| store::clear_log(c)).await?;
        *lock(&self.lease_until) = None;
        self.set_role(HaRole::Standby);
        {
            let mut d = lock(&self.dynamic);
            d.sync = SyncState::Waiting;
            d.last_seq = Some(final_seq);
            d.primary_contact = Some(Instant::now());
        }
        tracing::warn!("HA: handed the primary role over at epoch {epoch}; now following");
        self.audit("ha_handed_over", &format!("epoch {epoch}")).await;
        Ok(())
    }

    async fn unfence(self: &Arc<Self>, why: &str) {
        let _g = self.transition.lock().await;
        if self.role() != HaRole::Fenced {
            return;
        }
        tracing::warn!("HA: no longer fenced ({why}); resuming as primary");
        self.set_role(HaRole::Primary);
        self.go_active().await;
        self.audit("ha_unfenced", why).await;
    }

    /// A controlled hand-over was refused or failed before the peer took over:
    /// resume as the primary.
    pub(super) async fn resume_primary(self: &Arc<Self>) -> Result<()> {
        let _g = self.transition.lock().await;
        if self.witness.is_arbiter() {
            match self.witness.acquire(&self.cfg.node_id, self.epoch(), self.lease_ttl()).await? {
                Reply::Granted { epoch } => {
                    drop(_g);
                    self.adopt_epoch(epoch).await?;
                    let _g = self.transition.lock().await;
                    *lock(&self.lease_until) = Some(Instant::now() + self.lease_ttl() * 3 / 4);
                    self.set_role(HaRole::Primary);
                    self.go_active().await;
                    return Ok(());
                }
                Reply::Denied { holder, .. } => bail!("the witness lease is held by {holder}"),
            }
        }
        self.set_role(HaRole::Primary);
        self.go_active().await;
        Ok(())
    }

    /// Take the active role. See [`Promote`].
    pub(super) async fn promote(self: &Arc<Self>, why: Promote) -> Result<Transition> {
        let _g = self.transition.lock().await;
        let started = Instant::now();
        if self.role() != HaRole::Standby {
            bail!("this node is not a standby");
        }
        let force = matches!(why, Promote::Operator { force: true });
        let rec = self
            .storage
            .with_conn(|c| store::load_node(c))
            .await?
            .context("no HA record")?;
        if rec.needs_full_sync && !force {
            bail!("this standby has no complete copy of the data yet (a full sync has not finished); promote with force only if that is intended");
        }
        let peer_epoch = lock(&self.dynamic).peer_epoch.unwrap_or(0);
        let known_epoch = self.epoch().max(peer_epoch);

        let new_epoch = match &self.witness {
            Witness::Arbiter { .. } => match self
                .witness
                .acquire(&self.cfg.node_id, known_epoch, self.lease_ttl())
                .await
            {
                Ok(Reply::Granted { epoch }) => {
                    *lock(&self.lease_until) = Some(Instant::now() + self.lease_ttl() * 3 / 4);
                    epoch
                }
                Ok(Reply::Denied { holder, remaining, .. }) => {
                    let msg = format!(
                        "the witness lease is still held by {holder} for {} ms",
                        remaining.as_millis()
                    );
                    lock(&self.dynamic).promotion_blocked = Some(msg.clone());
                    bail!(msg);
                }
                Err(e) => {
                    let msg = format!("the witness is unreachable: {e:#}");
                    lock(&self.dynamic).promotion_blocked = Some(msg.clone());
                    bail!(msg);
                }
            },
            Witness::Reachability { .. } if !force => {
                if !self.witness.reachable().await {
                    let msg = "the witness host is unreachable from this node; it may be the one that is cut off".to_string();
                    lock(&self.dynamic).promotion_blocked = Some(msg.clone());
                    bail!(msg);
                }
                known_epoch + 1
            }
            _ => known_epoch + 1,
        };

        let reason = match &why {
            Promote::Auto(r) => r.clone(),
            Promote::Operator { force } => format!("operator request{}", if *force { " (forced)" } else { "" }),
            Promote::Handoff { .. } => "controlled switchover".to_string(),
        };
        let now = voltrus::timeutil::now_secs();
        let handoff_seq = match &why {
            Promote::Handoff { final_seq } => Some(*final_seq),
            _ => None,
        };
        // Persist the new term before acting on it.
        let reason2 = reason.clone();
        self.storage
            .with_conn(move |c| {
                store::clear_log(c)?;
                if let Some(seq) = handoff_seq {
                    // Number new changes after the old primary's last one, so it can
                    // follow this node from exactly that point without a full sync.
                    store::set_next_seq(c, seq)?;
                }
                let mut rec = store::load_node(c)?.context("no HA record")?;
                rec.role = StoredRole::Primary;
                rec.epoch = new_epoch;
                rec.applied_seq = 0;
                rec.needs_full_sync = false;
                rec.last_failover_at = Some(now);
                rec.last_failover_reason = Some(reason2);
                store::save_node(c, &rec)
            })
            .await?;
        self.epoch.fetch_max(new_epoch, Ordering::SeqCst);
        {
            let mut d = lock(&self.dynamic);
            d.needs_full_sync = false;
            d.failover_at = Some(now);
            d.failover_reason = Some(reason.clone());
            d.promotion_blocked = None;
            d.sync = SyncState::Waiting;
            d.sync_error = None;
        }
        self.set_role(HaRole::Primary);
        self.go_active().await;
        tracing::warn!("HA: promoted to primary at epoch {new_epoch} ({reason})");
        self.audit("ha_promoted", &format!("epoch {new_epoch}: {reason}")).await;
        Ok(Transition {
            role: HaRole::Primary,
            epoch: new_epoch,
            duration_ms: started.elapsed().as_millis() as u64,
        })
    }

    async fn audit(&self, action: &str, details: &str) {
        let _ = self
            .storage
            .log_audit_event(action, &self.cfg.node_id, "ha", Some(details))
            .await;
    }

    // ── the monitor ──

    async fn run_monitor(self: Arc<Self>) {
        let tick = (self.cfg.heartbeat / 4).clamp(Duration::from_millis(50), Duration::from_millis(500));
        let mut interval = tokio::time::interval(tick);
        let mut last_witness = Instant::now() - self.cfg.heartbeat;
        let mut last_housekeeping = Instant::now();
        let mut last_triggers = Instant::now();
        let mut warned_at: Option<Instant> = None;
        loop {
            interval.tick().await;
            match self.role() {
                HaRole::Standby => self.monitor_standby(&mut warned_at).await,
                HaRole::Primary => self.monitor_primary(&mut last_witness).await,
                HaRole::Fenced => self.monitor_fenced(&mut last_witness).await,
            }
            if last_housekeeping.elapsed() >= Duration::from_secs(5) {
                last_housekeeping = Instant::now();
                self.housekeeping(last_triggers.elapsed() >= Duration::from_secs(60)).await;
                if last_triggers.elapsed() >= Duration::from_secs(60) {
                    last_triggers = Instant::now();
                }
            }
        }
    }

    async fn housekeeping(&self, refresh_triggers: bool) {
        let role = self.role();
        let r = self
            .storage
            .with_conn(move |c| {
                if refresh_triggers {
                    store::install_triggers(c)?;
                }
                match role {
                    // The standby's own log holds only its local writes: nothing consumes it.
                    HaRole::Standby => store::clear_log(c),
                    _ => store::trim_log(c, LOG_CAP).map(|_| ()),
                }
            })
            .await;
        if let Err(e) = r {
            tracing::warn!("HA housekeeping failed: {e:#}");
        }
    }

    async fn monitor_standby(self: &Arc<Self>, warned_at: &mut Option<Instant>) {
        let timeout = self.cfg.failover_timeout();
        let silent = {
            let d = lock(&self.dynamic);
            d.primary_contact.unwrap_or(self.started).elapsed()
        };
        if silent < timeout {
            return;
        }
        let reason = format!("no answer from the primary for {} ms", silent.as_millis());
        match self.promote(Promote::Auto(reason)).await {
            Ok(t) => tracing::warn!("HA: failover complete in {} ms", t.duration_ms),
            Err(e) => {
                if warned_at.is_none_or(|t| t.elapsed() > Duration::from_secs(10)) {
                    tracing::warn!("HA: the primary is silent but this node cannot take over: {e:#}");
                    *warned_at = Some(Instant::now());
                }
                let mut d = lock(&self.dynamic);
                d.promotion_blocked.get_or_insert_with(|| format!("{e:#}"));
            }
        }
    }

    fn set_witness(&self, ok: Option<bool>, detail: Option<String>) {
        let mut d = lock(&self.dynamic);
        d.witness_ok = ok;
        d.witness_detail = detail;
    }

    async fn monitor_primary(self: &Arc<Self>, last_witness: &mut Instant) {
        match &self.witness {
            Witness::None => {}
            Witness::Arbiter { .. } => {
                if last_witness.elapsed() < self.cfg.heartbeat {
                    return;
                }
                *last_witness = Instant::now();
                let sent = Instant::now();
                let ttl = self.lease_ttl();
                match self.witness.acquire(&self.cfg.node_id, self.epoch(), ttl).await {
                    Ok(Reply::Granted { epoch }) => {
                        *lock(&self.lease_until) = Some(sent + ttl * 3 / 4);
                        self.set_witness(Some(true), Some(format!("lease held, epoch {epoch}")));
                        if epoch != self.epoch() {
                            let _ = self.adopt_epoch(epoch).await;
                        }
                    }
                    Ok(Reply::Denied { epoch, holder, .. }) => {
                        self.set_witness(Some(true), Some(format!("lease held by {holder}")));
                        self.demote(epoch, &format!("the witness granted the lease to {holder}")).await;
                    }
                    Err(e) => {
                        self.set_witness(Some(false), Some(format!("{e:#}")));
                        let expired = lock(&self.lease_until).is_none_or(|t| Instant::now() >= t);
                        if expired {
                            self.fence("the witness lease could not be renewed in time").await;
                        }
                    }
                }
            }
            Witness::Reachability { .. } => {
                let peer_ok = {
                    let d = lock(&self.dynamic);
                    d.session_up
                        || d.peer_seen.is_some_and(|t| t.elapsed() < self.cfg.failover_timeout())
                };
                if peer_ok {
                    lock(&self.dynamic).peer_lost_since = None;
                    if last_witness.elapsed() >= Duration::from_secs(5) {
                        *last_witness = Instant::now();
                        let ok = self.witness.reachable().await;
                        self.set_witness(Some(ok), None);
                    }
                    return;
                }
                let lost_for = *lock(&self.dynamic).peer_lost_since.get_or_insert_with(Instant::now);
                if last_witness.elapsed() >= self.cfg.heartbeat {
                    *last_witness = Instant::now();
                    let ok = self.witness.reachable().await;
                    self.set_witness(Some(ok), None);
                    if !ok && lost_for.elapsed() >= self.cfg.failover_timeout() {
                        self.fence("the standby and the witness host are both unreachable").await;
                    }
                }
            }
        }
    }

    async fn monitor_fenced(self: &Arc<Self>, last_witness: &mut Instant) {
        // A controlled hand-over holds this role on purpose until it concludes.
        if self.switching.load(Ordering::SeqCst) || last_witness.elapsed() < self.cfg.heartbeat {
            return;
        }
        *last_witness = Instant::now();
        match &self.witness {
            Witness::Arbiter { .. } => {
                match self.witness.acquire(&self.cfg.node_id, self.epoch(), self.lease_ttl()).await {
                    Ok(Reply::Granted { epoch }) => {
                        *lock(&self.lease_until) = Some(Instant::now() + self.lease_ttl() * 3 / 4);
                        let _ = self.adopt_epoch(epoch).await;
                        self.set_witness(Some(true), Some("lease regained".into()));
                        self.unfence("the witness granted the lease again").await;
                    }
                    Ok(Reply::Denied { epoch, holder, .. }) => {
                        self.demote(epoch, &format!("the witness granted the lease to {holder}")).await;
                    }
                    Err(e) => self.set_witness(Some(false), Some(format!("{e:#}"))),
                }
            }
            Witness::Reachability { .. } => {
                let ok = self.witness.reachable().await;
                self.set_witness(Some(ok), None);
                let peer_ok = lock(&self.dynamic)
                    .peer_seen
                    .is_some_and(|t| t.elapsed() < self.cfg.failover_timeout());
                if ok || peer_ok {
                    self.unfence("the witness or the peer is reachable again").await;
                }
            }
            Witness::None => self.unfence("no witness is configured").await,
        }
    }

    // ── the status ──

    pub async fn status(&self) -> HaStatus {
        let d = lock(&self.dynamic);
        let role = self.role();
        let peer_status = match (d.session_up, d.peer_seen) {
            (true, _) => "connected",
            (false, Some(t)) if t.elapsed() < self.cfg.failover_timeout() => "degraded",
            _ => "disconnected",
        };
        let sync_state = if role == HaRole::Standby
            && d.needs_full_sync
            && !matches!(d.sync, SyncState::FullSync { .. } | SyncState::Error(_))
        {
            SyncState::Unsynced
        } else {
            d.sync.clone()
        };
        let sync_error = match (&sync_state, &d.sync_error) {
            (SyncState::Error(e), _) => Some(e.clone()),
            (_, e) => e.clone(),
        };
        HaStatus {
            sync_state: sync_state.name().to_string(),
            sync_progress: sync_state.progress(),
            enabled: true,
            role,
            active: role == HaRole::Primary,
            node_id: self.cfg.node_id.clone(),
            epoch: self.epoch(),
            peer_id: d.peer_id.clone(),
            peer_role: d.peer_role,
            peer_epoch: d.peer_epoch,
            peer_status: peer_status.to_string(),
            sync_lag_ms: d.lag_ms,
            pending_changes: d.pending,
            last_sync_seq: d.last_seq,
            last_sync_at: d.last_sync_at.and_then(super::rfc3339),
            sync_error,
            failover_ts: d.failover_at.and_then(super::rfc3339),
            failover_reason: d.failover_reason.clone(),
            uptime_as_role_secs: d.role_since.map_or(0, |t| t.elapsed().as_secs()),
            failover_after_ms: self.cfg.failover_timeout().as_millis() as u64,
            can_take_over: role == HaRole::Standby && !d.needs_full_sync,
            promotion_blocked: d.promotion_blocked.clone(),
            witness: self.witness.kind_name().map(|k| WitnessStatus {
                kind: k.to_string(),
                reachable: d.witness_ok,
                detail: d.witness_detail.clone(),
            }),
            last_full_sync: d.last_full_sync.clone(),
            link_auth_failures: super::link::auth_failures(),
        }
    }

    // ── operator actions ──

    /// Run a full sync now. On the primary the standby is resynced from a fresh
    /// snapshot; on the standby the primary is asked to do so.
    pub async fn force_sync(&self, wait: Duration) -> Result<Option<FullSyncReport>> {
        let mut sync_rx = self.sync_tx.subscribe();
        sync_rx.mark_unchanged();
        match self.role() {
            HaRole::Primary => {
                let tx = lock(&self.primary_cmd).clone().context("no standby is connected")?;
                let (reply_tx, reply_rx) = oneshot::channel();
                tx.send(PrimaryCmd::Resync(reply_tx)).await.context("the standby session just ended")?;
                if let Ok(r) = timeout(wait, reply_rx).await {
                    r.context("the session ended before the sync finished")??;
                }
            }
            HaRole::Standby => {
                let tx = lock(&self.standby_cmd).clone().context("not connected to a primary")?;
                tx.send(StandbyCmd::Resync).await.context("the primary session just ended")?;
                let _ = timeout(wait, sync_rx.changed()).await;
            }
            HaRole::Fenced => bail!("this node is fenced and has no peer to sync with"),
        }
        Ok(lock(&self.dynamic).last_full_sync.clone())
    }

    /// Switch the roles of the pair. Returns once the other node is primary.
    pub async fn force_failover(self: &Arc<Self>, force: bool, wait: Duration) -> Result<Transition> {
        let started = Instant::now();
        match self.role() {
            HaRole::Primary => {
                let tx = lock(&self.primary_cmd)
                    .clone()
                    .context("no standby is connected: there is nothing to hand over to")?;
                let (reply_tx, reply_rx) = oneshot::channel();
                tx.send(PrimaryCmd::Switchover(reply_tx))
                    .await
                    .context("the standby session just ended")?;
                let report = timeout(wait, reply_rx)
                    .await
                    .context("the switchover is still running")?
                    .context("the session ended during the switchover")??;
                Ok(Transition {
                    role: HaRole::Standby,
                    epoch: report.epoch,
                    duration_ms: started.elapsed().as_millis() as u64,
                })
            }
            HaRole::Standby => {
                let linked = lock(&self.standby_cmd).clone();
                if let (Some(tx), false) = (linked, force) {
                    // The primary is there: ask it to hand over cleanly.
                    let mut role_rx = self.role_tx.subscribe();
                    role_rx.mark_unchanged();
                    tx.send(StandbyCmd::Switchover).await.context("the primary session just ended")?;
                    let deadline = Instant::now() + wait;
                    while self.role() != HaRole::Primary {
                        let left = deadline.saturating_duration_since(Instant::now());
                        if left.is_zero() || timeout(left, role_rx.changed()).await.is_err() {
                            bail!("the primary did not hand over in time");
                        }
                    }
                    Ok(Transition {
                        role: HaRole::Primary,
                        epoch: self.epoch(),
                        duration_ms: started.elapsed().as_millis() as u64,
                    })
                } else {
                    tracing::warn!("HA: operator takeover requested");
                    self.promote(Promote::Operator { force }).await
                }
            }
            HaRole::Fenced => bail!("this node is fenced; it resumes by itself once the witness answers"),
        }
    }

    /// Sorted map of the replicated tables, for tests and diagnostics.
    pub async fn replicated_tables(&self) -> Result<BTreeMap<String, usize>> {
        self.storage
            .with_conn(|c| Ok(store::table_schemas(c)?.into_iter().map(|(k, v)| (k, v.cols.len())).collect()))
            .await
    }
}

/// Whether two hellos describe the same lineage (used by the primary to choose
/// between an incremental and a full sync).
pub(super) fn can_resume(ph: &Hello, my_epoch: u64, floor: u64, max_seq: u64) -> bool {
    !ph.needs_full_sync && ph.epoch == my_epoch && ph.applied_seq >= floor && ph.applied_seq <= max_seq
}

#[allow(dead_code)]
fn _peer_is_used(_: &Peer) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(epoch: u64, applied: u64, needs: bool) -> Hello {
        Hello {
            proto: PROTOCOL_VERSION,
            node_id: "b".into(),
            role: WireRole::Standby,
            epoch,
            applied_seq: applied,
            needs_full_sync: needs,
            max_seq: 0,
            version: "x".into(),
            now_ms: 0,
        }
    }

    #[test]
    fn a_standby_resumes_only_inside_the_same_term_and_the_retained_log() {
        assert!(can_resume(&hello(3, 50, false), 3, 40, 80));
        assert!(can_resume(&hello(3, 40, false), 3, 40, 80), "exactly at the floor is fine");
        assert!(can_resume(&hello(3, 80, false), 3, 40, 80), "fully caught up");
        assert!(!can_resume(&hello(3, 39, false), 3, 40, 80), "behind the trimmed log");
        assert!(!can_resume(&hello(3, 81, false), 3, 40, 80), "ahead of the primary: a different history");
        assert!(!can_resume(&hello(2, 50, false), 3, 40, 80), "an older term");
        assert!(!can_resume(&hello(4, 50, false), 3, 40, 80), "a newer term");
        assert!(!can_resume(&hello(3, 50, true), 3, 40, 80), "marked for a full sync");
    }
}
