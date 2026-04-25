import { SensorConfig, SensorData } from './types'
import { Chart } from './Chart'
import { status, stColor } from './styles'
import { parseConfig } from './styles'

interface Props {
  sensor: SensorConfig
  editMode: boolean
  activeScreen: string
  editingId: number | null
  currentValues: Record<string, SensorData>
  shortcuts: Record<number, string>
  chartDataFor: (id: number) => number[]
  loadSensorsForScreen: () => void
  setEditingId: (id: number | null) => void
  setSelectedSensor: (id: number | null) => void
  showDragHandle?: boolean
  selected?: boolean
  onClick?: (e: React.MouseEvent) => void
}

export function SensorCard({
  sensor: s, editMode, activeScreen, editingId, currentValues, shortcuts,
  chartDataFor, loadSensorsForScreen, setEditingId, setSelectedSensor,
  showDragHandle, selected, onClick,
}: Props) {
  const cv = currentValues[s.id];
  const isOffline = cv && (cv.value == null || Number.isNaN(cv.value));
  const st = isOffline ? 'offline' : cv ? status(cv.value, cv.warn, cv.crit) : 'normal';
  const ct = s.chart_type || 'sparkline';
  const hideStatus = ct === 'image' || ct === 'text';

  return (
    <div
      style={{
        background: 'var(--card)',
        border: `1px solid ${selected ? 'var(--cyan)' : editingId === s.id ? 'var(--cyan)' : 'var(--border)'}`,
        borderRadius: 10,
        overflow: 'hidden',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        ...(selected ? { boxShadow: '0 0 0 2px var(--cyan)' } : {}),
      }}
      onClick={onClick}
    >
      {!hideStatus && <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '14px 16px 10px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.04em' }}>
          {showDragHandle && editMode && (
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
            <button onClick={e => { e.stopPropagation(); fetch(`/api/screens/${activeScreen}/data-sources/${s.id}`, { method: 'DELETE' }).then(loadSensorsForScreen); }} style={{ background: 'var(--bg)', border: '1px solid var(--crit)', borderRadius: 4, color: 'var(--crit)', cursor: 'pointer', padding: '2px 6px', fontSize: 10, lineHeight: 1 }}>Remove</button>
          )}
          <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.05em', padding: '2px 8px', borderRadius: 4,
            background: st === 'critical' ? 'rgba(239,68,68,.15)' : st === 'warning' ? 'rgba(234,179,8,.15)' : st === 'offline' ? 'rgba(100,100,100,.15)' : 'rgba(34,197,94,.15)', color: stColor(st) }}>{st}</div>
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
          <button onClick={e => { e.stopPropagation(); fetch(`/api/screens/${activeScreen}/data-sources/${s.id}`, { method: 'DELETE' }).then(loadSensorsForScreen); }} style={{ background: 'var(--bg)', border: '1px solid var(--crit)', borderRadius: 4, color: 'var(--crit)', cursor: 'pointer', padding: '2px 6px', fontSize: 10, lineHeight: 1 }}>Remove</button>
        </div>
      )}
      {!hideStatus && (
        <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 22, fontWeight: 700, padding: '0 16px' }}>
          <span style={{ color: st === 'normal' ? s.color : stColor(st) }}>{isOffline ? 'OFFLINE' : cv ? cv.value.toFixed(s.decimals) : s.base_val.toFixed(s.decimals)}</span>
          <span style={{ fontSize: 13, color: 'var(--muted)', fontWeight: 400, marginLeft: 4 }}>{s.unit}</span>
        </div>
      )}
      <div style={{ flex: 1, minHeight: 0 }}>
        <Chart
          chartType={s.chart_type || 'sparkline'}
          data={chartDataFor(s.id)}
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
  )
}
