# SCADA 2D Redesign — Fabric.js

## Overview

Replace the current raw SVG SCADA editor with a Fabric.js canvas-based implementation. Goals: traditional SCADA feel with modern UX, expression-based visual rules, rich label system, and AI-scaffoldable JSON serialization.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Rendering engine | Fabric.js | JSON serialization built-in, native drag/resize/rotate, AI-friendly |
| State management | Existing custom store | Direct `getState()` in render loop, no React re-renders needed |
| Expression engine | Custom recursive descent parser (~150 lines) | Safe AST evaluation, supports ternary/arithmetic/comparisons/boolean |
| Connection routing | Freeform polylines | Simple, flexible, easy for AI to generate waypoints |
| Label system | Attached + standalone + rich templates | Covers nameplates, independent annotations, dynamic text |
| Visual rules | Full expression engine with multi-property targets | Supersedes hardcoded warn/crit coloring, builds on existing alarm thresholds |

## Data Model

### Core Types

```ts
interface ScadaEquipmentProps {
  id: string
  equipment_type: ScadaEquipmentType
  name: string
  x: number; y: number
  width: number; height: number
  rotation: number
  bindings: SensorBinding[]
  visual_rules: VisualRule[]
  attached_labels: AttachedLabel[]
  ports: PortDef[]
}

interface ScadaConnectionProps {
  id: string
  from_id: string
  to_id: string
  points: [number, number][]
  connection_type: 'power' | 'data' | 'flow'
  visual_rules: VisualRule[]
  attached_labels: AttachedLabel[]
}

interface ScadaLabelProps {
  id: string
  template: string              // "Total Power: ${sensor_5.value} kW"
  bindings: SensorBinding[]
  visual_rules: VisualRule[]
  x: number; y: number
  style: { fontSize, fill, fontFamily, fontWeight }
}
```

### Supporting Types

```ts
interface SensorBinding {
  sensor_id: number
  alias?: string                // e.g. "voltage" for use in expressions
}

interface VisualRule {
  target: string                // "fill", "stroke", "opacity", "visible", "text", "strokeWidth"
  expression: string            // "if(value >= crit, '#ef4444', '#22c55e')"
}

interface AttachedLabel {
  template: string              // "${name}: ${value} ${unit}"
  offset: [number, number]      // relative to parent center
  style?: { fontSize?, fill?, fontWeight? }
}

interface PortDef {
  x: number; y: number          // 0-1 normalized
  side: 'top' | 'bottom' | 'left' | 'right'
}
```

### Expression Context

Available in every expression evaluation:

- `value` -- current sensor value
- `warn`, `crit` -- thresholds from sensor config
- `min`, `max` -- sensor range
- `status` -- alarm status string
- Named aliases from bindings

### AI Serialization Format

```json
{
  "version": "1.0",
  "screen_id": "screen_main",
  "objects": [
    { "type": "ScadaEquipment", "id": "meter_1", "equipment_type": "meter" },
    { "type": "ScadaConnection", "from_id": "meter_1", "to_id": "inverter_1" },
    { "type": "ScadaLabel", "template": "Site Output: ${value} kW" }
  ]
}
```

AI generates this JSON, `canvas.loadFromJSON()` renders it.

## Architecture

```
client/src/scada2/
├── ScadaView.tsx              # Top-level wrapper, mode switching
├── ScadaCanvas.tsx            # Fabric.js canvas mount + render loop
├── ScadaToolbar.tsx           # Edit toolbar (tools, save, zoom)
├── ScadaPalette.tsx           # Equipment palette
├── ScadaEditPanel.tsx         # Property panel for selected object
├── engine/
│   ├── types.ts               # All type definitions
│   ├── serializer.ts          # toJSON / loadFromJSON helpers
│   ├── expression.ts          # Expression evaluator (AST-based, safe)
│   ├── bindings.ts            # Sensor data -> expression context resolver
│   └── render-loop.ts         # Data tick -> evaluate rules -> update objects
├── objects/
│   ├── ScadaEquipment.ts      # Custom Fabric.Group subclass
│   ├── ScadaConnection.ts     # Custom Fabric.Polyline subclass
│   └── ScadaLabel.ts          # Custom Fabric.Text subclass
├── symbols/
│   ├── index.ts               # Symbol registry
│   ├── transformer.ts         # SVG string + port defs per type
│   ├── meter.ts
│   └── ...
└── defaults.ts                # Default sizes, port positions, colors
```

## Expression Engine

Custom recursive descent parser (~150 lines). AST-based evaluation, no dynamic code execution.

**Supported syntax:**

- Ternary: `if(value > 220, 'red', 'green')`, nested ternary
- Comparisons: `>`, `<`, `>=`, `<=`, `==`, `!=`
- Arithmetic: `+`, `-`, `*`, `/`, `abs()`
- Boolean: `and`, `or`, `not`
- Literals: color strings (`'#ef4444'`), numbers, booleans

**Rule targets and result types:**

| target | applies to | result type |
|--------|-----------|-------------|
| `fill` | equipment, connection, label | color string |
| `stroke` | equipment, connection | color string |
| `opacity` | any | number 0-1 |
| `visible` | any | boolean |
| `text` | label | string (overrides template) |
| `strokeWidth` | connection | number |

## Symbol System

Existing SVG React components convert to SVG strings loaded via `fabric.loadSVGFromString()`.

```ts
interface SymbolDef {
  svg: string
  ports: PortDef[]
  defaultSize: { width: number; height: number }
}
```

Each symbol file exports one `SymbolDef`. Registry in `symbols/index.ts` maps `ScadaEquipmentType` to `SymbolDef`. Adding new symbols: drop a file, register it. AI can generate SVG strings.

## Data Flow

### Editing

User clicks palette -> `createEquipmentObject(type)` -> Fabric object on canvas -> built-in drag/resize/rotate -> property panel edits bindings/rules -> changes written to Fabric object custom properties -> save serializes full canvas.

### Live Mode

`render-loop.ts` reads `dashboardStore.getState().currentValues` on each SSE tick -> iterates canvas objects -> resolves bindings -> evaluates visual rules -> applies property changes -> `canvas.renderAll()`.

### Save/Load

Serialize: canvas objects -> extract custom properties -> API format.
Load: API data -> `canvas.loadFromJSON()` with custom revivers per object type.

### AI Generation

Future: endpoint or local function outputs JSON blob -> `canvas.loadFromJSON()` -> canvas renders. No special handling.

## Interaction Model

**Tools:** `select`, `connect`, `label`, `delete`

**Select mode:** Click to select, drag to move, corner handles for resize/rotate. Shift+click for multi-select. Delete key removes.

**Connect mode:** Click equipment -> port dots appear -> click port -> drag to target equipment port -> creates polyline. Double-click connection to edit waypoints.

**Label mode:** Click canvas to place label. Inline text editing via Fabric.js. Property panel for template/bindings/rules/style.

**Keyboard:** `Ctrl+S` save, `Delete` remove, `Escape` deselect/cancel, `Space+drag` pan.

**Zoom/pan:** Fabric.js `canvas.zoomToPoint()` on wheel, viewport transform for panning. Native, no manual viewBox.

## Migration Notes

- Current `ScadaEquipmentType` union type is reused as-is (20 types)
- Current `PORT_POSITIONS` and `DEFAULT_SIZES` migrate to `defaults.ts`
- SVG React components become SVG strings in `symbols/` files
- Existing API endpoints for equipment/connections unchanged -- serializer maps between Fabric format and API format
- `client/src/scada/` directory replaced by `client/src/scada2/`
- `ScadaView.tsx` at top level updated to import from `scada2/`
