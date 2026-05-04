# v0.29.0 — WebSocket Bidirectional Streaming

**Date:** 2026-05-03
**Status:** Approved
**Version:** 0.29.0
**Timeline:** 4-5 weeks
**Tier:** Starter ($249)

---

## Goal

True bidirectional real-time for high-frequency control loops and state sync. WebSocket endpoint alongside existing REST+SSE. MessagePack binary format for efficiency. Browser-native, no proxy required.

---

## Architecture

**Endpoint:** `ws://host:port/api/v1/ws` — single WebSocket endpoint alongside existing REST+SSE (not replacing).

**Protocol:** MessagePack binary framing. Each WebSocket message is a MessagePack-encoded map with a `type` field.

**Coexistence:** SSE stays for backward compatibility and Free Trial users. WebSocket is opt-in for paid tiers. WebSocket is a **superset** of SSE: all SSE events available via WS, plus bidirectional writes.

**Authentication:** First client message must be `{"type": "auth", "token": "..."}` with JWT or API key. Server responds with `auth_ok` or `auth_error`. Connection closes after 30s if no valid auth received. No query-param auth (tokens in URLs get logged in proxies/access logs).

**Single new dependency:** `rmp-serde` crate (MessagePack serialization for Rust). Frontend: `@msgpack/msgpack` npm package.

---

## Message Types

### Client → Server

| Type | Fields | Description |
|------|--------|-------------|
| `auth` | `token: String` | Authenticate connection (required first message) |
| `subscribe` | `tag_ids: Vec<String>` | Subscribe to tag updates |
| `unsubscribe` | `tag_ids: Vec<String>` | Unsubscribe from tag updates |
| `tag_write` | `tag_id, value, command_id?` | Write value to tag |
| `ping` | (none) | Keep-alive ping |

### Server → Client

| Type | Fields | Description |
|------|--------|-------------|
| `auth_ok` | `user, permissions` | Auth succeeded |
| `auth_error` | `message` | Auth failed (connection closes) |
| `tag_update` | `tag_id, value, quality, ts, simulated` | Real-time tag value |
| `command_ack` | `command_id, status, device_response?, latency_ms?` | Command acknowledgement |
| `device_status` | `device_id, online, latency_ms?` | Device online/offline |
| `alarm` | `alarm_id, tag_id, severity, message, value, ts` | Alarm event |
| `pong` | (none) | Keep-alive response |

---

## Backend Components

### New file: `src/ws_types.rs`

MessagePack message type definitions.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Auth { token: String },
    Subscribe { tag_ids: Vec<String> },
    Unsubscribe { tag_ids: Vec<String> },
    TagWrite { tag_id: String, value: f64, command_id: Option<String> },
    Ping,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    AuthOk { user: String, permissions: Vec<String> },
    AuthError { message: String },
    TagUpdate { tag_id: String, value: f64, quality: String, ts: i64, simulated: bool },
    CommandAck { command_id: String, status: String, device_response: Option<String>, latency_ms: Option<u64> },
    DeviceStatus { device_id: String, online: bool, latency_ms: Option<u64> },
    Alarm { alarm_id: i64, tag_id: String, severity: String, message: String, value: f64, ts: i64 },
    Pong,
}
```

### New file: `src/ws_manager.rs`

`WsManager` — thread-safe connection manager tracking subscriptions and routing messages.

```rust
pub struct WsManager {
    connections: DashMap<String, WsConnection>,
}

struct WsConnection {
    user_id: String,
    permissions: Vec<String>,
    subscribed_tags: HashSet<String>,
    tx: UnboundedSender<ServerMessage>,
}
```

Methods:
- `connect(id, user_id, permissions, tx)` — register new connection
- `disconnect(id)` — remove connection, cleanup subscriptions
- `subscribe(conn_id, tag_ids)` — add tags to connection's subscription set
- `unsubscribe(conn_id, tag_ids)` — remove tags from subscription set
- `broadcast_tag_update(tag_id, value, quality, ts, simulated)` — send to all connections subscribed to this tag
- `broadcast_alarm(alarm)` — send to all connections
- `broadcast_command_ack(command_id, status, ...)` — send to all connections
- `broadcast_device_status(device_id, online, latency_ms)` — send to all connections

Integration: Spawns a background task that listens to the existing `DeviceCache` broadcast channel and calls `broadcast_tag_update` for each cache update. Also hooks into alarm events and command queue status changes.

### New file: `src/api/websocket.rs`

Axum WebSocket upgrade handler.

**Connection lifecycle:**
1. Client sends HTTP upgrade request to `/api/v1/ws`
2. Server upgrades to WebSocket via `axum::extract::ws::WebSocketUpgrade`
3. Wait for first message (30s timeout) — must be `auth`
4. Validate JWT/API key using existing auth logic
5. Register connection with `WsManager`
6. Main loop: receive `ClientMessage` → dispatch
   - `Subscribe` → `ws_manager.subscribe()`
   - `Unsubscribe` → `ws_manager.unsubscribe()`
   - `TagWrite` → validate permissions, submit to `CommandQueue`
   - `Ping` → send `Pong`
7. On disconnect → `ws_manager.disconnect()`

**Tag writes through WS:**
- Same validation as REST `POST /api/v1/tags/{id}/write`
- Goes through `CommandQueue` — same audit trail, same retry logic
- Response comes as `command_ack` server message (async, not request-response)

### Route registration

Add to `src/api/mod.rs` alongside existing routes:

```rust
let ws = ws::websocket_handler(state.clone());
let app = Router::new()
    // ... existing routes ...
    .route("/api/v1/ws", get(ws))
    // ...
```

No license-gate for WebSocket itself — license check happens at auth time via existing JWT validation.

---

## Frontend

### New dependency

`@msgpack/msgpack` — browser-native MessagePack for JavaScript.

### New file: `client/src/api/websocket.ts`

```typescript
class VoltrusWebSocket {
  private ws: WebSocket | null;
  private msgpack: typeof import('@msgpack/msgpack');

  connect(url: string, token: string): void
  subscribe(tagIds: string[]): void
  unsubscribe(tagIds: string[]): void
  writeTag(tagId: string, value: number): string  // returns command_id
  disconnect(): void

  // Event callbacks
  onTagUpdate(cb: (u: TagUpdate) => void): void
  onCommandAck(cb: (a: CommandAck) => void): void
  onAlarm(cb: (a: AlarmEvent) => void): void
  onDeviceStatus(cb: (s: DeviceStatus) => void): void
  onDisconnect(cb: () => void): void
}
```

Auto-reconnect with exponential backoff (1s → 2s → 4s → max 30s). Re-authenticates on reconnect.

### Integration

- Existing SSE consumers (SensorCard, Chart, alarm widgets) don't change
- `App.tsx` initializes `VoltrusWebSocket` when user has a paid-tier license
- Components needing bidirectional control use the WS client for writes
- Components needing high-frequency updates can opt into WS subscriptions
- SSE remains the default for Free Trial users

No new UI components — WebSocket is an infrastructure upgrade.

---

## Error Handling

| Scenario | Behavior |
|----------|----------|
| No auth within 30s | Send `auth_error`, close connection |
| Invalid message format | Send `error` message, keep connection open |
| Write permission denied | Send `command_ack` with `status: "rejected"` |
| Write validation failure | Send `command_ack` with `status: "rejected"` and reason |
| Client too slow (backpressure) | Buffer 1000 messages, then disconnect |
| Connection drops | `ws_manager` auto-cleans subscriptions |
| Server restart | Clients auto-reconnect with backoff |

---

## Permissions

No new permission constants. WebSocket uses existing auth:
- `subscribe`/`unsubscribe` requires authenticated user (any role)
- `tag_write` requires `tags:write` permission (same as REST)
- `tag_update` delivery respects subscription filter only (no per-message RBAC)
- `alarm` and `device_status` broadcast to all authenticated connections

---

## Testing

- Unit tests: MessagePack encode/decode round-trip for all message types
- Unit tests: `WsManager` subscribe/unsubscribe/broadcast logic
- Unit tests: Auth timeout (connection closed after 30s)
- Integration test: Full WS lifecycle — connect, auth, subscribe, receive update, write, ack, disconnect
- Integration test: Multiple concurrent connections with different subscriptions
- Load test: 100 concurrent connections receiving 10K tag updates/sec

---

## Files to Modify/Create

### Rust backend

| File | Change |
|------|--------|
| `Cargo.toml` | Add `rmp-serde` dependency, bump version to 0.29.0 |
| `src/ws_types.rs` | New — ClientMessage and ServerMessage enums |
| `src/ws_manager.rs` | New — WsManager connection tracker and broadcaster |
| `src/api/websocket.rs` | New — WebSocket upgrade handler and message loop |
| `src/api/mod.rs` | Register WS route |
| `src/main.rs` | Add `mod ws_types; mod ws_manager;`, initialize WsManager |
| `src/cache.rs` | Add method to hook WsManager into cache broadcast |

### Frontend

| File | Change |
|------|--------|
| `client/package.json` | Add `@msgpack/msgpack` dependency |
| `client/src/api/websocket.ts` | New — VoltrusWebSocket class |
| `client/src/App.tsx` | Initialize VoltrusWebSocket alongside SSE |
| `client/src/types.ts` | Add WebSocket message type interfaces |

---

## What's NOT in scope

- gRPC or gRPC-Web (roadmap says no — contradicts single-binary philosophy)
- Per-message compression (can add later if needed)
- WebSocket-only mode (SSE always available)
- Binary protocol for REST endpoints (REST stays JSON)
- Server-to-server WebSocket (only browser clients)
