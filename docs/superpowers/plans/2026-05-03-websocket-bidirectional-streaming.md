# v0.29.0 — WebSocket Bidirectional Streaming — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add WebSocket endpoint alongside REST+SSE for bidirectional real-time communication with MessagePack binary encoding.

**Architecture:** Axum built-in WebSocket (`axum::extract::ws`) + `rmp-serde` for MessagePack. New `WsManager` tracks connections and subscriptions. Reuses existing `DeviceCache` broadcast channel and `CommandQueue` for writes.

**Tech Stack:** Rust (Axum 0.8.9 WS support, rmp-serde), TypeScript (@msgpack/msgpack)

---

### Task 1: Bump version and add rmp-serde dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Update Cargo.toml**

Change version from `0.26.0` to `0.29.0`. Add `rmp-serde` dependency:

```toml
version = "0.29.0"
```

Add to `[dependencies]`:
```toml
rmp-serde = "=1.3.0"
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.29.0 and add rmp-serde dependency"
```

---

### Task 2: Define WebSocket message types (ws_types.rs)

**Files:**
- Create: `src/ws_types.rs`

- [ ] **Step 1: Create ws_types.rs with ClientMessage and ServerMessage enums**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Auth { token: String },
    Subscribe { tag_ids: Vec<String> },
    Unsubscribe { tag_ids: Vec<String> },
    TagWrite {
        tag_id: String,
        value: f64,
        command_id: Option<String>,
    },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    AuthOk {
        user: String,
        permissions: Vec<String>,
    },
    AuthError {
        message: String,
    },
    TagUpdate {
        tag_id: String,
        value: f64,
        quality: String,
        ts: i64,
        simulated: bool,
    },
    CommandAck {
        command_id: String,
        status: String,
        device_response: Option<String>,
        latency_ms: Option<u64>,
    },
    DeviceStatus {
        device_id: String,
        online: bool,
        latency_ms: Option<u64>,
    },
    Alarm {
        alarm_id: i64,
        tag_id: String,
        severity: String,
        message: String,
        value: f64,
        ts: i64,
    },
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_roundtrip() {
        let msg = ClientMessage::Subscribe {
            tag_ids: vec!["PLC01.voltage".to_string(), "PLC01.current".to_string()],
        };
        let bytes = rmp_serde::to_vec(&msg).unwrap();
        let decoded: ClientMessage = rmp_serde::from_slice(&bytes).unwrap();
        match decoded {
            ClientMessage::Subscribe { tag_ids } => {
                assert_eq!(tag_ids.len(), 2);
                assert_eq!(tag_ids[0], "PLC01.voltage");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_roundtrip() {
        let msg = ServerMessage::TagUpdate {
            tag_id: "PLC01.voltage".to_string(),
            value: 230.5,
            quality: "good".to_string(),
            ts: 1700000000,
            simulated: false,
        };
        let bytes = rmp_serde::to_vec(&msg).unwrap();
        let decoded: ServerMessage = rmp_serde::from_slice(&bytes).unwrap();
        match decoded {
            ServerMessage::TagUpdate { tag_id, value, .. } => {
                assert_eq!(tag_id, "PLC01.voltage");
                assert!((value - 230.5).abs() < 0.001);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_all_client_message_types() {
        let messages = vec![
            ClientMessage::Auth { token: "test-jwt".to_string() },
            ClientMessage::Subscribe { tag_ids: vec![] },
            ClientMessage::Unsubscribe { tag_ids: vec![] },
            ClientMessage::TagWrite { tag_id: "t1".to_string(), value: 42.0, command_id: None },
            ClientMessage::Ping,
        ];
        for msg in &messages {
            let bytes = rmp_serde::to_vec(msg).unwrap();
            let _: ClientMessage = rmp_serde::from_slice(&bytes).unwrap();
        }
    }

    #[test]
    fn test_all_server_message_types() {
        let messages = vec![
            ServerMessage::AuthOk { user: "admin".to_string(), permissions: vec![] },
            ServerMessage::AuthError { message: "bad token".to_string() },
            ServerMessage::TagUpdate { tag_id: "t".to_string(), value: 1.0, quality: "good".to_string(), ts: 0, simulated: false },
            ServerMessage::CommandAck { command_id: "c1".to_string(), status: "acked".to_string(), device_response: None, latency_ms: Some(50) },
            ServerMessage::DeviceStatus { device_id: "d1".to_string(), online: true, latency_ms: Some(10) },
            ServerMessage::Alarm { alarm_id: 1, tag_id: "t".to_string(), severity: "warning".to_string(), message: "high".to_string(), value: 99.0, ts: 0 },
            ServerMessage::Pong,
        ];
        for msg in &messages {
            let bytes = rmp_serde::to_vec(msg).unwrap();
            let _: ServerMessage = rmp_serde::from_slice(&bytes).unwrap();
        }
    }

    #[test]
    fn test_messagepack_size_efficiency() {
        let msg = ServerMessage::TagUpdate {
            tag_id: "PLC01.voltage".to_string(),
            value: 230.5,
            quality: "good".to_string(),
            ts: 1700000000,
            simulated: false,
        };
        let msgpack_bytes = rmp_serde::to_vec(&msg).unwrap();
        let json_bytes = serde_json::to_vec(&msg).unwrap();
        assert!(msgpack_bytes.len() < json_bytes.len(), "MessagePack should be smaller than JSON");
    }
}
```

- [ ] **Step 2: Add `mod ws_types;` to main.rs**

Add `pub mod ws_types;` alongside other `mod` declarations in `src/main.rs`.

- [ ] **Step 3: Run tests**

Run: `cargo test ws_types`
Expected: all 5 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/ws_types.rs src/main.rs
git commit -m "feat(ws): add WebSocket message types with MessagePack serialization"
```

---

### Task 3: Implement WsManager connection tracker (ws_manager.rs)

**Files:**
- Create: `src/ws_manager.rs`

- [ ] **Step 1: Create ws_manager.rs**

```rust
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::ws_types::ServerMessage;

type ConnectionId = String;

struct WsConnection {
    user_id: String,
    permissions: Vec<String>,
    subscribed_tags: HashSet<String>,
    subscribe_all: bool,
    tx: UnboundedSender<ServerMessage>,
}

pub struct WsManager {
    connections: DashMap<ConnectionId, WsConnection>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    pub fn connect(
        &self,
        id: ConnectionId,
        user_id: String,
        permissions: Vec<String>,
    ) -> UnboundedReceiver<ServerMessage> {
        let (tx, rx) = unbounded_channel();
        self.connections.insert(id, WsConnection {
            user_id,
            permissions,
            subscribed_tags: HashSet::new(),
            subscribe_all: false,
            tx,
        });
        info!("WS connected: {} connections active", self.connections.len());
        rx
    }

    pub fn disconnect(&self, id: &str) {
        if self.connections.remove(id).is_some() {
            debug!("WS disconnected: {} connections active", self.connections.len());
        }
    }

    pub fn subscribe(&self, conn_id: &str, tag_ids: Vec<String>) -> Result<(), String> {
        let mut conn = self.connections.get_mut(conn_id)
            .ok_or("Connection not found")?;
        for tag_id in &tag_ids {
            conn.subscribed_tags.insert(tag_id.clone());
        }
        debug!("WS subscribe: {} tags for conn {}", tag_ids.len(), conn_id);
        Ok(())
    }

    pub fn subscribe_all(&self, conn_id: &str) -> Result<(), String> {
        let mut conn = self.connections.get_mut(conn_id)
            .ok_or("Connection not found")?;
        conn.subscribe_all = true;
        Ok(())
    }

    pub fn unsubscribe(&self, conn_id: &str, tag_ids: Vec<String>) -> Result<(), String> {
        let mut conn = self.connections.get_mut(conn_id)
            .ok_or("Connection not found")?;
        for tag_id in &tag_ids {
            conn.subscribed_tags.remove(tag_id);
        }
        Ok(())
    }

    pub fn has_permission(&self, conn_id: &str, permission: &str) -> bool {
        self.connections.get(conn_id)
            .map(|c| c.permissions.contains(&permission.to_string()))
            .unwrap_or(false)
    }

    pub fn broadcast_tag_update(&self, tag_id: &str, value: f64, quality: &str, ts: i64, simulated: bool) {
        let msg = ServerMessage::TagUpdate {
            tag_id: tag_id.to_string(),
            value,
            quality: quality.to_string(),
            ts,
            simulated,
        };
        for mut entry in self.connections.iter_mut() {
            if entry.subscribe_all || entry.subscribed_tags.contains(tag_id) {
                if entry.tx.send(msg.clone()).is_err() {
                    drop(entry);
                    self.connections.remove(&entry.key().clone());
                }
            }
        }
    }

    pub fn broadcast_alarm(&self, alarm_id: i64, tag_id: &str, severity: &str, message: &str, value: f64, ts: i64) {
        let msg = ServerMessage::Alarm {
            alarm_id,
            tag_id: tag_id.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            value,
            ts,
        };
        for entry in self.connections.iter() {
            let _ = entry.tx.send(msg.clone());
        }
    }

    pub fn broadcast_command_ack(&self, command_id: &str, status: &str, device_response: Option<String>, latency_ms: Option<u64>) {
        let msg = ServerMessage::CommandAck {
            command_id: command_id.to_string(),
            status: status.to_string(),
            device_response,
            latency_ms,
        };
        for entry in self.connections.iter() {
            let _ = entry.tx.send(msg.clone());
        }
    }

    pub fn broadcast_device_status(&self, device_id: &str, online: bool, latency_ms: Option<u64>) {
        let msg = ServerMessage::DeviceStatus {
            device_id: device_id.to_string(),
            online,
            latency_ms,
        };
        for entry in self.connections.iter() {
            let _ = entry.tx.send(msg.clone());
        }
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

use tokio::sync::mpsc::UnboundedReceiver;

pub fn spawn_ws_broadcaster(ws_manager: Arc<WsManager>, cache: crate::cache::DeviceCache) {
    tokio::spawn(async move {
        let mut rx = cache.subscribe_sse();
        loop {
            match rx.recv().await {
                Ok(payload_json) => {
                    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&payload_json) {
                        if let Some(sensors) = payload.get("sensors").and_then(|s| s.as_array()) {
                            for sensor in sensors {
                                if let (Some(tag_id), Some(value)) = (
                                    sensor.get("tag_name").and_then(|v| v.as_str()),
                                    sensor.get("current_value").and_then(|v| v.as_f64()),
                                ) {
                                    let quality = sensor.get("quality").and_then(|v| v.as_str()).unwrap_or("good");
                                    let ts = sensor.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
                                    let simulated = sensor.get("simulated").and_then(|v| v.as_bool()).unwrap_or(false);
                                    ws_manager.broadcast_tag_update(tag_id, value, quality, ts, simulated);
                                }
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Arc<WsManager>, String) {
        let mgr = Arc::new(WsManager::new());
        let conn_id = "test-conn-1".to_string();
        let rx = mgr.connect(conn_id.clone(), "user1".to_string(), vec!["tags:read".to_string(), "tags:write".to_string()]);
        (mgr, conn_id)
    }

    #[test]
    fn test_connect_disconnect() {
        let mgr = WsManager::new();
        let _rx = mgr.connect("c1".to_string(), "u1".to_string(), vec![]);
        assert_eq!(mgr.connection_count(), 1);
        mgr.disconnect("c1");
        assert_eq!(mgr.connection_count(), 0);
    }

    #[test]
    fn test_subscribe_unsubscribe() {
        let (mgr, conn_id) = setup();
        mgr.subscribe(&conn_id, vec!["tag1".to_string(), "tag2".to_string()]).unwrap();
        mgr.broadcast_tag_update("tag1", 42.0, "good", 0, false);

        mgr.unsubscribe(&conn_id, vec!["tag1".to_string()]).unwrap();
        // tag2 still subscribed, tag1 unsubscribed
        mgr.broadcast_tag_update("tag1", 99.0, "good", 0, false);
    }

    #[test]
    fn test_subscribe_all() {
        let (mgr, conn_id) = setup();
        mgr.subscribe_all(&conn_id).unwrap();
        mgr.broadcast_tag_update("any-tag", 1.0, "good", 0, false);
    }

    #[test]
    fn test_has_permission() {
        let (mgr, conn_id) = setup();
        assert!(mgr.has_permission(&conn_id, "tags:read"));
        assert!(mgr.has_permission(&conn_id, "tags:write"));
        assert!(!mgr.has_permission(&conn_id, "admin"));
    }

    #[test]
    fn test_broadcast_alarm() {
        let (mgr, _) = setup();
        mgr.broadcast_alarm(1, "tag1", "warning", "High value", 99.0, 0);
    }

    #[test]
    fn test_broadcast_command_ack() {
        let (mgr, _) = setup();
        mgr.broadcast_command_ack("cmd1", "acked", Some("OK".to_string()), Some(50));
    }

    #[test]
    fn test_broadcast_device_status() {
        let (mgr, _) = setup();
        mgr.broadcast_device_status("PLC01", true, Some(10));
    }

    #[test]
    fn test_disconnect_removes_connection() {
        let mgr = WsManager::new();
        mgr.connect("c1".to_string(), "u1".to_string(), vec![]);
        mgr.disconnect("c1");
        assert!(mgr.subscribe("c1", vec!["tag1".to_string()]).is_err());
    }
}
```

- [ ] **Step 2: Add `mod ws_manager;` to main.rs**

Add `pub mod ws_manager;` alongside other `mod` declarations.

- [ ] **Step 3: Run tests**

Run: `cargo test ws_manager`
Expected: all 8 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/ws_manager.rs src/main.rs
git commit -m "feat(ws): add WsManager connection tracker with subscription routing"
```

---

### Task 4: Implement WebSocket handler (api/websocket.rs)

**Files:**
- Create: `src/api/websocket.rs`
- Modify: `src/api/mod.rs` (register route)
- Modify: `src/main.rs` (initialize WsManager, spawn broadcaster)

This is the main integration task. The handler:

1. Upgrades HTTP to WebSocket
2. Waits for auth message (30s timeout)
3. Registers with WsManager
4. Main loop: receive ClientMessage, dispatch
5. Background task: forward ServerMessage from WsManager to WebSocket

- [ ] **Step 1: Create src/api/websocket.rs**

The handler function signature:

```rust
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use crate::api::AppState;
use crate::ws_types::{ClientMessage, ServerMessage};
use crate::ws_manager::WsManager;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}
```

The `handle_socket` function:
- Splits socket into sender/receiver
- Waits for first message, decodes as ClientMessage::Auth
- 30s timeout for auth
- Validates token using existing auth logic (reuse JWT validation from `src/auth/`)
- On success: register with WsManager, spawn send loop, run recv loop
- On failure: send AuthError, close

The recv loop:
- `Subscribe` → `ws_manager.subscribe()`
- `Unsubscribe` → `ws_manager.unsubscribe()`
- `TagWrite` → check `tags:write` permission, submit to CommandQueue
- `Ping` → send `Pong`
- Invalid message → log warning, skip

The send loop:
- Receive from `UnboundedReceiver<ServerMessage>`
- Encode with rmp_serde
- Send as binary WebSocket message
- On send error → break (connection dead)

Use `uuid::Uuid::new_v4().to_string()` for connection IDs. If `uuid` is not a dependency, use a simple counter or `format!("ws-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())`.

For auth validation, look at how the existing SSE endpoint validates tokens. The SSE endpoint requires auth middleware. For WS, since the first message is the auth token, we need to validate it manually. Look at `src/auth/` for the JWT validation function and call it directly with the token string.

For tag writes, the existing REST handler in `src/api/` creates commands via `CommandQueue`. The WS handler should call the same queue submission logic. Look at the `write_tag` handler to understand the flow.

- [ ] **Step 2: Register route in api/mod.rs**

Add `pub mod websocket;` to module declarations.

In `create_router()`, add the WS route:

```rust
.route("/api/v1/ws", get(websocket::websocket_handler))
```

This goes in the protected section alongside `/api/v1/stream`.

- [ ] **Step 3: Initialize WsManager in main.rs**

In `main()`:
```rust
let ws_manager = Arc::new(WsManager::new());
```

Add `ws_manager: Arc<WsManager>` to `AppState` struct in `api/mod.rs`.

Spawn the broadcaster:
```rust
ws_manager::spawn_ws_broadcaster(ws_manager.clone(), cache.clone());
```

- [ ] **Step 4: Update AppState to include ws_manager**

In `src/api/mod.rs`, add `ws_manager` field to `AppState`:

```rust
pub struct AppState {
    pub cache: crate::cache::DeviceCache,
    pub storage: crate::storage::Storage,
    pub ws_manager: Arc<crate::ws_manager::WsManager>,
    // ... existing fields
}
```

Update all places where `AppState::new()` is called to include `ws_manager`.

- [ ] **Step 5: Verify it compiles**

Run: `cargo check`
Expected: compiles without errors

- [ ] **Step 6: Commit**

```bash
git add src/api/websocket.rs src/api/mod.rs src/main.rs
git commit -m "feat(ws): add WebSocket handler with auth, subscriptions, and tag writes"
```

---

### Task 5: Hook WsManager into command queue and alarm system

**Files:**
- Modify: `src/command_queue.rs`
- Modify: `src/api/alarms.rs` or alarm evaluation code

- [ ] **Step 1: Broadcast command_ack from CommandQueue**

In `command_queue.rs`, wherever command status changes to `acked`, `failed`, or `rejected`, call:

```rust
if let Some(ws_manager) = &self.ws_manager {
    ws_manager.broadcast_command_ack(&command.id, &status_str, device_response, latency_ms);
}
```

Add `ws_manager: Option<Arc<WsManager>>` to `CommandQueue` struct. Update `CommandQueue::new()` to accept an optional `Arc<WsManager>`.

- [ ] **Step 2: Broadcast alarm events**

Find where alarm events are created/fired (look in `src/api/alarms.rs` or the alarm evaluation code). Add a call to `ws_manager.broadcast_alarm()` when a new alarm triggers.

This may require passing `WsManager` through the alarm system. The simplest approach: store `Arc<WsManager>` in a field accessible from the alarm evaluation context, similar to how `Storage` is passed around.

- [ ] **Step 3: Update PollerManager for device status**

In `poller_manager.rs`, when device connect/disconnect events happen, call `ws_manager.broadcast_device_status()`.

Add `ws_manager: Option<Arc<WsManager>>` to `PollerManager`. This follows the same pattern as the `storage` field added in v0.26.0.

- [ ] **Step 4: Verify it compiles**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add src/command_queue.rs src/api/alarms.rs src/poller_manager.rs
git commit -m "feat(ws): hook WsManager into command queue, alarms, and device status"
```

---

### Task 6: Frontend WebSocket client

**Files:**
- Create: `client/src/api/websocket.ts`

- [ ] **Step 1: Install @msgpack/msgpack**

Run: `cd client && npm install @msgpack/msgpack`

- [ ] **Step 2: Create websocket.ts client**

```typescript
import { encode, decode } from '@msgpack/msgpack';

export interface TagUpdate {
  tag_id: string;
  value: number;
  quality: string;
  ts: number;
  simulated: boolean;
}

export interface CommandAck {
  command_id: string;
  status: string;
  device_response?: string;
  latency_ms?: number;
}

export interface DeviceStatus {
  device_id: string;
  online: boolean;
  latency_ms?: number;
}

export interface AlarmEvent {
  alarm_id: number;
  tag_id: string;
  severity: string;
  message: string;
  value: number;
  ts: number;
}

type TagUpdateCallback = (update: TagUpdate) => void;
type CommandAckCallback = (ack: CommandAck) => void;
type AlarmCallback = (alarm: AlarmEvent) => void;
type DeviceStatusCallback = (status: DeviceStatus) => void;
type DisconnectCallback = () => void;

export class VoltrusWebSocket {
  private ws: WebSocket | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectDelay = 1000;
  private maxReconnectDelay = 30000;
  private tagUpdateCallbacks: TagUpdateCallback[] = [];
  private commandAckCallbacks: CommandAckCallback[] = [];
  private alarmCallbacks: AlarmCallback[] = [];
  private deviceStatusCallbacks: DeviceStatusCallback[] = [];
  private disconnectCallbacks: DisconnectCallback[] = [];
  private url = '';
  private token = '';

  connect(url: string, token: string): void {
    this.url = url;
    this.token = token;
    this.reconnectDelay = 1000;
    this.doConnect();
  }

  private doConnect(): void {
    if (this.ws) {
      this.ws.close();
    }
    this.ws = new WebSocket(this.url);
    this.ws.binaryType = 'arraybuffer';

    this.ws.onopen = () => {
      this.reconnectDelay = 1000;
      const authMsg = encode({ type: 'auth', token: this.token });
      this.ws?.send(authMsg);
    };

    this.ws.onmessage = (event) => {
      const data = event.data as ArrayBuffer;
      const msg = decode(data) as Record<string, unknown>;
      this.handleMessage(msg);
    };

    this.ws.onclose = () => {
      this.disconnectCallbacks.forEach(cb => cb());
      this.scheduleReconnect();
    };

    this.ws.onerror = () => {
      // onclose will fire after this
    };
  }

  private handleMessage(msg: Record<string, unknown>): void {
    switch (msg.type) {
      case 'tag_update':
        this.tagUpdateCallbacks.forEach(cb => cb(msg as unknown as TagUpdate));
        break;
      case 'command_ack':
        this.commandAckCallbacks.forEach(cb => cb(msg as unknown as CommandAck));
        break;
      case 'alarm':
        this.alarmCallbacks.forEach(cb => cb(msg as unknown as AlarmEvent));
        break;
      case 'device_status':
        this.deviceStatusCallbacks.forEach(cb => cb(msg as unknown as DeviceStatus));
        break;
    }
  }

  subscribe(tagIds: string[]): void {
    this.send({ type: 'subscribe', tag_ids: tagIds });
  }

  unsubscribe(tagIds: string[]): void {
    this.send({ type: 'unsubscribe', tag_ids: tagIds });
  }

  writeTag(tagId: string, value: number): string {
    const commandId = `ws-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    this.send({ type: 'tag_write', tag_id: tagId, value, command_id: commandId });
    return commandId;
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
  }

  onTagUpdate(cb: TagUpdateCallback): void { this.tagUpdateCallbacks.push(cb); }
  onCommandAck(cb: CommandAckCallback): void { this.commandAckCallbacks.push(cb); }
  onAlarm(cb: AlarmCallback): void { this.alarmCallbacks.push(cb); }
  onDeviceStatus(cb: DeviceStatusCallback): void { this.deviceStatusCallbacks.push(cb); }
  onDisconnect(cb: DisconnectCallback): void { this.disconnectCallbacks.push(cb); }

  private send(msg: Record<string, unknown>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(encode(msg));
    }
  }

  private scheduleReconnect(): void {
    if (!this.url) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
      this.doConnect();
    }, this.reconnectDelay);
  }
}
```

- [ ] **Step 3: Add TypeScript types to client/src/types.ts**

Add the WebSocket message interfaces to the existing types file:

```typescript
export interface WsTagUpdate {
  tag_id: string;
  value: number;
  quality: string;
  ts: number;
  simulated: boolean;
}

export interface WsCommandAck {
  command_id: string;
  status: string;
  device_response?: string;
  latency_ms?: number;
}

export interface WsDeviceStatus {
  device_id: string;
  online: boolean;
  latency_ms?: number;
}

export interface WsAlarmEvent {
  alarm_id: number;
  tag_id: string;
  severity: string;
  message: string;
  value: number;
  ts: number;
}
```

- [ ] **Step 4: Verify TypeScript compiles**

Run: `cd client && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add client/src/api/websocket.ts client/src/types.ts client/package.json client/package-lock.json
git commit -m "feat(frontend): add WebSocket client with MessagePack and auto-reconnect"
```

---

### Task 7: Integrate WebSocket into App.tsx

**Files:**
- Modify: `client/src/App.tsx`

- [ ] **Step 1: Initialize VoltrusWebSocket in App.tsx**

Add WebSocket initialization alongside existing SSE setup. The WS client should be created when the user is authenticated and has a paid-tier license. Store it in React state or a ref.

```typescript
import { VoltrusWebSocket } from './api/websocket';

// In the component:
const wsRef = useRef<VoltrusWebSocket | null>(null);

useEffect(() => {
  if (isAuthenticated && !isFreeTrial) {
    const ws = new VoltrusWebSocket();
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws.connect(`${protocol}//${window.location.host}/api/v1/ws`, authToken);
    wsRef.current = ws;
    return () => ws.disconnect();
  }
}, [isAuthenticated, isFreeTrial]);
```

The WS client is available for components to use via ref or context. No components are changed to use it yet — that's future work.

- [ ] **Step 2: Verify TypeScript compiles**

Run: `cd client && npx tsc --noEmit`

- [ ] **Step 3: Commit**

```bash
git add client/src/App.tsx
git commit -m "feat(frontend): initialize WebSocket connection in App.tsx"
```

---

### Task 8: Integration tests and final verification

**Files:**
- Modify: `src/api/websocket.rs` (add integration tests)

- [ ] **Step 1: Add integration tests for WebSocket handler**

Add tests using `tokio-tungstenite` (dev-dependency) or Axum's test utilities. Test:

1. **Auth flow**: Connect, send auth with valid token → receive auth_ok
2. **Auth failure**: Connect, send invalid token → connection closes
3. **Auth timeout**: Connect, wait 31s without auth → connection closes
4. **Subscribe and receive**: Auth, subscribe to tag, verify tag_update received
5. **Write command**: Auth, send tag_write, verify command_ack received

Add `tokio-tungstenite` to `[dev-dependencies]` in Cargo.toml:

```toml
tokio-tungstenite = "=0.26.0"
```

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: all existing tests + new tests pass

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --all-targets`
Expected: no errors

- [ ] **Step 4: Verify frontend compiles**

Run: `cd client && npx tsc --noEmit`
Expected: clean

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "test(ws): add WebSocket integration tests and final verification"
```

---

### Task 9: Update landing page changelog

**Files:**
- Modify: `voltrus-landing/changelog.html` (add v0.29.0 entry)

- [ ] **Step 1: Add changelog entry for v0.29.0**

Add entry at the top of the changelog:

```html
<h3>v0.29.0 — WebSocket Bidirectional Streaming</h3>
<p>Real-time bidirectional communication via WebSocket with MessagePack binary encoding.</p>
<ul>
  <li>WebSocket endpoint alongside REST+SSE for high-frequency tag streams</li>
  <li>MessagePack binary format for efficient data transfer</li>
  <li>Per-connection tag subscriptions (no firehose)</li>
  <li>Tag writes over WebSocket through existing command queue</li>
  <li>Real-time command acknowledgement, device status, and alarm events</li>
  <li>Auto-reconnect with exponential backoff</li>
  <li>SSE remains available for backward compatibility</li>
</ul>
```

- [ ] **Step 2: Commit**

```bash
git add voltrus-landing/changelog.html
git commit -m "docs: add v0.29.0 changelog entry"
```
