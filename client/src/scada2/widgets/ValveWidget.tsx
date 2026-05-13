import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const ValveWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, data, editMode, onWriteValue } = props
  const config = instance.config ?? {}
  const onThreshold = config.onThreshold ?? 0.5
  const showValue = config.showValue !== false
  const failColor = config.failColor ?? 'var(--crit)'

  const value = data?.value
  const hasData = value !== undefined && value !== null
  const isOpen = hasData && value > onThreshold
  const fault = hasData && value > 0 && value < onThreshold && value > 0.01

  const handleClick = () => {
    if (editMode || !onWriteValue) return
    onWriteValue(isOpen ? 0 : 1)
  }

  const statusColor = !hasData ? 'var(--muted)' : isOpen ? 'var(--ok)' : fault ? failColor : 'var(--fg)'

  const butterflyAngle = isOpen ? 0 : 55

  return (
    <div
      style={{
        width: '100%', height: '100%', display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center',
        cursor: !editMode && onWriteValue ? 'pointer' : 'default',
      }}
      onClick={handleClick}
    >
      <svg viewBox="0 0 100 80" style={{ width: '90%', height: '80%' }} preserveAspectRatio="xMidYMid meet">
        <line x1="0" y1="30" x2="22" y2="30" stroke={statusColor} strokeWidth={4} />
        <line x1="78" y1="30" x2="100" y2="30" stroke={statusColor} strokeWidth={4} />

        <path d="M 30 8 L 50 30 L 30 52" fill="none" stroke={statusColor} strokeWidth={2.5} />
        <path d="M 70 8 L 50 30 L 70 52" fill="none" stroke={statusColor} strokeWidth={2.5} />

        <rect x="44" y="0" width="12" height="5" rx="1" fill={statusColor} />
        <line x1="50" y1="5" x2="50" y2="8" stroke={statusColor} strokeWidth={2} />

        <circle cx="50" cy="30" r={isOpen ? 4 : 3} fill={statusColor}
          style={{ transition: 'r 0.3s ease' }} />

        <line x1="44" y1="30" x2="56" y2="30"
          stroke={statusColor} strokeWidth={isOpen ? 1 : 2.5} opacity={isOpen ? 0.3 : 1}
          style={{
            transform: `rotate(${butterflyAngle}deg)`,
            transformOrigin: '50px 30px',
            transition: 'transform 0.4s ease, opacity 0.3s ease',
          }}
        />

        <circle cx="50" cy="68" r={4}
          fill={!hasData ? 'var(--muted)' : isOpen ? 'var(--ok)' : fault ? failColor : 'var(--border)'}
          style={{ transition: 'fill 0.3s ease' }}
        />
      </svg>

      {showValue && (
        <div style={{ fontSize: 9, color: 'var(--muted)', textAlign: 'center', lineHeight: 1.2 }}>
          {hasData ? (isOpen ? 'Open' : fault ? 'Fault' : 'Closed') : '--'}
        </div>
      )}
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.valve,
  component: ValveWidget,
})

export default ValveWidget
