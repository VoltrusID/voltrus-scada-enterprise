import { useId } from 'react'
import type React from 'react'
import type { SensorConfig, SensorData } from '../../types'
import type { WidgetProps } from '../widgets/types'
import type { EquipState, SymbolDef } from './symbols'
import { FOOTER, footerRows } from './resolve'

function tagState(data: SensorData | undefined, threshold: number): EquipState {
  if (!data || data.value == null) return 'stale'
  return data.value > threshold ? 'running' : 'stopped'
}

function formatValue(data: SensorData | undefined, sensor: SensorConfig | undefined): string {
  if (!data || data.value == null) return ''
  return data.value.toFixed(sensor?.decimals ?? 1)
}

export interface SymbolViewProps extends WidgetProps {
  def: SymbolDef
  /** Legacy pump/valve/motor widgets toggled on click; new symbols must opt in. */
  defaultWritable?: boolean
}

/** Renders an HMI symbol bound to live tags: state, level, label and an alarm-aware value badge. */
export function SymbolView({ def, defaultWritable = false, instance, sensor, data, currentValues, editMode, onWriteValue }: SymbolViewProps) {
  const uid = 's' + useId().replace(/[^a-zA-Z0-9]/g, '')
  const config = instance.config ?? {}
  const bound = instance.sensorId != null
  const threshold = Number(config.onThreshold ?? (def.binding === 'none' ? 0 : 0.5))

  // Fault tag overrides everything; then the primary binding decides.
  const faultData = config.faultSensorId != null ? currentValues[String(config.faultSensorId)] : undefined
  let state: EquipState = 'idle'
  if (faultData && faultData.value > 0.5) state = 'fault'
  else if (bound && def.binding === 'state') state = tagState(data, threshold)
  else if (bound && !data) state = 'stale'

  let level: number | null = null
  if (def.binding === 'level' && bound && data?.value != null) {
    const min = sensor?.min_val ?? 0
    const max = sensor?.max_val ?? 100
    level = Math.max(0, Math.min(1, (data.value - min) / ((max - min) || 1)))
  }

  // Secondary run tag (agitator, pipe flow); pipes fall back to their primary tag.
  let runState: EquipState = 'idle'
  if (config.runSensorId != null) runState = tagState(currentValues[String(config.runSensorId)], 0.5)
  else if (def.binding === 'none' && bound) runState = tagState(data, threshold)

  const value = formatValue(data, sensor)
  const unit = sensor?.unit ?? ''

  const label: string = config.label ?? ''
  const rows = footerRows(def, config, bound)
  const alarm = bound && data ? (data.status === 'crit' ? 'crit' : data.status === 'warn' ? 'warn' : null) : null

  const rotation = Number(config.rotation ?? 0)
  const flip = !!config.flipX
  const transform = [rotation ? `rotate(${rotation}deg)` : '', flip ? 'scaleX(-1)' : ''].filter(Boolean).join(' ')

  const writable = !editMode && !!onWriteValue && def.binding === 'state' && (config.writable ?? defaultWritable) && bound && !!data
  // Start/stop follows the run tag alone: a faulted pump that is still running must offer Stop.
  const running = bound && tagState(data, threshold) === 'running'
  const handleClick = (e: React.MouseEvent) => {
    if (!writable) return
    e.stopPropagation()
    const what = label || sensor?.name || def.name
    if (!window.confirm(`${running ? 'Stop' : 'Start'} ${what}?`)) return
    onWriteValue!(running ? 0 : 1)
  }

  const [vw, vh] = def.viewBox

  return (
    <div
      onClick={handleClick}
      style={{
        width: '100%', height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center',
        cursor: writable ? 'pointer' : undefined, userSelect: 'none',
      }}
      title={writable ? `Click to ${running ? 'stop' : 'start'}` : undefined}
    >
      <div style={{ flex: 1, minHeight: 0, width: '100%', position: 'relative' }}>
        <svg
          viewBox={`0 0 ${vw} ${vh}`}
          preserveAspectRatio={def.stretch ? 'none' : 'xMidYMid meet'}
          style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', overflow: 'visible', transform: transform || undefined }}
        >
          {def.render({ uid, state, level, value, unit, runState, config })}
        </svg>
      </div>
      {(rows.label || rows.value || rows.status) && (
        // Fixed row heights: connection ports are computed from them (resolve.ts footerHeight).
        <div style={{
          display: 'flex', flexDirection: 'column', alignItems: 'center', gap: FOOTER.gap, paddingTop: FOOTER.pad,
          flexShrink: 0, maxWidth: '100%',
        }}>
          {rows.label && (
            <div style={{
              height: FOOTER.label, lineHeight: `${FOOTER.label}px`, fontSize: 11, fontWeight: 600, color: 'var(--text-dim)',
              letterSpacing: '.02em', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '100%',
            }}>{label}</div>
          )}
          {rows.value && <ValueBadge value={value} unit={unit} alarm={alarm} stale={!data} />}
          {rows.status && <StatusChip state={state} />}
        </div>
      )}
    </div>
  )
}

export function ValueBadge({ value, unit, alarm, stale }: { value: string; unit: string; alarm: 'crit' | 'warn' | null; stale: boolean }) {
  const border = alarm === 'crit' ? 'var(--hmi-fault)' : alarm === 'warn' ? 'var(--hmi-warn)' : 'rgba(255,255,255,.08)'
  return (
    <div
      className={alarm === 'crit' ? 'hmi-blink' : undefined}
      style={{
        display: 'inline-flex', alignItems: 'baseline', gap: 4, padding: '0 6px', borderRadius: 3, height: FOOTER.badge, boxSizing: 'border-box',
        background: 'var(--hmi-badge)', border: `1px solid ${border}`,
        fontFamily: 'var(--font-mono)', fontSize: 12, lineHeight: '17px', whiteSpace: 'nowrap',
      }}
    >
      {alarm && <span style={{ width: 6, height: 6, alignSelf: 'center', background: border, transform: 'rotate(45deg)' }} />}
      <span style={{ color: stale ? 'var(--hmi-stale)' : '#e8edf2', fontWeight: 600 }}>{stale ? '????' : value}</span>
      {unit && <span style={{ color: '#8b95a1', fontSize: 10 }}>{unit}</span>}
    </div>
  )
}

const STATUS_TEXT: Record<EquipState, string> = {
  idle: '—', running: 'RUNNING', stopped: 'STOPPED', fault: 'FAULT', stale: 'NO DATA',
}

function StatusChip({ state }: { state: EquipState }) {
  const color = state === 'running' ? 'var(--hmi-run)' : state === 'fault' ? 'var(--hmi-fault)' : state === 'stale' ? 'var(--hmi-stale)' : 'var(--text-muted)'
  return (
    <div style={{ height: FOOTER.status, lineHeight: `${FOOTER.status}px`, fontSize: 9, fontWeight: 700, letterSpacing: '.08em', color, whiteSpace: 'nowrap' }}>
      {STATUS_TEXT[state]}
    </div>
  )
}

/** Static preview used in the library panel. */
export function SymbolThumb({ def, size = 44 }: { def: SymbolDef; size?: number }) {
  const uid = 't' + useId().replace(/[^a-zA-Z0-9]/g, '')
  const [vw, vh] = def.viewBox
  const demoLevel = def.binding === 'level' ? 0.62 : null
  return (
    <svg viewBox={`0 0 ${vw} ${vh}`} width={size} height={size} preserveAspectRatio="xMidYMid meet" style={{ overflow: 'visible' }}>
      {def.render({ uid, state: 'idle', level: demoLevel, value: def.key === 'meter' ? '230.4' : '', unit: '', runState: 'idle', config: { showPercent: false } })}
    </svg>
  )
}
