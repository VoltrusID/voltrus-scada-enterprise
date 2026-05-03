use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::cache::DeviceCache;
use crate::modbus::ModbusPool;
use crate::models::{DeviceConfig, OpcUaDeviceConfig, PollerStatus, EnipDeviceConfig};
use crate::s7::S7DeviceConfig;

#[derive(Clone)]
pub struct PollerManager {
    handles: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    cache: DeviceCache,
    modbus_pool: Arc<ModbusPool>,
}

impl PollerManager {
    pub fn new(cache: DeviceCache, modbus_pool: Arc<ModbusPool>) -> Self {
        Self {
            handles: Arc::new(RwLock::new(HashMap::new())),
            cache,
            modbus_pool,
        }
    }

    pub async fn start(&self, config: DeviceConfig) {
        let device_id = config.id.clone();
        if self.is_running(&device_id).await {
            self.stop(&device_id).await;
        }
        let cache = self.cache.clone();
        let pool = self.modbus_pool.clone();
        let handle = tokio::spawn(async move {
            crate::modbus::poll_device(config, cache, pool).await;
        });
        self.handles.write().await.insert(device_id, handle);
    }

    pub async fn start_opcua(&self, config: OpcUaDeviceConfig) {
        let device_id = config.id.clone();
        if self.is_running(&device_id).await {
            self.stop(&device_id).await;
        }
        let cache = self.cache.clone();
        let handle = tokio::spawn(async move {
            crate::opcua::poll_opcua_device(config, cache).await;
        });
        self.handles.write().await.insert(device_id, handle);
    }

    pub async fn start_s7(&self, config: S7DeviceConfig) {
        let device_id = config.id.clone();
        if self.is_running(&device_id).await {
            self.stop(&device_id).await;
        }
        let cache = self.cache.clone();
        let handle = tokio::spawn(async move {
            crate::s7::poll_s7_device(config, cache).await;
        });
        self.handles.write().await.insert(device_id, handle);
    }

    pub async fn start_ethernet_ip(&self, config: EnipDeviceConfig) {
        let device_id = config.id.clone();
        if self.is_running(&device_id).await {
            self.stop(&device_id).await;
        }
        let cache = self.cache.clone();
        let handle = tokio::spawn(async move {
            crate::ethernet_ip::poll_enip_device(
                crate::ethernet_ip::EnipDeviceRuntimeConfig {
                    id: config.id,
                    url: config.url,
                    slot: config.slot,
                    interval_ms: config.interval_ms,
                    tags: config.tags,
                },
                cache,
            ).await;
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
        let cache_statuses = self.cache.get_statuses().await;
        handles
            .keys()
            .map(|id| {
                let status = cache_statuses.get(id).map(|s| s.as_str()).unwrap_or("offline");
                PollerStatus {
                    device_id: id.clone(),
                    running: status == "online",
                    status: status.to_string(),
                }
            })
            .collect()
    }

    pub async fn is_running(&self, device_id: &str) -> bool {
        self.handles.read().await.contains_key(device_id)
    }
}
