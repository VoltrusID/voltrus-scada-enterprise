# React Client Deep-Dive Reference

> Compiled from full source analysis. Use this as the single source of truth for API contracts, UI structure, state management, and styling when building the Lit migration.

---

## 1. API Contract (Complete)

### Auth
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/auth/me` | GET | — | `{ authenticated: boolean, role: string }` |
| `/api/auth/login` | POST | `{ username, password }` | `{ role }` or `{ error }` |
| `/api/auth/logout` | POST | — | — |

### Screens
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/screens` | GET | — | `Screen[]` |
| `/api/screens` | POST | `{ name }` | `{ id, name }` |
| `/api/screens/:id` | PUT | `{ name }` | — |
| `/api/screens/:id` | DELETE | — | — |

### Data Sources (Sensors)
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/data-sources` | GET | — | `SensorConfig[]` |
| `/api/data-sources` | POST | `Partial<SensorConfig>` | `{ id }` |
| `/api/data-sources/:id` | PUT | `SensorConfig` | — |
| `/api/data-sources/:id` | DELETE | — | — |

### Screen-Specific Data Sources
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/screens/:id/data-sources` | GET | — | `SensorConfig[]` |
| `/api/screens/:id/data-sources` | POST | `{ data_source_id }` | — |
| `/api/screens/:id/data-sources/:sid` | DELETE | — | — |

> **Note:** `all` screen uses `/api/data-sources` directly. Other screens use `/api/screens/:id/data-sources`.

### Layouts
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/screens/:id/layouts` | GET | — | `{ lg, md, sm }` |
| `/api/screens/:id/layouts` | PUT | `ScreenLayouts` | — |
| `/api/screens/:id/canvas-layouts` | GET | — | `CanvasLayoutItem[]` |
| `/api/screens/:id/canvas-layouts` | PUT | `CanvasLayoutItem[]` | — |

> **Note:** For `all` screen, `__overview` is substituted as the actual ID.

### 3D Twin
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/screens/:id/twin-layouts` | GET | — | `{ items, camera }` |
| `/api/screens/:id/twin-layouts` | PUT | `TwinLayout[]` | — |
| `/api/screens/:id/twin-connections` | GET | — | `{ from_id, to_id }[]` |
| `/api/screens/:id/twin-connections` | PUT | `{ from_id, to_id }[]` | — |
| `/api/screens/:id/twin-camera` | PUT | `{ x, y, z }` | — |

### 2D SCADA
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/screens/:id/scada-equipment` | GET | — | `{ items: ScadaEquipmentItem[] }` |
| `/api/screens/:id/scada-equipment` | PUT | `{ items }` | — |
| `/api/screens/:id/scada-connections` | GET | — | `{ items: ScadaConnectionItem[] }` |
| `/api/screens/:id/scada-connections` | PUT | `{ items }` | — |

### History
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/history?range=30m&screen=&detail=&agg=` | GET | — | `{ data: Record<string, CompactData> }` |

**CompactData format:**
```ts
{ start: number, interval: number, values: number[] }
// OR legacy: DataPoint[]
```

### Shortcuts
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/shortcuts` | GET | — | `Record<number, string>` (sensorId → shortcut) |
| `/api/shortcuts` | PUT | `Record<number, string>` | — |
| `/api/nav-shortcuts` | GET | — | `Record<string, string>` (action → shortcut) |
| `/api/nav-shortcuts` | PUT | `Record<string, string>` | — |

### Branding
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/settings/branding` | GET | — | `{ title, subtitle, logo }` |
| `/api/settings/branding` | PUT | `BrandingSettings` | — |

### System
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/system` | GET | — | `{ hostname, os, uptime_secs, cpu_usage, memory_*, disk_*, process: { pid, memory_mb, cpu_percent, uptime_secs, binary_size_mb } }` |

### Alarms
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/alarms?status=&limit=` | GET | — | `Alarm[]` |
| `/api/alarms` | POST | — | ack all |
| `/api/alarms/:id/acknowledge` | POST | — | ack one |

### Modbus Devices
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/modbus-devices` | GET/POST | — / device | `ModbusDevice[]` / `{ ok, id }` |
| `/api/modbus-devices/:id` | GET/PUT/DELETE | — | — |
| `/api/modbus-devices/:id/registers` | PUT | `{ registers }` | — |
| `/api/modbus-devices/:id/restart` | POST | — | — |
| `/api/modbus-devices/status` | GET | — | `{ device_id, running, status }[]` |

### Users
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/users` | GET/POST | — / user | `UserItem[]` |
| `/api/users/:id` | PUT/DELETE | — | — |

### Misc
| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/api/seed` | POST | — | seeds demo data |
| `/api/settings/alarm-cooldown` | GET/PUT | `{ secs }` | `{ secs }` |

### SSE Live Stream
| Endpoint | Format | Payload |
|----------|--------|---------|
| `/api/stream` | EventSource | `SensorData[]` every flush interval (batched) |

**SensorData:**
```ts
{ id, key, value, warn, crit, source_type }
```

---

## 2. Type Definitions

```ts
interface SensorConfig {
  id: number; key: string; name: string; unit: string; color: string;
  min_val: number; max_val: number; warn_val: number; crit_val: number;
  base_val: number; sort_order: number; decimals: number;
  source_type: string; source_config: string;
  device_id: string;
  equipment_id: string | null;
  chart_type: string; // 'sparkline' | 'bignumber' | 'gauge' | 'image' | 'text'
}

interface SensorData {
  id: number; key: string; value: number; warn: number; crit: number; source_type: string;
}

interface DataPoint { value: number; ts: number }

interface Screen { id: string; name: string; type: string; sort_order: number }

interface LayoutItem { i: string; x: number; y: number; w: number; h: number }
type ScreenLayouts = { lg: LayoutItem[]; md: LayoutItem[]; sm: LayoutItem[] }

interface CanvasLayoutItem {
  data_source_id: number
  x: number; y: number; width: number; height: number
}

interface Alarm {
  id: number; source_key: string; source_name: string;
  severity: 'warning' | 'critical'; value: number; threshold: number;
  message: string; triggered_at: number; resolved_at: number | null;
  acknowledged_at: number | null; acknowledged_by: string | null;
  cooldown_until: number | null;
}

interface BrandingSettings { title: string; subtitle: string; logo: string }
const DEFAULT_BRANDING = { title: 'Voltrus Lite', subtitle: '', logo: '' }

interface TwinEquipment {
  id: string; type: 'solar_panel' | 'inverter' | 'gateway';
  name: string; position: [number, number, number]; rotation: number;
}

interface ScadaEquipmentItem {
  equipment_id: string; equipment_type: ScadaEquipmentType;
  equipment_name: string; x: number; y: number;
  width: number; height: number; rotation: number;
  sensor_bindings: string;
}

interface ScadaConnectionItem {
  connection_id: string; from_equipment: string; to_equipment: string;
  waypoints: string; connection_type: 'power' | 'data' | 'flow';
}
```

---

## 3. State Management (React)

**No Redux. No Zustand. TanStack Query + local `useState`.**

### Query Keys (used for cache invalidation)
```ts
['dataSources']                    // all sensors
['screenDataSources', screenId]    // screen-specific sensors
['screens']                        // all screens
['layouts', screenId]              // grid layouts
['twinLayouts', screenId]          // 3D twin layouts
['canvasLayouts', screenId]        // canvas layouts
['shortcuts']                      // sensor shortcuts
['navShortcuts']                   // navigation shortcuts
['branding']                       // branding settings
['history', range, screenId]       // history data
['systemInfo']                     // system stats
['alarms', status]                 // alarms
['modbusDevices']                  // modbus devices
['modbusDevice', id]
['modbusRegisters', id]
['modbusStatus']                   // poller status
['scadaEquipment', screenId]
['scadaConnections', screenId]
['users']
```

### Global App State (in App.tsx)
```ts
authed, authRole, authLoading, loginError
dataMap: Record<string, DataPoint[]>        // live sparkline data per sensor
historyMap: Record<string, DataPoint[]> | null
historyReady: boolean
currentValues: Record<string, SensorData>   // latest SSE value per sensor
alerts: number                              // count of active alarms
showAlarmPanel, alarmMuted, paused, turbo
editMode, editingId, dashSettings
selectedSensor, activeScreen, localLayouts
mode: 'live' | 'history' | 'system'
historyRange, historyDetail
showDataManager, showAddDialog
screenPrompt, screenConfirm
showBrandingDialog, layoutBreakpoint
viewMode: 'cards' | 'canvas' | 'twin' | 'scada'
localCanvasLayouts, showShortcutsPanel
```

### SSE Data Flow
1. `useSSEData()` opens `EventSource('/api/stream')`
2. Batches incoming `SensorData[]` into a buffer
3. Flushes every 1000ms to `setState({ data, connected })`
4. App.tsx reads `sseData.data` and:
   - Appends to `dataMap` (rolling window of 180 pts = `LIVE_PTS`)
   - Updates `currentValues`
   - Counts alarms (if `value >= crit` or `value >= warn`)
   - Triggers alarm audio if `alerts > 0`

### History Data Flow
1. On mode switch to `history`, fetch `/api/history?range=...&screen=...`
2. Expand compact format (`start` + `interval` + `values[]`) to `DataPoint[]`
3. Store in `historyMap`
4. On switch back to `live`, re-fetch 30m history to re-seed `dataMap`

---

## 4. UI Structure & Navigation

### Header Bar (always visible)
- **Left:** Logo + Title + Subtitle (clickable in edit mode → branding dialog)
- **Right:**
  - Alarm bell (with count badge, pulse animation)
  - View mode toggle: Grids / Canvas / 3D / 2D
  - Shortcuts config button (opens panel)
  - Live/History toggle (hidden in 3D/2D mode)
  - Pause/Resume button
  - Turbo button
  - Settings button (dashboard settings dialog)
  - Edit/Done button (admin only)
  - + Add Source button (edit mode only)
  - Data Manager button
  - Logout button

### Screen Tabs (below header)
- Overview (`all`)
- Custom screens (dynamic)
- System tab (switches to system mode)
- In edit mode: rename/delete icons on each tab, + button to add screen

### View Modes

#### Cards (Grid View)
- Uses `react-grid-layout` with responsive breakpoints: lg=24col, md=12col, sm=4col
- Row height = 50px, gap configurable
- Edit mode: drag/resize cards with `.drag-handle` and resize handles
- Each card = `SensorCard` with embedded `Chart`
- Layouts saved per-screen per-breakpoint

#### Canvas View
- Free-form absolute positioning using `react-rnd` (drag + resize)
- Each sensor is an `Rnd` wrapper around `SensorCard`
- Multi-select with Shift+click
- Alignment toolbar: left/center-h/right/top/center-v/bottom/distribute-h/distribute-v
- Z-index stacking (bring to front on click)
- Background slightly tinted in edit mode

#### 3D Digital Twin
- Three.js via `@react-three/fiber` + `@react-three/drei`
- `Canvas` with `OrbitControls`
- Ground plane + grid helper
- Equipment types: solar_panel, inverter, gateway
- Draggable equipment (raycast to ground plane)
- Rotation via mouse wheel (15° steps)
- Equipment palette (sidebar to add new items)
- Connection lines between equipment
- Labels with live sensor values
- Camera position saved per-screen

#### 2D SCADA
- SVG-based diagram editor
- Equipment palette: solar_panel, inverter, gateway, transformer, battery, meter, circuit_breaker
- Tools: select, connect, delete
- Click to place equipment from palette
- Multi-select, drag, resize
- Draw connections between equipment sides
- Waypoint routing for lines
- Edit toolbar with save/discard
- Equipment edit panel (name, bindings, etc.)

### System View
- Grid of info cards:
  - Host (hostname, OS, uptime)
  - CPU usage (with bar gauge)
  - Memory (used/total, percentage bar)
  - Disk (used/total, percentage bar)
  - Rust Service process stats (RSS, CPU%, uptime, binary size)

### Modals / Overlays
- `LoginScreen` — centered card, username/password
- `EditPanel` — fixed overlay, tabs: General / Source
- `SensorDetail` — fullscreen overlay, back button, detail chart, overlay multi-sensor compare
- `DashboardSettings` — grid gap, refresh interval, history agg method, alarm mute
- `DataManager` — table of all sensors, CRUD, duplicate
- `AddSensorDialog` — add existing sensor to screen
- `AlarmPanel` — slide-in from right, filter tabs, ack buttons
- `BrandingDialog` — title, subtitle, logo
- `ShortcutsConfigPanel` — keyboard shortcut config
- `ScreenDialog` — PromptDialog + ConfirmDialog for screen CRUD

---

## 5. Styling System

### CSS Variables (in `index.css`)
```css
:root {
  --bg: #09090b;
  --card: #111113;
  --border: #27272a;
  --border-light: #3f3f46;
  --text: #e4e4e7;
  --text-dim: #a1a1aa;
  --text-muted: #71717a;
  --warn: #f59e0b;
  --crit: #ef4444;
  --ok: #22c55e;
  --cyan: #22c55e;
  --fg: var(--text);
  --green: #22c55e;
  --muted: var(--text-muted);
}
```

### Global Styles
- Font: Inter (body), JetBrains Mono (monospace inputs, values)
- No CSS framework — all inline styles or CSS-in-JS objects
- Dark theme only
- `box-sizing: border-box` everywhere
- Inputs/selects: dark background, subtle border, no outline

### Shared Style Helpers (`styles.ts`)
```ts
inputStyle      // dark input with border
monoInputStyle  // inputStyle + JetBrains Mono
labelStyle      // flex column gap-4
labelTextStyle  // uppercase muted small text
tabStyle(active) // border-bottom indicator, uppercase
btnStyle        // card bg, border, flex center, gap-6
```

### Chart Types (in `Chart.tsx`)
1. **Sparkline** — Canvas 2D, line chart with:
   - Grid lines (4 horizontal)
   - Y-axis labels
   - Warn/crit dashed threshold lines
   - Gradient fill under line
   - Last value dot + halo
   - Last value text label
2. **BigNumber** — large centered number + unit + progress bar
3. **Gauge** — Canvas 2D, semi-circular arc with color segments (green/yellow/red), needle, value text
4. **Image** — `<img>` with object-fit contain
5. **Text** — Markdown rendered via `marked` + `DOMPurify`, `.md-text` CSS class

### SensorCard Structure
```
┌─────────────────────────────┐
│ ● Name  [TAG]       [SC] status│  ← header (hidden for image/text)
├─────────────────────────────┤
│ 123.45 V                    │  ← value (hidden for image/text)
├─────────────────────────────┤
│         [Chart]             │  ← sparkline/gauge/image/text
└─────────────────────────────┘
```
- Status dot color: green=normal, yellow=warning, red=critical, gray=offline
- Source tag badge (REAL/SIM/API/NONE)
- Shortcut kbd badge
- Remove button in edit mode
- Drag handle (6-dot grip) in edit mode

---

## 6. Keyboard Shortcuts

### Navigation Shortcuts (configurable)
```ts
'screen:all'      → Mod+1  (default)
'screen:system'   → Mod+0  (default)
'screen:<id>'     → Mod+2, Mod+3, ... (auto-assigned)
'view:cards'      → ''
'view:canvas'     → ''
'view:twin'       → ''
'view:scada'      → ''
'mode:live'       → ''
'mode:history'    → ''
'action:alarms'   → ''
```

### Sensor Shortcuts
- Each sensor can have a custom keyboard shortcut
- When pressed, opens `SensorDetail` for that sensor
- Recorded via `EditPanel` (capture keydown, normalize to `Mod+Alt+Shift+Key`)

### Global Shortcuts
- `Escape` — close SensorDetail
- `Mod+1..0` — switch screens
- Shortcuts disabled when focus is in input/textarea/select

---

## 7. Alarm System

### Trigger Logic (frontend)
```ts
if (source_type === 'modbus' && ((crit > 0 && value >= crit) || (warn > 0 && value >= warn))) {
  alertCount++;
}
```

### Audio
- `/alarm.wav` loops while alerts > 0
- Mute toggle in dashboard settings
- `useAlarmAudio()` hook: startAlarm, stopAlarm, toggleMute

### Alarm Panel
- Slide-in from right (360px)
- Filter tabs: all / active / acknowledged
- Each alarm card shows: severity badge, timestamp, source name, message
- Active alarms have pulsing dot
- Acknowledge button per alarm + Ack All button
- Resolved alarms at 60% opacity

---

## 8. Data Retention (Frontend Relevance)

The frontend doesn't manage retention directly, but:
- History API automatically queries across raw/hourly/daily tiers
- Frontend shows seamless data regardless of tier
- Archive path is backend-only

---

## 9. Source Types & Categories

```ts
const SOURCE_CATEGORY = {
  modbus:      { label: 'Modbus',        tag: 'REAL', color: 'var(--ok)' },
  random_walk: { label: 'Random Walk',   tag: 'SIM',  color: '#f59e0b' },
  sine:        { label: 'Sine Wave',     tag: 'SIM',  color: '#f59e0b' },
  step:        { label: 'Step / Spike',  tag: 'SIM',  color: '#f59e0b' },
  sawtooth:    { label: 'Sawtooth',      tag: 'SIM',  color: '#f59e0b' },
  constant:    { label: 'Constant',      tag: 'SIM',  color: '#f59e0b' },
  noise:       { label: 'Noise',         tag: 'SIM',  color: '#f59e0b' },
  api:         { label: 'API',           tag: 'API',  color: '#a78bfa' },
  none:        { label: 'None (static)', tag: 'NONE', color: 'var(--muted)' },
};
```

---

## 10. File Map

```
src/
├── App.tsx              # Root: auth, state, layout, keyboard, SSE integration
├── queries.ts           # ALL API calls: TanStack Query hooks + mutations
├── types.ts             # ALL TypeScript interfaces and constants
├── styles.ts            # Shared style objects + utility functions
├── useSSEData.ts        # SSE hook with buffering (1Hz flush)
├── useAlarmAudio.ts     # Alarm audio hook
├── index.css            # CSS vars, global styles, react-grid-layout overrides
├── main.tsx             # ReactDOM render
├── LoginScreen.tsx      # Login form
├── SensorGrid.tsx       # Grid view (react-grid-layout wrapper)
├── SensorCard.tsx       # Individual sensor card
├── Chart.tsx            # Chart dispatcher: sparkline/gauge/bignumber/image/text
├── CanvasView.tsx       # Free-form canvas (react-rnd wrapper)
├── DigitalTwinView.tsx  # 3D view (Three.js/R3F)
├── ScadaView.tsx        # 2D SCADA editor (SVG)
├── SensorDetail.tsx     # Fullscreen sensor detail + overlay chart
├── DetailChart.tsx      # Large history chart component
├── EditPanel.tsx        # Sensor edit modal (general + source tabs)
├── DashboardSettings.tsx # Dashboard config modal
├── DataManager.tsx      # Sensor CRUD table modal
├── AddSensorDialog.tsx  # Add sensor to screen modal
├── ScreenDialog.tsx     # Prompt + Confirm dialogs
├── ShortcutsConfigPanel.tsx # Keyboard shortcut config
├── BrandingDialog.tsx   # Branding settings modal
├── AlarmPanel.tsx       # Slide-in alarm panel
├── CustomRangePicker.tsx # History range picker
├── AlignmentToolbar.tsx  # Canvas alignment tools
├── SourceConfigFields.tsx # Source-specific config fields
├── ModbusDeviceForm.tsx  # Modbus device form
├── ModbusDeviceManager.tsx # Modbus device list
├── UserManager.tsx       # User CRUD
├── Chart.tsx             # (duplicate? or sub-chart?)
├── scada/                # 2D SCADA sub-components
│   ├── defaults.ts
│   ├── ScadaCanvas.tsx
│   ├── ScadaEditToolbar.tsx
│   ├── ScadaPalette.tsx
│   └── ScadaEditPanel.tsx
└── twin/                 # 3D twin sub-components
    ├── SolarPanelModel.tsx
    ├── InverterModel.tsx
    ├── GatewayModel.tsx
    ├── ConnectionLines.tsx
    ├── EquipmentLabel.tsx
    ├── EquipmentPalette.tsx
    └── defaults.ts
```

---

## 11. Critical Implementation Notes for Lit Migration

### What react-grid-layout does (hard to replace)
- Responsive breakpoints with different column counts
- Drag-and-drop with collision detection
- Resize with constraints
- Vertical compaction (gravity)
- Layout serialization per breakpoint
- **Lit replacement:** Build a custom grid using CSS Grid + native HTML5 DnD OR evaluate a lightweight vanilla JS grid library.

### What react-rnd does (easier to replace)
- Absolute positioning with drag/resize
- Bounds checking
- Resize handles
- **Lit replacement:** Native mouse events + CSS transforms. ~150 lines of custom code.

### What R3F does (can drop)
- React reconciler for Three.js
- Hooks: useThree, useFrame
- Components: Canvas, OrbitControls
- **Lit replacement:** Use Three.js directly. Create canvas, init renderer/scene/camera, run animation loop in `connectedCallback`, cleanup in `disconnectedCallback`.

### What TanStack Query does (replaced by thin service layer)
- Caching, deduping, background refetching, mutations
- **Lit replacement:** Plain `fetch()` + a tiny observable store. The cache invalidation pattern is just `EventTarget.dispatchEvent(new CustomEvent('invalidate', { detail: 'sensors' }))`.

### Chart rendering (keep exactly as-is)
- All charts are Canvas 2D or plain DOM
- No charting library dependency
- These can be copied almost verbatim into Lit components
- Only change: React refs → Lit `@query()` decorators or `renderRoot.querySelector('canvas')`

### Styling (adapt to Lit)
- React uses inline `style={{...}}` objects
- Lit uses `static styles = css\`...\`` (scoped to Shadow DOM)
- CSS custom properties (`var(--bg)`) work identically in both
- Import `index.css` globally in `index.html`, then use CSS vars in Lit component styles

### Keyboard shortcuts (keep logic, change binding)
- React: `useEffect(() => window.addEventListener('keydown', ...), [])`
- Lit: `connectedCallback() { window.addEventListener('keydown', ...) }`
- Same normalization logic, same shortcut map structure

---

*Document version: 2025-04-28*  
*Scope: Complete React client analysis for Lit migration reference*
