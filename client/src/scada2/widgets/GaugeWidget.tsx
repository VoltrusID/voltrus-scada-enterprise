import React, { useMemo } from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const GaugeWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, data } = props
  const config = instance.config ?? {}
  const showLabels = config.showLabels !== false

  const min = sensor?.min_val ?? 0
  const max = sensor?.max_val ?? 100
  const warn = sensor?.warn_val ?? (min + max) * 0.75
  const crit = sensor?.crit_val ?? (min + max) * 0.9
  const decimals = sensor?.decimals ?? 1
  const unit = sensor?.unit ?? ''

  const value = data?.value ?? min
  const range = max - min || 1
  const pct = Math.max(0, Math.min(1, (value - min) / range))

  // Arc geometry: 270-degree sweep, centered at bottom
  const cx = 100
  const cy = 100
  const r = 80
  const startAngle = 225 // degrees, bottom-left
  const endAngle = -45 // bottom-right (270 sweep)

  const polarToCartesian = (angleDeg: number) => {
    const rad = (angleDeg * Math.PI) / 180
    return { x: cx + r * Math.cos(rad), y: cy - r * Math.sin(rad) }
  }

  const arcPath = (startA: number, endA: number) => {
    const s = polarToCartesian(startA)
    const e = polarToCartesian(endA)
    const sweep = startA - endA
    const largeArc = sweep > 180 ? 1 : 0
    return `M ${s.x} ${s.y} A ${r} ${r} 0 ${largeArc} 0 ${e.x} ${e.y}`
  }

  // Tick marks
  const ticks = useMemo(() => {
    const result: { x1: number; y1: number; x2: number; y2: number; major: boolean }[] = []
    const tickCount = 27
    for (let i = 0; i <= tickCount; i++) {
      const angle = startAngle - (270 * i) / tickCount
      const rad = (angle * Math.PI) / 180
      const major = i % 9 === 0
      const innerR = major ? r - 12 : r - 8
      result.push({
        x1: cx + innerR * Math.cos(rad),
        y1: cy - innerR * Math.sin(rad),
        x2: cx + r * Math.cos(rad),
        y2: cy - r * Math.sin(rad),
        major,
      })
    }
    return result
  }, [])

  // Needle
  const needleAngle = startAngle - 270 * pct
  const needleRad = (needleAngle * Math.PI) / 180
  const needleTip = { x: cx + (r - 18) * Math.cos(needleRad), y: cy - (r - 18) * Math.sin(needleRad) }
  const needleBase1 = { x: cx + 4 * Math.cos(needleRad + Math.PI / 2), y: cy - 4 * Math.sin(needleRad + Math.PI / 2) }
  const needleBase2 = { x: cx + 4 * Math.cos(needleRad - Math.PI / 2), y: cy - 4 * Math.sin(needleRad - Math.PI / 2) }

  // Zone boundaries (percentage of arc)
  const warnPct = Math.max(0, Math.min(1, (warn - min) / range))
  const critPct = Math.max(0, Math.min(1, (crit - min) / range))

  const pctToAngle = (p: number) => startAngle - 270 * p

  // Color for current value
  const valueColor = pct >= critPct ? 'var(--crit)' : pct >= warnPct ? 'var(--warn)' : 'var(--ok)'

  const containerStyle: React.CSSProperties = {
    width: '100%',
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  }

  const labelStyle: React.CSSProperties = {
    fontSize: 12,
    color: 'var(--muted)',
    marginTop: 2,
  }

  return (
    <div style={containerStyle}>
      <svg viewBox="0 0 200 140" style={{ width: '100%', height: '85%' }} preserveAspectRatio="xMidYMid meet">
        {/* Background arc track */}
        <path d={arcPath(startAngle, endAngle)} fill="none" stroke="var(--border)" strokeWidth={8} strokeLinecap="round" />

        {/* Green zone (0 to warn) */}
        {warnPct > 0 && (
          <path
            d={arcPath(startAngle, pctToAngle(warnPct))}
            fill="none" stroke="var(--ok)" strokeWidth={8} strokeLinecap="round" opacity={0.35}
          />
        )}
        {/* Yellow zone (warn to crit) */}
        {critPct > warnPct && (
          <path
            d={arcPath(pctToAngle(warnPct), pctToAngle(critPct))}
            fill="none" stroke="var(--warn)" strokeWidth={8} strokeLinecap="round" opacity={0.4}
          />
        )}
        {/* Red zone (crit to 1) */}
        {critPct < 1 && (
          <path
            d={arcPath(pctToAngle(critPct), endAngle)}
            fill="none" stroke="var(--crit)" strokeWidth={8} strokeLinecap="round" opacity={0.4}
          />
        )}

        {/* Tick marks */}
        {ticks.map((t, i) => (
          <line key={i} x1={t.x1} y1={t.y1} x2={t.x2} y2={t.y2}
            stroke="var(--muted)" strokeWidth={t.major ? 2 : 1} />
        ))}

        {/* Min/Max labels */}
        {showLabels && (
          <>
            <text x={polarToCartesian(startAngle).x - 8} y={polarToCartesian(startAngle).y + 16}
              fontSize="10" fill="var(--muted)" textAnchor="middle">{min}</text>
            <text x={polarToCartesian(endAngle).x + 8} y={polarToCartesian(endAngle).y + 16}
              fontSize="10" fill="var(--muted)" textAnchor="middle">{max}</text>
          </>
        )}

        {/* Needle */}
        <polygon
          points={`${needleTip.x},${needleTip.y} ${needleBase1.x},${needleBase1.y} ${needleBase2.x},${needleBase2.y}`}
          fill={valueColor}
        />
        <circle cx={cx} cy={cy} r={6} fill="var(--fg)" />
        <circle cx={cx} cy={cy} r={3} fill="var(--bg)" />

        {/* Value text */}
        <text x={cx} y={cy + 32} textAnchor="middle" fontSize="16" fontWeight="700" fill="var(--fg)">
          {data?.value !== undefined ? value.toFixed(decimals) : '--'}
        </text>
        {unit && (
          <text x={cx} y={cy + 46} textAnchor="middle" fontSize="11" fill="var(--muted)">
            {unit}
          </text>
        )}
      </svg>
      {showLabels && sensor?.name && <div style={labelStyle}>{sensor.name}</div>}
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.gauge,
  component: GaugeWidget,
})

export default GaugeWidget
