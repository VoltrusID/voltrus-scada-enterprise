import { useState } from 'react'
import {
  useReports,
  useCreateReport,
  useUpdateReport,
  useDeleteReport,
  useGenerateReport,
} from '../queries/reports'
import type { Report } from '../queries/reports'
import { inputStyle, btnStyle } from '@voltrus/core/styles'

const SCHEDULE_COLORS: Record<string, string> = {
  daily: '#3b82f6',
  weekly: '#22c55e',
  monthly: '#a855f7',
}

function ScheduleBadge({ schedule }: { schedule: string }) {
  const color = SCHEDULE_COLORS[schedule] || 'var(--muted)'
  return (
    <span style={{
      display: 'inline-flex',
      alignItems: 'center',
      gap: 4,
      fontSize: 11,
      color,
      background: `${color}15`,
      border: `1px solid ${color}30`,
      borderRadius: 4,
      padding: '2px 8px',
      fontWeight: 600,
      textTransform: 'uppercase',
    }}>
      {schedule}
    </span>
  )
}

function formatLastGenerated(ts: string | null): string {
  if (!ts) return 'Never'
  const d = new Date(ts)
  const now = new Date()
  const ago = now.getTime() - d.getTime()
  if (ago < 60_000) return `${Math.round(ago / 1000)}s ago`
  if (ago < 3_600_000) return `${Math.round(ago / 60_000)}m ago`
  if (ago < 86_400_000) return `${Math.round(ago / 3_600_000)}h ago`
  return d.toLocaleDateString()
}

function openReportWindow(html: string) {
  const win = window.open('', '_blank')
  if (!win) return
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')
  win.document.replaceChild(
    doc.documentElement,
    win.document.documentElement,
  )
}

interface EditState {
  id: number
  name: string
  schedule: string
  email_recipients: string
}

export function ReportManager({ onClose, embedded }: { onClose: () => void; embedded?: boolean }) {
  const { data: reports = [], isLoading } = useReports()
  const createMut = useCreateReport()
  const updateMut = useUpdateReport()
  const deleteMut = useDeleteReport()
  const generateMut = useGenerateReport()

  const [showForm, setShowForm] = useState(false)
  const [editState, setEditState] = useState<EditState | null>(null)
  const [filter, setFilter] = useState('')

  const filtered = reports.filter(r =>
    r.name.toLowerCase().includes(filter.toLowerCase()),
  )

  const handleCreate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const fd = new FormData(e.currentTarget)
    const recipients = (fd.get('email_recipients') as string)
      .split('\n')
      .map((s: string) => s.trim())
      .filter(Boolean)
    createMut.mutate(
      {
        name: fd.get('name') as string,
        schedule: fd.get('schedule') as string,
        email_recipients: recipients,
        enabled: fd.get('enabled') === 'on',
      },
      { onSuccess: () => setShowForm(false) },
    )
  }

  const handleUpdate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    if (!editState) return
    const fd = new FormData(e.currentTarget)
    const recipients = (fd.get('email_recipients') as string)
      .split('\n')
      .map((s: string) => s.trim())
      .filter(Boolean)
    updateMut.mutate(
      {
        id: editState.id,
        name: fd.get('name') as string,
        schedule: fd.get('schedule') as string,
        email_recipients: recipients,
      },
      { onSuccess: () => setEditState(null) },
    )
  }

  const handleDelete = (r: Report) => {
    if (!confirm(`Delete report "${r.name}"?`)) return
    deleteMut.mutate(r.id)
  }

  const handleGenerate = async (id: number) => {
    try {
      const html = await generateMut.mutateAsync(id)
      openReportWindow(html)
    } catch {
      alert('Failed to generate report')
    }
  }

  const startEdit = (r: Report) => {
    setEditState({
      id: r.id,
      name: r.name,
      schedule: r.schedule,
      email_recipients: r.email_recipients.join('\n'),
    })
  }

  const thStyle: React.CSSProperties = {
    textAlign: 'left',
    padding: '8px 12px',
    fontSize: 11,
    color: 'var(--muted)',
    textTransform: 'uppercase',
    letterSpacing: '.05em',
    fontWeight: 600,
    borderBottom: '1px solid var(--border)',
  }

  const tdStyle: React.CSSProperties = {
    padding: '8px 12px',
    fontSize: 13,
    borderBottom: '1px solid var(--border)',
  }

  const formGridStyle: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: 12,
    marginBottom: 16,
    padding: 16,
    background: 'var(--card)',
    border: '1px solid var(--border)',
    borderRadius: 10,
  }

  const fieldStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: 4,
  }

  const labelStyle: React.CSSProperties = {
    fontSize: 11,
    color: 'var(--muted)',
    textTransform: 'uppercase',
  }

  const tableContent = (
    <>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
        <span style={{ color: 'var(--muted)', fontSize: 13 }}>
          {reports.length} report{reports.length !== 1 ? 's' : ''}
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
            aria-label="Add report"
          >
            + Add Report
          </button>
        </div>
      </div>

      {showForm && (
        <form onSubmit={handleCreate} style={formGridStyle}>
          <div style={fieldStyle}>
            <label style={labelStyle}>Report Name *</label>
            <input name="name" required style={inputStyle} placeholder="Daily Summary" />
          </div>
          <div style={fieldStyle}>
            <label style={labelStyle}>Schedule *</label>
            <select name="schedule" required style={inputStyle}>
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
              <option value="monthly">Monthly</option>
            </select>
          </div>
          <div style={{ ...fieldStyle, gridColumn: 'span 2' }}>
            <label style={labelStyle}>Email Recipients (one per line)</label>
            <textarea
              name="email_recipients"
              style={{ ...inputStyle, minHeight: 60, resize: 'vertical' }}
              placeholder="admin@example.com&#10;ops@example.com"
            />
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 13, cursor: 'pointer' }}>
              <input type="checkbox" name="enabled" defaultChecked style={{ cursor: 'pointer' }} />
              Enabled
            </label>
          </div>
          <div style={{ display: 'flex', alignItems: 'flex-end', justifyContent: 'flex-end', gap: 8 }}>
            <button type="submit" style={btnStyle} disabled={createMut.isPending}>
              {createMut.isPending ? 'Creating...' : 'Create Report'}
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
            {reports.length === 0
              ? 'No reports configured. Click "+ Add Report" to create one.'
              : 'No reports match your search.'}
          </div>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={thStyle}>Name</th>
                <th style={thStyle}>Schedule</th>
                <th style={thStyle}>Enabled</th>
                <th style={thStyle}>Recipients</th>
                <th style={thStyle}>Last Generated</th>
                <th style={{ ...thStyle, width: 220 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map(r => (
                <tr key={r.id}>
                  {editState && editState.id === r.id ? (
                    <>
                      <td style={tdStyle}>
                        <form id={`edit-${r.id}`} onSubmit={handleUpdate} style={{ display: 'contents' }}>
                          <input name="name" defaultValue={editState.name} required style={{ ...inputStyle, width: '100%' }} />
                        </form>
                      </td>
                      <td style={tdStyle}>
                        <select name="schedule" defaultValue={editState.schedule} form={`edit-${r.id}`} style={inputStyle}>
                          <option value="daily">Daily</option>
                          <option value="weekly">Weekly</option>
                          <option value="monthly">Monthly</option>
                        </select>
                      </td>
                      <td style={tdStyle}>
                        <input
                          type="checkbox"
                          checked={r.enabled}
                          onChange={() => updateMut.mutate({ id: r.id, enabled: !r.enabled })}
                          style={{ cursor: 'pointer' }}
                        />
                      </td>
                      <td style={tdStyle}>
                        <textarea
                          name="email_recipients"
                          defaultValue={editState.email_recipients}
                          form={`edit-${r.id}`}
                          style={{ ...inputStyle, minHeight: 40, resize: 'vertical', fontSize: 12 }}
                        />
                      </td>
                      <td style={{ ...tdStyle, color: 'var(--muted)', fontSize: 12 }}>
                        {formatLastGenerated(r.last_generated)}
                      </td>
                      <td style={tdStyle}>
                        <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                          <button type="submit" form={`edit-${r.id}`} style={btnStyle} disabled={updateMut.isPending}>
                            {updateMut.isPending ? 'Saving...' : 'Save'}
                          </button>
                          <button type="button" style={btnStyle} onClick={() => setEditState(null)}>Cancel</button>
                        </div>
                      </td>
                    </>
                  ) : (
                    <>
                      <td style={{ ...tdStyle, fontWeight: 500 }}>{r.name}</td>
                      <td style={tdStyle}>
                        <ScheduleBadge schedule={r.schedule} />
                      </td>
                      <td style={tdStyle}>
                        <input
                          type="checkbox"
                          checked={r.enabled}
                          onChange={() => updateMut.mutate({ id: r.id, enabled: !r.enabled })}
                          style={{ cursor: 'pointer' }}
                        />
                      </td>
                      <td style={{ ...tdStyle, color: 'var(--muted)', fontSize: 12 }}>
                        {r.email_recipients.length > 0
                          ? `${r.email_recipients.length} recipient${r.email_recipients.length !== 1 ? 's' : ''}`
                          : '-'}
                      </td>
                      <td style={{ ...tdStyle, color: 'var(--muted)', fontSize: 12 }}>
                        {formatLastGenerated(r.last_generated)}
                      </td>
                      <td style={tdStyle}>
                        <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                          <button
                            onClick={() => handleGenerate(r.id)}
                            disabled={generateMut.isPending}
                            style={{
                              padding: '4px 8px',
                              borderRadius: 4,
                              border: '1px solid var(--green)',
                              background: 'transparent',
                              color: 'var(--green)',
                              cursor: 'pointer',
                              fontSize: 11,
                            }}
                            aria-label={`Generate ${r.name}`}
                          >
                            {generateMut.isPending ? '...' : 'Generate'}
                          </button>
                          <button
                            onClick={() => startEdit(r)}
                            style={{
                              padding: '4px 8px',
                              borderRadius: 4,
                              border: '1px solid var(--cyan)',
                              background: 'transparent',
                              color: 'var(--cyan)',
                              cursor: 'pointer',
                              fontSize: 11,
                            }}
                            aria-label={`Edit ${r.name}`}
                          >
                            Edit
                          </button>
                          <button
                            onClick={() => handleDelete(r)}
                            style={{
                              padding: '4px 8px',
                              borderRadius: 4,
                              border: '1px solid var(--crit)',
                              background: 'transparent',
                              color: 'var(--crit)',
                              cursor: 'pointer',
                              fontSize: 11,
                            }}
                            aria-label={`Delete ${r.name}`}
                          >
                            Del
                          </button>
                        </div>
                      </td>
                    </>
                  )}
                </tr>
              ))}
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
          <button onClick={onClose} aria-label="Close reports" style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
          </button>
          <h1 style={{ fontSize: 20, fontWeight: 700 }}>PDF Reports</h1>
        </div>
        {tableContent}
      </div>
    </div>
  )
}
