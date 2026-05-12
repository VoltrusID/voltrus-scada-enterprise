import { useState, useEffect } from 'react'
import {
  useCameras,
  useCreateCamera,
  useUpdateCamera,
  useDeleteCamera,
  useCameraSnapshots,
  useCaptureSnapshot,
  useDeleteSnapshot,
  useStreamInfo,
} from './queries/video'
import type { Camera, CameraSnapshot } from './queries/video'
import { inputStyle, btnStyle } from './styles'
import { useLicense } from './LicenseContext'
import { FeatureBadge } from './components/FeatureBadge'

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleString('en-US', {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', hour12: false,
  })
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function isValidRtspUrl(url: string): boolean {
  return /^rtsp(s?):\/\/.+/i.test(url.trim())
}

function CameraSnapshotsPanel({ camera }: { camera: Camera }) {
  const { data: snapshots = [], isLoading } = useCameraSnapshots(camera.id)
  const captureMut = useCaptureSnapshot(camera.id)
  const deleteMut = useDeleteSnapshot()
  const { data: streamInfo } = useStreamInfo(camera.id)
  const [expanded, setExpanded] = useState(false)

  return (
    <div style={{ marginTop: 8 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <button
          onClick={() => setExpanded(v => !v)}
          style={{
            padding: '4px 10px',
            borderRadius: 4,
            border: '1px solid var(--border)',
            background: 'transparent',
            color: 'var(--fg)',
            cursor: 'pointer',
            fontSize: 11,
          }}
        >
          {expanded ? 'Hide Snapshots' : `Snapshots (${snapshots.length})`}
        </button>
        <button
          onClick={() => captureMut.mutate()}
          disabled={captureMut.isPending}
          style={{
            padding: '4px 10px',
            borderRadius: 4,
            border: '1px solid #27ae60',
            background: 'transparent',
            color: '#27ae60',
            cursor: captureMut.isPending ? 'not-allowed' : 'pointer',
            fontSize: 11,
            opacity: captureMut.isPending ? 0.5 : 1,
          }}
          aria-label="Capture snapshot"
        >
          {captureMut.isPending ? 'Capturing...' : 'Capture'}
        </button>
        {streamInfo && (
          <span style={{
            fontSize: 10,
            color: '#27ae60',
            background: 'rgba(39,174,96,0.1)',
            padding: '2px 8px',
            borderRadius: 3,
            display: 'flex',
            alignItems: 'center',
            gap: 4,
          }}>
            <span style={{ width: 6, height: 6, borderRadius: '50%', background: '#27ae60' }} />
            Stream ({streamInfo.protocol})
          </span>
        )}
      </div>

      {expanded && (
        <div style={{
          background: 'var(--bg)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          overflow: 'hidden',
        }}>
          {isLoading ? (
            <div style={{ padding: 16, textAlign: 'center', color: 'var(--muted)', fontSize: 12 }}>
              Loading snapshots...
            </div>
          ) : snapshots.length === 0 ? (
            <div style={{ padding: 16, textAlign: 'center', color: 'var(--muted)', fontSize: 12 }}>
              No snapshots yet. Click "Capture" to take one.
            </div>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr>
                  <th style={{ textAlign: 'left', padding: '6px 10px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600, borderBottom: '1px solid var(--border)' }}>Time</th>
                  <th style={{ textAlign: 'left', padding: '6px 10px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600, borderBottom: '1px solid var(--border)' }}>Source</th>
                  <th style={{ textAlign: 'left', padding: '6px 10px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600, borderBottom: '1px solid var(--border)' }}>Size</th>
                  <th style={{ textAlign: 'right', padding: '6px 10px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600, borderBottom: '1px solid var(--border)' }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {snapshots.map((snap: CameraSnapshot) => (
                  <tr key={snap.id}>
                    <td style={{ padding: '6px 10px', fontSize: 12, borderBottom: '1px solid var(--border)', fontFamily: "'JetBrains Mono',monospace" }}>
                      {formatTime(snap.timestamp)}
                    </td>
                    <td style={{ padding: '6px 10px', fontSize: 12, borderBottom: '1px solid var(--border)' }}>
                      {snap.alarm_source ? (
                        <span style={{ color: '#e74c3c', background: 'rgba(231,76,60,0.1)', padding: '1px 6px', borderRadius: 3, fontSize: 11 }}>
                          {snap.alarm_source}
                        </span>
                      ) : (
                        <span style={{ color: 'var(--muted)' }}>Manual</span>
                      )}
                    </td>
                    <td style={{ padding: '6px 10px', fontSize: 12, borderBottom: '1px solid var(--border)', color: 'var(--muted)' }}>
                      {formatSize(snap.file_size)}
                    </td>
                    <td style={{ padding: '6px 10px', borderBottom: '1px solid var(--border)', textAlign: 'right' }}>
                      <button
                        onClick={() => deleteMut.mutate({ cameraId: camera.id, snapshotId: snap.id })}
                        style={{
                          padding: '2px 8px',
                          borderRadius: 4,
                          border: '1px solid #e74c3c',
                          background: 'transparent',
                          color: '#e74c3c',
                          cursor: 'pointer',
                          fontSize: 10,
                        }}
                        aria-label="Delete snapshot"
                      >
                        Del
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </div>
  )
}

function CameraEditForm({ camera, onSave, onCancel }: {
  camera: Camera
  onSave: (data: { id: number; name?: string; url?: string; enabled?: boolean; snapshot_on_alarm?: boolean; retention_hours?: number }) => void
  onCancel: () => void
}) {
  const [name, setName] = useState(camera.name)
  const [url, setUrl] = useState(camera.url)
  const [enabled, setEnabled] = useState(camera.enabled)
  const [snapshotOnAlarm, setSnapshotOnAlarm] = useState(camera.snapshot_on_alarm)
  const [retention, setRetention] = useState(String(camera.retention_hours))
  const [urlError, setUrlError] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!isValidRtspUrl(url)) {
      setUrlError('Invalid RTSP URL (must start with rtsp:// or rtsps://)')
      return
    }
    const retentionVal = parseInt(retention, 10)
    onSave({
      id: camera.id,
      name: name !== camera.name ? name : undefined,
      url: url !== camera.url ? url : undefined,
      enabled: enabled !== camera.enabled ? enabled : undefined,
      snapshot_on_alarm: snapshotOnAlarm !== camera.snapshot_on_alarm ? snapshotOnAlarm : undefined,
      retention_hours: !isNaN(retentionVal) && retentionVal !== camera.retention_hours ? retentionVal : undefined,
    })
  }

  return (
    <form onSubmit={handleSubmit} style={{
      padding: 16,
      background: 'var(--bg)',
      border: '1px solid var(--border)',
      borderRadius: 8,
      marginBottom: 12,
      display: 'grid',
      gridTemplateColumns: '1fr 1fr',
      gap: 12,
    }}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>Name</label>
        <input value={name} onChange={e => setName(e.target.value)} required style={inputStyle} />
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>RTSP URL</label>
        <input value={url} onChange={e => { setUrl(e.target.value); setUrlError('') }} required style={{ ...inputStyle, borderColor: urlError ? '#e74c3c' : undefined }} />
        {urlError && <span style={{ fontSize: 10, color: '#e74c3c' }}>{urlError}</span>}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>Retention (hours)</label>
        <input type="number" value={retention} onChange={e => setRetention(e.target.value)} min={1} style={inputStyle} />
      </div>
      <div style={{ display: 'flex', alignItems: 'flex-end', gap: 12 }}>
        <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--fg)', cursor: 'pointer' }}>
          <input type="checkbox" checked={enabled} onChange={e => setEnabled(e.target.checked)} style={{ cursor: 'pointer' }} />
          Enabled
        </label>
        <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--fg)', cursor: 'pointer' }}>
          <input type="checkbox" checked={snapshotOnAlarm} onChange={e => setSnapshotOnAlarm(e.target.checked)} style={{ cursor: 'pointer' }} />
          Snapshot on Alarm
        </label>
      </div>
      <div style={{ gridColumn: '1 / -1', display: 'flex', gap: 8 }}>
        <button type="submit" style={btnStyle}>Save</button>
        <button type="button" style={btnStyle} onClick={onCancel}>Cancel</button>
      </div>
    </form>
  )
}

export function CameraManager({ onClose, embedded }: { onClose: () => void; embedded?: boolean }) {
  const { data: cameras = [], isLoading } = useCameras()
  const createMut = useCreateCamera()
  const updateMut = useUpdateCamera()
  const deleteMut = useDeleteCamera()

  const [filter, setFilter] = useState('')
  const [showForm, setShowForm] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [formError, setFormError] = useState('')

  const license = useLicense()
  const isVideoEnabled = license?.feature_flags?.rtsp_video !== false

  useEffect(() => {
    if (embedded) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose, embedded])

  const filtered = cameras.filter(c =>
    c.name.toLowerCase().includes(filter.toLowerCase()) ||
    c.url.toLowerCase().includes(filter.toLowerCase()),
  )

  const handleCreate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    setFormError('')
    const fd = new FormData(e.currentTarget)
    const url = (fd.get('url') as string).trim()
    if (!isValidRtspUrl(url)) {
      setFormError('Invalid RTSP URL (must start with rtsp:// or rtsps://)')
      return
    }
    createMut.mutate(
      {
        name: fd.get('name') as string,
        url,
        enabled: fd.get('enabled') === 'on',
        snapshot_on_alarm: fd.get('snapshot_on_alarm') === 'on',
        retention_hours: parseInt(fd.get('retention_hours') as string, 10) || 24,
      },
      { onSuccess: () => setShowForm(false) },
    )
  }

  const handleDelete = (id: number, name: string) => {
    if (!confirm(`Delete camera "${name}"? This cannot be undone.`)) return
    deleteMut.mutate(id)
  }

  const handleUpdate = (data: { id: number; name?: string; url?: string; enabled?: boolean; snapshot_on_alarm?: boolean; retention_hours?: number }) => {
    updateMut.mutate(data, { onSuccess: () => setEditingId(null) })
  }

  if (!isVideoEnabled) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', gap: 16 }}>
        <div style={{
          width: 64, height: 64, borderRadius: 16, background: 'rgba(34,197,94,.08)',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
        }}>
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M23 7l-7 5 7 5V7z"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
        </div>
        <h2 style={{ fontSize: 18, fontWeight: 600 }}>RTSP Video</h2>
        <p style={{ color: 'var(--muted)', fontSize: 13, maxWidth: 400, textAlign: 'center' }}>
          Connect and manage RTSP camera feeds with alarm-triggered snapshots. Requires Enterprise tier.
        </p>
        <FeatureBadge tier="Enterprise" />
      </div>
    )
  }

  const thStyle: React.CSSProperties = {
    textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)',
    textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600,
    borderBottom: '1px solid var(--border)',
  }
  const tdStyle: React.CSSProperties = {
    padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)',
  }

  const tableContent = (
    <>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
        <span style={{ color: 'var(--muted)', fontSize: 13 }}>
          {cameras.length} camera{cameras.length !== 1 ? 's' : ''}
          {cameras.filter(c => c.enabled).length > 0 && (
            <span style={{ color: '#27ae60', marginLeft: 8 }}>
              {cameras.filter(c => c.enabled).length} active
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
            aria-label="Add camera"
          >
            + Add Camera
          </button>
        </div>
      </div>

      {showForm && (
        <form onSubmit={handleCreate} style={{
          display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, marginBottom: 16,
          padding: 16, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10,
        }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>Name *</label>
            <input name="name" required style={inputStyle} placeholder="Entrance Camera" />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>RTSP URL *</label>
            <input name="url" required style={{ ...inputStyle, borderColor: formError ? '#e74c3c' : undefined }} placeholder="rtsp://admin:pass@192.168.1.100:554/stream" />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>Retention (hours)</label>
            <input name="retention_hours" type="number" defaultValue={24} min={1} style={inputStyle} />
          </div>
          <div style={{ display: 'flex', alignItems: 'flex-end', gap: 12, paddingBottom: 2 }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--fg)', cursor: 'pointer' }}>
              <input name="enabled" type="checkbox" defaultChecked style={{ cursor: 'pointer' }} /> Enabled
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--fg)', cursor: 'pointer' }}>
              <input name="snapshot_on_alarm" type="checkbox" defaultChecked style={{ cursor: 'pointer' }} /> Snapshot on Alarm
            </label>
          </div>
          {formError && (
            <div style={{ gridColumn: '1 / -1', fontSize: 12, color: '#e74c3c' }}>{formError}</div>
          )}
          <div style={{ display: 'flex', gap: 8 }}>
            <button type="submit" style={btnStyle} disabled={createMut.isPending}>
              {createMut.isPending ? 'Adding...' : 'Add Camera'}
            </button>
            <button type="button" style={btnStyle} onClick={() => { setShowForm(false); setFormError('') }}>Cancel</button>
          </div>
        </form>
      )}

      <div style={{ flex: 1, overflow: 'auto', background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10 }}>
        {isLoading ? (
          <div style={{ padding: 24, textAlign: 'center', color: 'var(--muted)' }}>Loading cameras...</div>
        ) : filtered.length === 0 ? (
          <div style={{ padding: 24, textAlign: 'center', color: 'var(--muted)' }}>
            {cameras.length === 0 ? 'No cameras configured. Click "+ Add Camera" to set up an RTSP feed.' : 'No cameras match your search.'}
          </div>
        ) : (
          <div>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr>
                  <th style={thStyle}>Name</th>
                  <th style={thStyle}>URL</th>
                  <th style={thStyle}>Enabled</th>
                  <th style={thStyle}>Alarm Snap</th>
                  <th style={thStyle}>Retention</th>
                  <th style={{ ...thStyle, width: 160 }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map(c => (
                  <tr key={c.id}>
                    <td style={{ ...tdStyle, fontWeight: 500 }}>{c.name}</td>
                    <td style={{ ...tdStyle, fontFamily: "'JetBrains Mono',monospace", fontSize: 11, maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={c.url}>
                      {c.url}
                    </td>
                    <td style={tdStyle}>
                      <input
                        type="checkbox"
                        checked={c.enabled}
                        onChange={() => updateMut.mutate({ id: c.id, enabled: !c.enabled })}
                        style={{ cursor: 'pointer' }}
                      />
                    </td>
                    <td style={tdStyle}>
                      <input
                        type="checkbox"
                        checked={c.snapshot_on_alarm}
                        onChange={() => updateMut.mutate({ id: c.id, snapshot_on_alarm: !c.snapshot_on_alarm })}
                        style={{ cursor: 'pointer' }}
                      />
                    </td>
                    <td style={{ ...tdStyle, color: 'var(--muted)', fontSize: 12 }}>
                      {c.retention_hours}h
                    </td>
                    <td style={tdStyle}>
                      <div style={{ display: 'flex', gap: 4 }}>
                        <button
                          onClick={() => setEditingId(editingId === c.id ? null : c.id)}
                          style={{
                            padding: '4px 8px', borderRadius: 4, border: '1px solid var(--border)',
                            background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 11,
                          }}
                          aria-label={`Edit ${c.name}`}
                        >
                          Edit
                        </button>
                        <button
                          onClick={() => handleDelete(c.id, c.name)}
                          style={{
                            padding: '4px 8px', borderRadius: 4, border: '1px solid var(--crit)',
                            background: 'transparent', color: 'var(--crit)', cursor: 'pointer', fontSize: 11,
                          }}
                          aria-label={`Delete ${c.name}`}
                        >
                          Del
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            {/* Inline edit forms and snapshot panels */}
            {filtered.map(c => (
              <div key={`detail-${c.id}`}>
                {editingId === c.id && (
                  <div style={{ padding: '0 12px 12px' }}>
                    <CameraEditForm
                      camera={c}
                      onSave={handleUpdate}
                      onCancel={() => setEditingId(null)}
                    />
                  </div>
                )}
                <div style={{ padding: '4px 12px 8px', borderTop: '1px solid var(--border)' }}>
                  <CameraSnapshotsPanel camera={c} />
                </div>
              </div>
            ))}
          </div>
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
          <button onClick={onClose} aria-label="Close cameras" style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
          </button>
          <h1 style={{ fontSize: 20, fontWeight: 700 }}>RTSP Cameras</h1>
        </div>
        {tableContent}
      </div>
    </div>
  )
}
