import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const AnnunciatorWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensors, currentValues } = props
  const config = instance.config ?? {}
  const maxItems = config.maxItems ?? 12

  const boundSensors = instance.sensorIds.length > 0
    ? sensors.filter(s => instance.sensorIds.includes(s.id))
    : instance.sensorId
      ? sensors.filter(s => s.id === instance.sensorId)
      : sensors

  const displaySensors = boundSensors.length > 0 ? boundSensors.slice(0, maxItems) : sensors.slice(0, maxItems)

  if (displaySensors.length === 0) {
    return (
      <div style={{
        width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center',
        color: 'var(--muted)', fontSize: 11,
      }}>
        Drop tags here
      </div>
    )
  }

  const items = displaySensors.map(s => {
    const d = currentValues[s.key]
    const isAlarm = d?.status === 'crit' || d?.status === 'warn'
    return {
      sensor: s,
      value: d?.value,
      status: d?.status ?? '',
      isAlarm,
      severity: d?.status === 'crit' ? 'crit' as const : d?.status === 'warn' ? 'warn' as const : null,
    }
  })

  const severityColor = (s: 'crit' | 'warn' | null) =>
    s === 'crit' ? 'var(--crit)' : s === 'warn' ? 'var(--warn)' : 'var(--border)'

  const severityBg = (s: 'crit' | 'warn' | null) =>
    s === 'crit' ? 'rgba(239,68,68,0.08)' : s === 'warn' ? 'rgba(234,179,8,0.06)' : 'transparent'

  return (
    <div style={{
      width: '100%', height: '100%', overflow: 'auto',
      display: 'grid',
      gridTemplateColumns: `repeat(auto-fill, minmax(120px, 1fr))`,
      gap: 3,
      padding: 3,
      alignContent: 'start',
    }}>
      {items.map((item, i) => {
        const color = severityColor(item.severity)
        const bg = severityBg(item.severity)
        return (
          <div key={i} style={{
            display: 'flex', alignItems: 'center', gap: 5,
            padding: '4px 6px', borderRadius: 4,
            border: `1px solid ${color}`,
            backgroundColor: bg,
            opacity: item.isAlarm ? 1 : 0.5,
            transition: 'opacity 0.3s ease, border-color 0.3s ease',
          }}>
            <div style={{
              width: 8, height: 8, borderRadius: '50%',
              backgroundColor: item.isAlarm ? color : 'var(--border)',
              boxShadow: item.isAlarm ? `0 0 6px ${color}` : 'none',
              flexShrink: 0,
              transition: 'background-color 0.3s ease',
            }} />
            <div style={{ minWidth: 0, flex: 1 }}>
              <div style={{
                fontSize: 8, color: 'var(--fg)', fontWeight: 600,
                overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
              }}>
                {item.sensor.name}
              </div>
              <div style={{ fontSize: 7, color: 'var(--muted)' }}>
                {item.value !== undefined ? `${item.value.toFixed(1)} ${item.sensor.unit ?? ''}` : '--'}
              </div>
            </div>
          </div>
        )
      })}
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.annunciator,
  component: AnnunciatorWidget,
})

export default AnnunciatorWidget
