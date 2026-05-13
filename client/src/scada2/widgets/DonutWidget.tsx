import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const DonutWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, data } = props
  const config = instance.config ?? {}
  const fillColor = config.fillColor ?? 'var(--cyan)'
  const showPercentage = config.showPercentage !== false
  const showName = config.showName !== false
  const thickness = config.thickness ?? 14

  const min = sensor?.min_val ?? 0
  const max = sensor?.max_val ?? 100
  const warn = sensor?.warn_val ?? (min + max) * 0.75
  const crit = sensor?.crit_val ?? (min + max) * 0.9
  const decimals = sensor?.decimals ?? 1
  const unit = sensor?.unit ?? ''

  const value = data?.value ?? min
  const range = max - min || 1
  const pct = Math.max(0, Math.min(1, (value - min) / range))

  const warnPct = Math.max(0, Math.min(1, (warn - min) / range))
  const critPct = Math.max(0, Math.min(1, (crit - min) / range))

  const activeColor = pct >= critPct
    ? 'var(--crit)'
    : pct >= warnPct
      ? 'var(--warn)'
      : fillColor

  const cx = 100
  const cy = 100
  const r = 80

  const circumference = 2 * Math.PI * r
  const dashOffset = circumference * (1 - pct)

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
      <svg viewBox="0 0 200 200" style={{ width: '100%', height: '90%' }} preserveAspectRatio="xMidYMid meet">
        <circle cx={cx} cy={cy} r={r}
          fill="none" stroke="var(--border)" strokeWidth={thickness} />

        <circle cx={cx} cy={cy} r={r}
          fill="none" stroke={activeColor} strokeWidth={thickness}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={dashOffset}
          transform={`rotate(-90 ${cx} ${cy})`}
          style={{ transition: 'stroke-dashoffset 0.6s ease, stroke 0.3s ease' }}
        />

        <text x={cx} y={cy - 8} textAnchor="middle" fontSize="28" fontWeight="700" fill="var(--fg)">
          {data?.value !== undefined ? value.toFixed(decimals) : '--'}
        </text>
        <text x={cx} y={cy + 10} textAnchor="middle" fontSize="11" fill="var(--muted)">
          {unit}
        </text>
        {showPercentage && (
          <text x={cx} y={cy + 26} textAnchor="middle" fontSize="10" fill="var(--muted)">
            {data?.value !== undefined ? `${(pct * 100).toFixed(0)}%` : '--%'}
          </text>
        )}
      </svg>
      {showName && sensor?.name && (
        <div style={{ fontSize: 10, color: 'var(--muted)', textAlign: 'center' }}>{sensor.name}</div>
      )}
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.donut,
  component: DonutWidget,
})

export default DonutWidget
