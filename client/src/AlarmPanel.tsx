import { useEffect, useRef, useState } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { useAlarms, useAcknowledgeAlarm, useAcknowledgeAllAlarms } from './queries'
import { fmtTs } from './styles'

const FILTER_TABS = ['all', 'active', 'acknowledged'] as const

const useIsMobile = () => {
  const [mobile, setMobile] = useState(window.innerWidth <= 768)
  useEffect(() => {
    const mq = window.matchMedia('(max-width: 768px)')
    const handler = (e: MediaQueryListEvent) => setMobile(e.matches)
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [])
  return mobile
}

/* ── Swipeable alarm row (mobile only) ─────────────────────────── */

function SwipeableAlarmRow({ alarm, isMobile, onAck, children }: {
  alarm: { id: number; acknowledged_at?: number | null }
  isMobile: boolean
  onAck: () => void
  children: React.ReactNode
}) {
  const touchStartX = useRef(0)
  const currentX = useRef(0)
  const rowRef = useRef<HTMLDivElement>(null)

  // Skip wrapper on desktop or already-acknowledged alarms
  if (!isMobile || alarm.acknowledged_at) return <>{children}</>

  const onTouchStart = (e: React.TouchEvent) => {
    touchStartX.current = e.touches[0].clientX
    currentX.current = 0
  }

  const onTouchMove = (e: React.TouchEvent) => {
    const deltaX = e.touches[0].clientX - touchStartX.current
    if (deltaX < 0) return // only swipe right
    const clampedX = Math.min(deltaX, 120)
    currentX.current = clampedX
    if (rowRef.current) {
      rowRef.current.style.transform = `translateX(${clampedX}px)`
      rowRef.current.style.transition = 'none'
    }
  }

  const onTouchEnd = () => {
    if (currentX.current > 80) {
      onAck()
    }
    // Animate back
    if (rowRef.current) {
      rowRef.current.style.transition = 'transform 0.2s ease'
      rowRef.current.style.transform = 'translateX(0)'
    }
    currentX.current = 0
  }

  return (
    <div style={{ position: 'relative', overflow: 'hidden', marginBottom: 8, borderRadius: 8 }}>
      {/* Green ack label behind */}
      <div style={{
        position: 'absolute', inset: 0, display: 'flex', alignItems: 'center',
        justifyContent: 'flex-end', paddingRight: 20,
        background: 'rgba(34,197,94,.2)', borderRadius: 8, color: 'var(--ok)',
        fontWeight: 700, fontSize: 14,
      }}>
        Ack &#10003;
      </div>
      {/* Sliding card */}
      <div
        ref={rowRef}
        onTouchStart={onTouchStart}
        onTouchMove={onTouchMove}
        onTouchEnd={onTouchEnd}
        style={{ position: 'relative', zIndex: 1 }}
      >
        {children}
      </div>
    </div>
  )
}

/* ── Alarm panel ──────────────────────────────────────────────── */

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

  const isMobile = useIsMobile()
  const [filter, setFilter] = useState<string>('active')
  const { data: alarms = [], isLoading } = useAlarms(filter)
  const ackMut = useAcknowledgeAlarm()
  const ackAllMut = useAcknowledgeAllAlarms()

  const parentRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: alarms.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => isMobile ? 120 : 100,
    overscan: 5,
  })

  // Mobile-friendly card sizing
  const cardPad = isMobile ? 16 : 12
  const sevBadgeSize = isMobile ? 12 : 10
  const sourceSize = isMobile ? 15 : 13
  const msgSize = isMobile ? 13 : 12
  const timeSize = isMobile ? 13 : 11

  return (
    <div role="dialog" aria-label="Alarm Panel" style={{
      position: 'fixed', inset: 0, zIndex: 1000,
      display: 'flex',
      justifyContent: isMobile ? 'center' : 'flex-end',
    }}>
      {/* Backdrop – hidden on mobile (panel is full-width, close via X button) */}
      {!isMobile && <div onClick={onClose} style={{ flex: 1, background: 'rgba(0,0,0,.5)' }} />}
      {/* Panel */}
      <div style={{
        width: isMobile ? '100%' : 360,
        height: '100%',
        background: 'var(--bg)',
        borderLeft: isMobile ? 'none' : '1px solid var(--border)',
        display: 'flex', flexDirection: 'column', overflow: 'hidden',
      }}>
        {/* Header */}
        <div style={{ padding: '16px 16px 12px', borderBottom: '1px solid var(--border)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: isMobile ? 17 : 15, fontWeight: 700 }}>Alarms</span>
          <div style={{ display: 'flex', gap: 8 }}>
            <button onClick={() => ackAllMut.mutate()} aria-label="Acknowledge all alarms" style={{ padding: '8px 14px', borderRadius: 6, border: '1px solid var(--warn)', background: 'transparent', color: 'var(--warn)', cursor: 'pointer', fontSize: 11, fontWeight: 600, minHeight: 44 }}>Ack All</button>
            <button onClick={onClose} aria-label="Close alarm panel" style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18, padding: '0 4px', minHeight: 44, minWidth: 44 }}>&times;</button>
          </div>
        </div>

        {/* Filter tabs */}
        <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
          {FILTER_TABS.map(tab => (
            <button key={tab} onClick={() => setFilter(tab)} aria-label={`Filter: ${tab}`} aria-selected={filter === tab} style={{
              flex: 1, padding: isMobile ? '10px 0' : '8px 0', fontSize: 11, fontWeight: 600, textTransform: 'uppercase',
              letterSpacing: '.04em', cursor: 'pointer', border: 'none', background: 'transparent',
              borderBottom: filter === tab ? '2px solid var(--cyan)' : '2px solid transparent',
              color: filter === tab ? 'var(--cyan)' : 'var(--muted)',
            }}>{tab}</button>
          ))}
        </div>

        {/* Alarm list */}
        <div ref={parentRef} role="list" aria-live="polite" style={{ flex: 1, overflow: 'auto', padding: isMobile ? 12 : 8 }}>
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

                const cardContent = (
                  <div role="listitem" style={{
                    padding: cardPad, borderRadius: 8,
                    border: `1px solid ${isActive ? sevColor : 'var(--border)'}`,
                    background: isActive ? sevBg : 'var(--card)',
                    opacity: isResolved ? 0.6 : 1,
                  }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                        {isActive && <div style={{ width: isMobile ? 10 : 8, height: isMobile ? 10 : 8, borderRadius: '50%', background: sevColor, animation: 'pulse 1.5s infinite' }} />}
                        <span style={{ padding: isMobile ? '3px 10px' : '2px 8px', borderRadius: 4, fontSize: sevBadgeSize, fontWeight: 700, textTransform: 'uppercase', background: sevBg, color: sevColor }}>{alarm.severity}</span>
                      </div>
                      <span style={{ fontSize: timeSize, color: 'var(--muted)' }}>{fmtTs(alarm.triggered_at * 1000)}</span>
                    </div>
                    <div style={{ fontSize: sourceSize, fontWeight: 600, marginBottom: 4 }}>{alarm.source_name}</div>
                    <div style={{ fontSize: msgSize, color: 'var(--muted)', marginBottom: 8 }}>{alarm.message}</div>
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

                return (
                  <div key={alarm.id} style={{
                    position: 'absolute', top: 0, left: 0, width: '100%',
                    transform: `translateY(${virtualItem.start}px)`,
                  }}>
                    <SwipeableAlarmRow
                      alarm={alarm}
                      isMobile={isMobile}
                      onAck={() => ackMut.mutate(alarm.id)}
                    >
                      {cardContent}
                    </SwipeableAlarmRow>
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
