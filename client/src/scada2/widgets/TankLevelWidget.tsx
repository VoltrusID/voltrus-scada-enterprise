import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const TankLevelWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, data } = props
  const config = instance.config ?? {}
  const fillColor = config.fillColor ?? '#3b82f6'
  const showPercentage = config.showPercentage !== false

  const min = sensor?.min_val ?? 0
  const max = sensor?.max_val ?? 100
  const warn = sensor?.warn_val ?? (min + max) * 0.75
  const crit = sensor?.crit_val ?? (min + max) * 0.9
  const decimals = sensor?.decimals ?? 1

  const value = data?.value ?? min
  const range = max - min || 1
  const pct = Math.max(0, Math.min(1, (value - min) / range))

  // Determine fill color based on thresholds
  const activeFill = pct >= (crit - min) / range
    ? 'var(--crit)'
    : pct >= (warn - min) / range
      ? 'var(--warn)'
      : fillColor

  const containerStyle: React.CSSProperties = {
    width: '100%',
    height: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  }

  // SVG tank dimensions
  const tankX = 10
  const tankY = 10
  const tankW = 80
  const tankH = 180
  const cornerR = 4

  // Fill rectangle from bottom
  const fillH = tankH * pct
  const fillY = tankY + tankH - fillH

  return (
    <div style={containerStyle}>
      <svg viewBox="0 0 100 200" style={{ width: '100%', height: '100%' }} preserveAspectRatio="xMidYMid meet">
        {/* Tank outline */}
        <rect
          x={tankX} y={tankY} width={tankW} height={tankH}
          rx={cornerR} ry={cornerR}
          fill="none" stroke="var(--border)" strokeWidth={3}
        />

        {/* Fill level */}
        <clipPath id={`tank-clip-${instance.id}`}>
          <rect x={tankX} y={tankY} width={tankW} height={tankH} rx={cornerR} ry={cornerR} />
        </clipPath>
        <rect
          x={tankX} y={fillY} width={tankW} height={fillH}
          fill={activeFill}
          clipPath={`url(#tank-clip-${instance.id})`}
          style={{ transition: 'y 0.5s ease, height 0.5s ease' }}
        />

        {/* Percentage text */}
        {showPercentage && (
          <text
            x={tankX + tankW / 2}
            y={tankY + tankH / 2 + 5}
            textAnchor="middle"
            fontSize="16"
            fontWeight="700"
            fill="var(--fg)"
          >
            {data?.value !== undefined ? `${(pct * 100).toFixed(0)}%` : '--'}
          </text>
        )}

        {/* Min/Max labels */}
        <text x={tankX + tankW + 4} y={tankY + 6} fontSize="9" fill="var(--muted)">
          {max}
        </text>
        <text x={tankX + tankW + 4} y={tankY + tankH} fontSize="9" fill="var(--muted)">
          {min}
        </text>

        {/* Value text at bottom */}
        <text
          x={tankX + tankW / 2}
          y={tankY + tankH + 16}
          textAnchor="middle"
          fontSize="10"
          fill="var(--muted)"
        >
          {data?.value !== undefined ? `${value.toFixed(decimals)}` : '--'}
        </text>
      </svg>
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.tank_level,
  component: TankLevelWidget,
})

export default TankLevelWidget
