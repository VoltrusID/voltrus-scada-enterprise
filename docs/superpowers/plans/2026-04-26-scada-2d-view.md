# SCADA 2D View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a P&ID schematic-style 2D SCADA view mode with SVG equipment symbols, real-time sensor data overlays, and independent layout storage.

**Architecture:** SVG canvas rendered natively in React with inline SVG components for equipment, polylines for connections, and SVG text for data labels. Backend follows the same screen-scoped pattern as twin_layouts with two new SQLite tables for scada equipment and connections.

**Tech Stack:** Rust/Axum (backend), SQLite/rusqlite (storage), React/TypeScript (frontend), TanStack Query (data fetching), SVG (2D rendering)

**Design spec:** `docs/superpowers/specs/2026-04-26-scada-2d-view-design.md`

---

## File Structure

### Backend (create/modify)
| File | Action | Responsibility |
|------|--------|----------------|
| `src/models.rs` | Modify | Add `ScadaEquipment`, `ScadaConnection` structs |
| `src/storage.rs` | Modify | Add DB schema, storage methods for scada data |
| `src/api/layouts.rs` | Modify | Add GET/PUT handlers for scada endpoints |
| `src/api/mod.rs` | Modify | Register new routes |

### Frontend (create/modify)
| File | Action | Responsibility |
|------|--------|----------------|
| `client/src/types.ts` | Modify | Add ScadaEquipment, ScadaConnection, ScadaEquipmentType types |
| `client/src/queries.ts` | Modify | Add query/mutation hooks for scada layout |
| `client/src/App.tsx` | Modify | Add 'scada' to viewMode, wire ScadaView component |
| `client/src/ScadaView.tsx` | Create | Main SCADA view orchestrator component |
| `client/src/scada/defaults.ts` | Create | Default sizes and properties per equipment type |
| `client/src/scada/ScadaCanvas.tsx` | Create | SVG root with pan/zoom, renders equipment + connections |
| `client/src/scada/ScadaEquipment.tsx` | Create | Equipment wrapper: SVG asset + data label + drag/resize |
| `client/src/scada/ScadaConnection.tsx` | Create | SVG polyline connection with flow animation |
| `client/src/scada/ScadaEditToolbar.tsx` | Create | Edit mode toolbar: tools, save, zoom |
| `client/src/scada/ScadaPalette.tsx` | Create | Equipment type picker |
| `client/src/scada/ScadaEditPanel.tsx` | Create | Side panel for editing equipment props & sensor binding |
| `client/src/scada/assets/SolarPanelSvg.tsx` | Create | P&ID solar panel symbol |
| `client/src/scada/assets/InverterSvg.tsx` | Create | P&ID inverter symbol |
| `client/src/scada/assets/GatewaySvg.tsx` | Create | P&ID gateway/PLC symbol |
| `client/src/scada/assets/TransformerSvg.tsx` | Create | P&ID transformer symbol |
| `client/src/scada/assets/BatterySvg.tsx` | Create | P&ID battery symbol |
| `client/src/scada/assets/MeterSvg.tsx` | Create | P&ID meter symbol |
| `client/src/scada/assets/CircuitBreakerSvg.tsx` | Create | P&ID circuit breaker symbol |
| `client/src/scada/assets/index.ts` | Create | Barrel export for all SVG assets |

---

## Task 1: Backend — Models and Schema

**Files:**
- Modify: `src/models.rs` (add after `CanvasLayoutItem` struct, around line 365)
- Modify: `src/storage.rs` (add to `init_schema` function, around line 1289; add to imports at line 6-9)

- [ ] **Step 1: Add ScadaEquipment and ScadaConnection models to models.rs**

Add after the `CanvasLayoutItem` struct (after line 365):

```rust
// ── SCADA 2D view types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScadaEquipment {
    pub screen_id: String,
    pub equipment_id: String,
    pub equipment_type: String,
    pub equipment_name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub sensor_bindings: String, // JSON array of sensor IDs as strings
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScadaConnection {
    pub screen_id: String,
    pub connection_id: String,
    pub from_equipment: String,
    pub to_equipment: String,
    pub waypoints: String, // JSON array of [x, y] pairs
    pub connection_type: String, // "power" | "data" | "flow"
}
```

- [ ] **Step 2: Update storage.rs imports to include new models**

In `src/storage.rs`, update the import at line 6-9 to include the new types:

```rust
use crate::models::{
    DataPoint, Device, ScreenLayoutRow, DataSource, Setting, TwinLayout, TwinConnection, User, UserShortcut,
    Alarm, CanvasLayoutItem,
    ModbusDevice, ModbusRegister, ModbusRegisterInput, DeviceConfig,
    ScadaEquipment, ScadaConnection,
};
```

- [ ] **Step 3: Add scada DB tables to init_schema**

In `src/storage.rs`, append to the `init_schema` function's `execute_batch` call (after the `idx_modbus_registers_device` index, around line 1291):

```sql
CREATE TABLE IF NOT EXISTS scada_equipment (
    screen_id TEXT NOT NULL,
    equipment_id TEXT NOT NULL,
    equipment_type TEXT NOT NULL,
    equipment_name TEXT NOT NULL DEFAULT '',
    x REAL NOT NULL DEFAULT 0,
    y REAL NOT NULL DEFAULT 0,
    width REAL NOT NULL DEFAULT 100,
    height REAL NOT NULL DEFAULT 60,
    rotation REAL NOT NULL DEFAULT 0,
    sensor_bindings TEXT NOT NULL DEFAULT '[]',
    PRIMARY KEY (screen_id, equipment_id)
);
CREATE TABLE IF NOT EXISTS scada_connections (
    screen_id TEXT NOT NULL,
    connection_id TEXT NOT NULL,
    from_equipment TEXT NOT NULL,
    to_equipment TEXT NOT NULL,
    waypoints TEXT NOT NULL DEFAULT '[]',
    connection_type TEXT NOT NULL DEFAULT 'power',
    PRIMARY KEY (screen_id, connection_id)
);
```

- [ ] **Step 4: Build and verify compilation**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles with warnings about unused types (that's fine, storage methods come next)

- [ ] **Step 5: Commit**

```bash
git add src/models.rs src/storage.rs
git commit -m "feat(scada): add ScadaEquipment and ScadaConnection models and DB schema"
```

---

## Task 2: Backend — Storage Methods

**Files:**
- Modify: `src/storage.rs` (add new methods in the `impl Storage` block, before the modbus section around line 1310)

- [ ] **Step 1: Add get/save scada equipment methods**

Add to `src/storage.rs` in the main `impl Storage` block (after `save_canvas_layouts`, before the Settings section around line 763):

```rust

    // ── SCADA 2D Equipment ──

    pub async fn get_scada_equipment(&self, screen_id: &str) -> Result<Vec<ScadaEquipment>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT screen_id, equipment_id, equipment_type, equipment_name, x, y, width, height, rotation, sensor_bindings FROM scada_equipment WHERE screen_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(ScadaEquipment {
                        screen_id: row.get(0)?,
                        equipment_id: row.get(1)?,
                        equipment_type: row.get(2)?,
                        equipment_name: row.get(3)?,
                        x: row.get(4)?,
                        y: row.get(5)?,
                        width: row.get(6)?,
                        height: row.get(7)?,
                        rotation: row.get(8)?,
                        sensor_bindings: row.get(9)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_scada_equipment(&self, screen_id: &str, items: &[ScadaEquipment]) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        let items = items.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM scada_equipment WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete scada equipment failed")?;
            for item in &items {
                tx.execute(
                    "INSERT INTO scada_equipment (screen_id, equipment_id, equipment_type, equipment_name, x, y, width, height, rotation, sensor_bindings)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    rusqlite::params![item.screen_id, item.equipment_id, item.equipment_type, item.equipment_name,
                                      item.x, item.y, item.width, item.height, item.rotation, item.sensor_bindings],
                )
                .context("Insert scada equipment failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }
```

- [ ] **Step 2: Add get/save scada connections methods**

Add right after the scada equipment methods:

```rust

    // ── SCADA 2D Connections ──

    pub async fn get_scada_connections(&self, screen_id: &str) -> Result<Vec<ScadaConnection>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT screen_id, connection_id, from_equipment, to_equipment, waypoints, connection_type FROM scada_connections WHERE screen_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(ScadaConnection {
                        screen_id: row.get(0)?,
                        connection_id: row.get(1)?,
                        from_equipment: row.get(2)?,
                        to_equipment: row.get(3)?,
                        waypoints: row.get(4)?,
                        connection_type: row.get(5)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_scada_connections(&self, screen_id: &str, connections: &[ScadaConnection]) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        let connections = connections.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM scada_connections WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete scada connections failed")?;
            for c in &connections {
                tx.execute(
                    "INSERT INTO scada_connections (screen_id, connection_id, from_equipment, to_equipment, waypoints, connection_type)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![c.screen_id, c.connection_id, c.from_equipment, c.to_equipment,
                                      c.waypoints, c.connection_type],
                )
                .context("Insert scada connection failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }
```

- [ ] **Step 3: Build and verify**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles with warnings about unused methods (API handlers come next)

- [ ] **Step 4: Commit**

```bash
git add src/storage.rs
git commit -m "feat(scada): add storage methods for scada equipment and connections"
```

---

## Task 3: Backend — API Handlers and Routes

**Files:**
- Modify: `src/api/layouts.rs` (add 4 new handlers at end of file)
- Modify: `src/api/mod.rs` (add 2 new routes in protected section)

- [ ] **Step 1: Add scada API handlers to layouts.rs**

Append to end of `src/api/layouts.rs`:

```rust

/// GET /api/screens/{id}/scada-equipment — get scada equipment for a screen
pub async fn get_scada_equipment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let items = state.storage.get_scada_equipment(&id).await?;
    Ok(Json(json!({"items": items})))
}

/// PUT /api/screens/{id}/scada-equipment — accept array of scada equipment
pub async fn save_scada_equipment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let arr = body
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Expected {items: [...]}"))?;

    let mut items = Vec::new();
    for item in arr {
        items.push(ScadaEquipment {
            screen_id: id.clone(),
            equipment_id: item["equipment_id"].as_str().unwrap_or("").to_string(),
            equipment_type: item["equipment_type"].as_str().unwrap_or("").to_string(),
            equipment_name: item["equipment_name"].as_str().unwrap_or("").to_string(),
            x: item["x"].as_f64().unwrap_or(0.0),
            y: item["y"].as_f64().unwrap_or(0.0),
            width: item["width"].as_f64().unwrap_or(100.0),
            height: item["height"].as_f64().unwrap_or(60.0),
            rotation: item["rotation"].as_f64().unwrap_or(0.0),
            sensor_bindings: item["sensor_bindings"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| serde_json::to_string(&item["sensor_bindings"]).unwrap_or("[]".to_string())),
        });
    }

    state.storage.save_scada_equipment(&id, &items).await?;
    Ok(Json(json!({"ok": true})))
}

/// GET /api/screens/{id}/scada-connections — get scada connections for a screen
pub async fn get_scada_connections(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let connections = state.storage.get_scada_connections(&id).await?;
    Ok(Json(json!({"items": connections})))
}

/// PUT /api/screens/{id}/scada-connections — accept array of scada connections
pub async fn save_scada_connections(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let arr = body
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Expected {items: [...]}"))?;

    let mut connections = Vec::new();
    for item in arr {
        connections.push(ScadaConnection {
            screen_id: id.clone(),
            connection_id: item["connection_id"].as_str().unwrap_or("").to_string(),
            from_equipment: item["from_equipment"].as_str().unwrap_or("").to_string(),
            to_equipment: item["to_equipment"].as_str().unwrap_or("").to_string(),
            waypoints: item["waypoints"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| serde_json::to_string(&item["waypoints"]).unwrap_or("[]".to_string())),
            connection_type: item["connection_type"].as_str().unwrap_or("power").to_string(),
        });
    }

    state.storage.save_scada_connections(&id, &connections).await?;
    Ok(Json(json!({"ok": true})))
}
```

- [ ] **Step 2: Register routes in mod.rs**

In `src/api/mod.rs`, add these routes in the `protected` router block (after the twin-camera route, around line 125):

```rust
        .route(
            "/api/screens/{id}/scada-equipment",
            get(layouts::get_scada_equipment).put(layouts::save_scada_equipment),
        )
        .route(
            "/api/screens/{id}/scada-connections",
            get(layouts::get_scada_connections).put(layouts::save_scada_connections),
        )
```

- [ ] **Step 3: Build and verify**

Run: `cargo build 2>&1 | head -20`
Expected: Clean compilation, no errors

- [ ] **Step 4: Run existing tests**

Run: `cargo test 2>&1 | tail -20`
Expected: All existing tests pass

- [ ] **Step 5: Commit**

```bash
git add src/api/layouts.rs src/api/mod.rs
git commit -m "feat(scada): add API endpoints for scada equipment and connections"
```

---

## Task 4: Frontend — Types and Query Hooks

**Files:**
- Modify: `client/src/types.ts`
- Modify: `client/src/queries.ts`

- [ ] **Step 1: Add SCADA types to types.ts**

Append to end of `client/src/types.ts`:

```typescript
// ── SCADA 2D view types ──

export type ScadaEquipmentType = 'solar_panel' | 'inverter' | 'gateway' | 'transformer' | 'battery' | 'meter' | 'circuit_breaker'
export type ScadaConnectionType = 'power' | 'data' | 'flow'

export const SCADA_EQUIPMENT_TYPES: { value: ScadaEquipmentType; label: string; defaultWidth: number; defaultHeight: number }[] = [
  { value: 'solar_panel', label: 'Solar Panel', defaultWidth: 100, defaultHeight: 60 },
  { value: 'inverter', label: 'Inverter', defaultWidth: 80, defaultHeight: 60 },
  { value: 'gateway', label: 'Gateway/PLC', defaultWidth: 80, defaultHeight: 70 },
  { value: 'transformer', label: 'Transformer', defaultWidth: 90, defaultHeight: 80 },
  { value: 'battery', label: 'Battery', defaultWidth: 70, defaultHeight: 80 },
  { value: 'meter', label: 'Meter', defaultWidth: 60, defaultHeight: 60 },
  { value: 'circuit_breaker', label: 'Circuit Breaker', defaultWidth: 60, defaultHeight: 70 },
]

export interface ScadaEquipmentItem {
  equipment_id: string
  equipment_type: ScadaEquipmentType
  equipment_name: string
  x: number
  y: number
  width: number
  height: number
  rotation: number
  sensor_bindings: string // JSON string of sensor ID array
}

export interface ScadaConnectionItem {
  connection_id: string
  from_equipment: string
  to_equipment: string
  waypoints: string // JSON string of [x, y] pairs
  connection_type: ScadaConnectionType
}
```

- [ ] **Step 2: Add SCADA query/mutation hooks to queries.ts**

In `client/src/queries.ts`:

First, add to the `queryKeys` object (after `modbusStatus`):

```typescript
  scadaEquipment:  (screenId: string) => ['scadaEquipment', screenId] as const,
  scadaConnections: (screenId: string) => ['scadaConnections', screenId] as const,
```

Also update the import at line 2 to include the new types:

```typescript
import type { SensorConfig, Screen, ScreenLayouts, DataPoint, Alarm, CanvasLayoutItem, ModbusDevice, ModbusRegister, ScadaEquipmentItem, ScadaConnectionItem } from './types'
```

Then add these hooks at the end of the file (before the closing, after the modbus hooks):

```typescript

// ── SCADA 2D view hooks ────────────────────────────────────────────────

export function useScadaEquipment(screenId: string, enabled: boolean) {
  const actualId = screenId === 'all' ? '__overview' : screenId
  return useQuery({
    queryKey: queryKeys.scadaEquipment(screenId),
    queryFn: () => fetchJson<{ items: ScadaEquipmentItem[] }>(`/api/screens/${actualId}/scada-equipment`).then(r => r.items),
    enabled,
  })
}

export function useScadaConnections(screenId: string, enabled: boolean) {
  const actualId = screenId === 'all' ? '__overview' : screenId
  return useQuery({
    queryKey: queryKeys.scadaConnections(screenId),
    queryFn: () => fetchJson<{ items: ScadaConnectionItem[] }>(`/api/screens/${actualId}/scada-connections`).then(r => r.items),
    enabled,
  })
}

export function useSaveScadaEquipment() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ screenId, items }: { screenId: string; items: ScadaEquipmentItem[] }) => {
      const actualId = screenId === 'all' ? '__overview' : screenId
      await fetchOk(`/api/screens/${actualId}/scada-equipment`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ items }),
      })
    },
    onSuccess: (_, { screenId }) => {
      qc.invalidateQueries({ queryKey: queryKeys.scadaEquipment(screenId) })
    },
  })
}

export function useSaveScadaConnections() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ screenId, items }: { screenId: string; items: ScadaConnectionItem[] }) => {
      const actualId = screenId === 'all' ? '__overview' : screenId
      await fetchOk(`/api/screens/${actualId}/scada-connections`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ items }),
      })
    },
    onSuccess: (_, { screenId }) => {
      qc.invalidateQueries({ queryKey: queryKeys.scadaConnections(screenId) })
    },
  })
}
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: May show errors about unused imports but no type errors in the new code

- [ ] **Step 4: Commit**

```bash
git add client/src/types.ts client/src/queries.ts
git commit -m "feat(scada): add frontend types and query hooks for scada layout"
```

---

## Task 5: Frontend — SVG Equipment Assets

**Files:**
- Create: `client/src/scada/assets/SolarPanelSvg.tsx`
- Create: `client/src/scada/assets/InverterSvg.tsx`
- Create: `client/src/scada/assets/GatewaySvg.tsx`
- Create: `client/src/scada/assets/TransformerSvg.tsx`
- Create: `client/src/scada/assets/BatterySvg.tsx`
- Create: `client/src/scada/assets/MeterSvg.tsx`
- Create: `client/src/scada/assets/CircuitBreakerSvg.tsx`
- Create: `client/src/scada/assets/index.ts`
- Create: `client/src/scada/defaults.ts`

- [ ] **Step 1: Create the defaults module**

Create `client/src/scada/defaults.ts`:

```typescript
import type { ScadaEquipmentType } from '../types'

export const DEFAULT_SIZES: Record<ScadaEquipmentType, { width: number; height: number }> = {
  solar_panel: { width: 100, height: 60 },
  inverter: { width: 80, height: 60 },
  gateway: { width: 80, height: 70 },
  transformer: { width: 90, height: 80 },
  battery: { width: 70, height: 80 },
  meter: { width: 60, height: 60 },
  circuit_breaker: { width: 60, height: 70 },
}

export const PORT_POSITIONS: Record<ScadaEquipmentType, { x: number; y: number; side: 'top' | 'bottom' | 'left' | 'right' }[]> = {
  solar_panel: [
    { x: 0.2, y: 1, side: 'bottom' },
    { x: 0.8, y: 1, side: 'bottom' },
  ],
  inverter: [
    { x: 0, y: 0.3, side: 'left' },
    { x: 1, y: 0.3, side: 'right' },
  ],
  gateway: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  transformer: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  battery: [
    { x: 0.3, y: 0, side: 'top' },
    { x: 0.7, y: 0, side: 'top' },
  ],
  meter: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  circuit_breaker: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
}

export const CONNECTION_COLORS: Record<string, string> = {
  power: '#f59e0b',
  data: '#3b82f6',
  flow: '#22c55e',
}
```

- [ ] **Step 2: Create all SVG equipment components**

Create `client/src/scada/assets/SolarPanelSvg.tsx`:

```tsx
import React from 'react'

interface Props {
  statusColor?: string
}

export const SolarPanelSvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 100 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    {/* Panel body */}
    <rect x="5" y="5" width="90" height="40" rx="2" />
    {/* Diagonal cell line */}
    <line x1="5" y1="25" x2="95" y2="25" />
    <line x1="35" y1="5" x2="35" y2="45" />
    <line x1="65" y1="5" x2="65" y2="45" />
    {/* Mounting legs */}
    <line x1="25" y1="45" x2="20" y2="56" />
    <line x1="75" y1="45" x2="80" y2="56" />
    {/* Sun indicator */}
    <circle cx="50" cy="15" r="4" strokeDasharray="2 2" />
    {/* Terminals */}
    <circle cx="30" cy="57" r="3" fill={statusColor} stroke="none" />
    <text x="30" y="59" textAnchor="middle" fill="#fff" fontSize="5" stroke="none">+</text>
    <circle cx="70" cy="57" r="3" fill={statusColor} stroke="none" />
    <text x="70" y="59" textAnchor="middle" fill="#fff" fontSize="5" stroke="none">-</text>
  </svg>
)
```

Create `client/src/scada/assets/InverterSvg.tsx`:

```tsx
import React from 'react'

interface Props {
  statusColor?: string
}

export const InverterSvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 80 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    {/* Body */}
    <rect x="5" y="5" width="70" height="50" rx="3" />
    {/* DC label left */}
    <text x="15" y="22" fontSize="8" fill={statusColor} stroke="none" fontWeight="bold">DC</text>
    {/* Arrow through */}
    <line x1="28" y1="30" x2="52" y2="30" />
    <polygon points="52,26 60,30 52,34" fill={statusColor} stroke="none" />
    {/* AC label right */}
    <text x="62" y="22" fontSize="8" fill={statusColor} stroke="none" fontWeight="bold">AC</text>
    {/* Sine wave icon */}
    <path d="M 30 38 Q 35 33, 40 38 Q 45 43, 50 38" />
    {/* Status LED */}
    <circle cx="40" cy="48" r="3" fill={statusColor} stroke="none" opacity="0.7" />
    {/* Input/Output terminals */}
    <circle cx="2" cy="20" r="3" fill="none" stroke={statusColor} />
    <circle cx="78" cy="20" r="3" fill="none" stroke={statusColor} />
  </svg>
)
```

Create `client/src/scada/assets/GatewaySvg.tsx`:

```tsx
import React from 'react'

interface Props {
  statusColor?: string
}

export const GatewaySvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 80 70" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    {/* Body */}
    <rect x="5" y="15" width="70" height="50" rx="3" />
    {/* Screen area */}
    <rect x="15" y="22" width="50" height="20" rx="1" strokeDasharray="3 2" />
    {/* Antenna */}
    <line x1="40" y1="15" x2="40" y2="5" />
    {/* Radio waves */}
    <path d="M 32 8 Q 36 3, 40 8" />
    <path d="M 40 8 Q 44 3, 48 8" />
    {/* Ethernet port symbol */}
    <rect x="30" y="60" width="20" height="6" rx="1" />
    <line x1="34" y1="62" x2="34" y2="65" />
    <line x1="38" y1="62" x2="38" y2="65" />
    <line x1="42" y1="62" x2="42" y2="65" />
    <line x1="46" y1="62" x2="46" y2="65" />
    {/* Status LEDs */}
    <circle cx="15" cy="55" r="2.5" fill={statusColor} stroke="none" opacity="0.7" />
    <circle cx="22" cy="55" r="2.5" fill={statusColor} stroke="none" opacity="0.4" />
  </svg>
)
```

Create `client/src/scada/assets/TransformerSvg.tsx`:

```tsx
import React from 'react'

interface Props {
  statusColor?: string
}

export const TransformerSvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 90 80" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    {/* Primary coil */}
    <path d="M 25 15 Q 10 15, 10 25 Q 10 35, 25 35 Q 10 35, 10 45 Q 10 55, 25 55 Q 10 55, 10 65 Q 10 75, 25 75" />
    {/* Secondary coil */}
    <path d="M 65 15 Q 80 15, 80 25 Q 80 35, 65 35 Q 80 35, 80 45 Q 80 55, 65 55 Q 80 55, 80 65 Q 80 75, 65 75" />
    {/* Core lines */}
    <line x1="38" y1="10" x2="38" y2="80" strokeWidth="3" />
    <line x1="52" y1="10" x2="52" y2="80" strokeWidth="3" />
    {/* Primary terminal */}
    <circle cx="5" cy="15" r="3" fill="none" stroke={statusColor} />
    <circle cx="5" cy="75" r="3" fill="none" stroke={statusColor} />
    {/* Secondary terminal */}
    <circle cx="85" cy="15" r="3" fill="none" stroke={statusColor} />
    <circle cx="85" cy="75" r="3" fill="none" stroke={statusColor} />
    {/* Labels */}
    <text x="20" y="8" fontSize="7" fill={statusColor} stroke="none">P</text>
    <text x="65" y="8" fontSize="7" fill={statusColor} stroke="none">S</text>
  </svg>
)
```

Create `client/src/scada/assets/BatterySvg.tsx`:

```tsx
import React from 'react'

interface Props {
  statusColor?: string
}

export const BatterySvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 70 80" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    {/* Body */}
    <rect x="10" y="10" width="50" height="65" rx="3" />
    {/* Top terminal bar */}
    <rect x="25" y="5" width="20" height="5" rx="1" fill={statusColor} stroke="none" opacity="0.3" />
    {/* Level bars (charge indicator) */}
    <rect x="16" y="50" width="38" height="8" rx="1" fill={statusColor} stroke="none" opacity="0.2" />
    <rect x="16" y="38" width="38" height="8" rx="1" fill={statusColor} stroke="none" opacity="0.3" />
    <rect x="16" y="26" width="38" height="8" rx="1" fill={statusColor} stroke="none" opacity="0.4" />
    {/* +/- terminals */}
    <text x="25" y="3" textAnchor="middle" fontSize="6" fill={statusColor} stroke="none" fontWeight="bold">+</text>
    <text x="45" y="3" textAnchor="middle" fontSize="6" fill={statusColor} stroke="none" fontWeight="bold">-</text>
    {/* Connection points */}
    <circle cx="25" cy="2" r="2" fill="none" stroke={statusColor} />
    <circle cx="45" cy="2" r="2" fill="none" stroke={statusColor} />
  </svg>
)
```

Create `client/src/scada/assets/MeterSvg.tsx`:

```tsx
import React from 'react'

interface Props {
  statusColor?: string
}

export const MeterSvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 60 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    {/* Circle body */}
    <circle cx="30" cy="30" r="25" />
    {/* M label */}
    <text x="30" y="36" textAnchor="middle" fontSize="16" fill={statusColor} stroke="none" fontWeight="bold">M</text>
    {/* Measurement lines left */}
    <line x1="0" y1="30" x2="5" y2="30" />
    <circle cx="2" cy="30" r="2" fill={statusColor} stroke="none" />
    {/* Measurement lines right */}
    <line x1="55" y1="30" x2="60" y2="30" />
    <circle cx="58" cy="30" r="2" fill={statusColor} stroke="none" />
  </svg>
)
```

Create `client/src/scada/assets/CircuitBreakerSvg.tsx`:

```tsx
import React from 'react'

interface Props {
  statusColor?: string
}

export const CircuitBreakerSvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 60 70" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    {/* Body */}
    <rect x="10" y="5" width="40" height="60" rx="3" />
    {/* Toggle switch */}
    <line x1="30" y1="20" x2="30" y2="35" />
    <circle cx="30" cy="17" r="4" fill={statusColor} stroke="none" opacity="0.3" />
    {/* ON/OFF indicator */}
    <rect x="18" y="42" width="24" height="10" rx="2" strokeDasharray="3 2" />
    <text x="30" y="50" textAnchor="middle" fontSize="7" fill={statusColor} stroke="none">ON</text>
    {/* Connection terminals */}
    <circle cx="5" cy="35" r="3" fill="none" stroke={statusColor} />
    <line x1="8" y1="35" x2="10" y2="35" />
    <circle cx="55" cy="35" r="3" fill="none" stroke={statusColor} />
    <line x1="50" y1="35" x2="52" y2="35" />
    {/* Arc symbol */}
    <path d="M 22 30 Q 30 22, 38 30" strokeDasharray="2 2" />
  </svg>
)
```

- [ ] **Step 3: Create barrel export**

Create `client/src/scada/assets/index.ts`:

```typescript
export { SolarPanelSvg } from './SolarPanelSvg'
export { InverterSvg } from './InverterSvg'
export { GatewaySvg } from './GatewaySvg'
export { TransformerSvg } from './TransformerSvg'
export { BatterySvg } from './BatterySvg'
export { MeterSvg } from './MeterSvg'
export { CircuitBreakerSvg } from './CircuitBreakerSvg'

import { SolarPanelSvg } from './SolarPanelSvg'
import { InverterSvg } from './InverterSvg'
import { GatewaySvg } from './GatewaySvg'
import { TransformerSvg } from './TransformerSvg'
import { BatterySvg } from './BatterySvg'
import { MeterSvg } from './MeterSvg'
import { CircuitBreakerSvg } from './CircuitBreakerSvg'
import type { ScadaEquipmentType } from '../../types'
import type { FC } from 'react'

export const ASSET_MAP: Record<ScadaEquipmentType, FC<{ statusColor?: string }>> = {
  solar_panel: SolarPanelSvg,
  inverter: InverterSvg,
  gateway: GatewaySvg,
  transformer: TransformerSvg,
  battery: BatterySvg,
  meter: MeterSvg,
  circuit_breaker: CircuitBreakerSvg,
}
```

- [ ] **Step 4: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors in new files

- [ ] **Step 5: Commit**

```bash
git add client/src/scada/
git commit -m "feat(scada): add SVG equipment assets and defaults for P&ID symbols"
```

---

## Task 6: Frontend — ScadaCanvas and ScadaEquipment Components

**Files:**
- Create: `client/src/scada/ScadaCanvas.tsx`
- Create: `client/src/scada/ScadaEquipment.tsx`

- [ ] **Step 1: Create ScadaCanvas component**

Create `client/src/scada/ScadaCanvas.tsx`:

```tsx
import React, { useState, useCallback, useRef, useEffect } from 'react'
import type { ScadaEquipmentItem, ScadaConnectionItem, ScadaEquipmentType, SensorData } from '../types'
import { ScadaEquipment } from './ScadaEquipment'
import { ScadaConnection } from './ScadaConnection'

interface Props {
  equipment: ScadaEquipmentItem[]
  connections: ScadaConnectionItem[]
  currentValues: Record<string, SensorData>
  editMode: boolean
  selectedId: string | null
  activeTool: 'select' | 'connect' | 'delete'
  pendingType: ScadaEquipmentType | null
  onSelect: (id: string | null) => void
  onEquipmentMove: (id: string, x: number, y: number) => void
  onEquipmentResize: (id: string, w: number, h: number) => void
  onAddEquipment: (type: ScadaEquipmentType, x: number, y: number) => void
  onStartConnection: (eqId: string, portSide: string) => void
  onEndConnection: (eqId: string, portSide: string) => void
  onDeleteEquipment: (id: string) => void
}

export const ScadaCanvas: React.FC<Props> = ({
  equipment, connections, currentValues, editMode, selectedId, activeTool, pendingType,
  onSelect, onEquipmentMove, onEquipmentResize, onAddEquipment, onStartConnection, onEndConnection, onDeleteEquipment,
}) => {
  const [viewBox, setViewBox] = useState({ x: 0, y: 0, w: 1200, h: 800 })
  const [isPanning, setIsPanning] = useState(false)
  const [panStart, setPanStart] = useState({ x: 0, y: 0 })
  const svgRef = useRef<SVGSVGElement>(null)

  const zoom = useCallback((delta: number, cx: number, cy: number) => {
    setViewBox(prev => {
      const scale = delta > 0 ? 0.9 : 1.1
      const newW = Math.max(150, Math.min(4800, prev.w * scale))
      const newH = Math.max(100, Math.min(3200, prev.h * scale))
      const dx = (newW - prev.w) * (cx / (svgRef.current?.clientWidth || 800))
      const dy = (newH - prev.h) * (cy / (svgRef.current?.clientHeight || 600))
      return { x: prev.x - dx, y: prev.y - dy, w: newW, h: newH }
    })
  }, [])

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault()
    const rect = svgRef.current?.getBoundingClientRect()
    if (!rect) return
    const cx = e.clientX - rect.left
    const cy = e.clientY - rect.top
    zoom(e.deltaY, cx, cy)
  }, [zoom])

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.target === svgRef.current || (e.target as Element).tagName === 'rect' && (e.target as Element).classList.contains('canvas-bg')) {
      if (pendingType) {
        const rect = svgRef.current?.getBoundingClientRect()
        if (rect) {
          const svgX = viewBox.x + (e.clientX - rect.left) / rect.width * viewBox.w
          const svgY = viewBox.y + (e.clientY - rect.top) / rect.height * viewBox.h
          onAddEquipment(pendingType, svgX, svgY)
        }
        return
      }
      if (activeTool === 'select' || !editMode) {
        onSelect(null)
        setIsPanning(true)
        setPanStart({ x: e.clientX, y: e.clientY })
      }
    }
  }, [editMode, activeTool, pendingType, viewBox, onAddEquipment, onSelect])

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (isPanning) {
      const rect = svgRef.current?.getBoundingClientRect()
      if (!rect) return
      const dx = (e.clientX - panStart.x) / rect.width * viewBox.w
      const dy = (e.clientY - panStart.y) / rect.height * viewBox.h
      setViewBox(prev => ({ ...prev, x: prev.x - dx, y: prev.y - dy }))
      setPanStart({ x: e.clientX, y: e.clientY })
    }
  }, [isPanning, panStart, viewBox])

  const handleMouseUp = useCallback(() => {
    setIsPanning(false)
  }, [])

  // Handle keyboard delete
  useEffect(() => {
    if (!editMode) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Delete' && selectedId) {
        onDeleteEquipment(selectedId)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [editMode, selectedId, onDeleteEquipment])

  const zoomIn = () => setViewBox(prev => {
    const newW = prev.w * 0.8
    const newH = prev.h * 0.8
    return { x: prev.x + (prev.w - newW) / 2, y: prev.y + (prev.h - newH) / 2, w: newW, h: newH }
  })
  const zoomOut = () => setViewBox(prev => {
    const newW = prev.w * 1.25
    const newH = prev.h * 1.25
    return { x: prev.x + (prev.w - newW) / 2, y: prev.y + (prev.h - newH) / 2, w: newW, h: newH }
  })
  const fitView = () => {
    if (equipment.length === 0) { setViewBox({ x: 0, y: 0, w: 1200, h: 800 }); return }
    const minX = Math.min(...equipment.map(e => e.x)) - 50
    const minY = Math.min(...equipment.map(e => e.y)) - 50
    const maxX = Math.max(...equipment.map(e => e.x + e.width)) + 50
    const maxY = Math.max(...equipment.map(e => e.y + e.height)) + 50
    setViewBox({ x: minX, y: minY, w: maxX - minX, h: maxY - minY })
  }

  return (
    <div style={{ position: 'relative', width: '100%', height: 'calc(100vh - 180px)', background: '#fafafa', border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden' }}>
      {/* Zoom controls */}
      <div style={{ position: 'absolute', top: 8, right: 8, zIndex: 10, display: 'flex', flexDirection: 'column', gap: 4 }}>
        <button onClick={zoomIn} style={{ width: 28, height: 28, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 4, cursor: 'pointer', fontSize: 16, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>+</button>
        <button onClick={zoomOut} style={{ width: 28, height: 28, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 4, cursor: 'pointer', fontSize: 16, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>-</button>
        <button onClick={fitView} style={{ width: 28, height: 28, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 4, cursor: 'pointer', fontSize: 10, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>Fit</button>
      </div>
      <svg
        ref={svgRef}
        style={{ width: '100%', height: '100%', cursor: isPanning ? 'grabbing' : editMode && pendingType ? 'crosshair' : 'grab' }}
        viewBox={`${viewBox.x} ${viewBox.y} ${viewBox.w} ${viewBox.h}`}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        {/* Background grid */}
        <defs>
          <pattern id="scada-grid" width="20" height="20" patternUnits="userSpaceOnUse">
            <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#e5e7eb" strokeWidth="0.5" />
          </pattern>
        </defs>
        <rect className="canvas-bg" x={viewBox.x - 100} y={viewBox.y - 100} width={viewBox.w + 200} height={viewBox.h + 200} fill="url(#scada-grid)" />

        {/* Connections */}
        {connections.map(conn => (
          <ScadaConnection
            key={conn.connection_id}
            connection={conn}
            equipment={equipment}
            selected={selectedId === conn.connection_id}
            editMode={editMode}
            onSelect={() => onSelect(conn.connection_id)}
          />
        ))}

        {/* Equipment */}
        {equipment.map(eq => (
          <ScadaEquipment
            key={eq.equipment_id}
            equipment={eq}
            currentValues={currentValues}
            editMode={editMode}
            selected={selectedId === eq.equipment_id}
            activeTool={activeTool}
            onSelect={() => onSelect(eq.equipment_id)}
            onMove={(x, y) => onEquipmentMove(eq.equipment_id, x, y)}
            onResize={(w, h) => onEquipmentResize(eq.equipment_id, w, h)}
            onStartConnection={(side) => onStartConnection(eq.equipment_id, side)}
            onEndConnection={(side) => onEndConnection(eq.equipment_id, side)}
          />
        ))}
      </svg>
    </div>
  )
}
```

- [ ] **Step 2: Create ScadaEquipment component**

Create `client/src/scada/ScadaEquipment.tsx`:

```tsx
import React, { useCallback, useRef } from 'react'
import type { ScadaEquipmentItem, SensorData } from '../types'
import { ASSET_MAP } from './assets'
import { PORT_POSITIONS } from './defaults'

interface Props {
  equipment: ScadaEquipmentItem
  currentValues: Record<string, SensorData>
  editMode: boolean
  selected: boolean
  activeTool: string
  onSelect: () => void
  onMove: (x: number, y: number) => void
  onResize: (w: number, h: number) => void
  onStartConnection: (portSide: string) => void
  onEndConnection: (portSide: string) => void
}

function getStatusColor(eq: ScadaEquipmentItem, currentValues: Record<string, SensorData>): string {
  const bindings: string[] = JSON.parse(eq.sensor_bindings || '[]')
  let worst = 'normal'
  for (const id of bindings) {
    const val = currentValues[String(id)]
    if (!val) continue
    if (val.crit > 0 && val.value >= val.crit) { worst = 'critical'; break }
    if (val.warn > 0 && val.value >= val.warn) { worst = 'warning' }
  }
  if (worst === 'critical') return '#ef4444'
  if (worst === 'warning') return '#eab308'
  return '#333'
}

export const ScadaEquipment: React.FC<Props> = ({ equipment, currentValues, editMode, selected, activeTool, onSelect, onMove, onResize, onStartConnection, onEndConnection }) => {
  const dragRef = useRef<{ startX: number; startY: number; origX: number; origY: number } | null>(null)
  const resizeRef = useRef<{ startX: number; origW: number; origH: number } | null>(null)
  const statusColor = getStatusColor(equipment, currentValues)
  const AssetComponent = ASSET_MAP[equipment.equipment_type]
  const ports = PORT_POSITIONS[equipment.equipment_type] || []
  const bindings: string[] = JSON.parse(equipment.sensor_bindings || '[]')

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!editMode || activeTool !== 'select') return
    e.stopPropagation()
    onSelect()
    dragRef.current = { startX: e.clientX, startY: e.clientY, origX: equipment.x, origY: equipment.y }

    const onMoveHandler = (ev: MouseEvent) => {
      if (!dragRef.current) return
      // Convert pixel delta to SVG coordinate delta — approximate using a fixed scale
      const scale = 1 // 1:1 since we control viewBox
      const dx = (ev.clientX - dragRef.current.startX) * scale
      const dy = (ev.clientY - dragRef.current.startY) * scale
      // Snap to 10px grid
      const newX = Math.round((dragRef.current.origX + dx) / 10) * 10
      const newY = Math.round((dragRef.current.origY + dy) / 10) * 10
      onMove(newX, newY)
    }
    const onUp = () => {
      dragRef.current = null
      window.removeEventListener('mousemove', onMoveHandler)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMoveHandler)
    window.addEventListener('mouseup', onUp)
  }, [editMode, activeTool, equipment.x, equipment.y, onSelect, onMove])

  const handleResizeMouseDown = useCallback((e: React.MouseEvent) => {
    e.stopPropagation()
    resizeRef.current = { startX: e.clientX, origW: equipment.width, origH: equipment.height }
    const onResizeHandler = (ev: MouseEvent) => {
      if (!resizeRef.current) return
      const dx = ev.clientX - resizeRef.current.startX
      const dy = ev.clientY - resizeRef.current.startY
      onResize(
        Math.max(40, Math.round((resizeRef.current.origW + dx) / 10) * 10),
        Math.max(30, Math.round((resizeRef.current.origH + dy) / 10) * 10),
      )
    }
    const onUp = () => {
      resizeRef.current = null
      window.removeEventListener('mousemove', onResizeHandler)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onResizeHandler)
    window.addEventListener('mouseup', onUp)
  }, [equipment.width, equipment.height, onResize])

  const handlePortClick = useCallback((e: React.MouseEvent, side: string) => {
    if (!editMode || activeTool !== 'connect') return
    e.stopPropagation()
    onStartConnection(side)
  }, [editMode, activeTool, onStartConnection])

  // Data label for bound sensors
  const dataLabel = bindings.map(id => {
    const val = currentValues[String(id)]
    if (!val) return null
    return `${val.value.toFixed(1)}`
  }).filter(Boolean).join(' | ')

  return (
    <g
      transform={`translate(${equipment.x}, ${equipment.y})${equipment.rotation ? ` rotate(${equipment.rotation}, ${equipment.width/2}, ${equipment.height/2})` : ''}`}
      onMouseDown={handleMouseDown}
      style={{ cursor: editMode ? 'move' : 'pointer' }}
    >
      {/* Selection highlight */}
      {selected && (
        <rect x="-4" y="-4" width={equipment.width + 8} height={equipment.height + 8} rx="4" fill="none" stroke="var(--cyan)" strokeWidth="2" strokeDasharray="6 3" />
      )}

      {/* Equipment SVG asset */}
      <foreignObject x="0" y="0" width={equipment.width} height={equipment.height}>
        {AssetComponent && <AssetComponent statusColor={statusColor} />}
      </foreignObject>

      {/* Equipment name label */}
      <text x={equipment.width / 2} y={equipment.height + 14} textAnchor="middle" fontSize="10" fill="#555" style={{ userSelect: 'none' }}>
        {equipment.equipment_name || equipment.equipment_type.replace(/_/g, ' ')}
      </text>

      {/* Real-time data label */}
      {dataLabel && (
        <g>
          <rect x={equipment.width - 8} y="-14" width={Math.max(dataLabel.length * 6 + 8, 40)} height="16" rx="3" fill="rgba(255,255,255,0.92)" stroke="#ddd" strokeWidth="0.5" />
          <text x={equipment.width - 4} y="-2" fontSize="9" fill={statusColor} style={{ userSelect: 'none' }}>
            {dataLabel}
          </text>
        </g>
      )}

      {/* Connection ports (visible in edit mode with connect tool) */}
      {editMode && activeTool === 'connect' && ports.map((port, i) => {
        const px = port.x * equipment.width
        const py = port.y * equipment.height
        return (
          <circle
            key={i}
            cx={px}
            cy={py}
            r="5"
            fill="var(--cyan)"
            stroke="#fff"
            strokeWidth="1.5"
            style={{ cursor: 'crosshair' }}
            onMouseDown={(e) => handlePortClick(e, port.side)}
          />
        )
      })}

      {/* Resize handle (edit mode, selected) */}
      {editMode && selected && (
        <rect
          x={equipment.width - 6}
          y={equipment.height - 6}
          width="8"
          height="8"
          rx="2"
          fill="var(--cyan)"
          stroke="#fff"
          strokeWidth="1"
          style={{ cursor: 'nwse-resize' }}
          onMouseDown={handleResizeMouseDown}
        />
      )}
    </g>
  )
}
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -30`
Expected: No errors in new files (may show warnings about unused vars in other files)

- [ ] **Step 4: Commit**

```bash
git add client/src/scada/ScadaCanvas.tsx client/src/scada/ScadaEquipment.tsx
git commit -m "feat(scada): add ScadaCanvas with pan/zoom and ScadaEquipment components"
```

---

## Task 7: Frontend — ScadaConnection Component

**Files:**
- Create: `client/src/scada/ScadaConnection.tsx`

- [ ] **Step 1: Create ScadaConnection component**

Create `client/src/scada/ScadaConnection.tsx`:

```tsx
import React from 'react'
import type { ScadaEquipmentItem, ScadaConnectionItem } from '../types'
import { CONNECTION_COLORS } from './defaults'

interface Props {
  connection: ScadaConnectionItem
  equipment: ScadaEquipmentItem[]
  selected: boolean
  editMode: boolean
  onSelect: () => void
}

function getPortPosition(eq: ScadaEquipmentItem, side: string): { x: number; y: number } {
  switch (side) {
    case 'top': return { x: eq.x + eq.width / 2, y: eq.y }
    case 'bottom': return { x: eq.x + eq.width / 2, y: eq.y + eq.height }
    case 'left': return { x: eq.x, y: eq.y + eq.height / 2 }
    case 'right': return { x: eq.x + eq.width, y: eq.y + eq.height / 2 }
    default: return { x: eq.x + eq.width / 2, y: eq.y + eq.height / 2 }
  }
}

function buildPath(from: { x: number; y: number }, to: { x: number; y: number }, waypoints: [number, number][]): string {
  const points = [from, ...waypoints, to]
  if (points.length <= 2) {
    // Manhattan routing: go horizontal then vertical (or vice versa based on direction)
    const midX = from.x + (to.x - from.x) * 0.5
    return `M ${from.x} ${from.y} L ${midX} ${from.y} L ${midX} ${to.y} L ${to.x} ${to.y}`
  }
  return points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p[0]} ${p[1]}`).join(' ')
}

export const ScadaConnection: React.FC<Props> = ({ connection, equipment, selected, editMode, onSelect }) => {
  const fromEq = equipment.find(e => e.equipment_id === connection.from_equipment)
  const toEq = equipment.find(e => e.equipment_id === connection.to_equipment)
  if (!fromEq || !toEq) return null

  // Determine best port sides based on relative positions
  const dx = toEq.x - fromEq.x
  const dy = toEq.y - fromEq.y
  const fromSide = Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 'right' : 'left') : (dy > 0 ? 'bottom' : 'top')
  const toSide = Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 'left' : 'right') : (dy > 0 ? 'top' : 'bottom')

  const from = getPortPosition(fromEq, fromSide)
  const to = getPortPosition(toEq, toSide)
  const waypoints: [number, number][] = JSON.parse(connection.waypoints || '[]')
  const pathD = buildPath(from, to, waypoints)
  const color = CONNECTION_COLORS[connection.connection_type] || CONNECTION_COLORS.power
  const isDashed = connection.connection_type === 'data'
  const strokeWidth = connection.connection_type === 'power' ? 3 : 2

  return (
    <g onClick={editMode ? onSelect : undefined} style={{ cursor: editMode ? 'pointer' : 'default' }}>
      {/* Invisible wider path for easier clicking */}
      <path d={pathD} fill="none" stroke="transparent" strokeWidth="12" />
      {/* Visible connection line */}
      <path
        d={pathD}
        fill="none"
        stroke={selected ? 'var(--cyan)' : color}
        strokeWidth={strokeWidth}
        strokeDasharray={isDashed ? '8 4' : undefined}
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        {connection.connection_type !== 'data' && (
          <animate
            attributeName="stroke-dashoffset"
            from={isDashed ? '24' : '0'}
            to={isDashed ? '0' : '0'}
            dur="1s"
            repeatCount="indefinite"
          />
        )}
      </path>
      {/* Flow direction arrow at midpoint */}
      {(() => {
        const midX = (from.x + to.x) / 2
        const midY = (from.y + to.y) / 2
        const angle = Math.atan2(to.y - from.y, to.x - from.x) * 180 / Math.PI
        return (
          <polygon
            points="-4,-3 4,0 -4,3"
            transform={`translate(${midX}, ${midY}) rotate(${angle})`}
            fill={selected ? 'var(--cyan)' : color}
          />
        )
      })()}
    </g>
  )
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add client/src/scada/ScadaConnection.tsx
git commit -m "feat(scada): add ScadaConnection with Manhattan routing and flow animation"
```

---

## Task 8: Frontend — ScadaEditToolbar and ScadaPalette

**Files:**
- Create: `client/src/scada/ScadaEditToolbar.tsx`
- Create: `client/src/scada/ScadaPalette.tsx`

- [ ] **Step 1: Create ScadaEditToolbar component**

Create `client/src/scada/ScadaEditToolbar.tsx`:

```tsx
import React from 'react'

type Tool = 'select' | 'connect' | 'delete'

interface Props {
  editMode: boolean
  activeTool: Tool
  onToggleEdit: () => void
  onToolChange: (tool: Tool) => void
  onSave: () => void
  hasChanges: boolean
}

const toolBtnStyle = (active: boolean): React.CSSProperties => ({
  padding: '6px 12px',
  borderRadius: 4,
  border: `1px solid ${active ? 'var(--cyan)' : 'var(--border)'}`,
  background: active ? 'rgba(34,197,94,.1)' : 'var(--card)',
  color: active ? 'var(--cyan)' : 'var(--fg)',
  cursor: 'pointer',
  fontSize: 12,
  fontWeight: 500,
})

export const ScadaEditToolbar: React.FC<Props> = ({ editMode, activeTool, onToggleEdit, onToolChange, onSave, hasChanges }) => {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
      <button
        onClick={onToggleEdit}
        style={{
          ...toolBtnStyle(editMode),
          borderColor: editMode ? 'var(--cyan)' : 'var(--border)',
        }}
      >
        {editMode ? '✓ Done' : '✎ Edit'}
      </button>

      {editMode && (
        <>
          <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
          <button style={toolBtnStyle(activeTool === 'select')} onClick={() => onToolChange('select')}>
            ↖ Select
          </button>
          <button style={toolBtnStyle(activeTool === 'connect')} onClick={() => onToolChange('connect')}>
            ⟷ Connect
          </button>
          <button style={toolBtnStyle(activeTool === 'delete')} onClick={() => onToolChange('delete')}>
            ✕ Delete
          </button>
          <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
          <button
            onClick={onSave}
            disabled={!hasChanges}
            style={{
              padding: '6px 14px',
              borderRadius: 4,
              border: `1px solid ${hasChanges ? 'var(--cyan)' : 'var(--border)'}`,
              background: hasChanges ? 'var(--cyan)' : 'var(--card)',
              color: hasChanges ? '#fff' : 'var(--muted)',
              cursor: hasChanges ? 'pointer' : 'not-allowed',
              fontSize: 12,
              fontWeight: 600,
            }}
          >
            Save Layout
          </button>
        </>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Create ScadaPalette component**

Create `client/src/scada/ScadaPalette.tsx`:

```tsx
import React from 'react'
import { SCADA_EQUIPMENT_TYPES, type ScadaEquipmentType } from '../types'
import { ASSET_MAP } from './assets'

interface Props {
  visible: boolean
  onSelect: (type: ScadaEquipmentType) => void
  selectedType: ScadaEquipmentType | null
}

export const ScadaPalette: React.FC<Props> = ({ visible, onSelect, selectedType }) => {
  if (!visible) return null

  return (
    <div style={{
      position: 'absolute', top: 8, left: 8, zIndex: 10,
      background: 'var(--card)', border: '1px solid var(--border)',
      borderRadius: 8, padding: 8, display: 'flex', gap: 6, flexWrap: 'wrap',
      maxWidth: 320, boxShadow: '0 2px 8px rgba(0,0,0,.08)',
    }}>
      {SCADA_EQUIPMENT_TYPES.map(({ value, label }) => {
        const SvgIcon = ASSET_MAP[value]
        return (
          <div
            key={value}
            onClick={() => onSelect(value)}
            style={{
              display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4,
              padding: '6px 8px', borderRadius: 6, cursor: 'pointer', minWidth: 56,
              border: `1px solid ${selectedType === value ? 'var(--cyan)' : 'transparent'}`,
              background: selectedType === value ? 'rgba(34,197,94,.1)' : 'transparent',
            }}
          >
            <div style={{ width: 32, height: 24 }}>
              {SvgIcon && <SvgIcon statusColor={selectedType === value ? 'var(--cyan)' : '#333'} />}
            </div>
            <span style={{ fontSize: 9, color: selectedType === value ? 'var(--cyan)' : 'var(--muted)', textAlign: 'center', lineHeight: 1.2 }}>
              {label}
            </span>
          </div>
        )
      })}
    </div>
  )
}
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add client/src/scada/ScadaEditToolbar.tsx client/src/scada/ScadaPalette.tsx
git commit -m "feat(scada): add edit toolbar and equipment palette components"
```

---

## Task 9: Frontend — ScadaEditPanel

**Files:**
- Create: `client/src/scada/ScadaEditPanel.tsx`

- [ ] **Step 1: Create ScadaEditPanel component**

Create `client/src/scada/ScadaEditPanel.tsx`:

```tsx
import React, { useState } from 'react'
import type { ScadaEquipmentItem, SensorConfig } from '../types'

interface Props {
  equipment: ScadaEquipmentItem | null
  sensors: SensorConfig[]
  onUpdate: (updates: Partial<ScadaEquipmentItem>) => void
  onClose: () => void
}

export const ScadaEditPanel: React.FC<Props> = ({ equipment, sensors, onUpdate, onClose }) => {
  const [bindingSearch, setBindingSearch] = useState('')

  if (!equipment) return null

  const currentBindings: string[] = JSON.parse(equipment.sensor_bindings || '[]')
  const filteredSensors = bindingSearch
    ? sensors.filter(s => s.name.toLowerCase().includes(bindingSearch.toLowerCase()) || s.key.toLowerCase().includes(bindingSearch.toLowerCase()))
    : sensors

  const toggleBinding = (sensorId: number) => {
    const id = String(sensorId)
    const next = currentBindings.includes(id)
      ? currentBindings.filter(b => b !== id)
      : [...currentBindings, id]
    onUpdate({ sensor_bindings: JSON.stringify(next) })
  }

  return (
    <div style={{
      position: 'absolute', top: 0, right: 0, bottom: 0, width: 280,
      background: 'var(--card)', borderLeft: '1px solid var(--border)',
      padding: 16, overflowY: 'auto', zIndex: 20,
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <span style={{ fontWeight: 600, fontSize: 14 }}>Properties</span>
        <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18 }}>x</button>
      </div>

      {/* Name */}
      <label style={{ display: 'block', marginBottom: 12 }}>
        <span style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em' }}>Name</span>
        <input
          value={equipment.equipment_name}
          onChange={e => onUpdate({ equipment_name: e.target.value })}
          style={{ width: '100%', padding: '6px 8px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', marginTop: 4, fontSize: 13, boxSizing: 'border-box' }}
        />
      </label>

      {/* Type */}
      <div style={{ marginBottom: 12 }}>
        <span style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em' }}>Type</span>
        <div style={{ marginTop: 4, fontSize: 13, textTransform: 'capitalize' }}>{equipment.equipment_type.replace(/_/g, ' ')}</div>
      </div>

      {/* Position */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 12 }}>
        <label>
          <span style={{ fontSize: 11, color: 'var(--muted)' }}>X</span>
          <input type="number" value={Math.round(equipment.x)} onChange={e => onUpdate({ x: +e.target.value })}
            style={{ width: '100%', padding: '4px 6px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', fontSize: 12, boxSizing: 'border-box' }} />
        </label>
        <label>
          <span style={{ fontSize: 11, color: 'var(--muted)' }}>Y</span>
          <input type="number" value={Math.round(equipment.y)} onChange={e => onUpdate({ y: +e.target.value })}
            style={{ width: '100%', padding: '4px 6px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', fontSize: 12, boxSizing: 'border-box' }} />
        </label>
      </div>

      {/* Size */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 16 }}>
        <label>
          <span style={{ fontSize: 11, color: 'var(--muted)' }}>Width</span>
          <input type="number" value={Math.round(equipment.width)} onChange={e => onUpdate({ width: Math.max(40, +e.target.value) })}
            style={{ width: '100%', padding: '4px 6px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', fontSize: 12, boxSizing: 'border-box' }} />
        </label>
        <label>
          <span style={{ fontSize: 11, color: 'var(--muted)' }}>Height</span>
          <input type="number" value={Math.round(equipment.height)} onChange={e => onUpdate({ height: Math.max(30, +e.target.value) })}
            style={{ width: '100%', padding: '4px 6px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', fontSize: 12, boxSizing: 'border-box' }} />
        </label>
      </div>

      {/* Rotation */}
      <label style={{ display: 'block', marginBottom: 16 }}>
        <span style={{ fontSize: 11, color: 'var(--muted)' }}>Rotation: {equipment.rotation}deg</span>
        <input type="range" min="0" max="360" step="15" value={equipment.rotation}
          onChange={e => onUpdate({ rotation: +e.target.value })}
          style={{ width: '100%', marginTop: 4 }} />
      </label>

      {/* Sensor bindings */}
      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12 }}>
        <span style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em' }}>Sensor Bindings ({currentBindings.length})</span>
        <input
          placeholder="Search sensors..."
          value={bindingSearch}
          onChange={e => setBindingSearch(e.target.value)}
          style={{ width: '100%', padding: '6px 8px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', marginTop: 8, fontSize: 12, boxSizing: 'border-box' }}
        />
        <div style={{ maxHeight: 250, overflowY: 'auto', marginTop: 8 }}>
          {filteredSensors.map(s => {
            const bound = currentBindings.includes(String(s.id))
            return (
              <div key={s.id} onClick={() => toggleBinding(s.id)}
                style={{
                  padding: '4px 6px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                  display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                  background: bound ? 'rgba(34,197,94,.1)' : 'transparent',
                  marginBottom: 2,
                }}>
                <span style={{ color: bound ? 'var(--cyan)' : 'var(--fg)' }}>{s.name}</span>
                <span style={{ fontSize: 10, color: 'var(--muted)' }}>{s.unit}</span>
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add client/src/scada/ScadaEditPanel.tsx
git commit -m "feat(scada): add ScadaEditPanel for equipment properties and sensor binding"
```

---

## Task 10: Frontend — ScadaView Main Component

**Files:**
- Create: `client/src/ScadaView.tsx`

- [ ] **Step 1: Create ScadaView orchestrator component**

Create `client/src/ScadaView.tsx`:

```tsx
import React, { useState, useCallback, useRef, useEffect } from 'react'
import type { ScadaEquipmentItem, ScadaConnectionItem, ScadaEquipmentType, SensorConfig, SensorData } from './types'
import { DEFAULT_SIZES } from './scada/defaults'
import { ScadaCanvas } from './scada/ScadaCanvas'
import { ScadaEditToolbar } from './scada/ScadaEditToolbar'
import { ScadaPalette } from './scada/ScadaPalette'
import { ScadaEditPanel } from './scada/ScadaEditPanel'

interface Props {
  sensors: SensorConfig[]
  currentValues: Record<string, SensorData>
  activeScreen: string
  scadaEquipment: ScadaEquipmentItem[]
  scadaConnections: ScadaConnectionItem[]
  onSaveEquipment: (items: ScadaEquipmentItem[]) => void
  onSaveConnections: (items: ScadaConnectionItem[]) => void
}

export const ScadaView: React.FC<Props> = ({
  sensors, currentValues, activeScreen, scadaEquipment: serverEquipment, scadaConnections: serverConnections,
  onSaveEquipment, onSaveConnections,
}) => {
  const [editMode, setEditMode] = useState(false)
  const [activeTool, setActiveTool] = useState<'select' | 'connect' | 'delete'>('select')
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [equipment, setEquipment] = useState<ScadaEquipmentItem[]>([])
  const [connections, setConnections] = useState<ScadaConnectionItem[]>([])
  const [pendingType, setPendingType] = useState<ScadaEquipmentType | null>(null)
  const [connectFrom, setConnectFrom] = useState<{ eqId: string; side: string } | null>(null)
  const dirtyRef = useRef(false)

  // Sync from server data
  useEffect(() => { setEquipment(serverEquipment) }, [serverEquipment])
  useEffect(() => { setConnections(serverConnections) }, [serverConnections])

  const hasChanges = dirtyRef.current

  const handleEquipmentMove = useCallback((id: string, x: number, y: number) => {
    setEquipment(prev => prev.map(e => e.equipment_id === id ? { ...e, x, y } : e))
    dirtyRef.current = true
  }, [])

  const handleEquipmentResize = useCallback((id: string, w: number, h: number) => {
    setEquipment(prev => prev.map(e => e.equipment_id === id ? { ...e, width: w, height: h } : e))
    dirtyRef.current = true
  }, [])

  const handleAddEquipment = useCallback((type: ScadaEquipmentType, x: number, y: number) => {
    const defaults = DEFAULT_SIZES[type]
    const newEq: ScadaEquipmentItem = {
      equipment_id: `scada_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
      equipment_type: type,
      equipment_name: type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
      x: Math.round(x - defaults.width / 2),
      y: Math.round(y - defaults.height / 2),
      width: defaults.width,
      height: defaults.height,
      rotation: 0,
      sensor_bindings: '[]',
    }
    setEquipment(prev => [...prev, newEq])
    setPendingType(null)
    setActiveTool('select')
    dirtyRef.current = true
  }, [])

  const handleDeleteEquipment = useCallback((id: string) => {
    setEquipment(prev => prev.filter(e => e.equipment_id !== id))
    setConnections(prev => prev.filter(c => c.from_equipment !== id && c.to_equipment !== id))
    setSelectedId(null)
    dirtyRef.current = true
  }, [])

  const handleStartConnection = useCallback((eqId: string, side: string) => {
    if (connectFrom) {
      // Complete connection
      if (connectFrom.eqId !== eqId) {
        const newConn: ScadaConnectionItem = {
          connection_id: `conn_${Date.now()}`,
          from_equipment: connectFrom.eqId,
          to_equipment: eqId,
          waypoints: '[]',
          connection_type: 'power',
        }
        setConnections(prev => [...prev, newConn])
        dirtyRef.current = true
      }
      setConnectFrom(null)
    } else {
      setConnectFrom({ eqId, side })
    }
  }, [connectFrom])

  const handleEndConnection = useCallback((_eqId: string, _side: string) => {
    // Handled by handleStartConnection logic (toggle-style)
  }, [])

  const handleSave = useCallback(() => {
    // Attach screen_id for the API
    onSaveEquipment(equipment)
    onSaveConnections(connections)
    dirtyRef.current = false
  }, [equipment, connections, onSaveEquipment, onSaveConnections])

  const handleToggleEdit = useCallback(() => {
    if (editMode) {
      setActiveTool('select')
      setPendingType(null)
      setConnectFrom(null)
    }
    setEditMode(prev => !prev)
    setSelectedId(null)
  }, [editMode])

  const selectedEquipment = equipment.find(e => e.equipment_id === selectedId) || null

  const handleEditPanelUpdate = useCallback((updates: Partial<ScadaEquipmentItem>) => {
    if (!selectedEquipment) return
    setEquipment(prev => prev.map(e =>
      e.equipment_id === selectedEquipment.equipment_id ? { ...e, ...updates } : e
    ))
    dirtyRef.current = true
  }, [selectedEquipment])

  return (
    <div style={{ position: 'relative' }}>
      <ScadaEditToolbar
        editMode={editMode}
        activeTool={activeTool}
        onToggleEdit={handleToggleEdit}
        onToolChange={setActiveTool}
        onSave={handleSave}
        hasChanges={hasChanges}
      />

      <div style={{ position: 'relative' }}>
        <ScadaPalette
          visible={editMode && activeTool === 'select'}
          onSelect={setPendingType}
          selectedType={pendingType}
        />

        <ScadaCanvas
          equipment={equipment}
          connections={connections}
          currentValues={currentValues}
          editMode={editMode}
          selectedId={selectedId}
          activeTool={activeTool}
          pendingType={pendingType}
          onSelect={setSelectedId}
          onEquipmentMove={handleEquipmentMove}
          onEquipmentResize={handleEquipmentResize}
          onAddEquipment={handleAddEquipment}
          onStartConnection={handleStartConnection}
          onEndConnection={handleEndConnection}
          onDeleteEquipment={handleDeleteEquipment}
        />

        {editMode && selectedEquipment && (
          <ScadaEditPanel
            equipment={selectedEquipment}
            sensors={sensors}
            onUpdate={handleEditPanelUpdate}
            onClose={() => setSelectedId(null)}
          />
        )}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add client/src/ScadaView.tsx
git commit -m "feat(scada): add ScadaView main orchestrator component"
```

---

## Task 11: Frontend — App.tsx Integration

**Files:**
- Modify: `client/src/App.tsx`

- [ ] **Step 1: Update viewMode type to include 'scada'**

In `client/src/App.tsx`, change line 55:

From:
```typescript
const [viewMode, setViewMode] = useState<'cards' | 'canvas' | 'twin'>('cards');
```

To:
```typescript
const [viewMode, setViewMode] = useState<'cards' | 'canvas' | 'twin' | 'scada'>('cards');
```

- [ ] **Step 2: Import ScadaView component and query hooks**

After line 20 (the long import from queries), add `useScadaEquipment`, `useScadaConnections`, `useSaveScadaEquipment`, `useSaveScadaConnections` to the import.

After the DigitalTwinView lazy import (line 21), add:

```typescript
const ScadaView = React.lazy(() => import('./ScadaView').then(m => ({ default: m.ScadaView })))
```

Note: import `React` at the top — check if it's already imported. It should be from line 1.

Actually, ScadaView is a named export, so use:

```typescript
const ScadaView = lazy(() => import('./ScadaView').then(m => ({ default: m.ScadaView })))
```

This is the same pattern as DigitalTwinView.

- [ ] **Step 3: Add query hooks for scada data**

After the twinConnections query (around line 84), add:

```typescript
  const { data: scadaEquipment = [] } = useScadaEquipment(activeScreen, authed && viewMode === 'scada')
  const { data: scadaConnections = [] } = useScadaConnections(activeScreen, authed && viewMode === 'scada')
```

Add mutation hooks after the existing mutations (around line 103):

```typescript
  const saveScadaEquipmentMut = useSaveScadaEquipment()
  const saveScadaConnectionsMut = useSaveScadaConnections()
```

- [ ] **Step 4: Add '2D' button to view mode toggle**

In the view mode toggle `<div>` (around line 421-425), add a new button after the 3D button:

```tsx
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'scada' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'scada' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setViewMode('scada')}>2D</button>
```

- [ ] **Step 5: Update the Live/History toggle visibility**

Change the condition on line 427 from `viewMode !== 'twin'` to `viewMode !== 'twin' && viewMode !== 'scada'` since SCADA view also doesn't need history mode:

```tsx
          {viewMode !== 'twin' && viewMode !== 'scada' && (<>
```

- [ ] **Step 6: Add ScadaView rendering**

In the main content area (around line 578-656), add a new condition. After the canvas view block and before the DigitalTwinView block (before the `: (` on line 620-621), add:

Change from:
```tsx
      ) : viewMode === 'canvas' ? (
        <CanvasView ... />
      ) : (
        <Suspense fallback={...}>
          <DigitalTwinView ... />
        </Suspense>
      ))}
```

To:
```tsx
      ) : viewMode === 'canvas' ? (
        <CanvasView ... />
      ) : viewMode === 'scada' ? (
        <Suspense fallback={<div style={{ height: 400, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--muted)' }}>Loading 2D View...</div>}>
          <ScadaView
            sensors={screenSensors}
            currentValues={currentValues}
            activeScreen={activeScreen}
            scadaEquipment={scadaEquipment}
            scadaConnections={scadaConnections}
            onSaveEquipment={(items) => saveScadaEquipmentMut.mutate({ screenId: activeScreen, items })}
            onSaveConnections={(items) => saveScadaConnectionsMut.mutate({ screenId: activeScreen, items })}
          />
        </Suspense>
      ) : (
        <Suspense fallback={...}>
          <DigitalTwinView ... />
        </Suspense>
      ))}
```

- [ ] **Step 7: Build and verify**

Run: `cd client && npx tsc --noEmit 2>&1 | head -30`
Expected: No errors

Run: `cd client && npm run build 2>&1 | tail -10`
Expected: Build succeeds

- [ ] **Step 8: Commit**

```bash
git add client/src/App.tsx
git commit -m "feat(scada): wire ScadaView into App with 2D view mode toggle"
```

---

## Task 12: Integration Testing and Final Polish

**Files:**
- All files (verification only)

- [ ] **Step 1: Build the full backend**

Run: `cargo build 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 2: Run backend tests**

Run: `cargo test 2>&1 | tail -15`
Expected: All tests pass

- [ ] **Step 3: Build the full frontend**

Run: `cd client && npm run build 2>&1 | tail -10`
Expected: Build succeeds with no errors

- [ ] **Step 4: Manual smoke test**

Start the dev server:
```bash
cargo run &
cd client && npm run dev
```

1. Open the app in a browser and log in
2. Click the "2D" view mode button in the header
3. Verify the empty SCADA canvas appears with grid background
4. Click "Edit" to enter edit mode
5. Click an equipment type from the palette
6. Click on the canvas to place the equipment
7. Verify the P&ID symbol renders correctly
8. Click the equipment to select it, verify the edit panel appears
9. Bind a sensor to the equipment
10. Use the "Connect" tool to connect two equipment
11. Click "Save Layout"
12. Refresh the page and verify the layout persists
13. Exit edit mode and verify real-time data values appear on equipment

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "feat(scada): complete 2D SCADA view with P&ID symbols and real-time data"
```
