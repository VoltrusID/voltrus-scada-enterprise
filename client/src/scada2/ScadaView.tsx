import React, { useReducer, useState, useCallback, useRef, useEffect } from 'react'
import type { ScadaWidgetType, ScadaCanvasState, ConnectionInstance } from './widgets/types'
import { WIDGET_MANIFESTS, getCompatibleWidgetTypes, getWidgetComponent } from './widgets/registry'
import type { ConfigFieldDef } from './widgets/types'
import { ScadaCanvas } from './ScadaCanvas'
import { ScadaPalette } from './ScadaPalette'
import { ConnectionLayer } from './connections/ConnectionLayer'
import { layoutReducer, createWidget, createConnection } from './engine/widgetState'
import type { ScadaLayoutState } from './engine/widgetState'
import type { SensorConfig, SensorData, ScadaEquipmentItem, ScadaConnectionItem } from '../types'
import { useDashboardSelector } from '../store/dashboardStore'
import { wsWriteTag } from '../api/websocket'
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
  toolbarSlotSetter?: (slot: React.ReactNode) => void
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
  toolbarSlotSetter,
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

  // Write value to tag via WebSocket (direct write), fall back to REST API
  const handleWriteValue = useCallback(async (_widgetId: string, sensorId: number, value: number) => {
    try {
      if (wsWriteTag(sensorId, value)) return
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

  // Palette tag click: in edit mode, show type picker; in live mode, open sensor detail
  const handlePaletteSelect = useCallback((sensorId: number) => {
    if (canvasState.editMode) {
      setCanvasState(prev => ({ ...prev, pendingSensor: { sensorId, x: 200, y: 200 } }))
    } else {
      onSensorClick(sensorId)
    }
  }, [canvasState.editMode, onSensorClick])

  // Toolbar icon button style helper
  const tBtn = (active: boolean, color?: string): React.CSSProperties => ({
    padding: '3px 8px', borderRadius: 5, display: 'inline-flex', alignItems: 'center', gap: 4,
    border: `1px solid ${active ? (color ?? 'var(--cyan)') : 'var(--border)'}`,
    background: active ? `${color ?? 'var(--cyan)'}15` : 'transparent',
    color: active ? (color ?? 'var(--cyan)') : 'var(--fg)',
    cursor: 'pointer', fontSize: 11, fontWeight: 500, whiteSpace: 'nowrap',
  })

  const IS = { width: 14, height: 14, flexShrink: 0 } as const

  // Push toolbar into parent ViewToolbar via slot
  useEffect(() => {
    if (!toolbarSlotSetter) return
    if (!isAdmin) { toolbarSlotSetter(null); return }

    toolbarSlotSetter(
      <>
        {canvasState.editMode && (
          <>
            <div style={{ width: 1, height: 16, background: 'var(--border)', margin: '0 4px' }} />
            <button onClick={() => handleToolChange('select')} title="Select" style={tBtn(canvasState.activeTool === 'select')}>
              <svg style={IS} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z"/><path d="M13 13l6 6"/></svg>
              Select
            </button>
            <button onClick={() => handleToolChange('connect')} title="Connect" style={tBtn(canvasState.activeTool === 'connect', 'var(--ok)')}>
              <svg style={IS} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>
              Connect
            </button>
            <button onClick={() => handleToolChange('delete')} title="Delete Widget" style={tBtn(canvasState.activeTool === 'delete', '#ef4444')}>
              <svg style={IS} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
              Del Widget
            </button>
            <button onClick={() => handleToolChange('delete_connection')} title="Delete Line" style={tBtn(canvasState.activeTool === 'delete_connection', '#f97316')}>
              <svg style={IS} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><line x1="20" y1="4" x2="8.12" y2="15.88"/><line x1="14.47" y1="14.48" x2="20" y2="20"/><line x1="8.12" y1="8.12" x2="12" y2="12"/></svg>
              Del Line
            </button>
            <div style={{ width: 1, height: 16, background: 'var(--border)' }} />
            <button onClick={handleSave} disabled={!dirtyRef.current} title="Save" style={{
              ...tBtn(!!dirtyRef.current, 'var(--cyan)'),
              opacity: dirtyRef.current ? 1 : 0.4,
              cursor: dirtyRef.current ? 'pointer' : 'not-allowed',
            }}>
              <svg style={IS} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
              Save
            </button>
          </>
        )}
        <div style={{ width: 1, height: 16, background: 'var(--border)', margin: '0 4px' }} />
        <button onClick={handleToggleEdit} title={canvasState.editMode ? 'Exit edit mode' : 'Enter edit mode'} style={{
          padding: '3px 8px', borderRadius: 5, display: 'inline-flex', alignItems: 'center', gap: 4,
          border: `1px solid ${canvasState.editMode ? 'var(--warn)' : 'var(--border)'}`,
          background: canvasState.editMode ? 'rgba(245,158,11,.1)' : 'transparent',
          color: canvasState.editMode ? 'var(--warn)' : 'var(--fg)',
          cursor: 'pointer', fontSize: 11, fontWeight: 600, whiteSpace: 'nowrap',
        }}>
          {canvasState.editMode
            ? <svg style={IS} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            : <svg style={IS} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
          }
          {canvasState.editMode ? 'Done' : 'Edit'}
        </button>
      </>
    )
    return () => toolbarSlotSetter(null)
  }, [canvasState.editMode, canvasState.activeTool, dirtyRef.current, isAdmin, toolbarSlotSetter])

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      <div style={{ flex: 1, position: 'relative' }}>
        <ScadaPalette
          visible={canvasState.editMode}
          sensors={sensors}
          onSelect={handlePaletteSelect}
        />
        <div style={{
          position: 'absolute', top: 0, left: 0, bottom: 0,
          ...(canvasState.pendingSensor || (canvasState.editMode && canvasState.selectedId) ? { right: 300 } : { right: 0 }),
          transition: 'right 0.2s ease',
        }}>
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
        </div>

        {/* Right sidebar: type picker OR config panel */}
        {canvasState.pendingSensor && (
          <WidgetTypeSidebar
            sensorId={canvasState.pendingSensor.sensorId}
            sensors={sensors}
            onSelect={handlePickWidgetType}
            onClose={handleCancelPick}
          />
        )}
        {!canvasState.pendingSensor && canvasState.editMode && canvasState.selectedId && (() => {
          const widget = layout.widgets.find(w => w.id === canvasState.selectedId)
          if (!widget) return null
          return (
            <WidgetConfigSidebar
              widget={widget}
              sensors={sensors}
              onUpdateConfig={(config) => {
                dispatch({ type: 'UPDATE_WIDGET_CONFIG', id: widget.id, config })
                dirtyRef.current = true
              }}
              onClose={() => setCanvasState(prev => ({ ...prev, selectedId: null }))}
            />
          )
        })()}
      </div>
    </div>
  )
}

// ── Right sidebar: Widget Type Picker ──

const sidebarStyle: React.CSSProperties = {
  position: 'absolute', top: 0, right: 0, bottom: 0, width: 300,
  background: 'var(--card)', borderLeft: '1px solid var(--border)',
  display: 'flex', flexDirection: 'column', zIndex: 20,
  overflow: 'hidden',
}

const WidgetTypeSidebar: React.FC<{
  sensorId: number
  sensors: SensorConfig[]
  onSelect: (type: ScadaWidgetType) => void
  onClose: () => void
}> = ({ sensorId, sensors, onSelect, onClose }) => {
  const sensor = sensors.find(s => s.id === sensorId)
  const isAnalog = sensor ? (sensor.max_val !== sensor.min_val) : true
  const types = getCompatibleWidgetTypes(isAnalog)

  const sampleSensor: SensorConfig = sensor ?? {
    id: sensorId, key: 'sample', name: 'Sample', unit: '', min_val: 0, max_val: 100, warn_val: 75, crit_val: 90,
  } as any

  const sampleData: SensorData = {
    id: sensorId, key: sampleSensor.key,
    value: ((sampleSensor.max_val ?? 100) - (sampleSensor.min_val ?? 0)) * 0.6 + (sampleSensor.min_val ?? 0),
    warn: sampleSensor.warn_val ?? 75, crit: sampleSensor.crit_val ?? 90, source_type: '', status: '',
  }
  const sampleValues: Record<string, SensorData> = { [sampleSensor.key]: sampleData }

  return (
    <div style={sidebarStyle}>
      <div style={{
        padding: '10px 12px', borderBottom: '1px solid var(--border)',
        display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexShrink: 0,
      }}>
        <div>
          <div style={{ fontSize: 12, fontWeight: 600 }}>Choose Visualization</div>
          <div style={{ fontSize: 10, color: 'var(--muted)', marginTop: 2 }}>{sensor?.name ?? `Tag #${sensorId}`}</div>
        </div>
        <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 16, lineHeight: 1 }}>x</button>
      </div>
      <div style={{ flex: 1, overflowY: 'auto', padding: 8 }}>
        {types.map(type => {
          const m = WIDGET_MANIFESTS[type]
          const Comp = getWidgetComponent(type)
          return (
            <div
              key={type}
              onClick={() => onSelect(type)}
              style={{
                display: 'flex', alignItems: 'center', gap: 10,
                padding: 8, borderRadius: 6, marginBottom: 4, cursor: 'pointer',
                border: '1px solid var(--border)', transition: 'border-color 0.15s',
              }}
              onMouseEnter={e => e.currentTarget.style.borderColor = 'var(--cyan)'}
              onMouseLeave={e => e.currentTarget.style.borderColor = 'var(--border)'}
            >
              {/* Mini preview */}
              <div style={{
                width: 72, height: 48, flexShrink: 0, borderRadius: 4,
                overflow: 'hidden', background: 'var(--bg)', border: '1px solid var(--border)',
              }}>
                {Comp && (
                  <Comp
                    instance={{
                      id: '__preview__', type, sensorId, sensorIds: [sensorId],
                      x: 0, y: 0, width: 72, height: 48, config: {},
                    }}
                    sensor={sampleSensor}
                    data={sampleData}
                    sensors={[sampleSensor]}
                    currentValues={sampleValues}
                    editMode={false}
                    selected={false}
                  />
                )}
              </div>
              <div>
                <div style={{ fontSize: 12, fontWeight: 500 }}>{m.displayName}</div>
                <div style={{ fontSize: 10, color: 'var(--muted)' }}>{m.icon} {m.compatibleDataTypes.join(', ')}</div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

// ── Right sidebar: Widget Config ──

const WidgetConfigSidebar: React.FC<{
  widget: import('./widgets/types').WidgetInstance
  sensors: SensorConfig[]
  onUpdateConfig: (config: Record<string, any>) => void
  onClose: () => void
}> = ({ widget, sensors, onUpdateConfig, onClose }) => {
  const manifest = WIDGET_MANIFESTS[widget.type]
  const sensor = widget.sensorId != null ? sensors.find(s => s.id === widget.sensorId) : undefined
  const schema: ConfigFieldDef[] = manifest?.configSchema ?? []
  const config = widget.config ?? {}

  const inputStyle: React.CSSProperties = {
    width: '100%', padding: '5px 7px', borderRadius: 4, fontSize: 12,
    border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', boxSizing: 'border-box',
  }

  const labelStyle: React.CSSProperties = {
    fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase',
    letterSpacing: '.04em', display: 'block', marginBottom: 3,
  }

  return (
    <div style={sidebarStyle}>
      <div style={{
        padding: '10px 12px', borderBottom: '1px solid var(--border)',
        display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexShrink: 0,
      }}>
        <div>
          <div style={{ fontSize: 12, fontWeight: 600 }}>{manifest?.displayName ?? widget.type}</div>
          <div style={{ fontSize: 10, color: 'var(--muted)', marginTop: 2 }}>{sensor?.name ?? 'No sensor'}</div>
        </div>
        <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 16, lineHeight: 1 }}>x</button>
      </div>
      <div style={{ flex: 1, overflowY: 'auto', padding: 12 }}>
        <div style={{ marginBottom: 12 }}>
          <span style={labelStyle}>Type</span>
          <div style={{ fontSize: 12, textTransform: 'capitalize' }}>{widget.type.replace(/_/g, ' ')}</div>
        </div>
        <div style={{ marginBottom: 12 }}>
          <span style={labelStyle}>Position</span>
          <div style={{ fontSize: 11, color: 'var(--muted)' }}>{widget.x}, {widget.y} — {widget.width}x{widget.height}</div>
        </div>
        {schema.length > 0 && (
          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 10 }}>
            <div style={{ ...labelStyle, marginBottom: 8 }}>Configuration</div>
            {schema.map(field => {
              const val = config[field.key] ?? field.default
              return (
                <div key={field.key} style={{ marginBottom: 10 }}>
                  <span style={labelStyle}>{field.label}</span>
                  {field.type === 'boolean' ? (
                    <label style={{ display: 'flex', alignItems: 'center', gap: 6, cursor: 'pointer' }}>
                      <input
                        type="checkbox"
                        checked={!!val}
                        onChange={e => onUpdateConfig({ [field.key]: e.target.checked })}
                      />
                      <span style={{ fontSize: 12 }}>{val ? 'On' : 'Off'}</span>
                    </label>
                  ) : field.type === 'color' ? (
                    <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
                      <input
                        type="color"
                        value={val.startsWith('var(') ? '#22d3ee' : val}
                        onChange={e => onUpdateConfig({ [field.key]: e.target.value })}
                        style={{ width: 28, height: 28, border: 'none', background: 'none', cursor: 'pointer', padding: 0 }}
                      />
                      <input
                        value={val}
                        onChange={e => onUpdateConfig({ [field.key]: e.target.value })}
                        style={inputStyle}
                      />
                    </div>
                  ) : field.type === 'select' ? (
                    <select
                      value={val}
                      onChange={e => onUpdateConfig({ [field.key]: e.target.value })}
                      style={inputStyle}
                    >
                      {field.options?.map(o => (
                        <option key={o.value} value={o.value}>{o.label}</option>
                      ))}
                    </select>
                  ) : (
                    <input
                      type={field.type === 'number' ? 'number' : 'text'}
                      value={val}
                      min={field.min}
                      max={field.max}
                      step={field.type === 'number' ? 'any' : undefined}
                      onChange={e => onUpdateConfig({
                        [field.key]: field.type === 'number' ? Number(e.target.value) : e.target.value,
                      })}
                      style={inputStyle}
                    />
                  )}
                </div>
              )
            })}
          </div>
        )}
      </div>
    </div>
  )
}
