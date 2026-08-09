import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const HorizontalTankWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, data } = props
  const config = instance.config ?? {}
  const fillColor = config.fillColor ?? '#3b82f6'
  const showPercentage = config.showPercentage !== false

  const min = sensor?.min_val ?? 0
  const max = sensor?.max_val ?? 100
  const warn = sensor?.warn_val ?? min + (max - min) * 0.75
  const crit = sensor?.crit_val ?? min + (max - min) * 0.9
  const decimals = sensor?.decimals ?? 1

  const value = data?.value ?? min
  const range = max - min || 1
  const pct = Math.max(0, Math.min(1, (value - min) / range))

  const activeFill = pct >= (crit - min) / range
    ? 'var(--crit)' : pct >= (warn - min) / range
    ? 'var(--warn)' : fillColor

  const tankX = 10
  const tankY = 20
  const tankW = 180
  const tankH = 80
  const endCapR = tankH / 2

  const fillW = tankW * pct

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <svg viewBox="0 0 200 120" style={{ width: '100%', height: '100%' }} preserveAspectRatio="xMidYMid meet">
        <defs>
          <clipPath id={`htank-clip-${instance.id}`}>
            <rect x={tankX} y={tankY} width={tankW} height={tankH} rx={endCapR} ry={endCapR} />
          </clipPath>
          <linearGradient id={`htank-fill-${instance.id}`} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor={activeFill} stopOpacity={0.7} />
            <stop offset="100%" stopColor={activeFill} />
          </linearGradient>
        </defs>

        <rect x={tankX} y={tankY} width={tankW} height={tankH} rx={endCapR} ry={endCapR}
          fill="none" stroke="var(--border)" strokeWidth={2.5} />

        {pct > 0 && (
          <rect
            x={tankX} y={tankY}
            width={Math.max(endCapR * 2, fillW)}
            height={tankH}
            rx={endCapR} ry={endCapR}
            fill={`url(#htank-fill-${instance.id})`}
            clipPath={`url(#htank-clip-${instance.id})`}
            style={{ transition: 'width 0.5s ease' }}
          />
        )}

        {showPercentage && (
          <text x={tankX + tankW / 2} y={tankY + tankH / 2 + 4}
            textAnchor="middle" fontSize="14" fontWeight="700" fill="var(--fg)">
            {data?.value !== undefined ? `${(pct * 100).toFixed(0)}%` : '--'}
          </text>
        )}

        <text x={tankX + tankW / 2} y={tankY + tankH + 14}
          textAnchor="middle" fontSize="9" fill="var(--muted)">
          {data?.value !== undefined ? `${value.toFixed(decimals)}` : '--'}
        </text>

        {[0.25, 0.5, 0.75].map((mark, i) => {
          const x = tankX + tankW * mark
          return (
            <line key={i} x1={x} y1={tankY - 3} x2={x} y2={tankY + 2}
              stroke="var(--muted)" strokeWidth={1} />
          )
        })}
        <text x={tankX} y={tankY - 5} fontSize="7" fill="var(--muted)">{min}</text>
        <text x={tankX + tankW} y={tankY - 5} fontSize="7" fill="var(--muted)" textAnchor="end">{max}</text>

        <line x1={tankX + tankW / 2} y1={tankY - 4} x2={tankX + tankW / 2} y2={tankY + tankH + 4}
          stroke="var(--fg)" strokeWidth={0.5} strokeDasharray="3 3" opacity={0.3} />
      </svg>
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.horizontal_tank,
  component: HorizontalTankWidget,
})

export default HorizontalTankWidget
