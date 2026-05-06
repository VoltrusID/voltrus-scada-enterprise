import { useState, useEffect } from 'react'
import {
  useHubSites,
  useHubStatus,
  useCreateHubSite,
  useUpdateHubSite,
  useDeleteHubSite,
  useStartHubSite,
  useStopHubSite,
} from './queries/hub'
import { inputStyle, btnStyle } from './styles'
import { useLicense } from './LicenseContext'
import { FeatureBadge } from './components/FeatureBadge'

type SiteRunning = 'running' | 'stopped' | 'connecting'

function getSiteStatus(siteId: number, statuses: { site_id: number; running: boolean; last_seen: number | null }[]): SiteRunning {
  const s = statuses.find(x => x.site_id === siteId)
  if (!s || !s.running) return 'stopped'
  const ago = s.last_seen ? Date.now() / 1000 - s.last_seen : Infinity
  if (ago > 30) return 'connecting'
  return 'running'
}

function StatusDot({ status }: { status: SiteRunning }) {
  const color = status === 'running' ? '#22c55e' : status === 'connecting' ? '#eab308' : '#ef4444'
  return (
    <span style={{
      display: 'inline-block', width: 8, height: 8, borderRadius: '50%', background: color,
      boxShadow: `0 0 6px ${color}`,
      flexShrink: 0,
    }} />
  )
}

function formatLastSeen(ts: number | null): string {
  if (!ts) return 'Never'
  const ago = Date.now() / 1000 - ts
  if (ago < 60) return `${Math.round(ago)}s ago`
  if (ago < 3600) return `${Math.round(ago / 60)}m ago`
  if (ago < 86400) return `${Math.round(ago / 3600)}h ago`
  return `${Math.round(ago / 86400)}d ago`
}

export function HubManager({ onClose, embedded }: { onClose: () => void; embedded?: boolean }) {
  const { data: sites = [], isLoading } = useHubSites()
  const { data: statuses = [] } = useHubStatus()
  const createMut = useCreateHubSite()
  const updateMut = useUpdateHubSite()
  const deleteMut = useDeleteHubSite()
  const startMut = useStartHubSite()
  const stopMut = useStopHubSite()

  const [filter, setFilter] = useState('')
  const [showForm, setShowForm] = useState(false)

  const license = useLicense()
  const isMultiSiteEnabled = license?.feature_flags?.multi_site !== false

  const filtered = sites.filter(s =>
    s.name.toLowerCase().includes(filter.toLowerCase()) ||
    s.url.toLowerCase().includes(filter.toLowerCase()),
  )

  useEffect(() => {
    if (embedded) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose, embedded])

  const handleCreate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const fd = new FormData(e.currentTarget)
    createMut.mutate(
      {
        name: fd.get('name') as string,
        url: (fd.get('url') as string).replace(/\/+$/, ''),
        api_key: fd.get('api_key') as string,
        enabled: true,
      },
      { onSuccess: () => setShowForm(false) },
    )
  }

  const handleDelete = (id: number) => {
    if (!confirm('Remove this site from the hub?')) return
    deleteMut.mutate(id)
  }

  const thStyle: React.CSSProperties = {
    textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)',
    textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600,
    borderBottom: '1px solid var(--border)',
  }
  const tdStyle: React.CSSProperties = {
    padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)',
  }

  if (!isMultiSiteEnabled) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', gap: 16 }}>
        <div style={{
          width: 64, height: 64, borderRadius: 16, background: 'rgba(34,197,94,.08)',
          display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 28,
        }}>
          {'\uD83C\uDF10'}
        </div>
        <h2 style={{ fontSize: 18, fontWeight: 600 }}>Multi-Site Hub</h2>
        <p style={{ color: 'var(--muted)', fontSize: 13, maxWidth: 400, textAlign: 'center' }}>
          Connect and manage multiple Voltrus instances from a single dashboard. Requires Professional tier.
        </p>
        <FeatureBadge tier="Professional" />
      </div>
    )
  }

  const tableContent = (
    <>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
        <span style={{ color: 'var(--muted)', fontSize: 13 }}>
          {sites.length} site{sites.length !== 1 ? 's' : ''}
          {statuses.filter(s => s.running).length > 0 && (
            <span style={{ color: '#22c55e', marginLeft: 8 }}>
              {statuses.filter(s => s.running).length} connected
            </span>
          )}
        </span>
        <div style={{ display: 'flex', gap: 8 }}>
          <input
            placeholder="Search..."
            value={filter}
            onChange={e => setFilter(e.target.value)}
            style={{ ...inputStyle, width: 200 }}
          />
          <button
            style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }}
            onClick={() => setShowForm(f => !f)}
            aria-label="Add site"
          >
            + Add Site
          </button>
        </div>
      </div>

      {showForm && (
        <form onSubmit={handleCreate} style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, marginBottom: 16, padding: 16, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10 }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>Site Name *</label>
            <input name="name" required style={inputStyle} placeholder="Factory B" />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>URL *</label>
            <input name="url" required style={inputStyle} placeholder="https://factory-b.example.com" />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>API Key *</label>
            <input name="api_key" required style={inputStyle} placeholder="vlk_..." type="password" />
          </div>
          <div style={{ display: 'flex', alignItems: 'flex-end', gap: 8 }}>
            <button type="submit" style={btnStyle} disabled={createMut.isPending}>
              {createMut.isPending ? 'Adding...' : 'Add Site'}
            </button>
            <button type="button" style={btnStyle} onClick={() => setShowForm(false)}>Cancel</button>
          </div>
        </form>
      )}

      <div style={{ flex: 1, overflow: 'auto', background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10 }}>
        {isLoading ? (
          <div style={{ padding: 24, textAlign: 'center', color: 'var(--muted)' }}>Loading...</div>
        ) : filtered.length === 0 ? (
          <div style={{ padding: 24, textAlign: 'center', color: 'var(--muted)' }}>
            {sites.length === 0 ? 'No hub sites configured. Click "+ Add Site" to connect a remote Voltrus instance.' : 'No sites match your search.'}
          </div>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={thStyle}>Status</th>
                <th style={thStyle}>Name</th>
                <th style={thStyle}>URL</th>
                <th style={thStyle}>Last Seen</th>
                <th style={thStyle}>Enabled</th>
                <th style={{ ...thStyle, width: 200 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map(s => {
                const status = getSiteStatus(s.id, statuses)
                const siteStatusEntry = statuses.find(x => x.site_id === s.id)
                return (
                  <tr key={s.id}>
                    <td style={tdStyle}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                        <StatusDot status={status} />
                        <span style={{ fontSize: 11, color: 'var(--muted)' }}>
                          {status === 'running' ? 'Connected' : status === 'connecting' ? 'Connecting...' : 'Disconnected'}
                        </span>
                      </div>
                    </td>
                    <td style={{ ...tdStyle, fontWeight: 500 }}>{s.name}</td>
                    <td style={{ ...tdStyle, fontFamily: "'JetBrains Mono',monospace", fontSize: 12 }}>{s.url}</td>
                    <td style={{ ...tdStyle, color: 'var(--muted)', fontSize: 12 }}>
                      {formatLastSeen(siteStatusEntry?.last_seen ?? s.last_seen)}
                    </td>
                    <td style={tdStyle}>
                      <input
                        type="checkbox"
                        checked={s.enabled}
                        onChange={() => updateMut.mutate({ id: s.id, enabled: !s.enabled })}
                        style={{ cursor: 'pointer' }}
                      />
                    </td>
                    <td style={tdStyle}>
                      <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                        {status === 'running' ? (
                          <button
                            onClick={() => stopMut.mutate(s.id)}
                            style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--crit)', background: 'transparent', color: 'var(--crit)', cursor: 'pointer', fontSize: 11 }}
                            aria-label={`Stop ${s.name}`}
                          >
                            Stop
                          </button>
                        ) : (
                          <button
                            onClick={() => startMut.mutate(s.id)}
                            style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--green)', background: 'transparent', color: 'var(--green)', cursor: 'pointer', fontSize: 11 }}
                            aria-label={`Start ${s.name}`}
                          >
                            Start
                          </button>
                        )}
                        <button
                          onClick={() => handleDelete(s.id)}
                          style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--crit)', background: 'transparent', color: 'var(--crit)', cursor: 'pointer', fontSize: 11 }}
                          aria-label={`Delete ${s.name}`}
                        >
                          Del
                        </button>
                      </div>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        )}
      </div>
    </>
  )

  if (embedded) {
    return <>{tableContent}</>
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 24 }}>
      <div style={{ maxWidth: 1200, width: '100%', margin: '0 auto', display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16, flexShrink: 0 }}>
          <button onClick={onClose} aria-label="Close hub" style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
          </button>
          <h1 style={{ fontSize: 20, fontWeight: 700 }}>Multi-Site Hub</h1>
        </div>
        {tableContent}
      </div>
    </div>
  )
}
