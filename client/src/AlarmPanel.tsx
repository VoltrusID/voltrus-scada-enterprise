import { useRef, useState } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { useAlarms, useAcknowledgeAlarm, useAcknowledgeAllAlarms } from './queries'
import { fmtTs } from './styles'

const FILTER_TABS = ['all', 'active', 'acknowledged'] as const

interface Props {
  onClose: () => void
}

export function AlarmPanel({ onClose }: Props) {
  // Inject pulse animation once
  if (typeof document !== 'undefined' && !document.getElementById('alarm-pulse-style')) {
    const style = document.createElement('style')
    style.id = 'alarm-pulse-style'
    style.textContent = '@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }'
    document.head.appendChild(style)
  }

  const [filter, setFilter] = useState<string>('active')
  const { data: alarms = [], isLoading } = useAlarms(filter)
  const ackMut = useAcknowledgeAlarm()
  const ackAllMut = useAcknowledgeAllAlarms()

  const parentRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: alarms.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 100,
    overscan: 5,
  })

  return (
    <div role="dialog" aria-label="Alarm Panel" style={{ position: 'fixed', inset: 0, zIndex: 1000, display: 'flex', justifyContent: 'flex-end' }}>
      <div onClick={onClose} style={{ flex: 1, background: 'rgba(0,0,0,.5)' }} />
      <div style={{ width: 360, height: '100%', background: 'var(--bg)', borderLeft: '1px solid var(--border)', display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {/* Header */}
        <div style={{ padding: '16px 16px 12px', borderBottom: '1px solid var(--border)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: 15, fontWeight: 700 }}>Alarms</span>
          <div style={{ display: 'flex', gap: 8 }}>
            <button onClick={() => ackAllMut.mutate()} aria-label="Acknowledge all alarms" style={{ padding: '8px 14px', borderRadius: 6, border: '1px solid var(--warn)', background: 'transparent', color: 'var(--warn)', cursor: 'pointer', fontSize: 11, fontWeight: 600, minHeight: 44 }}>Ack All</button>
            <button onClick={onClose} aria-label="Close alarm panel" style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18, padding: '0 4px', minHeight: 44, minWidth: 44 }}>&times;</button>
          </div>
        </div>

        {/* Filter tabs */}
        <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
          {FILTER_TABS.map(tab => (
            <button key={tab} onClick={() => setFilter(tab)} aria-label={`Filter: ${tab}`} aria-selected={filter === tab} style={{
              flex: 1, padding: '8px 0', fontSize: 11, fontWeight: 600, textTransform: 'uppercase',
              letterSpacing: '.04em', cursor: 'pointer', border: 'none', background: 'transparent',
              borderBottom: filter === tab ? '2px solid var(--cyan)' : '2px solid transparent',
              color: filter === tab ? 'var(--cyan)' : 'var(--muted)',
            }}>{tab}</button>
          ))}
        </div>

        {/* Alarm list */}
        <div ref={parentRef} role="list" aria-live="polite" style={{ flex: 1, overflow: 'auto', padding: 8 }}>
          {isLoading && <div style={{ padding: 20, color: 'var(--muted)', textAlign: 'center', fontSize: 13 }}>Loading...</div>}
          {!isLoading && alarms.length === 0 && <div style={{ padding: 20, color: 'var(--muted)', textAlign: 'center', fontSize: 13 }}>No alarms</div>}
          {!isLoading && alarms.length > 0 && (
            <div style={{ height: `${virtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }}>
              {virtualizer.getVirtualItems().map(virtualItem => {
                const alarm = alarms[virtualItem.index]
                const isCritical = alarm.severity === 'critical'
                const isActive = !alarm.acknowledged_at
                const isResolved = !!alarm.resolved_at
                const sevColor = isCritical ? 'var(--crit)' : 'var(--warn)'
                const sevBg = isCritical ? 'rgba(239,68,68,.15)' : 'rgba(234,179,8,.15)'
                return (
                  <div key={alarm.id} role="listitem" style={{
                    position: 'absolute', top: 0, left: 0, width: '100%',
                    transform: `translateY(${virtualItem.start}px)`,
                    padding: 12, marginBottom: 8, borderRadius: 8,
                    border: `1px solid ${isActive ? sevColor : 'var(--border)'}`,
                    background: isActive ? sevBg : 'var(--card)',
                    opacity: isResolved ? 0.6 : 1,
                  }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                        {isActive && <div style={{ width: 8, height: 8, borderRadius: '50%', background: sevColor, animation: 'pulse 1.5s infinite' }} />}
                        <span style={{ padding: '2px 8px', borderRadius: 4, fontSize: 10, fontWeight: 700, textTransform: 'uppercase', background: sevBg, color: sevColor }}>{alarm.severity}</span>
                      </div>
                      <span style={{ fontSize: 11, color: 'var(--muted)' }}>{fmtTs(alarm.triggered_at * 1000)}</span>
                    </div>
                    <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>{alarm.source_name}</div>
                    <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 8 }}>{alarm.message}</div>
                    {isActive && (
                      <button onClick={() => ackMut.mutate(alarm.id)} aria-label={`Acknowledge alarm: ${alarm.source_name}`} style={{
                        padding: '8px 14px', borderRadius: 6, fontSize: 11, fontWeight: 600, cursor: 'pointer',
                        border: `1px solid ${sevColor}`, background: 'transparent', color: sevColor, minHeight: 44,
                      }}>Acknowledge</button>
                    )}
                    {alarm.acknowledged_at && (
                      <span style={{ fontSize: 11, color: 'var(--ok)' }}>&#10003; Acknowledged{alarm.acknowledged_by ? ` by ${alarm.acknowledged_by}` : ''}</span>
                    )}
                  </div>
                )
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
