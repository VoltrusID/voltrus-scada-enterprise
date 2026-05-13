import React, { useReducer, useState, useCallback, useRef, useEffect } from 'react'
import type { ScadaWidgetType, ScadaCanvasState, ConnectionInstance } from './widgets/types'
import { WIDGET_MANIFESTS } from './widgets/registry'
import { ScadaCanvas, WidgetTypePicker } from './ScadaCanvas'
import { ScadaPalette } from './ScadaPalette'
import { ConnectionLayer } from './connections/ConnectionLayer'
import { layoutReducer, createWidget, createConnection } from './engine/widgetState'
import type { ScadaLayoutState } from './engine/widgetState'
import type { SensorConfig, SensorData, ScadaEquipmentItem, ScadaConnectionItem } from '../types'
import { useDashboardSelector } from '../store/dashboardStore'
import { writeTag } from '../api/commands'
import { toast } from '../components/Toast'

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
import './widgets/SliderWidget'
import './widgets/NumberInputWidget'
import './widgets/ToggleWidget'
import './widgets/MeterWidget'
import './widgets/SemicircleGaugeWidget'
import './widgets/DonutWidget'
import './widgets/LEDDisplayWidget'
import './widgets/MotorWidget'
import './widgets/ValveWidget'
import './widgets/PumpWidget'
import './widgets/PushButtonWidget'
import './widgets/BarChartWidget'
import './widgets/PieChartWidget'
import './widgets/MultiTrendWidget'
import './widgets/AnnunciatorWidget'
import './widgets/PIDFaceplateWidget'
import './widgets/HorizontalTankWidget'
import './widgets/ProgressBarWidget'

interface Props {
  sensors: SensorConfig[]
  activeScreen: string
  scadaEquipment: ScadaEquipmentItem[]
  scadaConnections: ScadaConnectionItem[]
  onSaveEquipment: (items: ScadaEquipmentItem[]) => void
  onSaveConnections: (items: ScadaConnectionItem[]) => void
  onSensorClick: (sensorId: number) => void
  isAdmin: boolean
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
  onSaveEquipment, onSaveConnections, onSensorClick, isAdmin,
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

  // Write value to tag
  const handleWriteValue = useCallback(async (_widgetId: string, sensorId: number, value: number) => {
    try {
      await writeTag(sensorId, value)
    } catch (err: any) {
      toast.error(err.message || 'Write failed')
    }
  }, [])

  // Delete a connection
  const handleDeleteConnection = useCallback((id: string) => {
    dispatch({ type: 'REMOVE_CONNECTION', id })
    dirtyRef.current = true
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

  const toolBtnStyle = (active: boolean): React.CSSProperties => ({
    padding: '4px 10px', borderRadius: 6,
    border: `1px solid ${active ? 'var(--cyan)' : 'var(--border)'}`,
    background: active ? 'rgba(34,197,94,.1)' : 'var(--card)',
    color: active ? 'var(--cyan)' : 'var(--fg)',
    cursor: 'pointer', fontSize: 11, fontWeight: 500,
  })

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      {/* Inline toolbar */}
      <div style={{
        display: 'flex', alignItems: 'center', gap: 6,
        padding: '6px 12px',
        borderBottom: '1px solid var(--border)',
        background: 'var(--card)',
        flexShrink: 0,
      }}>
        {canvasState.editMode && (
          <>
            <button onClick={() => handleToolChange('select')} style={toolBtnStyle(canvasState.activeTool === 'select')}>Select</button>
            <button onClick={() => handleToolChange('connect')} style={toolBtnStyle(canvasState.activeTool === 'connect')}>Connect</button>
            <button onClick={() => handleToolChange('delete')} style={toolBtnStyle(canvasState.activeTool === 'delete')}>Delete Widget</button>
            <button onClick={() => handleToolChange('delete_connection')} style={{
              ...toolBtnStyle(canvasState.activeTool === 'delete_connection'),
              borderColor: canvasState.activeTool === 'delete_connection' ? '#ef4444' : 'var(--border)',
              color: canvasState.activeTool === 'delete_connection' ? '#ef4444' : 'var(--fg)',
            }}>Delete Line</button>
            <div style={{ width: 1, height: 18, background: 'var(--border)' }} />
            <button onClick={handleSave} disabled={!dirtyRef.current} style={{
              padding: '4px 10px', borderRadius: 6,
              border: `1px solid ${dirtyRef.current ? 'var(--cyan)' : 'var(--border)'}`,
              background: dirtyRef.current ? 'var(--cyan)' : 'var(--card)',
              color: dirtyRef.current ? '#fff' : 'var(--muted)',
              cursor: dirtyRef.current ? 'pointer' : 'not-allowed',
              fontSize: 11, fontWeight: 600,
            }}>Save</button>
          </>
        )}
        {isAdmin && (
        <button onClick={handleToggleEdit} style={{
          marginLeft: canvasState.editMode ? 'auto' : 0,
          padding: '4px 10px', borderRadius: 6,
          border: `1px solid ${canvasState.editMode ? 'var(--warn)' : 'var(--border)'}`,
          background: canvasState.editMode ? 'rgba(245,158,11,.1)' : 'var(--card)',
          color: canvasState.editMode ? 'var(--warn)' : 'var(--fg)',
          cursor: 'pointer', fontSize: 11, fontWeight: 600,
        }}>
          {canvasState.editMode ? 'Done' : 'Edit'}
        </button>
        )}
      </div>
      <div style={{ flex: 1, position: 'relative' }}>
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
          onWriteValue={handleWriteValue}
          onClickEmpty={() => setCanvasState(prev => ({ ...prev, selectedId: null }))}
        >
          <ConnectionLayer
            widgets={layout.widgets}
            connections={layout.connections}
            editMode={canvasState.editMode}
            onDeleteConnection={canvasState.editMode ? handleDeleteConnection : undefined}
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
