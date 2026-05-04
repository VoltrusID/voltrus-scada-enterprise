import { useEffect, useRef } from 'react'
import { Canvas, FabricObject, Polyline, Point } from 'fabric'
import type { SensorConfig, SensorData } from '../types'
import type { ScadaCanvasState } from './engine/types'
import { createEquipmentObject } from './objects/ScadaEquipment'
import { updateConnectionPoints } from './objects/ScadaConnection'
import { createLabelObject } from './objects/ScadaLabel'
import { updateCanvasObjects } from './engine/render-loop'

const GRID_SIZE = 20

interface Props {
  sensors: SensorConfig[]
  getState: () => { currentValues: Record<string, SensorData> }
  canvasState: ScadaCanvasState
  onStateChange: (update: Partial<ScadaCanvasState>) => void
  onSaveNeeded: () => void
  onSensorClick: (sensorId: number) => void
}

export const ScadaCanvas: React.FC<Props> = ({
  sensors,
  getState,
  canvasState,
  onStateChange,
  onSaveNeeded,
  onSensorClick,
}) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<Canvas | null>(null)
  const stateRef = useRef(canvasState)
  stateRef.current = canvasState

  // ---------- Init effect ----------
  useEffect(() => {
    if (!containerRef.current) return

    const container = containerRef.current
    const canvasEl = container.querySelector('canvas') as HTMLCanvasElement

    const canvas = new Canvas(canvasEl, {
      width: container.clientWidth,
      height: container.clientHeight,
      backgroundColor: 'transparent',
      selection: true,
      preserveObjectStacking: true,
    })

    canvasRef.current = canvas
    ;(window as any).__scadaCanvas = canvas

    // Resize observer
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect
        canvas.setDimensions({ width, height })
        canvas.requestRenderAll()
      }
    })
    ro.observe(container)

    // Store-driven updates (no RAF polling)
    let lastValues: Record<string, SensorData> = {}
    const interval = setInterval(() => {
      const { currentValues } = getState()
      if (currentValues !== lastValues) {
        lastValues = currentValues
        updateCanvasObjects(canvas, currentValues, sensors)
      }
    }, 250)

    return () => {
      clearInterval(interval)
      ro.disconnect()
      canvas.dispose()
      canvasRef.current = null
      ;(window as any).__scadaCanvas = null
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // ---------- Tool handling effect ----------
  useEffect(() => {
    const cv = canvasRef.current
    if (!cv) return

    const { editMode, activeTool } = canvasState
    const isSelectMode = editMode && activeTool === 'select'

    cv.getObjects().forEach((obj) => {
      if ((obj as any)._isGrid) return
      if (isSelectMode) {
        obj.set({
          selectable: true,
          evented: true,
          hasControls: true,
          hasBorders: true,
        })
      } else {
        obj.set({
          selectable: false,
          evented: editMode,
          hasControls: false,
          hasBorders: false,
        })
      }
    })
    cv.requestRenderAll()
  }, [canvasState.editMode, canvasState.activeTool])

  // ---------- Selection effect ----------
  useEffect(() => {
    const cv = canvasRef.current
    if (!cv) return

    const handleSelection = () => {
      const active = cv.getActiveObject()
      const scada = active ? (active as any)._scada : null
      onStateChange({ selectedId: scada?.id ?? null })
    }

    const handleCleared = () => {
      onStateChange({ selectedId: null })
    }

    cv.on('selection:created', handleSelection)
    cv.on('selection:updated', handleSelection)
    cv.on('selection:cleared', handleCleared)

    return () => {
      cv.off('selection:created', handleSelection)
      cv.off('selection:updated', handleSelection)
      cv.off('selection:cleared', handleCleared)
    }
  }, [onStateChange])

  // ---------- Object modified + moving connections ----------
  useEffect(() => {
    const cv = canvasRef.current
    if (!cv) return

    const handleModified = () => {
      onSaveNeeded()
    }

    const handleMoving = () => {
      cv.getObjects().forEach((obj) => {
        if (obj instanceof Polyline && (obj as any)._scada?.type === 'ScadaConnection') {
          updateConnectionPoints(obj, cv)
        }
      })
      cv.requestRenderAll()
    }

    cv.on('object:modified', handleModified)
    cv.on('object:moving', handleMoving)

    return () => {
      cv.off('object:modified', handleModified)
      cv.off('object:moving', handleMoving)
    }
  }, [onSaveNeeded])

  // ---------- Click to add ----------
  useEffect(() => {
    const cv = canvasRef.current
    if (!cv) return

    const handleMouseDown = async (opt: any) => {
      if (!stateRef.current.editMode) return
      if (opt.target) return

      const pointer = cv.getScenePoint(opt.e)
      const st = stateRef.current

      if (st.pendingType) {
        const group = await createEquipmentObject(st.pendingType, {
          x: pointer.x,
          y: pointer.y,
        })
        cv.add(group)
        cv.setActiveObject(group)
        cv.requestRenderAll()
        onStateChange({ pendingType: null })
        onSaveNeeded()
        return
      }

      if (st.activeTool === 'label') {
        const label = createLabelObject({
          x: pointer.x,
          y: pointer.y,
        })
        cv.add(label)
        cv.setActiveObject(label)
        cv.requestRenderAll()
        onSaveNeeded()
        return
      }
    }

    cv.on('mouse:down', handleMouseDown)

    return () => {
      cv.off('mouse:down', handleMouseDown)
    }
  }, [onStateChange, onSaveNeeded])

  // ---------- Zoom ----------
  useEffect(() => {
    const cv = canvasRef.current
    if (!cv) return

    const handleWheel = (opt: any) => {
      const delta = opt.e.deltaY
      let zoom = cv.getZoom()
      zoom *= 0.999 ** delta
      zoom = Math.min(Math.max(0.1, zoom), 5)

      cv.zoomToPoint(
        new Point(opt.e.offsetX, opt.e.offsetY),
        zoom,
      )
      opt.e.preventDefault()
      opt.e.stopPropagation()
    }

    cv.on('mouse:wheel', handleWheel)

    return () => {
      cv.off('mouse:wheel', handleWheel)
    }
  }, [])

  // ---------- Pan (middle mouse or alt+left) ----------
  useEffect(() => {
    const cv = canvasRef.current
    if (!cv) return

    let isPanning = false
    let lastPosX = 0
    let lastPosY = 0

    const handleMiddleDown = (opt: any) => {
      const e = opt.e as MouseEvent
      if (e.button === 1) {
        isPanning = true
        lastPosX = e.clientX
        lastPosY = e.clientY
        cv.selection = false
        e.preventDefault()
      }
    }

    const handleAltDown = (opt: any) => {
      const e = opt.e as MouseEvent
      if (e.altKey && e.button === 0) {
        isPanning = true
        lastPosX = e.clientX
        lastPosY = e.clientY
        cv.selection = false
        e.preventDefault()
        e.stopPropagation()
      }
    }

    const handleMove = (opt: any) => {
      if (!isPanning) return
      const e = opt.e as MouseEvent
      const vpt = cv.viewportTransform!
      vpt[4] += e.clientX - lastPosX
      vpt[5] += e.clientY - lastPosY
      lastPosX = e.clientX
      lastPosY = e.clientY
      cv.requestRenderAll()
      e.preventDefault()
      e.stopPropagation()
    }

    const handleUp = () => {
      isPanning = false
      cv.selection = true
    }

    cv.on('mouse:down', handleMiddleDown)
    cv.on('mouse:down', handleAltDown)
    cv.on('mouse:move', handleMove)
    cv.on('mouse:up', handleUp)

    return () => {
      cv.off('mouse:down', handleMiddleDown)
      cv.off('mouse:down', handleAltDown)
      cv.off('mouse:move', handleMove)
      cv.off('mouse:up', handleUp)
    }
  }, [])

  return (
    <div style={{
      height: 'calc(100vh - 180px)',
      border: '1px solid var(--border)',
      borderRadius: 8,
      overflow: 'hidden',
      backgroundColor: '#060e09',
      backgroundImage: `linear-gradient(to right, #0d1a12 1px, transparent 1px), linear-gradient(to bottom, #0d1a12 1px, transparent 1px)`,
      backgroundSize: `${GRID_SIZE}px ${GRID_SIZE}px`,
    }}>
      <div ref={containerRef} style={{ width: '100%', height: '100%' }}>
        <canvas />
      </div>
    </div>
  )
}
