import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const BarChartWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensors, currentValues } = props
  const config = instance.config ?? {}
  const maxBars = config.maxBars ?? 8
  const showValues = config.showValues !== false

  const boundSensors = instance.sensorIds.length > 0
    ? sensors.filter(s => instance.sensorIds.includes(s.id))
    : instance.sensorId
      ? sensors.filter(s => s.id === instance.sensorId)
      : sensors.slice(0, maxBars)

  const displaySensors = boundSensors.length > 0 ? boundSensors.slice(0, maxBars) : sensors.slice(0, maxBars)

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

  let globalMin = Infinity
  let globalMax = -Infinity
  const items = displaySensors.map(s => {
    const d = currentValues[s.key]
    const v = d?.value ?? 0
    const sMin = s.min_val ?? 0
    const sMax = s.max_val ?? 100
    globalMin = Math.min(globalMin, sMin)
    globalMax = Math.max(globalMax, sMax)
    return { sensor: s, value: v, status: d?.status ?? '' }
  })
  const range = globalMax - globalMin || 1

  const padLeft = 10
  const padRight = 10
  const padTop = 8
  const padBottom = 24
  const chartW = 200 - padLeft - padRight
  const chartH = 120 - padTop - padBottom
  const barGap = 4
  const barW = Math.max(8, (chartW - barGap * (items.length + 1)) / items.length)

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <svg viewBox="0 0 200 120" style={{ width: '100%', height: '100%' }} preserveAspectRatio="xMidYMid meet">
        {[0, 0.25, 0.5, 0.75, 1].map((p, i) => {
          const y = padTop + chartH * (1 - p)
          const label = (globalMin + range * p).toFixed(0)
          return (
            <g key={i}>
              <line x1={padLeft} y1={y} x2={padLeft + chartW} y2={y}
                stroke="var(--border)" strokeWidth={0.5} strokeDasharray="2 2" />
              <text x={padLeft - 3} y={y + 3} textAnchor="end" fontSize="6" fill="var(--muted)">
                {label}
              </text>
            </g>
          )
        })}

        {items.map((item, i) => {
          const pct = Math.max(0, Math.min(1, (item.value - globalMin) / range))
          const barH = chartH * pct
          const x = padLeft + barGap + i * (barW + barGap)
          const y = padTop + chartH - barH

          const statusColor = item.status === 'crit' ? 'var(--crit)'
            : item.status === 'warn' ? 'var(--warn)' : 'var(--ok)'

          const barColor = item.status ? statusColor : 'var(--cyan)'

          const label = item.sensor.name.length > 6
            ? item.sensor.name.slice(0, 5) + '...'
            : item.sensor.name

          return (
            <g key={i}>
              <rect x={x} y={padTop} width={barW} height={chartH}
                fill="var(--border)" opacity={0.15} rx={2} />
              <rect x={x} y={y} width={barW} height={barH}
                fill={barColor} rx={2}
                style={{ transition: 'height 0.4s ease, y 0.4s ease' }} />
              {showValues && (
                <text x={x + barW / 2} y={y - 3} textAnchor="middle" fontSize="6" fill="var(--fg)">
                  {item.value.toFixed(1)}
                </text>
              )}
              <text x={x + barW / 2} y={padTop + chartH + 12} textAnchor="middle" fontSize="6" fill="var(--muted)">
                {label}
              </text>
            </g>
          )
        })}
      </svg>
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.bar_chart,
  component: BarChartWidget,
})

export default BarChartWidget
