import type { SystemInfo } from '../types'

interface Props {
  systemInfo: SystemInfo | null | undefined
}

export function SystemPanel({ systemInfo }: Props) {
  if (!systemInfo) {
    return <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 300, color: 'var(--muted)' }}>Connecting to server...</div>
  }

  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: 16, padding: 8 }}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
        <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>Host</div>
        <div style={{ fontSize: 18, fontWeight: 700 }}>{systemInfo.hostname}</div>
        <div style={{ fontSize: 13, color: 'var(--muted)' }}>{systemInfo.os} — up {Math.floor(systemInfo.uptime_secs / 86400)}d {Math.floor((systemInfo.uptime_secs % 86400) / 3600)}h {Math.floor((systemInfo.uptime_secs % 3600) / 60)}m</div>
      </div>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
        <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>CPU Usage</div>
        <div style={{ fontSize: 28, fontWeight: 700, color: systemInfo.cpu_usage > 80 ? 'var(--crit)' : systemInfo.cpu_usage > 50 ? 'var(--warn)' : 'var(--ok)' }}>{systemInfo.cpu_usage.toFixed(1)}%</div>
        <div style={{ marginTop: 8, height: 6, borderRadius: 3, background: 'var(--border)' }}>
          <div style={{ height: '100%', borderRadius: 3, background: systemInfo.cpu_usage > 80 ? 'var(--crit)' : systemInfo.cpu_usage > 50 ? 'var(--warn)' : 'var(--ok)', width: `${Math.min(systemInfo.cpu_usage, 100)}%`, transition: 'width 0.3s' }} />
        </div>
      </div>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
        <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>Memory</div>
        <div style={{ fontSize: 28, fontWeight: 700 }}>{systemInfo.memory_used_mb.toFixed(0)} <span style={{ fontSize: 14, color: 'var(--muted)' }}>/ {systemInfo.memory_total_mb.toFixed(0)} MB</span></div>
        <div style={{ fontSize: 13, color: 'var(--muted)', marginTop: 4 }}>{systemInfo.memory_percent.toFixed(1)}%</div>
        <div style={{ marginTop: 8, height: 6, borderRadius: 3, background: 'var(--border)' }}>
          <div style={{ height: '100%', borderRadius: 3, background: systemInfo.memory_percent > 90 ? 'var(--crit)' : systemInfo.memory_percent > 70 ? 'var(--warn)' : 'var(--ok)', width: `${Math.min(systemInfo.memory_percent, 100)}%`, transition: 'width 0.3s' }} />
        </div>
      </div>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
        <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>Disk</div>
        <div style={{ fontSize: 28, fontWeight: 700 }}>{systemInfo.disk_used_gb.toFixed(1)} <span style={{ fontSize: 14, color: 'var(--muted)' }}>/ {systemInfo.disk_total_gb.toFixed(1)} GB</span></div>
        <div style={{ fontSize: 13, color: 'var(--muted)', marginTop: 4 }}>{systemInfo.disk_percent.toFixed(1)}%</div>
        <div style={{ marginTop: 8, height: 6, borderRadius: 3, background: 'var(--border)' }}>
          <div style={{ height: '100%', borderRadius: 3, background: systemInfo.disk_percent > 90 ? 'var(--crit)' : systemInfo.disk_percent > 70 ? 'var(--warn)' : 'var(--ok)', width: `${Math.min(systemInfo.disk_percent, 100)}%`, transition: 'width 0.3s' }} />
        </div>
      </div>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
        <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>Database</div>
        <div style={{ fontSize: 28, fontWeight: 700, color: systemInfo.db_size_mb > 512 ? 'var(--warn)' : 'var(--cyan)' }}>{systemInfo.db_size_mb > 1024 ? `${(systemInfo.db_size_mb / 1024).toFixed(1)} GB` : `${systemInfo.db_size_mb.toFixed(1)} MB`}</div>
        <div style={{ fontSize: 13, color: 'var(--muted)', marginTop: 4 }}>{systemInfo.db_size_mb > 1024 ? `${systemInfo.db_size_mb.toFixed(0)} MB` : 'SQLite file'}</div>
      </div>
      <div style={{ background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)', border: '1px solid #22c55e', borderRadius: 10, padding: 20, gridColumn: 'span 2' }}>
        <div style={{ fontSize: 11, color: '#22c55e', textTransform: 'uppercase', letterSpacing: '.08em', marginBottom: 12, fontWeight: 700 }}>Rust Service (PID {systemInfo.process.pid})</div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16 }}>
          <div>
            <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{systemInfo.process.memory_mb.toFixed(1)} MB</div>
            <div style={{ fontSize: 11, color: 'var(--muted)' }}>RSS Memory</div>
          </div>
          <div>
            <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{systemInfo.process.cpu_percent.toFixed(1)}%</div>
            <div style={{ fontSize: 11, color: 'var(--muted)' }}>CPU Usage</div>
          </div>
          <div>
            <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{Math.floor(systemInfo.process.uptime_secs / 86400)}d {Math.floor((systemInfo.process.uptime_secs % 86400) / 3600)}h {Math.floor((systemInfo.process.uptime_secs % 3600) / 60)}m</div>
            <div style={{ fontSize: 11, color: 'var(--muted)' }}>Uptime</div>
          </div>
          <div>
            <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{systemInfo.process.binary_size_mb.toFixed(1)} MB</div>
            <div style={{ fontSize: 11, color: 'var(--muted)' }}>Binary Size</div>
          </div>
        </div>
      </div>
    </div>
  )
}
