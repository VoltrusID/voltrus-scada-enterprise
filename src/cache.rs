use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio_stream::wrappers::BroadcastStream;

use crate::models::{DeviceSnapshot, DeviceStatus};

const SSE_CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone)]
pub struct DeviceCache {
    devices: Arc<RwLock<HashMap<String, DeviceSnapshot>>>,
    tx: broadcast::Sender<Vec<DeviceSnapshot>>,
}

impl DeviceCache {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(SSE_CHANNEL_CAPACITY);
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            tx,
        }
    }

    pub async fn update(&self, device_id: &str, metrics: HashMap<String, f64>) {
        {
            let mut devices = self.devices.write().await;
            let timestamp = crate::timeutil::now_secs();

            devices
                .entry(device_id.to_string())
                .and_modify(|e| {
                    e.status = DeviceStatus::Online;
                    e.timestamp = timestamp;
                    e.metrics = metrics.clone();
                })
                .or_insert_with(|| DeviceSnapshot {
                    device_id: device_id.to_string(),
                    status: DeviceStatus::Online,
                    timestamp,
                    metrics,
                });
        }

        let snapshots = self.get_all().await;
        let _ = self.tx.send(snapshots);
    }

    pub async fn set_status(&self, device_id: &str, status: DeviceStatus) {
        let mut devices = self.devices.write().await;
        if let Some(entry) = devices.get_mut(device_id) {
            entry.status = status;
        }
    }

    pub async fn get_all(&self) -> Vec<DeviceSnapshot> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    pub async fn get_statuses(&self) -> HashMap<String, String> {
        let devices = self.devices.read().await;
        devices
            .iter()
            .map(|(id, snap)| {
                let status_str = match snap.status {
                    DeviceStatus::Online => "online",
                    DeviceStatus::Offline => "offline",
                };
                (id.clone(), status_str.to_string())
            })
            .collect()
    }

    pub fn subscribe(&self) -> BroadcastStream<Vec<DeviceSnapshot>> {
        BroadcastStream::new(self.tx.subscribe())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_and_get_all() {
        let cache = DeviceCache::new();

        let metrics = HashMap::from([
            ("voltage".to_string(), 220.5),
            ("current".to_string(), 5.2),
        ]);

        cache.update("plts_1", metrics).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].device_id, "plts_1");
        assert_eq!(all[0].status, DeviceStatus::Online);
        assert_eq!(all[0].metrics.get("voltage"), Some(&220.5));
        assert_eq!(all[0].metrics.get("current"), Some(&5.2));
    }

    #[tokio::test]
    async fn test_multiple_devices() {
        let cache = DeviceCache::new();

        let m1 = HashMap::from([("voltage".into(), 220.0)]);
        let m2 = HashMap::from([("voltage".into(), 380.0)]);
        cache.update("dev1", m1).await;
        cache.update("dev2", m2).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_update_overwrites() {
        let cache = DeviceCache::new();

        let m1 = HashMap::from([("voltage".into(), 220.0)]);
        let m2 = HashMap::from([("voltage".into(), 225.0)]);
        cache.update("dev1", m1).await;
        cache.update("dev1", m2).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].metrics.get("voltage"), Some(&225.0));
    }

    #[tokio::test]
    async fn test_set_status_offline() {
        let cache = DeviceCache::new();

        let metrics = HashMap::from([("voltage".into(), 220.0)]);
        cache.update("dev1", metrics).await;
        cache.set_status("dev1", DeviceStatus::Offline).await;

        let all = cache.get_all().await;
        assert_eq!(all[0].status, DeviceStatus::Offline);

        let statuses = cache.get_statuses().await;
        assert_eq!(statuses.get("dev1"), Some(&"offline".to_string()));
    }

    #[tokio::test]
    async fn test_get_statuses() {
        let cache = DeviceCache::new();

        let m1 = HashMap::from([("voltage".into(), 220.0)]);
        let m2 = HashMap::from([("voltage".into(), 380.0)]);
        cache.update("dev1", m1).await;
        cache.update("dev2", m2).await;
        cache.set_status("dev2", DeviceStatus::Offline).await;

        let statuses = cache.get_statuses().await;
        assert_eq!(statuses.get("dev1"), Some(&"online".to_string()));
        assert_eq!(statuses.get("dev2"), Some(&"offline".to_string()));
    }

    #[tokio::test]
    async fn test_set_status_nonexistent() {
        let cache = DeviceCache::new();
        cache.set_status("ghost", DeviceStatus::Offline).await;

        let all = cache.get_all().await;
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_broadcast_on_update() {
        let cache = DeviceCache::new();
        let mut stream = cache.subscribe();

        let metrics = HashMap::from([("voltage".into(), 220.0)]);
        cache.update("dev1", metrics).await;

        use tokio_stream::StreamExt;
        let msg: Vec<DeviceSnapshot> = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stream.next(),
        )
        .await
        .expect("should receive broadcast")
        .expect("should be Some")
        .expect("broadcast should not error");

        assert_eq!(msg.len(), 1);
        assert_eq!(msg[0].device_id, "dev1");
    }
}
