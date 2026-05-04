pub mod config;
pub mod auth;

use std::sync::Arc;
use tokio::sync::{watch, RwLock};

use crate::models::MqttBrokerConfig;

/// Runtime status of the embedded MQTT broker.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BrokerStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

/// Connected client info.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectedClient {
    pub client_id: String,
    pub connected_at: i64,
    pub subscriptions: Vec<String>,
}

/// Shared broker state accessible from API handlers.
#[derive(Clone)]
pub struct MqttBrokerState {
    pub status: Arc<watch::Sender<BrokerStatus>>,
    pub status_rx: watch::Receiver<BrokerStatus>,
    pub config: Arc<RwLock<Option<MqttBrokerConfig>>>,
    pub clients: Arc<RwLock<Vec<ConnectedClient>>>,
    pub shutdown_tx: Arc<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl MqttBrokerState {
    pub fn new() -> Self {
        let (status_tx, status_rx) = watch::channel(BrokerStatus::Stopped);
        Self {
            status: Arc::new(status_tx),
            status_rx,
            config: Arc::new(RwLock::new(None)),
            clients: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    pub async fn current_status(&self) -> BrokerStatus {
        self.status_rx.borrow().clone()
    }
}

impl Default for MqttBrokerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the embedded MQTT broker as a background task.
pub async fn start_broker(
    state: MqttBrokerState,
    config: MqttBrokerConfig,
) -> Result<(), String> {
    if state.current_status().await == BrokerStatus::Running {
        return Err("Broker is already running".to_string());
    }

    let _ = state.status.send(BrokerStatus::Starting);

    {
        let mut cfg = state.config.write().await;
        *cfg = Some(config.clone());
    }

    let broker_config = config::build_rumqttd_config(&config);
    let port = config.port;
    let status_tx = state.status.clone();
    let clients = state.clients.clone();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    {
        let mut handle = state.shutdown_tx.lock().await;
        *handle = Some(shutdown_tx);
    }

    let clients_arc = clients.clone();
    let status_tx_clone = status_tx.clone();
    let status_tx_final = status_tx.clone();
    tokio::task::spawn_blocking(move || {
        let mut broker = rumqttd::Broker::new(broker_config);

        let broker_handle = std::thread::spawn(move || {
            if let Err(e) = broker.start() {
                tracing::error!("MQTT broker error: {}", e);
            }
        });

        let _ = shutdown_rx.blocking_recv();

        tracing::info!("MQTT broker shutting down on port {}", port);
        let _ = status_tx_clone.send(BrokerStatus::Stopped);
        drop(broker_handle);
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let _ = status_tx_final.send(BrokerStatus::Running);
    tracing::info!("MQTT broker started on port {}", port);

    Ok(())
}

/// Stop the embedded MQTT broker.
pub async fn stop_broker(state: MqttBrokerState) -> Result<(), String> {
    if state.current_status().await != BrokerStatus::Running {
        return Err("Broker is not running".to_string());
    }

    let mut handle = state.shutdown_tx.lock().await;
    if let Some(tx) = handle.take() {
        let _ = tx.send(());
    }

    {
        let mut clients = state.clients.write().await;
        clients.clear();
    }

    let _ = state.status.send(BrokerStatus::Stopped);
    tracing::info!("MQTT broker stopped");

    Ok(())
}
