//! Deciding the role of this node before any device poller starts.
//!
//! The core calls [`resolve_node_role`] right after the licence is resolved. The
//! decision must be made here, not later, because a node that starts as the
//! wrong role would poll devices for the seconds it takes to find out.
//!
//! The recorded role (in the database) rules after the first start; `ha.role`
//! only chooses the role of a node that has no record yet. A node that was
//! primary does not simply resume: it first asks its peer and its witness, so a
//! restarted old primary never comes up beside the primary that replaced it.

use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::time::timeout;
use voltrus::license::LicenseStatus;
use voltrus::models::AppConfig;
use voltrus::node_role::Start;
use voltrus::storage::Storage;

use super::link::LinkSecurity;
use super::node::Boot;
use super::settings::{HaSettings, InitialRole};
use super::store::{self, NodeRecord, StoredRole};
use super::wire::{Hello, Msg, PROTOCOL_VERSION, WireRole};
use super::witness::{Reply, Witness};
use super::HaRole;

/// Paths a passive node still accepts writes on: sign-in, the pair's own
/// controls, and the machine-bound licence activation.
const PASSIVE_WRITE_EXEMPTIONS: &[&str] = &["/api/v1/auth/", "/api/v1/ha/", "/api/license/"];

pub(super) struct BootInfo {
    pub settings: HaSettings,
    pub security: LinkSecurity,
    pub boot: Boot,
    pub record: NodeRecord,
}

static BOOT: Mutex<Option<BootInfo>> = Mutex::new(None);

pub(super) fn take() -> Option<BootInfo> {
    BOOT.lock().unwrap_or_else(|e| e.into_inner()).take()
}

fn hello(node_id: &str, role: WireRole, epoch: u64) -> Hello {
    Hello {
        proto: PROTOCOL_VERSION,
        node_id: node_id.to_string(),
        role,
        epoch,
        applied_seq: 0,
        needs_full_sync: false,
        max_seq: 0,
        version: env!("CARGO_PKG_VERSION").to_string(),
        now_ms: voltrus::timeutil::now_millis(),
    }
}

/// Ask the peer who it is and what it holds. `None` when it cannot be reached.
async fn probe_peer(security: &LinkSecurity, settings: &HaSettings, mine: Hello) -> Result<Option<Hello>> {
    let wait = settings.failover_timeout().max(Duration::from_secs(2));
    let attempt = async {
        let link = security.dial(&settings.peer_address, wait).await?;
        let mut peer = link.into_peer(wait);
        peer.send(&Msg::Hello(mine)).await?;
        match peer.recv().await {
            Some(Ok(Msg::Hello(h))) => Ok::<_, anyhow::Error>(h),
            Some(Ok(Msg::Reject { reason })) => bail!("peer refused: {reason}"),
            _ => bail!("peer sent no hello"),
        }
    };
    match timeout(wait, attempt).await {
        Ok(Ok(h)) => {
            if h.node_id == settings.node_id {
                bail!("both nodes are called '{}': give each node its own ha.node_id", h.node_id);
            }
            Ok(Some(h))
        }
        _ => Ok(None),
    }
}

/// The core's `Extension::resolve_node_role`.
pub async fn resolve_node_role(
    config: &AppConfig,
    storage: &Storage,
    license: &LicenseStatus,
) -> Result<Start> {
    if !config.ha.enabled {
        // Capture triggers without a consumer would fill the database: remove them.
        storage
            .with_conn(|c| {
                if store::has_triggers(c)? {
                    store::teardown(c)?;
                }
                Ok(())
            })
            .await?;
        return Ok(Start::default());
    }
    if !license.tier().ha() {
        return unlicensed(config, storage).await;
    }

    let settings = HaSettings::from_config(&config.ha).context("invalid ha configuration")?;
    let security = LinkSecurity::new(
        &settings.node_id,
        &settings.secret,
        settings.tls_files.as_ref().map(|(c, k)| (c.as_str(), k.as_str())),
    )?;
    voltrus::node_role::set_passive_write_exemptions(PASSIVE_WRITE_EXEMPTIONS);

    let record = storage
        .with_conn(|c| {
            store::init_schema(c)?;
            store::install_triggers(c)?;
            store::load_node(c)
        })
        .await?;

    let mut rec = match record {
        Some(mut r) => {
            r.node_id = settings.node_id.clone();
            r
        }
        None => {
            let (role, epoch, needs_full_sync) = match settings.initial_role {
                InitialRole::Primary => (StoredRole::Primary, 1, false),
                InitialRole::Standby => (StoredRole::Standby, 0, true),
                InitialRole::Auto => {
                    match probe_peer(&security, &settings, hello(&settings.node_id, WireRole::Fenced, 0)).await? {
                        Some(ph) if ph.role == WireRole::Primary => (StoredRole::Standby, 0, true),
                        // Both fresh and both up: the lower node id leads.
                        Some(ph) if settings.node_id > ph.node_id => (StoredRole::Standby, 0, true),
                        _ => (StoredRole::Primary, 1, false),
                    }
                }
            };
            tracing::info!("HA: first start of this node, role {role:?}");
            NodeRecord {
                node_id: settings.node_id.clone(),
                role,
                epoch,
                applied_seq: 0,
                needs_full_sync,
                peer_running: vec![],
                last_failover_at: None,
                last_failover_reason: None,
            }
        }
    };

    let mut role = match rec.role {
        StoredRole::Primary => HaRole::Primary,
        StoredRole::Standby => HaRole::Standby,
    };

    if role == HaRole::Primary {
        // A node that was primary must not simply resume.
        let peer = probe_peer(&security, &settings, hello(&settings.node_id, WireRole::Primary, rec.epoch)).await?;
        if let Some(ph) = &peer {
            let outranked = ph.epoch > rec.epoch
                || (ph.role == WireRole::Primary && ph.epoch == rec.epoch && ph.node_id < settings.node_id);
            if outranked {
                tracing::warn!(
                    "HA: peer {} is already primary at epoch {}; this node starts as its standby",
                    ph.node_id,
                    ph.epoch
                );
                rec.role = StoredRole::Standby;
                rec.epoch = ph.epoch.max(rec.epoch);
                rec.applied_seq = 0;
                rec.needs_full_sync = true;
                role = HaRole::Standby;
            }
        }
        if role == HaRole::Primary {
            let witness = Witness::from_config(settings.witness.as_ref(), &security)?;
            match &witness {
                Witness::None => {}
                Witness::Arbiter { .. } => {
                    let ttl = settings.failover_timeout().max(Duration::from_secs(1));
                    match witness.acquire(&settings.node_id, rec.epoch, ttl).await {
                        Ok(Reply::Granted { epoch }) => rec.epoch = rec.epoch.max(epoch),
                        Ok(Reply::Denied { epoch, holder, .. }) => {
                            tracing::warn!("HA: the witness lease is held by {holder}; this node starts as its standby");
                            rec.role = StoredRole::Standby;
                            rec.epoch = epoch.max(rec.epoch);
                            rec.applied_seq = 0;
                            rec.needs_full_sync = true;
                            role = HaRole::Standby;
                        }
                        Err(e) => {
                            tracing::error!("HA: the witness is unreachable ({e:#}); this node starts fenced");
                            role = HaRole::Fenced;
                        }
                    }
                }
                Witness::Reachability { .. } => {
                    if peer.is_none() && !witness.reachable().await {
                        tracing::error!("HA: neither the peer nor the witness host is reachable; this node starts fenced");
                        role = HaRole::Fenced;
                    }
                }
            }
        }
    }

    let saved = rec.clone();
    storage.with_conn(move |c| store::save_node(c, &saved)).await?;

    let start = Start {
        active: role == HaRole::Primary,
        passive_polling: settings.standby_polling,
    };
    *BOOT.lock().unwrap_or_else(|e| e.into_inner()) = Some(BootInfo {
        settings,
        security,
        boot: Boot { role, epoch: rec.epoch },
        record: rec,
    });
    Ok(start)
}

/// HA is configured but the licence does not include it. The node does not
/// take part in a pair, but it must not become a second active node either:
/// a node that was, or is configured as, the standby stays passive.
async fn unlicensed(config: &AppConfig, storage: &Storage) -> Result<Start> {
    let recorded = storage
        .with_conn(|c| {
            let exists: i64 = c.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'ha_node')",
                [],
                |r| r.get(0),
            )?;
            if exists == 0 { Ok(None) } else { store::load_node(c) }
        })
        .await?;
    let standby = match recorded {
        Some(r) => r.role == StoredRole::Standby,
        None => config.ha.role == "standby",
    };
    voltrus::node_role::set_passive_write_exemptions(PASSIVE_WRITE_EXEMPTIONS);
    if standby {
        tracing::error!(
            "Hot standby is configured but the licence does not include it. This node is the standby, so it stays passive and does not poll devices."
        );
    } else {
        tracing::warn!("Hot standby is configured but the licence does not include it: running as a single node.");
    }
    Ok(Start { active: !standby, passive_polling: config.ha.standby_polling })
}
