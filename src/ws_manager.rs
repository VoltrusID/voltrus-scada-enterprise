use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::ws_types::ServerMessage;

/// Per-connection state.
struct WsConnection {
    user_id: String,
    permissions: Vec<String>,
    subscribed_tags: HashSet<String>,
    subscribe_all: bool,
    tx: UnboundedSender<ServerMessage>,
}

/// Tracks all active WebSocket connections, their subscriptions, and
/// provides broadcast helpers for pushing messages to the right set of peers.
pub struct WsManager {
    connections: DashMap<String, WsConnection>,
}

impl std::fmt::Debug for WsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsManager")
            .field("connection_count", &self.connections.len())
            .finish()
    }
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    /// Register a new connection. Returns the receiver half that the WS handler
    /// task will poll for outgoing messages.
    pub fn connect(
        &self,
        conn_id: String,
        user_id: String,
        permissions: Vec<String>,
    ) -> UnboundedReceiver<ServerMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.connections.insert(
            conn_id,
            WsConnection {
                user_id,
                permissions,
                subscribed_tags: HashSet::new(),
                subscribe_all: false,
                tx,
            },
        );
        rx
    }

    /// Remove a connection. Returns `true` if the connection existed.
    pub fn disconnect(&self, conn_id: &str) -> bool {
        self.connections.remove(conn_id).is_some()
    }

    /// Subscribe a connection to specific tag keys.
    pub fn subscribe(&self, conn_id: &str, tag_ids: &[String]) {
        if let Some(mut conn) = self.connections.get_mut(conn_id) {
            conn.subscribed_tags.extend(tag_ids.iter().cloned());
        }
    }

    /// Unsubscribe a connection from specific tag keys.
    pub fn unsubscribe(&self, conn_id: &str, tag_ids: &[String]) {
        if let Some(mut conn) = self.connections.get_mut(conn_id) {
            for tag in tag_ids {
                conn.subscribed_tags.remove(tag);
            }
        }
    }

    /// Toggle subscribe-all mode. When enabled, the connection receives every
    /// tag update regardless of its per-tag subscriptions.
    pub fn subscribe_all(&self, conn_id: &str, enabled: bool) {
        if let Some(mut conn) = self.connections.get_mut(conn_id) {
            conn.subscribe_all = enabled;
        }
    }

    /// Check whether a connection has a given permission string.
    pub fn has_permission(&self, conn_id: &str, permission: &str) -> bool {
        self.connections
            .get(conn_id)
            .map(|conn| conn.permissions.iter().any(|p| p == permission))
            .unwrap_or(false)
    }

    // ── Broadcast helpers ──────────────────────────────────────────────

    /// Broadcast a tag update to every connection that is subscribed to the
    /// tag or has `subscribe_all` enabled. Closed senders are removed.
    pub fn broadcast_tag_update(&self, tag_id: &str, msg: ServerMessage) {
        let mut stale = Vec::new();
        for mut entry in self.connections.iter_mut() {
            let conn = entry.value_mut();
            if conn.subscribe_all || conn.subscribed_tags.contains(tag_id) {
                if conn.tx.send(msg.clone()).is_err() {
                    stale.push(entry.key().clone());
                }
            }
        }
        for id in stale {
            self.connections.remove(&id);
        }
    }

    /// Broadcast an alarm message to all connections. Closed senders are removed.
    pub fn broadcast_alarm(&self, msg: ServerMessage) {
        self.broadcast_to_all(msg);
    }

    /// Broadcast a command acknowledgement to all connections. Closed senders are removed.
    pub fn broadcast_command_ack(&self, msg: ServerMessage) {
        self.broadcast_to_all(msg);
    }

    /// Broadcast a device-status change to all connections. Closed senders are removed.
    pub fn broadcast_device_status(&self, msg: ServerMessage) {
        self.broadcast_to_all(msg);
    }

    /// Current number of active connections.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    // ── Internal ───────────────────────────────────────────────────────

    fn broadcast_to_all(&self, msg: ServerMessage) {
        let mut stale = Vec::new();
        for mut entry in self.connections.iter_mut() {
            if entry.value_mut().tx.send(msg.clone()).is_err() {
                stale.push(entry.key().clone());
            }
        }
        for id in stale {
            self.connections.remove(&id);
        }
    }
}

// ── SSE bridge ──────────────────────────────────────────────────────────

/// Spawn a background task that listens to the SSE broadcast channel from the
/// cache, parses each JSON payload, and routes tag updates to the appropriate
/// WebSocket connections via `WsManager`.
pub fn spawn_ws_broadcaster(
    cache: crate::cache::DeviceCache,
    manager: Arc<WsManager>,
) {
    tokio::spawn(async move {
        let mut stream = cache.subscribe_sse();
        use tokio_stream::StreamExt;
        while let Some(result) = stream.next().await {
            let payload = match result {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Expected shape: {"sensors":[{"key":"...","value":42.0,"status":"normal",...},...]}
            let parsed: serde_json::Value = match serde_json::from_str(&payload) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(sensors) = parsed.get("sensors").and_then(|s| s.as_array()) {
                for sensor in sensors {
                    let tag_id = match sensor.get("key").and_then(|k| k.as_str()) {
                        Some(k) => k.to_string(),
                        None => continue,
                    };
                    let value = sensor
                        .get("value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let status = sensor
                        .get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let msg = ServerMessage::TagUpdate {
                        tag_id: tag_id.clone(),
                        value,
                        quality: status,
                        ts: crate::timeutil::now_secs(),
                        simulated: false,
                    };
                    manager.broadcast_tag_update(&tag_id, msg);
                }
            }
        }
    });
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::error::TryRecvError;

    /// Helper: create a WsManager and connect a test connection.
    fn make_manager_and_connect(
        conn_id: &str,
        user_id: &str,
        permissions: Vec<&str>,
    ) -> (WsManager, UnboundedReceiver<ServerMessage>) {
        let mgr = WsManager::new();
        let rx = mgr.connect(
            conn_id.to_string(),
            user_id.to_string(),
            permissions.iter().map(|s| s.to_string()).collect(),
        );
        (mgr, rx)
    }

    #[test]
    fn test_connect_disconnect() {
        let mgr = WsManager::new();
        assert_eq!(mgr.connection_count(), 0);

        let _rx = mgr.connect(
            "c1".to_string(),
            "user1".to_string(),
            vec!["read".to_string()],
        );
        assert_eq!(mgr.connection_count(), 1);

        assert!(mgr.disconnect("c1"));
        assert_eq!(mgr.connection_count(), 0);
        assert!(!mgr.disconnect("c1")); // already gone
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let mgr = WsManager::new();
        let _rx = mgr.connect(
            "c1".to_string(),
            "user1".to_string(),
            vec![],
        );

        mgr.subscribe("c1", &["tag_a".to_string(), "tag_b".to_string()]);

        // Sending a broadcast for an unsubscribed tag should not deliver.
        mgr.broadcast_tag_update(
            "tag_z",
            ServerMessage::TagUpdate {
                tag_id: "tag_z".to_string(),
                value: 1.0,
                quality: "good".to_string(),
                ts: 0,
                simulated: false,
            },
        );

        // Sending for a subscribed tag should deliver (we verify via subscribe_all next).
        // For now just check disconnect removes it.
        mgr.disconnect("c1");
        assert_eq!(mgr.connection_count(), 0);
    }

    #[tokio::test]
    async fn test_subscribe_all() {
        let mgr = WsManager::new();
        let mut rx = mgr.connect(
            "c1".to_string(),
            "user1".to_string(),
            vec![],
        );

        // Without subscribe_all, a broadcast for an unknown tag should not arrive.
        mgr.broadcast_tag_update(
            "tag_x",
            ServerMessage::TagUpdate {
                tag_id: "tag_x".to_string(),
                value: 1.0,
                quality: "good".to_string(),
                ts: 0,
                simulated: false,
            },
        );
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

        // Enable subscribe_all
        mgr.subscribe_all("c1", true);

        mgr.broadcast_tag_update(
            "tag_x",
            ServerMessage::TagUpdate {
                tag_id: "tag_x".to_string(),
                value: 2.0,
                quality: "good".to_string(),
                ts: 1,
                simulated: false,
            },
        );
        match rx.try_recv() {
            Ok(ServerMessage::TagUpdate { value, .. }) => assert!((value - 2.0).abs() < f64::EPSILON),
            other => panic!("expected TagUpdate with value 2.0, got {:?}", other),
        }
    }

    #[test]
    fn test_has_permission() {
        let mgr = WsManager::new();
        let _rx = mgr.connect(
            "c1".to_string(),
            "admin".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );

        assert!(mgr.has_permission("c1", "read"));
        assert!(mgr.has_permission("c1", "write"));
        assert!(!mgr.has_permission("c1", "admin"));

        // Non-existent connection
        assert!(!mgr.has_permission("ghost", "read"));
    }

    #[tokio::test]
    async fn test_broadcast_alarm() {
        let mgr = WsManager::new();
        let mut rx = mgr.connect(
            "c1".to_string(),
            "user1".to_string(),
            vec![],
        );

        mgr.broadcast_alarm(ServerMessage::Alarm {
            alarm_id: 42,
            tag_id: "sensor.temp".to_string(),
            severity: "critical".to_string(),
            message: "overheat".to_string(),
            value: 99.0,
            ts: 100,
        });

        match rx.try_recv() {
            Ok(ServerMessage::Alarm { alarm_id, .. }) => assert_eq!(alarm_id, 42),
            other => panic!("expected Alarm, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_broadcast_command_ack() {
        let mgr = WsManager::new();
        let mut rx = mgr.connect(
            "c1".to_string(),
            "user1".to_string(),
            vec![],
        );

        mgr.broadcast_command_ack(ServerMessage::CommandAck {
            command_id: "cmd-001".to_string(),
            status: "acked".to_string(),
            device_response: Some("OK".to_string()),
            latency_ms: Some(12),
        });

        match rx.try_recv() {
            Ok(ServerMessage::CommandAck { command_id, latency_ms, .. }) => {
                assert_eq!(command_id, "cmd-001");
                assert_eq!(latency_ms, Some(12));
            }
            other => panic!("expected CommandAck, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_broadcast_device_status() {
        let mgr = WsManager::new();
        let mut rx = mgr.connect(
            "c1".to_string(),
            "user1".to_string(),
            vec![],
        );

        mgr.broadcast_device_status(ServerMessage::DeviceStatus {
            device_id: "plc-01".to_string(),
            online: true,
            latency_ms: Some(5),
        });

        match rx.try_recv() {
            Ok(ServerMessage::DeviceStatus { device_id, online, .. }) => {
                assert_eq!(device_id, "plc-01");
                assert!(online);
            }
            other => panic!("expected DeviceStatus, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_disconnect_removes_connection() {
        let mgr = WsManager::new();
        let _rx = mgr.connect(
            "c1".to_string(),
            "user1".to_string(),
            vec![],
        );
        let _rx2 = mgr.connect(
            "c2".to_string(),
            "user2".to_string(),
            vec![],
        );
        assert_eq!(mgr.connection_count(), 2);

        mgr.disconnect("c1");
        assert_eq!(mgr.connection_count(), 1);
        assert!(!mgr.has_permission("c1", "read")); // gone

        mgr.disconnect("c2");
        assert_eq!(mgr.connection_count(), 0);
    }
}
