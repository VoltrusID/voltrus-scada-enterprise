//! Hot standby: two nodes, one active, with replicated configuration and
//! history, fenced failover and controlled switchover.
//!
//! * [`store`]: the change feed (SQLite triggers into a change log, row images,
//!   full snapshots) and the persistent node record (role and epoch).
//! * [`link`]: the authenticated, encrypted node-to-node link (TLS 1.3 plus a
//!   shared-secret proof bound to the TLS session).
//! * [`witness`]: the optional third party that prevents two active nodes.
//! * `node` and `session`: roles, epochs, the replication sessions, failover.
//! * `boot`: deciding the role before any poller starts.
//!
//! The design notes and the data-loss bound are in `docs/hot-standby.md`.

pub mod ha_api;
mod boot;
pub mod link;
mod node;
mod session;
pub mod settings;
pub mod store;
pub mod wire;
pub mod witness;

use std::sync::Arc;

use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use voltrus::models::HaConfig;
use voltrus::poller_manager::PollerManager;
use voltrus::storage::Storage;

pub use boot::resolve_node_role;
pub use node::{Node, SwitchoverReport, Transition};

use wire::WireRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HaRole {
    Primary,
    Standby,
    /// Passive, though it wants to be primary: it lost its lease, or both its
    /// peer and its witness. It resumes by itself when the doubt ends.
    Fenced,
}

impl HaRole {
    pub fn to_wire(self) -> WireRole {
        match self {
            HaRole::Primary => WireRole::Primary,
            HaRole::Standby => WireRole::Standby,
            HaRole::Fenced => WireRole::Fenced,
        }
    }

    pub fn from_wire(w: WireRole) -> Self {
        match w {
            WireRole::Primary => HaRole::Primary,
            WireRole::Standby => HaRole::Standby,
            WireRole::Fenced => HaRole::Fenced,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SyncState {
    /// No session yet.
    #[default]
    Waiting,
    /// A standby whose copy is incomplete.
    Unsynced,
    /// Changes flow.
    Streaming,
    FullSync {
        tables_done: usize,
        tables_total: usize,
        rows: u64,
    },
    Error(String),
}

impl SyncState {
    pub fn name(&self) -> &'static str {
        match self {
            SyncState::Waiting => "waiting",
            SyncState::Unsynced => "unsynced",
            SyncState::Streaming => "streaming",
            SyncState::FullSync { .. } => "full_sync",
            SyncState::Error(_) => "error",
        }
    }

    pub fn progress(&self) -> Option<SyncProgress> {
        match self {
            SyncState::FullSync { tables_done, tables_total, rows } => Some(SyncProgress {
                tables_done: *tables_done,
                tables_total: *tables_total,
                rows: *rows,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncProgress {
    pub tables_done: usize,
    pub tables_total: usize,
    pub rows: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FullSyncReport {
    pub at: String,
    pub tables: usize,
    pub rows: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WitnessStatus {
    pub kind: String,
    /// Whether the last check succeeded. `None` before the first.
    pub reachable: Option<bool>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HaStatus {
    pub enabled: bool,
    pub role: HaRole,
    /// This node polls devices, writes to them and stores history.
    pub active: bool,
    pub node_id: String,
    /// The term. Grows with every new primary.
    pub epoch: u64,
    pub peer_id: Option<String>,
    pub peer_role: Option<HaRole>,
    pub peer_epoch: Option<u64>,
    /// `connected` (a replication session is up), `degraded` (the peer answers
    /// but there is no session), `disconnected`.
    pub peer_status: String,
    /// `waiting`, `unsynced`, `streaming`, `full_sync`, `error`.
    pub sync_state: String,
    pub sync_progress: Option<SyncProgress>,
    /// Age of the oldest change the standby has not acknowledged, in ms.
    pub sync_lag_ms: Option<u64>,
    /// Changes the standby has not applied yet.
    pub pending_changes: Option<u64>,
    /// Standby: the last change of the primary applied. Primary: the last acknowledged.
    pub last_sync_seq: Option<u64>,
    pub last_sync_at: Option<String>,
    pub sync_error: Option<String>,
    /// When this node last took over the primary role.
    pub failover_ts: Option<String>,
    pub failover_reason: Option<String>,
    pub uptime_as_role_secs: u64,
    /// How long a silent primary is tolerated before the standby takes over.
    pub failover_after_ms: u64,
    /// Standby: its copy is complete, so it may take over.
    pub can_take_over: bool,
    /// Standby: why it has not taken over although the primary is silent.
    pub promotion_blocked: Option<String>,
    pub witness: Option<WitnessStatus>,
    pub last_full_sync: Option<FullSyncReport>,
    /// Links that failed the shared-secret proof since this process started.
    pub link_auth_failures: u64,
}

impl HaStatus {
    fn disabled() -> Self {
        HaStatus {
            enabled: false,
            role: HaRole::Primary,
            active: true,
            node_id: String::new(),
            epoch: 0,
            peer_id: None,
            peer_role: None,
            peer_epoch: None,
            peer_status: "disconnected".into(),
            sync_state: "waiting".into(),
            sync_progress: None,
            sync_lag_ms: None,
            pending_changes: None,
            last_sync_seq: None,
            last_sync_at: None,
            sync_error: None,
            failover_ts: None,
            failover_reason: None,
            uptime_as_role_secs: 0,
            failover_after_ms: 0,
            can_take_over: false,
            promotion_blocked: None,
            witness: None,
            last_full_sync: None,
            link_auth_failures: 0,
        }
    }
}

pub(crate) fn rfc3339(secs: i64) -> Option<String> {
    OffsetDateTime::from_unix_timestamp(secs).ok()?.format(&Rfc3339).ok()
}

/// The handle the rest of the add-on holds. Disabled when HA is off or not licensed.
#[derive(Clone)]
pub struct HaState {
    node: Option<Arc<Node>>,
}

impl HaState {
    pub fn disabled() -> Self {
        Self { node: None }
    }

    /// Build the node from the role decision made at boot. Disabled unless the
    /// process started with HA configured and licensed.
    pub fn build(config: &HaConfig, storage: Storage, pollers: PollerManager) -> anyhow::Result<Self> {
        if !config.enabled {
            return Ok(Self::disabled());
        }
        match boot::take() {
            Some(b) => Ok(Self { node: Some(Node::new(b.settings, b.security, storage, pollers, b.boot, &b.record)?) }),
            None => {
                tracing::warn!(
                    "Hot standby is configured but this process started without a hot standby licence; restart after activating one"
                );
                Ok(Self::disabled())
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.node.is_some()
    }

    /// Start the link, the replication and the failover monitor.
    pub fn start(&self) {
        if let Some(n) = &self.node {
            n.start();
        }
    }

    pub async fn get_status(&self) -> HaStatus {
        match &self.node {
            Some(n) => n.status().await,
            None => HaStatus::disabled(),
        }
    }

    pub(crate) fn node(&self) -> Option<&Arc<Node>> {
        self.node.as_ref()
    }
}
