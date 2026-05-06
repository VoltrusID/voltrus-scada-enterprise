# Industrial Data Flow Engine — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an embedded low-code data pipeline engine for Voltrus that transforms Modbus device data through configurable JSON-defined flows, producing virtual data sources, HTTP webhooks, and alarms.

**Architecture:** Channel-based event bus where the existing `DeviceCache` publishes `DeviceSnapshot` via `tokio::sync::broadcast`. A `FlowEngine` task subscribes, matches triggers, and spawns lightweight Tokio tasks per flow. Each flow runs nodes sequentially through an enum-based pipeline. Outputs write back to the cache or trigger side effects. All flow definitions and run logs persist to SQLite.

**Tech Stack:** Rust (Tokio, Axum, serde_json), SQLite (rusqlite), existing expression engine (extended)

---

## File Structure

```
src/
  flow_engine/
    mod.rs              -- Module public API, re-exports
    message.rs          -- Message struct (payload + metadata)
    node.rs             -- NodeConfig enum, NodeResult, processing logic
    trigger.rs          -- Trigger matching logic (modbus, timer)
    state.rs            -- In-memory FlowState (DashMap for aggregation)
    run_log.rs          -- Flow run logging to SQLite
    engine.rs           -- FlowEngine task, trigger matching, task spawning
  api/
    flows.rs            -- REST API handlers for flow CRUD + test
  expression.rs         -- EXTENDED (dot notation, comparisons, logical ops, if, string literals)
  cache.rs              -- MODIFIED (add broadcast channel)
  storage.rs            -- MODIFIED (flow CRUD, flow_runs table)
  models.rs             -- MODIFIED (Flow, FlowRun, TriggerConfig structs)
  api/mod.rs            -- MODIFIED (add /api/flows routes)
  main.rs               -- MODIFIED (start FlowEngine)
Cargo.toml              -- MODIFIED (add dashmap)
```

---

## Task 1: Add dashmap dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add dashmap to dependencies**

Add `dashmap = "=6.1.0"` in the `[dependencies]` section, after `rand = "=0.9.4"`.

- [ ] **Step 2: Verify**

Run: `cargo check`  
Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "deps: add dashmap for flow engine state"
```

---

## Task 2: Extend expression engine

**Files:**
- Modify: `src/expression.rs`

Extend the existing recursive descent parser to support dot notation JSON paths, comparison operators (`>`, `<`, `>=`, `<=`, `==`), logical operators (`&&`, `||`), string literals (`"..."`), multi-arg functions (`if(cond,a,b)`, `avg(a,b,c)`), and evaluation against `serde_json::Value`.

- [ ] **Step 1: Extend Token enum and tokenize function**

Add new tokens: `Ident(String)`, `StringLit(String)`, `Comma`, `Dot`, `Gt`, `Lt`, `Gte`, `Lte`, `Eq`, `And`, `Or`. Update `tokenize` to handle multi-char operators (`>=`, `<=`, `==`, `&&`, `||`), dot notation, string literals, and identifiers (alphanumeric + underscore). Keep `{Register}` syntax for backward compatibility.

- [ ] **Step 2: Extend Expr enum and Parser**

Add new variants: `JsonPath(Vec<String>)`, `StringLit(String)`, `Func(String, Vec<Expr>)`, `Compare { op: CompareOp, left, right }`, `LogicalAnd(left, right)`, `LogicalOr(left, right)`. Update parser precedence: `or` -> `and` -> `comparison` -> `expr` -> `term` -> `unary` -> `atom`. In `parse_atom`, handle `Ident` followed by `.` (JSON path) or `(` (multi-arg function call).

- [ ] **Step 3: Add JSON evaluator**

Add `pub fn evaluate_json(input: &str, context: &Value) -> Result<Value, String>`. Implement `eval_expr_json` that resolves `JsonPath` against the context `Value`, handles comparisons for both numbers and strings, short-circuits logical operators, and supports functions: `if`, `sqrt`, `abs`, `round`, `clamp`, `avg`. Add helpers: `to_f64`, `to_bool`, `compare_values`. Null comparisons return `false` except `null == null`.

- [ ] **Step 4: Add tests**

Add tests for: `payload.metrics.temperature` JSON path, `temp > 80` comparison, `a && b` logical, `if(temp > 80, 'CRITICAL', 'OK')`, `clamp(val, 0, 100)`, and complex combined expression.

- [ ] **Step 5: Run tests**

Run: `cargo test expression::tests`  
Expected: All existing + new tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/expression.rs
git commit -m "feat(expression): add dot notation, comparisons, logical ops, if for flow engine"
```

---

## Task 3: Create flow engine core types

**Files:**
- Create: `src/flow_engine/mod.rs`
- Create: `src/flow_engine/message.rs`
- Create: `src/flow_engine/node.rs`
- Create: `src/flow_engine/trigger.rs`
- Create: `src/flow_engine/state.rs`
- Create: `src/flow_engine/run_log.rs`

- [ ] **Step 1: Create `src/flow_engine/mod.rs`**

Declare submodules: `pub mod message; pub mod node; pub mod trigger; pub mod state; pub mod run_log; pub mod engine;`

- [ ] **Step 2: Create `src/flow_engine/message.rs`**

```rust
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Message {
    pub payload: Value,
    pub metadata: HashMap<String, String>,
}

impl Message {
    pub fn new(payload: Value) -> Self {
        Self { payload, metadata: HashMap::new() }
    }
}
```

- [ ] **Step 3: Create `src/flow_engine/node.rs`**

Define `NodeResult` enum (`Continue`, `FilterReject`), `NodeContext` struct (flow_id, node_id, state, storage, cache), and `NodeConfig` enum with serde `tag = "type"`:

Variants: `Filter { id, condition }`, `Transform { id, map }`, `Threshold { id, field, rules }`, `Lookup { id, field, output_field, table, default }`, `Expression { id, output_field, expr }`, `DataSourceOut { id, value_field, source_key, name, unit, decimals }`, `HttpOut { id, url, headers, timeout_ms, retry_count }`, `AlarmOut { id, severity, source_key, message }`.

Add `ThresholdRule` struct with optional `gt`, `gte`, `lt`, `lte`, `eq`, and `set: HashMap<String, Value>`.

Implement `NodeConfig::process(&self, msg, ctx) -> Result<NodeResult, String>`:
- **Filter:** Evaluate condition with `evaluate_json`. Truthy -> `Continue`, falsy -> `FilterReject`.
- **Transform:** For each `(key, expr)` in map, evaluate expr and set at JSON path on new payload. Replace `msg.payload` entirely.
- **Threshold:** Get field value. For each rule, check if value matches (numeric comparisons or eq). First match wins. Merge `rule.set` into payload via JSON paths.
- **Lookup:** Get field value, look up in table, set `output_field` to result or default.
- **Expression:** Evaluate expr, set result at `output_field` JSON path.
- **DataSourceOut:** Extract numeric value from `value_field`, spawn async task to call `cache.update(device_id, metrics)` where device_id = `format!("flow_{}", flow_id)` and metrics = `{source_key: numeric_value}`.
- **HttpOut:** Spawn async task to POST payload to URL with headers, retry up to `retry_count` times with exponential backoff (100ms * attempt).
- **AlarmOut:** Spawn async task to call `storage.create_alarm(source_key, severity, 0.0, 0.0, message)`.

Add helper functions: `get_json_path(value, path)`, `set_json_path(map, path, value)`, `rule_matches(value, rule)`.

- [ ] **Step 4: Create `src/flow_engine/trigger.rs`**

Define `TriggerConfig` enum with serde `tag = "type"`:
- `Modbus { device_id: String, metric: Option<String>, debounce_ms: Option<u64>, on_change_only: Option<bool> }`
- `Timer { interval_ms: u64 }`
- `Mqtt { topic: String, debounce_ms: Option<u64> }` (stub for V2)

Implement `TriggerConfig::matches(&self, msg: &Message) -> bool`:
- **Modbus:** Check `msg.metadata["source"] == "modbus"`. If `device_id != "*"`, check `msg.payload["device_id"] == device_id`. If `metric` specified, check it exists in `msg.payload["metrics"]`.
- **Timer:** Always false (timer flows are handled separately by interval tasks).
- **Mqtt:** Always false in V1.

- [ ] **Step 5: Create `src/flow_engine/state.rs`**

```rust
use dashmap::DashMap;
use std::collections::VecDeque;

pub struct FlowState {
    rolling_windows: DashMap<String, VecDeque<f64>>,
}

impl FlowState {
    pub fn new() -> Self {
        Self { rolling_windows: DashMap::new() }
    }

    pub fn push_value(&self, key: &str, value: f64, window_size: usize) {
        let mut entry = self.rolling_windows.entry(key.to_string()).or_insert_with(VecDeque::new);
        entry.push_back(value);
        while entry.len() > window_size {
            entry.pop_front();
        }
    }

    pub fn rolling_avg(&self, key: &str) -> Option<f64> {
        self.rolling_windows.get(key).map(|entry| {
            let sum: f64 = entry.iter().sum();
            sum / entry.len() as f64
        })
    }
}
```

- [ ] **Step 6: Create `src/flow_engine/run_log.rs`**

```rust
use crate::storage::Storage;

pub async fn log_run(
    storage: &Storage,
    flow_id: &str,
    duration_ms: u64,
    success: bool,
    error: Option<String>,
    output: Option<serde_json::Value>,
) {
    if let Err(e) = storage.insert_flow_run(flow_id, duration_ms, success, error, output).await {
        tracing::warn!("Failed to log flow run: {}", e);
    }
}
```

- [ ] **Step 7: Add `flow_engine` module to main.rs**

In `src/main.rs`, add `mod flow_engine;` alongside existing modules.

- [ ] **Step 8: Commit**

```bash
git add src/flow_engine/
git add src/main.rs
git commit -m "feat(flow-engine): add core types - message, node, trigger, state, run_log"
```

---

## Task 4: Create FlowEngine

**Files:**
- Create: `src/flow_engine/engine.rs`

- [ ] **Step 1: Define Flow struct and FlowEngine**

```rust
use crate::flow_engine::{message::Message, node::NodeConfig, trigger::TriggerConfig, state::FlowState, run_log};
use crate::storage::Storage;
use crate::cache::DeviceCache;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Flow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger: TriggerConfig,
    pub nodes: Vec<NodeConfig>,
}

pub struct FlowEngine {
    flows: Arc<RwLock<HashMap<String, Flow>>>,
    state: Arc<FlowState>,
    storage: Storage,
    cache: DeviceCache,
    snapshot_rx: broadcast::Receiver<crate::models::DeviceSnapshot>,
}
```

- [ ] **Step 2: Implement FlowEngine::new and spawn**

```rust
impl FlowEngine {
    pub fn new(
        storage: Storage,
        cache: DeviceCache,
        snapshot_rx: broadcast::Receiver<crate::models::DeviceSnapshot>,
    ) -> Self {
        Self {
            flows: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(FlowState::new()),
            storage,
            cache,
            snapshot_rx,
        }
    }

    pub async fn spawn(self) {
        let flows = self.flows.clone();
        let state = self.state.clone();
        let storage = self.storage.clone();
        let cache = self.cache.clone();
        let mut rx = self.snapshot_rx;

        // Spawn timer flows
        {
            let flows = flows.clone();
            let state = state.clone();
            let storage = storage.clone();
            let cache = cache.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    let flows_guard = flows.read().await;
                    for flow in flows_guard.values() {
                        if !flow.enabled { continue; }
                        if let TriggerConfig::Timer { interval_ms } = &flow.trigger {
                            // Simple interval check using flow id hash for staggering
                            // Full timer implementation in Step 4
                        }
                    }
                }
            });
        }

        // Main loop: process broadcast messages
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(snapshot) => {
                        let msg = device_snapshot_to_message(snapshot);
                        let flows_guard = flows.read().await;
                        for flow in flows_guard.values() {
                            if !flow.enabled { continue; }
                            if flow.trigger.matches(&msg) {
                                let flow = flow.clone();
                                let state = state.clone();
                                let storage = storage.clone();
                                let cache = cache.clone();
                                tokio::spawn(async move {
                                    run_flow(&flow, msg, &state, &storage, &cache).await;
                                });
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("FlowEngine lagged by {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::info!("FlowEngine broadcast channel closed");
                        break;
                    }
                }
            }
        });
    }
}

fn device_snapshot_to_message(snapshot: crate::models::DeviceSnapshot) -> Message {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "modbus".to_string());
    let payload = serde_json::json!({
        "device_id": snapshot.device_id,
        "status": match snapshot.status {
            crate::models::DeviceStatus::Online => "online",
            crate::models::DeviceStatus::Offline => "offline",
        },
        "timestamp": snapshot.timestamp,
        "metrics": snapshot.metrics,
    });
    Message::new(payload)
}

async fn run_flow(flow: &Flow, mut msg: Message, state: &FlowState, storage: &Storage, cache: &DeviceCache) {
    let start = std::time::Instant::now();
    let ctx = crate::flow_engine::node::NodeContext {
        flow_id: &flow.id,
        node_id: "",
        state,
        storage,
        cache,
    };

    for node in &flow.nodes {
        let mut node_ctx = crate::flow_engine::node::NodeContext {
            flow_id: &flow.id,
            node_id: node.id(),
            state,
            storage,
            cache,
        };
        match node.process(&mut msg, &node_ctx) {
            Ok(crate::flow_engine::node::NodeResult::Continue) => {}
            Ok(crate::flow_engine::node::NodeResult::FilterReject) => {
                run_log::log_run(storage, &flow.id, start.elapsed().as_millis() as u64, true, None, None).await;
                return;
            }
            Err(e) => {
                run_log::log_run(storage, &flow.id, start.elapsed().as_millis() as u64, false, Some(e), None).await;
                return;
            }
        }
    }

    run_log::log_run(storage, &flow.id, start.elapsed().as_millis() as u64, true, None, Some(msg.payload)).await;
}
```

- [ ] **Step 3: Add flow management methods**

```rust
impl FlowEngine {
    pub async fn add_flow(&self, flow: Flow) {
        let mut flows = self.flows.write().await;
        flows.insert(flow.id.clone(), flow);
    }

    pub async fn remove_flow(&self, flow_id: &str) {
        let mut flows = self.flows.write().await;
        flows.remove(flow_id);
    }

    pub async fn get_flow(&self, flow_id: &str) -> Option<Flow> {
        let flows = self.flows.read().await;
        flows.get(flow_id).cloned()
    }

    pub async fn list_flows(&self) -> Vec<Flow> {
        let flows = self.flows.read().await;
        flows.values().cloned().collect()
    }

    pub async fn update_flow(&self, flow: Flow) {
        let mut flows = self.flows.write().await;
        flows.insert(flow.id.clone(), flow);
    }
}
```

- [ ] **Step 4: Implement timer trigger support**

Add a `last_run: DashMap<String, Instant>` to `FlowEngine`. In the timer spawn task, track last run per flow. On each 1-second tick, check timer flows: if `elapsed() >= interval_ms`, run the flow with an empty Message (or a message containing just timestamp) and update `last_run`.

- [ ] **Step 5: Commit**

```bash
git add src/flow_engine/engine.rs
git commit -m "feat(flow-engine): add FlowEngine with broadcast subscription and timer support"
```

---

## Task 5: Add broadcast channel to DeviceCache

**Files:**
- Modify: `src/cache.rs`

- [ ] **Step 1: Add broadcast sender to DeviceCache**

In `src/cache.rs`, add a `tokio::sync::broadcast::Sender<DeviceSnapshot>` field to `DeviceCache`. Update `DeviceCache::new()` to create a broadcast channel with capacity 256. Provide a `pub fn subscribe(&self) -> broadcast::Receiver<DeviceSnapshot>` method.

- [ ] **Step 2: Publish on update**

In the `update` method (or wherever snapshots are written), after updating the internal state, call `let _ = self.tx.send(snapshot.clone());` to broadcast the snapshot.

- [ ] **Step 3: Commit**

```bash
git add src/cache.rs
git commit -m "feat(cache): add broadcast channel for DeviceSnapshot to feed flow engine"
```

---

## Task 6: Add database schema and storage methods

**Files:**
- Modify: `src/storage.rs`

- [ ] **Step 1: Add flow tables to schema**

In `init_schema`, add:

```sql
CREATE TABLE IF NOT EXISTS flows (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  enabled INTEGER DEFAULT 1,
  trigger_config TEXT NOT NULL,
  nodes TEXT NOT NULL,
  created_at INTEGER,
  updated_at INTEGER
);

CREATE TABLE IF NOT EXISTS flow_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  flow_id TEXT NOT NULL,
  started_at INTEGER,
  duration_ms INTEGER,
  success INTEGER,
  error TEXT,
  output TEXT
);
```

- [ ] **Step 2: Add flow CRUD methods to Storage**

Add methods:
- `create_flow(&self, id, name, enabled, trigger_config, nodes) -> Result<()>`
- `get_flow(&self, id) -> Result<Option<Flow>>`
- `list_flows(&self) -> Result<Vec<Flow>>`
- `update_flow(&self, id, name, enabled, trigger_config, nodes) -> Result<()>`
- `delete_flow(&self, id) -> Result<()>`
- `insert_flow_run(&self, flow_id, duration_ms, success, error, output) -> Result<()>`
- `list_flow_runs(&self, flow_id, limit) -> Result<Vec<FlowRun>>`

Parse `trigger_config` and `nodes` from JSON strings using `serde_json`.

- [ ] **Step 3: Commit**

```bash
git add src/storage.rs
git commit -m "feat(storage): add flows and flow_runs tables with CRUD methods"
```

---

## Task 7: Add model types

**Files:**
- Modify: `src/models.rs`

- [ ] **Step 1: Add Flow and FlowRun structs**

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Flow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger_config: String,
    pub nodes: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlowRun {
    pub id: i64,
    pub flow_id: String,
    pub started_at: i64,
    pub duration_ms: i64,
    pub success: bool,
    pub error: Option<String>,
    pub output: Option<String>,
}
```

- [ ] **Step 2: Commit**

```bash
git add src/models.rs
git commit -m "feat(models): add Flow and FlowRun structs"
```

---

## Task 8: Add REST API for flows

**Files:**
- Create: `src/api/flows.rs`
- Modify: `src/api/mod.rs`

- [ ] **Step 1: Create API handlers**

Create `src/api/flows.rs` with handlers:

```rust
use axum::{extract::{Path, State}, Json, http::StatusCode};
use serde_json::json;
use crate::api::{AppState, AppError};
use crate::flow_engine::engine::Flow;

pub async fn list_flows(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let flows = state.storage.list_flows().await?;
    Ok(Json(json!({ "flows": flows })))
}

pub async fn get_flow(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Flow>, AppError> {
    let flow = state.storage.get_flow(&id).await?
        .ok_or_else(|| anyhow::anyhow!("Flow not found"))?;
    Ok(Json(flow))
}

pub async fn create_flow(State(state): State<AppState>, Json(req): Json<Flow>) -> Result<Json<serde_json::Value>, AppError> {
    let now = crate::timeutil::now_secs();
    state.storage.create_flow(&req.id, &req.name, req.enabled, &req.trigger_config, &req.nodes).await?;
    if let Some(engine) = &state.flow_engine {
        engine.add_flow(req).await;
    }
    Ok(Json(json!({ "id": req.id })))
}

pub async fn update_flow(State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<Flow>) -> Result<StatusCode, AppError> {
    state.storage.update_flow(&id, &req.name, req.enabled, &req.trigger_config, &req.nodes).await?;
    if let Some(engine) = &state.flow_engine {
        engine.update_flow(req).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_flow(State(state): State<AppState>, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    state.storage.delete_flow(&id).await?;
    if let Some(engine) = &state.flow_engine {
        engine.remove_flow(&id).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_flow(State(state): State<AppState>, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    if let Some(engine) = &state.flow_engine {
        if let Some(mut flow) = engine.get_flow(&id).await {
            flow.enabled = true;
            engine.update_flow(flow).await;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn stop_flow(State(state): State<AppState>, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    if let Some(engine) = &state.flow_engine {
        if let Some(mut flow) = engine.get_flow(&id).await {
            flow.enabled = false;
            engine.update_flow(flow).await;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_flow(State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let flow = state.storage.get_flow(&id).await?
        .ok_or_else(|| anyhow::anyhow!("Flow not found"))?;
    
    // Deserialize nodes
    let nodes: Vec<crate::flow_engine::node::NodeConfig> = serde_json::from_str(&flow.nodes)
        .map_err(|e| anyhow::anyhow!("Invalid nodes: {}", e))?;
    
    let payload = req.get("payload").cloned().unwrap_or(serde_json::Value::Null);
    let mut msg = crate::flow_engine::message::Message::new(payload);
    let state_arc = crate::flow_engine::state::FlowState::new();
    let cache = state.cache.clone();
    
    let mut results = Vec::new();
    for node in &nodes {
        let ctx = crate::flow_engine::node::NodeContext {
            flow_id: &flow.id,
            node_id: node.id(),
            state: &state_arc,
            storage: &state.storage,
            cache: &cache,
        };
        let before = msg.payload.clone();
        match node.process(&mut msg, &ctx) {
            Ok(result) => {
                results.push(json!({
                    "node_id": node.id(),
                    "type": std::format!("{:?}", std::mem::discriminant(node)),
                    "result": match result {
                        crate::flow_engine::node::NodeResult::Continue => "continue",
                        crate::flow_engine::node::NodeResult::FilterReject => "filter_reject",
                    },
                    "output": msg.payload,
                }));
                if result == crate::flow_engine::node::NodeResult::FilterReject {
                    break;
                }
            }
            Err(e) => {
                results.push(json!({
                    "node_id": node.id(),
                    "error": e,
                }));
                break;
            }
        }
    }
    
    Ok(Json(json!({
        "success": true,
        "output": msg.payload,
        "node_results": results,
    })))
}

pub async fn list_flow_runs(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    let runs = state.storage.list_flow_runs(&id, 20).await?;
    Ok(Json(json!({ "runs": runs })))
}
```

- [ ] **Step 2: Update AppState to include FlowEngine**

In `src/api/mod.rs`, add `pub flow_engine: Option<Arc<crate::flow_engine::engine::FlowEngine>>` to `AppState`.

- [ ] **Step 3: Add routes**

In `src/api/mod.rs`, in the `admin_protected` router, add:

```rust
.route("/api/flows", get(flows::list_flows).post(flows::create_flow))
.route("/api/flows/{id}", get(flows::get_flow).put(flows::update_flow).delete(flows::delete_flow))
.route("/api/flows/{id}/start", post(flows::start_flow))
.route("/api/flows/{id}/stop", post(flows::stop_flow))
.route("/api/flows/{id}/test", post(flows::test_flow))
.route("/api/flows/{id}/runs", get(flows::list_flow_runs))
```

- [ ] **Step 4: Commit**

```bash
git add src/api/flows.rs src/api/mod.rs
git commit -m "feat(api): add flow CRUD, test, and run history endpoints"
```

---

## Task 9: Wire up FlowEngine in main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Initialize FlowEngine**

After creating `cache` and `storage`, create the broadcast receiver:

```rust
let snapshot_rx = cache.subscribe();
```

Then create and spawn the FlowEngine:

```rust
let flow_engine = crate::flow_engine::engine::FlowEngine::new(
    storage.clone(),
    cache.clone(),
    snapshot_rx,
);
flow_engine.spawn().await;
```

Load existing flows from storage and add them:

```rust
if let Ok(flows) = storage.list_flows().await {
    for flow in flows {
        if flow.enabled {
            flow_engine.add_flow(flow).await;
        }
    }
}
```

Add `flow_engine: Some(Arc::new(flow_engine))` to `AppState`.

- [ ] **Step 2: Commit**

```bash
git add src/main.rs
git commit -m "feat(main): wire up FlowEngine with cache broadcast and existing flows"
```

---

## Task 10: Integration test

**Files:**
- Modify: `src/api/mod.rs` (test module)

- [ ] **Step 1: Add flow engine test**

In the existing `#[cfg(test)]` mod in `src/api/mod.rs`, add a test:

```rust
#[tokio::test]
async fn test_flow_crud() {
    let app = test_app();
    
    // Get auth cookie via login
    let storage = Storage::new_in_memory().unwrap();
    storage.seed_database().await.unwrap();
    
    // ... login and get cookie ...
    
    // Create flow
    let flow = serde_json::json!({
        "id": "test_flow",
        "name": "Test Flow",
        "enabled": true,
        "trigger_config": serde_json::json!({"type": "modbus", "device_id": "*"}).to_string(),
        "nodes": serde_json::json!([
            {"id": "n1", "type": "filter", "condition": "payload.metrics.temp > 0"}
        ]).to_string(),
    });
    
    // POST /api/flows
    // GET /api/flows/test_flow
    // PUT /api/flows/test_flow
    // DELETE /api/flows/test_flow
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test`  
Expected: All existing + new tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/api/mod.rs
git commit -m "test(flow-engine): add integration test for flow CRUD"
```

---

## Spec Coverage Check

| Spec Requirement | Task |
|-----------------|------|
| Dot notation expression engine | Task 2 |
| Comparison/logical operators | Task 2 |
| `if` function | Task 2 |
| Message struct with payload + metadata | Task 3 |
| NodeConfig enum with all MVP types | Task 3 |
| Filter node | Task 3 |
| Transform node | Task 3 |
| Threshold node | Task 3 |
| Lookup node | Task 3 |
| Expression node | Task 3 |
| DataSourceOut node | Task 3 |
| HttpOut node | Task 3 |
| AlarmOut node | Task 3 |
| Trigger matching (modbus, timer) | Task 3, Task 4 |
| FlowState (DashMap) | Task 3 |
| Run logging | Task 3, Task 6 |
| FlowEngine task spawning | Task 4 |
| DeviceCache broadcast | Task 5 |
| SQLite schema (flows, flow_runs) | Task 6 |
| Flow CRUD storage methods | Task 6 |
| Flow + FlowRun models | Task 7 |
| REST API (CRUD, test, start/stop, runs) | Task 8 |
| Wire up in main.rs | Task 9 |
| Integration test | Task 10 |

---

*End of Implementation Plan*
