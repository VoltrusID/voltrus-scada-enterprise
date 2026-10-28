//! The witness: a third party that keeps a network partition from leaving two
//! active nodes.
//!
//! Two nodes that lose sight of each other cannot tell "the other node died"
//! from "the link between us broke". Without a third opinion the standby has to
//! guess, and a wrong guess puts two nodes on the same devices. Two witnesses
//! are supported:
//!
//! * **Arbiter** (`kind: arbiter`): a tiny process, `voltrus --ha-witness`, on
//!   any third machine. The active node holds a lease from it and renews it with
//!   every heartbeat; a node that cannot renew in time stops acting. The standby
//!   may take over only by acquiring the lease, which the arbiter grants only
//!   after the holder's lease ran out. The arbiter also hands out the epoch, so
//!   terms are strictly ordered. This prevents two active nodes under every
//!   partition: for both to act, the arbiter would have to grant the lease twice.
//! * **Reachability** (`kind: tcp` or `http`): a standby promotes only if it can
//!   reach a host that must be reachable from the real network (the plant
//!   gateway, a PLC); an active node that has lost its peer and the witness
//!   stops. It protects against a node that is itself cut off. It cannot tell a
//!   dead peer from a broken link between two healthy nodes, so that case can
//!   still produce two active nodes until the link heals and the epochs
//!   resolve it. Use an arbiter when that matters.
//!
//! Safety of the lease does not depend on synchronised clocks: the arbiter
//! counts the lease from when it answered, the holder from when it asked, so the
//! holder always stops no later than the arbiter lets another node start.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::time::timeout;
use voltrus::models::HaWitnessConfig;

use super::link::LinkSecurity;
use super::wire::Msg;

/// How long after its own restart the arbiter lets only the recorded holder
/// act, so a restart of the witness never hands the lease to the standby while
/// the holder is alive.
const RESTART_GRACE: Duration = Duration::from_secs(10);

// ── the arbiter's state machine ──

#[derive(Debug, Clone, PartialEq)]
pub struct LeaseDecision {
    pub granted: bool,
    pub epoch: u64,
    pub holder: String,
    pub remaining: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
struct Persisted {
    epoch: u64,
    holder: Option<String>,
}

pub struct Arbiter {
    epoch: u64,
    holder: Option<String>,
    expires: Instant,
}

impl Arbiter {
    pub fn new(now: Instant) -> Self {
        Self {
            epoch: 0,
            holder: None,
            expires: now,
        }
    }

    /// Resume from a persisted term. The recorded holder is protected for the grace period.
    fn restored(p: Persisted, now: Instant) -> Self {
        Self {
            epoch: p.epoch,
            holder: p.holder,
            expires: now + RESTART_GRACE,
        }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn holder(&self) -> Option<&str> {
        self.holder.as_deref()
    }

    /// `node` asks for the lease at `epoch` (the term it holds or has seen).
    pub fn acquire(
        &mut self,
        now: Instant,
        node: &str,
        epoch: u64,
        ttl: Duration,
    ) -> LeaseDecision {
        let remaining = self.expires.saturating_duration_since(now);
        match self.holder.as_deref() {
            Some(h) if h == node => {
                if epoch < self.epoch {
                    // A node from an older term: it was superseded.
                    return self.denied(remaining);
                }
                // The arbiter forgot a term the holder remembers: adopt it.
                self.epoch = self.epoch.max(epoch);
                self.expires = now + ttl;
                self.granted(ttl)
            }
            Some(_) if remaining > Duration::ZERO => self.denied(remaining),
            _ => {
                // Free, or the holder's lease ran out: a new term starts.
                self.epoch = self.epoch.max(epoch) + 1;
                self.holder = Some(node.to_string());
                self.expires = now + ttl;
                self.granted(ttl)
            }
        }
    }

    /// The holder gives the lease up (a controlled hand-over).
    pub fn release(&mut self, now: Instant, node: &str, epoch: u64) {
        if self.holder.as_deref() == Some(node) && epoch >= self.epoch {
            self.expires = now;
        }
    }

    fn granted(&self, ttl: Duration) -> LeaseDecision {
        LeaseDecision {
            granted: true,
            epoch: self.epoch,
            holder: self.holder.clone().unwrap_or_default(),
            remaining: ttl,
        }
    }

    fn denied(&self, remaining: Duration) -> LeaseDecision {
        LeaseDecision {
            granted: false,
            epoch: self.epoch,
            holder: self.holder.clone().unwrap_or_default(),
            remaining,
        }
    }

    fn persisted(&self) -> Persisted {
        Persisted {
            epoch: self.epoch,
            holder: self.holder.clone(),
        }
    }
}

fn load_state(path: &Path, now: Instant) -> Arbiter {
    match std::fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<Persisted>(&text) {
            Ok(p) => Arbiter::restored(p, now),
            Err(e) => {
                tracing::warn!(
                    "HA witness: ignoring unreadable state {}: {e}",
                    path.display()
                );
                Arbiter::new(now)
            }
        },
        Err(_) => Arbiter::new(now),
    }
}

fn save_state(path: &Path, a: &Arbiter) {
    let tmp = path.with_extension("tmp");
    let write = serde_json::to_vec(&a.persisted())
        .map_err(anyhow::Error::from)
        .and_then(|b| std::fs::write(&tmp, b).map_err(anyhow::Error::from))
        .and_then(|_| std::fs::rename(&tmp, path).map_err(anyhow::Error::from));
    if let Err(e) = write {
        tracing::error!("HA witness: cannot save state to {}: {e:#}", path.display());
    }
}

/// Run the arbiter until the process ends: `voltrus --ha-witness`.
pub async fn run_arbiter(bind: &str, secret: &str, state_path: PathBuf) -> Result<()> {
    let listener = TcpListener::bind(bind)
        .await
        .with_context(|| format!("cannot listen on {bind}"))?;
    run_arbiter_on(listener, secret, state_path).await
}

/// The arbiter on an already bound listener.
pub async fn run_arbiter_on(
    listener: TcpListener,
    secret: &str,
    state_path: PathBuf,
) -> Result<()> {
    let security = LinkSecurity::new("ha-witness", secret, None)?;
    let state = Arc::new(Mutex::new(load_state(&state_path, Instant::now())));
    tracing::info!(
        "HA witness listening on {} (epoch {})",
        listener.local_addr()?,
        state.lock().unwrap_or_else(|e| e.into_inner()).epoch()
    );
    loop {
        let (tcp, addr) = listener.accept().await?;
        let (security, state, state_path) = (security.clone(), state.clone(), state_path.clone());
        tokio::spawn(async move {
            if let Err(e) = serve_one(security, tcp, state, &state_path).await {
                tracing::warn!("HA witness: connection from {addr}: {e:#}");
            }
        });
    }
}

async fn serve_one(
    security: LinkSecurity,
    tcp: tokio::net::TcpStream,
    state: Arc<Mutex<Arbiter>>,
    state_path: &Path,
) -> Result<()> {
    let mut peer = security
        .accept(tcp)
        .await?
        .into_peer(Duration::from_secs(5));
    while let Ok(Some(msg)) = timeout(Duration::from_secs(30), peer.recv()).await {
        let reply = match msg? {
            Msg::LeaseAcquire {
                node_id,
                epoch,
                ttl_ms,
            } => {
                // A lease longer than a minute would defeat the failover time.
                let ttl = Duration::from_millis(ttl_ms.clamp(500, 60_000));
                let (decision, changed) = {
                    let mut a = state.lock().unwrap_or_else(|e| e.into_inner());
                    let before = (a.epoch(), a.holder().map(str::to_owned));
                    let d = a.acquire(Instant::now(), &node_id, epoch, ttl);
                    let changed = before != (a.epoch(), a.holder().map(str::to_owned));
                    if changed {
                        save_state(state_path, &a);
                        tracing::info!(
                            "HA witness: epoch {} held by {} (asked by {node_id})",
                            a.epoch(),
                            a.holder().unwrap_or("nobody")
                        );
                    }
                    (d, changed)
                };
                let _ = changed;
                Msg::LeaseReply {
                    granted: decision.granted,
                    epoch: decision.epoch,
                    holder: decision.holder,
                    remaining_ms: decision.remaining.as_millis() as u64,
                }
            }
            Msg::LeaseRelease { node_id, epoch } => {
                let mut a = state.lock().unwrap_or_else(|e| e.into_inner());
                a.release(Instant::now(), &node_id, epoch);
                Msg::LeaseReply {
                    granted: false,
                    epoch: a.epoch(),
                    holder: a.holder().unwrap_or_default().to_string(),
                    remaining_ms: 0,
                }
            }
            other => bail!("unexpected message from a node: {other:?}"),
        };
        peer.send(&reply).await?;
    }
    Ok(())
}

// ── the command line ──

pub const HELP: &str = "\
HOT STANDBY WITNESS:
    voltrus --ha-witness [--bind ADDR] [--state FILE] [--secret-file FILE]
                                       Run the lease arbiter a hot standby pair uses to avoid
                                       two active nodes. Runs on a third machine. ADDR defaults
                                       to 0.0.0.0:7902, FILE to ha-witness.json. The secret is
                                       VOLTRUS_HA_SECRET or the first line of --secret-file and
                                       must equal the pair's ha.shared_secret.
";

/// `voltrus --ha-witness ...`. Returns false when the arguments are not for it.
pub async fn handle_cli(args: &[String]) -> bool {
    if args.get(1).map(String::as_str) != Some("--ha-witness") {
        return false;
    }
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let secret = match flag("--secret-file") {
        Some(path) => std::fs::read_to_string(&path)
            .map(|s| s.lines().next().unwrap_or("").trim().to_string())
            .unwrap_or_else(|e| {
                eprintln!("cannot read {path}: {e}");
                std::process::exit(1);
            }),
        None => std::env::var("VOLTRUS_HA_SECRET").unwrap_or_default(),
    };
    let bind = flag("--bind").unwrap_or_else(|| "0.0.0.0:7902".to_string());
    let state = PathBuf::from(flag("--state").unwrap_or_else(|| "ha-witness.json".to_string()));
    if let Err(e) = run_arbiter(&bind, &secret, state).await {
        eprintln!("ha-witness: {e:#}");
        std::process::exit(1);
    }
    true
}

// ── the node's view of its witness ──

#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    Granted {
        epoch: u64,
    },
    Denied {
        epoch: u64,
        holder: String,
        remaining: Duration,
    },
}

#[derive(Clone)]
pub enum Witness {
    None,
    Arbiter {
        address: String,
        timeout: Duration,
        security: LinkSecurity,
    },
    Reachability {
        kind: Reach,
        address: String,
        timeout: Duration,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    Tcp,
    Http,
}

impl Witness {
    pub fn from_config(cfg: Option<&HaWitnessConfig>, security: &LinkSecurity) -> Result<Self> {
        let Some(cfg) = cfg else {
            return Ok(Witness::None);
        };
        anyhow::ensure!(!cfg.address.is_empty(), "ha.witness.address is required");
        let t = Duration::from_millis(cfg.timeout_ms.clamp(100, 30_000));
        Ok(match cfg.kind.as_str() {
            "arbiter" => Witness::Arbiter {
                address: cfg.address.clone(),
                timeout: t,
                security: security.clone(),
            },
            "tcp" => Witness::Reachability {
                kind: Reach::Tcp,
                address: cfg.address.clone(),
                timeout: t,
            },
            "http" => {
                anyhow::ensure!(
                    cfg.address.starts_with("http://") || cfg.address.starts_with("https://"),
                    "ha.witness.address must be an http(s) URL for kind http"
                );
                Witness::Reachability {
                    kind: Reach::Http,
                    address: cfg.address.clone(),
                    timeout: t,
                }
            }
            other => bail!("ha.witness.kind must be arbiter, tcp or http, not '{other}'"),
        })
    }

    pub fn kind_name(&self) -> Option<&'static str> {
        match self {
            Witness::None => None,
            Witness::Arbiter { .. } => Some("arbiter"),
            Witness::Reachability {
                kind: Reach::Tcp, ..
            } => Some("tcp"),
            Witness::Reachability {
                kind: Reach::Http, ..
            } => Some("http"),
        }
    }

    pub fn is_arbiter(&self) -> bool {
        matches!(self, Witness::Arbiter { .. })
    }

    /// Ask the arbiter for (or renew) the lease. `Err` when it cannot be reached.
    pub async fn acquire(&self, node_id: &str, epoch: u64, ttl: Duration) -> Result<Reply> {
        let Witness::Arbiter {
            address,
            timeout: t,
            security,
        } = self
        else {
            bail!("not an arbiter witness");
        };
        let msg = Msg::LeaseAcquire {
            node_id: node_id.to_string(),
            epoch,
            ttl_ms: ttl.as_millis() as u64,
        };
        Self::exchange(security, address, *t, &msg).await
    }

    /// Give the lease up before a controlled hand-over.
    pub async fn release(&self, node_id: &str, epoch: u64) -> Result<()> {
        let Witness::Arbiter {
            address,
            timeout: t,
            security,
        } = self
        else {
            return Ok(());
        };
        let msg = Msg::LeaseRelease {
            node_id: node_id.to_string(),
            epoch,
        };
        Self::exchange(security, address, *t, &msg)
            .await
            .map(|_| ())
    }

    async fn exchange(
        security: &LinkSecurity,
        address: &str,
        t: Duration,
        msg: &Msg,
    ) -> Result<Reply> {
        let attempt = async {
            let mut peer = security.dial(address, t).await?.into_peer(t);
            peer.send(msg).await?;
            match peer.recv().await {
                Some(Ok(Msg::LeaseReply {
                    granted,
                    epoch,
                    holder,
                    remaining_ms,
                })) => Ok(if granted {
                    Reply::Granted { epoch }
                } else {
                    Reply::Denied {
                        epoch,
                        holder,
                        remaining: Duration::from_millis(remaining_ms),
                    }
                }),
                Some(Ok(other)) => bail!("unexpected answer from the witness: {other:?}"),
                Some(Err(e)) => Err(e.into()),
                None => bail!("the witness closed the connection"),
            }
        };
        timeout(t * 2, attempt)
            .await
            .context("the witness did not answer in time")?
    }

    /// Whether the witness host answers (reachability kinds). Arbiters are
    /// judged by [`Witness::acquire`], not by this.
    pub async fn reachable(&self) -> bool {
        match self {
            Witness::Reachability {
                kind: Reach::Tcp,
                address,
                timeout: t,
            } => {
                matches!(
                    timeout(*t, tokio::net::TcpStream::connect(address)).await,
                    Ok(Ok(_))
                )
            }
            Witness::Reachability {
                kind: Reach::Http,
                address,
                timeout: t,
            } => {
                let client = match reqwest::Client::builder().timeout(*t).build() {
                    Ok(c) => c,
                    Err(_) => return false,
                };
                // Any HTTP answer proves the path to the host; the status does not matter.
                client.get(address).send().await.is_ok()
            }
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn first_holder_gets_epoch_one_and_a_second_node_is_refused_while_the_lease_lives() {
        let t0 = Instant::now();
        let mut a = Arbiter::new(t0);
        let d = a.acquire(t0, "a", 0, s(3));
        assert!(d.granted);
        assert_eq!(d.epoch, 1);
        let d = a.acquire(t0 + s(1), "b", 0, s(3));
        assert!(!d.granted);
        assert_eq!((d.holder.as_str(), d.epoch), ("a", 1));
        assert_eq!(d.remaining, s(2));
    }

    #[test]
    fn renewing_extends_the_lease_without_a_new_term() {
        let t0 = Instant::now();
        let mut a = Arbiter::new(t0);
        a.acquire(t0, "a", 0, s(3));
        for i in 1..10 {
            let d = a.acquire(t0 + s(i), "a", 1, s(3));
            assert!(d.granted && d.epoch == 1);
        }
        // Still held by a at t0 + 11: renewed at t0 + 9 for 3 s.
        assert!(!a.acquire(t0 + s(11), "b", 0, s(3)).granted);
    }

    #[test]
    fn the_standby_gets_the_next_term_once_the_lease_ran_out_and_the_old_holder_is_then_refused() {
        let t0 = Instant::now();
        let mut a = Arbiter::new(t0);
        a.acquire(t0, "a", 0, s(3));
        let d = a.acquire(t0 + s(4), "b", 1, s(3));
        assert!(d.granted);
        assert_eq!(d.epoch, 2, "a new holder starts a new term");
        // The old primary comes back and still believes it is in term 1.
        let d = a.acquire(t0 + s(5), "a", 1, s(3));
        assert!(!d.granted, "a superseded node can never renew");
        assert_eq!((d.holder.as_str(), d.epoch), ("b", 2));
    }

    #[test]
    fn release_lets_the_peer_take_over_at_once_and_the_epoch_never_goes_back() {
        let t0 = Instant::now();
        let mut a = Arbiter::new(t0);
        a.acquire(t0, "a", 5, s(30));
        assert_eq!(
            a.epoch(),
            6,
            "the epoch follows the highest the caller has seen"
        );
        a.release(t0 + s(1), "a", 6);
        let d = a.acquire(t0 + s(1), "b", 6, s(30));
        assert!(d.granted && d.epoch == 7);
        // A stale release from the previous holder changes nothing.
        a.release(t0 + s(2), "a", 6);
        assert!(!a.acquire(t0 + s(3), "a", 6, s(30)).granted);
    }

    #[test]
    fn a_node_that_remembers_a_newer_term_than_a_restarted_arbiter_is_believed() {
        let t0 = Instant::now();
        let mut a = Arbiter::new(t0);
        a.acquire(t0, "a", 0, s(3));
        // The arbiter lost its state and restarted blank; the same holder returns.
        let mut a2 = Arbiter::new(t0 + s(1));
        let d = a2.acquire(t0 + s(1), "a", 9, s(3));
        assert!(d.granted);
        assert_eq!(
            d.epoch, 10,
            "no term below one the node has seen is ever granted"
        );
    }

    #[test]
    fn a_restarted_arbiter_protects_the_recorded_holder_for_a_grace_period() {
        let t0 = Instant::now();
        let mut a = Arbiter::restored(
            Persisted {
                epoch: 4,
                holder: Some("a".into()),
            },
            t0,
        );
        assert!(!a.acquire(t0 + s(1), "b", 4, s(3)).granted);
        assert!(a.acquire(t0 + s(2), "a", 4, s(3)).granted);
        let mut a = Arbiter::restored(
            Persisted {
                epoch: 4,
                holder: Some("a".into()),
            },
            t0,
        );
        assert!(a.acquire(t0 + RESTART_GRACE + s(1), "b", 4, s(3)).granted);
    }

    #[tokio::test]
    async fn the_arbiter_over_the_wire_grants_denies_and_persists() {
        let secret = "0123456789abcdef0123456789abcdef0123456789abcdef";
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("witness.json");
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let task = tokio::spawn(run_arbiter_on(listener, secret, state.clone()));

        let witness = |node: &str| {
            let sec = LinkSecurity::new(node, secret, None).unwrap();
            Witness::from_config(
                Some(&HaWitnessConfig {
                    kind: "arbiter".into(),
                    address: addr.clone(),
                    timeout_ms: 2000,
                }),
                &sec,
            )
            .unwrap()
        };
        let (a, b) = (witness("a"), witness("b"));

        assert_eq!(
            a.acquire("a", 0, s(5)).await.unwrap(),
            Reply::Granted { epoch: 1 }
        );
        match b.acquire("b", 0, s(5)).await.unwrap() {
            Reply::Denied {
                epoch: 1, holder, ..
            } => assert_eq!(holder, "a"),
            other => panic!("{other:?}"),
        }
        a.release("a", 1).await.unwrap();
        assert_eq!(
            b.acquire("b", 1, s(5)).await.unwrap(),
            Reply::Granted { epoch: 2 }
        );
        assert!(matches!(
            a.acquire("a", 1, s(5)).await.unwrap(),
            Reply::Denied { epoch: 2, .. }
        ));

        let saved: Persisted = serde_json::from_slice(&std::fs::read(&state).unwrap()).unwrap();
        assert_eq!((saved.epoch, saved.holder.as_deref()), (2, Some("b")));

        // A node with the wrong secret cannot reach the arbiter at all.
        let intruder = LinkSecurity::new("a", &secret.replace('0', "7"), None).unwrap();
        let w = Witness::from_config(
            Some(&HaWitnessConfig {
                kind: "arbiter".into(),
                address: addr.clone(),
                timeout_ms: 2000,
            }),
            &intruder,
        )
        .unwrap();
        assert!(w.acquire("a", 0, s(5)).await.is_err());
        task.abort();
    }

    #[tokio::test]
    async fn reachability_witnesses_report_what_they_can_connect_to() {
        let sec = LinkSecurity::new(
            "a",
            "0123456789abcdef0123456789abcdef0123456789abcdef",
            None,
        )
        .unwrap();
        let open = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let open_addr = open.local_addr().unwrap().to_string();
        let tcp = |addr: &str| {
            Witness::from_config(
                Some(&HaWitnessConfig {
                    kind: "tcp".into(),
                    address: addr.into(),
                    timeout_ms: 300,
                }),
                &sec,
            )
            .unwrap()
        };
        assert!(tcp(&open_addr).reachable().await);
        let closed = {
            let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
            l.local_addr().unwrap().to_string()
        };
        assert!(!tcp(&closed).reachable().await);
        assert!(
            Witness::from_config(
                Some(&HaWitnessConfig {
                    kind: "http".into(),
                    address: "no-scheme".into(),
                    timeout_ms: 300
                }),
                &sec
            )
            .is_err()
        );
    }
}
