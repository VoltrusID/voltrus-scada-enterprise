pub mod ha_api;
pub mod readback;
pub mod replication;
pub mod watchdog;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::HaConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HaRole {
    Primary,
    Standby,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HaStatus {
    pub enabled: bool,
    pub role: HaRole,
    pub node_id: String,
    pub peer_id: Option<String>,
    pub peer_status: String,
    pub sync_lag_ms: Option<u64>,
    pub last_sync_seq: Option<u64>,
    pub failover_ts: Option<String>,
    pub uptime_as_role_secs: u64,
}

#[derive(Clone)]
pub struct HaState {
    inner: Arc<RwLock<HaStatus>>,
}

impl HaState {
    pub fn new(config: &HaConfig) -> Self {
        let role = match config.role.as_str() {
            "primary" => HaRole::Primary,
            "standby" => HaRole::Standby,
            _ => HaRole::Primary,
        };

        Self {
            inner: Arc::new(RwLock::new(HaStatus {
                enabled: config.enabled,
                role,
                node_id: config.node_id.clone(),
                peer_id: None,
                peer_status: "disconnected".to_string(),
                sync_lag_ms: None,
                last_sync_seq: None,
                failover_ts: None,
                uptime_as_role_secs: 0,
            })),
        }
    }

    pub fn disabled() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HaStatus {
                enabled: false,
                role: HaRole::Primary,
                node_id: String::new(),
                peer_id: None,
                peer_status: "disconnected".to_string(),
                sync_lag_ms: None,
                last_sync_seq: None,
                failover_ts: None,
                uptime_as_role_secs: 0,
            })),
        }
    }

    pub async fn get_status(&self) -> HaStatus {
        self.inner.read().await.clone()
    }

    pub async fn set_role(&self, role: HaRole) {
        self.inner.write().await.role = role;
    }

    pub async fn set_peer_status(&self, status: &str, lag_ms: Option<u64>) {
        let mut inner = self.inner.write().await;
        inner.peer_status = status.to_string();
        inner.sync_lag_ms = lag_ms;
    }
}

pub fn start_ha(config: &HaConfig, ha_state: HaState) {
    if !config.enabled {
        return;
    }

    let config = config.clone();

    match config.role.as_str() {
        "primary" => {
            let cfg = config.clone();
            let state = ha_state.clone();
            tokio::spawn(async move {
                if let Err(e) = replication::run_replication_server(&cfg, state).await {
                    tracing::error!("Replication server error: {}", e);
                }
            });
        }
        "standby" => {
            let cfg = config.clone();
            let state = ha_state.clone();
            tokio::spawn(async move {
                if let Err(e) = replication::run_replication_client(&cfg, state).await {
                    tracing::error!("Replication client error: {}", e);
                }
            });
        }
        "auto" => {
            let server_cfg = config.clone();
            let server_state = ha_state.clone();
            tokio::spawn(async move {
                if let Err(e) = replication::run_replication_server(&server_cfg, server_state).await
                {
                    tracing::error!("Replication server error: {}", e);
                }
            });

            let client_cfg = config.clone();
            let client_state = ha_state.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if let Err(e) = replication::run_replication_client(&client_cfg, client_state).await
                {
                    tracing::error!("Replication client error: {}", e);
                }
            });
        }
        _ => {}
    }
}

/// Periodically broadcast HA status via SSE and WebSocket.
pub fn start_ha_broadcast(ha_state: HaState, cache: crate::cache::DeviceCache) {
    use std::time::Duration;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let status = ha_state.get_status().await;
            if status.enabled {
                cache.broadcast_ha_status(&status).await;
            }
        }
    });
}
