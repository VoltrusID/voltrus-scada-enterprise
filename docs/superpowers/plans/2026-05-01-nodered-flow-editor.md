# Node-RED Style Flow Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign FlowManager to a Node-RED style interface with left palette, center canvas with auto-layout nodes, SVG connections, drag-and-drop, and right config panel.

**Architecture:** Custom mouse-event drag-and-drop (no libraries). Auto-layout computes node positions from array index. Trigger rendered as first visual node (diamond shape). Palette categorized into INPUT/PROCESS/OUTPUT.

**Tech Stack:** React 19, TypeScript, inline styles (existing codebase pattern), no external DnD libraries.

---

## File Structure

| File | Responsibility |
|------|---------------|
| `client/src/components/flow-editor/FlowEditor.tsx` | Main editor container — orchestrates palette, canvas, config panel, toolbar |
| `client/src/components/flow-editor/PaletteSidebar.tsx` | Left sidebar with categorized draggable node items |
| `client/src/components/flow-editor/CanvasArea.tsx` | Center canvas — dot grid, auto-layout nodes, SVG connections, drop target |
| `client/src/components/flow-editor/FlowNode.tsx` | Individual node card (trigger or process/output) with drag handle |
| `client/src/components/flow-editor/ConfigPanel.tsx` | Right sidebar — editable form for selected node/trigger |
| `client/src/components/flow-editor/DragGhost.tsx` | Semi-transparent ghost node that follows cursor during drag |
| `client/src/components/flow-editor/constants.ts` | NODE_META, TRIGGER_META, TRIGGER_DEFAULTS, layout constants |
| `client/src/FlowManager.tsx` | Updated main component — wires FlowEditor + FlowList |

---

## Prerequisites

Run from repo root:
```bash
git checkout master
git pull origin master
```

Ensure frontend builds:
```bash
cd client && npm run build
```

---

### Task 1: Extract constants to shared file

**Files:**
- Create: `client/src/components/flow-editor/constants.ts`
- Modify: `client/src/FlowManager.tsx` (remove inline constants)

**Context:** The current `NODE_META`, `TRIGGER_META`, and `TRIGGER_DEFAULTS` are defined inline in FlowManager.tsx. We need them shared so PaletteSidebar and FlowNode can import them.

- [ ] **Step 1: Create constants file**

```typescript
// client/src/components/flow-editor/constants.ts
import type { FlowNodeType } from '../../types'

export const NODE_WIDTH = 180
export const NODE_GAP = 48
export const NODE_HEIGHT = 120
export const CANVAS_PADDING = 60

export const NODE_META: Record<FlowNodeType, { label: string; color: string; icon: string; desc: string; defaultConfig: Record<string, any>; category: 'process' | 'output' }> = {
  filter:        { label: 'Filter',        color: '#22d3ee', icon: 'M3 6h18M3 12h18M3 18h18', desc: 'Pass only if condition is true', defaultConfig: { condition: 'payload.metrics.temp > 0' }, category: 'process' },
  transform:     { label: 'Transform',     color: '#60a5fa', icon: 'M23 4l-11 16-5-5-5 5M23 4h-6', desc: 'Map fields to new schema', defaultConfig: { map: { status: "if(payload.metrics.temp > 80, 'CRITICAL', 'OK')" } }, category: 'process' },
  threshold:     { label: 'Threshold',     color: '#fbbf24', icon: 'M12 2v20M2 12h20', desc: 'Check values against rules', defaultConfig: { field: 'payload.metrics.temp', rules: [{ gt: 80, set: { alarm: 'HIGH' } }] }, category: 'process' },
  lookup:        { label: 'Lookup',        color: '#c084fc', icon: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z', desc: 'Map values via lookup table', defaultConfig: { field: 'payload.machine_code', output_field: 'payload.machine_name', table: { M1: 'Cutting' }, default: 'Unknown' }, category: 'process' },
  expression:    { label: 'Expression',    color: '#4ade80', icon: 'M9 7h6M9 11h6M9 15h6M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z', desc: 'Compute new field from formula', defaultConfig: { output_field: 'payload.power_kw', expr: 'payload.metrics.voltage * payload.metrics.current * 0.001' }, category: 'process' },
  data_source_out: { label: 'Data Source',  color: '#a3e635', icon: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5', desc: 'Write to virtual sensor', defaultConfig: { value_field: 'payload.power_kw', source_key: 'flow_power', name: 'Power', unit: 'kW', decimals: 2 }, category: 'output' },
  http_out:      { label: 'HTTP Webhook',  color: '#818cf8', icon: 'M10 13a5 5 0 007 0M4 15l3-3-3-3M20 15l-3-3 3-3', desc: 'POST to external URL', defaultConfig: { url: 'https://example.com/webhook', headers: {}, timeout_ms: 5000, retry_count: 3 }, category: 'output' },
  alarm_out:     { label: 'Alarm',         color: '#f87171', icon: 'M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0zM12 9v4M12 17h.01', desc: 'Trigger an alarm', defaultConfig: { severity: 'critical', source_key: 'flow_alert', message: 'Alert triggered' }, category: 'output' },
}

export const TRIGGER_META: Record<string, { label: string; color: string; icon: string }> = {
  modbus: { label: 'Modbus', color: '#22d3ee', icon: 'M13 2L3 14h9l-1 8 10-12h-9l1-8z' },
  timer:  { label: 'Timer',  color: '#fbbf24', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
  mqtt:   { label: 'MQTT',   color: '#c084fc', icon: 'M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071a9 9 0 0112.728 0M1.394 9.393a13 13 0 0121.212 0' },
}

export const TRIGGER_DEFAULTS: Record<string, Record<string, any>> = {
  modbus: { type: 'modbus', device_id: '*', metric: '', debounce_ms: 1000, on_change_only: false },
  timer:  { type: 'timer', interval_ms: 5000 },
  mqtt:   { type: 'mqtt', topic: 'sensors/+/temp', debounce_ms: 1000 },
}

export const PALETTE_CATEGORIES = [
  { key: 'input', label: 'INPUT', items: Object.entries(TRIGGER_META).map(([type, meta]) => ({ type, ...meta, isTrigger: true })) },
  { key: 'process', label: 'PROCESS', items: Object.entries(NODE_META).filter(([,m]) => m.category === 'process').map(([type, meta]) => ({ type, ...meta, isTrigger: false })) },
  { key: 'output', label: 'OUTPUT', items: Object.entries(NODE_META).filter(([,m]) => m.category === 'output').map(([type, meta]) => ({ type, ...meta, isTrigger: false })) },
]
```

- [ ] **Step 2: Remove inline constants from FlowManager.tsx**

Delete lines 6-28 (the `NODE_META`, `TRIGGER_META`, `TRIGGER_DEFAULTS` definitions) from `client/src/FlowManager.tsx`.

Add import at top:
```typescript
import { NODE_META, TRIGGER_META, TRIGGER_DEFAULTS } from './components/flow-editor/constants'
```

- [ ] **Step 3: Build to verify**

Run: `cd client && npm run build`
Expected: Builds successfully (FlowManager still compiles, just moved constants).

- [ ] **Step 4: Commit**

```bash
git add client/src/components/flow-editor/constants.ts client/src/FlowManager.tsx
git commit -m "refactor(flow): extract node meta constants to shared file"
```

---

### Task 2: Build PaletteSidebar component

**Files:**
- Create: `client/src/components/flow-editor/PaletteSidebar.tsx`

**Context:** Left sidebar showing INPUT, PROCESS, OUTPUT categories. Each item is draggable (mouse down starts drag). Shows color dot, icon, label.

- [ ] **Step 1: Create PaletteSidebar.tsx**

```typescript
// client/src/components/flow-editor/PaletteSidebar.tsx
import { useState } from 'react'
import { PALETTE_CATEGORIES } from './constants'
import type { FlowNodeType } from '../../types'

interface Props {
  onDragStart: (type: FlowNodeType, isTrigger: boolean) => void
}

export function PaletteSidebar({ onDragStart }: Props) {
  const [expanded, setExpanded] = useState<Record<string, boolean>>({
    input: true, process: true, output: true,
  })

  return (
    <div style={{
      width: 220, flexShrink: 0, borderRight: '1px solid var(--border)',
      background: 'var(--card)', display: 'flex', flexDirection: 'column', overflow: 'auto',
    }}>
      <div style={{ padding: '14px 16px', borderBottom: '1px solid var(--border)' }}>
        <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1, color: 'var(--muted)' }}>Palette</div>
      </div>
      {PALETTE_CATEGORIES.map(cat => (
        <div key={cat.key} style={{ borderBottom: '1px solid var(--border)' }}>
          <button
            style={{
              width: '100%', padding: '10px 16px', background: 'none', border: 'none',
              color: 'var(--fg)', fontSize: 11, fontWeight: 700, textTransform: 'uppercase',
              letterSpacing: 0.5, cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'space-between',
            }}
            onClick={() => setExpanded(e => ({ ...e, [cat.key]: !e[cat.key] }))}
          >
            {cat.label}
            <span style={{ color: 'var(--muted)', fontSize: 10 }}>{expanded[cat.key] ? '▼' : '▶'}</span>
          </button>
          {expanded[cat.key] && (
            <div style={{ padding: '4px 12px 12px' }}>
              {cat.items.map(item => (
                <div
                  key={item.type}
                  style={{
                    display: 'flex', alignItems: 'center', gap: 10, padding: '8px 10px',
                    borderRadius: 6, cursor: 'grab', userSelect: 'none',
                    transition: 'background .15s',
                  }}
                  onMouseDown={() => onDragStart(item.type as FlowNodeType, item.isTrigger)}
                  onMouseEnter={e => { e.currentTarget.style.background = 'var(--bg)' }}
                  onMouseLeave={e => { e.currentTarget.style.background = 'transparent' }}
                >
                  <span style={{ width: 8, height: 8, borderRadius: '50%', background: item.color, flexShrink: 0 }} />
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={item.color} strokeWidth="2">
                    <path d={item.icon} />
                  </svg>
                  <span style={{ fontSize: 12, fontWeight: 600 }}>{item.label}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}
```

- [ ] **Step 2: Build to verify**

Run: `cd client && npm run build`
Expected: Builds successfully (no imports of PaletteSidebar yet, so just verify no TS errors).

- [ ] **Step 3: Commit**

```bash
git add client/src/components/flow-editor/PaletteSidebar.tsx
git commit -m "feat(flow): add PaletteSidebar component with categorized nodes"
```

---

### Task 3: Build DragGhost component

**Files:**
- Create: `client/src/components/flow-editor/DragGhost.tsx`

**Context:** Semi-transparent ghost that follows cursor during drag. Shows node icon + label.

- [ ] **Step 1: Create DragGhost.tsx**

```typescript
// client/src/components/flow-editor/DragGhost.tsx
import { NODE_META, TRIGGER_META } from './constants'
import type { FlowNodeType } from '../../types'

interface Props {
  type: FlowNodeType | string
  isTrigger: boolean
  x: number
  y: number
}

export function DragGhost({ type, isTrigger, x, y }: Props) {
  const meta = isTrigger ? TRIGGER_META[type] : NODE_META[type as FlowNodeType]
  if (!meta) return null

  return (
    <div style={{
      position: 'fixed', left: x + 12, top: y + 12, pointerEvents: 'none', zIndex: 9999,
      opacity: 0.85, transform: 'scale(1.05) rotate(2deg)',
    }}>
      <div style={{
        width: 160, borderRadius: 8, border: `2px solid ${meta.color}`,
        background: 'var(--card)', padding: '10px 14px',
        boxShadow: `0 8px 24px rgba(0,0,0,.4), 0 0 0 1px ${meta.color}44`,
        display: 'flex', alignItems: 'center', gap: 8,
      }}>
        <span style={{ width: 8, height: 8, borderRadius: '50%', background: meta.color, flexShrink: 0 }} />
        <span style={{ fontSize: 12, fontWeight: 700, color: 'var(--fg)' }}>{meta.label}</span>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Build to verify**

Run: `cd client && npm run build`
Expected: Builds successfully.

- [ ] **Step 3: Commit**

```bash
git add client/src/components/flow-editor/DragGhost.tsx
git commit -m "feat(flow): add DragGhost component for drag preview"
```

---

### Task 4: Build FlowNode component

**Files:**
- Create: `client/src/components/flow-editor/FlowNode.tsx`

**Context:** Renders a single node card. Trigger nodes have diamond shape styling. Process/output nodes are rectangular. Shows colored header bar, icon, label, mini config preview. Supports drag-to-reorder via mousedown on header.

- [ ] **Step 1: Create FlowNode.tsx**

```typescript
// client/src/components/flow-editor/FlowNode.tsx
import { NODE_META, TRIGGER_META, NODE_WIDTH } from './constants'
import type { FlowNodeConfig } from '../../types'

interface Props {
  node?: FlowNodeConfig
  triggerType?: string
  isTrigger: boolean
  isSelected: boolean
  index: number
  onSelect: () => void
  onRemove?: () => void
  onDragStart?: (index: number) => void
}

export function FlowNode({ node, triggerType, isTrigger, isSelected, index, onSelect, onRemove, onDragStart }: Props) {
  const meta = isTrigger ? TRIGGER_META[triggerType || 'modbus'] : NODE_META[node!.type]
  const color = meta?.color || '#9ca3af'
  const label = meta?.label || node?.type || triggerType || 'Unknown'

  const cardStyle: React.CSSProperties = {
    width: NODE_WIDTH, borderRadius: isTrigger ? 4 : 10,
    border: `1px solid ${isSelected ? color : 'var(--border)'}`,
    background: 'var(--card)', overflow: 'hidden', cursor: 'pointer',
    boxShadow: isSelected ? `0 0 0 2px ${color}33, 0 4px 12px rgba(0,0,0,.2)` : '0 2px 8px rgba(0,0,0,.15)',
    transition: 'box-shadow .15s, border-color .15s, transform .15s',
    transform: isTrigger ? 'rotate(45deg)' : 'none',
  }

  const innerStyle: React.CSSProperties = {
    padding: '12px 14px',
    transform: isTrigger ? 'rotate(-45deg)' : 'none',
  }

  // For trigger, we need a wrapper to maintain diamond shape without affecting layout flow
  if (isTrigger) {
    return (
      <div style={{ width: NODE_WIDTH + 20, height: NODE_WIDTH + 20, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <div style={cardStyle} onClick={onSelect}>
          <div style={{ height: 4, background: color }} />
          <div style={innerStyle}>
            <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: 0.5, fontWeight: 600, marginBottom: 4 }}>Trigger</div>
            <div style={{ fontSize: 12, fontWeight: 700, display: 'flex', alignItems: 'center', gap: 6 }}>
              <span style={{ width: 8, height: 8, borderRadius: '50%', background: color }} />
              {label}
            </div>
          </div>
        </div>
      </div>
    )
  }

  // Mini config preview value
  const preview = node ? (
    node.condition || node.field || node.output_field || node.url || node.message ||
    (node.map ? JSON.stringify(node.map).slice(0, 30) : '') ||
    (node.table ? JSON.stringify(node.table).slice(0, 30) : '') ||
    ''
  ) : ''

  return (
    <div style={{ width: NODE_WIDTH }}>
      <div
        style={cardStyle}
        onClick={onSelect}
      >
        {/* Drag handle header */}
        <div
          style={{ height: 32, background: `${color}15`, borderBottom: `1px solid ${color}30`, display: 'flex', alignItems: 'center', gap: 8, padding: '0 12px', cursor: onDragStart ? 'grab' : 'pointer' }}
          onMouseDown={e => {
            if (onDragStart) {
              e.stopPropagation()
              onDragStart(index)
            }
          }}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2">
            <path d={(meta as any).icon} />
          </svg>
          <span style={{ fontSize: 11, fontWeight: 700, flex: 1 }}>{label}</span>
          {onRemove && (
            <button
              style={{ background: 'none', border: 'none', color: 'var(--crit)', cursor: 'pointer', fontSize: 16, padding: '0 2px', lineHeight: 1 }}
              onClick={e => { e.stopPropagation(); onRemove() }}
              title="Remove"
            >&times;</button>
          )}
        </div>
        <div style={{ padding: '10px 12px' }}>
          <div style={{ fontSize: 10, color: 'var(--muted)', marginBottom: 6, lineHeight: 1.4 }}>{(meta as any).desc}</div>
          {preview && (
            <div style={{ fontSize: 10, color: 'var(--muted)', fontFamily: "'JetBrains Mono', monospace", whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {preview}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Build to verify**

Run: `cd client && npm run build`
Expected: Builds successfully.

- [ ] **Step 3: Commit**

```bash
git add client/src/components/flow-editor/FlowNode.tsx
git commit -m "feat(flow): add FlowNode component with trigger diamond shape"
```

---

### Task 5: Build ConfigPanel component

**Files:**
- Create: `client/src/components/flow-editor/ConfigPanel.tsx`

**Context:** Right sidebar. Shows form fields for selected node or trigger. Uses the existing NodeForm logic but as a standalone panel.

- [ ] **Step 1: Create ConfigPanel.tsx**

```typescript
// client/src/components/flow-editor/ConfigPanel.tsx
import { NODE_META, TRIGGER_META, TRIGGER_DEFAULTS } from './constants'
import type { FlowNodeConfig } from '../../types'

interface Props {
  node: FlowNodeConfig | null
  trigger: any
  onNodeChange: (key: string, value: any) => void
  onTriggerChange: (key: string, value: any) => void
  onTriggerTypeChange: (type: string) => void
}

export function ConfigPanel({ node, trigger, onNodeChange, onTriggerChange, onTriggerTypeChange }: Props) {
  const inputStyle: React.CSSProperties = {
    background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6,
    padding: '6px 10px', color: 'var(--fg)', fontSize: 12,
    fontFamily: "'JetBrains Mono', monospace", width: '100%',
  }
  const labelStyle: React.CSSProperties = {
    fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase',
    letterSpacing: 0.5, marginBottom: 4, display: 'block', fontWeight: 600,
  }

  const isTrigger = !node
  const meta = isTrigger ? TRIGGER_META[trigger?.type || 'modbus'] : NODE_META[node!.type]
  const color = meta?.color || '#9ca3af'
  const label = meta?.label || (isTrigger ? trigger?.type : node?.type) || 'Unknown'

  return (
    <div style={{
      width: 300, flexShrink: 0, borderLeft: '1px solid var(--border)',
      background: 'var(--card)', overflow: 'auto', display: 'flex', flexDirection: 'column',
    }}>
      <div style={{ padding: '14px 16px', borderBottom: '1px solid var(--border)' }}>
        <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1, color: 'var(--muted)' }}>Configuration</div>
      </div>

      <div style={{ padding: '16px', display: 'flex', flexDirection: 'column', gap: 14 }}>
        {/* Header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
          <span style={{ width: 8, height: 8, borderRadius: '50%', background: color }} />
          <span style={{ fontSize: 13, fontWeight: 700 }}>{label}</span>
        </div>

        {isTrigger ? (
          <>
            <div>
              <label style={labelStyle}>Trigger Type</label>
              <select style={inputStyle} value={trigger?.type || 'modbus'} onChange={e => onTriggerTypeChange(e.target.value)}>
                <option value="modbus">Modbus</option>
                <option value="timer">Timer</option>
                <option value="mqtt">MQTT</option>
              </select>
            </div>
            {trigger?.type === 'modbus' && (
              <>
                <div>
                  <label style={labelStyle}>Device ID</label>
                  <input style={inputStyle} value={trigger.device_id || ''} onChange={e => onTriggerChange('device_id', e.target.value)} />
                </div>
                <div>
                  <label style={labelStyle}>Metric (optional)</label>
                  <input style={inputStyle} value={trigger.metric || ''} onChange={e => onTriggerChange('metric', e.target.value)} />
                </div>
                <div>
                  <label style={labelStyle}>Debounce (ms)</label>
                  <input style={inputStyle} type="number" value={trigger.debounce_ms || 1000} onChange={e => onTriggerChange('debounce_ms', Number(e.target.value))} />
                </div>
                <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12, cursor: 'pointer' }}>
                  <input type="checkbox" checked={trigger.on_change_only || false} onChange={e => onTriggerChange('on_change_only', e.target.checked)} />
                  On change only
                </label>
              </>
            )}
            {trigger?.type === 'timer' && (
              <div>
                <label style={labelStyle}>Interval (ms)</label>
                <input style={inputStyle} type="number" value={trigger.interval_ms || 5000} onChange={e => onTriggerChange('interval_ms', Number(e.target.value))} />
              </div>
            )}
            {trigger?.type === 'mqtt' && (
              <>
                <div>
                  <label style={labelStyle}>Topic</label>
                  <input style={inputStyle} value={trigger.topic || ''} onChange={e => onTriggerChange('topic', e.target.value)} />
                </div>
                <div>
                  <label style={labelStyle}>Debounce (ms)</label>
                  <input style={inputStyle} type="number" value={trigger.debounce_ms || 1000} onChange={e => onTriggerChange('debounce_ms', Number(e.target.value))} />
                </div>
              </>
            )}
          </>
        ) : (
          <NodeFormFields node={node!} onChange={onNodeChange} inputStyle={inputStyle} labelStyle={labelStyle} />
        )}
      </div>
    </div>
  )
}

function NodeFormFields({ node, onChange, inputStyle, labelStyle }: {
  node: FlowNodeConfig
  onChange: (k: string, v: any) => void
  inputStyle: React.CSSProperties
  labelStyle: React.CSSProperties
}) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      {Object.entries(node).filter(([k]) => k !== 'id' && k !== 'type').map(([key, value]) => {
        if (typeof value === 'boolean') {
          return (
            <label key={key} style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12, cursor: 'pointer' }}>
              <input type="checkbox" checked={value} onChange={e => onChange(key, e.target.checked)} />
              {key}
            </label>
          )
        }
        if (typeof value === 'number') {
          return (
            <div key={key}>
              <label style={labelStyle}>{key}</label>
              <input style={inputStyle} type="number" value={value} onChange={e => onChange(key, Number(e.target.value))} />
            </div>
          )
        }
        if (typeof value === 'object') {
          return (
            <div key={key}>
              <label style={labelStyle}>{key}</label>
              <textarea
                style={{ ...inputStyle, minHeight: 80, resize: 'vertical' }}
                value={JSON.stringify(value, null, 2)}
                onChange={e => { try { onChange(key, JSON.parse(e.target.value)) } catch { onChange(key, e.target.value) } }}
              />
            </div>
          )
        }
        return (
          <div key={key}>
            <label style={labelStyle}>{key}</label>
            <input style={inputStyle} value={value} onChange={e => onChange(key, e.target.value)} />
          </div>
        )
      })}
    </div>
  )
}
```

- [ ] **Step 2: Build to verify**

Run: `cd client && npm run build`
Expected: Builds successfully.

- [ ] **Step 3: Commit**

```bash
git add client/src/components/flow-editor/ConfigPanel.tsx
git commit -m "feat(flow): add ConfigPanel component for node/trigger editing"
```

---

### Task 6: Build CanvasArea with auto-layout and SVG connections

**Files:**
- Create: `client/src/components/flow-editor/CanvasArea.tsx`

**Context:** Center canvas. Contains:
- Dot grid background (SVG pattern)
- Auto-laid-out nodes (trigger first, then nodes left-to-right)
- SVG bezier connections between adjacent nodes
- Drop zone for palette drag
- Reorder drag support on node headers

- [ ] **Step 1: Create CanvasArea.tsx**

```typescript
// client/src/components/flow-editor/CanvasArea.tsx
import { useRef, useMemo, useEffect, useState } from 'react'
import { FlowNode } from './FlowNode'
import { NODE_WIDTH, NODE_GAP, NODE_HEIGHT } from './constants'
import type { FlowNodeConfig } from '../../types'

interface Props {
  trigger: any
  nodes: FlowNodeConfig[]
  selectedIndex: number | null // null = trigger selected, >=0 = node index
  onSelectTrigger: () => void
  onSelectNode: (index: number) => void
  onRemoveNode: (index: number) => void
  onDragNodeStart: (index: number) => void
  onDrop: (insertIndex: number) => void
  isDragOver: boolean
}

export function CanvasArea({
  trigger, nodes, selectedIndex,
  onSelectTrigger, onSelectNode, onRemoveNode, onDragNodeStart, onDrop, isDragOver,
}: Props) {
  const canvasRef = useRef<HTMLDivElement>(null)
  const [canvasSize, setCanvasSize] = useState({ width: 800, height: 400 })

  useEffect(() => {
    const el = canvasRef.current
    if (!el) return
    const ro = new ResizeObserver(entries => {
      const cr = entries[0].contentRect
      setCanvasSize({ width: cr.width, height: cr.height })
    })
    ro.observe(el)
    return () => ro.disconnect()
  }, [])

  // Auto-layout: trigger + nodes, centered horizontally
  const positions = useMemo(() => {
    const totalItems = 1 + nodes.length
    const totalWidth = totalItems * NODE_WIDTH + (totalItems - 1) * NODE_GAP
    const startX = Math.max(CANVAS_PADDING, (canvasSize.width - totalWidth) / 2)
    const y = canvasSize.height / 2 - NODE_HEIGHT / 2

    const pos: { x: number; y: number }[] = []
    for (let i = 0; i < totalItems; i++) {
      pos.push({ x: startX + i * (NODE_WIDTH + NODE_GAP), y })
    }
    return pos
  }, [nodes.length, canvasSize.width, canvasSize.height])

  // Connection points: right edge of item i to left edge of item i+1
  const edges = useMemo(() => {
    const result: { from: { x: number; y: number }; to: { x: number; y: number } }[] = []
    for (let i = 0; i < positions.length - 1; i++) {
      const from = { x: positions[i].x + NODE_WIDTH, y: positions[i].y + NODE_HEIGHT / 2 }
      const to = { x: positions[i + 1].x, y: positions[i + 1].y + NODE_HEIGHT / 2 }
      result.push({ from, to })
    }
    return result
  }, [positions])

  const handleMouseUp = (e: React.MouseEvent) => {
    if (!isDragOver) return
    const rect = canvasRef.current?.getBoundingClientRect()
    if (!rect) return
    const x = e.clientX - rect.left

    // Find nearest insertion gap
    const totalItems = 1 + nodes.length
    let bestIndex = totalItems
    let bestDist = Infinity

    // Check gaps: before trigger (0), between items, after last (totalItems)
    for (let i = 0; i <= totalItems; i++) {
      const gapX = i === 0
        ? positions[0]?.x - NODE_GAP / 2 || 0
        : i === totalItems
          ? (positions[totalItems - 1]?.x + NODE_WIDTH + NODE_GAP / 2) || canvasSize.width
          : (positions[i - 1].x + NODE_WIDTH + positions[i].x) / 2
      const dist = Math.abs(x - gapX)
      if (dist < bestDist) {
        bestDist = dist
        bestIndex = i
      }
    }
    onDrop(bestIndex)
  }

  return (
    <div
      ref={canvasRef}
      style={{
        flex: 1, position: 'relative', overflow: 'hidden',
        background: 'var(--bg)',
      }}
      onMouseUp={handleMouseUp}
    >
      {/* Dot grid background */}
      <svg style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', pointerEvents: 'none' }}>
        <defs>
          <pattern id="dotGrid" width="20" height="20" patternUnits="userSpaceOnUse">
            <circle cx="1" cy="1" r="1" fill="var(--border)" opacity="0.15" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#dotGrid)" />
      </svg>

      {/* Drop zone indicator */}
      {isDragOver && (
        <div style={{
          position: 'absolute', inset: 0,
          border: '2px dashed var(--green)', background: 'rgba(34,197,94,.05)',
          pointerEvents: 'none', zIndex: 5,
        }} />
      )}

      {/* SVG Connections */}
      <svg style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', pointerEvents: 'none', zIndex: 1 }}>
        <defs>
          <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
            <polygon points="0 0, 8 3, 0 6" fill="#4b5563" />
          </marker>
        </defs>
        {edges.map((edge, i) => {
          const mx = (edge.from.x + edge.to.x) / 2
          const d = `M ${edge.from.x} ${edge.from.y} C ${mx} ${edge.from.y}, ${mx} ${edge.to.y}, ${edge.to.x} ${edge.to.y}`
          return <path key={i} d={d} fill="none" stroke="#4b5563" strokeWidth={2} markerEnd="url(#arrowhead)" />
        })}
      </svg>

      {/* Nodes */}
      {positions.map((pos, i) => {
        const isTriggerNode = i === 0
        const nodeIndex = i - 1
        return (
          <div
            key={isTriggerNode ? 'trigger' : nodes[nodeIndex].id}
            style={{
              position: 'absolute', left: pos.x, top: pos.y, zIndex: 2,
            }}
          >
            {isTriggerNode ? (
              <FlowNode
                isTrigger={true}
                triggerType={trigger?.type}
                isSelected={selectedIndex === null}
                index={0}
                onSelect={onSelectTrigger}
              />
            ) : (
              <FlowNode
                node={nodes[nodeIndex]}
                isTrigger={false}
                isSelected={selectedIndex === nodeIndex}
                index={nodeIndex}
                onSelect={() => onSelectNode(nodeIndex)}
                onRemove={() => onRemoveNode(nodeIndex)}
                onDragStart={onDragNodeStart}
              />
            )}
          </div>
        )
      })}
    </div>
  )
}

const CANVAS_PADDING = 60
```

- [ ] **Step 2: Build to verify**

Run: `cd client && npm run build`
Expected: Builds successfully.

- [ ] **Step 3: Commit**

```bash
git add client/src/components/flow-editor/CanvasArea.tsx
git commit -m "feat(flow): add CanvasArea with auto-layout, SVG edges, dot grid"
```

---

### Task 7: Build new FlowEditor orchestrator

**Files:**
- Create: `client/src/components/flow-editor/FlowEditor.tsx`
- Delete: Old FlowEditor logic from `client/src/FlowManager.tsx` (will be replaced)

**Context:** The new FlowEditor replaces the old inline editor in FlowManager.tsx. It coordinates PaletteSidebar, CanvasArea, ConfigPanel, DragGhost, toolbar, and test panel.

- [ ] **Step 1: Create FlowEditor.tsx**

```typescript
// client/src/components/flow-editor/FlowEditor.tsx
import { useState, useMemo, useRef, useEffect, useCallback } from 'react'
import { PaletteSidebar } from './PaletteSidebar'
import { CanvasArea } from './CanvasArea'
import { ConfigPanel } from './ConfigPanel'
import { DragGhost } from './DragGhost'
import { TRIGGER_DEFAULTS } from './constants'
import { useTestFlow } from '../../queries'
import type { Flow, FlowNodeConfig, FlowNodeType } from '../../types'
import { btnStyle } from '../../styles'

interface Props {
  flow: Flow
  onSave: (f: Flow) => void
  onCancel: () => void
}

export function FlowEditor({ flow, onSave, onCancel }: Props) {
  const [form, setForm] = useState<Flow>({ ...flow })
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null) // null = trigger, >=0 = node
  const [error, setError] = useState('')

  // Drag state
  const [dragState, setDragState] = useState<{
    type: 'palette' | 'reorder'
    nodeType?: FlowNodeType | string
    isTrigger?: boolean
    fromIndex?: number
  } | null>(null)
  const [ghostPos, setGhostPos] = useState({ x: 0, y: 0 })

  const [testPayload, setTestPayload] = useState('{"metrics": {"temp": 85}}')
  const [testResult, setTestResult] = useState<any>(null)
  const [testLoading, setTestLoading] = useState(false)
  const testFlow = useTestFlow()

  const parsedNodes = useMemo(() => {
    try { return JSON.parse(form.nodes) as FlowNodeConfig[] } catch { return [] }
  }, [form.nodes])

  const parsedTrigger = useMemo(() => {
    try { return JSON.parse(form.trigger_config) as any } catch { return { type: 'modbus' } }
  }, [form.trigger_config])

  // Document-level mouse move/up for drag
  useEffect(() => {
    if (!dragState) return
    const handleMove = (e: MouseEvent) => setGhostPos({ x: e.clientX, y: e.clientY })
    const handleUp = () => setDragState(null)
    document.addEventListener('mousemove', handleMove)
    document.addEventListener('mouseup', handleUp)
    return () => {
      document.removeEventListener('mousemove', handleMove)
      document.removeEventListener('mouseup', handleUp)
    }
  }, [dragState])

  const handleSave = () => {
    setError('')
    try { JSON.parse(form.trigger_config); JSON.parse(form.nodes) } catch { setError('Invalid JSON'); return }
    if (!form.id || !form.name) { setError('ID and Name required'); return }
    onSave({ ...form, updated_at: Math.floor(Date.now() / 1000) })
  }

  const addNode = useCallback((type: FlowNodeType | string, isTrigger: boolean, insertIndex: number) => {
    if (isTrigger) {
      // Replace trigger
      setForm(f => ({ ...f, trigger_config: JSON.stringify(TRIGGER_DEFAULTS[type] || { type }, null, 2) }))
      setSelectedIndex(null)
    } else {
      const meta = (await import('./constants')).NODE_META[type as FlowNodeType]
      if (!meta) return
      const newNode: FlowNodeConfig = { id: `n${Date.now()}`, type: type as FlowNodeType, ...meta.defaultConfig }
      const idx = Math.min(insertIndex, parsedNodes.length)
      const updated = [...parsedNodes.slice(0, idx), newNode, ...parsedNodes.slice(idx)]
      setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
      setSelectedIndex(idx)
    }
  }, [parsedNodes])

  // Handle drop from palette
  const handlePaletteDrop = useCallback((insertIndex: number) => {
    if (!dragState || dragState.type !== 'palette' || !dragState.nodeType) return
    addNode(dragState.nodeType, !!dragState.isTrigger, insertIndex)
    setDragState(null)
  }, [dragState, addNode])

  // Handle reorder drop
  const handleReorderDrop = useCallback((insertIndex: number) => {
    if (!dragState || dragState.type !== 'reorder' || dragState.fromIndex === undefined) return
    const from = dragState.fromIndex
    if (from === insertIndex || from + 1 === insertIndex) {
      setDragState(null)
      return
    }
    const item = parsedNodes[from]
    const without = parsedNodes.filter((_, i) => i !== from)
    const targetIdx = insertIndex > from ? insertIndex - 1 : insertIndex
    const updated = [...without.slice(0, targetIdx), item, ...without.slice(targetIdx)]
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
    setSelectedIndex(targetIdx)
    setDragState(null)
  }, [dragState, parsedNodes])

  const handleDrop = useCallback((insertIndex: number) => {
    if (!dragState) return
    if (dragState.type === 'palette') handlePaletteDrop(insertIndex)
    else if (dragState.type === 'reorder') handleReorderDrop(insertIndex)
  }, [dragState, handlePaletteDrop, handleReorderDrop])

  const removeNode = (index: number) => {
    const updated = parsedNodes.filter((_, i) => i !== index)
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
    if (selectedIndex === index) setSelectedIndex(null)
    else if (selectedIndex !== null && selectedIndex > index) setSelectedIndex(selectedIndex - 1)
  }

  const updateNode = (index: number, key: string, value: any) => {
    const updated = parsedNodes.map((n, i) => i === index ? { ...n, [key]: value } : n)
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
  }

  const updateTrigger = (key: string, value: any) => {
    const updated = { ...parsedTrigger, [key]: value }
    setForm(f => ({ ...f, trigger_config: JSON.stringify(updated, null, 2) }))
  }

  const handleTriggerTypeChange = (type: string) => {
    setForm(f => ({ ...f, trigger_config: JSON.stringify(TRIGGER_DEFAULTS[type] || { type }, null, 2) }))
  }

  const handleTest = async () => {
    if (!form.id) return
    setTestLoading(true)
    setTestResult(null)
    try {
      const payload = JSON.parse(testPayload)
      const result = await testFlow.mutateAsync({ id: form.id, payload })
      setTestResult(result)
    } catch (e: any) {
      setTestResult({ error: e.message })
    }
    setTestLoading(false)
  }

  const selectedNode = selectedIndex !== null ? parsedNodes[selectedIndex] : null

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Toolbar */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 20px', borderBottom: '1px solid var(--border)', flexShrink: 0, background: 'var(--card)' }}>
        <button style={btnStyle} onClick={onCancel}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        </button>
        <div style={{ flex: 1 }}>
          <input
            style={{ background: 'transparent', border: 'none', color: 'var(--fg)', fontSize: 16, fontWeight: 700, padding: 0, fontFamily: 'inherit', width: '100%' }}
            value={form.name}
            onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
            placeholder="Flow name"
          />
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontSize: 12, color: 'var(--muted)' }}>{form.id}</span>
          <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
          <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, cursor: 'pointer' }}>
            <input type="checkbox" checked={form.enabled} onChange={e => setForm(f => ({ ...f, enabled: e.target.checked }))} />
            Enabled
          </label>
          <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={handleSave}>Save</button>
        </div>
      </div>

      {error && (
        <div style={{ background: 'rgba(239,68,68,.1)', borderBottom: '1px solid var(--crit)', color: 'var(--crit)', padding: '10px 24px', fontSize: 13, flexShrink: 0 }}>
          {error}
        </div>
      )}

      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        <PaletteSidebar onDragStart={(type, isTrigger) => {
          setDragState({ type: 'palette', nodeType: type, isTrigger })
        }} />

        <CanvasArea
          trigger={parsedTrigger}
          nodes={parsedNodes}
          selectedIndex={selectedIndex}
          onSelectTrigger={() => setSelectedIndex(null)}
          onSelectNode={setSelectedIndex}
          onRemoveNode={removeNode}
          onDragNodeStart={(index) => setDragState({ type: 'reorder', fromIndex: index })}
          onDrop={handleDrop}
          isDragOver={!!dragState}
        />

        <ConfigPanel
          node={selectedNode}
          trigger={parsedTrigger}
          onNodeChange={(k, v) => selectedIndex !== null && updateNode(selectedIndex, k, v)}
          onTriggerChange={updateTrigger}
          onTriggerTypeChange={handleTriggerTypeChange}
        />
      </div>

      {/* Test panel (collapsible, below main area) */}
      <div style={{ borderTop: '1px solid var(--border)', background: 'var(--card)', flexShrink: 0 }}>
        <details style={{ padding: '0 20px' }}>
          <summary style={{ padding: '10px 0', cursor: 'pointer', fontSize: 12, fontWeight: 600, color: 'var(--muted)', userSelect: 'none' }}>Test Flow</summary>
          <div style={{ padding: '0 0 16px', display: 'flex', gap: 16, alignItems: 'flex-start' }}>
            <div style={{ flex: 1 }}>
              <label style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: 0.5, marginBottom: 4, display: 'block', fontWeight: 600 }}>Payload JSON</label>
              <textarea
                style={{ background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6, padding: '8px 10px', color: 'var(--fg)', fontSize: 12, fontFamily: "'JetBrains Mono', monospace", width: '100%', minHeight: 80, resize: 'vertical' }}
                value={testPayload}
                onChange={e => setTestPayload(e.target.value)}
              />
            </div>
            <div>
              <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={handleTest} disabled={testLoading || !form.id}>
                {testLoading ? 'Running...' : 'Run Test'}
              </button>
            </div>
            {testResult && (
              <div style={{ flex: 1, background: 'var(--bg)', borderRadius: 8, padding: 12, fontSize: 11, fontFamily: "'JetBrains Mono', monospace", overflow: 'auto', maxHeight: 120 }}>
                {testResult.error ? (
                  <div style={{ color: 'var(--crit)' }}>{testResult.error}</div>
                ) : (
                  <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>{JSON.stringify(testResult, null, 2)}</pre>
                )}
              </div>
            )}
          </div>
        </details>
      </div>

      {/* Drag ghost overlay */}
      {dragState && dragState.nodeType && (
        <DragGhost
          type={dragState.nodeType}
          isTrigger={!!dragState.isTrigger}
          x={ghostPos.x}
          y={ghostPos.y}
        />
      )}
    </div>
  )
}
```

- [ ] **Step 2: Build to verify**

Run: `cd client && npm run build`
Expected: May have errors — the `addNode` callback uses dynamic import which is wrong. Fix it.

- [ ] **Step 3: Fix dynamic import in addNode**

In FlowEditor.tsx, replace the `addNode` callback:

```typescript
  const addNode = useCallback((type: FlowNodeType | string, isTrigger: boolean, insertIndex: number) => {
    if (isTrigger) {
      setForm(f => ({ ...f, trigger_config: JSON.stringify(TRIGGER_DEFAULTS[type] || { type }, null, 2) }))
      setSelectedIndex(null)
    } else {
      // Import NODE_META at top of file instead
      const meta = NODE_META[type as FlowNodeType]
      if (!meta) return
      const newNode: FlowNodeConfig = { id: `n${Date.now()}`, type: type as FlowNodeType, ...meta.defaultConfig }
      const idx = Math.min(insertIndex, parsedNodes.length)
      const updated = [...parsedNodes.slice(0, idx), newNode, ...parsedNodes.slice(idx)]
      setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
      setSelectedIndex(idx)
    }
  }, [parsedNodes])
```

Also add `NODE_META` to the import from `./constants`.

- [ ] **Step 4: Rebuild**

Run: `cd client && npm run build`
Expected: Builds successfully.

- [ ] **Step 5: Commit**

```bash
git add client/src/components/flow-editor/FlowEditor.tsx
git commit -m "feat(flow): add FlowEditor orchestrator with DnD and config panel"
```

---

### Task 8: Update FlowManager.tsx to use new FlowEditor

**Files:**
- Modify: `client/src/FlowManager.tsx`

**Context:** Replace the old inline FlowEditor with imports from the new component. Keep FlowList unchanged.

- [ ] **Step 1: Replace FlowEditor in FlowManager.tsx**

Replace the entire `FlowEditor` function (lines 125-454) with an import:

```typescript
import { FlowEditor } from './components/flow-editor/FlowEditor'
```

Also remove the old `NodeAddButton` and `NodeForm` functions (lines 456-565) since they're now in separate files.

The file should now only contain:
1. Imports
2. `FlowList` function (keep as-is)
3. `FlowManager` export function (keep as-is, but FlowEditor is now imported)

- [ ] **Step 2: Verify imports are correct**

Ensure FlowManager.tsx imports:
```typescript
import { useState } from 'react'
import { useFlows, useCreateFlow, useUpdateFlow, useDeleteFlow, useStartFlow, useStopFlow } from './queries'
import type { Flow } from './types'
import { btnStyle } from './styles'
import { FlowEditor } from './components/flow-editor/FlowEditor'
import { TRIGGER_DEFAULTS } from './components/flow-editor/constants'
```

Remove unused imports (`useTestFlow`, `FlowNodeConfig`, `FlowNodeType`, etc. if not used).

- [ ] **Step 3: Build to verify**

Run: `cd client && npm run build`
Expected: Builds successfully with no TS errors.

- [ ] **Step 4: Commit**

```bash
git add client/src/FlowManager.tsx
git commit -m "refactor(flow): replace inline FlowEditor with new component"
```

---

### Task 9: Final build and integration test

- [ ] **Step 1: Full build**

```bash
cd client && npm run build
```
Expected: Builds successfully with 0 errors.

- [ ] **Step 2: Verify file structure**

```bash
ls -la client/src/components/flow-editor/
```
Expected: 5 files + constants.ts = 6 files total.

- [ ] **Step 3: Commit any final fixes**

```bash
git add -A
git commit -m "feat(flow): Node-RED style editor with palette, canvas, and config panel"
```

- [ ] **Step 4: Bump version and push**

```bash
# In Cargo.toml, change version from "0.17.3" to "0.17.4"
```

```bash
git add Cargo.toml && git commit -m "chore: bump version to 0.17.4"
git push origin master
```

- [ ] **Step 5: Create release**

```bash
gh release create v0.17.4 --title "v0.17.4 - Node-RED Style Flow Editor" --notes "## What's New

### Frontend
- **Node-RED Style Flow Editor**: Complete visual redesign
  - Left palette with INPUT / PROCESS / OUTPUT categories
  - Drag-and-drop from palette to canvas
  - Drag-to-reorder nodes on canvas
  - Auto-layout with SVG bezier connections
  - Dot grid background
  - Diamond-shaped trigger node
  - Right-side config panel for node/trigger editing
  - Collapsible test panel"
```

---

## Self-Review Checklist

**Spec coverage:**
- [x] Left palette with categories — Task 2 (PaletteSidebar)
- [x] Dot grid background — Task 6 (CanvasArea SVG pattern)
- [x] Auto-layout nodes — Task 6 (positions useMemo)
- [x] SVG bezier connections — Task 6 (edges useMemo + SVG paths)
- [x] Drag from palette — Task 7 (dragState + PaletteSidebar onDragStart)
- [x] Drag to reorder — Task 7 (onDragNodeStart + handleReorderDrop)
- [x] Ghost preview — Task 3 (DragGhost)
- [x] Right config panel — Task 5 (ConfigPanel)
- [x] Trigger as diamond — Task 4 (FlowNode isTrigger branch)
- [x] Test panel — Task 7 (collapsible details element)

**Placeholder scan:**
- [x] No "TBD", "TODO", "implement later"
- [x] All code blocks contain actual code
- [x] No vague "add error handling" steps

**Type consistency:**
- [x] `FlowNodeType` used consistently
- [x] `selectedIndex: number | null` — null means trigger, >=0 means node
- [x] `dragState.type: 'palette' | 'reorder'`
- [x] `addNode` signature matches usage

**Gaps found:** None.

**Missing from spec but included:**
- ResizeObserver for responsive canvas — included in Task 6
- ESC to cancel drag — not explicitly in spec, but dragState clears on mouseup so it's naturally bounded
- Keyboard navigation — spec says Phase 2, not included (YAGNI)
