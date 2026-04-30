import { useEffect, useState } from 'react'
import { useSseConnected } from '../store/dashboardStore'

export function Footer() {
  const sseConnected = useSseConnected()
  const [version, setVersion] = useState('')
  useEffect(() => {
    fetch('/api/version').then(r => r.json()).then(d => setVersion(d.version)).catch(() => {})
  }, [])

  return (
    <div style={{ position: 'fixed', bottom: 12, right: 12, display: 'flex', alignItems: 'center', gap: 8, zIndex: 51 }}>
      <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 11, color: 'var(--muted)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 10px', borderRadius: 4, display: 'flex', alignItems: 'center', gap: 6 }}>
        <span style={{ color: 'var(--cyan)', fontWeight: 600 }}>Voltrus Lite</span>
        {version && <span style={{ opacity: 0.6 }}>v{version}</span>}
      </div>
      <a href="https://joyodigitama.com" target="_blank" rel="noopener noreferrer" style={{ fontFamily: 'Inter,sans-serif', fontSize: 11, color: 'var(--muted)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 10px', borderRadius: 4, textDecoration: 'none' }}>
        by <span style={{ color: 'var(--cyan)', fontWeight: 600 }}>JoyoDigitama</span>
      </a>
      <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 11, color: sseConnected ? 'var(--ok)' : 'var(--warn)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 8px', borderRadius: 4, display: 'flex', alignItems: 'center', gap: 4 }}>
        <span style={{ width: 6, height: 6, borderRadius: '50%', background: sseConnected ? 'var(--ok)' : 'var(--warn)' }} />
        {sseConnected ? 'LIVE' : 'SSE...'}
      </div>
    </div>
  )
}
