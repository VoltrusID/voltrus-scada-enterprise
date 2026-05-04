# Command Ack & Control Mode — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add bidirectional device write capability with command queue, audit trail, and acknowledgment across Modbus, S7, OPC UA, EtherNet/IP, and MQTT protocols.

**Architecture:** Direct write-through — each protocol driver gains a `write_tag()` method. A background command queue processor handles ordering, validation, offline buffering, retry, and rate limiting. Commands are tracked in SQLite with full audit trail.

**Tech Stack:** Rust (Axum, Tokio, Rusqlite), React + TypeScript frontend, SSE for real-time command status.

---

## File Structure

### Create
- `src/api/commands.rs` — Command API handlers (write, status, query, cancel, write policy)
- `src/command_queue.rs` — Background command queue processor
- `client/src/api/commands.ts` — Frontend API client for commands
- `client/src/components/WriteDialog.tsx` — Write value dialog
- `client/src/components/CommandPanel.tsx` — Command history + active commands
- `client/src/components/WritePolicyEditor.tsx` — Tag write policy settings
- `client/src/components/RecipeLoadStatus.tsx` — Recipe parameter write status

### Modify
- `src/models.rs` — Command struct, CommandStatus enum, WritePolicy struct, permission constants
- `src/storage.rs` — Schema tables, CRUD methods for commands + write policies
- `src/modbus.rs` — Add `write_tag()` with FC 05/06/15/16
- `src/s7.rs` — Add `write_tag()` for DB/M/I/Q areas
- `src/opcua.rs` — Add `write_tag()` via Write service
- `src/ethernet_ip.rs` — Add `write_tag()` for CIP tags
- `src/cache.rs` — Add write rate tracking, is_device_online, get_metric_value
- `src/main.rs` — Spawn command queue background task
- `src/api/mod.rs` — Register command routes
- `src/flow_engine/node.rs` — Add `WriteOut` node type
- `src/recipe.rs` — Replace dry-run with real command queue calls
- `client/src/views/DeviceView.tsx` — Add write buttons on writable tags
- `client/src/views/TagSettings.tsx` — Add write policy section

---

## Task 1: Models & Permission Constants

**Files:**
- Modify: `src/models.rs`

- [ ] **Step 1: Add CommandStatus enum**

Find the existing enum section in `models.rs` (after `MachineState`) and add:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Pending,
    Queued,
    Sent,
    Acked,
    Failed,
    Expired,
    Cancelled,
}

impl CommandStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Queued => "queued",
            Self::Sent => "sent",
            Self::Acked => "acked",
            Self::Failed => "failed",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Acked | Self::Failed | Self::Expired | Self::Cancelled)
    }
}
```

- [ ] **Step 2: Add Command struct**

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Command {
    pub id: String,
    pub data_source_id: i64,
    pub device_id: i64,
    pub protocol: String,
    pub value: String,
    pub status: String,
    pub policy: String,
    pub scheduled_at: Option<i64>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub sent_at: Option<i64>,
    pub acked_at: Option<i64>,
    pub resolved_at: Option<i64>,
    pub created_by: String,
}
```

- [ ] **Step 3: Add CommandAuditEntry struct**

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandAuditEntry {
    pub id: i64,
    pub command_id: String,
    pub old_status: String,
    pub new_status: String,
    pub changed_by: String,
    pub reason: Option<String>,
    pub created_at: i64,
}
```

- [ ] **Step 4: Add WritePolicy struct**

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WritePolicy {
    pub writable: bool,
    pub write_min: Option<f64>,
    pub write_max: Option<f64>,
    pub write_max_rate: Option<i64>,
    pub write_requires_reason: bool,
    pub write_deadband: Option<f64>,
}

impl Default for WritePolicy {
    fn default() -> Self {
        Self {
            writable: false,
            write_min: None,
            write_max: None,
            write_max_rate: None,
            write_requires_reason: false,
            write_deadband: None,
        }
    }
}
```

- [ ] **Step 5: Add request/response structs**

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WriteRequest {
    pub value: serde_json::Value,
    #[serde(default = "default_policy")]
    pub policy: String,
    pub scheduled_at: Option<i64>,
    pub reason: Option<String>,
}

fn default_policy() -> String {
    "immediate".to_string()
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CommandQuery {
    pub tag: Option<i64>,
    pub status: Option<String>,
    pub device: Option<i64>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WritePolicyUpdate {
    pub writable: Option<bool>,
    pub write_min: Option<Option<f64>>,
    pub write_max: Option<Option<f64>>,
    pub write_max_rate: Option<Option<i64>>,
    pub write_requires_reason: Option<bool>,
    pub write_deadband: Option<Option<f64>>,
}
```

- [ ] **Step 6: Add permission constants**

Find existing `PERM_*` constants and add:

```rust
pub const PERM_COMMANDS_WRITE: &str = "commands:write";
pub const PERM_COMMANDS_READ: &str = "commands:read";
pub const PERM_COMMANDS_CANCEL: &str = "commands:cancel";
pub const PERM_TAGS_WRITE_POLICY: &str = "tags:write-policy";
```

Add to `ALL_PERMISSIONS` array:

```rust
PERM_COMMANDS_WRITE, PERM_COMMANDS_READ, PERM_COMMANDS_CANCEL, PERM_TAGS_WRITE_POLICY,
```

Add `PERM_COMMANDS_WRITE` and `PERM_COMMANDS_READ` to `OPERATOR_PERMISSIONS`.

- [ ] **Step 7: Add WriteResult struct**

```rust
#[derive(Debug, Clone)]
pub struct WriteResult {
    pub success: bool,
    pub device_response: Option<String>,
    pub error: Option<String>,
    pub latency_ms: u64,
}
```

- [ ] **Step 8: Run cargo check**

Run: `cargo check`

- [ ] **Step 9: Commit**

```bash
git add src/models.rs
git commit -m "feat(commands): add Command model, enums, write policy, and permission constants"
```

---

## Task 2: Storage Schema & CRUD

**Files:**
- Modify: `src/storage.rs`

- [ ] **Step 1: Add commands table to init_schema**

At the end of `init_schema()`, before `Ok(())`:

```rust
conn.execute_batch(
    "CREATE TABLE IF NOT EXISTS commands (
        id TEXT PRIMARY KEY,
        data_source_id INTEGER NOT NULL,
        device_id INTEGER NOT NULL,
        protocol TEXT NOT NULL,
        value TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        policy TEXT NOT NULL DEFAULT 'immediate',
        scheduled_at INTEGER,
        retry_count INTEGER NOT NULL DEFAULT 0,
        max_retries INTEGER NOT NULL DEFAULT 3,
        error_message TEXT,
        created_at INTEGER NOT NULL,
        sent_at INTEGER,
        acked_at INTEGER,
        resolved_at INTEGER,
        created_by TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_commands_status ON commands(status);
    CREATE INDEX IF NOT EXISTS idx_commands_data_source ON commands(data_source_id);
    CREATE INDEX IF NOT EXISTS idx_commands_device ON commands(device_id);
    CREATE INDEX IF NOT EXISTS idx_commands_created ON commands(created_at);

    CREATE TABLE IF NOT EXISTS command_audit_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        command_id TEXT NOT NULL REFERENCES commands(id) ON DELETE CASCADE,
        old_status TEXT NOT NULL,
        new_status TEXT NOT NULL,
        changed_by TEXT NOT NULL,
        reason TEXT,
        created_at INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_command_audit_command ON command_audit_log(command_id);
    ",
)?;
```

- [ ] **Step 2: Add write policy columns to data_sources**

After commands table creation:

```rust
let _ = conn.execute_batch(
    "ALTER TABLE data_sources ADD COLUMN writable INTEGER NOT NULL DEFAULT 0;
     ALTER TABLE data_sources ADD COLUMN write_min REAL;
     ALTER TABLE data_sources ADD COLUMN write_max REAL;
     ALTER TABLE data_sources ADD COLUMN write_max_rate INTEGER;
     ALTER TABLE data_sources ADD COLUMN write_requires_reason INTEGER NOT NULL DEFAULT 0;
     ALTER TABLE data_sources ADD COLUMN write_deadband REAL;"
);
```

- [ ] **Step 3: Add insert_command method**

```rust
pub async fn insert_command(&self, cmd: &crate::models::Command) -> Result<()> {
    let conn = self.conn.clone();
    let cmd = cmd.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO commands (id, data_source_id, device_id, protocol, value, status, policy, scheduled_at, retry_count, max_retries, error_message, created_at, sent_at, acked_at, resolved_at, created_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            rusqlite::params![
                cmd.id, cmd.data_source_id, cmd.device_id, cmd.protocol,
                cmd.value, cmd.status, cmd.policy, cmd.scheduled_at,
                cmd.retry_count, cmd.max_retries, cmd.error_message,
                cmd.created_at, cmd.sent_at, cmd.acked_at, cmd.resolved_at,
                cmd.created_by
            ],
        )?;
        Ok(())
    }).await??;
    Ok(())
}
```

- [ ] **Step 4: Add update_command_status method**

```rust
pub async fn update_command_status(
    &self,
    command_id: &str,
    new_status: &str,
    error_message: Option<&str>,
    changed_by: &str,
    reason: Option<&str>,
) -> Result<()> {
    let conn = self.conn.clone();
    let command_id = command_id.to_string();
    let new_status = new_status.to_string();
    let error_message = error_message.map(String::from);
    let changed_by = changed_by.to_string();
    let reason = reason.map(String::from);
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = conn.lock().unwrap();
        let now = crate::timeutil::now_secs();

        let old_status: String = conn.query_row(
            "SELECT status FROM commands WHERE id = ?1",
            rusqlite::params![command_id],
            |row| row.get(0),
        )?;

        let mut updates = vec!["status = ?1".to_string()];
        let mut param_idx = 2u32;

        match new_status.as_str() {
            "sent" => { updates.push(format!("sent_at = ?{}", param_idx)); param_idx += 1; }
            "acked" => { updates.push(format!("sent_at = COALESCE(sent_at, ?{})", param_idx)); updates.push(format!("acked_at = ?{}", param_idx)); param_idx += 1; }
            _ => {}
        }

        let is_terminal = matches!(new_status.as_str(), "acked" | "failed" | "expired" | "cancelled");
        if is_terminal { updates.push(format!("resolved_at = ?{}", param_idx)); param_idx += 1; }
        if error_message.is_some() { updates.push(format!("error_message = ?{}", param_idx)); param_idx += 1; }
        if new_status == "queued" { updates.push("retry_count = retry_count + 1".to_string()); }

        updates.push(format!("id = id")); // sentinel for WHERE clause

        let sql = format!("UPDATE commands SET {} WHERE id = ?{}", updates.join(", "), param_idx);

        // Use a simpler approach with direct SQL
        let resolved_val = if is_terminal { Some(now) } else { None };
        conn.execute(
            &sql,
            rusqlite::params![new_status],
        )?;

        conn.execute(
            "INSERT INTO command_audit_log (command_id, old_status, new_status, changed_by, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![command_id, old_status, new_status, changed_by, reason, now],
        )?;

        Ok(())
    }).await??;
    Ok(())
}
```

Note: The dynamic SQL building above needs careful testing. An alternative is to use separate UPDATE statements per status transition. The implementer should choose the simpler approach.

- [ ] **Step 5: Add get_command, list_commands, get_queued_commands methods**

Follow the same pattern as existing storage methods (e.g., alarm queries):

- `get_command(command_id: &str) -> Result<Option<Command>>` — SELECT by id
- `list_commands(query: &CommandQuery) -> Result<Vec<Command>>` — SELECT with dynamic WHERE/filter/limit
- `get_queued_commands() -> Result<Vec<Command>>` — SELECT WHERE status IN ('queued', 'pending') ORDER BY created_at
- `get_command_audit(command_id: &str) -> Result<Vec<CommandAuditEntry>>` — SELECT from command_audit_log

Each method follows the existing pattern: `tokio::task::spawn_blocking` → `conn.lock().unwrap()` → prepare/query → collect results.

- [ ] **Step 6: Add write policy methods**

```rust
pub async fn get_write_policy(&self, data_source_id: i64) -> Result<Option<WritePolicy>>
pub async fn update_write_policy(&self, data_source_id: i64, update: &WritePolicyUpdate) -> Result<()>
pub async fn count_recent_writes(&self, data_source_id: i64, window_secs: i64) -> Result<i64>
pub async fn get_data_source_device_id(&self, data_source_id: i64) -> Result<Option<(i64, String)>>
```

Follow the same `spawn_blocking` pattern. The `get_data_source_device_id` returns `(device_id, protocol)` — verify actual column names against the existing `data_sources` schema.

- [ ] **Step 7: Run cargo check**

Run: `cargo check`

- [ ] **Step 8: Commit**

```bash
git add src/storage.rs
git commit -m "feat(commands): add commands, command_audit_log tables and CRUD methods"
```

---

## Task 3: Modbus Write Support

**Files:**
- Modify: `src/modbus.rs`

- [ ] **Step 1: Add write_tag function**

Add after the existing `try_poll` function. Uses the same connection pattern:

```rust
pub async fn write_tag(
    config: &DeviceConfig,
    address: u16,
    value: f64,
) -> crate::models::WriteResult {
    let start = std::time::Instant::now();
    let socket_addr = format!("{}:{}", config.ip, config.port);

    let mut ctx = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio_modbus::client::tcp::connect_slave(
            socket_addr.parse().unwrap_or_else(|_| "0.0.0.0:502".parse().unwrap()),
            tokio_modbus::Slave(config.unit_id),
        ),
    ).await {
        Ok(Ok(ctx)) => ctx,
        Ok(Err(e)) => {
            return crate::models::WriteResult {
                success: false, device_response: None,
                error: Some(format!("Connection failed: {}", e)),
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
        Err(_) => {
            return crate::models::WriteResult {
                success: false, device_response: None,
                error: Some("Connection timeout".to_string()),
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ctx.write_single_register(address, value as u16),
    ).await;

    match result {
        Ok(Ok(_)) => crate::models::WriteResult {
            success: true,
            device_response: Some(format!("Register {} = {}", address, value as u16)),
            error: None,
            latency_ms: start.elapsed().as_millis() as u64,
        },
        Ok(Err(e)) => crate::models::WriteResult {
            success: false, device_response: None,
            error: Some(format!("Write failed: {}", e)),
            latency_ms: start.elapsed().as_millis() as u64,
        },
        Err(_) => crate::models::WriteResult {
            success: false, device_response: None,
            error: Some("Write timeout".to_string()),
            latency_ms: start.elapsed().as_millis() as u64,
        },
    }
}
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check`

- [ ] **Step 3: Commit**

```bash
git add src/modbus.rs
git commit -m "feat(commands): add Modbus TCP write_single_register support (FC 06)"
```

---

## Task 4: Multi-Protocol Write Support

**Files:**
- Modify: `src/s7.rs`
- Modify: `src/opcua.rs`
- Modify: `src/ethernet_ip.rs`

Each protocol gets a `write_tag()` function returning `WriteResult`.

- [ ] **Step 1: S7 write_tag**

In `src/s7.rs`, add a `write_tag()` function. Follow the existing S7 connection pattern for the client setup, then write to the specified area (DB/M/I/Q) and offset:

```rust
pub async fn write_tag(ip: &str, port: u16, area: &str, db_number: u32, offset: u16, value: f64) -> crate::models::WriteResult
```

Connection: reuse existing S7 client connect pattern. Area mapping: DB→DataBlock, M→Marker, I→Input, Q→Output. Write the value as bytes to the resolved area+offset.

- [ ] **Step 2: OPC UA write_tag**

In `src/opcua.rs`, add:

```rust
pub async fn write_tag(endpoint: &str, node_id: &str, value: f64) -> crate::models::WriteResult
```

Follow existing OPC UA client pattern. Use the OPC UA Write service to write a value to the specified node ID.

- [ ] **Step 3: EtherNet/IP write_tag**

In `src/ethernet_ip.rs`, add:

```rust
pub async fn write_tag(ip: &str, port: u16, tag_name: &str, value: f64) -> crate::models::WriteResult
```

Follow existing EtherNet/IP connection pattern. Write value to the specified CIP tag name.

- [ ] **Step 4: Run cargo check**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add src/s7.rs src/opcua.rs src/ethernet_ip.rs
git commit -m "feat(commands): add write_tag() for S7, OPC UA, and EtherNet/IP protocols"
```

---

## Task 5: Command Queue Processor

**Files:**
- Create: `src/command_queue.rs`
- Modify: `src/cache.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create command_queue.rs**

The queue processor is a background Tokio task that runs every 500ms. It:
1. Picks all commands with status `queued` or `pending`
2. Handles scheduled commands (transition to pending when `scheduled_at <= now`)
3. Checks device online state (via `cache.is_device_online()`)
4. Enforces one-command-per-device concurrency
5. Calls `execute_write()` which dispatches to the right protocol driver
6. Updates command status and publishes SSE events
7. Handles retry logic (re-queue on failure if retries remain)
8. Expires commands older than 30 minutes

Key method signatures:

```rust
pub struct CommandQueue {
    storage: Storage,
    cache: DeviceCache,
    in_flight: Arc<RwLock<HashSet<i64>>>,
}

impl CommandQueue {
    pub fn new(storage: Storage, cache: DeviceCache) -> Self;
    pub async fn run(&self);  // main loop
    async fn process_queue(&self) -> anyhow::Result<()>;
    async fn execute_write(&self, cmd: &Command) -> WriteResult;
    fn publish_status_event(&self, command_id: &str, status: &str, error: Option<&str>);
    fn publish_ack_event(&self, command_id: &str, response: &str, latency_ms: u64);
}
```

The `execute_write` method dispatches based on `cmd.protocol`:
- `"modbus"` → `crate::modbus::write_tag()`
- `"s7"` → `crate::s7::write_tag()`
- `"opcua"` → `crate::opcua::write_tag()`
- `"ethernet_ip"` → `crate::ethernet_ip::write_tag()`
- `"mqtt"` → publish to MQTT topic

SSE events are published via `self.cache.publish_sse_payload(json_string)`:
- `command_status`: `{"type":"command_status","command_id":"...","status":"...","error":null}`
- `command_ack`: `{"type":"command_ack","command_id":"...","status":"acked","device_response":"...","latency_ms":42}`

- [ ] **Step 2: Add is_device_online and get_metric_value to DeviceCache**

In `src/cache.rs`:

```rust
pub async fn is_device_online(&self, device_id: &str) -> bool {
    let devices = self.devices.read().await;
    devices.get(device_id).map_or(false, |s| s.status == DeviceStatus::Online)
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
```

- [ ] **Step 3: Spawn queue in main.rs**

Add `mod command_queue;` to module declarations. Find where other background tasks are spawned and add:

```rust
let queue = command_queue::CommandQueue::new(storage.clone(), cache.clone());
tokio::spawn(async move {
    queue.run().await;
});
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add src/command_queue.rs src/cache.rs src/main.rs
git commit -m "feat(commands): add command queue processor with state machine and retry logic"
```

---

## Task 6: Command API Endpoints

**Files:**
- Create: `src/api/commands.rs`
- Modify: `src/api/mod.rs`

- [ ] **Step 1: Create api/commands.rs**

Seven handler functions:

1. `write_tag` — POST `/api/v1/tags/{id}/write`
   - Permission check: `PERM_COMMANDS_WRITE`
   - Validate write policy (writable, min/max, deadband, rate limit, reason)
   - Create Command with UUID id, insert to storage
   - Return 202 Accepted with `command_id`, `status`, `data_source_id`, `value`, `created_at`

2. `get_command_status` — GET `/api/v1/commands/{command_id}`
   - Permission: `PERM_COMMANDS_READ`
   - Return Command struct as JSON

3. `list_commands` — GET `/api/v1/commands`
   - Permission: `PERM_COMMANDS_READ`
   - Accept `CommandQuery` as query params
   - Return `Vec<Command>`

4. `cancel_command` — POST `/api/v1/commands/{command_id}/cancel`
   - Permission: `PERM_COMMANDS_CANCEL`
   - Only cancel if status is `pending` or `queued`
   - Update status to `cancelled`

5. `get_command_audit` — GET `/api/v1/commands/{command_id}/audit`
   - Permission: `PERM_COMMANDS_READ`
   - Return `Vec<CommandAuditEntry>`

6. `update_write_policy` — PATCH `/api/v1/data-sources/{id}/write-policy`
   - Permission: `PERM_TAGS_WRITE_POLICY`
   - Accept `WritePolicyUpdate` body
   - Call `storage.update_write_policy()`

All handlers follow the existing pattern:
```rust
pub async fn handler_name(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(param): Path<Type>,
    Json(body): Json<Struct>,
) -> Result<impl IntoResponse, crate::api::AppError>
```

Use the existing permission check pattern from `api/mod.rs` (look for `has_permission` or similar).

- [ ] **Step 2: Register routes in api/mod.rs**

Add `mod commands;` declaration. Add routes to the protected router:

```rust
.route("/api/v1/tags/{id}/write", post(commands::write_tag))
.route("/api/v1/commands/{command_id}", get(commands::get_command_status))
.route("/api/v1/commands", get(commands::list_commands))
.route("/api/v1/commands/{command_id}/cancel", post(commands::cancel_command))
.route("/api/v1/commands/{command_id}/audit", get(commands::get_command_audit))
.route("/api/v1/data-sources/{id}/write-policy", patch(commands::update_write_policy))
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check`

- [ ] **Step 4: Commit**

```bash
git add src/api/commands.rs src/api/mod.rs
git commit -m "feat(commands): add command API endpoints for write, status, query, cancel, and write policy"
```

---

## Task 7: SSE Integration

The SSE events are already published via `cache.publish_sse_payload()` in the command queue processor (Task 5). This task verifies the integration works end-to-end.

**Files:**
- Modify: `src/api/stream.rs` (verify, may not need changes)

- [ ] **Step 1: Verify SSE stream forwards command events**

Check `src/api/stream.rs` — verify it doesn't filter event types. The `command_status` and `command_ack` JSON payloads flow through the same broadcast channel as existing sensor data events.

- [ ] **Step 2: Test with curl**

Subscribe to SSE stream: `curl -N http://localhost:3000/api/v1/stream`
Issue a write command via API.
Verify `command_status` and `command_ack` events appear in the stream.

- [ ] **Step 3: Commit if changes needed**

```bash
git add src/api/stream.rs
git commit -m "feat(commands): ensure SSE stream forwards command events"
```

---

## Task 8: Flow Engine write_out Node

**Files:**
- Modify: `src/flow_engine/node.rs`

- [ ] **Step 1: Add WriteOut variant to NodeConfig enum**

Add a new variant:

```rust
#[serde(rename = "write_out")]
WriteOut {
    id: String,
    data_source_id: i64,
    #[serde(default)]
    value_expr: Option<String>,
    #[serde(default = "default_flow_policy")]
    policy: String,
    #[serde(default = "default_flow_reason")]
    reason: String,
},
```

Add helpers:
```rust
fn default_flow_policy() -> String { "immediate".to_string() }
fn default_flow_reason() -> String { "flow:unknown".to_string() }
```

- [ ] **Step 2: Add WriteOut processing in the match arm**

In the `process` method, handle `NodeConfig::WriteOut`:
1. Resolve the write value from `value_expr` (using expression engine) or from message data
2. Look up `(device_id, protocol)` from storage via `data_source_id`
3. Create a `Command` struct and insert via `storage.insert_command()`
4. Return `NodeResult::Continue(msg)`

The `NodeContext` needs access to `Storage`. Check if it already has it or needs to be added. If `NodeContext` doesn't have storage, add an optional `storage: Option<Arc<Storage>>` field.

- [ ] **Step 3: Run cargo check**

Run: `cargo check`

- [ ] **Step 4: Commit**

```bash
git add src/flow_engine/node.rs
git commit -m "feat(commands): add write_out flow engine node for automated device writes"
```

---

## Task 9: Recipe Activation

**Files:**
- Modify: `src/recipe.rs`
- Modify: `src/api/recipes.rs`

- [ ] **Step 1: Replace dry-run in load_recipe_to_plc**

Replace the `[DRY-RUN]` logging loop with real command creation:
1. Parse recipe values JSON (existing code)
2. For each register/value pair: look up the corresponding data_source_id and device info
3. Create a `Command` with policy `"immediate"`, created_by `"recipe:{recipe_name}"`
4. Insert via `storage.insert_command()`
5. Return `Vec<String>` of command_ids

The function signature changes from `Result<()>` to `Result<Vec<String>>`.

- [ ] **Step 2: Update recipe load API response**

In `src/api/recipes.rs`, update the load endpoint response to include command_ids:

```json
{
    "ok": true,
    "command_ids": ["cmd_a1b2...", "cmd_e5f6..."],
    "message": "Recipe load queued: 5 commands"
}
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check`

- [ ] **Step 4: Commit**

```bash
git add src/recipe.rs src/api/recipes.rs
git commit -m "feat(commands): replace recipe dry-run with real command queue writes"
```

---

## Task 10: Frontend — Write Dialog & Command Panel

**Files:**
- Create: `client/src/api/commands.ts`
- Create: `client/src/components/WriteDialog.tsx`
- Create: `client/src/components/CommandPanel.tsx`
- Modify: `client/src/views/DeviceView.tsx`

- [ ] **Step 1: Create commands.ts API client**

TypeScript API client with typed interfaces for `Command`, `CommandAuditEntry`, `WritePolicy`, `CommandQuery`, and async functions: `writeTag`, `getCommand`, `listCommands`, `cancelCommand`, `getCommandAudit`, `updateWritePolicy`. Use the existing `apiFetch` pattern from the project.

- [ ] **Step 2: Create WriteDialog.tsx**

Modal dialog component:
- Shows current tag name and value
- Numeric input for new value with min/max display
- Required reason textarea if `write_requires_reason`
- Policy selector (immediate/queued/scheduled)
- Inline validation errors
- Loading state, success toast with command_id on submit
- Match existing dialog/modal styling

- [ ] **Step 3: Create CommandPanel.tsx**

Panel with two tabs:
- **History**: filterable table (time, tag, value, status badge, user) with pagination
- **Active**: auto-refreshing list of pending/queued/sent commands with cancel buttons
- Status badge colors: pending=yellow, queued=blue, sent=orange, acked=green, failed=red, expired/cancelled=grey
- Click row → expand to show audit trail
- SSE listener for real-time updates

- [ ] **Step 4: Integrate into DeviceView**

Add "Write" button next to writable tags. Only shown when `writable === true`. Opens WriteDialog on click.

- [ ] **Step 5: Commit**

```bash
git add client/src/api/commands.ts client/src/components/WriteDialog.tsx client/src/components/CommandPanel.tsx client/src/views/DeviceView.tsx
git commit -m "feat(commands): frontend write dialog and command history panel"
```

---

## Task 11: Frontend — Write Policy Editor & Recipe Status

**Files:**
- Create: `client/src/components/WritePolicyEditor.tsx`
- Create: `client/src/components/RecipeLoadStatus.tsx`
- Modify: `client/src/views/TagSettings.tsx`

- [ ] **Step 1: Create WritePolicyEditor.tsx**

Settings component:
- Toggle `writable` on/off (default off, confirmation warning when enabling)
- When on: min/max value inputs, max writes/minute input, require reason checkbox, deadband input
- Saves via `updateWritePolicy()`
- Permission-gated: only visible with `tags:write-policy`

- [ ] **Step 2: Create RecipeLoadStatus.tsx**

Recipe load progress:
- Each parameter shown as row with command status badge
- Auto-refreshes until all commands terminal
- Summary: "5/7 parameters written"
- "Retry Failed" button per failed parameter

- [ ] **Step 3: Integrate into TagSettings**

Add WritePolicyEditor as new "Write Policy" section in tag settings.

- [ ] **Step 4: Commit**

```bash
git add client/src/components/WritePolicyEditor.tsx client/src/components/RecipeLoadStatus.tsx client/src/views/TagSettings.tsx
git commit -m "feat(commands): frontend write policy editor and recipe load status"
```

---

## Task 12: Integration Tests

**Files:**
- Add tests to `src/storage.rs` and protocol files

- [ ] **Step 1: Command CRUD tests in storage.rs**

Add tests following the existing `#[tokio::test]` pattern in storage.rs:

Test 1: Insert command → get by id → verify fields
Test 2: Update status pending → acked → verify timestamps
Test 3: Rate limiting — insert N commands → count_recent_writes matches
Test 4: Audit trail — insert → update twice → verify audit entries
Test 5: Expired command cleanup — insert with old timestamp → verify expiry logic
Test 6: List commands with filters — insert multiple → filter by status/device/tag

- [ ] **Step 2: Write validation tests**

Test write policy validation:
- Non-writable tag returns error
- Value below write_min returns error
- Value above write_max returns error
- Deadband check against current value
- Rate limit exceeded returns error
- Missing reason on requires_reason tag returns error

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add src/storage.rs
git commit -m "test(commands): add integration tests for command CRUD, validation, and audit trail"
```

---

## Task 13: Bump Version & Final Checks

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Bump version to 0.24.0**

Change `version = "0.23.0"` to `version = "0.24.0"`.

- [ ] **Step 2: Run full test suite**

Run: `cargo test`

- [ ] **Step 3: Run cargo check + clippy**

Run: `cargo check && cargo clippy`

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "feat: v0.24.0 — Command Ack & Control Mode"
```
