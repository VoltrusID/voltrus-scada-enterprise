import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const MotorWidget: React.FC<WidgetProps> = (props) => {
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

  const color = !hasData ? 'var(--muted)' : isRunning ? 'var(--ok)' : 'var(--crit)'
  const bg = isRunning ? 'var(--ok)' : 'var(--card)'
  const bgOpacity = isRunning ? 0.1 : 0

  const spokeAngles = [0, 60, 120, 180, 240, 300]
  const cx = 50
  const cy = 50

  return (
    <div
      style={{
        width: '100%', height: '100%', display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center', cursor: !editMode && onWriteValue ? 'pointer' : 'default',
      }}
      onClick={handleClick}
    >
      <svg viewBox="0 0 100 100" style={{ width: '80%', height: '80%' }} preserveAspectRatio="xMidYMid meet">
        {bgOpacity > 0 && (
          <circle cx={cx} cy={cy} r={40} fill={bg} opacity={bgOpacity} />
        )}

        <circle cx={cx} cy={cy} r={36} fill="none" stroke="var(--border)" strokeWidth={2} />
        <circle cx={cx} cy={cy} r={36} fill="none" stroke={color} strokeWidth={2}
          strokeDasharray={isRunning ? 'none' : '4 4'} opacity={isRunning ? 1 : 0.4}
        />

        <text x={cx} y={cy + 1} textAnchor="middle" dominantBaseline="central"
          fontSize="20" fontWeight="700" fill={color}>
          M
        </text>

        {isRunning && spokeAngles.map((angle, i) => {
          const rad = (angle * Math.PI) / 180
          const x1 = cx + 20 * Math.cos(rad)
          const y1 = cy + 20 * Math.sin(rad)
          const x2 = cx + 34 * Math.cos(rad)
          const y2 = cy + 34 * Math.sin(rad)
          return (
            <line key={i} x1={x1} y1={y1} x2={x2} y2={y2}
              stroke={color} strokeWidth={1.5} opacity={0.3}
              style={{
                transformOrigin: `${cx}px ${cy}px`,
                animation: `motor-spin-${instance.id} 2s linear infinite`,
              }}
            />
          )
        })}

        <line x1="5" y1={cy} x2="14" y2={cy} stroke="var(--muted)" strokeWidth={2} />
        <line x1="86" y1={cy} x2="95" y2={cy} stroke="var(--muted)" strokeWidth={2} />

        <circle cx={cx} cy={cy + 36 + 8} r={3}
          fill={!hasData ? 'var(--muted)' : isRunning ? 'var(--ok)' : 'var(--crit)'}
          style={{ transition: 'fill 0.3s ease' }}
        />
      </svg>

      <style>{`
        @keyframes motor-spin-${instance.id} {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>

      {showValue && (
        <div style={{
          fontSize: 10, color: 'var(--muted)', textAlign: 'center', lineHeight: 1.2,
        }}>
          {hasData ? (isRunning ? 'Running' : 'Stopped') : '--'}
        </div>
      )}
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.motor,
  component: MotorWidget,
})

export default MotorWidget
