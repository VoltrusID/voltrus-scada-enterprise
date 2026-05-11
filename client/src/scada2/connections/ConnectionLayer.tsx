import React, { useCallback, useState } from 'react'
import type { WidgetInstance, ConnectionInstance } from '../widgets/types'
import { CONNECTION_COLORS } from '../defaults'

interface Props {
  widgets: WidgetInstance[]
  connections: ConnectionInstance[]
  editMode: boolean
  onDeleteConnection?: (id: string) => void
}

function getWidgetCenter(widgets: WidgetInstance[], id: string): { x: number; y: number } | null {
  const w = widgets.find(w => w.id === id)
  if (!w) return null
  return { x: w.x + w.width / 2, y: w.y + w.height / 2 }
}

function getConnectionMidpoint(
  widgets: WidgetInstance[],
  conn: ConnectionInstance,
): { x: number; y: number } | null {
  const from = getWidgetCenter(widgets, conn.fromWidgetId)
  const to = getWidgetCenter(widgets, conn.toWidgetId)
  if (!from || !to) return null

  if (conn.waypoints.length > 0) {
    const allPts: { x: number; y: number }[] = [from, ...conn.waypoints.map(p => ({ x: p[0], y: p[1] })), to]
    const midIdx = Math.floor(allPts.length / 2)
    return allPts[midIdx]
  }

  return { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 }
}

export const ConnectionLayer: React.FC<Props> = ({ widgets, connections, editMode, onDeleteConnection }) => {
  const [hoveredId, setHoveredId] = useState<string | null>(null)

  return (
    <svg
      style={{
        position: 'absolute', inset: 0, width: '100%', height: '100%',
        pointerEvents: editMode ? 'auto' : 'none',
        zIndex: 1,
      }}
    >
      {connections.map(conn => {
        const from = getWidgetCenter(widgets, conn.fromWidgetId)
        const to = getWidgetCenter(widgets, conn.toWidgetId)
        if (!from || !to) return null

        const color = CONNECTION_COLORS[conn.connectionType] ?? CONNECTION_COLORS.power
        const isHovered = hoveredId === conn.id

        const points = conn.waypoints.length > 0
          ? conn.waypoints.map(p => p.join(',')).join(' ')
          : `${from.x},${from.y} ${to.x},${to.y}`

        const mid = getConnectionMidpoint(widgets, conn)

        return (
          <g key={conn.id}>
            {/* Invisible fat line for easier hover/click */}
            {editMode && (
              <polyline
                points={conn.waypoints.length > 0
                  ? `${from.x},${from.y} ${points} ${to.x},${to.y}`
                  : points
                }
                fill="none"
                stroke="transparent"
                strokeWidth={16}
                style={{ cursor: onDeleteConnection ? 'pointer' : 'default' }}
                onMouseEnter={() => setHoveredId(conn.id)}
                onMouseLeave={() => setHoveredId(null)}
                onClick={(e) => {
                  e.stopPropagation()
                  onDeleteConnection?.(conn.id)
                }}
              />
            )}
            {/* Visible line */}
            <polyline
              points={conn.waypoints.length > 0
                ? `${from.x},${from.y} ${points} ${to.x},${to.y}`
                : points
              }
              fill="none"
              stroke={isHovered && editMode ? '#ef4444' : color}
              strokeWidth={conn.connectionType === 'power' ? 3 : 2}
              strokeDasharray={conn.connectionType === 'data' ? '8 4' : undefined}
              strokeLinecap="round"
              strokeLinejoin="round"
              style={{
                transition: 'stroke 0.15s',
                pointerEvents: 'none',
              }}
            />
            {/* Delete button at midpoint in delete mode */}
            {editMode && mid && onDeleteConnection && (
              <g
                style={{ cursor: 'pointer', pointerEvents: 'auto' }}
                onClick={(e) => {
                  e.stopPropagation()
                  onDeleteConnection(conn.id)
                }}
                onMouseEnter={() => setHoveredId(conn.id)}
                onMouseLeave={() => setHoveredId(null)}
              >
                <circle
                  cx={mid.x}
                  cy={mid.y}
                  r={9}
                  fill={isHovered ? '#ef4444' : 'var(--card)'}
                  stroke={isHovered ? '#ef4444' : 'var(--border)'}
                  strokeWidth={1.5}
                  style={{ transition: 'fill 0.15s, stroke 0.15s' }}
                />
                <text
                  x={mid.x}
                  y={mid.y + 1}
                  textAnchor="middle"
                  dominantBaseline="central"
                  fontSize={12}
                  fontWeight={700}
                  fill={isHovered ? '#fff' : 'var(--muted)'}
                  style={{ pointerEvents: 'none', transition: 'fill 0.15s' }}
                >×</text>
              </g>
            )}
          </g>
        )
      })}
    </svg>
  )
}
