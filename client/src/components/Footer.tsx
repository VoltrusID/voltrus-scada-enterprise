import { useSseConnected } from '../store/dashboardStore'

export function Footer() {
  const sseConnected = useSseConnected()

  return (
    <div style={{ position: 'fixed', bottom: 12, right: 12, display: 'flex', alignItems: 'center', gap: 6, zIndex: 51 }}>
      <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 11, color: 'var(--muted)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 10px', borderRadius: 6, display: 'flex', alignItems: 'center', gap: 6, boxShadow: '0 1px 4px rgba(0,0,0,0.2)' }}>
        <span style={{ color: 'var(--cyan)', fontWeight: 600 }}>Voltrus</span>
        <span style={{ opacity: 0.6 }}>v0.40.0</span>
      </div>
      <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 11, color: sseConnected ? 'var(--ok)' : 'var(--warn)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 8px', borderRadius: 4, display: 'flex', alignItems: 'center', gap: 4 }}>
        <span style={{ width: 6, height: 6, borderRadius: '50%', background: sseConnected ? 'var(--ok)' : 'var(--warn)' }} />
        {sseConnected ? 'LIVE' : 'SSE...'}
      </div>
    </div>
  )
}
