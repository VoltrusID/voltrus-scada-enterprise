use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::cache::DeviceCache;
use crate::models::{DeviceConfig, PollerStatus};

#[derive(Clone)]
pub struct PollerManager {
    handles: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    cache: DeviceCache,
}

impl PollerManager {
    pub fn new(cache: DeviceCache) -> Self {
        Self {
            handles: Arc::new(RwLock::new(HashMap::new())),
            cache,
        }
    }

    pub async fn start(&self, config: DeviceConfig) {
        let device_id = config.id.clone();
        if self.is_running(&device_id).await {
            self.stop(&device_id).await;
        }
        let cache = self.cache.clone();
        let handle = tokio::spawn(async move {
            crate::modbus::poll_device(config, cache).await;
        });
        self.handles.write().await.insert(device_id, handle);
    }

    pub async fn stop(&self, device_id: &str) {
        if let Some(handle) = self.handles.write().await.remove(device_id) {
            handle.abort();
        }
    }

    pub async fn restart(&self, config: DeviceConfig) {
        self.stop(&config.id).await;
        self.start(config).await;
    }

    pub async fn stop_all(&self) {
        let mut handles = self.handles.write().await;
        for (_, handle) in handles.drain() {
            handle.abort();
        }
    }

    pub async fn get_statuses(&self) -> Vec<PollerStatus> {
        let handles = self.handles.read().await;
        handles
            .keys()
            .map(|id| PollerStatus {
                device_id: id.clone(),
                running: true,
            })
            .collect()
    }

    pub async fn is_running(&self, device_id: &str) -> bool {
        self.handles.read().await.contains_key(device_id)
    }
}
