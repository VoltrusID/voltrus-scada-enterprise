import { useRef, useCallback } from 'react'
import { ResponsiveGridLayout, useContainerWidth, verticalCompactor } from 'react-grid-layout'
import { SensorConfig, SensorData, ScreenLayouts } from './types'
import { Chart } from './Chart'
import { parseConfig } from './styles'
import { EditPanel } from './EditPanel'
import { status, stColor } from './styles'

const BREAKPOINTS = { lg: 1200, md: 768, sm: 0 }
const COLS = { lg: 12, md: 6, sm: 2 }
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
  setSelectedSensor: (key: string | null) => void
  chartDataFor: (key: string) => number[]
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

  // In edit mode with forced breakpoint, simulate that breakpoint's width
  const width = forcedBreakpoint ? BP_WIDTHS[forcedBreakpoint] : measuredWidth

  const handleInteractionEnd = useCallback((layout: any) => {
    didDrag.current = true
    // Build allLayouts from current screenLayouts, replacing the active breakpoint
    const activeBp = forcedBreakpoint || 'lg'
    const normalize = (items: readonly any[]) => items.map((l: any) => ({ i: String(l.i), x: l.x, y: l.y, w: l.w, h: l.h }))
    const allLayouts: Record<string, any[]> = {
      lg: activeBp === 'lg' ? normalize(layout) : screenLayouts.lg,
      md: activeBp === 'md' ? normalize(layout) : screenLayouts.md,
      sm: activeBp === 'sm' ? normalize(layout) : screenLayouts.sm,
    }
    onLayoutChange(layout, allLayouts)
  }, [onLayoutChange, forcedBreakpoint, screenLayouts])

  return (
    <div ref={containerRef} style={{ maxWidth: '100%' }}>
      {mounted && (
        <ResponsiveGridLayout
          width={width}
          layouts={screenLayouts as any}
          breakpoints={BREAKPOINTS}
          breakpoint={forcedBreakpoint || undefined}
          cols={COLS}
          rowHeight={100}
          margin={[gap, gap] as [number, number]}
          compactor={verticalCompactor}
          onDragStop={editMode ? (handleInteractionEnd as any) : undefined}
          onResizeStop={editMode ? (handleInteractionEnd as any) : undefined}
          dragConfig={editMode ? { handle: '.drag-handle' } : { enabled: false }}
          resizeConfig={editMode ? { enabled: true } : { enabled: false }}
        >
          {screenSensors.map((s) => {
            const cv = currentValues[s.key];
            const st = cv ? status(cv.value, cv.warn, cv.crit) : 'normal';
            const ct = s.chart_type || 'sparkline';
            const hideStatus = ct === 'image' || ct === 'text';
            return (
              <div key={String(s.id)} style={{ position: 'relative' }}>
                <div style={{ background: 'var(--card)', border: `1px solid ${editingId === s.id ? 'var(--cyan)' : 'var(--border)'}`, borderRadius: 10, overflow: 'hidden', height: '100%', display: 'flex', flexDirection: 'column' }} onClick={e => {
                  if (didDrag.current) { didDrag.current = false; return; }
                  if (editMode) { e.stopPropagation(); setEditingId(editingId === s.id ? null : s.id); }
                  else { setSelectedSensor(s.key); }
                }}>
                  {!hideStatus && <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '14px 16px 10px' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.04em' }}>
                      {editMode && (
                        <span className="drag-handle" style={{ cursor: 'grab', display: 'inline-flex', alignItems: 'center', padding: '0 2px', opacity: 0.4 }}>
                          <svg width="8" height="12" viewBox="0 0 8 12">
                            <circle cx="2" cy="2" r="1.2" fill="var(--fg)"/><circle cx="6" cy="2" r="1.2" fill="var(--fg)"/>
                            <circle cx="2" cy="6" r="1.2" fill="var(--fg)"/><circle cx="6" cy="6" r="1.2" fill="var(--fg)"/>
                            <circle cx="2" cy="10" r="1.2" fill="var(--fg)"/><circle cx="6" cy="10" r="1.2" fill="var(--fg)"/>
                          </svg>
                        </span>
                      )}
                      <div style={{ width: 8, height: 8, borderRadius: '50%', background: stColor(st) }} />
                      {s.name}
                      <span style={{ fontSize: 10, color: 'var(--muted)', fontWeight: 400, textTransform: 'none', letterSpacing: 0 }}>({s.source_type})</span>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      {shortcuts[s.id] && (
                        <kbd style={{
                          fontFamily: "'JetBrains Mono',monospace", fontSize: 10, fontWeight: 600,
                          padding: '2px 6px', borderRadius: 3,
                          background: 'var(--bg)', border: '1px solid var(--border)',
                          color: 'var(--muted)', lineHeight: 1,
                        }}>{shortcuts[s.id]}</kbd>
                      )}
                      {editMode && activeScreen !== 'all' && (
                        <button onClick={e => { e.stopPropagation(); fetch(`/api/screens/${activeScreen}/sensors/${s.id}`, { method: 'DELETE' }).then(loadSensorsForScreen); }} style={{ background: 'var(--bg)', border: '1px solid var(--crit)', borderRadius: 4, color: 'var(--crit)', cursor: 'pointer', padding: '2px 6px', fontSize: 10, lineHeight: 1 }}>Remove</button>
                      )}
                      <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.05em', padding: '2px 8px', borderRadius: 4,
                        background: st === 'critical' ? 'rgba(239,68,68,.15)' : st === 'warning' ? 'rgba(234,179,8,.15)' : 'rgba(34,197,94,.15)', color: stColor(st) }}>{st}</div>
                    </div>
                  </div>}
                  {hideStatus && editMode && (
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '4px 8px', opacity: 0.5 }}>
                    <span className="drag-handle" style={{ cursor: 'grab', display: 'inline-flex', alignItems: 'center', gap: 4 }}>
                      <svg width="8" height="12" viewBox="0 0 8 12">
                        <circle cx="2" cy="2" r="1.2" fill="var(--fg)"/><circle cx="6" cy="2" r="1.2" fill="var(--fg)"/>
                        <circle cx="2" cy="6" r="1.2" fill="var(--fg)"/><circle cx="6" cy="6" r="1.2" fill="var(--fg)"/>
                        <circle cx="2" cy="10" r="1.2" fill="var(--fg)"/><circle cx="6" cy="10" r="1.2" fill="var(--fg)"/>
                      </svg>
                      <span style={{ fontSize: 10, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.04em' }}>{s.name}</span>
                    </span>
                    <button onClick={e => { e.stopPropagation(); fetch(`/api/screens/${activeScreen}/sensors/${s.id}`, { method: 'DELETE' }).then(loadSensorsForScreen); }} style={{ background: 'var(--bg)', border: '1px solid var(--crit)', borderRadius: 4, color: 'var(--crit)', cursor: 'pointer', padding: '2px 6px', fontSize: 10, lineHeight: 1 }}>Remove</button>
                  </div>
                  )}
                  {!hideStatus && (
                  <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 22, fontWeight: 700, padding: '0 16px' }}>
                    <span style={{ color: st === 'normal' ? s.color : stColor(st) }}>{cv ? cv.value.toFixed(s.decimals) : s.base_val.toFixed(s.decimals)}</span>
                    <span style={{ fontSize: 13, color: 'var(--muted)', fontWeight: 400, marginLeft: 4 }}>{s.unit}</span>
                  </div>
                  )}
                  <div style={{ flex: 1, minHeight: 0 }}>
                    <Chart
                      chartType={s.chart_type || 'sparkline'}
                      data={chartDataFor(s.key)}
                      color={s.color}
                      warn={s.warn_val}
                      crit={s.crit_val}
                      min={s.min_val}
                      max={s.max_val}
                      decimals={s.decimals}
                      unit={s.unit}
                      imageUrl={(parseConfig(s.source_config) as Record<string, string>).image_url}
                      textContent={(parseConfig(s.source_config) as Record<string, string>).text_content}
                    />
                  </div>
                </div>
                {editingId === s.id && <EditPanel sensor={s} activeScreen={activeScreen} onSave={onSave} onDelete={onDelete} onClose={() => setEditingId(null)} shortcuts={shortcuts} />}
              </div>
            );
          })}
        </ResponsiveGridLayout>
      )}
    </div>
  );
}
