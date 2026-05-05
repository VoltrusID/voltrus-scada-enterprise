import React, { useCallback, useRef } from 'react'
import type { WidgetInstance, ScadaWidgetType } from './widgets/types'
import type { ScadaLayoutState } from './engine/widgetState'
import { WIDGET_MANIFESTS, getCompatibleWidgetTypes } from './widgets/registry'
import WidgetShell from './widgets/WidgetShell'
import type { SensorConfig, SensorData } from '../types'

const GRID_SIZE = 20

interface Props {
  layout: ScadaLayoutState
  sensors: SensorConfig[]
  currentValues: Record<string, SensorData>
  editMode: boolean
  selectedId: string | null
  onSelectWidget: (id: string | null) => void
  onMoveWidget: (id: string, x: number, y: number) => void
  onResizeWidget: (id: string, x: number, y: number, width: number, height: number) => void
  onDropSensor: (sensorId: number, x: number, y: number) => void
  onClickEmpty: () => void
  children?: React.ReactNode  // for ConnectionLayer overlay
}

export const ScadaCanvas: React.FC<Props> = ({
  layout, sensors, currentValues, editMode, selectedId,
  onSelectWidget, onMoveWidget, onResizeWidget, onDropSensor, onClickEmpty, children,
}) => {
  const containerRef = useRef<HTMLDivElement>(null)

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    const sensorId = parseInt(e.dataTransfer.getData('text/plain'), 10)
    if (isNaN(sensorId)) return
    const rect = containerRef.current?.getBoundingClientRect()
    if (!rect) return
    const x = Math.round((e.clientX - rect.left) / GRID_SIZE) * GRID_SIZE
    const y = Math.round((e.clientY - rect.top) / GRID_SIZE) * GRID_SIZE
    onDropSensor(sensorId, x, y)
  }, [onDropSensor])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'copy'
  }, [])

  const sensorMap = new Map(sensors.map(s => [s.id, s]))

  return (
    <div
      ref={containerRef}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onClick={e => {
        // Only deselect if clicking the canvas background, not a widget
        if (e.target === containerRef.current) onClickEmpty()
      }}
      style={{
        position: 'relative',
        height: 'calc(100vh - 180px)',
        border: '1px solid var(--border)',
        borderRadius: 8,
        overflow: 'hidden',
        background: 'var(--bg)',
        backgroundImage: editMode
          ? `linear-gradient(to right, var(--border) 1px, transparent 1px), linear-gradient(to bottom, var(--border) 1px, transparent 1px)`
          : 'none',
        backgroundSize: `${GRID_SIZE}px ${GRID_SIZE}px`,
        transition: 'background 0.2s',
      }}
    >
      {/* Connection layer (SVG overlay) */}
      {children}

      {/* Widget shells */}
      {layout.widgets.map(w => {
        const sensor = w.sensorId != null ? sensorMap.get(w.sensorId) : undefined
        const data = w.sensorId != null ? currentValues[w.sensorId] : undefined
        return (
          <WidgetShell
            key={w.id}
            instance={w}
            sensor={sensor}
            data={data}
            editMode={editMode}
            selected={selectedId === w.id}
            onSelect={onSelectWidget}
            onMove={onMoveWidget}
            onResize={onResizeWidget}
          />
        )
      })}
    </div>
  )
}

// ── Widget Type Picker ──

interface TypePickerProps {
  x: number
  y: number
  sensorId: number
  sensors: SensorConfig[]
  onSelect: (type: ScadaWidgetType) => void
  onCancel: () => void
}

export const WidgetTypePicker: React.FC<TypePickerProps> = ({ x, y, sensorId, sensors, onSelect, onCancel }) => {
  const sensor = sensors.find(s => s.id === sensorId)
  const isAnalog = sensor ? (sensor.max_val !== sensor.min_val) : true
  const types = getCompatibleWidgetTypes(isAnalog)

  return (
    <>
      {/* Backdrop */}
      <div style={{ position: 'fixed', inset: 0, zIndex: 99 }} onClick={onCancel} />
      {/* Picker */}
      <div style={{
        position: 'absolute', left: x, top: y, zIndex: 100,
        background: 'var(--card)', border: '1px solid var(--border)',
        borderRadius: 8, padding: 8, minWidth: 200,
        boxShadow: '0 4px 16px rgba(0,0,0,.2)',
      }}>
        <div style={{ fontSize: 11, color: 'var(--muted)', marginBottom: 6, fontWeight: 600 }}>
          Choose visualization for {sensor?.name ?? `Tag #${sensorId}`}
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 4 }}>
          {types.map(type => {
            const m = WIDGET_MANIFESTS[type]
            return (
              <button
                key={type}
                onClick={() => onSelect(type)}
                style={{
                  display: 'flex', alignItems: 'center', gap: 6,
                  padding: '6px 8px', borderRadius: 6,
                  border: '1px solid var(--border)', background: 'var(--bg)',
                  color: 'var(--fg)', cursor: 'pointer', fontSize: 12,
                  transition: 'border-color 0.15s',
                  textAlign: 'left',
                }}
                onMouseEnter={e => e.currentTarget.style.borderColor = 'var(--cyan)'}
                onMouseLeave={e => e.currentTarget.style.borderColor = 'var(--border)'}
              >
                <span style={{ fontSize: 16 }}>{m.icon}</span>
                <span>{m.displayName}</span>
              </button>
            )
          })}
        </div>
      </div>
    </>
  )
}
