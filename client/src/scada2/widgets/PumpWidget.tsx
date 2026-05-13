import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const PumpWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, data, editMode, onWriteValue } = props
  const config = instance.config ?? {}
  const onThreshold = config.onThreshold ?? 0.5
  const showValue = config.showValue !== false

  const value = data?.value
  const hasData = value !== undefined && value !== null
  const isRunning = hasData && value > onThreshold

  const handleClick = () => {
    if (editMode || !onWriteValue) return
    onWriteValue(isRunning ? 0 : 1)
  }

  const color = !hasData ? 'var(--muted)' : isRunning ? 'var(--ok)' : 'var(--fg)'
  const bgOpacity = isRunning ? 0.08 : 0

  const cx = 50
  const cy = 45
  const bladeCount = 3

  return (
    <div
      style={{
        width: '100%', height: '100%', display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center',
        cursor: !editMode && onWriteValue ? 'pointer' : 'default',
      }}
      onClick={handleClick}
    >
      <svg viewBox="0 0 100 90" style={{ width: '85%', height: '80%' }} preserveAspectRatio="xMidYMid meet">
        {bgOpacity > 0 && (
          <circle cx={cx} cy={cy} r={32} fill={color} opacity={bgOpacity} />
        )}

        <line x1="0" y1={cy} x2="14" y2={cy} stroke="var(--muted)" strokeWidth={3} />
        <line x1="86" y1={cy} x2="100" y2={cy} stroke="var(--muted)" strokeWidth={3} />

        <circle cx={cx} cy={cy} r={30} fill="none" stroke={color} strokeWidth={2.5} />

        <g style={{
          transformOrigin: `${cx}px ${cy}px`,
          animation: isRunning ? `pump-spin-${instance.id} 1.5s linear infinite` : 'none',
        }}>
          {Array.from({ length: bladeCount }).map((_, i) => {
            const angle = (i * 360) / bladeCount
            const rad = (angle * Math.PI) / 180
            const tipX = cx + 22 * Math.cos(rad)
            const tipY = cy + 22 * Math.sin(rad)
            const side1X = cx + 22 * Math.cos(rad + 0.3)
            const side1Y = cy + 22 * Math.sin(rad + 0.3)
            const side2X = cx + 22 * Math.cos(rad - 0.3)
            const side2Y = cy + 22 * Math.sin(rad - 0.3)
            return (
              <path key={i}
                d={`M ${cx} ${cy} L ${side1X} ${side1Y} A 22 22 0 0 1 ${tipX} ${tipY} L ${cx} ${cy}`}
                fill={color} opacity={0.25}
              />
            )
          })}
        </g>

        <circle cx={cx} cy={cy} r={6} fill={color} />
        <circle cx={cx} cy={cy} r={3} fill="var(--bg)" />

        <text x={cx} y={cy + 1} textAnchor="middle" dominantBaseline="central"
          fontSize="7" fontWeight="700" fill="var(--bg)">
          {isRunning ? '' : ''}
        </text>

        <circle cx={cx} cy={cy + 40} r={3}
          fill={!hasData ? 'var(--muted)' : isRunning ? 'var(--ok)' : 'var(--border)'}
          style={{ transition: 'fill 0.3s ease' }}
        />
      </svg>

      <style>{`
        @keyframes pump-spin-${instance.id} {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>

      {showValue && (
        <div style={{ fontSize: 9, color: 'var(--muted)', textAlign: 'center', lineHeight: 1.2 }}>
          {hasData ? (isRunning ? 'Running' : 'Stopped') : '--'}
        </div>
      )}
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.pump,
  component: PumpWidget,
})

export default PumpWidget
