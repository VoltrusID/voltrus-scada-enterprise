import React, { useReducer, useState, useCallback, useRef, useEffect } from 'react'
import type { ScadaWidgetType, ScadaCanvasState, ConnectionInstance } from './widgets/types'
import { WIDGET_MANIFESTS } from './widgets/registry'
import { ScadaCanvas, WidgetTypePicker } from './ScadaCanvas'
import { ScadaToolbar } from './ScadaToolbar'
import { ScadaPalette } from './ScadaPalette'
import { ConnectionLayer } from './connections/ConnectionLayer'
import { layoutReducer, createWidget, createConnection } from './engine/widgetState'
import type { ScadaLayoutState } from './engine/widgetState'
import type { SensorConfig, SensorData, ScadaEquipmentItem, ScadaConnectionItem } from '../types'
import { useDashboardSelector } from '../store/dashboardStore'

// Import all widgets to trigger registration
import './widgets/IndicatorWidget'
import './widgets/GaugeWidget'
import './widgets/BarWidget'
import './widgets/StatusLightWidget'
import './widgets/TankLevelWidget'
import './widgets/OnOffWidget'
import './widgets/LabelWidget'
import './widgets/TrendWidget'
import './widgets/BigNumberWidget'
import './widgets/DataTableWidget'
import './widgets/PipeWidget'

interface Props {
  sensors: SensorConfig[]
  activeScreen: string
  scadaEquipment: ScadaEquipmentItem[]
  scadaConnections: ScadaConnectionItem[]
  onSaveEquipment: (items: ScadaEquipmentItem[]) => void
  onSaveConnections: (items: ScadaConnectionItem[]) => void
  onSensorClick: (sensorId: number) => void
}

const initialCanvasState: ScadaCanvasState = {
  editMode: false,
  activeTool: 'select',
  selectedId: null,
  connectFrom: null,
  pendingSensor: null,
}

function loadLayoutFromAPI(
  equipment: ScadaEquipmentItem[],
  connections: ScadaConnectionItem[],
): ScadaLayoutState {
  const widgets = equipment.map(eq => {
    const bindings: number[] = JSON.parse(eq.sensor_bindings || '[]')
    return {
      id: eq.equipment_id,
      type: eq.equipment_type as ScadaWidgetType,
      sensorId: bindings[0] ?? null,
      sensorIds: bindings,
      x: eq.x,
      y: eq.y,
      width: eq.width,
      height: eq.height,
      config: {} as Record<string, any>,
    }
  })

  const conns: ConnectionInstance[] = connections.map(c => ({
    id: c.connection_id,
    fromWidgetId: c.from_equipment,
    toWidgetId: c.to_equipment,
    connectionType: c.connection_type as ConnectionInstance['connectionType'],
    waypoints: JSON.parse(c.waypoints || '[]'),
  }))

  return { widgets, connections: conns }
}

export const ScadaView: React.FC<Props> = ({
  sensors, activeScreen, scadaEquipment, scadaConnections,
  onSaveEquipment, onSaveConnections, onSensorClick,
}) => {
  const currentValues = useDashboardSelector(s => s.currentValues)
  const [layout, dispatch] = useReducer(layoutReducer, { widgets: [], connections: [] })
  const [canvasState, setCanvasState] = useState<ScadaCanvasState>(initialCanvasState)
  const dirtyRef = useRef(false)

  // Load layout when screen changes
  useEffect(() => {
    const loaded = loadLayoutFromAPI(scadaEquipment, scadaConnections)
    dispatch({ type: 'LOAD_LAYOUT', widgets: loaded.widgets, connections: loaded.connections })
    dirtyRef.current = false
  }, [activeScreen, scadaEquipment, scadaConnections])

  // ── Handlers ──

  const handleToggleEdit = useCallback(() => {
    setCanvasState(prev => ({
      ...prev,
      editMode: !prev.editMode,
      activeTool: 'select',
      connectFrom: null,
      pendingSensor: null,
    }))
  }, [])

  const handleToolChange = useCallback((tool: ScadaCanvasState['activeTool']) => {
    setCanvasState(prev => ({ ...prev, activeTool: tool, connectFrom: null }))
  }, [])

  const handleSelectWidget = useCallback((id: string | null) => {
    setCanvasState(prev => ({ ...prev, selectedId: id }))

    // Connect mode: clicking a second widget creates a connection
    if (id && canvasState.activeTool === 'connect' && canvasState.editMode) {
      if (canvasState.connectFrom && canvasState.connectFrom !== id) {
        dispatch({ type: 'ADD_CONNECTION', connection: createConnection(canvasState.connectFrom, id, 'power') })
        dirtyRef.current = true
        setCanvasState(prev => ({ ...prev, connectFrom: null }))
      } else {
        setCanvasState(prev => ({ ...prev, connectFrom: id }))
      }
    }

    // Delete mode
    if (id && canvasState.activeTool === 'delete' && canvasState.editMode) {
      dispatch({ type: 'REMOVE_WIDGET', id })
      dirtyRef.current = true
      setCanvasState(prev => ({ ...prev, selectedId: null }))
    }
  }, [canvasState.activeTool, canvasState.editMode, canvasState.connectFrom])

  const handleMoveWidget = useCallback((id: string, x: number, y: number) => {
    dispatch({ type: 'MOVE_WIDGET', id, x, y })
    dirtyRef.current = true
  }, [])

  const handleResizeWidget = useCallback((id: string, x: number, y: number, width: number, height: number) => {
    dispatch({ type: 'RESIZE_WIDGET', id, x, y, width, height })
    dirtyRef.current = true
  }, [])

  const handleDropSensor = useCallback((sensorId: number, x: number, y: number) => {
    setCanvasState(prev => ({ ...prev, pendingSensor: { sensorId, x, y } }))
  }, [])

  const handlePickWidgetType = useCallback((type: ScadaWidgetType) => {
    if (!canvasState.pendingSensor) return
    const { sensorId, x, y } = canvasState.pendingSensor
    const manifest = WIDGET_MANIFESTS[type]
    const widget = createWidget(type, sensorId, x, y, manifest.defaultWidth, manifest.defaultHeight)
    dispatch({ type: 'ADD_WIDGET', widget })
    dirtyRef.current = true
    setCanvasState(prev => ({ ...prev, pendingSensor: null }))
  }, [canvasState.pendingSensor])

  const handleCancelPick = useCallback(() => {
    setCanvasState(prev => ({ ...prev, pendingSensor: null }))
  }, [])

  // Save: convert widget layout back to API format
  const handleSave = useCallback(() => {
    const equipment: ScadaEquipmentItem[] = layout.widgets.map(w => ({
      equipment_id: w.id,
      equipment_type: w.type as any,
      equipment_name: sensors.find(s => s.id === w.sensorId)?.name ?? w.type,
      x: w.x,
      y: w.y,
      width: w.width,
      height: w.height,
      rotation: 0,
      sensor_bindings: JSON.stringify(w.sensorId != null ? [w.sensorId] : []),
    }))

    const connections: ScadaConnectionItem[] = layout.connections.map(c => ({
      connection_id: c.id,
      from_equipment: c.fromWidgetId,
      to_equipment: c.toWidgetId,
      waypoints: JSON.stringify(c.waypoints),
      connection_type: c.connectionType,
    }))

    onSaveEquipment(equipment)
    onSaveConnections(connections)
    dirtyRef.current = false
  }, [layout, sensors, onSaveEquipment, onSaveConnections])

  // Palette tag click: enter drop mode
  const handlePaletteSelect = useCallback((sensorId: number) => {
    onSensorClick(sensorId)
  }, [onSensorClick])

  return (
    <div style={{ position: 'relative' }}>
      <ScadaToolbar
        editMode={canvasState.editMode}
        activeTool={canvasState.activeTool}
        onToggleEdit={handleToggleEdit}
        onToolChange={handleToolChange}
        onSave={handleSave}
        hasChanges={dirtyRef.current}
      />
      <div style={{ position: 'relative' }}>
        <ScadaPalette
          visible={canvasState.editMode}
          sensors={sensors}
          onSelect={handlePaletteSelect}
        />
        <ScadaCanvas
          layout={layout}
          sensors={sensors}
          currentValues={currentValues}
          editMode={canvasState.editMode}
          selectedId={canvasState.selectedId}
          onSelectWidget={handleSelectWidget}
          onMoveWidget={handleMoveWidget}
          onResizeWidget={handleResizeWidget}
          onDropSensor={handleDropSensor}
          onClickEmpty={() => setCanvasState(prev => ({ ...prev, selectedId: null }))}
        >
          <ConnectionLayer
            widgets={layout.widgets}
            connections={layout.connections}
            editMode={canvasState.editMode}
          />
        </ScadaCanvas>
        {canvasState.pendingSensor && (
          <WidgetTypePicker
            x={canvasState.pendingSensor.x}
            y={canvasState.pendingSensor.y}
            sensorId={canvasState.pendingSensor.sensorId}
            sensors={sensors}
            onSelect={handlePickWidgetType}
            onCancel={handleCancelPick}
          />
        )}
      </div>
    </div>
  )
}
