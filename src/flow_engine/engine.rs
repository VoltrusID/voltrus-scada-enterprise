use crate::flow_engine::{message::Message, node::NodeConfig, trigger::TriggerConfig, state::FlowState, run_log};
use crate::storage::Storage;
use crate::cache::DeviceCache;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Flow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger: TriggerConfig,
    pub nodes: Vec<NodeConfig>,
}

pub struct FlowEngine {
    flows: Arc<RwLock<HashMap<String, Flow>>>,
    state: Arc<FlowState>,
    storage: Storage,
    cache: DeviceCache,
}

impl FlowEngine {
    pub fn new(
        storage: Storage,
        cache: DeviceCache,
    ) -> Self {
        Self {
            flows: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(FlowState::new()),
            storage,
            cache,
        }
    }

    pub async fn spawn(&self) {
        let flows = self.flows.clone();
        let state = self.state.clone();
        let storage = self.storage.clone();
        let cache = self.cache.clone();
        let mut rx = self.cache.subscribe();

        // Spawn timer flows task
        {
            let flows = flows.clone();
            let state = state.clone();
            let storage = storage.clone();
            let cache = cache.clone();
            tokio::spawn(async move {
                let mut last_run: HashMap<String, std::time::Instant> = HashMap::new();
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    let flows_guard = flows.read().await;
                    for flow in flows_guard.values() {
                        if !flow.enabled { continue; }
                        if let TriggerConfig::Timer { interval_ms } = &flow.trigger {
                            let now = std::time::Instant::now();
                            let should_run = match last_run.get(&flow.id) {
                                Some(last) => now.duration_since(*last).as_millis() as u64 >= *interval_ms,
                                None => true,
                            };
                            if should_run {
                                last_run.insert(flow.id.clone(), now);
                                let flow = flow.clone();
                                let state = state.clone();
                                let storage = storage.clone();
                                let cache = cache.clone();
                                let msg = Message::new(serde_json::json!({"timestamp": crate::timeutil::now_secs()}));
                                tokio::spawn(async move {
                                    run_flow(&flow, msg, state.as_ref(), &storage, &cache).await;
                                });
                            }
                        }
                    }
                }
            });
        }

        // Main loop: process broadcast messages
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(snapshot) => {
                        let msg = device_snapshot_to_message(snapshot);
                        let flows_guard = flows.read().await;
                        for flow in flows_guard.values() {
                            if !flow.enabled { continue; }
                            if flow.trigger.matches(&msg) {
                                let flow = flow.clone();
                                let state = state.clone();
                                let storage = storage.clone();
                                let cache = cache.clone();
                                let msg = msg.clone();
                                tokio::spawn(async move {
                                    run_flow(&flow, msg, state.as_ref(), &storage, &cache).await;
                                });
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("FlowEngine lagged by {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::info!("FlowEngine broadcast channel closed");
                        break;
                    }
                }
            }
        });
    }

    pub async fn add_flow(&self, flow: Flow) {
        let mut flows = self.flows.write().await;
        flows.insert(flow.id.clone(), flow);
    }

    pub async fn remove_flow(&self, flow_id: &str) {
        let mut flows = self.flows.write().await;
        flows.remove(flow_id);
    }

    pub async fn get_flow(&self, flow_id: &str) -> Option<Flow> {
        let flows = self.flows.read().await;
        flows.get(flow_id).cloned()
    }

    pub async fn list_flows(&self) -> Vec<Flow> {
        let flows = self.flows.read().await;
        flows.values().cloned().collect()
    }

    pub async fn update_flow(&self, flow: Flow) {
        let mut flows = self.flows.write().await;
        flows.insert(flow.id.clone(), flow);
    }
}

fn device_snapshot_to_message(snapshot: crate::models::DeviceSnapshot) -> Message {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "modbus".to_string());
    let payload = serde_json::json!({
        "device_id": snapshot.device_id,
        "status": match snapshot.status {
            crate::models::DeviceStatus::Online => "online",
            crate::models::DeviceStatus::Offline => "offline",
        },
        "timestamp": snapshot.timestamp,
        "metrics": snapshot.metrics,
        "simulated": snapshot.simulated,
    });
    let mut msg = Message::new(payload);
    msg.metadata = metadata;
    msg
}

async fn run_flow(flow: &Flow, mut msg: Message, state: &FlowState, storage: &Storage, cache: &DeviceCache) {
    let start = std::time::Instant::now();
    for node in &flow.nodes {
        let ctx = crate::flow_engine::node::NodeContext {
            flow_id: flow.id.clone(),
            node_id: node_id(node),
            state: state.clone(),
            storage: storage.clone(),
            cache: cache.clone(),
        };
        match node.process(msg, &ctx) {
            Ok(crate::flow_engine::node::NodeResult::Continue(new_msg)) => {
                msg = new_msg;
            }
            Ok(crate::flow_engine::node::NodeResult::FilterReject) => {
                run_log::log_run(storage, &flow.id, start.elapsed().as_millis() as u64, true, None, None).await;
                return;
            }
            Err(e) => {
                run_log::log_run(storage, &flow.id, start.elapsed().as_millis() as u64, false, Some(e), None).await;
                return;
            }
        }
    }
    run_log::log_run(storage, &flow.id, start.elapsed().as_millis() as u64, true, None, Some(msg.payload)).await;
}

fn node_id(node: &NodeConfig) -> String {
    match node {
        NodeConfig::Filter { id, .. } => id.clone(),
        NodeConfig::Transform { id, .. } => id.clone(),
        NodeConfig::Threshold { id, .. } => id.clone(),
        NodeConfig::Lookup { id, .. } => id.clone(),
        NodeConfig::Expression { id, .. } => id.clone(),
        NodeConfig::DataSourceOut { id, .. } => id.clone(),
        NodeConfig::HttpOut { id, .. } => id.clone(),
        NodeConfig::AlarmOut { id, .. } => id.clone(),
        NodeConfig::WriteOut { id, .. } => id.clone(),
        NodeConfig::Script { id, .. } => id.clone(),
    }
}
