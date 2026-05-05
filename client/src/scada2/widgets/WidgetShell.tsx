import { useCallback } from 'react'
import { Rnd } from 'react-rnd'
import type { SensorConfig, SensorData } from '../../types'
import type { WidgetInstance } from './types'
import { getWidgetComponent } from './registry'

interface WidgetShellProps {
  instance: WidgetInstance
  sensor: SensorConfig | undefined
  data: SensorData | undefined
  editMode: boolean
  selected: boolean
  onSelect: (id: string) => void
  onMove: (id: string, x: number, y: number) => void
  onResize: (id: string, x: number, y: number, width: number, height: number) => void
}

export default function WidgetShell({
  instance,
  sensor,
  data,
  editMode,
  selected,
  onSelect,
  onMove,
  onResize,
}: WidgetShellProps) {
  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation()
      onSelect(instance.id)
    },
    [onSelect, instance.id],
  )

  const handleDragStop = useCallback(
    (_e: any, d: { x: number; y: number }) => {
      onMove(instance.id, Math.round(d.x), Math.round(d.y))
    },
    [onMove, instance.id],
  )

  const handleResizeStop = useCallback(
    (
      _e: MouseEvent | TouchEvent,
      _dir: string,
      _ref: HTMLElement,
      _delta: { width: number; height: number },
      position: { x: number; y: number },
    ) => {
      onResize(
        instance.id,
        Math.round(position.x),
        Math.round(position.y),
        Math.round(_ref.offsetWidth),
        Math.round(_ref.offsetHeight),
      )
    },
    [onResize, instance.id],
  )

  const WidgetComponent = getWidgetComponent(instance.type)
  if (!WidgetComponent) return null

  return (
    <Rnd
      position={{ x: instance.x, y: instance.y }}
      size={{ width: instance.width, height: instance.height }}
      disableDragging={!editMode}
      enableResizing={editMode}
      onDragStop={handleDragStop}
      onResizeStop={handleResizeStop}
      minWidth={60}
      minHeight={40}
      bounds="parent"
      dragGrid={[10, 10]}
      resizeGrid={[10, 10]}
      style={{ display: 'flex' }}
    >
      <div
        onClick={handleClick}
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          border: `1px solid ${editMode ? (selected ? 'var(--cyan)' : 'transparent') : 'transparent'}`,
          borderRadius: 8,
          overflow: 'hidden',
          cursor: editMode ? 'move' : 'default',
          boxShadow: selected ? '0 0 0 2px var(--cyan)' : 'none',
          background: 'transparent',
        }}
      >
        <div style={{ flex: 1, minHeight: 0, overflow: 'hidden' }}>
          <WidgetComponent
            instance={instance}
            sensor={sensor}
            data={data}
            editMode={editMode}
            selected={selected}
          />
        </div>
        {editMode && instance.sensorId != null && (
          <div
            style={{
              padding: '2px 6px',
              fontSize: 10,
              color: 'var(--muted)',
              textAlign: 'center',
              whiteSpace: 'nowrap',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              lineHeight: '14px',
            }}
          >
            {sensor?.name ?? `Sensor #${instance.sensorId}`}
          </div>
        )}
      </div>
    </Rnd>
  )
}
