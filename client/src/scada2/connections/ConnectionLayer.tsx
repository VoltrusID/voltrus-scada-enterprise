import React from 'react'
import type { WidgetInstance, ConnectionInstance } from '../widgets/types'
import { CONNECTION_COLORS } from '../defaults'

interface Props {
  widgets: WidgetInstance[]
  connections: ConnectionInstance[]
  editMode: boolean
}

function getWidgetCenter(widgets: WidgetInstance[], id: string): { x: number; y: number } | null {
  const w = widgets.find(w => w.id === id)
  if (!w) return null
  return { x: w.x + w.width / 2, y: w.y + w.height / 2 }
}

export const ConnectionLayer: React.FC<Props> = ({ widgets, connections, editMode }) => {
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

        const points = conn.waypoints.length > 0
          ? conn.waypoints.map(p => p.join(',')).join(' ')
          : `${from.x},${from.y} ${to.x},${to.y}`

        return (
          <polyline
            key={conn.id}
            points={conn.waypoints.length > 0
              ? `${from.x},${from.y} ${points} ${to.x},${to.y}`
              : points
            }
            fill="none"
            stroke={color}
            strokeWidth={conn.connectionType === 'power' ? 3 : 2}
            strokeDasharray={conn.connectionType === 'data' ? '8 4' : undefined}
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        )
      })}
    </svg>
  )
}
