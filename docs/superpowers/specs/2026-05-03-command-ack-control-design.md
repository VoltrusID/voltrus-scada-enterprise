# v0.24.0 — Command Ack & Control Mode

**Date:** 2026-05-03
**Status:** Approved
**Version:** 0.24.0
**Timeline:** 4-5 weeks
**Tier:** Starter ($249, basic write) / Professional ($499, full command tracking + queue)

---

## Goal

Voltrus is SCADA, not just monitoring. Reliable bidirectional control with acknowledgement across all protocols (Modbus TCP, Siemens S7, OPC UA, EtherNet/IP, MQTT).

---

## Architecture: Direct Write-Through

```
API / Flow Engine / Recipe
        │
        ▼
   CommandQueue (SQLite + background task)
        │
        ▼
   ProtocolDriver::write_tag()  ──►  Device
        │
        ▼
   Audit log (append-only)
```

Each protocol driver gains a `write_tag()` method. The command queue handles ordering, validation, offline buffering, retry, and rate limiting. Drivers just do the write.

---

## Data Model

### New table: `commands`

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT (UUID) | Command ID (returned to client as 202 response) |
| data_source_id | INTEGER | FK to the tag being written |
| device_id | INTEGER | FK to the target device (denormalized for fast queries) |
| protocol | TEXT | `modbus` / `s7` / `opcua` / `ethernet_ip` / `mqtt` |
| value | TEXT | JSON-encoded value (number, bool, string) |
| status | TEXT | `pending` / `queued` / `sent` / `acked` / `failed` / `expired` / `cancelled` |
| policy | TEXT | `immediate` / `queued` / `scheduled` |
| scheduled_at | INTEGER | Unix timestamp for scheduled execution |
| retry_count | INTEGER | Current retry attempts (default 0) |
| max_retries | INTEGER | Max retries (default 3) |
| error_message | TEXT | Last error if failed |
| created_at | INTEGER | When command was created |
| sent_at | INTEGER | When command was sent to device |
| acked_at | INTEGER | When device acknowledged |
| resolved_at | INTEGER | When command reached terminal state |
| created_by | TEXT | User who issued the command |

Indexes: `commands(status)`, `commands(data_source_id)`, `commands(device_id)`, `commands(created_at)`.

### New table: `command_audit_log`

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Auto PK |
| command_id | TEXT | FK to commands.id |
| old_status | TEXT | Previous status |
| new_status | TEXT | New status |
| changed_by | TEXT | User or `system` |
| reason | TEXT | Optional reason (e.g., "device offline, queued") |
| created_at | INTEGER | When the change happened |

Index: `command_audit_log(command_id)`.

### Modifications to `data_sources` table

| Column | Type | Default | Description |
|--------|------|---------|-------------|
| writable | BOOLEAN | 0 | Whether this tag accepts writes |
| write_min | REAL | NULL | Minimum allowed value |
| write_max | REAL | NULL | Maximum allowed value |
| write_max_rate | INTEGER | NULL | Max writes per minute |
| write_requires_reason | BOOLEAN | 0 | Require operator reason on write |
| write_deadband | REAL | NULL | Minimum change from current value to accept write |

---

## Protocol Write Layer

Consistent interface across all protocols:

```rust
pub struct WriteResult {
    pub success: bool,
    pub device_response: Option<String>,
    pub error: Option<String>,
    pub latency_ms: u64,
}
```

### Per-protocol write operations

| Protocol | Write Operations | Address Format |
|----------|-----------------|----------------|
| Modbus TCP | FC 05 (write single coil), FC 06 (write single register), FC 15 (write multiple coils), FC 16 (write multiple registers) | Existing register address (e.g., `40001`) |
| Siemens S7 | Write to DB/M/I/Q areas | Existing area+offset format |
| OPC UA | Write service call on node ID | Existing node ID (e.g., `ns=2;s=Temperature`) |
| EtherNet/IP | Write CIP tag | Existing tag name (e.g., `Program:Main.Pump_Speed`) |
| MQTT | Publish to topic | Existing topic (e.g., `plant/pump/speed`) |

### Connection reuse

Each driver already maintains a connection pool / reconnect loop for reads. Write reuses the same connection. If the connection is down, `write_tag()` returns an error and the command stays `queued`.

### Write validation (before sending to device)

1. Tag `writable == true`
2. Value within `write_min` / `write_max`
3. Value exceeds `write_deadband` from current cached value
4. Rate limit: count writes in last 60s vs `write_max_rate`

Validation failures return immediately with `failed` status — no retry.

---

## API Endpoints

### Write endpoints

| Method | Path | Permission | Description |
|--------|------|------------|-------------|
| POST | `/api/v1/tags/{id}/write` | `commands:write` | Issue a write command |
| GET | `/api/v1/commands/{command_id}` | `commands:read` | Poll command status |
| GET | `/api/v1/commands` | `commands:read` | List/filter command history |
| POST | `/api/v1/commands/{command_id}/cancel` | `commands:cancel` | Cancel a pending/queued command |
| PATCH | `/api/v1/data-sources/{id}/write-policy` | `tags:write-policy` | Update write policy for a tag |

### Write request body

```json
{
  "value": 100.5,
  "policy": "immediate",
  "scheduled_at": null,
  "reason": "Adjusting setpoint for batch changeover"
}
```

- `value` is required
- `policy` defaults to `"immediate"`, options: `immediate`, `queued`, `scheduled`
- `scheduled_at` required when `policy == "scheduled"`
- `reason` required when tag has `write_requires_reason == true`

### Write response (202 Accepted)

```json
{
  "command_id": "cmd_a1b2c3d4",
  "status": "pending",
  "data_source_id": 42,
  "value": 100.5,
  "created_at": 1746312345
}
```

### Command query params

`GET /api/v1/commands?tag=42&status=failed&device=7&from=1746300000&to=1746310000&limit=50&offset=0`

### Permissions

| Permission | Description |
|------------|-------------|
| `commands:write` | Issue write commands |
| `commands:read` | View command history |
| `commands:cancel` | Cancel pending commands |
| `tags:write-policy` | Modify write policies on tags |

---

## SSE Events

| Event | Payload | When |
|-------|---------|------|
| `command_status` | `{command_id, status, error?, device_response?}` | Any command status change |
| `command_ack` | `{command_id, status: "acked", device_response, latency_ms}` | Device acknowledges write |

All existing SSE subscribers automatically receive these events.

---

## Command Queue & Lifecycle

### State machine

```
                    ┌─────────────────────────────────┐
                    │           cancelled              │
                    └─────────────────────────────────┘
                       ▲
                       │ cancel API
                       │
  created ──► pending ──► sent ──► acked
                 │          │
                 │          │ error
                 │          ▼
                 └────► failed
                 │          │
                 │          │ retry_count < max_retries
                 │          ▼
                 └────► queued ──► pending (retry)
                               │
                               │ retry_count >= max_retries
                               ▼
                            expired
```

### Status semantics

| Status | Meaning |
|--------|---------|
| `pending` | Validated, about to send to device |
| `queued` | Device offline or rate-limited, waiting |
| `sent` | Written to device, awaiting async confirmation (only for protocols that defer ack; synchronous protocols like Modbus go directly pending → acked/failed) |
| `acked` | Device confirmed write success |
| `failed` | Write failed (validation or device error) |
| `expired` | Queued too long or max retries exhausted |
| `cancelled` | Operator cancelled before execution |

### Queue processing (background Tokio task, 500ms interval)

1. Pick all commands in `queued` status ordered by `created_at`
2. For each: check device connection state
   - Device online: transition to `pending`, call `write_tag()`
     - Success → `acked` with latency (synchronous protocols like Modbus skip `sent`)
     - Async protocol → `sent` (await device-side confirmation)
     - Error → `queued` (increment retry) or `failed` if max retries hit
   - Device offline: skip, stays `queued`
3. Expire commands where `created_at + 30min < now` and status is still `queued`
4. For `scheduled` commands: if `scheduled_at <= now`, transition to `pending`

### Rate limiting

Per-tag, the queue processor checks write count in the last 60s against `write_max_rate`. If exceeded, command stays `queued` with reason `"rate_limited"`.

### Concurrency

Only one command per device is in-flight at a time. If a device has a command in `sent` status, subsequent commands for that device stay `queued` until the in-flight command resolves.

---

## Flow Engine Integration

### `write_out` node type

```rust
WriteOut {
    data_source_id: i64,
    value_expr: Option<String>,
    policy: String,     // "immediate" | "queued"
    reason: String,     // auto: "flow:{flow_name}"
}
```

When a flow triggers (timer, threshold, expression), `write_out` creates a command through the same queue — same validation, same audit trail, same `created_by: "flow:{flow_name}"`.

---

## Recipe Activation

The existing `load_recipe_to_plc()` dry-run gets replaced:

- Each recipe parameter becomes a batch of `immediate` commands
- Recipe load creates commands for all parameters, returns list of `command_id`s
- Recipe status = aggregate of all command statuses (all acked → recipe loaded)
- If any command fails, recipe load is partial — operators see which params succeeded/failed
- `POST /api/recipes/{id}/load` now hits real devices instead of logging

---

## Frontend

| View | Content |
|------|---------|
| Write button | On any writable tag widget — opens write dialog with value input, reason field, policy selector |
| Command history | Filterable table of recent commands with status badges |
| Command detail | Click a command → full audit trail (every status change, timestamps, who) |
| Write policy editor | In tag settings — toggle writable, set min/max/rate/reason requirement |
| Active commands | Live-updating list of pending/queued/sent commands with cancel buttons |
| Recipe load status | Per-parameter write status during recipe activation |

---

## Files to Modify/Create

### Rust backend

| File | Change |
|------|--------|
| `storage.rs` | Add `commands` + `command_audit_log` tables, add write policy columns to `data_sources`, new CRUD methods |
| `models.rs` | `Command` struct, `CommandStatus` enum, `WritePolicy` struct, new permission constants |
| `api/mod.rs` | Register new command routes |
| `api/commands.rs` | New file — all command API handlers |
| `modbus.rs` | Add `write_tag()` with FC 05/06/15/16 |
| `s7.rs` | Add `write_tag()` for DB/M/I/Q areas |
| `opcua.rs` | Add `write_tag()` via OPC UA Write service |
| `ethernet_ip.rs` | Add `write_tag()` for CIP tags |
| `flow_engine/node.rs` | Add `WriteOut` node type |
| `recipe.rs` | Replace dry-run with real command queue calls |
| `cache.rs` | Add write rate tracking per tag |
| `main.rs` | Spawn command queue background task |

### Frontend

| File | Change |
|------|--------|
| `client/src/api/commands.ts` | New file — command API client functions |
| `client/src/components/CommandPanel.tsx` | New file — command history + active commands |
| `client/src/components/WriteDialog.tsx` | New file — write value dialog |
| `client/src/components/WritePolicyEditor.tsx` | New file — tag write policy settings |
| `client/src/components/RecipeLoadStatus.tsx` | New file — recipe parameter write status |
| `client/src/views/DeviceView.tsx` | Add write buttons on writable tags |
| `client/src/views/TagSettings.tsx` | Add write policy section |

---

## Testing Strategy

- Unit tests for each protocol's `write_tag()` (using protocol simulators in `src/bin/`)
- Unit tests for command queue state machine transitions
- Unit tests for write validation (min/max, deadband, rate limit)
- Integration tests for full command lifecycle: create → validate → send → ack
- Integration tests for offline queue + reconnect replay
- Integration tests for flow engine `write_out` node
- Integration tests for recipe activation with real writes
- API endpoint tests matching existing pattern in `src/api/`
