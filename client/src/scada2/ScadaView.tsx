import React, { useState, useCallback, useRef, useEffect } from 'react'
import { Point, Group } from 'fabric'
import type { SensorConfig, ScadaEquipmentItem, ScadaConnectionItem } from '../types'
import type { ScadaCanvasState } from './engine/types'
import type { ScadaEquipmentProps, ScadaConnectionProps, ScadaLabelProps } from './engine/types'
import { ScadaCanvas } from './ScadaCanvas'
import { ScadaToolbar } from './ScadaToolbar'
import { ScadaPalette } from './ScadaPalette'
import { ScadaEditPanel } from './ScadaEditPanel'
import { loadFromAPI, exportToAPI } from './engine/serializer'
import { useDashboardSelector } from '../store/dashboardStore'

type SelectedObject =
  | { kind: 'equipment'; props: ScadaEquipmentProps }
  | { kind: 'connection'; props: ScadaConnectionProps }
  | { kind: 'label'; props: ScadaLabelProps }

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
  pendingType: null,
  connectFrom: null,
}

export const ScadaView: React.FC<Props> = ({
  sensors,
  activeScreen,
  scadaEquipment,
  scadaConnections,
  onSaveEquipment,
  onSaveConnections,
  onSensorClick,
}) => {
  const currentValues = useDashboardSelector(s => s.currentValues)

  const [editMode, setEditMode] = useState(false)
  const [canvasState, setCanvasState] = useState<ScadaCanvasState>(initialCanvasState)
  const dirtyRef = useRef(false)

  const getState = useCallback(() => ({ currentValues }), [currentValues])

  // Load equipment/connections when screen changes
  useEffect(() => {
    const cv = (window as any).__scadaCanvas
    if (!cv) return
    loadFromAPI(cv, scadaEquipment, scadaConnections)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeScreen])

  // Keep canvasState.editMode in sync
  useEffect(() => {
    setCanvasState(prev => ({ ...prev, editMode }))
  }, [editMode])

  const handleStateChange = useCallback((update: Partial<ScadaCanvasState>) => {
    setCanvasState(prev => ({ ...prev, ...update }))
  }, [])

  const handleSaveNeeded = useCallback(() => {
    dirtyRef.current = true
  }, [])

  // Save
  const handleSave = useCallback(() => {
    const cv = (window as any).__scadaCanvas
    if (!cv) return
    const { equipment, connections } = exportToAPI(cv)
    onSaveEquipment(equipment)
    onSaveConnections(connections)
    dirtyRef.current = false
  }, [onSaveEquipment, onSaveConnections])

  // Toggle edit mode
  const handleToggleEdit = useCallback(() => {
    if (editMode) {
      setCanvasState(prev => ({
        ...prev,
        editMode: false,
        activeTool: 'select',
        pendingType: null,
        connectFrom: null,
      }))
    } else {
      setCanvasState(prev => ({
        ...prev,
        editMode: true,
      }))
    }
    setEditMode(prev => !prev)
  }, [editMode])

  // Tool change
  const handleToolChange = useCallback((tool: ScadaCanvasState['activeTool']) => {
    setCanvasState(prev => ({
      ...prev,
      activeTool: tool,
      pendingType: tool === 'select' ? null : prev.pendingType,
      connectFrom: null,
    }))
  }, [])

  // Palette selection
  const handlePaletteSelect = useCallback((type: ScadaCanvasState['pendingType']) => {
    setCanvasState(prev => ({
      ...prev,
      pendingType: type,
      activeTool: 'select',
    }))
  }, [])

  // Zoom controls
  const handleZoomIn = useCallback(() => {
    const cv = (window as any).__scadaCanvas
    if (!cv) return
    const zoom = Math.min(cv.getZoom() * 1.2, 5)
    const center = new Point(cv.getWidth() / 2, cv.getHeight() / 2)
    cv.zoomToPoint(center, zoom)
  }, [])

  const handleZoomOut = useCallback(() => {
    const cv = (window as any).__scadaCanvas
    if (!cv) return
    const zoom = Math.max(cv.getZoom() / 1.2, 0.1)
    const center = new Point(cv.getWidth() / 2, cv.getHeight() / 2)
    cv.zoomToPoint(center, zoom)
  }, [])

  const handleFitView = useCallback(() => {
    const cv = (window as any).__scadaCanvas
    if (!cv) return
    const objects = cv.getObjects().filter((o: any) => o._scada && !o._isGrid)
    if (objects.length === 0) {
      cv.setViewportTransform([1, 0, 0, 1, 0, 0])
      cv.requestRenderAll()
      return
    }
    const group = new Group(objects)
    const bounds = group.getBoundingRect(true)
    group.destroy()
    const padding = 40
    const scaleX = (cv.getWidth() - padding * 2) / bounds.width
    const scaleY = (cv.getHeight() - padding * 2) / bounds.height
    const zoom = Math.min(scaleX, scaleY, 2)
    const center = new Point(cv.getWidth() / 2, cv.getHeight() / 2)
    cv.setViewportTransform([1, 0, 0, 1, 0, 0])
    cv.zoomToPoint(center, zoom)
    const vpt = cv.viewportTransform!
    vpt[4] = cv.getWidth() / 2 - (bounds.left + bounds.width / 2) * zoom
    vpt[5] = cv.getHeight() / 2 - (bounds.top + bounds.height / 2) * zoom
    cv.requestRenderAll()
  }, [])

  // Derive selectedObject from canvasState.selectedId
  const selectedObject: SelectedObject | null = (() => {
    if (!canvasState.selectedId) return null
    const cv = (window as any).__scadaCanvas
    if (!cv) return null
    const obj = cv.getObjects().find((o: any) => o._scada?.id === canvasState.selectedId)
    if (!obj) return null
    const scada = (obj as any)._scada
    if (scada.type === 'ScadaEquipment') {
      return { kind: 'equipment', props: scada as ScadaEquipmentProps }
    }
    if (scada.type === 'ScadaConnection') {
      return { kind: 'connection', props: scada as ScadaConnectionProps }
    }
    if (scada.type === 'ScadaLabel') {
      return { kind: 'label', props: scada as ScadaLabelProps }
    }
    return null
  })()

  // Edit panel update: patch _scada metadata on the canvas object
  const handleEditUpdate = useCallback((updates: Record<string, any>) => {
    const cv = (window as any).__scadaCanvas
    if (!cv || !canvasState.selectedId) return
    const obj = cv.getObjects().find((o: any) => o._scada?.id === canvasState.selectedId)
    if (!obj) return
    Object.assign((obj as any)._scada, updates)
    dirtyRef.current = true
    cv.requestRenderAll()
  }, [canvasState.selectedId])

  const handleCloseEditPanel = useCallback(() => {
    setCanvasState(prev => ({ ...prev, selectedId: null }))
  }, [])

  return (
    <div style={{ position: 'relative' }}>
      <ScadaToolbar
        editMode={canvasState.editMode}
        activeTool={canvasState.activeTool}
        onToggleEdit={handleToggleEdit}
        onToolChange={handleToolChange}
        onSave={handleSave}
        hasChanges={dirtyRef.current}
        onZoomIn={handleZoomIn}
        onZoomOut={handleZoomOut}
        onFitView={handleFitView}
      />
      <div style={{ position: 'relative' }}>
        <ScadaPalette
          visible={canvasState.editMode && canvasState.activeTool === 'select'}
          onSelect={handlePaletteSelect}
          selectedType={canvasState.pendingType}
        />
        <ScadaCanvas
          sensors={sensors}
          getState={getState}
          canvasState={canvasState}
          onStateChange={handleStateChange}
          onSaveNeeded={handleSaveNeeded}
          onSensorClick={onSensorClick}
        />
        {canvasState.editMode && selectedObject && (
          <ScadaEditPanel
            selected={selectedObject}
            sensors={sensors}
            onUpdate={handleEditUpdate}
            onClose={handleCloseEditPanel}
          />
        )}
      </div>
    </div>
  )
}
