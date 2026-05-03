# v0.34.0 — OT Asset Inventory

**Date:** 2026-05-03
**Status:** Approved
**Tier:** Professional ($499)
**Timeline:** 4-5 weeks

---

## Goal

Kill the spreadsheet. Lightweight asset tracking built into the SCADA system. Auto-populated from configured devices, extended with SNMP scanning, enriched with maintenance history and firmware tracking, exported for NIS2 compliance.

## Design Principle

Asset inventory is a **new module** that sits alongside existing protocol modules. It links to protocol devices via a `device_ref` field but does not modify existing device tables or creation flows. Protocol device handlers create a corresponding asset record as a side effect.

---

## Architecture

**Approach A: Unified Asset Table** — single `assets` table linked to protocol devices via `device_ref`. SNMP-only devices (switches, routers, gateways) have no `device_ref` and exist as standalone asset records.

### New Files

```
src/
├── asset/                    # New module
│   ├── mod.rs                # Module exports
│   ├── models.rs             # Asset, MaintenanceLog, FirmwareCatalog, SnmpScanResult structs
│   ├── storage.rs            # SQLite CRUD for all asset tables
│   ├── risk.rs               # Risk score computation engine
│   ├── snmp.rs               # SNMP scanning (scan trigger, result parsing)
│   └── sync.rs               # Device-to-asset auto-sync hooks
├── api/
│   ├── assets.rs             # Asset CRUD endpoints
│   ├── maintenance.rs        # Maintenance log endpoints
│   ├── firmware_catalog.rs   # Firmware catalog CRUD
│   ├── snmp_scan.rs          # SNMP scan trigger + results + import
│   ├── asset_import.rs       # CSV/Excel bulk import
│   └── nis2_export.rs        # NIS2 compliance CSV export
client/src/
├── views/
│   └── AssetInventory.tsx    # Main asset inventory page
├── components/
│   ├── AssetList.tsx          # Filterable table
│   ├── AssetDetail.tsx        # Detail view with tabs
│   ├── MaintenanceTimeline.tsx
│   ├── SnmpScanPanel.tsx
│   ├── AssetImportWizard.tsx
│   └── Nis2ExportButton.tsx
```

### Dependencies

- `rs-snmp` or equivalent pure-Rust SNMP crate (SNMPv1/v2c/v3 support, async)
- `calamine` for Excel (.xlsx) import parsing
- Existing: `rusqlite`, `axum`, `utoipa`, `serde`, `tokio`

---

## Data Model

### Table: `assets`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUID |
| device_ref | TEXT | NULL, UNIQUE | FK format: `modbus:uuid`, `opcua:uuid`, `s7:uuid`, `enip:uuid` |
| name | TEXT | NOT NULL | Display name |
| asset_type | TEXT | NOT NULL | `plc`, `rtu`, `hmi`, `sensor`, `switch`, `router`, `gateway`, `other` |
| vendor | TEXT | | Manufacturer |
| model | TEXT | | Device model |
| firmware_version | TEXT | | Current firmware |
| serial_number | TEXT | | Hardware serial |
| ip_address | TEXT | | IPv4 or IPv6 |
| mac_address | TEXT | | From ARP, SNMP, or manual |
| location | TEXT | | Physical location (building/room/rack) |
| criticality | TEXT | NOT NULL, DEFAULT 'medium' | `low`, `medium`, `high`, `critical` |
| tags | TEXT | DEFAULT '[]' | JSON array of arbitrary tag strings |
| risk_score | INTEGER | DEFAULT 0 | 0-100, computed |
| last_maintenance | TEXT | | ISO 8601 date |
| notes | TEXT | | Free-form notes |
| created_at | TEXT | NOT NULL | ISO 8601 |
| updated_at | TEXT | NOT NULL | ISO 8601 |

Indexes: `idx_assets_device_ref` on `device_ref`, `idx_assets_ip` on `ip_address`, `idx_assets_type` on `asset_type`, `idx_assets_criticality` on `criticality`.

### Table: `maintenance_logs`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUID |
| asset_id | TEXT | FK → assets.id, ON DELETE CASCADE | Parent asset |
| description | TEXT | NOT NULL | What was done |
| technician | TEXT | NOT NULL | Who performed it |
| performed_at | TEXT | NOT NULL | ISO 8601 date |
| created_by | TEXT | NOT NULL | User who logged it |
| created_at | TEXT | NOT NULL | ISO 8601 |

Index: `idx_maintenance_asset_id` on `asset_id`.

### Table: `firmware_catalog`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUID |
| vendor | TEXT | NOT NULL | e.g. "Siemens" |
| model | TEXT | NOT NULL | e.g. "S7-1200" |
| latest_version | TEXT | NOT NULL | e.g. "V4.6" |
| release_date | TEXT | | ISO 8601 |
| notes | TEXT | | Changelog summary |
| updated_at | TEXT | NOT NULL | ISO 8601 |

Unique constraint: `(vendor, model)`. Index: `idx_firmware_vendor_model`.

Seed data: Pre-populate common devices (Siemens S7-1200/S7-1500, Allen-Bradley ControlLogix/CompactLogix, Modicon M340/M580, etc.).

### Table: `snmp_scan_results`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| ip | TEXT | PK | Scanned IP address |
| sys_descr | TEXT | | SNMP sysDescr.0 |
| sys_name | TEXT | | SNMP sysName.0 |
| sys_uptime | INTEGER | | SNMP sysUpTime.0 (centiseconds) |
| vendor | TEXT | | Parsed from sysDescr |
| model | TEXT | | Parsed from sysDescr |
| firmware | TEXT | | Parsed from sysDescr |
| mac_address | TEXT | | From ipNetToMediaPhysAddress or ifPhysAddress |
| scanned_at | TEXT | NOT NULL | ISO 8601 |

This table is ephemeral — cleared and repopulated on each scan.

---

## API Endpoints

### Asset CRUD

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/assets` | List assets. Query params: `type`, `criticality`, `vendor`, `location`, `tag`, `risk_min`, `risk_max`, `search` |
| POST | `/api/v1/assets` | Create asset manually |
| GET | `/api/v1/assets/{id}` | Get asset detail (includes linked device info, last 10 maintenance logs, firmware comparison) |
| PATCH | `/api/v1/assets/{id}` | Update asset metadata |
| DELETE | `/api/v1/assets/{id}` | Delete asset. Clears `device_ref` on linked device if present |

### Maintenance Logs

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/assets/{id}/maintenance` | List maintenance history (paginated, newest first) |
| POST | `/api/v1/assets/{id}/maintenance` | Add maintenance log entry |

### Firmware Catalog

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/firmware-catalog` | List all entries. Filterable by vendor, model |
| POST | `/api/v1/firmware-catalog` | Add catalog entry |
| PATCH | `/api/v1/firmware-catalog/{id}` | Update latest version |
| DELETE | `/api/v1/firmware-catalog/{id}` | Remove entry |

### SNMP Scanning

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/snmp/scan` | Start scan. Body: `{ subnet: "192.168.1.0/24", community: "public", timeout_ms: 2000 }` |
| GET | `/api/v1/snmp/scan/status` | Get scan status (idle/scanning/complete, progress %, device count) |
| GET | `/api/v1/snmp/scan/results` | Get latest scan results |
| POST | `/api/v1/snmp/scan/{ip}/import` | Import scanned device as asset |

### IP Conflict Detection

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/assets/ip-conflicts` | Return array of `{ ip, assets: [...], devices: [...] }` for duplicate IPs |

### NIS2 Compliance Export

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/assets/nis2-export` | Download CSV. Columns: name, type, vendor, model, serial_number, firmware, latest_firmware, firmware_status, last_maintenance, risk_score, criticality, location, ip_address |

`firmware_status` values: `current` (matches catalog), `outdated` (older than catalog), `unknown` (no catalog entry).

### CSV/Excel Import

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/assets/import` | Multipart upload. Body: file + optional column mapping. Parses file, validates rows, returns preview with `{ rows[], total, valid, errors[] }`. Does NOT create assets yet |
| POST | `/api/v1/assets/import/confirm` | Confirms and commits the previewed import. Body: `{ import_id }` (returned from preview step). Creates assets and returns `{ total, created, skipped_duplicates, errors[] }` |

Supported formats: `.csv`, `.xlsx`. Column auto-mapping by header name (case-insensitive match to asset fields). Unmapped columns ignored. Preview expires after 10 minutes.

---

## SNMP Scanning Flow

1. User triggers `POST /api/v1/snmp/scan` with subnet CIDR and community string
2. Backend spawns async task:
   - Enumerate IPs in CIDR range
   - For each IP: SNMP GET `sysDescr.0` (1.3.6.1.2.1.1.1.0), `sysName.0` (1.3.6.1.2.1.1.5.0), `sysUpTime.0` (1.3.6.1.2.1.1.3.0)
   - Parse `sysDescr` to extract vendor, model, firmware (regex patterns for common vendors)
   - GET `ifPhysAddress` (1.3.6.1.2.1.2.2.1.6) for MAC address
3. Results upsert `snmp_scan_results` table
4. User views results in frontend, clicks "Import" per device to create asset record
5. Imported assets have no `device_ref` (network infrastructure is not polled by protocol drivers)

**Concurrency:** Scan runs as a background `tokio::spawn` task. Progress tracked in memory (scan status endpoint). Only one scan at a time — reject if scan already in progress.

**sysDescr parsing patterns:**
- Siemens: `Siemens, SIMATIC S7, ...`
- Allen-Bradley: `Rockwell Automation, ...` or `Allen-Bradley, ...`
- Schneider: `Schneider Electric, ...` or `Modicon, ...`
- Cisco: `Cisco IOS Software, ...`
- Generic: Use full sysDescr as-is, leave vendor/model blank

---

## Risk Score Engine

Computed on each relevant change (firmware update, maintenance log, metadata change). Stored in `assets.risk_score`.

| Factor | Points | Detection Method |
|--------|--------|-----------------|
| Firmware outdated | +30 | `asset.firmware_version` exists and `firmware_catalog` has matching vendor/model with different `latest_version` |
| No maintenance in 12+ months | +20 | `asset.last_maintenance` is NULL or >365 days from today |
| Critical asset unmonitored | +25 | `asset.criticality` is `critical` and `device_ref` is NULL |
| Default credentials | +25 | SNMP community is "public" or asset has known default credentials flag |

Score capped at 100. Factors are additive. If firmware is current AND maintenance is recent AND asset is monitored AND no default creds, score = 0.

**Risk level display:**
- 0-25: Low (green)
- 26-50: Medium (yellow)
- 51-75: High (orange)
- 76-100: Critical (red)

**Firmware alerting:** When risk score recomputes and firmware factor triggers (+30), emit a notification via existing notification system if configured. Notification type: `firmware_outdated`.

---

## Device-to-Asset Auto-Sync

When a protocol device is created via existing API:
1. Device handler creates the protocol device as normal
2. Handler calls `asset::sync::link_device(device_ref, name, ip, device_type)`
3. This creates an asset record with the `device_ref` set and basic fields populated from device config

When a protocol device is deleted:
1. Handler calls `asset::sync::unlink_device(device_ref)`
2. Asset record's `device_ref` is set to NULL, but the asset stays (may represent decommissioned equipment)
3. `criticality` is auto-set to `low` if it was `critical` (decommissioned)

When a protocol device's IP or name changes:
1. Handler calls `asset::sync::update_device_ref(device_ref, new_name, new_ip)`
2. Corresponding asset fields are updated

---

## Frontend Views

### Asset Inventory Page (`AssetInventory.tsx`)

Top-level nav item: "Assets" (icon: server/rack).

**List View (`AssetList.tsx`):**
- Filterable table with columns: Name, Type, Vendor, IP, Criticality, Risk Score (badge), Last Maintenance
- Filters: type dropdown, criticality dropdown, vendor dropdown (populated from data), risk range slider, text search (name/IP/vendor)
- Sortable by any column
- Click row → detail view
- Toolbar: "Scan Network" button, "Import CSV" button, "NIS2 Export" button

**Detail View (`AssetDetail.tsx`):**
- Header: Name, type badge, risk score badge, linked device link (if any)
- Tab 1 — Overview: All metadata fields (editable inline)
- Tab 2 — Maintenance History (`MaintenanceTimeline.tsx`): Reverse-chronological list with add button
- Tab 3 — Firmware: Current version, catalog latest, status indicator (current/outdated/unknown)
- Tab 4 — Risk Breakdown: List of contributing factors with points

**SNMP Scan Panel (`SnmpScanPanel.tsx`):**
- Modal/sidebar with subnet input, community string, timeout
- Progress bar during scan
- Results table: IP, Name, Vendor, Model, Firmware, MAC
- "Import" button per row (creates asset)
- "Import All" button

**Import Wizard (`AssetImportWizard.tsx`):**
- Step 1: Upload file (CSV or XLSX)
- Step 2: Preview with auto-mapped columns (editable mapping)
- Step 3: Validation summary (duplicates flagged, missing required fields)
- Step 4: Confirm import

**NIS2 Export Button (`Nis2ExportButton.tsx`):**
- Downloads CSV directly from `GET /api/v1/assets/nis2-export`
- Filename: `voltrus-nis2-export-{date}.csv`

### Device List Integration

Existing device list pages (Modbus, OPC-UA, S7, EtherNet/IP) show a small "Asset tracked" badge next to devices that have a linked asset record. Clicking the badge navigates to the asset detail view.

---

## Error Handling

- SNMP scan timeout: Individual IP timeouts don't fail the whole scan. Timed-out IPs are excluded from results.
- Import validation: Rows with missing required fields (name, type) are skipped and reported in `errors[]`.
- Firmware catalog: If no matching vendor/model entry exists, firmware status is `unknown` (not an error).
- IP conflict detection: Compares across both `assets` table and all protocol device tables.

---

## Testing Strategy

- Unit tests: Risk score computation (all factor combinations), sysDescr parsing, CSV import column mapping
- Integration tests: API endpoint CRUD, SNMP scan mock, firmware catalog matching
- Frontend: Component render tests for AssetList, AssetDetail, ImportWizard
