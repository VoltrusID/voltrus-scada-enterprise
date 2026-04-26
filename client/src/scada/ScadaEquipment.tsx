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
  const resizeRef = useRef<{ startX: number; startY: number; origW: number; origH: number } | null>(null)
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
      const dx = ev.clientX - dragRef.current.startX
      const dy = ev.clientY - dragRef.current.startY
      onMove(Math.round((dragRef.current.origX + dx) / 10) * 10, Math.round((dragRef.current.origY + dy) / 10) * 10)
    }
    const onUp = () => { dragRef.current = null; window.removeEventListener('mousemove', onMoveHandler); window.removeEventListener('mouseup', onUp) }
    window.addEventListener('mousemove', onMoveHandler)
    window.addEventListener('mouseup', onUp)
  }, [editMode, activeTool, equipment.x, equipment.y, onSelect, onMove])

  const handleResizeMouseDown = useCallback((e: React.MouseEvent) => {
    e.stopPropagation()
    resizeRef.current = { startX: e.clientX, startY: e.clientY, origW: equipment.width, origH: equipment.height }
    const onResizeHandler = (ev: MouseEvent) => {
      if (!resizeRef.current) return
      onResize(Math.max(40, Math.round((resizeRef.current.origW + ev.clientX - resizeRef.current.startX) / 10) * 10), Math.max(30, Math.round((resizeRef.current.origH + ev.clientY - resizeRef.current.startY) / 10) * 10))
    }
    const onUp = () => { resizeRef.current = null; window.removeEventListener('mousemove', onResizeHandler); window.removeEventListener('mouseup', onUp) }
    window.addEventListener('mousemove', onResizeHandler)
    window.addEventListener('mouseup', onUp)
  }, [equipment.width, equipment.height, onResize])

  const handlePortClick = useCallback((e: React.MouseEvent, side: string) => {
    if (!editMode || activeTool !== 'connect') return
    e.stopPropagation()
    onStartConnection(side)
  }, [editMode, activeTool, onStartConnection])

  const dataLabel = bindings.map(id => { const val = currentValues[String(id)]; return val ? `${val.value.toFixed(1)}` : null }).filter(Boolean).join(' | ')

  return (
    <g transform={`translate(${equipment.x}, ${equipment.y})${equipment.rotation ? ` rotate(${equipment.rotation}, ${equipment.width/2}, ${equipment.height/2})` : ''}`} onMouseDown={handleMouseDown} style={{ cursor: editMode ? 'move' : 'pointer' }}>
      {selected && <rect x="-4" y="-4" width={equipment.width + 8} height={equipment.height + 8} rx="4" fill="none" stroke="var(--cyan)" strokeWidth="2" strokeDasharray="6 3" />}
      <foreignObject x="0" y="0" width={equipment.width} height={equipment.height}>
        {AssetComponent && <AssetComponent statusColor={statusColor} />}
      </foreignObject>
      <text x={equipment.width / 2} y={equipment.height + 14} textAnchor="middle" fontSize="10" fill="#555" style={{ userSelect: 'none' }}>{equipment.equipment_name || equipment.equipment_type.replace(/_/g, ' ')}</text>
      {dataLabel && (
        <g>
          <rect x={equipment.width - 8} y="-14" width={Math.max(dataLabel.length * 6 + 8, 40)} height="16" rx="3" fill="rgba(255,255,255,0.92)" stroke="#ddd" strokeWidth="0.5" />
          <text x={equipment.width - 4} y="-2" fontSize="9" fill={statusColor} style={{ userSelect: 'none' }}>{dataLabel}</text>
        </g>
      )}
      {editMode && activeTool === 'connect' && ports.map((port, i) => (
        <circle key={i} cx={port.x * equipment.width} cy={port.y * equipment.height} r="5" fill="var(--cyan)" stroke="#fff" strokeWidth="1.5" style={{ cursor: 'crosshair' }} onMouseDown={(e) => handlePortClick(e, port.side)} />
      ))}
      {editMode && selected && (
        <rect x={equipment.width - 6} y={equipment.height - 6} width="8" height="8" rx="2" fill="var(--cyan)" stroke="#fff" strokeWidth="1" style={{ cursor: 'nwse-resize' }} onMouseDown={handleResizeMouseDown} />
      )}
    </g>
  )
}
