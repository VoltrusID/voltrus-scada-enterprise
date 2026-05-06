# Node-RED Style Flow Editor — Design Spec

## Summary
Redesign the FlowManager component to resemble Node-RED: a left-side categorized command palette, a dot-grid canvas with horizontally auto-laid-out nodes, custom mouse-event drag-and-drop (both from palette and for reordering), SVG bezier connections, and a right-side config panel for the selected node.

## Context
The current FlowManager has a basic node editor but lacks the intuitive visual feel of Node-RED. Users want a more familiar industrial automation interface with clear INPUT/PROCESS/OUTPUT categorization.

## Goals
- Lower cognitive load for operators familiar with Node-RED
- Faster flow creation via categorized palette + drag-drop
- Clear visual separation of triggers (input), transforms (process), and actions (output)
- Maintain auto-layout so flows are always readable

## Non-Goals
- Full 2D free-form canvas (keep 1D sequence semantics)
- Node-RED compatibility / import-export
- Sub-flows or groups

---

## Architecture

### Component Hierarchy
```
FlowManager (container, manages state)
├── FlowList (existing — list of flows, unchanged)
└── FlowEditor (new — main editor view)
    ├── PaletteSidebar (left — categorized node palette)
    ├── CanvasArea (center — nodes + connections)
    │   ├── DotGridBackground (SVG pattern)
    │   ├── NodeSequence (horizontal layout)
    │   │   ├── TriggerNode (first, diamond style)
    │   │   ├── ProcessNode (rectangular)
    │   │   └── ...
    │   └── EdgeLayer (SVG bezier curves between nodes)
    ├── ConfigPanel (right — selected node config form)
    └── Toolbar (top — save, delete, test, start/stop)
```

### State Management
- `selectedFlowId`: which flow is being edited
- `form`: the editable flow state (id, name, trigger_config, nodes string)
- `parsedNodes`: memoized `FlowNodeConfig[]` from `form.nodes`
- `parsedTrigger`: memoized trigger config object
- `selectedNodeIndex`: which node (or trigger) is selected for config
- `dragState`: `{ type: 'palette'|'reorder', nodeType?: FlowNodeType, fromIndex?: number, currentX: number, currentY: number } | null`
- `ghostPosition`: `{ x, y }` for rendering the drag ghost

### Drag & Drop Logic (Approach B — Custom Mouse Events)

#### From Palette to Canvas
1. `mousedown` on palette item → set `dragState = { type: 'palette', nodeType }`, capture initial mouse position
2. `mousemove` on document → update `ghostPosition` to follow cursor, show semi-transparent ghost node
3. `mouseup` on canvas → calculate insertion index from drop X position (snap to nearest gap between nodes), insert new node, clear drag state
4. `mouseup` outside canvas → clear drag state, no insertion

#### Reordering Nodes on Canvas
1. `mousedown` on node header (not config buttons) → set `dragState = { type: 'reorder', fromIndex }`
2. `mousemove` on document → update `ghostPosition`, other nodes slide apart to show insertion gap
3. `mouseup` → calculate target index from drop position, move node in array, clear drag state

#### Ghost Rendering
- Rendered via React portal or absolute-positioned overlay at `ghostPosition`
- Semi-transparent (opacity 0.7), slight scale-up (1.05x), drop shadow
- Shows node icon + label

### Auto-Layout Algorithm
Canvas width is measured. Node width is fixed (180px). Gap is 48px.

**Data model note:** The trigger is stored separately in `trigger_config` (not in the `nodes` array). Visually, it is rendered as the first element in the sequence (position 0), followed by the nodes from the `nodes` array.

Total visual sequence width = `(1 + nodes.length) * 180 + nodes.length * 48` (trigger + nodes + gaps).
Nodes are centered in canvas: `startX = (canvasWidth - totalWidth) / 2`.
Each element position: `x = startX + i * (180 + 48)`, `y = canvasHeight / 2 - 60` (vertically centered).
Trigger node is visually distinct (diamond shape, different border color).

### SVG Connections
- Bezier curves: `M from.x from.y C mx from.y, mx to.y, to.x to.y` where `mx = (from.x + to.x) / 2`
- Arrowhead marker at end
- Animated dash offset on data flow (optional, low priority)

### Dot Grid Background
- SVG `<pattern>` with 20px spacing, 1px dots at `var(--border)` 15% opacity
- Fills the entire canvas area

### Config Panel (Right Sidebar)
- Width: 320px
- Collapsible (chevron toggle)
- Shows form fields based on `NODE_META` or `TRIGGER_META` schema
- For trigger: shows trigger type selector + type-specific fields
- For nodes: shows all fields from `defaultConfig` as editable inputs
- Auto-saves to `form.nodes` / `form.trigger_config` on change

### Palette Sidebar (Left)
- Width: 220px
- Fixed position
- Collapsible categories: INPUT, PROCESS, OUTPUT
- Each category starts expanded
- Items show: color dot, icon (16px), label, short desc on hover
- `cursor: grab` on hover, `cursor: grabbing` on active drag

---

## Node Categorization

### INPUT (Triggers)
| Type | Label | Color | Icon |
|------|-------|-------|------|
| modbus | Modbus | #22d3ee | zap |
| timer | Timer | #fbbf24 | clock |
| mqtt | MQTT | #c084fc | radio |

### PROCESS
| Type | Label | Color | Icon |
|------|-------|-------|------|
| filter | Filter | #22d3ee | filter |
| transform | Transform | #60a5fa | shuffle |
| threshold | Threshold | #fbbf24 | alert-triangle |
| lookup | Lookup | #c084fc | search |
| expression | Expression | #4ade80 | code |

### OUTPUT
| Type | Label | Color | Icon |
|------|-------|-------|------|
| data_source_out | Data Source | #a3e635 | database |
| http_out | HTTP Webhook | #818cf8 | globe |
| alarm_out | Alarm | #f87171 | bell |

---

## Visual Design

### Industrial Theme (matches Voltrus)
- Background: `var(--bg)` (#0f172a)
- Card/Node background: `var(--card)` (#1e293b)
- Border: `var(--border)` (#334155)
- Text: `var(--fg)` (#e2e8f0)
- Muted: `var(--muted)` (#64748b)
- Grid dots: `var(--border)` at 15% opacity

### Node Styling
- Width: 180px
- Header height: 36px with colored left border (4px)
- Border radius: 8px
- Selected state: 2px solid white ring + subtle glow
- Trigger node: diamond shape (rotate 45deg inner content counter-rotate)

### Connection Styling
- Stroke: `var(--border)` (#334155), 2px
- Active data flow: `var(--cyan)` (#22d3ee) with animated dash

---

## Data Flow

### Creating a New Flow
1. User clicks "New Flow" in FlowList
2. Switches to FlowEditor with empty form
3. User drags trigger from INPUT palette → canvas
4. Trigger config panel opens automatically
5. User drags process/output nodes from palette
6. Nodes auto-connect left-to-right
7. User clicks each node to configure in right panel
8. User clicks Save → `onSave` called

### Editing Existing Flow
1. User clicks flow in FlowList
2. FlowEditor loads with parsed nodes
3. User can drag-reorder, delete, or add new nodes
4. Config panel shows selected node
5. Save updates the flow

---

## Error Handling
- Invalid JSON in nodes/trigger: show red border on editor, error tooltip
- Missing trigger: disable Save button, show "Add a trigger to start" banner
- Empty flow name: inline validation
- DnD edge cases: ESC cancels drag, window blur cancels drag

## Performance
- Node positions computed once on mount + resize (use `ResizeObserver`)
- Connections re-render only when node count/order changes
- Ghost rendered via `requestAnimationFrame` during drag for smoothness
- Debounce config panel saves (100ms)

## Accessibility
- Keyboard: Tab through palette items, Enter to select/add
- Arrow keys to navigate between nodes on canvas
- Delete key to remove selected node
- ARIA labels on all interactive elements

---

## Open Questions
1. Should we support copy-paste of nodes? (Phase 2)
2. Should connections be clickable/deletable? (Not needed for 1D flow)
3. Should the canvas zoom/pan? (No — keep simple, flows are short)
