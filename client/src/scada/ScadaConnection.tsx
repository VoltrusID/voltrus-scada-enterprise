import React from 'react'
import type { ScadaEquipmentItem, ScadaConnectionItem } from '../types'
import { CONNECTION_COLORS } from './defaults'

interface Props {
  connection: ScadaConnectionItem
  equipment: ScadaEquipmentItem[]
  selected: boolean
  editMode: boolean
  activeTool: string
  onSelect: () => void
  onDelete: () => void
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
  if (waypoints.length > 0) {
    const points = [from, ...waypoints.map(wp => ({ x: wp[0], y: wp[1] })), to]
    return points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ')
  }
  const midX = from.x + (to.x - from.x) * 0.5
  return `M ${from.x} ${from.y} L ${midX} ${from.y} L ${midX} ${to.y} L ${to.x} ${to.y}`
}

export const ScadaConnection: React.FC<Props> = ({ connection, equipment, selected, editMode, activeTool, onSelect, onDelete }) => {
  const fromEq = equipment.find(e => e.equipment_id === connection.from_equipment)
  const toEq = equipment.find(e => e.equipment_id === connection.to_equipment)
  if (!fromEq || !toEq) return null

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

  const midX = (from.x + to.x) / 2
  const midY = (from.y + to.y) / 2
  const angle = Math.atan2(to.y - from.y, to.x - from.x) * 180 / Math.PI

  return (
    <g onClick={editMode ? (activeTool === 'delete' ? onDelete : onSelect) : undefined} style={{ cursor: editMode ? (activeTool === 'delete' ? 'crosshair' : 'pointer') : 'default' }}>
      <path d={pathD} fill="none" stroke="transparent" strokeWidth="12" />
      <path d={pathD} fill="none" stroke={selected ? 'var(--cyan)' : color} strokeWidth={strokeWidth} strokeDasharray={isDashed ? '8 4' : undefined} strokeLinecap="round" strokeLinejoin="round" />
      <polygon points="-4,-3 4,0 -4,3" transform={`translate(${midX}, ${midY}) rotate(${angle})`} fill={selected ? 'var(--cyan)' : color} />
    </g>
  )
}
