# SCADA 2D View Design

## Overview

Add a new "2D SCADA" view mode to Voltrus Lite, providing a P&ID schematic-style visualization of industrial equipment with real-time sensor data overlays. This is a fourth view mode alongside the existing grid, canvas, and 3D views.

The 2D view uses an SVG canvas with inline React SVG components for equipment assets, connection lines, and data labels. It has its own layout storage separate from the 3D view so users can build independent arrangements.

## Approach

**SVG Canvas** rendered natively in React. No new dependencies. Equipment are SVG groups, connections are polylines, data labels are SVG text elements. Pan/zoom implemented via SVG `viewBox` transform.

## Equipment Types

| Type | Symbol Description | Default Size |
|------|-------------------|--------------|
| Solar Panel | Rectangle with diagonal line, sun rays, +/- terminals | 100x60 |
| Inverter | Rectangle with DC/AC labels, sine wave icon | 80x60 |
| Gateway/PLC | Rectangle with antenna icon, Ethernet port | 80x70 |
| Transformer | Two overlapping coils (classic P&ID), primary/secondary terminals | 90x80 |
| Battery | Rectangle with level bars, +/- terminals | 70x80 |
| Meter | Circle with "M" inside, measurement lines | 60x60 |
| Circuit Breaker | Rectangle with toggle switch, ON/OFF indicator | 60x70 |

## Visual Style

- **Background**: White/light with subtle grid lines
- **Equipment**: Stroke-based line-art, `#333` stroke, stroke-width 2, no fills
- **Connection ports**: Small circles at equipment edges
- **Status colors**: Normal = green border, Warning = yellow, Critical = red (overrides stroke)
- **Connection lines**:
  - Power: solid thick line, amber
  - Data: dashed thin line, blue
  - Flow direction: animated dash offset
- **Labels**: Equipment name below symbol, sensor values in small overlay box at top-right

## Backend Data Model

### New storage table: `scada_layouts`

```rust
#[derive(Serialize, Deserialize)]
struct ScadaEquipment {
    id: String,
    equipment_type: ScadaEquipmentType,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    label: String,
    sensor_bindings: Vec<String>,
    props: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
enum ScadaEquipmentType {
    SolarPanel,
    Inverter,
    Gateway,
    Transformer,
    Battery,
    Meter,
    CircuitBreaker,
}

#[derive(Serialize, Deserialize)]
struct ScadaConnection {
    id: String,
    from_equipment: String,
    to_equipment: String,
    waypoints: Vec<(f64, f64)>,
    connection_type: ConnectionType,
}

#[derive(Serialize, Deserialize)]
enum ConnectionType {
    Power,
    Data,
    Flow,
}

#[derive(Serialize, Deserialize)]
struct ScadaLayout {
    equipment: Vec<ScadaEquipment>,
    connections: Vec<ScadaConnection>,
}
```

### New API endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/scada/layout` | Load all equipment and connections |
| PUT | `/api/scada/layout` | Save entire layout (equipment positions, connections, sensor bindings) |

Storage follows the same pattern as `TwinLayout` — single JSON blob persisted via the existing storage layer.

## Frontend Component Structure

```
client/src/
├── ScadaView.tsx              # Main view component, edit/view mode toggle
├── scada/
│   ├── ScadaCanvas.tsx        # SVG canvas with pan/zoom, drag, selection
│   ├── ScadaEquipment.tsx     # Renders SVG asset + data label overlay
│   ├── ScadaConnection.tsx    # SVG polyline with flow animation
│   ├── ScadaEditToolbar.tsx   # Toolbar: add equipment, connect, delete, save
│   ├── ScadaEditPanel.tsx     # Side panel: edit props, bind sensors
│   ├── ScadaPalette.tsx       # Equipment type picker
│   ├── defaults.ts            # Default sizes and properties per type
│   └── assets/
│       ├── SolarPanelSvg.tsx
│       ├── InverterSvg.tsx
│       ├── GatewaySvg.tsx
│       ├── TransformerSvg.tsx
│       ├── BatterySvg.tsx
│       ├── MeterSvg.tsx
│       └── CircuitBreakerSvg.tsx
```

### Component responsibilities

- **ScadaView**: Top-level component. Holds layout state, edit mode flag, fetches/saves layout, passes sensor data down. Manages view mode toggle in `App.tsx`.
- **ScadaCanvas**: Root `<svg>` element. Handles viewBox for pan/zoom (wheel=zoom, drag empty space=pan). Renders equipment and connections as children. Manages selection state.
- **ScadaEquipment**: Wraps an SVG asset component. Handles drag (in edit mode), resize handles, rotation. Renders status color border and sensor data label overlay.
- **ScadaConnection**: `<polyline>` with optional waypoints. Animates dash-offset for flow direction. Click to select in edit mode.
- **ScadaEditToolbar**: Mode buttons (select, connect, delete), save button, zoom controls.
- **ScadaEditPanel**: Appears when equipment selected in edit mode. Shows label editor, sensor binding picker, type-specific properties.
- **ScadaPalette**: Grid of equipment type icons. Click to select type, then click canvas to place.

### View mode integration in App.tsx

Add `'scada'` to the `viewMode` union type. Add a toggle button in the header alongside grid/canvas/3D. When `'scada'` is active, render `<ScadaView />` instead of other views.

## Edit Mode Interactions

| Action | Interaction |
|--------|-------------|
| Add equipment | Click type in palette, click on canvas to place at that position |
| Move equipment | Drag equipment body (edit mode only) |
| Resize | Corner handles, minimum size per type enforced |
| Rotate | Shift+scroll or rotation handle, 15-degree snaps |
| Connect | Click connect tool, click source port, click target port — auto-routes with right-angle waypoints (Manhattan routing) |
| Bind sensors | Select equipment, use sensor picker in side panel, multi-select |
| Delete | Select, press Delete key or toolbar delete button |
| Save | Single save button, PUT entire layout to backend |

## Real-time Data Flow

1. `useSensorData()` TanStack Query polls sensor values (same query used by 3D view)
2. `ScadaEquipment` reads bound sensor IDs from layout state
3. Looks up latest values from polled data
4. Renders values as small text overlay on the SVG equipment
5. Status color computed from sensor warning/critical thresholds (same logic as 3D view EquipmentLabel)

## Pan & Zoom

- Mouse wheel: zoom in/out (range 0.25x to 4x)
- Click+drag on empty canvas: pan
- Toolbar buttons: zoom in, zoom out, fit-to-view
- Implemented via SVG `viewBox` attribute manipulation
