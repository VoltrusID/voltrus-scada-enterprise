import React, { useState, useCallback, useRef, useEffect } from 'react'
import type { ScadaEquipmentItem, ScadaConnectionItem, ScadaEquipmentType, SensorData, SensorConfig } from '../types'
import { ScadaEquipment } from './ScadaEquipment'
import { ScadaConnection } from './ScadaConnection'

interface Props {
  equipment: ScadaEquipmentItem[]
  connections: ScadaConnectionItem[]
  currentValues: Record<string, SensorData>
  sensors: SensorConfig[]
  editMode: boolean
  selectedId: string | null
  activeTool: 'select' | 'connect' | 'delete'
  pendingType: ScadaEquipmentType | null
  onSelect: (id: string | null) => void
  onEquipmentMove: (id: string, x: number, y: number) => void
  onEquipmentResize: (id: string, w: number, h: number) => void
  onAddEquipment: (type: ScadaEquipmentType, x: number, y: number) => void
  onStartConnection: (eqId: string, portSide: string) => void
  onEndConnection: (eqId: string, portSide: string) => void
  onDeleteEquipment: (id: string) => void
  onDeleteConnection: (id: string) => void
  onSensorClick: (sensorId: number) => void
}

export const ScadaCanvas: React.FC<Props> = ({
  equipment, connections, currentValues, sensors, editMode, selectedId, activeTool, pendingType,
  onSelect, onEquipmentMove, onEquipmentResize, onAddEquipment, onStartConnection, onEndConnection, onDeleteEquipment, onDeleteConnection, onSensorClick,
}) => {
  const [viewBox, setViewBox] = useState({ x: 0, y: 0, w: 1200, h: 800 })
  const [isPanning, setIsPanning] = useState(false)
  const [panStart, setPanStart] = useState({ x: 0, y: 0 })
  const svgRef = useRef<SVGSVGElement>(null)

  const zoom = useCallback((delta: number, cx: number, cy: number) => {
    setViewBox(prev => {
      const scale = delta > 0 ? 0.9 : 1.1
      const newW = Math.max(150, Math.min(4800, prev.w * scale))
      const newH = Math.max(100, Math.min(3200, prev.h * scale))
      const dx = (newW - prev.w) * (cx / (svgRef.current?.clientWidth || 800))
      const dy = (newH - prev.h) * (cy / (svgRef.current?.clientHeight || 600))
      return { x: prev.x - dx, y: prev.y - dy, w: newW, h: newH }
    })
  }, [])

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault()
    const rect = svgRef.current?.getBoundingClientRect()
    if (!rect) return
    zoom(e.deltaY, e.clientX - rect.left, e.clientY - rect.top)
  }, [zoom])

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.target === svgRef.current || ((e.target as Element).tagName === 'rect' && (e.target as Element).classList.contains('canvas-bg'))) {
      if (pendingType) {
        const rect = svgRef.current?.getBoundingClientRect()
        if (rect) {
          const svgX = viewBox.x + (e.clientX - rect.left) / rect.width * viewBox.w
          const svgY = viewBox.y + (e.clientY - rect.top) / rect.height * viewBox.h
          onAddEquipment(pendingType, svgX, svgY)
        }
        return
      }
      if (activeTool === 'select' || !editMode) {
        onSelect(null)
        setIsPanning(true)
        setPanStart({ x: e.clientX, y: e.clientY })
      }
    }
  }, [editMode, activeTool, pendingType, viewBox, onAddEquipment, onSelect])

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (isPanning) {
      const rect = svgRef.current?.getBoundingClientRect()
      if (!rect) return
      const dx = (e.clientX - panStart.x) / rect.width * viewBox.w
      const dy = (e.clientY - panStart.y) / rect.height * viewBox.h
      setViewBox(prev => ({ ...prev, x: prev.x - dx, y: prev.y - dy }))
      setPanStart({ x: e.clientX, y: e.clientY })
    }
  }, [isPanning, panStart, viewBox])

  const handleMouseUp = useCallback(() => { setIsPanning(false) }, [])

  useEffect(() => {
    if (!editMode) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Delete' && selectedId) onDeleteEquipment(selectedId)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [editMode, selectedId, onDeleteEquipment])

  const zoomIn = () => setViewBox(prev => {
    const newW = prev.w * 0.8; const newH = prev.h * 0.8
    return { x: prev.x + (prev.w - newW) / 2, y: prev.y + (prev.h - newH) / 2, w: newW, h: newH }
  })
  const zoomOut = () => setViewBox(prev => {
    const newW = prev.w * 1.25; const newH = prev.h * 1.25
    return { x: prev.x + (prev.w - newW) / 2, y: prev.y + (prev.h - newH) / 2, w: newW, h: newH }
  })
  const fitView = () => {
    if (equipment.length === 0) { setViewBox({ x: 0, y: 0, w: 1200, h: 800 }); return }
    const minX = Math.min(...equipment.map(e => e.x)) - 50
    const minY = Math.min(...equipment.map(e => e.y)) - 50
    const maxX = Math.max(...equipment.map(e => e.x + e.width)) + 50
    const maxY = Math.max(...equipment.map(e => e.y + e.height)) + 50
    setViewBox({ x: minX, y: minY, w: maxX - minX, h: maxY - minY })
  }

  return (
    <div style={{ position: 'relative', width: '100%', height: 'calc(100vh - 180px)', background: '#060e09', border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden' }}>
      <div style={{ position: 'absolute', top: 8, right: 8, zIndex: 10, display: 'flex', flexDirection: 'column', gap: 4 }}>
        <button onClick={zoomIn} style={{ width: 28, height: 28, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 4, cursor: 'pointer', fontSize: 16, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>+</button>
        <button onClick={zoomOut} style={{ width: 28, height: 28, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 4, cursor: 'pointer', fontSize: 16, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>-</button>
        <button onClick={fitView} style={{ width: 28, height: 28, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 4, cursor: 'pointer', fontSize: 10, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>Fit</button>
      </div>
      <svg
        ref={svgRef}
        style={{ width: '100%', height: '100%', cursor: isPanning ? 'grabbing' : editMode && pendingType ? 'crosshair' : 'grab' }}
        viewBox={`${viewBox.x} ${viewBox.y} ${viewBox.w} ${viewBox.h}`}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        <defs>
          <pattern id="scada-grid" width="20" height="20" patternUnits="userSpaceOnUse">
            <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#0e1f16" strokeWidth="0.5" />
          </pattern>
        </defs>
        <rect className="canvas-bg" x={viewBox.x - 100} y={viewBox.y - 100} width={viewBox.w + 200} height={viewBox.h + 200} fill="#060e09" />
        <rect className="canvas-bg" x={viewBox.x - 100} y={viewBox.y - 100} width={viewBox.w + 200} height={viewBox.h + 200} fill="url(#scada-grid)" />
        {connections.map(conn => (
          <ScadaConnection key={conn.connection_id} connection={conn} equipment={equipment} selected={selectedId === conn.connection_id} editMode={editMode} activeTool={activeTool} onSelect={() => onSelect(conn.connection_id)} onDelete={() => onDeleteConnection(conn.connection_id)} />
        ))}
        {equipment.map(eq => (
          <ScadaEquipment key={eq.equipment_id} equipment={eq} currentValues={currentValues} sensors={sensors} editMode={editMode} selected={selectedId === eq.equipment_id} activeTool={activeTool} onSelect={() => onSelect(eq.equipment_id)} onDelete={() => onDeleteEquipment(eq.equipment_id)} onMove={(x, y) => onEquipmentMove(eq.equipment_id, x, y)} onResize={(w, h) => onEquipmentResize(eq.equipment_id, w, h)} onStartConnection={(side) => onStartConnection(eq.equipment_id, side)} onEndConnection={(side) => onEndConnection(eq.equipment_id, side)} onSensorClick={onSensorClick} />
        ))}
      </svg>
    </div>
  )
}
