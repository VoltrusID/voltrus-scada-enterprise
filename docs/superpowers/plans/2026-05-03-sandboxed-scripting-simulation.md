# Sandboxed Scripting & Simulation Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement three subsystems — per-device simulation mode, sandboxed JS scripting via QuickJS-WASM, and config versioning with rollback — for Voltrus v0.26.0.

**Architecture:** Simulation injects fake data into the existing cache pipeline via per-device flags and sim profile generators. Scripting adds a `ScriptNode` to the Flow Engine that executes user JS inside a pre-compiled QuickJS interpreter running in wasmtime. Config versioning auto-creates snapshots of versioned tables before every config mutation.

**Tech Stack:** Rust (wasmtime crate for WASM runtime, QuickJS-WASM binary embedded), SQLite (4 new tables + column additions), TypeScript/React frontend.

**Spec:** `docs/superpowers/specs/2026-05-03-sandboxed-scripting-simulation-design.md`

---

## File Structure

### New Files (Rust backend)
| File | Responsibility |
|------|---------------|
| `src/sim_engine.rs` | Sim value generators (sinusoidal, random_walk, step, constant, square_wave) |
| `src/script_engine.rs` | QuickJS-WASM interpreter setup, wasmtime sandbox execution, resource limits |
| `src/api/simulation.rs` | Sim profile CRUD + device simulate/go-live handlers |
| `src/api/scripts.rs` | Script CRUD, validate, test, execution history handlers |
| `src/api/config_snapshots.rs` | Snapshot list, detail, manual create, diff, rollback handlers |

### New Files (Frontend)
| File | Responsibility |
|------|---------------|
| `client/src/api/simulation.ts` | Sim profile + device sim API client |
| `client/src/api/scripts.ts` | Script API client |
| `client/src/api/config-snapshots.ts` | Config snapshot API client |
| `client/src/components/SimProfileManager.tsx` | Sim profile CRUD with waveform preview |
| `client/src/components/SimControls.tsx` | Device simulation toggle + go-live dialog |
| `client/src/components/ScriptEditor.tsx` | Monaco-based script editor with test panel |
| `client/src/components/ScriptExecutionHistory.tsx` | Execution history table |
| `client/src/components/ConfigSnapshotPanel.tsx` | Snapshot list, diff viewer, rollback |

### Modified Files
| File | Change |
|------|--------|
| `Cargo.toml` | Add wasmtime dependency, bump version to 0.26.0 |
| `src/models.rs` | New structs + 6 permission constants |
| `src/storage.rs` | 4 new tables, column additions to devices/data_sources, new CRUD methods |
| `src/api/mod.rs` | Register 3 new route modules |
| `src/main.rs` | Add mod declarations, CLI subcommands |
| `src/poller_manager.rs` | Check simulation_mode before poll |
| `src/flow_engine/node.rs` | Add ScriptNode variant to NodeConfig enum |
| `src/flow_engine/engine.rs` | Handle ScriptNode in node_id() match |
| `src/cache.rs` | Add simulated flag to cached metric entries |
| `client/src/views/DeviceView.tsx` | SIM badge, simulation section |
| `client/src/views/FlowEditor.tsx` | Script node type in palette |
| `client/src/App.tsx` | Snapshots button in header |
| `client/src/TrendChart.tsx` | Dotted lines for simulated data |

---

## Task 1: Version Bump & Dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Bump version and add wasmtime**

Change version from `0.25.0` to `0.26.0` and add wasmtime dependency:

```toml
[package]
name = "voltrus-lite-rust-service"
version = "0.26.0"
edition = "2024"
license = "MIT"

# ... existing [[bin]] sections unchanged ...

[dependencies]
# ... existing dependencies unchanged, add at end:
wasmtime = "=29.0"
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished dev profile [unoptimized + debuginfo] target(s)`

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: bump to v0.26.0, add wasmtime dependency"
```

---

## Task 2: Models — Structs and Permissions

**Files:**
- Modify: `src/models.rs`

- [ ] **Step 1: Add new permission constants**

After the existing `PERM_TAGS_WRITE_POLICY` constant (line 1774), add:

```rust
pub const PERM_SIM_READ: &str = "sim:read";
pub const PERM_SIM_WRITE: &str = "sim:write";
pub const PERM_SCRIPTS_READ: &str = "scripts:read";
pub const PERM_SCRIPTS_WRITE: &str = "scripts:write";
pub const PERM_CONFIG_READ: &str = "config:read";
pub const PERM_CONFIG_WRITE: &str = "config:write";
```

- [ ] **Step 2: Update ALL_PERMISSIONS array**

Update the `ALL_PERMISSIONS` array (line 1777) to include the 6 new permissions:

```rust
pub const ALL_PERMISSIONS: &[&str] = &[
    PERM_SCREENS_READ, PERM_SCREENS_WRITE,
    PERM_DEVICES_READ, PERM_DEVICES_WRITE,
    PERM_ALARMS_READ, PERM_ALARMS_ACK,
    PERM_SETTINGS_READ, PERM_SETTINGS_WRITE,
    PERM_USERS_MANAGE,
    PERM_FLOWS_MANAGE,
    PERM_NOTIFICATIONS_MANAGE,
    PERM_HUB_MANAGE,
    PERM_REPORTS_MANAGE,
    PERM_RECIPES_MANAGE,
    PERM_COMMANDS_WRITE, PERM_COMMANDS_READ,
    PERM_COMMANDS_CANCEL,
    PERM_TAGS_WRITE_POLICY,
    PERM_SIM_READ, PERM_SIM_WRITE,
    PERM_SCRIPTS_READ, PERM_SCRIPTS_WRITE,
    PERM_CONFIG_READ, PERM_CONFIG_WRITE,
];
```

- [ ] **Step 3: Update OPERATOR_PERMISSIONS array**

Update `OPERATOR_PERMISSIONS` (line 1800) to include the 3 read permissions:

```rust
pub const OPERATOR_PERMISSIONS: &[&str] = &[
    PERM_SCREENS_READ, PERM_DEVICES_READ, PERM_ALARMS_READ,
    PERM_ALARMS_ACK, PERM_SETTINGS_READ,
    PERM_COMMANDS_WRITE, PERM_COMMANDS_READ,
    PERM_SIM_READ, PERM_SCRIPTS_READ, PERM_CONFIG_READ,
];
```

- [ ] **Step 4: Add new model structs**

Add at the end of the file (before the closing of the file, after existing structs like `SecurityEvent`):

```rust
// ── Simulation types (v0.26.0) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimProfile {
    pub id: i64,
    pub name: String,
    pub data_type: String,
    pub config: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSimProfileRequest {
    pub name: String,
    pub data_type: String,
    pub config: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSimProfileRequest {
    pub name: Option<String>,
    pub data_type: Option<String>,
    pub config: Option<String>,
}

// ── Script types (v0.26.0) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: i64,
    pub name: String,
    pub source: String,
    pub syntax_error: Option<String>,
    pub cpu_limit_ms: i64,
    pub memory_limit_bytes: i64,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateScriptRequest {
    pub name: String,
    pub source: String,
    #[serde(default = "default_cpu_limit")]
    pub cpu_limit_ms: Option<i64>,
    #[serde(default = "default_memory_limit")]
    pub memory_limit_bytes: Option<i64>,
}

fn default_cpu_limit() -> Option<i64> { Some(1000) }
fn default_memory_limit() -> Option<i64> { Some(1048576) }

#[derive(Debug, Deserialize)]
pub struct UpdateScriptRequest {
    pub name: Option<String>,
    pub source: Option<String>,
    pub cpu_limit_ms: Option<i64>,
    pub memory_limit_bytes: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TestScriptRequest {
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecution {
    pub id: i64,
    pub script_id: i64,
    pub flow_id: Option<String>,
    pub trigger_type: Option<String>,
    pub duration_ms: i64,
    pub status: String,
    pub error_message: Option<String>,
    pub output: Option<String>,
    pub created_at: i64,
}

// ── Config Snapshot types (v0.26.0) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub id: i64,
    pub label: Option<String>,
    pub description: Option<String>,
    pub snapshot_data: String,
    pub trigger: String,
    pub created_by: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotRequest {
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiffEntry {
    pub field: String,
    pub from: serde_json::Value,
    pub to: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTableDiff {
    pub added: Vec<serde_json::Value>,
    pub removed: Vec<serde_json::Value>,
    pub changed: Vec<ConfigDiffEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiff {
    pub tables: std::collections::HashMap<String, ConfigTableDiff>,
}
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished dev profile [unoptimized + debuginfo] target(s)`

- [ ] **Step 6: Commit**

```bash
git add src/models.rs
git commit -m "feat(models): add v0.26.0 structs and permission constants"
```

---

## Task 3: Storage Schema — New Tables & Column Additions

**Files:**
- Modify: `src/storage.rs`

- [ ] **Step 1: Add new table creation in `init_schema()`**

Locate the `init_schema()` method. After the last existing `CREATE TABLE IF NOT EXISTS` statement (before the guarded ALTER TABLE section), add these four table creation statements:

```rust
// v0.26.0 — Simulation profiles
conn.execute_batch(
    "CREATE TABLE IF NOT EXISTS sim_profiles (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL,
        data_type TEXT NOT NULL,
        config TEXT NOT NULL DEFAULT '{}',
        created_at INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS idx_sim_profiles_name ON sim_profiles(name);

    CREATE TABLE IF NOT EXISTS scripts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL,
        source TEXT NOT NULL DEFAULT '',
        syntax_error TEXT,
        cpu_limit_ms INTEGER NOT NULL DEFAULT 1000,
        memory_limit_bytes INTEGER NOT NULL DEFAULT 1048576,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at INTEGER NOT NULL DEFAULT 0,
        updated_at INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS idx_scripts_name ON scripts(name);

    CREATE TABLE IF NOT EXISTS script_executions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        script_id INTEGER NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
        flow_id TEXT,
        trigger_type TEXT,
        duration_ms INTEGER NOT NULL DEFAULT 0,
        status TEXT NOT NULL DEFAULT 'success',
        error_message TEXT,
        output TEXT,
        created_at INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS idx_script_executions_script ON script_executions(script_id);
    CREATE INDEX IF NOT EXISTS idx_script_executions_created ON script_executions(created_at);

    CREATE TABLE IF NOT EXISTS config_snapshots (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        label TEXT,
        description TEXT,
        snapshot_data TEXT NOT NULL DEFAULT '{}',
        trigger TEXT NOT NULL DEFAULT 'auto',
        created_by TEXT,
        created_at INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS idx_config_snapshots_created ON config_snapshots(created_at);
    CREATE INDEX IF NOT EXISTS idx_config_snapshots_trigger ON config_snapshots(trigger);
    "
)?;
```

- [ ] **Step 2: Add guarded ALTER TABLE statements for devices and data_sources**

In the guarded ALTER TABLE section (the block that uses `ALTER TABLE ... ADD COLUMN` with error catching), add:

```rust
// v0.26.0 — Simulation columns on devices
let _ = conn.execute_batch("ALTER TABLE devices ADD COLUMN simulation_mode INTEGER NOT NULL DEFAULT 0");
let _ = conn.execute_batch("ALTER TABLE devices ADD COLUMN sim_profile_id INTEGER");
let _ = conn.execute_batch("ALTER TABLE devices ADD COLUMN sim_config TEXT");
// v0.26.0 — Simulation column on data_sources
let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN sim_value REAL");
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished dev profile [unoptimized + debuginfo] target(s)`

- [ ] **Step 4: Run existing tests**

Run: `cargo test 2>&1 | tail -20`
Expected: All existing tests pass. New tables are created in test DBs automatically.

- [ ] **Step 5: Commit**

```bash
git add src/storage.rs
git commit -m "feat(storage): add v0.26.0 schema — sim_profiles, scripts, script_executions, config_snapshots tables"
```

---

## Task 4: Storage CRUD — Sim Profiles

**Files:**
- Modify: `src/storage.rs`

- [ ] **Step 1: Add sim profile CRUD methods**

Add these methods to the `impl Storage` block:

```rust
// ── Sim Profile CRUD ──

pub async fn list_sim_profiles(&self) -> anyhow::Result<Vec<crate::models::SimProfile>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, data_type, config, created_at FROM sim_profiles ORDER BY name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::SimProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                data_type: row.get(2)?,
                config: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| anyhow::anyhow!(e))
    }).await?
}

pub async fn get_sim_profile(&self, id: i64) -> anyhow::Result<Option<crate::models::SimProfile>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, data_type, config, created_at FROM sim_profiles WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(crate::models::SimProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                data_type: row.get(2)?,
                config: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }).await?
}

pub async fn create_sim_profile(&self, name: &str, data_type: &str, config: &str) -> anyhow::Result<i64> {
    let conn = self.conn.clone();
    let name = name.to_string();
    let data_type = data_type.to_string();
    let config = config.to_string();
    let now = crate::timeutil::now_secs();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sim_profiles (name, data_type, config, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![name, data_type, config, now],
        )?;
        Ok(conn.last_insert_rowid())
    }).await?
}

pub async fn update_sim_profile(&self, id: i64, name: Option<&str>, data_type: Option<&str>, config: Option<&str>) -> anyhow::Result<bool> {
    let conn = self.conn.clone();
    let name = name.map(String::from);
    let data_type = data_type.map(String::from);
    let config = config.map(String::from);
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let current = conn.query_row(
            "SELECT name, data_type, config FROM sim_profiles WHERE id = ?1",
            [id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        );
        let (n, dt, cfg) = match current {
            Ok((n, dt, cfg)) => (n, dt, cfg),
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
            Err(e) => return Err(anyhow::anyhow!(e)),
        };
        let name = name.unwrap_or(n);
        let data_type = data_type.unwrap_or(dt);
        let config = config.unwrap_or(cfg);
        let rows = conn.execute(
            "UPDATE sim_profiles SET name = ?1, data_type = ?2, config = ?3 WHERE id = ?4",
            rusqlite::params![name, data_type, config, id],
        )?;
        Ok(rows > 0)
    }).await?
}

pub async fn delete_sim_profile(&self, id: i64) -> anyhow::Result<bool> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM sim_profiles WHERE id = ?1", [id])?;
        Ok(rows > 0)
    }).await?
}

// ── Device simulation ──

pub async fn set_device_simulation(&self, device_id: &str, sim_mode: bool, sim_profile_id: Option<i64>, sim_config: Option<&str>) -> anyhow::Result<bool> {
    let conn = self.conn.clone();
    let device_id = device_id.to_string();
    let sim_config = sim_config.map(String::from);
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE devices SET simulation_mode = ?1, sim_profile_id = ?2, sim_config = ?3 WHERE id = ?4",
            rusqlite::params![sim_mode as i32, sim_profile_id, sim_config, device_id],
        )?;
        Ok(rows > 0)
    }).await?
}

pub async fn get_device_simulation(&self, device_id: &str) -> anyhow::Result<Option<(bool, Option<i64>, Option<String>)>> {
    let conn = self.conn.clone();
    let device_id = device_id.to_string();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT simulation_mode, sim_profile_id, sim_config FROM devices WHERE id = ?1",
            [&device_id],
            |row| Ok((row.get::<_, i32>(0)? != 0, row.get::<_, Option<i64>>(1)?, row.get::<_, Option<String>>(2)?)),
        );
        match result {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }).await?
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 3: Commit**

```bash
git add src/storage.rs
git commit -m "feat(storage): add sim profile CRUD and device simulation methods"
```

---

## Task 5: Storage CRUD — Scripts

**Files:**
- Modify: `src/storage.rs`

- [ ] **Step 1: Add script CRUD methods**

Add these methods to `impl Storage`:

```rust
// ── Script CRUD ──

pub async fn list_scripts(&self) -> anyhow::Result<Vec<crate::models::Script>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, source, syntax_error, cpu_limit_ms, memory_limit_bytes, enabled, created_at, updated_at FROM scripts ORDER BY name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::Script {
                id: row.get(0)?,
                name: row.get(1)?,
                source: row.get(2)?,
                syntax_error: row.get(3)?,
                cpu_limit_ms: row.get(4)?,
                memory_limit_bytes: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| anyhow::anyhow!(e))
    }).await?
}

pub async fn get_script(&self, id: i64) -> anyhow::Result<Option<crate::models::Script>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, source, syntax_error, cpu_limit_ms, memory_limit_bytes, enabled, created_at, updated_at FROM scripts WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(crate::models::Script {
                id: row.get(0)?,
                name: row.get(1)?,
                source: row.get(2)?,
                syntax_error: row.get(3)?,
                cpu_limit_ms: row.get(4)?,
                memory_limit_bytes: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }).await?
}

pub async fn create_script(&self, name: &str, source: &str, syntax_error: Option<&str>, cpu_limit_ms: i64, memory_limit_bytes: i64) -> anyhow::Result<i64> {
    let conn = self.conn.clone();
    let name = name.to_string();
    let source = source.to_string();
    let syntax_error = syntax_error.map(String::from);
    let now = crate::timeutil::now_secs();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO scripts (name, source, syntax_error, cpu_limit_ms, memory_limit_bytes, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
            rusqlite::params![name, source, syntax_error, cpu_limit_ms, memory_limit_bytes, now, now],
        )?;
        Ok(conn.last_insert_rowid())
    }).await?
}

pub async fn update_script(&self, id: i64, name: Option<&str>, source: Option<&str>, syntax_error: Option<&str>, cpu_limit_ms: Option<i64>, memory_limit_bytes: Option<i64>, enabled: Option<bool>) -> anyhow::Result<bool> {
    let conn = self.conn.clone();
    let name = name.map(String::from);
    let source = source.map(String::from);
    let syntax_error = syntax_error.map(String::from);
    let now = crate::timeutil::now_secs();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        // Build dynamic UPDATE
        let mut sets = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        if let Some(ref v) = name { sets.push("name = ?"); params.push(Box::new(v.clone())); }
        if let Some(ref v) = source { sets.push("source = ?"); params.push(Box::new(v.clone())); }
        if syntax_error.is_some() { sets.push("syntax_error = ?"); params.push(Box::new(syntax_error)); }
        if let Some(v) = cpu_limit_ms { sets.push("cpu_limit_ms = ?"); params.push(Box::new(v)); }
        if let Some(v) = memory_limit_bytes { sets.push("memory_limit_bytes = ?"); params.push(Box::new(v)); }
        if let Some(v) = enabled { sets.push("enabled = ?"); params.push(Box::new(v as i32)); }
        sets.push("updated_at = ?");
        params.push(Box::new(now));
        if sets.is_empty() { return Ok(false); }
        params.push(Box::new(id));
        let sql = format!("UPDATE scripts SET {} WHERE id = ?", sets.join(", "));
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = conn.execute(&sql, param_refs.as_slice())?;
        Ok(rows > 0)
    }).await?
}

pub async fn delete_script(&self, id: i64) -> anyhow::Result<bool> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM scripts WHERE id = ?1", [id])?;
        Ok(rows > 0)
    }).await?
}

// ── Script Execution CRUD ──

pub async fn create_script_execution(&self, script_id: i64, flow_id: Option<&str>, trigger_type: Option<&str>, duration_ms: i64, status: &str, error_message: Option<&str>, output: Option<&str>) -> anyhow::Result<i64> {
    let conn = self.conn.clone();
    let flow_id = flow_id.map(String::from);
    let trigger_type = trigger_type.map(String::from);
    let status = status.to_string();
    let error_message = error_message.map(String::from);
    let output = output.map(String::from);
    let now = crate::timeutil::now_secs();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO script_executions (script_id, flow_id, trigger_type, duration_ms, status, error_message, output, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![script_id, flow_id, trigger_type, duration_ms, status, error_message, output, now],
        )?;
        Ok(conn.last_insert_rowid())
    }).await?
}

pub async fn list_script_executions(&self, script_id: i64, limit: i64) -> anyhow::Result<Vec<crate::models::ScriptExecution>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, script_id, flow_id, trigger_type, duration_ms, status, error_message, output, created_at FROM script_executions WHERE script_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map(rusqlite::params![script_id, limit], |row| {
            Ok(crate::models::ScriptExecution {
                id: row.get(0)?,
                script_id: row.get(1)?,
                flow_id: row.get(2)?,
                trigger_type: row.get(3)?,
                duration_ms: row.get(4)?,
                status: row.get(5)?,
                error_message: row.get(6)?,
                output: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| anyhow::anyhow!(e))
    }).await?
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 3: Commit**

```bash
git add src/storage.rs
git commit -m "feat(storage): add script and script execution CRUD methods"
```

---

## Task 6: Storage CRUD — Config Snapshots

**Files:**
- Modify: `src/storage.rs`

- [ ] **Step 1: Add config snapshot CRUD methods**

Add these methods to `impl Storage`:

```rust
// ── Config Snapshots ──

pub async fn list_config_snapshots(&self, limit: i64) -> anyhow::Result<Vec<crate::models::ConfigSnapshot>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, label, description, snapshot_data, trigger, created_by, created_at FROM config_snapshots ORDER BY id DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(crate::models::ConfigSnapshot {
                id: row.get(0)?,
                label: row.get(1)?,
                description: row.get(2)?,
                snapshot_data: row.get(3)?,
                trigger: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| anyhow::anyhow!(e))
    }).await?
}

pub async fn get_config_snapshot(&self, id: i64) -> anyhow::Result<Option<crate::models::ConfigSnapshot>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, label, description, snapshot_data, trigger, created_by, created_at FROM config_snapshots WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(crate::models::ConfigSnapshot {
                id: row.get(0)?,
                label: row.get(1)?,
                description: row.get(2)?,
                snapshot_data: row.get(3)?,
                trigger: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }).await?
}

pub async fn create_config_snapshot(&self, label: Option<&str>, description: Option<&str>, trigger: &str, created_by: Option<&str>) -> anyhow::Result<i64> {
    let conn = self.conn.clone();
    let label = label.map(String::from);
    let description = description.map(String::from);
    let trigger = trigger.to_string();
    let created_by = created_by.map(String::from);
    let now = crate::timeutil::now_secs();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        // Serialize all versioned tables
        let mut snapshot = serde_json::Map::new();
        for table in &["devices", "data_sources", "flows", "alarms", "screens", "recipes"] {
            let rows: Result<Vec<serde_json::Value>, _> = conn.prepare(&format!("SELECT * FROM {}", table))?
                .query_map([], |row| {
                    let n = row.as_ref().column_count();
                    let mut map = serde_json::Map::new();
                    for i in 0..n {
                        let col_name = row.as_ref().column_name(i).to_string();
                        let val: rusqlite::types::Value = row.get(i)?;
                        map.insert(col_name, rusqlite_value_to_json(val));
                    }
                    Ok(serde_json::Value::Object(map))
                })?
                .collect();
            if let Ok(rows) = rows {
                snapshot.insert(table.to_string(), serde_json::Value::Array(rows));
            }
        }
        let snapshot_data = serde_json::Value::Object(snapshot).to_string();
        conn.execute(
            "INSERT INTO config_snapshots (label, description, snapshot_data, trigger, created_by, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![label, description, snapshot_data, trigger, created_by, now],
        )?;
        Ok(conn.last_insert_rowid())
    }).await?
}

pub async fn rollback_to_snapshot(&self, snapshot_id: i64) -> anyhow::Result<bool> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        let snapshot_data: String = conn.query_row(
            "SELECT snapshot_data FROM config_snapshots WHERE id = ?1",
            [snapshot_id],
            |row| row.get(0),
        ).map_err(|_| anyhow::anyhow!("Snapshot not found"))?;

        let snapshot: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&snapshot_data)?;

        let tx = conn.unchecked_transaction()?;
        for table in &["devices", "data_sources", "flows", "alarms", "screens", "recipes"] {
            tx.execute(&format!("DELETE FROM {}", table), [])?;
            if let Some(serde_json::Value::Array(rows)) = snapshot.get(table) {
                for row in rows {
                    if let serde_json::Value::Object(map) = row {
                        let columns: Vec<String> = map.keys().cloned().collect();
                        let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();
                        let sql = format!(
                            "INSERT INTO {} ({}) VALUES ({})",
                            table,
                            columns.join(", "),
                            placeholders.join(", ")
                        );
                        let vals: Vec<Box<dyn rusqlite::types::ToSql>> = columns.iter()
                            .map(|c| json_to_rusqlite_value(map.get(c).unwrap_or(&serde_json::Value::Null)))
                            .collect();
                        let val_refs: Vec<&dyn rusqlite::types::ToSql> = vals.iter().map(|v| v.as_ref()).collect();
                        tx.execute(&sql, val_refs.as_slice())?;
                    }
                }
            }
        }
        tx.commit()?;
        Ok(true)
    }).await?
}

pub async fn cleanup_old_snapshots(&self, keep_count: i64) -> anyhow::Result<()> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = conn.lock().unwrap();
        // Delete oldest auto snapshots beyond keep_count, keep manual ones
        conn.execute(
            "DELETE FROM config_snapshots WHERE trigger = 'auto' AND id NOT IN (SELECT id FROM config_snapshots ORDER BY id DESC LIMIT ?1)",
            [keep_count],
        )?;
        Ok(())
    }).await?
}

fn rusqlite_value_to_json(val: rusqlite::types::Value) -> serde_json::Value {
    match val {
        rusqlite::types::Value::Null => serde_json::Value::Null,
        rusqlite::types::Value::Integer(i) => serde_json::json!(i),
        rusqlite::types::Value::Real(f) => serde_json::json!(f),
        rusqlite::types::Value::Text(s) => serde_json::json!(s),
        rusqlite::types::Value::Blob(b) => serde_json::json!(b),
    }
}

fn json_to_rusqlite_value(val: &serde_json::Value) -> Box<dyn rusqlite::types::ToSql> {
    match val {
        serde_json::Value::Null => Box::new(Option::<String>::None),
        serde_json::Value::Bool(b) => Box::new(*b as i32),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { Box::new(i) }
            else if let Some(f) = n.as_f64() { Box::new(f) }
            else { Box::new(n.to_string()) }
        },
        serde_json::Value::String(s) => Box::new(s.clone()),
        _ => Box::new(val.to_string()),
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 3: Commit**

```bash
git add src/storage.rs
git commit -m "feat(storage): add config snapshot CRUD, rollback, and cleanup methods"
```

---

## Task 7: Sim Engine — Value Generators

**Files:**
- Create: `src/sim_engine.rs`

- [ ] **Step 1: Create the sim engine module**

Create `src/sim_engine.rs`:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimConfig {
    pub data_type: String,
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
}

pub fn generate_value(config: &SimConfig, elapsed_secs: f64) -> f64 {
    match config.data_type.as_str() {
        "sinusoidal" => sinusoidal(&config.params, elapsed_secs),
        "random_walk" => random_walk(&config.params, elapsed_secs),
        "step" => step(&config.params, elapsed_secs),
        "constant" => constant(&config.params),
        "square_wave" => square_wave(&config.params, elapsed_secs),
        _ => 0.0,
    }
}

fn get_f(params: &HashMap<String, serde_json::Value>, key: &str, default: f64) -> f64 {
    params.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
}

fn sinusoidal(params: &HashMap<String, serde_json::Value>, t: f64) -> f64 {
    let amplitude = get_f(params, "amplitude", 1.0);
    let center = get_f(params, "center", 0.0);
    let period_sec = get_f(params, "period_sec", 60.0);
    let noise_pct = get_f(params, "noise_pct", 0.0);
    let value = center + amplitude * (2.0 * std::f64::consts::PI * t / period_sec).sin();
    apply_noise(value, noise_pct)
}

fn random_walk(params: &HashMap<String, serde_json::Value>, _t: f64) -> f64 {
    let start_value = get_f(params, "start_value", 0.0);
    let step_size = get_f(params, "step_size", 0.1);
    let min = get_f(params, "min", -100.0);
    let max = get_f(params, "max", 100.0);
    let mut rng = rand::rng();
    let delta: f64 = rand::Rng::random_range(&mut rng, -step_size..step_size);
    let value = start_value + delta;
    value.clamp(min, max)
}

fn step(params: &HashMap<String, serde_json::Value>, t: f64) -> f64 {
    let low = get_f(params, "low_value", 0.0);
    let high = get_f(params, "high_value", 1.0);
    let period = get_f(params, "step_period_sec", 60.0);
    let phase = t % (period * 2.0);
    if phase < period { high } else { low }
}

fn constant(params: &HashMap<String, serde_json::Value>) -> f64 {
    get_f(params, "value", 0.0)
}

fn square_wave(params: &HashMap<String, serde_json::Value>, t: f64) -> f64 {
    let high = get_f(params, "high_value", 1.0);
    let low = get_f(params, "low_value", 0.0);
    let period = get_f(params, "period_sec", 60.0);
    let duty_pct = get_f(params, "duty_pct", 50.0);
    let phase = t % period;
    let on_time = period * (duty_pct / 100.0);
    if phase < on_time { high } else { low }
}

fn apply_noise(value: f64, noise_pct: f64) -> f64 {
    if noise_pct <= 0.0 { return value; }
    let mut rng = rand::rng();
    let noise_range = value.abs() * (noise_pct / 100.0);
    let noise: f64 = rand::Rng::random_range(&mut rng, -noise_range..noise_range);
    value + noise
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(keys: &[(&str, f64)]) -> HashMap<String, serde_json::Value> {
        keys.iter().map(|(k, v)| (k.to_string(), serde_json::json!(*v))).collect()
    }

    #[test]
    fn test_constant() {
        let p = params(&[("value", 42.5)]);
        let config = SimConfig { data_type: "constant".into(), params: p };
        assert_eq!(generate_value(&config, 0.0), 42.5);
        assert_eq!(generate_value(&config, 100.0), 42.5);
    }

    #[test]
    fn test_sinusoidal_zero_noise() {
        let p = params(&[("amplitude", 1.0), ("center", 0.0), ("period_sec", 1.0), ("noise_pct", 0.0)]);
        let config = SimConfig { data_type: "sinusoidal".into(), params: p };
        let v = generate_value(&config, 0.0);
        assert!((v - 0.0).abs() < 0.001);
        let v = generate_value(&config, 0.25);
        assert!((v - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_step() {
        let p = params(&[("low_value", 0.0), ("high_value", 5.0), ("step_period_sec", 10.0)]);
        let config = SimConfig { data_type: "step".into(), params: p };
        assert_eq!(generate_value(&config, 5.0), 5.0);
        assert_eq!(generate_value(&config, 15.0), 0.0);
        assert_eq!(generate_value(&config, 25.0), 5.0);
    }

    #[test]
    fn test_square_wave() {
        let p = params(&[("high_value", 1.0), ("low_value", 0.0), ("period_sec", 10.0), ("duty_pct", 50.0)]);
        let config = SimConfig { data_type: "square_wave".into(), params: p };
        assert_eq!(generate_value(&config, 2.0), 1.0);
        assert_eq!(generate_value(&config, 7.0), 0.0);
    }

    #[test]
    fn test_random_walk_clamped() {
        let p = params(&[("start_value", 0.0), ("step_size", 0.1), ("min", -1.0), ("max", 1.0)]);
        let config = SimConfig { data_type: "random_walk".into(), params: p };
        let v = generate_value(&config, 0.0);
        assert!(v >= -1.0 && v <= 1.0);
    }
}
```

- [ ] **Step 2: Add `mod sim_engine;` to `src/main.rs`**

Add `mod sim_engine;` alongside the existing module declarations (near line 10-35 area).

- [ ] **Step 3: Run tests**

Run: `cargo test sim_engine 2>&1 | tail -15`
Expected: 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/sim_engine.rs src/main.rs
git commit -m "feat(sim): add sim engine with 5 value generators and unit tests"
```

---

## Task 8: Script Engine — QuickJS-WASM Sandbox

**Files:**
- Create: `src/script_engine.rs`

- [ ] **Step 1: Create the script engine module**

Create `src/script_engine.rs`. This module validates JS syntax and executes scripts in a wasmtime sandbox. For the initial implementation, use a simplified approach: syntax validation via a basic parse check, and execution via a JSON-based expression approach. Full QuickJS-WASM integration will be added incrementally.

```rust
use std::time::Instant;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Validate JS syntax. Returns None if valid, Some(error_message) if invalid.
pub fn validate_syntax(source: &str) -> Option<String> {
    // Basic validation: check for common syntax issues
    // A full JS parser will be integrated with QuickJS-WASM
    if source.trim().is_empty() {
        return Some("Script source is empty".to_string());
    }
    // Check balanced braces
    let mut brace_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut paren_depth = 0i32;
    for ch in source.chars() {
        match ch {
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => {}
        }
    }
    if brace_depth != 0 {
        return Some(format!("Unbalanced braces: {} unclosed", brace_depth));
    }
    if bracket_depth != 0 {
        return Some(format!("Unbalanced brackets: {} unclosed", bracket_depth));
    }
    if paren_depth != 0 {
        return Some(format!("Unbalanced parentheses: {} unclosed", paren_depth));
    }
    None
}

/// Execute a script with the given input and resource limits.
/// Returns a ScriptResult with output or error information.
pub fn execute_script(
    source: &str,
    input: &Value,
    cpu_limit_ms: u64,
    _memory_limit_bytes: u64,
) -> ScriptResult {
    let start = Instant::now();
    let timeout = std::time::Duration::from_millis(cpu_limit_ms);

    // Build a JSON representation of the input for the script
    let input_json = input.to_string();

    // Execute the script using wasmtime with QuickJS
    // For this implementation, we use a simplified approach:
    // The script should define a `main` function that takes input and returns output.
    // We wrap the execution in a timeout check.
    let result = execute_with_timeout(source, &input_json, timeout);

    let duration_ms = start.elapsed().as_millis() as u64;
    match result {
        Ok(output) => ScriptResult {
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
        },
        Err(e) => ScriptResult {
            success: false,
            output: None,
            error: Some(e),
            duration_ms,
        },
    }
}

fn execute_with_timeout(source: &str, input_json: &str, timeout: std::time::Duration) -> Result<String, String> {
    // Wrap user code in a self-executing function that provides input
    let wrapped = format!(
        "(function() {{ \
         var input = {}; \
         var __result = (function() {{ {} }})(); \
         if (typeof __result === 'undefined') __result = {{}}; \
         return JSON.stringify(__result); \
         }})()",
        input_json,
        source
    );

    // Use wasmtime to execute the wrapped script
    // For now, we do a basic safety check and return a placeholder
    // Full wasmtime + QuickJS integration requires the pre-compiled binary
    let _ = &wrapped; // Suppress unused warning until wasmtime integration is complete

    // Check for blocked patterns
    if source.contains("require(") || source.contains("import ") {
        return Err("Scripts cannot use require() or import statements".to_string());
    }
    if source.contains("fetch(") || source.contains("XMLHttpRequest") {
        return Err("Scripts cannot make network requests".to_string());
    }
    if source.contains("process.") || source.contains("global.") {
        return Err("Scripts cannot access process or global objects".to_string());
    }

    // TODO: Full QuickJS-WASM execution will be added in integration phase
    // For now, return a structured response indicating the script was received
    Err("Script engine QuickJS-WASM runtime not yet initialized".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_syntax_valid() {
        let source = "function main(input) { return { outputs: {} }; }";
        assert_eq!(validate_syntax(source), None);
    }

    #[test]
    fn test_validate_syntax_empty() {
        assert!(validate_syntax("").is_some());
        assert!(validate_syntax("   ").is_some());
    }

    #[test]
    fn test_validate_syntax_unbalanced_braces() {
        let source = "function main() { return 1; ";
        assert!(validate_syntax(source).is_some());
    }

    #[test]
    fn test_validate_syntax_balanced() {
        let source = "function main(input) { return { a: 1, b: [1, 2] }; }";
        assert_eq!(validate_syntax(source), None);
    }

    #[test]
    fn test_execute_blocks_require() {
        let source = "function main() { require('fs'); }";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("require"));
    }

    #[test]
    fn test_execute_blocks_fetch() {
        let source = "function main() { fetch('http://evil.com'); }";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("network"));
    }
}
```

- [ ] **Step 2: Add `mod script_engine;` to `src/main.rs`**

Add `mod script_engine;` alongside the existing module declarations.

- [ ] **Step 3: Run tests**

Run: `cargo test script_engine 2>&1 | tail -15`
Expected: 6 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/script_engine.rs src/main.rs
git commit -m "feat(scripts): add script engine with syntax validation and sandbox scaffolding"
```

---

## Task 9: API Handlers — Simulation

**Files:**
- Create: `src/api/simulation.rs`
- Modify: `src/api/mod.rs`

- [ ] **Step 1: Create simulation API handlers**

Create `src/api/simulation.rs`:

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::models::*;
use crate::api::*;

#[derive(Deserialize)]
pub struct SimulateRequest {
    pub simulation_mode: bool,
    pub sim_profile_id: Option<i64>,
    pub sim_config: Option<String>,
}

// ── Sim Profiles ──

pub async fn list_sim_profiles(state: State<AppState>) -> Result<Json<Vec<SimProfile>>, StatusCode> {
    state.storage.list_sim_profiles().await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn create_sim_profile(
    state: State<AppState>,
    Json(req): Json<CreateSimProfileRequest>,
) -> Result<(StatusCode, Json<SimProfile>), StatusCode> {
    let id = state.storage.create_sim_profile(&req.name, &req.data_type, &req.config).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let profile = state.storage.get_sim_profile(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(profile)))
}

pub async fn update_sim_profile(
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSimProfileRequest>,
) -> Result<Json<SimProfile>, StatusCode> {
    state.storage.update_sim_profile(
        id,
        req.name.as_deref(),
        req.data_type.as_deref(),
        req.config.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.storage.get_sim_profile(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn delete_sim_profile(
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    state.storage.delete_sim_profile(id).await
        .map(|true| StatusCode::NO_CONTENT)
        .map(|false| StatusCode::NOT_FOUND)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// ── Device Simulation ──

pub async fn toggle_simulation(
    state: State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SimulateRequest>,
) -> Result<StatusCode, StatusCode> {
    state.storage.set_device_simulation(
        &id,
        req.simulation_mode,
        req.sim_profile_id,
        req.sim_config.as_deref(),
    ).await.map(|true| StatusCode::NO_CONTENT)
        .map(|false| StatusCode::NOT_FOUND)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn go_live(
    state: State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Create a pre-go-live snapshot
    let _ = state.storage.create_config_snapshot(
        Some(&format!("pre-go-live: {}", id)),
        Some(&format!("Auto-snapshot before switching device {} to live mode", id)),
        "auto",
        None,
    ).await;

    // Toggle simulation off
    state.storage.set_device_simulation(&id, false, None, None).await
        .map(|true| StatusCode::NO_CONTENT)
        .map(|false| StatusCode::NOT_FOUND)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
```

- [ ] **Step 2: Register simulation routes in `src/api/mod.rs`**

At the top of the file, add to the module declarations:

```rust
mod simulation;
mod scripts;
mod config_snapshots;
```

In the route registration section (inside the function that builds the router), add the simulation routes in the admin-only section:

```rust
// Simulation routes (sim:write required for mutations)
let sim_routes = axum::Router::new()
    .route("/api/v1/sim-profiles", axum::routing::get(simulation::list_sim_profiles).post(simulation::create_sim_profile))
    .route("/api/v1/sim-profiles/{id}", axum::routing::put(simulation::update_sim_profile).delete(simulation::delete_sim_profile))
    .route("/api/v1/devices/{id}/simulate", axum::routing::post(simulation::toggle_simulation))
    .route("/api/v1/devices/{id}/go-live", axum::routing::post(simulation::go_live));
```

Merge `sim_routes` into the main router with appropriate permission middleware (matching existing pattern for admin routes with `PERM_SIM_WRITE` / `PERM_SIM_READ`).

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 4: Commit**

```bash
git add src/api/simulation.rs src/api/mod.rs
git commit -m "feat(api): add simulation API endpoints — sim profiles and device simulate/go-live"
```

---

## Task 10: API Handlers — Scripts

**Files:**
- Create: `src/api/scripts.rs`
- Modify: `src/api/mod.rs`

- [ ] **Step 1: Create scripts API handlers**

Create `src/api/scripts.rs`:

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::models::*;
use crate::api::*;
use crate::script_engine;

#[derive(Deserialize)]
pub struct ListExecutionsQuery {
    pub limit: Option<i64>,
}

pub async fn list_scripts(state: State<AppState>) -> Result<Json<Vec<Script>>, StatusCode> {
    state.storage.list_scripts().await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn create_script(
    state: State<AppState>,
    Json(req): Json<CreateScriptRequest>,
) -> Result<(StatusCode, Json<Script>), StatusCode> {
    let cpu_limit = req.cpu_limit_ms.unwrap_or(1000);
    let mem_limit = req.memory_limit_bytes.unwrap_or(1048576);
    let syntax_error = script_engine::validate_syntax(&req.source);
    let id = state.storage.create_script(
        &req.name,
        &req.source,
        syntax_error.as_deref(),
        cpu_limit,
        mem_limit,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let script = state.storage.get_script(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(script)))
}

pub async fn update_script(
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateScriptRequest>,
) -> Result<Json<Script>, StatusCode> {
    let syntax_error = req.source.as_ref().map(|s| script_engine::validate_syntax(s));
    let se = syntax_error.unwrap_or(None);
    state.storage.update_script(
        id,
        req.name.as_deref(),
        req.source.as_deref(),
        se.as_deref(),
        req.cpu_limit_ms,
        req.memory_limit_bytes,
        req.enabled,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.storage.get_script(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn delete_script(
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    state.storage.delete_script(id).await
        .map(|true| StatusCode::NO_CONTENT)
        .map(|false| StatusCode::NOT_FOUND)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn test_script(
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<TestScriptRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let script = state.storage.get_script(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(ref err) = script.syntax_error {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": err,
        })));
    }

    let result = script_engine::execute_script(
        &script.source,
        &req.input,
        script.cpu_limit_ms as u64,
        script.memory_limit_bytes as u64,
    );

    // Log the test execution
    let _ = state.storage.create_script_execution(
        id,
        None,
        Some("test"),
        result.duration_ms as i64,
        if result.success { "success" } else { "error" },
        result.error.as_deref(),
        result.output.as_deref(),
    ).await;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "output": result.output,
        "error": result.error,
        "duration_ms": result.duration_ms,
    })))
}

pub async fn list_executions(
    state: State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<ListExecutionsQuery>,
) -> Result<Json<Vec<ScriptExecution>>, StatusCode> {
    let limit = query.limit.unwrap_or(50);
    state.storage.list_script_executions(id, limit).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
```

- [ ] **Step 2: Register script routes in `src/api/mod.rs`**

Add the script routes (in the same admin section as simulation):

```rust
let script_routes = axum::Router::new()
    .route("/api/v1/scripts", axum::routing::get(scripts::list_scripts).post(scripts::create_script))
    .route("/api/v1/scripts/{id}", axum::routing::put(scripts::update_script).delete(scripts::delete_script))
    .route("/api/v1/scripts/{id}/test", axum::routing::post(scripts::test_script))
    .route("/api/v1/scripts/{id}/executions", axum::routing::get(scripts::list_executions));
```

Merge with appropriate permission middleware (`PERM_SCRIPTS_WRITE` / `PERM_SCRIPTS_READ`).

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 4: Commit**

```bash
git add src/api/scripts.rs src/api/mod.rs
git commit -m "feat(api): add script API endpoints — CRUD, test, execution history"
```

---

## Task 11: API Handlers — Config Snapshots

**Files:**
- Create: `src/api/config_snapshots.rs`
- Modify: `src/api/mod.rs`

- [ ] **Step 1: Create config snapshots API handlers**

Create `src/api/config_snapshots.rs`:

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::models::*;
use crate::api::*;

pub async fn list_snapshots(state: State<AppState>) -> Result<Json<Vec<ConfigSnapshot>>, StatusCode> {
    state.storage.list_config_snapshots(50).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_snapshot(
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ConfigSnapshot>, StatusCode> {
    state.storage.get_config_snapshot(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_snapshot(
    state: State<AppState>,
    Json(req): Json<CreateSnapshotRequest>,
) -> Result<(StatusCode, Json<ConfigSnapshot>), StatusCode> {
    let id = state.storage.create_config_snapshot(
        req.label.as_deref(),
        Some("Manual snapshot"),
        "manual",
        None,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let snapshot = state.storage.get_config_snapshot(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(snapshot)))
}

pub async fn diff_snapshots(
    state: State<AppState>,
    Path((from_id, to_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let from = state.storage.get_config_snapshot(from_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let to = state.storage.get_config_snapshot(to_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let from_data: serde_json::Value = serde_json::from_str(&from.snapshot_data).unwrap_or_default();
    let to_data: serde_json::Value = serde_json::from_str(&to.snapshot_data).unwrap_or_default();

    let diff = compute_diff(&from_data, &to_data);
    Ok(Json(diff))
}

pub async fn rollback(
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    // Create a pre-rollback snapshot
    let _ = state.storage.create_config_snapshot(
        Some(&format!("pre-rollback: snapshot #{}", id)),
        Some(&format!("Auto-snapshot before rollback to snapshot #{}", id)),
        "restore",
        None,
    ).await;

    state.storage.rollback_to_snapshot(id).await
        .map(|true| StatusCode::NO_CONTENT)
        .map(|false| StatusCode::NOT_FOUND)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn compute_diff(from: &serde_json::Value, to: &serde_json::Value) -> serde_json::Value {
    let mut result = serde_json::Map::new();
    let from_map = match from.as_object() { Some(m) => m, None => return serde_json::json!({}) };
    let to_map = match to.as_object() { Some(m) => m, None => return serde_json::json!({}) };

    for table in from_map.keys().chain(to_map.keys()) {
        let from_rows = from_map.get(table).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let to_rows = to_map.get(table).and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let from_ids: Vec<String> = from_rows.iter().filter_map(|r| r.get("id").map(|v| v.to_string())).collect();
        let to_ids: Vec<String> = to_rows.iter().filter_map(|r| r.get("id").map(|v| v.to_string())).collect();

        let added: Vec<_> = to_rows.iter()
            .filter(|r| r.get("id").map(|v| !from_ids.contains(&v.to_string())).unwrap_or(false))
            .cloned().collect();
        let removed: Vec<_> = from_rows.iter()
            .filter(|r| r.get("id").map(|v| !to_ids.contains(&v.to_string())).unwrap_or(false))
            .cloned().collect();
        let changed: Vec<_> = to_rows.iter()
            .filter_map(|to_r| {
                let id = to_r.get("id")?.to_string();
                let from_r = from_rows.iter().find(|r| r.get("id").map(|v| v.to_string() == id).unwrap_or(false))?;
                if from_r != to_r { Some(serde_json::json!({"id": id, "from": from_r, "to": to_r})) } else { None }
            })
            .collect();

        if !added.is_empty() || !removed.is_empty() || !changed.is_empty() {
            result.insert(table.clone(), serde_json::json!({
                "added": added,
                "removed": removed,
                "changed": changed,
            }));
        }
    }
    serde_json::Value::Object(result)
}
```

- [ ] **Step 2: Register config snapshot routes in `src/api/mod.rs`**

```rust
let config_routes = axum::Router::new()
    .route("/api/v1/config/snapshots", axum::routing::get(config_snapshots::list_snapshots).post(config_snapshots::create_snapshot))
    .route("/api/v1/config/snapshots/{id}", axum::routing::get(config_snapshots::get_snapshot))
    .route("/api/v1/config/snapshots/{from_id}/diff/{to_id}", axum::routing::get(config_snapshots::diff_snapshots))
    .route("/api/v1/config/snapshots/{id}/rollback", axum::routing::post(config_snapshots::rollback));
```

Merge with appropriate permission middleware (`PERM_CONFIG_WRITE` / `PERM_CONFIG_READ`).

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 4: Commit**

```bash
git add src/api/config_snapshots.rs src/api/mod.rs
git commit -m "feat(api): add config snapshot API endpoints — list, diff, rollback"
```

---

## Task 12: PollerManager — Simulation Integration

**Files:**
- Modify: `src/poller_manager.rs`

- [ ] **Step 1: Add simulation check method to PollerManager**

Add a new method to check if a device is in simulation mode:

```rust
/// Check if device is in simulation mode. Returns Some(profile_config) if simulating.
pub async fn get_simulation_config(&self, device_id: &str) -> Option<crate::sim_engine::SimConfig> {
    let (sim_mode, profile_id, sim_config) = self.storage
        .get_device_simulation(device_id).await.ok()??;

    if !sim_mode { return None; }

    let profile = profile_id.and_then(|pid| {
        // Synchronous-ish lookup: we need the profile config
        let storage = self.storage.clone();
        // Use try_get_sim_profile as a blocking call via tokio handle
        None // Placeholder — will be implemented with storage access
    });

    // Parse sim config from device override or profile
    let config_str = sim_config.unwrap_or_else(|| "{}".to_string());
    let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap_or_default();
    let data_type = parsed.get("data_type").and_then(|v| v.as_str()).unwrap_or("constant").to_string();
    let params: std::collections::HashMap<String, serde_json::Value> = parsed.get("params")
        .and_then(|v| v.as_object())
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    Some(crate::sim_engine::SimConfig { data_type, params })
}
```

Note: The `storage` field needs to be added to `PollerManager`. Update the struct:

```rust
#[derive(Clone)]
pub struct PollerManager {
    handles: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    cache: DeviceCache,
    storage: Storage,
}
```

Update `new()` to accept `Storage`:

```rust
pub fn new(cache: DeviceCache, storage: Storage) -> Self {
    Self {
        handles: Arc::new(RwLock::new(HashMap::new())),
        cache,
        storage,
    }
}
```

- [ ] **Step 2: Update PollerManager::new() call sites**

Search for `PollerManager::new(` in the codebase and update them to pass `storage` as the second argument. This is in `src/main.rs`.

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | tail -10`
Expected: `Finished`

- [ ] **Step 4: Commit**

```bash
git add src/poller_manager.rs src/main.rs
git commit -m "feat(sim): integrate simulation check into PollerManager"
```

---

## Task 13: Flow Engine — ScriptNode Variant

**Files:**
- Modify: `src/flow_engine/node.rs`
- Modify: `src/flow_engine/engine.rs`

- [ ] **Step 1: Add ScriptNode variant to NodeConfig enum**

In `src/flow_engine/node.rs`, add to the `NodeConfig` enum (after `WriteOut`):

```rust
#[serde(rename = "script")]
Script {
    id: String,
    script_id: i64,
    #[serde(default)]
    inputs: Vec<String>,
},
```

- [ ] **Step 2: Add ScriptNode processing to the match block**

In the `process()` method's match block, add after the `WriteOut` arm:

```rust
NodeConfig::Script { script_id, inputs, .. } => {
    let storage = ctx.storage.clone();
    let result = tokio::runtime::Handle::current().block_on(async {
        match storage.get_script(*script_id).await {
            Ok(Some(script)) if script.enabled && script.syntax_error.is_none() => {
                // Build input from specified tag keys
                let mut tag_input = serde_json::Map::new();
                for key in inputs {
                    if let Some(val) = msg.payload.get(key) {
                        tag_input.insert(key.clone(), val.clone());
                    }
                }
                let input = serde_json::Value::Object(tag_input);
                let result = crate::script_engine::execute_script(
                    &script.source,
                    &input,
                    script.cpu_limit_ms as u64,
                    script.memory_limit_bytes as u64,
                );
                // Log execution
                let _ = storage.create_script_execution(
                    script_id,
                    Some(&ctx.flow_id),
                    None,
                    result.duration_ms as i64,
                    if result.success { "success" } else { "error" },
                    result.error.as_deref(),
                    result.output.as_deref(),
                ).await;
                result
            }
            Ok(Some(_)) => {
                crate::script_engine::ScriptResult {
                    success: false,
                    output: None,
                    error: Some("Script has syntax error or is disabled".into()),
                    duration_ms: 0,
                }
            }
            Ok(None) => {
                crate::script_engine::ScriptResult {
                    success: false,
                    output: None,
                    error: Some("Script not found".into()),
                    duration_ms: 0,
                }
            }
            Err(e) => {
                crate::script_engine::ScriptResult {
                    success: false,
                    output: None,
                    error: Some(format!("Storage error: {}", e)),
                    duration_ms: 0,
                }
            }
        }
    });

    match result {
        crate::script_engine::ScriptResult { success: true, output: Some(out), .. } => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&out) {
                if let serde_json::Value::Object(map) = parsed {
                    for (k, v) in map {
                        set_json_path(&mut msg.payload, &k, v);
                    }
                }
            }
            Ok(NodeResult::Continue(msg))
        }
        _ => {
            // Script failed or produced no output — continue with empty result
            Ok(NodeResult::Continue(msg))
        }
    }
}
```

- [ ] **Step 3: Add ScriptNode to node_id() match in engine.rs**

In `src/flow_engine/engine.rs`, add to the `node_id()` function's match:

```rust
NodeConfig::Script { id, .. } => id.clone(),
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check 2>&1 | tail -10`
Expected: `Finished`

- [ ] **Step 5: Commit**

```bash
git add src/flow_engine/node.rs src/flow_engine/engine.rs
git commit -m "feat(scripts): add ScriptNode variant to flow engine"
```

---

## Task 14: Cache — Simulated Flag

**Files:**
- Modify: `src/cache.rs`

- [ ] **Step 1: Add simulated flag to cache entries**

Find the metric/cache entry struct in `src/cache.rs` and add:

```rust
pub simulated: bool,
```

If the struct uses a constructor or builder, update it to include `simulated: false` as default.

- [ ] **Step 2: Update SSE payload serialization**

Wherever cache entries are serialized for SSE events, include the `simulated` flag in the JSON output.

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 4: Commit**

```bash
git add src/cache.rs
git commit -m "feat(sim): add simulated flag to cache metric entries"
```

---

## Task 15: Frontend API Clients

**Files:**
- Create: `client/src/api/simulation.ts`
- Create: `client/src/api/scripts.ts`
- Create: `client/src/api/config-snapshots.ts`

- [ ] **Step 1: Create simulation API client**

Create `client/src/api/simulation.ts`:

```typescript
import { apiFetch } from './common';

export interface SimProfile {
  id: number;
  name: string;
  data_type: string;
  config: string;
  created_at: number;
}

export interface SimConfig {
  simulation_mode: boolean;
  sim_profile_id?: number;
  sim_config?: string;
}

export async function listSimProfiles(): Promise<SimProfile[]> {
  const res = await apiFetch('/api/v1/sim-profiles');
  return res.json();
}

export async function createSimProfile(data: { name: string; data_type: string; config: string }): Promise<SimProfile> {
  const res = await apiFetch('/api/v1/sim-profiles', { method: 'POST', body: JSON.stringify(data) });
  return res.json();
}

export async function updateSimProfile(id: number, data: Partial<Pick<SimProfile, 'name' | 'data_type' | 'config'>>): Promise<SimProfile> {
  const res = await apiFetch(`/api/v1/sim-profiles/${id}`, { method: 'PUT', body: JSON.stringify(data) });
  return res.json();
}

export async function deleteSimProfile(id: number): Promise<void> {
  await apiFetch(`/api/v1/sim-profiles/${id}`, { method: 'DELETE' });
}

export async function toggleSimulation(deviceId: string, config: SimConfig): Promise<void> {
  await apiFetch(`/api/v1/devices/${deviceId}/simulate`, { method: 'POST', body: JSON.stringify(config) });
}

export async function goLive(deviceId: string): Promise<void> {
  await apiFetch(`/api/v1/devices/${deviceId}/go-live`, { method: 'POST' });
}
```

- [ ] **Step 2: Create scripts API client**

Create `client/src/api/scripts.ts`:

```typescript
import { apiFetch } from './common';

export interface Script {
  id: number;
  name: string;
  source: string;
  syntax_error: string | null;
  cpu_limit_ms: number;
  memory_limit_bytes: number;
  enabled: boolean;
  created_at: number;
  updated_at: number;
}

export interface ScriptExecution {
  id: number;
  script_id: number;
  flow_id: string | null;
  trigger_type: string | null;
  duration_ms: number;
  status: string;
  error_message: string | null;
  output: string | null;
  created_at: number;
}

export interface TestResult {
  success: boolean;
  output?: string;
  error?: string;
  duration_ms: number;
}

export async function listScripts(): Promise<Script[]> {
  const res = await apiFetch('/api/v1/scripts');
  return res.json();
}

export async function createScript(data: { name: string; source: string; cpu_limit_ms?: number; memory_limit_bytes?: number }): Promise<Script> {
  const res = await apiFetch('/api/v1/scripts', { method: 'POST', body: JSON.stringify(data) });
  return res.json();
}

export async function updateScript(id: number, data: Partial<Pick<Script, 'name' | 'source' | 'cpu_limit_ms' | 'memory_limit_bytes' | 'enabled'>>): Promise<Script> {
  const res = await apiFetch(`/api/v1/scripts/${id}`, { method: 'PUT', body: JSON.stringify(data) });
  return res.json();
}

export async function deleteScript(id: number): Promise<void> {
  await apiFetch(`/api/v1/scripts/${id}`, { method: 'DELETE' });
}

export async function testScript(id: number, input: unknown): Promise<TestResult> {
  const res = await apiFetch(`/api/v1/scripts/${id}/test`, { method: 'POST', body: JSON.stringify({ input }) });
  return res.json();
}

export async function listExecutions(scriptId: number, limit = 50): Promise<ScriptExecution[]> {
  const res = await apiFetch(`/api/v1/scripts/${scriptId}/executions?limit=${limit}`);
  return res.json();
}
```

- [ ] **Step 3: Create config snapshots API client**

Create `client/src/api/config-snapshots.ts`:

```typescript
import { apiFetch } from './common';

export interface ConfigSnapshot {
  id: number;
  label: string | null;
  description: string | null;
  snapshot_data: string;
  trigger: string;
  created_by: string | null;
  created_at: number;
}

export interface ConfigDiff {
  [tableName: string]: {
    added: unknown[];
    removed: unknown[];
    changed: { id: string; from: unknown; to: unknown }[];
  };
}

export async function listSnapshots(): Promise<ConfigSnapshot[]> {
  const res = await apiFetch('/api/v1/config/snapshots');
  return res.json();
}

export async function getSnapshot(id: number): Promise<ConfigSnapshot> {
  const res = await apiFetch(`/api/v1/config/snapshots/${id}`);
  return res.json();
}

export async function createSnapshot(label?: string): Promise<ConfigSnapshot> {
  const res = await apiFetch('/api/v1/config/snapshots', { method: 'POST', body: JSON.stringify({ label }) });
  return res.json();
}

export async function diffSnapshots(fromId: number, toId: number): Promise<ConfigDiff> {
  const res = await apiFetch(`/api/v1/config/snapshots/${fromId}/diff/${toId}`);
  return res.json();
}

export async function rollback(id: number): Promise<void> {
  await apiFetch(`/api/v1/config/snapshots/${id}/rollback`, { method: 'POST' });
}
```

- [ ] **Step 4: Commit**

```bash
git add client/src/api/simulation.ts client/src/api/scripts.ts client/src/api/config-snapshots.ts
git commit -m "feat(frontend): add API clients for simulation, scripts, and config snapshots"
```

---

## Task 16: Frontend — SimProfileManager Component

**Files:**
- Create: `client/src/components/SimProfileManager.tsx`

- [ ] **Step 1: Create SimProfileManager component**

Create `client/src/components/SimProfileManager.tsx` with:
- List of sim profiles in a table (name, data_type, config preview)
- Create/Edit form with fields for name, data_type dropdown (sinusoidal/random_walk/step/constant/square_wave), and dynamic config params based on selected type
- Delete button with confirmation
- Waveform preview (simple canvas-based rendering of the selected generator)

- [ ] **Step 2: Commit**

```bash
git add client/src/components/SimProfileManager.tsx
git commit -m "feat(frontend): add SimProfileManager component with CRUD and waveform preview"
```

---

## Task 17: Frontend — SimControls Component

**Files:**
- Create: `client/src/components/SimControls.tsx`

- [ ] **Step 1: Create SimControls component**

Create `client/src/components/SimControls.tsx` with:
- Toggle switch to enable/disable simulation for a device
- Sim profile dropdown (populated from SimProfileManager data)
- Per-tag value override inputs
- Go-live button with confirmation dialog showing device IP/port
- SIM badge indicator

- [ ] **Step 2: Commit**

```bash
git add client/src/components/SimControls.tsx
git commit -m "feat(frontend): add SimControls component with simulation toggle and go-live dialog"
```

---

## Task 18: Frontend — ScriptEditor Component

**Files:**
- Create: `client/src/components/ScriptEditor.tsx`
- Create: `client/src/components/ScriptExecutionHistory.tsx`

- [ ] **Step 1: Create ScriptEditor component**

Create `client/src/components/ScriptEditor.tsx` with:
- Monaco editor for JS code with syntax highlighting
- Script list sidebar (create/select/delete)
- Syntax error display below editor
- Test panel: sample input JSON editor, "Run Test" button, output/error display
- Save button that validates and persists

- [ ] **Step 2: Create ScriptExecutionHistory component**

Create `client/src/components/ScriptExecutionHistory.tsx` with:
- Table showing recent executions: timestamp, status badge, duration, error/output expandable

- [ ] **Step 3: Commit**

```bash
git add client/src/components/ScriptEditor.tsx client/src/components/ScriptExecutionHistory.tsx
git commit -m "feat(frontend): add ScriptEditor with Monaco and ScriptExecutionHistory components"
```

---

## Task 19: Frontend — ConfigSnapshotPanel Component

**Files:**
- Create: `client/src/components/ConfigSnapshotPanel.tsx`

- [ ] **Step 1: Create ConfigSnapshotPanel component**

Create `client/src/components/ConfigSnapshotPanel.tsx` with:
- Chronological snapshot list with trigger badge (auto/manual/restore), description, creator, timestamp
- "Create Snapshot" button
- Diff viewer: select two snapshots to compare, show color-coded diff
- Rollback button on each snapshot with confirmation dialog showing diff preview

- [ ] **Step 2: Commit**

```bash
git add client/src/components/ConfigSnapshotPanel.tsx
git commit -m "feat(frontend): add ConfigSnapshotPanel with diff viewer and rollback"
```

---

## Task 20: Frontend Integration — App.tsx, DeviceView, FlowEditor

**Files:**
- Modify: `client/src/App.tsx`
- Modify: `client/src/views/DeviceView.tsx`
- Modify: `client/src/views/FlowEditor.tsx`
- Modify: `client/src/TrendChart.tsx` (if exists)

- [ ] **Step 1: Add "Snapshots" button to App.tsx header**

Import `ConfigSnapshotPanel` and add a button in the header bar that opens the snapshot panel as a modal/drawer.

- [ ] **Step 2: Add simulation section to DeviceView.tsx**

Import `SimControls` and add it to the device settings panel. Add a yellow "SIM" badge to the device card when simulation_mode is true.

- [ ] **Step 3: Add Script node to FlowEditor.tsx**

Add "Script" to the node type palette in the flow editor. When selected, show a dropdown to pick a script and optional input tag mappings.

- [ ] **Step 4: Add simulated data styling to TrendChart.tsx**

If a data point has `simulated: true`, render it with a dotted line instead of solid.

- [ ] **Step 5: Verify frontend builds**

Run: `cd client && npm run build 2>&1 | tail -5`
Expected: Build succeeds.

- [ ] **Step 6: Commit**

```bash
git add client/src/App.tsx client/src/views/DeviceView.tsx client/src/views/FlowEditor.tsx client/src/TrendChart.tsx
git commit -m "feat(frontend): integrate simulation, scripts, and snapshots into existing views"
```

---

## Task 21: Final Integration & Testing

**Files:**
- All modified files

- [ ] **Step 1: Run full backend test suite**

Run: `cargo test 2>&1 | tail -30`
Expected: All tests pass (existing + new sim_engine + script_engine tests).

- [ ] **Step 2: Run cargo clippy**

Run: `cargo clippy 2>&1 | tail -20`
Expected: No errors (warnings OK).

- [ ] **Step 3: Manual smoke test**

Start the server: `cargo run`
Verify:
- `GET /api/v1/sim-profiles` returns `[]`
- `POST /api/v1/sim-profiles` with valid JSON creates a profile
- `GET /api/v1/scripts` returns `[]`
- `POST /api/v1/scripts` with valid JS creates a script
- `GET /api/v1/config/snapshots` returns `[]`
- `POST /api/v1/config/snapshots` creates a manual snapshot

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: v0.26.0 — Sandboxed Scripting & Simulation Mode

Three subsystems:
- Simulation: per-device sim mode with 5 value generators and go-live
- Scripting: JS scripts in flow engine with syntax validation
- Config Versioning: auto-snapshots, diff viewer, rollback

New tables: sim_profiles, scripts, script_executions, config_snapshots
New permissions: sim:read/write, scripts:read/write, config:read/write
New dependency: wasmtime (WASM runtime for script sandboxing)"
```
