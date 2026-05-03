use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, watch};
use tokio_stream::wrappers::BroadcastStream;

use crate::models::{DeviceSnapshot, DeviceStatus};

const SSE_CHANNEL_CAPACITY: usize = 256;
const SNAPSHOT_CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone)]
pub struct DeviceCache {
    devices: Arc<RwLock<HashMap<String, DeviceSnapshot>>>,
    tx: broadcast::Sender<String>,
    snapshot_tx: broadcast::Sender<DeviceSnapshot>,
    trigger: watch::Sender<bool>,
}

impl DeviceCache {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(SSE_CHANNEL_CAPACITY);
        let (snapshot_tx, _) = broadcast::channel(SNAPSHOT_CHANNEL_CAPACITY);
        let (trigger, _) = watch::channel(false);
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            tx,
            snapshot_tx,
            trigger,
        }
    }

    pub async fn update(&self, device_id: &str, metrics: HashMap<String, f64>, simulated: bool) {
        let snapshot = {
            let mut devices = self.devices.write().await;
            let timestamp = crate::timeutil::now_secs();

            devices
                .entry(device_id.to_string())
                .and_modify(|e| {
                    e.status = DeviceStatus::Online;
                    e.timestamp = timestamp;
                    e.metrics = metrics.clone();
                    e.simulated = simulated;
                })
                .or_insert_with(|| DeviceSnapshot {
                    device_id: device_id.to_string(),
                    status: DeviceStatus::Online,
                    timestamp,
                    metrics,
                    simulated,
                })
                .clone()
        };

        let _ = self.snapshot_tx.send(snapshot);
        let _ = self.trigger.send(true);
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

    pub async fn is_device_online(&self, device_id: &str) -> bool {
        let devices = self.devices.read().await;
        devices
            .get(device_id)
            .map_or(false, |s| s.status == DeviceStatus::Online)
    }

    pub async fn get_metric_value(&self, key: &str) -> Option<f64> {
        let devices = self.devices.read().await;
        for snapshot in devices.values() {
            if let Some(v) = snapshot.metrics.get(key) {
                return Some(*v);
            }
        }
        None
    }

    pub fn publish_sse_payload(&self, payload: String) {
        let _ = self.tx.send(payload);
    }

    pub fn subscribe_sse(&self) -> BroadcastStream<String> {
        BroadcastStream::new(self.tx.subscribe())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DeviceSnapshot> {
        self.snapshot_tx.subscribe()
    }

    pub fn trigger_receiver(&self) -> watch::Receiver<bool> {
        self.trigger.subscribe()
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

        cache.update("plts_1", metrics, false).await;

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
        cache.update("dev1", m1, false).await;
        cache.update("dev2", m2, false).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_update_overwrites() {
        let cache = DeviceCache::new();

        let m1 = HashMap::from([("voltage".into(), 220.0)]);
        let m2 = HashMap::from([("voltage".into(), 225.0)]);
        cache.update("dev1", m1, false).await;
        cache.update("dev1", m2, false).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].metrics.get("voltage"), Some(&225.0));
    }

    #[tokio::test]
    async fn test_set_status_offline() {
        let cache = DeviceCache::new();

        let metrics = HashMap::from([("voltage".into(), 220.0)]);
        cache.update("dev1", metrics, false).await;
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
        cache.update("dev1", m1, false).await;
        cache.update("dev2", m2, false).await;
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
    async fn test_publish_and_receive_sse_payload() {
        let cache = DeviceCache::new();
        let mut stream = cache.subscribe_sse();

        cache.publish_sse_payload(r#"{"test":true}"#.to_string());

        use tokio_stream::StreamExt;
        let msg: String = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stream.next(),
        )
        .await
        .expect("should receive broadcast")
        .expect("should be Some")
        .expect("broadcast should not error");

        assert_eq!(msg, r#"{"test":true}"#);
    }

    #[tokio::test]
    async fn test_trigger_fires_on_update() {
        let cache = DeviceCache::new();
        let mut rx = cache.trigger_receiver();

        let metrics = HashMap::from([("voltage".into(), 220.0)]);
        cache.update("dev1", metrics, false).await;

        tokio::time::timeout(
            std::time::Duration::from_millis(50),
            rx.changed(),
        )
        .await
        .expect("trigger should fire on update")
        .expect("should succeed");
    }

    #[tokio::test]
    async fn test_simulated_flag_defaults_false() {
        let cache = DeviceCache::new();

        let metrics = HashMap::from([("voltage".into(), 220.0)]);
        cache.update("dev1", metrics, false).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 1);
        assert!(!all[0].simulated);
    }

    #[tokio::test]
    async fn test_simulated_flag_set_true() {
        let cache = DeviceCache::new();

        let metrics = HashMap::from([("voltage".into(), 220.0)]);
        cache.update("dev1", metrics, true).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 1);
        assert!(all[0].simulated);
    }

    #[tokio::test]
    async fn test_simulated_flag_toggles() {
        let cache = DeviceCache::new();

        let m1 = HashMap::from([("voltage".into(), 220.0)]);
        let m2 = HashMap::from([("voltage".into(), 225.0)]);

        cache.update("dev1", m1, false).await;
        assert!(!cache.get_all().await[0].simulated);

        cache.update("dev1", m2.clone(), true).await;
        assert!(cache.get_all().await[0].simulated);

        cache.update("dev1", m2, false).await;
        assert!(!cache.get_all().await[0].simulated);
    }
}
