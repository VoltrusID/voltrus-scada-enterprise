# Industrial Data Flow Engine — Design Specification

**Date:** 2026-05-01  
**Status:** Draft  
**Author:** OpenCode Agent  
**Scope:** V1 MVP — embedded low-code data pipeline engine for Voltrus

---

## 1. Overview

### Problem
Users need to transform raw machine/IoT data, integrate with external systems (ERP, MES, alerts), and apply logic (thresholds, mapping, enrichment) without hardcoding or relying on external tools like Node-RED.

### Solution
An embedded, constrained data flow engine that allows users to define data pipelines via JSON config, transform and route data without scripting, and integrate with external systems. It is not a general-purpose automation platform — it is domain-specific to industrial SCADA/IoT workflows.

### Positioning
**"Industrial Data Flow Engine"** — not a "low-code platform." Tightly scoped to data transformation, alerting, and integration within the existing Voltrus architecture.

---

## 2. Goals & Non-Goals

### Goals
- Enable data transformation without scripting.
- Support real-time processing at MQTT/Modbus scale.
- Provide a simple integration layer (HTTP webhooks, MQTT publish).
- Run lightweight on edge and cloud.
- Produce virtual data sources that integrate seamlessly with existing dashboard views.

### Non-Goals
- Full programming environment (no JS/Python).
- General automation platform (no loops, async orchestration, complex branching).
- Replace Node-RED.
- Visual drag-and-drop builder in V1 (JSON config only).
- Complex workflow engine (no loops, sub-flows, or parallel branches).

---

## 3. Target Users

- **Primary:** SME factory operators, system integrators doing lightweight deployments.
- **Secondary:** Internal engineers configuring deployments.

---

## 4. Core Use Cases

| ID | Use Case | Description |
|----|----------|-------------|
| UC1 | Threshold Alert | If temperature > 80°C → send critical alarm |
| UC2 | Data Normalization | Map PLC payload → standardized schema for dashboard |
| UC3 | External Integration | Send production data → ERP / MES via HTTP webhook |
| UC4 | Derived Metrics | Compute simple KPIs (avg, status flags, unit conversions) |
| UC5 | Multi-Source Merge | Combine Modbus + MQTT inputs into a single derived metric |

---

## 5. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Browser (React SPA)                                        │
│         ↓ REST / SSE                                        │
├─────────────────────────────────────────────────────────────┤
│  Rust Backend (Axum)                                        │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Modbus   │───▶│ DeviceCache  │───▶│ Flow Engine      │  │
│  │ Poller   │    │ (broadcast)  │    │ ┌─────┐┌─────┐  │  │
│  └──────────┘    └──────────────┘    │ │Flow ││Flow │  │  │
│  ┌──────────┐                        │ │  A  ││  B  │  │  │
│  │ MQTT     │───────────────────────▶│ └─────┘└─────┘  │  │
│  │ Client   │                        └──────────────────┘  │
│  └──────────┘                            │                 │
│                                          ▼                 │
│                              ┌──────────────────┐         │
│                              │ Virtual outputs  │         │
│                              │ → DeviceCache    │         │
│                              │ → HTTP webhooks  │         │
│                              │ → Alarms         │         │
│                              │ → MQTT publish   │         │
│                              └──────────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Decisions
1. **Channel-based event bus:** A `tokio::sync::broadcast` channel sends `DeviceSnapshot` from the cache to the Flow Engine. MQTT client sends parsed JSON messages to the same engine. Each flow runs in its own Tokio task.
2. **Shared cache for outputs:** `data_source_out` nodes write virtual values back to the `DeviceCache` so they appear in `/api/live`, SSE, and history queries exactly like Modbus sensors.
3. **No poller blocking:** Flow execution is completely decoupled from Modbus polling. A slow or crashing flow cannot affect device polling.
4. **SQLite persistence:** Flow definitions and run logs are stored in the existing SQLite database. Virtual data sources reuse the existing `data_sources` table.

---

## 6. Message Format

All nodes pass a single `Message` struct:

```rust
pub struct Message {
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
}
```

### Input Payload Examples

**Modbus input:**
```json
{
  "device_id": "plts_1",
  "timestamp": 1710000000,
  "metrics": {
    "voltage": 220.5,
    "current": 5.2,
    "temperature": 78.0
  }
}
```

**MQTT input:**
```json
{
  "sensor_id": "temp_01",
  "value": 82.5,
  "unit": "C"
}
```

---

## 7. Flow Definition

A flow is a JSON object with an array of nodes.

```json
{
  "id": "temp_alert_flow",
  "name": "Temperature Critical Alert",
  "enabled": true,
  "trigger": {
    "type": "modbus",
    "device_id": "plts_1",
    "metric": "temperature",
    "debounce_ms": 1000,
    "on_change_only": false
  },
  "nodes": [
    { "id": "n1", "type": "filter", "condition": "payload.metrics.temperature > 0" },
    { "id": "n2", "type": "threshold", "field": "payload.metrics.temperature", "rules": [
      { "gt": 80, "set": { "alarm_level": "CRITICAL" } }
    ]},
    { "id": "n3", "type": "filter", "condition": "payload.alarm_level == 'CRITICAL'" },
    { "id": "n4", "type": "alarm_out", "severity": "critical", "message": "Temperature critical: {{payload.metrics.temperature}}°C" }
  ]
}
```

### Node Schema (Common)

Every node has:
- `id` (string, unique within flow): reference for debugging
- `type` (string): one of the types below
- type-specific fields

---

## 8. Node Types (MVP)

### 8.1 Input Nodes

These are defined in the flow's `trigger` block, not as explicit nodes in the `nodes` array.

| Trigger Type | Fields | Behavior |
|-------------|--------|----------|
| `modbus` | `device_id`, `metric` (optional), `debounce_ms`, `on_change_only` | Fires when matching device snapshot arrives from cache broadcast |
| `mqtt` | `topic` (supports `+` and `#` wildcards), `debounce_ms` | Fires when MQTT message on matching topic arrives |
| `timer` | `interval_ms` | Fires repeatedly on the given interval |

### 8.2 Processing Nodes

#### `filter`

Passes the message through only if the condition evaluates to `true`.

```json
{
  "id": "f1",
  "type": "filter",
  "condition": "payload.metrics.temperature > 70 && payload.metrics.pressure < 100"
}
```

#### `transform`

Rewrites the payload using expression mappings.

```json
{
  "id": "t1",
  "type": "transform",
  "map": {
    "machine_id": "payload.device_id",
    "temp_c": "payload.metrics.temperature",
    "temp_f": "payload.metrics.temperature * 1.8 + 32",
    "status": "if(payload.metrics.temperature > 80, 'CRITICAL', 'OK')"
  }
}
```

After `transform`, the `payload` is replaced with the mapped object. `metadata` is preserved.

#### `threshold`

Checks a field against rules and sets additional fields.

```json
{
  "id": "th1",
  "type": "threshold",
  "field": "payload.metrics.temperature",
  "rules": [
    { "gt": 80, "set": { "alarm": "HIGH", "status": "critical" } },
    { "gt": 60, "set": { "alarm": "MEDIUM", "status": "warning" } }
  ]
}
```

Rules are evaluated in order; the first match wins. The original payload is preserved and merged with `set` fields.

#### `lookup`

Maps one field value to another via a static table.

```json
{
  "id": "l1",
  "type": "lookup",
  "field": "payload.machine_code",
  "output_field": "payload.machine_name",
  "table": {
    "M1": "Cutting Machine",
    "M2": "Packing Machine"
  },
  "default": "Unknown"
}
```

#### `expression`

Computes a new field and adds it to the payload.

```json
{
  "id": "e1",
  "type": "expression",
  "output_field": "payload.power_kw",
  "expr": "payload.metrics.voltage * payload.metrics.current * 0.001"
}
```

#### `rolling_avg`

Computes a rolling window average and stores it in the payload.

```json
{
  "id": "r1",
  "type": "rolling_avg",
  "field": "payload.metrics.speed",
  "window": 10,
  "output_field": "payload.speed_avg"
}
```

State is stored in memory (`DashMap<flow_id_node_id, Vec<f64>>`). Lost on restart.

### 8.3 Output Nodes

#### `data_source_out`

Writes the computed value back to the shared cache as a virtual data source.

```json
{
  "id": "out1",
  "type": "data_source_out",
  "value_field": "payload.temp_f",
  "source_key": "flow_temp_alert_temp_f",
  "name": "Temperature (F)",
  "unit": "°F",
  "decimals": 1
}
```

The engine auto-creates or updates a `DataSource` record with `source_type = "flow"` so it appears in the dashboard.

#### `http_out`

POSTs the current message payload to an external URL.

```json
{
  "id": "out2",
  "type": "http_out",
  "url": "https://erp.example.com/api/metrics",
  "headers": {
    "Authorization": "Bearer {{env.ERP_TOKEN}}",
    "Content-Type": "application/json"
  },
  "timeout_ms": 5000,
  "retry_count": 3
}
```

#### `mqtt_out`

Publishes the current message payload to an MQTT topic.

```json
{
  "id": "out3",
  "type": "mqtt_out",
  "topic": "alerts/critical",
  "qos": 1
}
```

#### `alarm_out`

Creates or resolves an alarm in the existing alarm system.

```json
{
  "id": "out4",
  "type": "alarm_out",
  "severity": "critical",
  "source_key": "flow_temp_alert",
  "message": "Temperature critical: {{payload.metrics.temperature}}°C"
}
```

If the message reaches this node, the alarm is created. If the flow stops reaching this node (e.g., filter blocks it), the alarm is auto-resolved on the next run.

---

## 9. Expression Engine

### Requirements
- No user scripting (JS/Python).
- Deterministic evaluation.
- Safe execution (no arbitrary code).

### Supported Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+`, `-`, `*`, `/` |
| Comparison | `>`, `<`, `>=`, `<=`, `==` |
| Logical | `&&`, `\|\|` |

### Supported Functions

| Function | Description |
|----------|-------------|
| `if(cond, a, b)` | Returns `a` if `cond` is true, else `b` |
| `round(x)` | Round to nearest integer |
| `round(x, n)` | Round to `n` decimal places |
| `clamp(x, min, max)` | Clamp `x` between `min` and `max` |
| `avg(a, b, ...)` | Average of arguments |
| `sqrt(x)` | Square root |
| `abs(x)` | Absolute value |

### Data Access

Dot notation resolves into JSON payloads:

```
payload.metrics.temperature
payload.device_id
metadata.source
```

If a path does not exist, the expression evaluates to `null` (which fails comparisons safely).

### Implementation

Extend the existing recursive descent parser in `src/expression.rs`:
1. Add new tokens: `Gt`, `Lt`, `Gte`, `Lte`, `Eq`, `And`, `Or`, `Dot`, `StringLit`
2. Add `Expr` variants: `Compare`, `LogicalAnd`, `LogicalOr`, `JsonPath`, `If`, `StringLit`
3. Extend evaluator to accept `serde_json::Value` as context and resolve dot paths
4. Add new functions to the function dispatcher

---

## 10. Execution Model

### Trigger Matching

When the Flow Engine receives a `Message`:

1. Iterate all active flows.
2. Check trigger match:
   - **Modbus:** `device_id` matches (or `"*"`), and `metric` exists in payload if specified.
   - **MQTT:** `topic` matches the subscription pattern.
   - **Timer:** Timer task fires independently; no channel message needed.
3. Check debounce: skip if flow ran within `debounce_ms`.
4. Check `on_change_only`: skip if the relevant metric hasn't changed since last run.
5. Spawn a lightweight Tokio task for the flow.

### Flow Task Lifecycle

```
spawn async task:
  let mut msg = input_message;
  for node in flow.nodes:
    match node.process(&mut msg):
      Ok(Continue)     → next node
      Ok(FilterReject) → stop flow (don't run remaining nodes)
      Err(e)           → log error, stop flow
  log run result
```

### State Management

- **Stateless by default:** Most nodes process a single message and produce an output.
- **Stateful nodes:** `rolling_avg` maintains a `VecDeque<f64>` in a `DashMap` keyed by `(flow_id, node_id)`.
- **No persistence of node state:** Rolling window state is lost on restart. This is acceptable for V1.

---

## 11. Database Schema

### Flows Table

```sql
CREATE TABLE flows (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  enabled INTEGER DEFAULT 1,
  trigger_config TEXT NOT NULL,  -- JSON
  nodes TEXT NOT NULL,           -- JSON array
  created_at INTEGER,
  updated_at INTEGER
);
```

### Flow Runs Table (for debugging)

```sql
CREATE TABLE flow_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  flow_id TEXT NOT NULL,
  started_at INTEGER,
  duration_ms INTEGER,
  success INTEGER,
  error TEXT,
  output TEXT,  -- JSON snapshot of final message payload
  FOREIGN KEY (flow_id) REFERENCES flows(id)
);
```

### Data Sources Extension

No new table needed. Virtual data sources reuse the existing `data_sources` table with `source_type = "flow"`. The `source_config` JSON stores `flow_id` and `node_id` for traceability.

---

## 12. API Design

### Flow Management (Admin-only)

| Method | Endpoint | Body | Response |
|--------|----------|------|----------|
| GET | `/api/flows` | — | `[{ id, name, enabled, trigger, node_count }]` |
| POST | `/api/flows` | Flow JSON | `{ id }` |
| GET | `/api/flows/{id}` | — | Full flow JSON |
| PUT | `/api/flows/{id}` | Flow JSON | — |
| DELETE | `/api/flows/{id}` | — | — |
| POST | `/api/flows/{id}/start` | — | — |
| POST | `/api/flows/{id}/stop` | — | — |

### Flow Testing (Admin-only)

| Method | Endpoint | Body | Response |
|--------|----------|------|----------|
| POST | `/api/flows/{id}/test` | `{ "payload": { ... } }` | `{ "success": true, "output": { ... }, "node_results": [...] }` |

The test endpoint runs the flow synchronously with a sample payload and returns the output at each node for debugging.

### Flow Runs (Read-only, any authenticated user)

| Method | Endpoint | Query | Response |
|--------|----------|-------|----------|
| GET | `/api/flows/{id}/runs` | `?limit=20&since=` | `[{ id, started_at, duration_ms, success, error }]` |

---

## 13. Error Handling

### Node Errors
- If any node returns `Err`, the flow stops and the error is logged.
- Output nodes (`http_out`, `mqtt_out`) have built-in retry with exponential backoff.
- `http_out` failures are logged but do not stop the flow.

### Flow Engine Errors
- If the Flow Engine task panics, it is restarted automatically.
- If MQTT client disconnects, it reconnects with backoff.
- A single bad flow cannot crash the engine or block other flows.

### Debugging
Every flow run is logged to `flow_runs` with:
- Start time
- Duration
- Success/failure
- Error message (if any)
- Final output snapshot

The test endpoint (`POST /api/flows/{id}/test`) allows step-by-step inspection.

---

## 14. Non-Functional Requirements

### Performance
- Target: ≥ 1,000 msg/sec across all flows combined.
- Per-flow latency: < 10ms for simple filter → transform → output chains.
- Memory: < 50MB overhead for the Flow Engine (excluding node state).

### Reliability
- No message loss from Modbus (broadcast channel handles backpressure).
- HTTP output retry: 3 attempts with exponential backoff.
- MQTT output: QoS 1 where supported.

### Resource Usage
- Each flow runs in its own lightweight Tokio task.
- No thread-per-flow. Total threads = Tokio worker pool (default: num CPUs).
- Aggregation state stored in memory only.

---

## 15. Integration with Existing Systems

### DeviceCache
- Modbus poller publishes `DeviceSnapshot` to a new `tokio::sync::broadcast` channel.
- Flow Engine subscribes to this channel.
- `data_source_out` nodes write back to `DeviceCache` using the same `update()` method.

### Storage
- Flow definitions stored in SQLite via new `Storage` methods.
- Virtual data sources created automatically in `data_sources` table.
- Flow runs logged to new `flow_runs` table.

### API Router
- New routes added under `/api/flows/*` in `api/mod.rs`.
- Admin-only middleware applied to mutation endpoints.

### SSE Stream
- Virtual data sources from flows appear in `/api/live` and SSE automatically because they are written to the shared cache.

---

## 16. Roadmap

### V1 (MVP — 3–4 weeks)
- [ ] Flow engine core (channel-based, task-per-flow)
- [ ] Node types: `filter`, `transform`, `threshold`, `expression`, `lookup`, `data_source_out`, `http_out`, `alarm_out`
- [ ] Trigger types: `modbus`, `timer`
- [ ] Extended expression engine (dot notation, comparisons, logical ops, `if`)
- [ ] REST API for CRUD + test + start/stop
- [ ] SQLite schema + persistence
- [ ] Integration with DeviceCache for virtual outputs
- [ ] Basic run logging

### V2 (2–3 weeks)
- [ ] MQTT input (`mqtt_in` trigger)
- [ ] MQTT output (`mqtt_out` node)
- [ ] `rolling_avg` aggregation node
- [ ] `on_change_only` trigger option
- [ ] Flow run inspector UI (frontend)

### V3 (2–3 weeks)
- [ ] Visual flow builder (frontend drag-and-drop)
- [ ] Flow templates (threshold alert, OEE, downtime)
- [ ] Reusable sub-flows
- [ ] Flow import/export

---

## 17. Risks & Mitigation

| Risk | Mitigation |
|------|------------|
| Scope creep | Strict node limit in V1; only industrial SCADA use cases |
| Too technical for SMEs | Prebuilt nodes (threshold, lookup) + templates in V3 |
| Debug difficulty | Mandatory run logging + test endpoint + per-node output tracing |
| Competing with Node-RED | Stay domain-specific; embed deeply into Voltrus |
| Performance at scale | Channel-based async; stateless design; benchmark at 1,000 msg/sec |

---

## 18. Open Questions

1. **MQTT Broker:** Should Voltrus act as an MQTT broker, or connect to an external one (e.g., Mosquitto, EMQX)? Recommendation: external broker in V1, embedded broker optional in V2.
2. **Flow Versioning:** Should flow definitions be versioned? Recommendation: defer to V3.
3. **Expression null handling:** Should `null > 80` evaluate to `false` or error? Recommendation: `false` (safe comparison).

---

## 19. Appendix: File Structure

```
src/
  flow_engine/
    mod.rs           -- public API, engine startup
    engine.rs        -- FlowEngine task, trigger matching, task spawning
    message.rs       -- Message struct
    node.rs          -- Node trait + registry
    nodes/
      filter.rs
      transform.rs
      threshold.rs
      lookup.rs
      expression.rs
      rolling_avg.rs
      data_source_out.rs
      http_out.rs
      mqtt_out.rs
      alarm_out.rs
    trigger.rs       -- Trigger matching logic
    state.rs         -- In-memory state for aggregation nodes
    run_log.rs       -- Flow run logging
  expression.rs      -- EXTENDED with dot notation, comparisons, logical ops
```

---

*End of Design Specification*
