import { useRef, useCallback } from 'react'
import { ResponsiveGridLayout, useContainerWidth, verticalCompactor } from 'react-grid-layout'
import { SensorConfig, SensorData, ScreenLayouts } from './types'
import { EditPanel } from './EditPanel'
import { SensorCard } from './SensorCard'

const BREAKPOINTS = { lg: 1200, md: 768, sm: 0 }
const COLS = { lg: 24, md: 12, sm: 4 }
const BP_WIDTHS: Record<string, number> = { lg: 1200, md: 996, sm: 480 }

interface Props {
  screenLayouts: ScreenLayouts
  screenSensors: SensorConfig[]
  editMode: boolean
  activeScreen: string
  gap: number
  editingId: number | null
  currentValues: Record<string, SensorData>
  setEditingId: (id: number | null) => void
  setSelectedSensor: (id: number | null) => void
  chartDataFor: (id: number) => number[]
  loadSensorsForScreen: () => void
  onLayoutChange: (currentLayout: any[], allLayouts: { [bp: string]: any[] }) => void
  onSave: (s: SensorConfig, shortcut?: string | null) => void
  onDelete: (id: number) => void
  forcedBreakpoint: string | null
  shortcuts: Record<number, string>
}

export function SensorGrid({ screenLayouts, screenSensors, editMode, activeScreen, gap, editingId, currentValues, setEditingId, setSelectedSensor, chartDataFor, loadSensorsForScreen, onLayoutChange, onSave, onDelete, forcedBreakpoint, shortcuts }: Props) {
  const { width: measuredWidth, containerRef, mounted } = useContainerWidth({ initialWidth: 1200 })
  const didDrag = useRef(false)

  const width = forcedBreakpoint ? BP_WIDTHS[forcedBreakpoint] : measuredWidth

  const handleInteractionEnd = useCallback((layout: any) => {
    didDrag.current = true
    const activeBp = forcedBreakpoint || 'lg'
    const normalize = (items: readonly any[]) => items.map((l: any) => ({ i: String(l.i), x: l.x, y: l.y, w: l.w, h: l.h }))
    const allLayouts: Record<string, any[]> = {
      lg: activeBp === 'lg' ? normalize(layout) : screenLayouts.lg,
      md: activeBp === 'md' ? normalize(layout) : screenLayouts.md,
      sm: activeBp === 'sm' ? normalize(layout) : screenLayouts.sm,
    }
    onLayoutChange(layout, allLayouts)
  }, [onLayoutChange, forcedBreakpoint, screenLayouts])

  const ready = screenSensors.length > 0 && (screenLayouts.lg.length > 0 || screenLayouts.md.length > 0 || screenLayouts.sm.length > 0)

  return (
    <div ref={containerRef} style={{ maxWidth: '100%' }}>
      {mounted && ready && (
        <ResponsiveGridLayout
          key={screenSensors.map(s => s.id).join(',')}
          width={width}
          layouts={screenLayouts as any}
          breakpoints={BREAKPOINTS}
          breakpoint={forcedBreakpoint || undefined}
          cols={COLS}
          rowHeight={50}
          margin={[gap, gap] as [number, number]}
          compactor={verticalCompactor}
          onDragStop={editMode ? (handleInteractionEnd as any) : undefined}
          onResizeStop={editMode ? (handleInteractionEnd as any) : undefined}
          dragConfig={editMode ? { handle: '.drag-handle' } : { enabled: false }}
          resizeConfig={editMode ? { enabled: true } : { enabled: false }}
        >
          {screenSensors.map((s) => (
            <div key={String(s.id)} style={{ position: 'relative' }}>
              <SensorCard
                sensor={s}
                editMode={editMode}
                activeScreen={activeScreen}
                editingId={editingId}
                currentValue={currentValues[String(s.id)]}
                shortcut={shortcuts[s.id]}
                chartDataFor={chartDataFor}
                loadSensorsForScreen={loadSensorsForScreen}
                setEditingId={setEditingId}
                setSelectedSensor={setSelectedSensor}
                showDragHandle={true}
                onClick={e => {
                  if (didDrag.current) { didDrag.current = false; return; }
                  if (editMode) { e.stopPropagation(); setEditingId(editingId === s.id ? null : s.id); }
                  else { setSelectedSensor(s.id); }
                }}
              />
              {editingId === s.id && <EditPanel sensor={s} activeScreen={activeScreen} onSave={onSave} onDelete={onDelete} onClose={() => setEditingId(null)} shortcuts={shortcuts} />}
            </div>
          ))}
        </ResponsiveGridLayout>
      )}
    </div>
  );
}
