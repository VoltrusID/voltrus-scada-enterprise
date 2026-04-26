import React, { useCallback, useRef } from 'react'
import type { ScadaEquipmentItem, SensorData, SensorConfig } from '../types'
import { ASSET_MAP } from './assets'
import { PORT_POSITIONS } from './defaults'

interface Props {
  equipment: ScadaEquipmentItem
  currentValues: Record<string, SensorData>
  sensors: SensorConfig[]
  editMode: boolean
  selected: boolean
  activeTool: string
  onSelect: () => void
  onDelete: () => void
  onMove: (x: number, y: number) => void
  onResize: (w: number, h: number) => void
  onStartConnection: (portSide: string) => void
  onEndConnection: (portSide: string) => void
  onSensorClick: (sensorId: number) => void
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
  return '#ccc'
}

export const ScadaEquipment: React.FC<Props> = ({ equipment, currentValues, sensors, editMode, selected, activeTool, onSelect, onDelete, onMove, onResize, onStartConnection, onEndConnection, onSensorClick }) => {
  const dragRef = useRef<{ startX: number; startY: number; origX: number; origY: number } | null>(null)
  const resizeRef = useRef<{ startX: number; startY: number; origW: number; origH: number } | null>(null)
  const statusColor = getStatusColor(equipment, currentValues)
  const AssetComponent = ASSET_MAP[equipment.equipment_type]
  const ports = PORT_POSITIONS[equipment.equipment_type] || []
  const bindings: string[] = JSON.parse(equipment.sensor_bindings || '[]')

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!editMode) return
    e.stopPropagation()
    if (activeTool === 'delete') { onDelete(); return }
    if (activeTool !== 'select') return
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
  }, [editMode, activeTool, equipment.x, equipment.y, onSelect, onDelete, onMove])

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

  const dataCards = bindings.map(id => {
    const val = currentValues[String(id)]
    const cfg = sensors.find(s => String(s.id) === String(id))
    if (!val || !cfg) return null
    let color = '#22c55e'
    if (val.crit > 0 && val.value >= val.crit) color = '#ef4444'
    else if (val.warn > 0 && val.value >= val.warn) color = '#eab308'
    else color = cfg.color || '#22c55e'
    return { sensorId: cfg.id, value: val.value.toFixed(cfg.decimals ?? 1), unit: cfg.unit || '', name: cfg.name, color }
  }).filter(Boolean) as { sensorId: number; value: string; unit: string; name: string; color: string }[]

  return (
    <g transform={`translate(${equipment.x}, ${equipment.y})${equipment.rotation ? ` rotate(${equipment.rotation}, ${equipment.width/2}, ${equipment.height/2})` : ''}`} onMouseDown={handleMouseDown} style={{ cursor: editMode ? 'move' : 'pointer' }}>
      {selected && <rect x="-4" y="-4" width={equipment.width + 8} height={equipment.height + 8} rx="4" fill="none" stroke="var(--cyan)" strokeWidth="2" strokeDasharray="6 3" />}
      <foreignObject x="0" y="0" width={equipment.width} height={equipment.height}>
        {AssetComponent && <AssetComponent statusColor={statusColor} />}
      </foreignObject>
      <text x={equipment.width / 2} y={equipment.height + 14} textAnchor="middle" fontSize="10" fill="#999" style={{ userSelect: 'none' }}>{equipment.equipment_name || equipment.equipment_type.replace(/_/g, ' ')}</text>
      {dataCards.length > 0 && (
        <g>
          {dataCards.map((card, i) => {
            const nameW = Math.max(card.name.length, 4) * 4.2 + 14
            const valW = (card.value.length + (card.unit ? card.unit.length + 1 : 0)) * 6.2 + 14
            const cardW = Math.max(nameW, valW, 40)
            const cardH = 22
            const offsetX = -cardW - 4
            const offsetY = i * (cardH + 3)
            return (
              <g key={i} transform={`translate(${offsetX}, ${offsetY})`} style={{ cursor: 'pointer' }} onClick={(e) => { e.stopPropagation(); onSensorClick(card.sensorId) }}>
                <rect x="0" y="0" width={cardW} height={cardH} rx="3" fill="rgba(20,20,30,0.93)" stroke={card.color} strokeWidth="0.8" />
                <rect x="0" y="0" width="3" height={cardH} rx="1" fill={card.color} />
                <text x="8" y="10" fontSize="7" fill="#888" style={{ userSelect: 'none' }}>{card.name.length > 10 ? card.name.slice(0, 10) + '…' : card.name}</text>
                <text x="8" y="19" fontSize="10" fontWeight="600" fill={card.color} style={{ userSelect: 'none' }}>{card.value}<tspan fontSize="8" fill="#aaa">{card.unit ? ` ${card.unit}` : ''}</tspan></text>
              </g>
            )
          })}
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
