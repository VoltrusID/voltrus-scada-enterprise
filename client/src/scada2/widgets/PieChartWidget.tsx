import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const PieChartWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensors, currentValues } = props
  const config = instance.config ?? {}
  const maxSlices = config.maxSlices ?? 8

  const boundSensors = instance.sensorIds.length > 0
    ? sensors.filter(s => instance.sensorIds.includes(s.id))
    : instance.sensorId
      ? sensors.filter(s => s.id === instance.sensorId)
      : sensors

  const displaySensors = boundSensors.length > 0 ? boundSensors.slice(0, maxSlices) : sensors.slice(0, maxSlices)

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
    const v = Math.max(0, d?.value ?? 0)
    return { sensor: s, value: v }
  })

  const total = items.reduce((sum, item) => sum + item.value, 0) || 1

  const palette = [
    'var(--cyan)', 'var(--ok)', 'var(--warn)', 'var(--crit)',
    '#8b5cf6', '#f97316', '#ec4899', '#6366f1',
  ]

  const cx = 80
  const cy = 80
  const r = 60

  let accumulated = 0
  const slices = items.map((item, i) => {
    const pct = item.value / total
    const startAngle = accumulated * 360
    const endAngle = (accumulated + pct) * 360
    accumulated += pct

    const startRad = ((startAngle - 90) * Math.PI) / 180
    const endRad = ((endAngle - 90) * Math.PI) / 180
    const largeArc = pct > 0.5 ? 1 : 0

    const x1 = cx + r * Math.cos(startRad)
    const y1 = cy + r * Math.sin(startRad)
    const x2 = cx + r * Math.cos(endRad)
    const y2 = cy + r * Math.sin(endRad)

    const path = pct >= 1
      ? `M ${cx} ${cy - r} A ${r} ${r} 0 1 1 ${cx - 0.01} ${cy - r} Z`
      : `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2} Z`

    const labelAngle = ((startAngle + endAngle) / 2 - 90) * Math.PI / 180
    const labelR = r * 0.65
    const labelX = cx + labelR * Math.cos(labelAngle)
    const labelY = cy + labelR * Math.sin(labelAngle)

    return { path, color: palette[i % palette.length], pct, label: item.sensor.name, labelX, labelY }
  })

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <svg viewBox="0 0 200 170" style={{ width: '100%', height: '100%' }} preserveAspectRatio="xMidYMid meet">
        {slices.map((s, i) => (
          <path key={i} d={s.path} fill={s.color} stroke="var(--bg)" strokeWidth={1.5}
            style={{ transition: 'opacity 0.3s ease' }} />
        ))}

        {slices.map((s, i) => {
          const name = s.label.length > 5 ? s.label.slice(0, 4) + '..' : s.label
          return s.pct > 0.08 ? (
            <g key={`l-${i}`}>
              <text x={s.labelX} y={s.labelY - 2} textAnchor="middle" fontSize="7" fontWeight="600" fill="var(--bg)">
                {name}
              </text>
              <text x={s.labelX} y={s.labelY + 7} textAnchor="middle" fontSize="6" fill="var(--bg)" opacity={0.8}>
                {(s.pct * 100).toFixed(0)}%
              </text>
            </g>
          ) : null
        })}

        <g transform="translate(0, 160)">
          {slices.map((s, i) => {
            const cols = Math.min(slices.length, 4)
            const row = Math.floor(i / cols)
            const col = i % cols
            const colW = 200 / cols
            return (
              <g key={`lg-${i}`} transform={`translate(${col * colW + 4}, ${row * 12})`}>
                <rect x={0} y={0} width={6} height={6} rx={1} fill={s.color} />
                <text x={9} y={6} fontSize="5.5" fill="var(--muted)">
                  {s.label}
                </text>
              </g>
            )
          })}
        </g>
      </svg>
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.pie_chart,
  component: PieChartWidget,
})

export default PieChartWidget
