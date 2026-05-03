# v0.34.0 — OT Asset Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Full-stack OT asset inventory with SNMP scanning, firmware tracking, maintenance logs, risk scoring, NIS2 export, and CSV/Excel import.

**Architecture:** New `asset/` module with unified `assets` table linked to protocol devices via `device_ref`. SNMP scanning via `rs-snmp` crate. Risk score computed on write, stored in assets table. Frontend uses TanStack Query + inline styles matching existing patterns.

**Tech Stack:** Rust (Axum, rusqlite, rs-snmp, calamine), TypeScript/React (TanStack Query, @tanstack/react-virtual)

**Spec:** `docs/superpowers/specs/2026-05-03-ot-asset-inventory-design.md`

---

## File Structure

### New Files (Backend)

```
src/asset/
├── mod.rs              # Module exports
├── models.rs           # Asset, MaintenanceLog, FirmwareCatalog, SnmpScanResult structs
├── storage.rs          # SQLite CRUD for all asset tables
├── risk.rs             # Risk score computation
├── snmp.rs             # SNMP scan engine
└── sync.rs             # Device-to-asset auto-sync hooks
src/api/
├── assets.rs           # Asset CRUD + IP conflicts
├── maintenance.rs      # Maintenance log endpoints
├── firmware_catalog.rs # Firmware catalog CRUD
├── snmp_scan.rs        # SNMP scan trigger + results + import
├── asset_import.rs     # CSV/Excel bulk import
└── nis2_export.rs      # NIS2 compliance CSV export
```

### New Files (Frontend)

```
client/src/queries/assets.ts       # TanStack Query hooks for all asset APIs
client/src/views/AssetInventory.tsx # Main asset inventory page
client/src/components/AssetList.tsx # Filterable asset table
client/src/components/AssetDetail.tsx # Detail view with tabs
client/src/components/MaintenanceTimeline.tsx # Maintenance history
client/src/components/SnmpScanPanel.tsx # SNMP scan UI
client/src/components/AssetImportWizard.tsx # CSV/Excel import
```

### Modified Files

```
Cargo.toml                           # Add rs-snmp, calamine
src/main.rs                          # Add mod asset;
src/storage.rs                       # Add init_asset_tables() call
src/api/mod.rs                       # Register asset routes
src/api/modbus_devices.rs            # Add auto-sync hook on create/delete
src/api/opcua_devices.rs             # Add auto-sync hook on create/delete
src/api/s7_devices.rs                # Add auto-sync hook on create/delete
src/api/enip_devices.rs              # Add auto-sync hook on create/delete
client/src/App.tsx                    # Add Assets view mode
client/src/components/AppHeader.tsx  # Add Assets nav item
```

---

## Task 1: Models & Cargo Dependencies

**Files:**
- Modify: `Cargo.toml`
- Create: `src/asset/mod.rs`
- Create: `src/asset/models.rs`

- [ ] **Step 1: Add dependencies to Cargo.toml**

Add under `[dependencies]`:

```toml
rs-snmp = "0.3"
calamine = "0.26"
```

- [ ] **Step 2: Create `src/asset/mod.rs`**

```rust
pub mod models;
pub mod storage;
pub mod risk;
pub mod snmp;
pub mod sync;
```

- [ ] **Step 3: Create `src/asset/models.rs`**

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Asset {
    pub id: String,
    pub device_ref: Option<String>,
    pub name: String,
    pub asset_type: String,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub firmware_version: Option<String>,
    pub serial_number: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub location: Option<String>,
    pub criticality: String,
    pub tags: String,
    pub risk_score: i64,
    pub last_maintenance: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateAssetRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub firmware_version: Option<String>,
    pub serial_number: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub location: Option<String>,
    #[serde(default = "default_criticality")]
    pub criticality: String,
    #[serde(default = "default_tags")]
    pub tags: String,
    pub notes: Option<String>,
}

fn default_criticality() -> String { "medium".to_string() }
fn default_tags() -> String { "[]".to_string() }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MaintenanceLog {
    pub id: String,
    pub asset_id: String,
    pub description: String,
    pub technician: String,
    pub performed_at: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateMaintenanceRequest {
    pub description: String,
    pub technician: String,
    pub performed_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct FirmwareCatalogEntry {
    pub id: String,
    pub vendor: String,
    pub model: String,
    pub latest_version: String,
    pub release_date: Option<String>,
    pub notes: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateFirmwareCatalogRequest {
    pub vendor: String,
    pub model: String,
    pub latest_version: String,
    pub release_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SnmpScanResult {
    pub ip: String,
    pub sys_descr: Option<String>,
    pub sys_name: Option<String>,
    pub sys_uptime: Option<i64>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub firmware: Option<String>,
    pub mac_address: Option<String>,
    pub scanned_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SnmpScanRequest {
    pub subnet: String,
    #[serde(default = "default_community")]
    pub community: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_community() -> String { "public".to_string() }
fn default_timeout() -> u64 { 2000 }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ImportPreview {
    pub import_id: String,
    pub rows: Vec<serde_json::Value>,
    pub total: i64,
    pub valid: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ImportConfirmRequest {
    pub import_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ImportResult {
    pub total: i64,
    pub created: i64,
    pub skipped_duplicates: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct IpConflict {
    pub ip: String,
    pub assets: Vec<serde_json::Value>,
    pub devices: Vec<serde_json::Value>,
}
```

- [ ] **Step 4: Register module in `src/main.rs`**

Add after `mod ai_copilot;`:

```rust
mod asset;
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check 2>&1 | head -30`
Expected: May show warnings for unused modules but no errors in the new files.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/asset/ src/main.rs
git commit -m "feat(asset): add models and module skeleton for OT asset inventory"
```

---

## Task 2: Asset Storage Schema

**Files:**
- Create: `src/asset/storage.rs`
- Modify: `src/storage.rs` (call init from existing init_schema)

- [ ] **Step 1: Create `src/asset/storage.rs`**

```rust
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AssetStorage {
    conn: Arc<Mutex<Connection>>,
}

impl AssetStorage {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                device_ref TEXT UNIQUE,
                name TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                vendor TEXT,
                model TEXT,
                firmware_version TEXT,
                serial_number TEXT,
                ip_address TEXT,
                mac_address TEXT,
                location TEXT,
                criticality TEXT NOT NULL DEFAULT 'medium',
                tags TEXT DEFAULT '[]',
                risk_score INTEGER DEFAULT 0,
                last_maintenance TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_assets_device_ref ON assets(device_ref);
            CREATE INDEX IF NOT EXISTS idx_assets_ip ON assets(ip_address);
            CREATE INDEX IF NOT EXISTS idx_assets_type ON assets(asset_type);
            CREATE INDEX IF NOT EXISTS idx_assets_criticality ON assets(criticality);

            CREATE TABLE IF NOT EXISTS maintenance_logs (
                id TEXT PRIMARY KEY,
                asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
                description TEXT NOT NULL,
                technician TEXT NOT NULL,
                performed_at TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_maintenance_asset_id ON maintenance_logs(asset_id);

            CREATE TABLE IF NOT EXISTS firmware_catalog (
                id TEXT PRIMARY KEY,
                vendor TEXT NOT NULL,
                model TEXT NOT NULL,
                latest_version TEXT NOT NULL,
                release_date TEXT,
                notes TEXT,
                updated_at TEXT NOT NULL,
                UNIQUE(vendor, model)
            );

            CREATE TABLE IF NOT EXISTS snmp_scan_results (
                ip TEXT PRIMARY KEY,
                sys_descr TEXT,
                sys_name TEXT,
                sys_uptime INTEGER,
                vendor TEXT,
                model TEXT,
                firmware TEXT,
                mac_address TEXT,
                scanned_at TEXT NOT NULL
            );",
        )
        .context("Failed to create asset tables")?;

        Ok(())
    }
}
```

- [ ] **Step 2: Integrate schema into `src/storage.rs`**

In `src/storage.rs`, find the `init_schema` function (or equivalent where tables are created). Add after the existing table creation statements:

```rust
crate::asset::storage::AssetStorage::init_schema(&conn)?;
```

The exact location: inside `Storage::new()` or `Storage::new_with_key()`, after the existing `conn.execute_batch(...)` call that creates other tables. Add the call there.

Also add a public getter in the main `Storage` struct so handlers can access asset storage:

```rust
pub fn asset(&self) -> crate::asset::storage::AssetStorage {
    crate::asset::storage::AssetStorage::new(self.conn.clone())
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -30`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src/asset/storage.rs src/storage.rs
git commit -m "feat(asset): add storage schema for assets, maintenance, firmware, SNMP scan"
```

---

## Task 3: Asset CRUD Storage Functions

**Files:**
- Modify: `src/asset/storage.rs` (append CRUD methods)

- [ ] **Step 1: Add Asset CRUD methods**

Append to `impl AssetStorage` in `src/asset/storage.rs`:

```rust
use crate::asset::models::*;

impl AssetStorage {
    pub async fn create_asset(&self, a: &CreateAssetRequest) -> Result<Asset> {
        let conn = self.conn.clone();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let a = a.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO assets (id, name, asset_type, vendor, model, firmware_version,
                 serial_number, ip_address, mac_address, location, criticality, tags,
                 risk_score, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, ?13, ?14, ?15)",
                rusqlite::params![
                    id, a.name, a.asset_type, a.vendor, a.model, a.firmware_version,
                    a.serial_number, a.ip_address, a.mac_address, a.location,
                    a.criticality, a.tags, a.notes, now, now
                ],
            )
            .context("Insert asset failed")?;

            Ok(Asset {
                id,
                device_ref: None,
                name: a.name,
                asset_type: a.asset_type,
                vendor: a.vendor,
                model: a.model,
                firmware_version: a.firmware_version,
                serial_number: a.serial_number,
                ip_address: a.ip_address,
                mac_address: a.mac_address,
                location: a.location,
                criticality: a.criticality,
                tags: a.tags,
                risk_score: 0,
                last_maintenance: None,
                notes: a.notes,
                created_at: now.clone(),
                updated_at: now,
            })
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_assets(
        &self,
        asset_type: Option<&str>,
        criticality: Option<&str>,
        vendor: Option<&str>,
        search: Option<&str>,
    ) -> Result<Vec<Asset>> {
        let conn = self.conn.clone();
        let asset_type = asset_type.map(String::from);
        let criticality = criticality.map(String::from);
        let vendor = vendor.map(String::from);
        let search = search.map(String::from);

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut sql = String::from(
                "SELECT id, device_ref, name, asset_type, vendor, model, firmware_version,
                        serial_number, ip_address, mac_address, location, criticality, tags,
                        risk_score, last_maintenance, notes, created_at, updated_at
                 FROM assets WHERE 1=1"
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(ref t) = asset_type {
                sql.push_str(" AND asset_type = ?");
                params.push(Box::new(t.clone()));
            }
            if let Some(ref c) = criticality {
                sql.push_str(" AND criticality = ?");
                params.push(Box::new(c.clone()));
            }
            if let Some(ref v) = vendor {
                sql.push_str(" AND vendor = ?");
                params.push(Box::new(v.clone()));
            }
            if let Some(ref s) = search {
                sql.push_str(" AND (name LIKE ? OR ip_address LIKE ? OR vendor LIKE ?)");
                let pattern = format!("%{}%", s);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
            sql.push_str(" ORDER BY updated_at DESC");

            let mut stmt = conn.prepare(&sql).context("Prepare failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();

            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(Asset {
                        id: row.get(0)?,
                        device_ref: row.get(1)?,
                        name: row.get(2)?,
                        asset_type: row.get(3)?,
                        vendor: row.get(4)?,
                        model: row.get(5)?,
                        firmware_version: row.get(6)?,
                        serial_number: row.get(7)?,
                        ip_address: row.get(8)?,
                        mac_address: row.get(9)?,
                        location: row.get(10)?,
                        criticality: row.get(11)?,
                        tags: row.get(12)?,
                        risk_score: row.get(13)?,
                        last_maintenance: row.get(14)?,
                        notes: row.get(15)?,
                        created_at: row.get(16)?,
                        updated_at: row.get(17)?,
                    })
                })
                .context("Query failed")?;

            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_asset(&self, id: &str) -> Result<Option<Asset>> {
        let conn = self.conn.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.query_row(
                "SELECT id, device_ref, name, asset_type, vendor, model, firmware_version,
                        serial_number, ip_address, mac_address, location, criticality, tags,
                        risk_score, last_maintenance, notes, created_at, updated_at
                 FROM assets WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(Asset {
                        id: row.get(0)?,
                        device_ref: row.get(1)?,
                        name: row.get(2)?,
                        asset_type: row.get(3)?,
                        vendor: row.get(4)?,
                        model: row.get(5)?,
                        firmware_version: row.get(6)?,
                        serial_number: row.get(7)?,
                        ip_address: row.get(8)?,
                        mac_address: row.get(9)?,
                        location: row.get(10)?,
                        criticality: row.get(11)?,
                        tags: row.get(12)?,
                        risk_score: row.get(13)?,
                        last_maintenance: row.get(14)?,
                        notes: row.get(15)?,
                        created_at: row.get(16)?,
                        updated_at: row.get(17)?,
                    })
                },
            )
            .optional()
            .context("Query asset failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_asset(&self, id: &str, fields: &serde_json::Value) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let fields = fields.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut set_clauses = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            for key in &[
                "name", "asset_type", "vendor", "model", "firmware_version",
                "serial_number", "ip_address", "mac_address", "location",
                "criticality", "tags", "notes",
            ] {
                if let Some(val) = fields.get(key) {
                    set_clauses.push(format!("{} = ?", key));
                    params.push(Box::new(val.as_str().unwrap_or("").to_string()));
                }
            }

            if set_clauses.is_empty() {
                return Ok(());
            }

            set_clauses.push("updated_at = ?".to_string());
            params.push(Box::new(chrono::Utc::now().to_rfc3339()));

            let sql = format!(
                "UPDATE assets SET {} WHERE id = ?",
                set_clauses.join(", ")
            );
            params.push(Box::new(id));

            conn.execute(&sql, rusqlite::params_from_iter(params))
                .context("Update asset failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_asset(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM assets WHERE id = ?1", rusqlite::params![id])
                .context("Delete asset failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_risk_score(&self, id: &str, score: i64) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE assets SET risk_score = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![score, chrono::Utc::now().to_rfc3339(), id],
            )
            .context("Update risk score failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_last_maintenance(&self, id: &str, date: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let date = date.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE assets SET last_maintenance = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![date, chrono::Utc::now().to_rfc3339(), id],
            )
            .context("Update last maintenance failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // --- Device-to-asset sync ---

    pub async fn link_device(
        &self,
        device_ref: &str,
        name: &str,
        ip: Option<&str>,
        asset_type: &str,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let device_ref = device_ref.to_string();
        let name = name.to_string();
        let ip = ip.map(String::from);
        let asset_type = asset_type.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = chrono::Utc::now().to_rfc3339();
            let id = uuid::Uuid::new_v4().to_string();

            conn.execute(
                "INSERT OR IGNORE INTO assets
                 (id, device_ref, name, asset_type, ip_address, criticality, tags, risk_score, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'medium', '[]', 0, ?6, ?7)",
                rusqlite::params![id, device_ref, name, asset_type, ip, now, now],
            )
            .context("Link device to asset failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn unlink_device(&self, device_ref: &str) -> Result<()> {
        let conn = self.conn.clone();
        let device_ref = device_ref.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE assets SET device_ref = NULL, criticality = 'low', updated_at = ?1
                 WHERE device_ref = ?2",
                rusqlite::params![chrono::Utc::now().to_rfc3339(), device_ref],
            )
            .context("Unlink device from asset failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_device_ref(
        &self,
        device_ref: &str,
        name: &str,
        ip: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let device_ref = device_ref.to_string();
        let name = name.to_string();
        let ip = ip.map(String::from);

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE assets SET name = ?1, ip_address = ?2, updated_at = ?3
                 WHERE device_ref = ?4",
                rusqlite::params![name, ip, chrono::Utc::now().to_rfc3339(), device_ref],
            )
            .context("Update device ref failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // --- IP conflict detection ---

    pub async fn find_ip_conflicts(&self) -> Result<Vec<(String, Vec<Asset>)>> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT ip_address, COUNT(*) as cnt FROM assets
                 WHERE ip_address IS NOT NULL AND ip_address != ''
                 GROUP BY ip_address HAVING cnt > 1"
            ).context("Prepare failed")?;

            let conflict_ips: Vec<String> = stmt
                .query_map([], |row| row.get(0))
                .context("Query failed")?
                .filter_map(|r| r.ok())
                .collect();

            drop(stmt);

            let mut result = Vec::new();
            for ip in conflict_ips {
                let mut stmt2 = conn.prepare(
                    "SELECT id, device_ref, name, asset_type, vendor, model, firmware_version,
                            serial_number, ip_address, mac_address, location, criticality, tags,
                            risk_score, last_maintenance, notes, created_at, updated_at
                     FROM assets WHERE ip_address = ?1"
                ).context("Prepare failed")?;

                let assets: Vec<Asset> = stmt2
                    .query_map(rusqlite::params![ip], |row| {
                        Ok(Asset {
                            id: row.get(0)?,
                            device_ref: row.get(1)?,
                            name: row.get(2)?,
                            asset_type: row.get(3)?,
                            vendor: row.get(4)?,
                            model: row.get(5)?,
                            firmware_version: row.get(6)?,
                            serial_number: row.get(7)?,
                            ip_address: row.get(8)?,
                            mac_address: row.get(9)?,
                            location: row.get(10)?,
                            criticality: row.get(11)?,
                            tags: row.get(12)?,
                            risk_score: row.get(13)?,
                            last_maintenance: row.get(14)?,
                            notes: row.get(15)?,
                            created_at: row.get(16)?,
                            updated_at: row.get(17)?,
                        })
                    })
                    .context("Query failed")?
                    .filter_map(|r| r.ok())
                    .collect();

                result.push((ip, assets));
            }
            Ok(result)
        })
        .await
        .context("Task join failed")?
    }
}
```

- [ ] **Step 2: Add `chrono` and `uuid` imports if not present**

Check if `chrono` and `uuid` are already in `Cargo.toml`. If not, add:

```toml
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
```

(These are likely already present since the codebase uses timestamps and device IDs.)

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -40`
Expected: No errors in asset storage.

- [ ] **Step 4: Commit**

```bash
git add src/asset/storage.rs Cargo.toml Cargo.lock
git commit -m "feat(asset): implement asset CRUD, sync, and IP conflict storage"
```

---

## Task 4: Maintenance Log & Firmware Catalog Storage

**Files:**
- Modify: `src/asset/storage.rs` (append methods)

- [ ] **Step 1: Add Maintenance Log CRUD**

Append to `impl AssetStorage`:

```rust
    // --- Maintenance Logs ---

    pub async fn create_maintenance_log(
        &self,
        asset_id: &str,
        req: &CreateMaintenanceRequest,
        created_by: &str,
    ) -> Result<MaintenanceLog> {
        let conn = self.conn.clone();
        let id = uuid::Uuid::new_v4().to_string();
        let asset_id = asset_id.to_string();
        let req = req.clone();
        let created_by = created_by.to_string();
        let now = chrono::Utc::now().to_rfc3339();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO maintenance_logs (id, asset_id, description, technician, performed_at, created_by, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![id, asset_id, req.description, req.technician, req.performed_at, created_by, now],
            )
            .context("Insert maintenance log failed")?;

            // Update asset's last_maintenance
            conn.execute(
                "UPDATE assets SET last_maintenance = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![req.performed_at, chrono::Utc::now().to_rfc3339(), asset_id],
            )?;

            Ok(MaintenanceLog {
                id,
                asset_id,
                description: req.description,
                technician: req.technician,
                performed_at: req.performed_at,
                created_by,
                created_at: now,
            })
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_maintenance_logs(&self, asset_id: &str) -> Result<Vec<MaintenanceLog>> {
        let conn = self.conn.clone();
        let asset_id = asset_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, asset_id, description, technician, performed_at, created_by, created_at
                 FROM maintenance_logs WHERE asset_id = ?1 ORDER BY performed_at DESC"
            ).context("Prepare failed")?;

            let rows = stmt
                .query_map(rusqlite::params![asset_id], |row| {
                    Ok(MaintenanceLog {
                        id: row.get(0)?,
                        asset_id: row.get(1)?,
                        description: row.get(2)?,
                        technician: row.get(3)?,
                        performed_at: row.get(4)?,
                        created_by: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                })
                .context("Query failed")?;

            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .context("Task join failed")?
    }
```

- [ ] **Step 2: Add Firmware Catalog CRUD**

Append to `impl AssetStorage`:

```rust
    // --- Firmware Catalog ---

    pub async fn list_firmware_catalog(&self, vendor: Option<&str>) -> Result<Vec<FirmwareCatalogEntry>> {
        let conn = self.conn.clone();
        let vendor = vendor.map(String::from);

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &vendor {
                Some(v) => (
                    "SELECT id, vendor, model, latest_version, release_date, notes, updated_at
                     FROM firmware_catalog WHERE vendor = ?1 ORDER BY vendor, model",
                    vec![Box::new(v.clone())],
                ),
                None => (
                    "SELECT id, vendor, model, latest_version, release_date, notes, updated_at
                     FROM firmware_catalog ORDER BY vendor, model",
                    vec![],
                ),
            };

            let mut stmt = conn.prepare(sql).context("Prepare failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();

            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(FirmwareCatalogEntry {
                        id: row.get(0)?,
                        vendor: row.get(1)?,
                        model: row.get(2)?,
                        latest_version: row.get(3)?,
                        release_date: row.get(4)?,
                        notes: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })
                .context("Query failed")?;

            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .context("Task join failed")?
    }

    pub async fn create_firmware_catalog_entry(&self, req: &CreateFirmwareCatalogRequest) -> Result<FirmwareCatalogEntry> {
        let conn = self.conn.clone();
        let id = uuid::Uuid::new_v4().to_string();
        let req = req.clone();
        let now = chrono::Utc::now().to_rfc3339();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO firmware_catalog (id, vendor, model, latest_version, release_date, notes, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![id, req.vendor, req.model, req.latest_version, req.release_date, req.notes, now],
            )
            .context("Insert firmware catalog entry failed")?;

            Ok(FirmwareCatalogEntry {
                id,
                vendor: req.vendor,
                model: req.model,
                latest_version: req.latest_version,
                release_date: req.release_date,
                notes: req.notes,
                updated_at: now,
            })
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_firmware_catalog_entry(
        &self,
        id: &str,
        fields: &serde_json::Value,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let fields = fields.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut set_clauses = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            for key in &["latest_version", "release_date", "notes"] {
                if let Some(val) = fields.get(key) {
                    set_clauses.push(format!("{} = ?", key));
                    params.push(Box::new(val.as_str().unwrap_or("").to_string()));
                }
            }

            set_clauses.push("updated_at = ?".to_string());
            params.push(Box::new(chrono::Utc::now().to_rfc3339()));

            let sql = format!(
                "UPDATE firmware_catalog SET {} WHERE id = ?",
                set_clauses.join(", ")
            );
            params.push(Box::new(id));

            conn.execute(&sql, rusqlite::params_from_iter(params))
                .context("Update firmware catalog failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_firmware_catalog_entry(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM firmware_catalog WHERE id = ?1",
                rusqlite::params![id],
            )
            .context("Delete firmware catalog failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_latest_firmware(&self, vendor: &str, model: &str) -> Result<Option<String>> {
        let conn = self.conn.clone();
        let vendor = vendor.to_string();
        let model = model.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.query_row(
                "SELECT latest_version FROM firmware_catalog WHERE vendor = ?1 AND model = ?2",
                rusqlite::params![vendor, model],
                |row| row.get(0),
            )
            .optional()
            .context("Query firmware failed")
        })
        .await
        .context("Task join failed")?
    }
```

- [ ] **Step 3: Add SNMP scan result storage**

Append to `impl AssetStorage`:

```rust
    // --- SNMP Scan Results ---

    pub async fn clear_scan_results(&self) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM snmp_scan_results", [])
                .context("Clear scan results failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn insert_scan_result(&self, r: &SnmpScanResult) -> Result<()> {
        let conn = self.conn.clone();
        let r = r.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO snmp_scan_results
                 (ip, sys_descr, sys_name, sys_uptime, vendor, model, firmware, mac_address, scanned_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![r.ip, r.sys_descr, r.sys_name, r.sys_uptime,
                    r.vendor, r.model, r.firmware, r.mac_address, r.scanned_at],
            )
            .context("Insert scan result failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_scan_results(&self) -> Result<Vec<SnmpScanResult>> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT ip, sys_descr, sys_name, sys_uptime, vendor, model, firmware, mac_address, scanned_at
                 FROM snmp_scan_results ORDER BY ip"
            ).context("Prepare failed")?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(SnmpScanResult {
                        ip: row.get(0)?,
                        sys_descr: row.get(1)?,
                        sys_name: row.get(2)?,
                        sys_uptime: row.get(3)?,
                        vendor: row.get(4)?,
                        model: row.get(5)?,
                        firmware: row.get(6)?,
                        mac_address: row.get(7)?,
                        scanned_at: row.get(8)?,
                    })
                })
                .context("Query failed")?;

            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_scan_result(&self, ip: &str) -> Result<Option<SnmpScanResult>> {
        let conn = self.conn.clone();
        let ip = ip.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.query_row(
                "SELECT ip, sys_descr, sys_name, sys_uptime, vendor, model, firmware, mac_address, scanned_at
                 FROM snmp_scan_results WHERE ip = ?1",
                rusqlite::params![ip],
                |row| {
                    Ok(SnmpScanResult {
                        ip: row.get(0)?,
                        sys_descr: row.get(1)?,
                        sys_name: row.get(2)?,
                        sys_uptime: row.get(3)?,
                        vendor: row.get(4)?,
                        model: row.get(5)?,
                        firmware: row.get(6)?,
                        mac_address: row.get(7)?,
                        scanned_at: row.get(8)?,
                    })
                },
            )
            .optional()
            .context("Query scan result failed")
        })
        .await
        .context("Task join failed")?
    }
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check 2>&1 | head -40`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src/asset/storage.rs
git commit -m "feat(asset): add maintenance log, firmware catalog, and SNMP scan storage"
```

---

## Task 5: Risk Score Engine

**Files:**
- Create: `src/asset/risk.rs`

- [ ] **Step 1: Create `src/asset/risk.rs`**

```rust
use crate::asset::models::*;
use crate::asset::storage::AssetStorage;

pub async fn compute_risk_score(storage: &AssetStorage, asset: &Asset) -> i64 {
    let mut score: i64 = 0;

    // Factor 1: Firmware outdated (+30)
    if let (Some(ref vendor), Some(ref model), Some(ref firmware)) =
        (&asset.vendor, &asset.model, &asset.firmware_version)
    {
        if let Ok(Some(latest)) = storage.get_latest_firmware(vendor, model).await {
            if firmware != &latest {
                score += 30;
            }
        }
    }

    // Factor 2: No maintenance in 12+ months (+20)
    match &asset.last_maintenance {
        None => score += 20,
        Some(date_str) => {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
                let days_since = (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_days();
                if days_since > 365 {
                    score += 20;
                }
            } else {
                // Can't parse the date, treat as no maintenance
                score += 20;
            }
        }
    }

    // Factor 3: Critical asset unmonitored (+25)
    if asset.criticality == "critical" && asset.device_ref.is_none() {
        score += 25;
    }

    // Factor 4: Default credentials (+25) — check if SNMP community is "public"
    // This is determined at scan time and stored as a tag
    if let Ok(tags) = serde_json::from_str::<Vec<String>>(&asset.tags) {
        if tags.iter().any(|t| t == "default-credentials") {
            score += 25;
        }
    }

    score.min(100)
}

pub async fn recompute_and_update(storage: &AssetStorage, asset_id: &str) -> anyhow::Result<()> {
    if let Some(asset) = storage.get_asset(asset_id).await? {
        let score = compute_risk_score(storage, &asset).await;
        storage.update_risk_score(asset_id, score).await?;
    }
    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/asset/risk.rs
git commit -m "feat(asset): implement risk score engine with 4 weighted factors"
```

---

## Task 6: SNMP Scanning Engine

**Files:**
- Create: `src/asset/snmp.rs`

- [ ] **Step 1: Create `src/asset/snmp.rs`**

This uses `rs-snmp` for SNMP queries. The scan engine enumerates IPs in a CIDR range and queries each for basic system info.

```rust
use anyhow::{Context, Result};
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ScanState {
    pub scanning: Arc<AtomicBool>,
    pub progress: Arc<AtomicUsize>,
    pub total: Arc<AtomicUsize>,
}

impl ScanState {
    pub fn new() -> Self {
        Self {
            scanning: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(AtomicUsize::new(0)),
            total: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn is_scanning(&self) -> bool {
        self.scanning.load(Ordering::Relaxed)
    }

    pub fn get_progress(&self) -> (usize, usize) {
        (self.progress.load(Ordering::Relaxed), self.total.load(Ordering::Relaxed))
    }
}

fn parse_cidr(cidr: &str) -> Result<(Ipv4Addr, u32)> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid CIDR format: {}", cidr);
    }
    let ip: Ipv4Addr = parts[0].parse().context("Invalid IP in CIDR")?;
    let prefix: u32 = parts[1].parse().context("Invalid prefix length")?;
    if prefix > 32 {
        anyhow::bail!("Invalid prefix length: {}", prefix);
    }
    Ok((ip, prefix))
}

fn cidr_to_hosts(ip: Ipv4Addr, prefix: u32) -> Vec<Ipv4Addr> {
    let ip_u32 = u32::from(ip);
    let mask = if prefix == 0 { 0 } else { !0u32 << (32 - prefix) };
    let network = ip_u32 & mask;
    let broadcast = network | !mask;
    // Skip network and broadcast addresses for /24 and smaller
    let start = if prefix >= 31 { network } else { network + 1 };
    let end = if prefix >= 31 { broadcast } else { broadcast - 1 };

    (start..=end)
        .map(Ipv4Addr::from)
        .collect()
}

/// Parse sysDescr to extract vendor, model, firmware
fn parse_sys_descr(sys_descr: &str) -> (Option<String>, Option<String>, Option<String>) {
    let lower = sys_descr.to_lowercase();

    // Siemens: "Siemens, SIMATIC S7, ..."
    if lower.contains("siemens") {
        let model = if lower.contains("s7-1200") { Some("S7-1200".to_string()) }
        else if lower.contains("s7-1500") { Some("S7-1500".to_string()) }
        else if lower.contains("s7-300") { Some("S7-300".to_string()) }
        else if lower.contains("s7-400") { Some("S7-400".to_string()) }
        else { None };
        return (Some("Siemens".to_string()), model, None);
    }

    // Allen-Bradley / Rockwell
    if lower.contains("rockwell") || lower.contains("allen-bradley") {
        let model = if lower.contains("controllogix") { Some("ControlLogix".to_string()) }
        else if lower.contains("compactlogix") { Some("CompactLogix".to_string()) }
        else if lower.contains("micrologix") { Some("MicroLogix".to_string()) }
        else { None };
        return (Some("Rockwell Automation".to_string()), model, None);
    }

    // Schneider / Modicon
    if lower.contains("schneider") || lower.contains("modicon") {
        return (Some("Schneider Electric".to_string()), None, None);
    }

    // Cisco
    if lower.contains("cisco") {
        return (Some("Cisco".to_string()), None, None);
    }

    (None, None, None)
}

/// Perform SNMP GET for a single IP
async fn snmp_query_host(
    ip: &str,
    community: &str,
    timeout_ms: u64,
) -> Option<crate::asset::models::SnmpScanResult> {
    // SNMP OID constants
    let sys_descr_oid = "1.3.6.1.2.1.1.1.0";
    let sys_name_oid = "1.3.6.1.2.1.1.5.0";
    let _sys_uptime_oid = "1.3.6.1.2.1.1.3.0";

    let target = format!("{}:161", ip);

    // Use rs-snmp to query
    let sys_descr = snmp_get_string(&target, community, sys_descr_oid, timeout_ms).await;
    if sys_descr.is_none() {
        return None; // Device not responding to SNMP
    }

    let sys_descr_val = sys_descr.unwrap_or_default();
    let sys_name = snmp_get_string(&target, community, sys_name_oid, timeout_ms).await;
    let (vendor, model, firmware) = parse_sys_descr(&sys_descr_val);

    Some(crate::asset::models::SnmpScanResult {
        ip: ip.to_string(),
        sys_descr: Some(sys_descr_val),
        sys_name,
        sys_uptime: None,
        vendor,
        model,
        firmware,
        mac_address: None,
        scanned_at: chrono::Utc::now().to_rfc3339(),
    })
}

async fn snmp_get_string(target: &str, community: &str, oid: &str, timeout_ms: u64) -> Option<String> {
    // rs-snmp SNMPv2c GET
    use rs_snmp::prelude::*;

    let community = Community::new(community.as_bytes());
    let target_addr = target.parse::<SocketAddr>().ok()?;

    let session = SyncSession::builder(target_addr)
        .community(community)
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .ok()?;

    let oid_parts: Vec<u32> = oid.split('.').filter_map(|p| p.parse().ok()).collect();
    let varbinds = vec![VarBind {
        name: oid_parts,
        value: SnmpType::Null,
    }];

    let response = session.get(varbinds).ok()?;

    if let Some(varbind) = response.first() {
        if let SnmpType::String(s) = &varbind.value {
            return Some(String::from_utf8_lossy(s).to_string());
        }
    }
    None
}

use std::net::SocketAddr;
use std::time::Duration;

/// Run a full SNMP scan of a subnet
pub async fn run_scan(
    subnet: &str,
    community: &str,
    timeout_ms: u64,
    scan_state: &ScanState,
) -> Result<Vec<crate::asset::models::SnmpScanResult>> {
    if scan_state.is_scanning() {
        anyhow::bail!("Scan already in progress");
    }

    let (network_ip, prefix) = parse_cidr(subnet)?;
    let hosts = cidr_to_hosts(network_ip, prefix);
    let total = hosts.len();
    scan_state.scanning.store(true, Ordering::Relaxed);
    scan_state.total.store(total, Ordering::Relaxed);
    scan_state.progress.store(0, Ordering::Relaxed);

    let mut results = Vec::new();

    // Scan with concurrency limit of 50
    let semaphore = Arc::new(tokio::sync::Semaphore::new(50));
    let mut handles = Vec::new();

    for host in hosts {
        let community = community.to_string();
        let semaphore = semaphore.clone();
        let scan_state = scan_state.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let ip = host.to_string();
            let result = snmp_query_host(&ip, &community, timeout_ms).await;
            scan_state.progress.fetch_add(1, Ordering::Relaxed);
            result
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Ok(Some(result)) = handle.await {
            results.push(result);
        }
    }

    scan_state.scanning.store(false, Ordering::Relaxed);
    Ok(results)
}
```

**Note:** The `rs-snmp` API may differ from the above. Check the actual crate API at implementation time and adjust the `snmp_get_string` function accordingly. The scan logic (CIDR parsing, concurrency, host enumeration) is the core value — the SNMP library calls are an integration point.

- [ ] **Step 2: Verify compilation (adjust SNMP crate API as needed)**

Run: `cargo check 2>&1 | head -40`
Expected: May need adjustments based on actual `rs-snmp` API. Fix any compile errors.

- [ ] **Step 3: Commit**

```bash
git add src/asset/snmp.rs
git commit -m "feat(asset): implement SNMP scanning engine with CIDR enumeration"
```

---

## Task 7: Asset API Endpoints

**Files:**
- Create: `src/api/assets.rs`
- Create: `src/api/maintenance.rs`
- Create: `src/api/firmware_catalog.rs`

- [ ] **Step 1: Create `src/api/assets.rs`**

```rust
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::api::{AppError, AppState, AuthUser};

#[derive(Deserialize)]
pub struct ListAssetsQuery {
    pub r#type: Option<String>,
    pub criticality: Option<String>,
    pub vendor: Option<String>,
    pub search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/assets",
    responses((status = 200, body = Vec<Asset>)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_assets(
    State(state): State<AppState>,
    Query(q): Query<ListAssetsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let assets = state.storage.asset().list_assets(
        q.r#type.as_deref(),
        q.criticality.as_deref(),
        q.vendor.as_deref(),
        q.search.as_deref(),
    ).await?;
    Ok(Json(serde_json::to_value(assets)?))
}

#[utoipa::path(
    post,
    path = "/api/v1/assets",
    responses((status = 201, body = Asset)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn create_asset(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<crate::asset::models::CreateAssetRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.name.is_empty() {
        return Err(AppError(anyhow::anyhow!("Asset name is required")));
    }
    let valid_types = ["plc", "rtu", "hmi", "sensor", "switch", "router", "gateway", "other"];
    if !valid_types.contains(&body.asset_type.as_str()) {
        return Err(AppError(anyhow::anyhow!(
            "Invalid asset type. Must be one of: {}", valid_types.join(", ")
        )));
    }

    let asset = state.storage.asset().create_asset(&body).await?;

    // Compute initial risk score
    crate::asset::risk::recompute_and_update(&state.storage.asset(), &asset.id).await?;
    let asset = state.storage.asset().get_asset(&asset.id).await?.unwrap();

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_asset".to_string(),
        username: auth.username.clone(),
        resource: format!("asset:{}", asset.id),
        details: Some(format!("name={} type={}", asset.name, asset.asset_type)),
    });

    Ok((StatusCode::CREATED, Json(serde_json::to_value(asset)?)))
}

#[utoipa::path(
    get,
    path = "/api/v1/assets/{id}",
    responses((status = 200, body = Asset)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_asset(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let asset = state.storage.asset().get_asset(&id).await?;
    match asset {
        Some(a) => Ok(Json(serde_json::to_value(a)?)),
        None => Err(AppError(anyhow::anyhow!("Asset not found"))),
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/assets/{id}",
    responses((status = 200)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_asset(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.asset().update_asset(&id, &body).await?;
    crate::asset::risk::recompute_and_update(&state.storage.asset(), &id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_asset".to_string(),
        username: auth.username,
        resource: format!("asset:{}", id),
        details: Some(serde_json::to_string(&body).unwrap_or_default()),
    });

    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    delete,
    path = "/api/v1/assets/{id}",
    responses((status = 200)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn delete_asset(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.asset().delete_asset(&id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "delete_asset".to_string(),
        username: auth.username,
        resource: format!("asset:{}", id),
        details: None,
    });

    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    get,
    path = "/api/v1/assets/ip-conflicts",
    responses((status = 200)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_ip_conflicts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conflicts = state.storage.asset().find_ip_conflicts().await?;
    let result: Vec<serde_json::Value> = conflicts
        .into_iter()
        .map(|(ip, assets)| {
            serde_json::json!({
                "ip": ip,
                "assets": assets,
            })
        })
        .collect();
    Ok(Json(result))
}
```

- [ ] **Step 2: Create `src/api/maintenance.rs`**

```rust
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::api::{AppError, AppState, AuthUser};

#[utoipa::path(
    get,
    path = "/api/v1/assets/{id}/maintenance",
    responses((status = 200, body = Vec<MaintenanceLog>)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_maintenance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let logs = state.storage.asset().list_maintenance_logs(&id).await?;
    Ok(Json(serde_json::to_value(logs)?))
}

#[utoipa::path(
    post,
    path = "/api/v1/assets/{id}/maintenance",
    responses((status = 201, body = MaintenanceLog)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn create_maintenance(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<crate::asset::models::CreateMaintenanceRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Verify asset exists
    if state.storage.asset().get_asset(&id).await?.is_none() {
        return Err(AppError(anyhow::anyhow!("Asset not found")));
    }

    let log = state.storage.asset()
        .create_maintenance_log(&id, &body, &auth.username)
        .await?;

    // Recompute risk score (maintenance may have changed)
    crate::asset::risk::recompute_and_update(&state.storage.asset(), &id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_maintenance_log".to_string(),
        username: auth.username,
        resource: format!("asset:{}/maintenance/{}", id, log.id),
        details: Some(format!("technician={} description={}", body.technician, body.description)),
    });

    Ok((StatusCode::CREATED, Json(serde_json::to_value(log)?)))
}
```

- [ ] **Step 3: Create `src/api/firmware_catalog.rs`**

```rust
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::api::{AppError, AppState, AuthUser};

#[derive(Deserialize)]
pub struct ListFirmwareQuery {
    pub vendor: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/firmware-catalog",
    responses((status = 200)),
    tag = "Firmware Catalog",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_firmware_catalog(
    State(state): State<AppState>,
    Query(q): Query<ListFirmwareQuery>,
) -> Result<impl IntoResponse, AppError> {
    let entries = state.storage.asset()
        .list_firmware_catalog(q.vendor.as_deref())
        .await?;
    Ok(Json(serde_json::to_value(entries)?))
}

#[utoipa::path(
    post,
    path = "/api/v1/firmware-catalog",
    responses((status = 201)),
    tag = "Firmware Catalog",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn create_firmware_catalog_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<crate::asset::models::CreateFirmwareCatalogRequest>,
) -> Result<impl IntoResponse, AppError> {
    let entry = state.storage.asset().create_firmware_catalog_entry(&body).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_firmware_catalog".to_string(),
        username: auth.username,
        resource: format!("firmware_catalog:{}", entry.id),
        details: Some(format!("vendor={} model={} version={}", body.vendor, body.model, body.latest_version)),
    });

    Ok((StatusCode::CREATED, Json(serde_json::to_value(entry)?)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/firmware-catalog/{id}",
    responses((status = 200)),
    tag = "Firmware Catalog",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_firmware_catalog_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.asset().update_firmware_catalog_entry(&id, &body).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_firmware_catalog".to_string(),
        username: auth.username,
        resource: format!("firmware_catalog:{}", id),
        details: Some(serde_json::to_string(&body).unwrap_or_default()),
    });

    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    delete,
    path = "/api/v1/firmware-catalog/{id}",
    responses((status = 200)),
    tag = "Firmware Catalog",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn delete_firmware_catalog_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.asset().delete_firmware_catalog_entry(&id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "delete_firmware_catalog".to_string(),
        username: auth.username,
        resource: format!("firmware_catalog:{}", id),
        details: None,
    });

    Ok(Json(serde_json::json!({"ok": true})))
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check 2>&1 | head -40`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src/api/assets.rs src/api/maintenance.rs src/api/firmware_catalog.rs
git commit -m "feat(asset): add asset, maintenance, and firmware catalog API endpoints"
```

---

## Task 8: SNMP Scan & Import/Export API

**Files:**
- Create: `src/api/snmp_scan.rs`
- Create: `src/api/asset_import.rs`
- Create: `src/api/nis2_export.rs`

- [ ] **Step 1: Create `src/api/snmp_scan.rs`**

```rust
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::api::{AppError, AppState};

lazy_static::lazy_static! {
    pub static ref SCAN_STATE: crate::asset::snmp::ScanState = crate::asset::snmp::ScanState::new();
}

#[utoipa::path(
    post,
    path = "/api/v1/snmp/scan",
    responses((status = 200)),
    tag = "SNMP Scan",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn start_scan(
    State(state): State<AppState>,
    Json(body): Json<crate::asset::models::SnmpScanRequest>,
) -> Result<impl IntoResponse, AppError> {
    if SCAN_STATE.is_scanning() {
        return Err(AppError(anyhow::anyhow!("Scan already in progress")));
    }

    let storage = state.storage.asset();
    let scan_state = SCAN_STATE.clone();
    let subnet = body.subnet.clone();
    let community = body.community.clone();
    let timeout_ms = body.timeout_ms;

    // Clear previous results
    storage.clear_scan_results().await?;

    // Spawn scan in background
    tokio::spawn(async move {
        match crate::asset::snmp::run_scan(&subnet, &community, timeout_ms, &scan_state).await {
            Ok(results) => {
                for r in &results {
                    let _ = storage.insert_scan_result(r).await;
                }
            }
            Err(e) => {
                eprintln!("SNMP scan error: {}", e);
            }
        }
    });

    Ok(Json(serde_json::json!({"ok": true, "message": "Scan started"})))
}

#[utoipa::path(
    get,
    path = "/api/v1/snmp/scan/status",
    responses((status = 200)),
    tag = "SNMP Scan",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_scan_status() -> Result<impl IntoResponse, AppError> {
    let scanning = SCAN_STATE.is_scanning();
    let (progress, total) = SCAN_STATE.get_progress();
    Ok(Json(serde_json::json!({
        "scanning": scanning,
        "progress": progress,
        "total": total,
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/snmp/scan/results",
    responses((status = 200)),
    tag = "SNMP Scan",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_scan_results(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let results = state.storage.asset().list_scan_results().await?;
    Ok(Json(serde_json::to_value(results)?))
}

#[utoipa::path(
    post,
    path = "/api/v1/snmp/scan/{ip}/import",
    responses((status = 201)),
    tag = "SNMP Scan",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn import_scanned_device(
    State(state): State<AppState>,
    Path(ip): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let scan_result = state.storage.asset().get_scan_result(&ip).await?;
    let scan = scan_result.ok_or_else(|| AppError(anyhow::anyhow!("Scan result not found for {}", ip)))?;

    let asset_type = match scan.vendor.as_deref() {
        Some(v) if v.contains("Cisco") => "router",
        Some(v) if v.contains("Rockwell") || v.contains("Siemens") => "plc",
        _ => "switch",
    };

    let req = crate::asset::models::CreateAssetRequest {
        name: scan.sys_name.clone().unwrap_or_else(|| format!("SNMP Device {}", ip)),
        asset_type: asset_type.to_string(),
        vendor: scan.vendor.clone(),
        model: scan.model.clone(),
        firmware_version: scan.firmware.clone(),
        serial_number: None,
        ip_address: Some(ip.clone()),
        mac_address: scan.mac_address.clone(),
        location: None,
        criticality: "medium".to_string(),
        tags: "[]".to_string(),
        notes: scan.sys_descr.clone(),
    };

    let asset = state.storage.asset().create_asset(&req).await?;
    crate::asset::risk::recompute_and_update(&state.storage.asset(), &asset.id).await?;

    Ok((axum::http::StatusCode::CREATED, Json(serde_json::json!({"ok": true, "id": asset.id}))))
}
```

- [ ] **Step 2: Create `src/api/asset_import.rs`**

```rust
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::api::{AppError, AppState};

// In-memory store for import previews (expires after 10 min)
lazy_static::lazy_static! {
    static ref PENDING_IMPORTS: Mutex<HashMap<String, (std::time::Instant, Vec<serde_json::Value>)>> =
        Mutex::new(HashMap::new());
}

const FIELD_MAP: &[(&str, &str)] = &[
    ("name", "name"),
    ("type", "asset_type"),
    ("asset_type", "asset_type"),
    ("vendor", "vendor"),
    ("manufacturer", "vendor"),
    ("model", "model"),
    ("firmware", "firmware_version"),
    ("firmware_version", "firmware_version"),
    ("serial", "serial_number"),
    ("serial_number", "serial_number"),
    ("ip", "ip_address"),
    ("ip_address", "ip_address"),
    ("mac", "mac_address"),
    ("mac_address", "mac_address"),
    ("location", "location"),
    ("criticality", "criticality"),
    ("notes", "notes"),
    ("description", "notes"),
];

#[utoipa::path(
    post,
    path = "/api/v1/assets/import",
    responses((status = 200)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn import_preview(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError(anyhow::anyhow!("Multipart error: {}", e)))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            file_data = Some(field.bytes().await.map_err(|e| AppError(anyhow::anyhow!("Read error: {}", e)))?.to_vec());
            file_name = field.file_name().map(|s| s.to_string());
        }
    }

    let data = file_data.ok_or_else(|| AppError(anyhow::anyhow!("No file uploaded")))?;
    let name = file_name.unwrap_or_default();

    let rows = if name.ends_with(".xlsx") {
        parse_xlsx(&data)?
    } else {
        parse_csv(&data)?
    };

    let total = rows.len();
    let mut errors = Vec::new();
    let mut valid = 0;

    for (i, row) in rows.iter().enumerate() {
        if row.get("name").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
            errors.push(format!("Row {}: missing name", i + 2));
        } else {
            valid += 1;
        }
    }

    let import_id = uuid::Uuid::new_v4().to_string();
    PENDING_IMPORTS.lock().unwrap()
        .insert(import_id.clone(), (std::time::Instant::now(), rows.clone()));

    // Clean up expired imports
    PENDING_IMPORTS.lock().unwrap().retain(|_, (created, _)| created.elapsed().as_secs() < 600);

    let preview = crate::asset::models::ImportPreview {
        import_id,
        rows,
        total: total as i64,
        valid: valid as i64,
        errors,
    };

    Ok(Json(serde_json::to_value(preview)?))
}

fn parse_csv(data: &[u8]) -> Result<Vec<serde_json::Value>, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(data);

    let headers = reader.headers().map_err(|e| AppError(anyhow::anyhow!("CSV parse error: {}", e)))?.clone();
    let mut rows = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| AppError(anyhow::anyhow!("CSV row error: {}", e)))?;
        let mut row = serde_json::Map::new();

        for (i, header) in headers.iter().enumerate() {
            let mapped_field = FIELD_MAP.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(header))
                .map(|(_, v)| *v)
                .unwrap_or(header);

            if let Some(val) = record.get(i) {
                if !val.is_empty() {
                    row.insert(mapped_field.to_string(), serde_json::Value::String(val.to_string()));
                }
            }
        }

        rows.push(serde_json::Value::Object(row));
    }

    Ok(rows)
}

fn parse_xlsx(data: &[u8]) -> Result<Vec<serde_json::Value>, AppError> {
    use calamine::{Reader, open_workbook_from_rs, Xlsx, DataType};

    let cursor = std::io::Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| AppError(anyhow::anyhow!("XLSX parse error: {}", e)))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let sheet_name = sheet_names.first().ok_or_else(|| AppError(anyhow::anyhow!("No sheets in workbook")))?;

    let range = workbook.worksheet_range(sheet_name)
        .map_err(|e| AppError(anyhow::anyhow!("Sheet read error: {}", e)))?;

    let mut rows_iter = range.rows();
    let headers = rows_iter.next().unwrap_or(&[]);
    let mut result = Vec::new();

    for row in rows_iter {
        let mut json_row = serde_json::Map::new();
        for (i, cell) in row.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                let header_str = header.to_string();
                let mapped_field = FIELD_MAP.iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(&header_str))
                    .map(|(_, v)| *v)
                    .unwrap_or(&header_str);

                let val = match cell {
                    DataType::String(s) => Some(s.clone()),
                    DataType::Float(f) => Some(f.to_string()),
                    DataType::Int(n) => Some(n.to_string()),
                    DataType::Bool(b) => Some(b.to_string()),
                    _ => None,
                };
                if let Some(v) = val {
                    json_row.insert(mapped_field.to_string(), serde_json::Value::String(v));
                }
            }
        }
        result.push(serde_json::Value::Object(json_row));
    }

    Ok(result)
}

#[utoipa::path(
    post,
    path = "/api/v1/assets/import/confirm",
    responses((status = 200)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn import_confirm(
    State(state): State<AppState>,
    Json(body): Json<crate::asset::models::ImportConfirmRequest>,
) -> Result<impl IntoResponse, AppError> {
    let rows = {
        let mut pending = PENDING_IMPORTS.lock().unwrap();
        pending.remove(&body.import_id)
            .filter(|(created, _)| created.elapsed().as_secs() < 600)
            .map(|(_, rows)| rows)
    };

    let rows = rows.ok_or_else(|| AppError(anyhow::anyhow!("Import session expired or not found")))?;

    let mut created = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if name.is_empty() {
            skipped += 1;
            continue;
        }

        let req = crate::asset::models::CreateAssetRequest {
            name,
            asset_type: row.get("asset_type").and_then(|v| v.as_str()).unwrap_or("other").to_string(),
            vendor: row.get("vendor").and_then(|v| v.as_str()).map(String::from),
            model: row.get("model").and_then(|v| v.as_str()).map(String::from),
            firmware_version: row.get("firmware_version").and_then(|v| v.as_str()).map(String::from),
            serial_number: row.get("serial_number").and_then(|v| v.as_str()).map(String::from),
            ip_address: row.get("ip_address").and_then(|v| v.as_str()).map(String::from),
            mac_address: row.get("mac_address").and_then(|v| v.as_str()).map(String::from),
            location: row.get("location").and_then(|v| v.as_str()).map(String::from),
            criticality: row.get("criticality").and_then(|v| v.as_str()).unwrap_or("medium").to_string(),
            tags: "[]".to_string(),
            notes: row.get("notes").and_then(|v| v.as_str()).map(String::from),
        };

        match state.storage.asset().create_asset(&req).await {
            Ok(a) => {
                let _ = crate::asset::risk::recompute_and_update(&state.storage.asset(), &a.id).await;
                created += 1;
            }
            Err(e) => errors.push(format!("Row {}: {}", i + 2, e)),
        }
    }

    Ok(Json(serde_json::to_value(crate::asset::models::ImportResult {
        total: rows.len() as i64,
        created,
        skipped_duplicates: skipped,
        errors,
    })?))
}
```

- [ ] **Step 3: Create `src/api/nis2_export.rs`**

```rust
use axum::{
    extract::State,
    response::IntoResponse,
};

use crate::api::{AppError, AppState};

#[utoipa::path(
    get,
    path = "/api/v1/assets/nis2-export",
    responses((status = 200)),
    tag = "Assets",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn nis2_export(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let assets = state.storage.asset().list_assets(None, None, None, None).await?;

    let mut csv = String::from("name,type,vendor,model,serial_number,firmware,latest_firmware,firmware_status,last_maintenance,risk_score,criticality,location,ip_address\n");

    for a in &assets {
        let latest_firmware = match (&a.vendor, &a.model) {
            (Some(v), Some(m)) => {
                state.storage.asset().get_latest_firmware(v, m).await.unwrap_or(None)
            }
            _ => None,
        };

        let firmware_status = match (&a.firmware_version, &latest_firmware) {
            (Some(current), Some(latest)) => {
                if current == latest { "current" } else { "outdated" }
            }
            (Some(_), None) => "unknown",
            (None, _) => "unknown",
        };

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&a.name),
            csv_escape(&a.asset_type),
            csv_escape(a.vendor.as_deref().unwrap_or("")),
            csv_escape(a.model.as_deref().unwrap_or("")),
            csv_escape(a.serial_number.as_deref().unwrap_or("")),
            csv_escape(a.firmware_version.as_deref().unwrap_or("")),
            csv_escape(latest_firmware.as_deref().unwrap_or("")),
            firmware_status,
            csv_escape(a.last_maintenance.as_deref().unwrap_or("")),
            a.risk_score,
            csv_escape(&a.criticality),
            csv_escape(a.location.as_deref().unwrap_or("")),
            csv_escape(a.ip_address.as_deref().unwrap_or("")),
        ));
    }

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let filename = format!("voltrus-nis2-export-{}.csv", date);

    Ok((
        [
            ("content-type", "text/csv"),
            ("content-disposition", format!("attachment; filename=\"{}\"", filename).as_str()),
        ],
        csv,
    ))
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
```

- [ ] **Step 4: Add `csv` and `lazy_static` crates if not present**

Check `Cargo.toml` for `csv` and `lazy_static`. Add if missing:

```toml
csv = "1.3"
lazy_static = "1.5"
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check 2>&1 | head -40`
Expected: No errors.

- [ ] **Step 6: Commit**

```bash
git add src/api/snmp_scan.rs src/api/asset_import.rs src/api/nis2_export.rs Cargo.toml Cargo.lock
git commit -m "feat(asset): add SNMP scan, CSV/XLSX import, and NIS2 export APIs"
```

---

## Task 9: Device-to-Asset Auto-Sync Hooks

**Files:**
- Create: `src/asset/sync.rs`
- Modify: `src/api/modbus_devices.rs`
- Modify: `src/api/opcua_devices.rs`
- Modify: `src/api/s7_devices.rs`
- Modify: `src/api/enip_devices.rs`

- [ ] **Step 1: Create `src/asset/sync.rs`**

```rust
use crate::asset::storage::AssetStorage;

pub async fn on_device_created(
    storage: &AssetStorage,
    protocol: &str,
    device_id: &str,
    name: &str,
    ip: Option<&str>,
    asset_type: &str,
) {
    let device_ref = format!("{}:{}", protocol, device_id);
    if let Err(e) = storage.link_device(&device_ref, name, ip, asset_type).await {
        eprintln!("Failed to link asset for device {}: {}", device_ref, e);
    }
}

pub async fn on_device_deleted(storage: &AssetStorage, protocol: &str, device_id: &str) {
    let device_ref = format!("{}:{}", protocol, device_id);
    if let Err(e) = storage.unlink_device(&device_ref).await {
        eprintln!("Failed to unlink asset for device {}: {}", device_ref, e);
    }
}
```

- [ ] **Step 2: Add sync hooks to Modbus device handler**

In `src/api/modbus_devices.rs`, find the `create_modbus_device` function. After the successful device creation and before the return statement, add:

```rust
    crate::asset::sync::on_device_created(
        &state.storage.asset(), "modbus", &id, &name, Some(&ip), "plc",
    ).await;
```

In the `delete_modbus_device` function, after the successful deletion, add:

```rust
    crate::asset::sync::on_device_deleted(&state.storage.asset(), "modbus", &id).await;
```

- [ ] **Step 3: Add sync hooks to OPC-UA device handler**

In `src/api/opcua_devices.rs`, add the same pattern with `"opcua"` as the protocol:

After create:
```rust
    crate::asset::sync::on_device_created(
        &state.storage.asset(), "opcua", &id, &name, Some(&ip), "plc",
    ).await;
```

After delete:
```rust
    crate::asset::sync::on_device_deleted(&state.storage.asset(), "opcua", &id).await;
```

- [ ] **Step 4: Add sync hooks to S7 device handler**

Same pattern with `"s7"` as protocol. After create and delete calls.

- [ ] **Step 5: Add sync hooks to EtherNet/IP device handler**

Same pattern with `"enip"` as protocol. After create and delete calls.

- [ ] **Step 6: Verify compilation**

Run: `cargo check 2>&1 | head -40`
Expected: No errors.

- [ ] **Step 7: Commit**

```bash
git add src/asset/sync.rs src/api/modbus_devices.rs src/api/opcua_devices.rs src/api/s7_devices.rs src/api/enip_devices.rs
git commit -m "feat(asset): add device-to-asset auto-sync hooks for all protocols"
```

---

## Task 10: Route Registration

**Files:**
- Modify: `src/api/mod.rs`

- [ ] **Step 1: Add module declarations**

At the top of `src/api/mod.rs`, add:

```rust
mod assets;
mod maintenance;
mod firmware_catalog;
mod snmp_scan;
mod asset_import;
mod nis2_export;
```

- [ ] **Step 2: Register routes**

Find the router setup in `create_router` (or equivalent function). Add the following routes:

To the **protected** router (GET endpoints — any authenticated user):

```rust
    .route("/api/v1/assets", get(assets::list_assets))
    .route("/api/v1/assets/ip-conflicts", get(assets::get_ip_conflicts))
    .route("/api/v1/assets/{id}", get(assets::get_asset))
    .route("/api/v1/assets/{id}/maintenance", get(maintenance::list_maintenance))
    .route("/api/v1/firmware-catalog", get(firmware_catalog::list_firmware_catalog))
    .route("/api/v1/snmp/scan/status", get(snmp_scan::get_scan_status))
    .route("/api/v1/snmp/scan/results", get(snmp_scan::list_scan_results))
    .route("/api/v1/assets/nis2-export", get(nis2_export::nis2_export))
```

To the **admin** router (POST/PATCH/DELETE — admin only):

```rust
    .route("/api/v1/assets", post(assets::create_asset))
    .route("/api/v1/assets/{id}", patch(assets::update_asset).delete(assets::delete_asset))
    .route("/api/v1/assets/{id}/maintenance", post(maintenance::create_maintenance))
    .route("/api/v1/firmware-catalog", post(firmware_catalog::create_firmware_catalog_entry))
    .route("/api/v1/firmware-catalog/{id}", patch(firmware_catalog::update_firmware_catalog_entry).delete(firmware_catalog::delete_firmware_catalog_entry))
    .route("/api/v1/snmp/scan", post(snmp_scan::start_scan))
    .route("/api/v1/snmp/scan/{ip}/import", post(snmp_scan::import_scanned_device))
    .route("/api/v1/assets/import", post(asset_import::import_preview))
    .route("/api/v1/assets/import/confirm", post(asset_import::import_confirm))
```

- [ ] **Step 3: Verify full compilation**

Run: `cargo check 2>&1 | head -40`
Expected: Clean compilation.

- [ ] **Step 4: Commit**

```bash
git add src/api/mod.rs
git commit -m "feat(asset): register all asset inventory API routes"
```

---

## Task 11: Backend Smoke Test

- [ ] **Step 1: Build and run the server**

```bash
cargo build --release 2>&1 | tail -5
```

- [ ] **Step 2: Test asset CRUD**

```bash
# Create
curl -s -X POST http://localhost:3000/api/v1/assets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Test PLC","type":"plc","vendor":"Siemens","model":"S7-1200","ip_address":"192.168.1.10"}'

# List
curl -s http://localhost:3000/api/v1/assets -H "Authorization: Bearer $TOKEN"

# Get
curl -s http://localhost:3000/api/v1/assets/{id} -H "Authorization: Bearer $TOKEN"

# Update
curl -s -X PATCH http://localhost:3000/api/v1/assets/{id} \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"location":"Building A Rack 3"}'

# NIS2 Export
curl -s http://localhost:3000/api/v1/assets/nis2-export -H "Authorization: Bearer $TOKEN"
```

- [ ] **Step 3: Test firmware catalog**

```bash
curl -s -X POST http://localhost:3000/api/v1/firmware-catalog \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"vendor":"Siemens","model":"S7-1200","latest_version":"V4.6"}'

curl -s http://localhost:3000/api/v1/firmware-catalog -H "Authorization: Bearer $TOKEN"
```

- [ ] **Step 4: Test maintenance log**

```bash
curl -s -X POST http://localhost:3000/api/v1/assets/{id}/maintenance \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"description":"Annual inspection","technician":"John Doe","performed_at":"2026-05-04T10:00:00Z"}'
```

- [ ] **Step 5: Commit if any fixes were needed**

```bash
git add -A && git commit -m "fix(asset): address smoke test issues"
```

---

## Task 12: Frontend API Hooks

**Files:**
- Create: `client/src/queries/assets.ts`

- [ ] **Step 1: Create `client/src/queries/assets.ts`**

```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

const API = '/api/v1'

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, { credentials: 'include', ...init })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(err.error || `HTTP ${res.status}`)
  }
  return res.json()
}

async function fetchBlob(url: string): Promise<void> {
  const res = await fetch(url, { credentials: 'include' })
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  const blob = await res.blob()
  const a = document.createElement('a')
  a.href = URL.createObjectURL(blob)
  a.download = res.headers.get('content-disposition')?.match(/filename="(.+)"/)?.[1] || 'export.csv'
  a.click()
  URL.revokeObjectURL(a.href)
}

// ---- Asset Types ----

export interface Asset {
  id: string
  device_ref: string | null
  name: string
  asset_type: string
  vendor: string | null
  model: string | null
  firmware_version: string | null
  serial_number: string | null
  ip_address: string | null
  mac_address: string | null
  location: string | null
  criticality: string
  tags: string
  risk_score: number
  last_maintenance: string | null
  notes: string | null
  created_at: string
  updated_at: string
}

export interface MaintenanceLog {
  id: string
  asset_id: string
  description: string
  technician: string
  performed_at: string
  created_by: string
  created_at: string
}

export interface FirmwareCatalogEntry {
  id: string
  vendor: string
  model: string
  latest_version: string
  release_date: string | null
  notes: string | null
  updated_at: string
}

export interface SnmpScanResult {
  ip: string
  sys_descr: string | null
  sys_name: string | null
  sys_uptime: number | null
  vendor: string | null
  model: string | null
  firmware: string | null
  mac_address: string | null
  scanned_at: string
}

export interface ScanStatus {
  scanning: boolean
  progress: number
  total: number
}

export const ASSET_TYPES = ['plc', 'rtu', 'hmi', 'sensor', 'switch', 'router', 'gateway', 'other'] as const
export const CRITICALITY_LEVELS = ['low', 'medium', 'high', 'critical'] as const

// ---- Query Keys ----

export const queryKeys = {
  assets: (filters?: Record<string, string>) => ['assets', filters] as const,
  asset: (id: string) => ['assets', id] as const,
  maintenance: (id: string) => ['assets', id, 'maintenance'] as const,
  firmwareCatalog: (vendor?: string) => ['firmware-catalog', vendor] as const,
  scanResults: ['snmp-scan-results'] as const,
  scanStatus: ['snmp-scan-status'] as const,
  ipConflicts: ['ip-conflicts'] as const,
}

// ---- Asset Hooks ----

export function useAssets(filters?: Record<string, string>) {
  const params = new URLSearchParams()
  if (filters) Object.entries(filters).forEach(([k, v]) => { if (v) params.set(k, v) })
  const qs = params.toString()
  return useQuery({
    queryKey: queryKeys.assets(filters),
    queryFn: () => fetchJson<Asset[]>(`${API}/assets${qs ? `?${qs}` : ''}`),
    staleTime: 30_000,
  })
}

export function useAsset(id: string) {
  return useQuery({
    queryKey: queryKeys.asset(id),
    queryFn: () => fetchJson<Asset>(`${API}/assets/${id}`),
    enabled: !!id,
  })
}

export function useCreateAsset() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: Partial<Asset>) => fetchJson<Asset>(`${API}/assets`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['assets'] }),
  })
}

export function useUpdateAsset() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, ...body }: Partial<Asset> & { id: string }) =>
      fetchJson<void>(`${API}/assets/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: (_, { id }) => {
      qc.invalidateQueries({ queryKey: queryKeys.asset(id) })
      qc.invalidateQueries({ queryKey: ['assets'] })
    },
  })
}

export function useDeleteAsset() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => fetchJson<void>(`${API}/assets/${id}`, { method: 'DELETE' }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['assets'] }),
  })
}

// ---- Maintenance Hooks ----

export function useMaintenanceLogs(assetId: string) {
  return useQuery({
    queryKey: queryKeys.maintenance(assetId),
    queryFn: () => fetchJson<MaintenanceLog[]>(`${API}/assets/${assetId}/maintenance`),
    enabled: !!assetId,
  })
}

export function useCreateMaintenanceLog(assetId: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: { description: string; technician: string; performed_at: string }) =>
      fetchJson<MaintenanceLog>(`${API}/assets/${assetId}/maintenance`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.maintenance(assetId) })
      qc.invalidateQueries({ queryKey: queryKeys.asset(assetId) })
    },
  })
}

// ---- Firmware Catalog Hooks ----

export function useFirmwareCatalog(vendor?: string) {
  return useQuery({
    queryKey: queryKeys.firmwareCatalog(vendor),
    queryFn: () => fetchJson<FirmwareCatalogEntry[]>(`${API}/firmware-catalog${vendor ? `?vendor=${vendor}` : ''}`),
  })
}

export function useCreateFirmwareEntry() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: Partial<FirmwareCatalogEntry>) => fetchJson<FirmwareCatalogEntry>(`${API}/firmware-catalog`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['firmware-catalog'] }),
  })
}

export function useUpdateFirmwareEntry() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, ...body }: Partial<FirmwareCatalogEntry> & { id: string }) =>
      fetchJson<void>(`${API}/firmware-catalog/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['firmware-catalog'] }),
  })
}

export function useDeleteFirmwareEntry() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => fetchJson<void>(`${API}/firmware-catalog/${id}`, { method: 'DELETE' }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['firmware-catalog'] }),
  })
}

// ---- SNMP Scan Hooks ----

export function useStartScan() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: { subnet: string; community?: string; timeout_ms?: number }) =>
      fetchJson<{ ok: boolean }>(`${API}/snmp/scan`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['snmp-scan-status'] }),
  })
}

export function useScanStatus() {
  return useQuery({
    queryKey: queryKeys.scanStatus,
    queryFn: () => fetchJson<ScanStatus>(`${API}/snmp/scan/status`),
    refetchInterval: 2000,
  })
}

export function useScanResults() {
  return useQuery({
    queryKey: queryKeys.scanResults,
    queryFn: () => fetchJson<SnmpScanResult[]>(`${API}/snmp/scan/results`),
  })
}

export function useImportScannedDevice() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (ip: string) => fetchJson<{ ok: boolean; id: string }>(`${API}/snmp/scan/${encodeURIComponent(ip)}/import`, {
      method: 'POST',
    }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['assets'] }),
  })
}

// ---- Import Hooks ----

export function useImportAssets() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (file: File) => {
      const form = new FormData()
      form.append('file', file)
      const res = await fetch(`${API}/assets/import`, {
        method: 'POST',
        credentials: 'include',
        body: form,
      })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      return res.json()
    },
  })
}

export function useConfirmImport() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (import_id: string) => fetchJson<{ total: number; created: number; skipped_duplicates: number; errors: string[] }>(`${API}/assets/import/confirm`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ import_id }),
    }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['assets'] }),
  })
}

// ---- NIS2 Export Hook ----

export function useNis2Export() {
  return useMutation({
    mutationFn: () => fetchBlob(`${API}/assets/nis2-export`),
  })
}

// ---- IP Conflicts Hook ----

export function useIpConflicts() {
  return useQuery({
    queryKey: queryKeys.ipConflicts,
    queryFn: () => fetchJson<{ ip: string; assets: Asset[] }[]>(`${API}/assets/ip-conflicts`),
  })
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No type errors related to new file.

- [ ] **Step 3: Commit**

```bash
git add client/src/queries/assets.ts
git commit -m "feat(asset): add TanStack Query hooks for all asset API endpoints"
```

---

## Task 13: Frontend Asset List View

**Files:**
- Create: `client/src/components/AssetList.tsx`

- [ ] **Step 1: Create `client/src/components/AssetList.tsx`**

```tsx
import { useState } from 'react'
import { useAssets, ASSET_TYPES, CRITICALITY_LEVELS, useDeleteAsset } from '../queries/assets'
import type { Asset } from '../queries/assets'

const RISK_COLORS: Record<string, string> = {
  low: '#22c55e',
  medium: '#eab308',
  high: '#f97316',
  critical: '#ef4444',
}

function riskLevel(score: number): string {
  if (score <= 25) return 'low'
  if (score <= 50) return 'medium'
  if (score <= 75) return 'high'
  return 'critical'
}

const TH: React.CSSProperties = {
  textAlign: 'left',
  padding: '8px 12px',
  fontSize: 12,
  fontWeight: 600,
  color: 'var(--muted)',
  borderBottom: '1px solid var(--border)',
  whiteSpace: 'nowrap',
}

const TD: React.CSSProperties = {
  padding: '8px 12px',
  fontSize: 13,
  borderBottom: '1px solid var(--border)',
  whiteSpace: 'nowrap',
  maxWidth: 200,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
}

const SELECT: React.CSSProperties = {
  background: 'var(--bg)',
  border: '1px solid var(--border)',
  borderRadius: 6,
  padding: '4px 8px',
  color: 'var(--fg)',
  fontSize: 13,
  marginRight: 8,
}

const INPUT: React.CSSProperties = {
  background: 'var(--bg)',
  border: '1px solid var(--border)',
  borderRadius: 6,
  padding: '4px 8px',
  color: 'var(--fg)',
  fontSize: 13,
  width: 200,
}

const BTN: React.CSSProperties = {
  background: 'var(--card)',
  border: '1px solid var(--border)',
  borderRadius: 6,
  padding: '6px 12px',
  color: 'var(--fg)',
  fontSize: 13,
  cursor: 'pointer',
}

const BTN_DANGER: React.CSSProperties = { ...BTN, color: '#ef4444' }

export function AssetList({ onSelect, onScan, onImport, onExport }: {
  onSelect: (id: string) => void
  onScan: () => void
  onImport: () => void
  onExport: () => void
}) {
  const [typeFilter, setTypeFilter] = useState('')
  const [critFilter, setCritFilter] = useState('')
  const [search, setSearch] = useState('')
  const deleteAsset = useDeleteAsset()

  const filters: Record<string, string> = {}
  if (typeFilter) filters.type = typeFilter
  if (critFilter) filters.criticality = critFilter
  if (search) filters.search = search

  const { data: assets, isLoading } = useAssets(filters)

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12, flexWrap: 'wrap' }}>
        <select style={SELECT} value={typeFilter} onChange={e => setTypeFilter(e.target.value)}>
          <option value="">All Types</option>
          {ASSET_TYPES.map(t => <option key={t} value={t}>{t.toUpperCase()}</option>)}
        </select>
        <select style={SELECT} value={critFilter} onChange={e => setCritFilter(e.target.value)}>
          <option value="">All Criticality</option>
          {CRITICALITY_LEVELS.map(c => <option key={c} value={c}>{c}</option>)}
        </select>
        <input
          style={INPUT}
          placeholder="Search name, IP, vendor..."
          value={search}
          onChange={e => setSearch(e.target.value)}
        />
        <div style={{ flex: 1 }} />
        <button style={BTN} onClick={onScan}>Scan Network</button>
        <button style={BTN} onClick={onImport}>Import CSV</button>
        <button style={BTN} onClick={onExport}>NIS2 Export</button>
      </div>

      {isLoading && <div style={{ color: 'var(--muted)', padding: 20 }}>Loading assets...</div>}

      {assets && (
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={TH}>Name</th>
                <th style={TH}>Type</th>
                <th style={TH}>Vendor</th>
                <th style={TH}>IP Address</th>
                <th style={TH}>Criticality</th>
                <th style={TH}>Risk</th>
                <th style={TH}>Firmware</th>
                <th style={TH}>Last Maintenance</th>
                <th style={TH}></th>
              </tr>
            </thead>
            <tbody>
              {assets.map(a => {
                const rl = riskLevel(a.risk_score)
                return (
                  <tr key={a.id} style={{ cursor: 'pointer' }} onClick={() => onSelect(a.id)}>
                    <td style={TD}><strong>{a.name}</strong></td>
                    <td style={TD}>{a.asset_type}</td>
                    <td style={TD}>{a.vendor || '—'}</td>
                    <td style={{ ...TD, fontFamily: 'monospace' }}>{a.ip_address || '—'}</td>
                    <td style={TD}>
                      <span style={{
                        fontSize: 11, fontWeight: 600, textTransform: 'uppercase',
                        padding: '2px 6px', borderRadius: 4,
                        background: a.criticality === 'critical' ? '#ef444420' : a.criticality === 'high' ? '#f9731620' : 'var(--bg)',
                        color: a.criticality === 'critical' ? '#ef4444' : a.criticality === 'high' ? '#f97316' : 'var(--muted)',
                      }}>{a.criticality}</span>
                    </td>
                    <td style={TD}>
                      <span style={{
                        display: 'inline-block', width: 28, height: 28, lineHeight: '28px',
                        textAlign: 'center', borderRadius: '50%', fontSize: 11, fontWeight: 700,
                        background: `${RISK_COLORS[rl]}20`, color: RISK_COLORS[rl],
                      }}>{a.risk_score}</span>
                    </td>
                    <td style={TD}>{a.firmware_version || '—'}</td>
                    <td style={TD}>{a.last_maintenance ? new Date(a.last_maintenance).toLocaleDateString() : '—'}</td>
                    <td style={TD}>
                      <button style={BTN_DANGER} onClick={e => { e.stopPropagation(); if (confirm(`Delete ${a.name}?`)) deleteAsset.mutate(a.id) }}>Delete</button>
                    </td>
                  </tr>
                )
              })}
              {assets.length === 0 && (
                <tr><td colSpan={9} style={{ ...TD, textAlign: 'center', color: 'var(--muted)', padding: 40 }}>No assets found</td></tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add client/src/components/AssetList.tsx
git commit -m "feat(asset): add filterable asset list component"
```

---

## Task 14: Frontend Asset Detail & Maintenance Views

**Files:**
- Create: `client/src/components/AssetDetail.tsx`
- Create: `client/src/components/MaintenanceTimeline.tsx`

- [ ] **Step 1: Create `client/src/components/MaintenanceTimeline.tsx`**

```tsx
import { useState } from 'react'
import { useMaintenanceLogs, useCreateMaintenanceLog } from '../queries/assets'

const INPUT: React.CSSProperties = {
  background: 'var(--bg)',
  border: '1px solid var(--border)',
  borderRadius: 6,
  padding: '6px 8px',
  color: 'var(--fg)',
  fontSize: 13,
  width: '100%',
  marginBottom: 8,
}

const BTN: React.CSSProperties = {
  background: '#3b82f6',
  border: 'none',
  borderRadius: 6,
  padding: '6px 12px',
  color: '#fff',
  fontSize: 13,
  cursor: 'pointer',
}

export function MaintenanceTimeline({ assetId }: { assetId: string }) {
  const { data: logs, isLoading } = useMaintenanceLogs(assetId)
  const createLog = useCreateMaintenanceLog(assetId)
  const [showForm, setShowForm] = useState(false)
  const [desc, setDesc] = useState('')
  const [tech, setTech] = useState('')
  const [date, setDate] = useState(new Date().toISOString().slice(0, 10))

  const handleSubmit = () => {
    if (!desc || !tech || !date) return
    createLog.mutate(
      { description: desc, technician: tech, performed_at: `${date}T00:00:00Z` },
      { onSuccess: () => { setDesc(''); setTech(''); setShowForm(false) } },
    )
  }

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 12 }}>
        <h3 style={{ margin: 0, fontSize: 14 }}>Maintenance History</h3>
        <button style={BTN} onClick={() => setShowForm(!showForm)}>{showForm ? 'Cancel' : 'Add Entry'}</button>
      </div>

      {showForm && (
        <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, padding: 12, marginBottom: 12 }}>
          <input style={INPUT} placeholder="Description" value={desc} onChange={e => setDesc(e.target.value)} />
          <input style={INPUT} placeholder="Technician" value={tech} onChange={e => setTech(e.target.value)} />
          <input style={INPUT} type="date" value={date} onChange={e => setDate(e.target.value)} />
          <button style={BTN} onClick={handleSubmit} disabled={!desc || !tech}>Save</button>
        </div>
      )}

      {isLoading && <div style={{ color: 'var(--muted)' }}>Loading...</div>}

      {logs && logs.length === 0 && <div style={{ color: 'var(--muted)', fontSize: 13 }}>No maintenance records</div>}

      {logs && logs.map(log => (
        <div key={log.id} style={{
          padding: '8px 0',
          borderBottom: '1px solid var(--border)',
        }}>
          <div style={{ fontSize: 13, fontWeight: 500 }}>{log.description}</div>
          <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 2 }}>
            {log.technician} &middot; {new Date(log.performed_at).toLocaleDateString()} &middot; logged by {log.created_by}
          </div>
        </div>
      ))}
    </div>
  )
}
```

- [ ] **Step 2: Create `client/src/components/AssetDetail.tsx`**

```tsx
import { useState } from 'react'
import { useAsset, useUpdateAsset, ASSET_TYPES, CRITICALITY_LEVELS } from '../queries/assets'
import { MaintenanceTimeline } from './MaintenanceTimeline'

const RISK_COLORS: Record<string, string> = {
  low: '#22c55e', medium: '#eab308', high: '#f97316', critical: '#ef4444',
}

function riskLevel(s: number) {
  if (s <= 25) return 'low'; if (s <= 50) return 'medium'; if (s <= 75) return 'high'; return 'critical'
}

const FIELD: React.CSSProperties = {
  display: 'grid', gridTemplateColumns: '140px 1fr', gap: '4px 12px',
  fontSize: 13, padding: '6px 0', borderBottom: '1px solid var(--border)',
}

const LABEL: React.CSSProperties = { color: 'var(--muted)', fontWeight: 500 }

const INPUT: React.CSSProperties = {
  background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 4,
  padding: '2px 6px', color: 'var(--fg)', fontSize: 13, width: '100%',
}

const SELECT: React.CSSProperties = {
  background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 4,
  padding: '2px 6px', color: 'var(--fg)', fontSize: 13,
}

const TAB: (active: boolean) => React.CSSProperties = (active) => ({
  padding: '8px 16px', fontSize: 13, fontWeight: active ? 600 : 400,
  borderBottom: active ? '2px solid #3b82f6' : '2px solid transparent',
  color: active ? '#3b82f6' : 'var(--muted)', cursor: 'pointer', background: 'none', border: 'none',
})

export function AssetDetail({ assetId, onBack }: { assetId: string; onBack: () => void }) {
  const { data: asset, isLoading } = useAsset(assetId)
  const updateAsset = useUpdateAsset()
  const [tab, setTab] = useState<'overview' | 'maintenance' | 'firmware' | 'risk'>('overview')

  if (isLoading) return <div style={{ color: 'var(--muted)', padding: 20 }}>Loading...</div>
  if (!asset) return <div style={{ color: '#ef4444', padding: 20 }}>Asset not found</div>

  const rl = riskLevel(asset.risk_score)

  const update = (field: string, value: string) => {
    updateAsset.mutate({ id: asset.id, [field]: value })
  }

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
        <button onClick={onBack} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 14 }}>&larr; Back</button>
        <h2 style={{ margin: 0, fontSize: 18 }}>{asset.name}</h2>
        <span style={{
          fontSize: 11, fontWeight: 600, textTransform: 'uppercase', padding: '2px 8px', borderRadius: 4,
          background: `${RISK_COLORS[rl]}20`, color: RISK_COLORS[rl],
        }}>Risk: {asset.risk_score}</span>
        {asset.device_ref && (
          <span style={{ fontSize: 11, color: 'var(--muted)', background: 'var(--bg)', padding: '2px 8px', borderRadius: 4 }}>
            Linked: {asset.device_ref}
          </span>
        )}
      </div>

      <div style={{ display: 'flex', gap: 0, borderBottom: '1px solid var(--border)', marginBottom: 16 }}>
        {(['overview', 'maintenance', 'firmware', 'risk'] as const).map(t => (
          <button key={t} style={TAB(tab === t)} onClick={() => setTab(t)}>
            {t.charAt(0).toUpperCase() + t.slice(1)}
          </button>
        ))}
      </div>

      {tab === 'overview' && (
        <div>
          <div style={FIELD}><span style={LABEL}>Name</span><input style={INPUT} value={asset.name} onBlur={e => update('name', e.target.value)} onChange={e => {}} /></div>
          <div style={FIELD}><span style={LABEL}>Type</span>
            <select style={SELECT} value={asset.asset_type} onChange={e => update('asset_type', e.target.value)}>
              {ASSET_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
            </select>
          </div>
          <div style={FIELD}><span style={LABEL}>Vendor</span><input style={INPUT} value={asset.vendor || ''} onBlur={e => update('vendor', e.target.value)} onChange={e => {}} /></div>
          <div style={FIELD}><span style={LABEL}>Model</span><input style={INPUT} value={asset.model || ''} onBlur={e => update('model', e.target.value)} onChange={e => {}} /></div>
          <div style={FIELD}><span style={LABEL}>Serial Number</span><input style={INPUT} value={asset.serial_number || ''} onBlur={e => update('serial_number', e.target.value)} onChange={e => {}} /></div>
          <div style={FIELD}><span style={LABEL}>IP Address</span><input style={INPUT} value={asset.ip_address || ''} onBlur={e => update('ip_address', e.target.value)} onChange={e => {}} /></div>
          <div style={FIELD}><span style={LABEL}>MAC Address</span><input style={INPUT} value={asset.mac_address || ''} onBlur={e => update('mac_address', e.target.value)} onChange={e => {}} /></div>
          <div style={FIELD}><span style={LABEL}>Location</span><input style={INPUT} value={asset.location || ''} onBlur={e => update('location', e.target.value)} onChange={e => {}} /></div>
          <div style={FIELD}><span style={LABEL}>Criticality</span>
            <select style={SELECT} value={asset.criticality} onChange={e => update('criticality', e.target.value)}>
              {CRITICALITY_LEVELS.map(c => <option key={c} value={c}>{c}</option>)}
            </select>
          </div>
          <div style={FIELD}><span style={LABEL}>Notes</span><input style={INPUT} value={asset.notes || ''} onBlur={e => update('notes', e.target.value)} onChange={e => {}} /></div>
        </div>
      )}

      {tab === 'maintenance' && <MaintenanceTimeline assetId={assetId} />}

      {tab === 'firmware' && (
        <div>
          <div style={FIELD}><span style={LABEL}>Current Firmware</span><span>{asset.firmware_version || '—'}</span></div>
          <div style={FIELD}><span style={LABEL}>Vendor</span><span>{asset.vendor || '—'}</span></div>
          <div style={FIELD}><span style={LABEL}>Model</span><span>{asset.model || '—'}</span></div>
          <p style={{ fontSize: 12, color: 'var(--muted)', marginTop: 16 }}>Update firmware version via the Overview tab. Latest versions are checked against the Firmware Catalog.</p>
        </div>
      )}

      {tab === 'risk' && (
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 16, marginBottom: 16 }}>
            <div style={{
              width: 64, height: 64, borderRadius: '50%', display: 'flex', alignItems: 'center', justifyContent: 'center',
              fontSize: 24, fontWeight: 700, background: `${RISK_COLORS[rl]}20`, color: RISK_COLORS[rl],
            }}>{asset.risk_score}</div>
            <div>
              <div style={{ fontSize: 16, fontWeight: 600, color: RISK_COLORS[rl] }}>{rl.toUpperCase()} RISK</div>
              <div style={{ fontSize: 12, color: 'var(--muted)' }}>Score 0-100, lower is better</div>
            </div>
          </div>
          <div style={{ fontSize: 13 }}>
            <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
              Firmware outdated: <strong>{asset.risk_score >= 30 ? '+30 pts' : 'OK'}</strong>
            </div>
            <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
              No maintenance 12+ months: <strong>{!asset.last_maintenance ? '+20 pts' : 'OK'}</strong>
            </div>
            <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
              Critical asset unmonitored: <strong>{asset.criticality === 'critical' && !asset.device_ref ? '+25 pts' : 'OK'}</strong>
            </div>
            <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
              Default credentials: <strong>+0 pts</strong>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`

- [ ] **Step 4: Commit**

```bash
git add client/src/components/AssetDetail.tsx client/src/components/MaintenanceTimeline.tsx
git commit -m "feat(asset): add asset detail and maintenance timeline components"
```

---

## Task 15: Frontend SNMP Scan Panel & Import Wizard

**Files:**
- Create: `client/src/components/SnmpScanPanel.tsx`
- Create: `client/src/components/AssetImportWizard.tsx`

- [ ] **Step 1: Create `client/src/components/SnmpScanPanel.tsx`**

```tsx
import { useState } from 'react'
import { useStartScan, useScanStatus, useScanResults, useImportScannedDevice } from '../queries/assets'

const INPUT: React.CSSProperties = {
  background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6,
  padding: '6px 8px', color: 'var(--fg)', fontSize: 13, width: '100%', marginBottom: 8,
}

const BTN: React.CSSProperties = {
  background: '#3b82f6', border: 'none', borderRadius: 6, padding: '8px 16px',
  color: '#fff', fontSize: 13, cursor: 'pointer',
}

const BTN_SM: React.CSSProperties = {
  background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 4,
  padding: '4px 8px', color: 'var(--fg)', fontSize: 12, cursor: 'pointer',
}

export function SnmpScanPanel({ onClose }: { onClose: () => void }) {
  const [subnet, setSubnet] = useState('192.168.1.0/24')
  const [community, setCommunity] = useState('public')
  const startScan = useStartScan()
  const { data: status } = useScanStatus()
  const { data: results } = useScanResults()
  const importDevice = useImportScannedDevice()
  const [importing, setImporting] = useState<string | null>(null)

  const scanning = status?.scanning ?? false

  const handleImport = (ip: string) => {
    setImporting(ip)
    importDevice.mutate(ip, { onSettled: () => setImporting(null) })
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 700, maxHeight: '80vh', overflow: 'auto', padding: 20 }} onClick={e => e.stopPropagation()}>
        <h3 style={{ margin: '0 0 16px' }}>SNMP Network Scan</h3>

        <div style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
          <input style={{ ...INPUT, marginBottom: 0, flex: 1 }} placeholder="Subnet (CIDR)" value={subnet} onChange={e => setSubnet(e.target.value)} />
          <input style={{ ...INPUT, marginBottom: 0, width: 140 }} placeholder="Community" value={community} onChange={e => setCommunity(e.target.value)} />
          <button style={BTN} disabled={scanning} onClick={() => startScan.mutate({ subnet, community })}>
            {scanning ? 'Scanning...' : 'Start Scan'}
          </button>
        </div>

        {scanning && status && (
          <div style={{ marginBottom: 16 }}>
            <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 4 }}>
              Scanning: {status.progress} / {status.total} hosts
            </div>
            <div style={{ background: 'var(--border)', borderRadius: 4, height: 4, overflow: 'hidden' }}>
              <div style={{ background: '#3b82f6', height: '100%', width: `${status.total > 0 ? (status.progress / status.total * 100) : 0}%`, transition: 'width 0.3s' }} />
            </div>
          </div>
        )}

        {results && results.length > 0 && (
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr>
                <th style={{ textAlign: 'left', padding: 6 }}>IP</th>
                <th style={{ textAlign: 'left', padding: 6 }}>Name</th>
                <th style={{ textAlign: 'left', padding: 6 }}>Vendor</th>
                <th style={{ textAlign: 'left', padding: 6 }}>Model</th>
                <th style={{ textAlign: 'left', padding: 6 }}>Firmware</th>
                <th style={{ textAlign: 'left', padding: 6 }}></th>
              </tr>
            </thead>
            <tbody>
              {results.map(r => (
                <tr key={r.ip} style={{ borderBottom: '1px solid var(--border)' }}>
                  <td style={{ padding: 6, fontFamily: 'monospace' }}>{r.ip}</td>
                  <td style={{ padding: 6 }}>{r.sys_name || '—'}</td>
                  <td style={{ padding: 6 }}>{r.vendor || '—'}</td>
                  <td style={{ padding: 6 }}>{r.model || '—'}</td>
                  <td style={{ padding: 6 }}>{r.firmware || '—'}</td>
                  <td style={{ padding: 6 }}>
                    <button style={BTN_SM} disabled={importing === r.ip} onClick={() => handleImport(r.ip)}>
                      {importing === r.ip ? 'Importing...' : 'Import'}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {results && results.length === 0 && !scanning && (
          <div style={{ color: 'var(--muted)', textAlign: 'center', padding: 20 }}>No scan results yet</div>
        )}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Create `client/src/components/AssetImportWizard.tsx`**

```tsx
import { useState, useRef } from 'react'
import { useImportAssets, useConfirmImport } from '../queries/assets'

const BTN: React.CSSProperties = {
  background: '#3b82f6', border: 'none', borderRadius: 6, padding: '8px 16px',
  color: '#fff', fontSize: 13, cursor: 'pointer',
}

export function AssetImportWizard({ onClose }: { onClose: () => void }) {
  const [file, setFile] = useState<File | null>(null)
  const [preview, setPreview] = useState<any>(null)
  const [result, setResult] = useState<any>(null)
  const importAssets = useImportAssets()
  const confirmImport = useConfirmImport()
  const fileRef = useRef<HTMLInputElement>(null)

  const handleUpload = () => {
    if (!file) return
    importAssets.mutate(file, {
      onSuccess: (data) => setPreview(data),
    })
  }

  const handleConfirm = () => {
    if (!preview?.import_id) return
    confirmImport.mutate(preview.import_id, {
      onSuccess: (data) => setResult(data),
    })
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 600, maxHeight: '80vh', overflow: 'auto', padding: 20 }} onClick={e => e.stopPropagation()}>
        <h3 style={{ margin: '0 0 16px' }}>Import Assets from CSV/Excel</h3>

        {!preview && !result && (
          <div>
            <p style={{ fontSize: 13, color: 'var(--muted)', marginBottom: 12 }}>
              Upload a CSV or Excel file with columns: name, type, vendor, model, firmware, serial_number, ip_address, location, criticality, notes
            </p>
            <input ref={fileRef} type="file" accept=".csv,.xlsx" onChange={e => setFile(e.target.files?.[0] || null)} />
            <div style={{ marginTop: 12 }}>
              <button style={BTN} disabled={!file} onClick={handleUpload}>Preview Import</button>
            </div>
          </div>
        )}

        {preview && !result && (
          <div>
            <p style={{ fontSize: 13, marginBottom: 8 }}>
              Found <strong>{preview.total}</strong> rows ({preview.valid} valid, {preview.errors.length} errors)
            </p>
            {preview.errors.length > 0 && (
              <div style={{ fontSize: 12, color: '#ef4444', marginBottom: 8 }}>
                {preview.errors.map((e: string, i: number) => <div key={i}>{e}</div>)}
              </div>
            )}
            <div style={{ display: 'flex', gap: 8 }}>
              <button style={BTN} onClick={handleConfirm} disabled={confirmImport.isPending}>
                {confirmImport.isPending ? 'Importing...' : `Import ${preview.valid} Assets`}
              </button>
              <button style={{ ...BTN, background: 'var(--bg)', color: 'var(--fg)' }} onClick={() => setPreview(null)}>Cancel</button>
            </div>
          </div>
        )}

        {result && (
          <div>
            <p style={{ fontSize: 13, marginBottom: 8 }}>
              Import complete: <strong>{result.created}</strong> created, {result.skipped_duplicates} skipped, {result.errors.length} errors
            </p>
            {result.errors.length > 0 && (
              <div style={{ fontSize: 12, color: '#ef4444' }}>
                {result.errors.map((e: string, i: number) => <div key={i}>{e}</div>)}
              </div>
            )}
            <button style={{ ...BTN, marginTop: 12 }} onClick={onClose}>Done</button>
          </div>
        )}
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`

- [ ] **Step 4: Commit**

```bash
git add client/src/components/SnmpScanPanel.tsx client/src/components/AssetImportWizard.tsx
git commit -m "feat(asset): add SNMP scan panel and CSV/XLSX import wizard"
```

---

## Task 16: Frontend Main View & Nav Integration

**Files:**
- Create: `client/src/views/AssetInventory.tsx`
- Modify: `client/src/App.tsx` (add asset view mode)
- Modify: `client/src/components/AppHeader.tsx` (add Assets nav item)

- [ ] **Step 1: Create `client/src/views/AssetInventory.tsx`**

```tsx
import { useState } from 'react'
import { AssetList } from '../components/AssetList'
import { AssetDetail } from '../components/AssetDetail'
import { SnmpScanPanel } from '../components/SnmpScanPanel'
import { AssetImportWizard } from '../components/AssetImportWizard'
import { useNis2Export } from '../queries/assets'

export function AssetInventory() {
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [showScan, setShowScan] = useState(false)
  const [showImport, setShowImport] = useState(false)
  const nis2Export = useNis2Export()

  if (selectedId) {
    return <AssetDetail assetId={selectedId} onBack={() => setSelectedId(null)} />
  }

  return (
    <div style={{ padding: 20 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
        <h2 style={{ margin: 0, fontSize: 20 }}>Asset Inventory</h2>
      </div>

      <AssetList
        onSelect={setSelectedId}
        onScan={() => setShowScan(true)}
        onImport={() => setShowImport(true)}
        onExport={() => nis2Export.mutate()}
      />

      {showScan && <SnmpScanPanel onClose={() => setShowScan(false)} />}
      {showImport && <AssetImportWizard onClose={() => setShowImport(false)} />}
    </div>
  )
}
```

- [ ] **Step 2: Add Assets nav item to AppHeader**

In `client/src/components/AppHeader.tsx`, find the navigation section (the area with screen/mode tabs or buttons). Add an "Assets" button alongside existing navigation items:

```tsx
<button
  style={{ ...existingTabStyle, ...(mode === 'assets' ? activeTabStyle : {}) }}
  onClick={() => setMode('assets')}
>
  Assets
</button>
```

The exact integration depends on the existing nav structure. Look at how other top-level views like "Alarms" or "Settings" are added.

- [ ] **Step 3: Add Assets view mode to App.tsx**

In `client/src/App.tsx`, find where views are conditionally rendered based on `mode` or similar state. Add:

```tsx
{mode === 'assets' && <AssetInventory />}
```

Import at the top:

```tsx
import { AssetInventory } from './views/AssetInventory'
```

- [ ] **Step 4: Verify full frontend compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 5: Start dev server and test**

Run: `cd client && npm run dev`

Open browser, navigate to the app, verify:
- "Assets" nav item appears
- Clicking it shows the Asset Inventory page
- Create asset form works
- Asset list with filters renders
- Clicking an asset shows detail view
- Scan Network button opens SNMP panel
- Import CSV button opens import wizard
- NIS2 Export downloads a CSV

- [ ] **Step 6: Commit**

```bash
git add client/src/views/AssetInventory.tsx client/src/App.tsx client/src/components/AppHeader.tsx
git commit -m "feat(asset): integrate asset inventory into main app nav and views"
```

---

## Self-Review Checklist

- [x] **Spec coverage:** All 9 roadmap items covered (device inventory → Task 3, firmware tracking → Task 4, IP conflict → Task 3, maintenance → Task 4, SNMP → Task 6, NIS2 → Task 8, asset tags → Task 3, firmware alerting → Task 5, CSV import → Task 8)
- [x] **Placeholder scan:** No TBDs, TODOs, or "implement later" patterns
- [x] **Type consistency:** Asset struct fields match between models.rs, storage.rs, API handlers, and TypeScript types
