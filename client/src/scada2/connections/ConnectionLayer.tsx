import React from 'react'
import type { WidgetInstance, ConnectionInstance } from '../widgets/types'
import type { SensorData } from '../../types'
import { portPoint, resolveSymbol } from '../hmi/resolve'
import type { PortSide } from '../hmi/symbols'

interface Props {
  widgets: WidgetInstance[]
  connections: ConnectionInstance[]
  currentValues: Record<string, SensorData>
  editMode: boolean
  selectedId: string | null
  onSelect?: (id: string) => void
  /** Rubber-band line while the connect tool has a source picked. */
  pending?: { fromId: string; x: number; y: number } | null
}

type Pt = [number, number]

const center = (w: WidgetInstance): Pt => [w.x + w.width / 2, w.y + w.height / 2]

/** Side of `w` that faces `toward`, weighted by the box aspect. */
function facingSide(w: WidgetInstance, toward: Pt): PortSide {
  const [cx, cy] = center(w)
  const dx = toward[0] - cx
  const dy = toward[1] - cy
  if (Math.abs(dx) * w.height >= Math.abs(dy) * w.width) return dx >= 0 ? 'right' : 'left'
  return dy >= 0 ? 'bottom' : 'top'
}

/** Orthogonal route between equipment nozzles: straight when aligned, otherwise one dog-leg at the midpoint. */
export function routeConnection(a: WidgetInstance, b: WidgetInstance, waypoints: Pt[]): Pt[] {
  if (waypoints.length > 0) {
    const first = waypoints[0]
    const last = waypoints[waypoints.length - 1]
    return [portPoint(a, facingSide(a, first)), ...waypoints, portPoint(b, facingSide(b, last))]
  }
  const [ax, ay] = center(a)
  const [bx, by] = center(b)
  const horizontal = Math.abs(bx - ax) >= Math.abs(by - ay)
  const sa: PortSide = horizontal ? (bx >= ax ? 'right' : 'left') : (by >= ay ? 'bottom' : 'top')
  const sb: PortSide = horizontal ? (bx >= ax ? 'left' : 'right') : (by >= ay ? 'top' : 'bottom')
  const pa = portPoint(a, sa)
  const pb = portPoint(b, sb)
  if (horizontal) {
    if (Math.abs(pa[1] - pb[1]) < 0.5) return [pa, pb]
    const mx = Math.round((pa[0] + pb[0]) / 2)
    return simplify([pa, [mx, pa[1]], [mx, pb[1]], pb])
  }
  if (Math.abs(pa[0] - pb[0]) < 0.5) return [pa, pb]
  const my = Math.round((pa[1] + pb[1]) / 2)
  return simplify([pa, [pa[0], my], [pb[0], my], pb])
}

function simplify(pts: Pt[]): Pt[] {
  return pts.filter((p, i) => {
    if (i === 0 || i === pts.length - 1) return true
    const [a, c] = [pts[i - 1], pts[i + 1]]
    return !((a[0] === p[0] && p[0] === c[0]) || (a[1] === p[1] && p[1] === c[1]))
  })
}

/** Polyline with rounded corners. */
function roundedPath(pts: Pt[], r = 10): string {
  if (pts.length < 2) return ''
  let d = `M${pts[0][0]} ${pts[0][1]}`
  for (let i = 1; i < pts.length - 1; i++) {
    const [px, py] = pts[i - 1]
    const [x, y] = pts[i]
    const [nx, ny] = pts[i + 1]
    const l1 = Math.hypot(x - px, y - py)
    const l2 = Math.hypot(nx - x, ny - y)
    const rr = Math.min(r, l1 / 2, l2 / 2)
    const ax = x - ((x - px) / l1) * rr
    const ay = y - ((y - py) / l1) * rr
    const bx = x + ((nx - x) / l2) * rr
    const by = y + ((ny - y) / l2) * rr
    d += ` L${ax} ${ay} Q${x} ${y} ${bx} ${by}`
  }
  const last = pts[pts.length - 1]
  return d + ` L${last[0]} ${last[1]}`
}

/** Only a run signal counts: an explicit run tag, or the primary tag of a state-bound symbol (pump, valve…). */
function isRunning(w: WidgetInstance | undefined, values: Record<string, SensorData>): boolean {
  if (!w) return false
  const r = resolveSymbol(w)
  const cfg = r?.config ?? w.config ?? {}
  if (cfg.runSensorId != null) return (values[String(cfg.runSensorId)]?.value ?? 0) > 0.5
  if (!r || r.def.binding !== 'state' || w.sensorId == null) return false
  const v = values[String(w.sensorId)]?.value
  return v != null && v > Number(cfg.onThreshold ?? 0.5)
}

export const CONNECTION_TYPES: { value: ConnectionInstance['connectionType']; label: string }[] = [
  { value: 'flow', label: 'Process pipe' },
  { value: 'power', label: 'Power line' },
  { value: 'data', label: 'Signal / data' },
]

export const ConnectionLayer: React.FC<Props> = ({ widgets, connections, currentValues, editMode, selectedId, onSelect, pending }) => {
  const byId = new Map(widgets.map(w => [w.id, w]))

  return (
    <svg style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', pointerEvents: 'none', zIndex: 1, overflow: 'visible' }}>
      {connections.map(conn => {
        const from = byId.get(conn.fromWidgetId)
        const to = byId.get(conn.toWidgetId)
        if (!from || !to) return null
        const d = roundedPath(routeConnection(from, to, conn.waypoints))
        const selected = editMode && selectedId === conn.id
        // A pipe flows when the equipment at either end is running (tank → running pump counts).
        const flowing = conn.connectionType === 'flow' && (isRunning(from, currentValues) || isRunning(to, currentValues))

        return (
          <g key={conn.id}>
            {selected && <path d={d} fill="none" stroke="var(--cyan)" strokeWidth={conn.connectionType === 'flow' ? 18 : 9} opacity={0.35} strokeLinejoin="round" />}
            {conn.connectionType === 'flow' ? (
              <>
                <path d={d} fill="none" stroke="var(--hmi-pipe-lo)" strokeWidth={11} strokeLinejoin="round" />
                <path d={d} fill="none" stroke="var(--hmi-pipe-hi)" strokeWidth={4.5} strokeLinejoin="round" opacity={0.7} />
                {flowing && (
                  <path d={d} fill="none" stroke="var(--hmi-liquid-hi)" strokeWidth={3} strokeDasharray="8 16"
                    strokeLinecap="round" className="hmi-flow" />
                )}
              </>
            ) : conn.connectionType === 'power' ? (
              <path d={d} fill="none" stroke="#c9a23a" strokeWidth={3} strokeLinejoin="round" />
            ) : (
              <path d={d} fill="none" stroke="#5b8fd1" strokeWidth={1.75} strokeDasharray="6 4" strokeLinejoin="round" />
            )}
            {editMode && onSelect && (
              <path d={d} fill="none" stroke="transparent" strokeWidth={16} style={{ pointerEvents: 'stroke', cursor: 'pointer' }}
                onMouseDown={e => e.stopPropagation()}
                onClick={e => { e.stopPropagation(); onSelect(conn.id) }} />
            )}
          </g>
        )
      })}
      {pending && byId.get(pending.fromId) && (() => {
        const src = byId.get(pending.fromId)!
        const [ax, ay] = portPoint(src, facingSide(src, [pending.x, pending.y]))
        return <line x1={ax} y1={ay} x2={pending.x} y2={pending.y} stroke="var(--cyan)" strokeWidth={2} strokeDasharray="5 4" />
      })()}
    </svg>
  )
}
