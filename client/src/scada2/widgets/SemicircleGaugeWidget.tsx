import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const SemicircleGaugeWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, data } = props
  const config = instance.config ?? {}
  const showTicks = config.showTicks !== false
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

  const cx = 100
  const cy = 90
  const r = 70

  const toRad = (p: number) => Math.PI * (1 - p)

  const arcPath = (startPct: number, endPct: number) => {
    const s = toRad(startPct)
    const e = toRad(endPct)
    const x1 = cx + r * Math.cos(s)
    const y1 = cy - r * Math.sin(s)
    const x2 = cx + r * Math.cos(e)
    const y2 = cy - r * Math.sin(e)
    const largeArc = (endPct - startPct) > 0.5 ? 1 : 0
    return `M ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2}`
  }

  const ticks = []
  if (showTicks) {
    for (let i = 0; i <= 20; i++) {
      const p = i / 20
      const angle = toRad(p)
      const major = i % 5 === 0
      const innerR = major ? r - 12 : r - 6
      ticks.push({
        x1: cx + (r + 4) * Math.cos(angle),
        y1: cy - (r + 4) * Math.sin(angle),
        x2: cx + innerR * Math.cos(angle),
        y2: cy - innerR * Math.sin(angle),
        major,
        label: major ? (min + range * p).toFixed(0) : null,
        lx: cx + (r + 16) * Math.cos(angle),
        ly: cy - (r + 16) * Math.sin(angle),
      })
    }
  }

  const valueColor = pct >= critPct ? 'var(--crit)' : pct >= warnPct ? 'var(--warn)' : 'var(--ok)'

  const needleAngle = toRad(pct)
  const needleTip = { x: cx + (r - 20) * Math.cos(needleAngle), y: cy - (r - 20) * Math.sin(needleAngle) }
  const base1 = { x: cx + 3 * Math.cos(needleAngle + Math.PI / 2), y: cy - 3 * Math.sin(needleAngle + Math.PI / 2) }
  const base2 = { x: cx + 3 * Math.cos(needleAngle - Math.PI / 2), y: cy - 3 * Math.sin(needleAngle - Math.PI / 2) }

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'flex-end', justifyContent: 'center' }}>
      <svg viewBox="0 0 200 110" style={{ width: '100%', height: '100%' }} preserveAspectRatio="xMidYMid meet">
        <path d={arcPath(0, 1)} fill="none" stroke="var(--border)" strokeWidth={thickness} strokeLinecap="round" />

        {warnPct > 0 && (
          <path d={arcPath(0, warnPct)} fill="none" stroke="var(--ok)" strokeWidth={thickness}
            strokeLinecap="round" opacity={0.3} />
        )}
        {critPct > warnPct && (
          <path d={arcPath(warnPct, critPct)} fill="none" stroke="var(--warn)" strokeWidth={thickness}
            strokeLinecap="butt" opacity={0.35} />
        )}
        {critPct < 1 && (
          <path d={arcPath(critPct, 1)} fill="none" stroke="var(--crit)" strokeWidth={thickness}
            strokeLinecap="round" opacity={0.35} />
        )}

        {ticks.map((t, i) => (
          <g key={i}>
            <line x1={t.x1} y1={t.y1} x2={t.x2} y2={t.y2}
              stroke="var(--muted)" strokeWidth={t.major ? 1.5 : 0.7} />
            {t.label && (
              <text x={t.lx} y={t.ly + 3} textAnchor="middle" fontSize="7" fill="var(--muted)">
                {t.label}
              </text>
            )}
          </g>
        ))}

        <polygon
          points={`${needleTip.x},${needleTip.y} ${base1.x},${base1.y} ${base2.x},${base2.y}`}
          fill={valueColor}
          style={{ transition: 'fill 0.3s ease' }}
        />
        <circle cx={cx} cy={cy} r={5} fill="var(--fg)" />
        <circle cx={cx} cy={cy} r={2.5} fill="var(--bg)" />

        <text x={cx} y={cy - 12} textAnchor="middle" fontSize="18" fontWeight="700" fill="var(--fg)">
          {data?.value !== undefined ? value.toFixed(decimals) : '--'}
        </text>
        {unit && (
          <text x={cx} y={cy + 1} textAnchor="middle" fontSize="9" fill="var(--muted)">
            {unit}
          </text>
        )}
      </svg>
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.semicircle_gauge,
  component: SemicircleGaugeWidget,
})

export default SemicircleGaugeWidget
