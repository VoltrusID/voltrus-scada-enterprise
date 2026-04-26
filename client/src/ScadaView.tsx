import React, { useState, useCallback, useRef, useEffect } from 'react'
import type { ScadaEquipmentItem, ScadaConnectionItem, ScadaEquipmentType, SensorConfig, SensorData } from './types'
import { DEFAULT_SIZES } from './scada/defaults'
import { ScadaCanvas } from './scada/ScadaCanvas'
import { ScadaEditToolbar } from './scada/ScadaEditToolbar'
import { ScadaPalette } from './scada/ScadaPalette'
import { ScadaEditPanel } from './scada/ScadaEditPanel'

interface Props {
  sensors: SensorConfig[]
  currentValues: Record<string, SensorData>
  activeScreen: string
  scadaEquipment: ScadaEquipmentItem[]
  scadaConnections: ScadaConnectionItem[]
  onSaveEquipment: (items: ScadaEquipmentItem[]) => void
  onSaveConnections: (items: ScadaConnectionItem[]) => void
  onSensorClick: (sensorId: number) => void
}

export const ScadaView: React.FC<Props> = ({
  sensors, currentValues, activeScreen, scadaEquipment: serverEquipment, scadaConnections: serverConnections,
  onSaveEquipment, onSaveConnections, onSensorClick,
}) => {
  const [editMode, setEditMode] = useState(false)
  const [activeTool, setActiveTool] = useState<'select' | 'connect' | 'delete'>('select')
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [equipment, setEquipment] = useState<ScadaEquipmentItem[]>([])
  const [connections, setConnections] = useState<ScadaConnectionItem[]>([])
  const [pendingType, setPendingType] = useState<ScadaEquipmentType | null>(null)
  const [connectFrom, setConnectFrom] = useState<{ eqId: string; side: string } | null>(null)
  const dirtyRef = useRef(false)

  useEffect(() => { setEquipment(serverEquipment) }, [serverEquipment])
  useEffect(() => { setConnections(serverConnections) }, [serverConnections])

  const hasChanges = dirtyRef.current

  const handleEquipmentMove = useCallback((id: string, x: number, y: number) => {
    setEquipment(prev => prev.map(e => e.equipment_id === id ? { ...e, x, y } : e))
    dirtyRef.current = true
  }, [])

  const handleEquipmentResize = useCallback((id: string, w: number, h: number) => {
    setEquipment(prev => prev.map(e => e.equipment_id === id ? { ...e, width: w, height: h } : e))
    dirtyRef.current = true
  }, [])

  const handleAddEquipment = useCallback((type: ScadaEquipmentType, x: number, y: number) => {
    const defaults = DEFAULT_SIZES[type]
    const newEq: ScadaEquipmentItem = {
      equipment_id: `scada_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
      equipment_type: type,
      equipment_name: type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
      x: Math.round(x - defaults.width / 2),
      y: Math.round(y - defaults.height / 2),
      width: defaults.width,
      height: defaults.height,
      rotation: 0,
      sensor_bindings: '[]',
    }
    setEquipment(prev => [...prev, newEq])
    setPendingType(null)
    setActiveTool('select')
    dirtyRef.current = true
  }, [])

  const handleDeleteEquipment = useCallback((id: string) => {
    setEquipment(prev => prev.filter(e => e.equipment_id !== id))
    setConnections(prev => prev.filter(c => c.from_equipment !== id && c.to_equipment !== id))
    setSelectedId(null)
    dirtyRef.current = true
  }, [])

  const handleDeleteConnection = useCallback((id: string) => {
    setConnections(prev => prev.filter(c => c.connection_id !== id))
    setSelectedId(null)
    dirtyRef.current = true
  }, [])

  const handleStartConnection = useCallback((eqId: string, _side: string) => {
    if (connectFrom) {
      if (connectFrom.eqId !== eqId) {
        const newConn: ScadaConnectionItem = {
          connection_id: `conn_${Date.now()}`,
          from_equipment: connectFrom.eqId,
          to_equipment: eqId,
          waypoints: '[]',
          connection_type: 'power',
        }
        setConnections(prev => [...prev, newConn])
        dirtyRef.current = true
      }
      setConnectFrom(null)
    } else {
      setConnectFrom({ eqId, side: _side })
    }
  }, [connectFrom])

  const handleEndConnection = useCallback((_eqId: string, _side: string) => {}, [])

  const handleSave = useCallback(() => {
    onSaveEquipment(equipment)
    onSaveConnections(connections)
    dirtyRef.current = false
  }, [equipment, connections, onSaveEquipment, onSaveConnections])

  const handleToggleEdit = useCallback(() => {
    if (editMode) { setActiveTool('select'); setPendingType(null); setConnectFrom(null) }
    setEditMode(prev => !prev)
    setSelectedId(null)
  }, [editMode])

  const selectedEquipment = equipment.find(e => e.equipment_id === selectedId) || null

  const handleEditPanelUpdate = useCallback((updates: Partial<ScadaEquipmentItem>) => {
    if (!selectedEquipment) return
    setEquipment(prev => prev.map(e => e.equipment_id === selectedEquipment.equipment_id ? { ...e, ...updates } : e))
    dirtyRef.current = true
  }, [selectedEquipment])

  return (
    <div style={{ position: 'relative' }}>
      <ScadaEditToolbar editMode={editMode} activeTool={activeTool} onToggleEdit={handleToggleEdit} onToolChange={setActiveTool} onSave={handleSave} hasChanges={hasChanges} />
      <div style={{ position: 'relative' }}>
        <ScadaPalette visible={editMode && activeTool === 'select'} onSelect={setPendingType} selectedType={pendingType} />
        <ScadaCanvas
          equipment={equipment} connections={connections} currentValues={currentValues} sensors={sensors}
          editMode={editMode} selectedId={selectedId} activeTool={activeTool} pendingType={pendingType}
          onSelect={setSelectedId}
          onEquipmentMove={handleEquipmentMove} onEquipmentResize={handleEquipmentResize}
          onAddEquipment={handleAddEquipment}
          onStartConnection={handleStartConnection} onEndConnection={handleEndConnection}
          onDeleteEquipment={handleDeleteEquipment}
          onDeleteConnection={handleDeleteConnection}
          onSensorClick={onSensorClick}
        />
        {editMode && selectedEquipment && (
          <ScadaEditPanel equipment={selectedEquipment} sensors={sensors} onUpdate={handleEditPanelUpdate} onClose={() => setSelectedId(null)} />
        )}
      </div>
    </div>
  )
}
