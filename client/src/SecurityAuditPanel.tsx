import { useState, useEffect } from 'react'
import { useSecurityEvents, useSecurityThresholds, useUpdateSecurityThresholds, useExportSecurityCsv } from './queries'
import type { SecurityEvent, SecurityThresholds } from './queries'
import { btnStyle, inputStyle, labelStyle, labelTextStyle } from './styles'

const EVENT_TYPES = [
  { value: '', label: 'All Events' },
  { value: 'login_failed', label: 'Login Failed' },
  { value: 'login_locked', label: 'Account Locked' },
  { value: 'auth_401', label: 'Auth 401' },
  { value: 'config_change', label: 'Config Change' },
] as const

const SEVERITY_OPTIONS = [
  { value: '', label: 'All Severities' },
  { value: 'info', label: 'Info' },
  { value: 'warning', label: 'Warning' },
] as const

function formatTimestamp(epoch: number): string {
  return new Date(epoch * 1000).toLocaleString('en-US', {
    year: 'numeric', month: 'short', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
  })
}

function severityColor(sev: string): { color: string; bg: string } {
  switch (sev) {
    case 'critical': return { color: 'var(--crit)', bg: 'rgba(239,68,68,.15)' }
    case 'warning': return { color: 'var(--warn)', bg: 'rgba(234,179,8,.15)' }
    default: return { color: 'var(--ok)', bg: 'rgba(34,197,94,.1)' }
  }
}

function eventTypeLabel(t: string): string {
  return EVENT_TYPES.find(e => e.value === t)?.label ?? t
}

function triggerCsvDownload(csvText: string) {
  const blob = new Blob([csvText], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `security-audit-${new Date().toISOString().slice(0, 10)}.csv`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function ShieldIcon({ size = 16, color = 'var(--cyan)' }: { size?: number; color?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
    </svg>
  )
}

function DownloadIcon({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </svg>
  )
}

function ChevronIcon({ open, size = 14 }: { open: boolean; size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" style={{ transition: 'transform 0.15s ease', transform: open ? 'rotate(180deg)' : 'rotate(0deg)' }}>
      <polyline points="6 9 12 15 18 9" />
    </svg>
  )
}

interface Props {
  onClose: () => void
  embedded?: boolean
}

export function SecurityAuditPanel({ onClose, embedded }: Props) {
  useEffect(() => {
    if (embedded) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose, embedded])

  const [eventTypeFilter, setEventTypeFilter] = useState('')
  const [severityFilter, setSeverityFilter] = useState('')
  const [showThresholds, setShowThresholds] = useState(false)

  const { data: eventsData, isLoading: eventsLoading } = useSecurityEvents({
    event_type: eventTypeFilter || undefined,
    severity: severityFilter || undefined,
    limit: 200,
  })
  const events = eventsData?.events ?? []
  const eventCount = eventsData?.count ?? 0

  const { data: thresholds } = useSecurityThresholds(showThresholds)
  const updateThresholdsMut = useUpdateSecurityThresholds()
  const exportCsvMut = useExportSecurityCsv()

  const [thresholdForm, setThresholdForm] = useState<SecurityThresholds | null>(null)

  useEffect(() => {
    if (thresholds && !thresholdForm) {
      setThresholdForm({ ...thresholds })
    }
  }, [thresholds, thresholdForm])

  const handleExport = () => {
    exportCsvMut.mutate({}, { onSuccess: (csv) => triggerCsvDownload(csv) })
  }

  const handleSaveThresholds = () => {
    if (!thresholdForm) return
    updateThresholdsMut.mutate(thresholdForm)
  }

  const thStyle: React.CSSProperties = {
    textAlign: 'left', padding: '10px 14px', fontSize: 11, color: 'var(--muted)',
    textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600,
    borderBottom: '1px solid var(--border)', whiteSpace: 'nowrap',
  }
  const tdStyle: React.CSSProperties = {
    padding: '10px 14px', fontSize: 13, borderBottom: '1px solid var(--border)', verticalAlign: 'middle',
  }
  const selectStyle: React.CSSProperties = {
    ...inputStyle, cursor: 'pointer', appearance: 'auto' as const, minWidth: 140,
  }

  const content = (
    <>
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '18px 24px', borderBottom: '1px solid var(--border)', flexShrink: 0,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <ShieldIcon size={20} />
          <span style={{ fontSize: 17, fontWeight: 700 }}>Security Audit</span>
          <span style={{
            fontSize: 11, color: 'var(--muted)', background: 'var(--card)',
            border: '1px solid var(--border)', borderRadius: 6, padding: '2px 8px',
            fontFamily: 'JetBrains Mono, monospace',
          }}>
            {eventCount} event{eventCount !== 1 ? 's' : ''}
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <button onClick={handleExport} disabled={exportCsvMut.isPending} style={{
            ...btnStyle, fontSize: 12,
            opacity: exportCsvMut.isPending ? 0.6 : 1,
            cursor: exportCsvMut.isPending ? 'wait' : 'pointer',
          }} aria-label="Export security events as CSV">
            <DownloadIcon />{exportCsvMut.isPending ? 'Exporting...' : 'Export CSV'}
          </button>
          {!embedded && (
            <button onClick={onClose} aria-label="Close" style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', padding: 4, display: 'flex', alignItems: 'center', justifyContent: 'center', borderRadius: 4, transition: 'color 0.15s' }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          )}
        </div>
      </div>

      <div style={{
        display: 'flex', alignItems: 'center', gap: 12, padding: '14px 24px',
        borderBottom: '1px solid var(--border)', flexShrink: 0, flexWrap: 'wrap',
      }}>
        <label style={labelStyle}>
          <span style={labelTextStyle}>Event Type</span>
          <select value={eventTypeFilter} onChange={e => setEventTypeFilter(e.target.value)} style={selectStyle}>
            {EVENT_TYPES.map(t => <option key={t.value} value={t.value}>{t.label}</option>)}
          </select>
        </label>
        <label style={labelStyle}>
          <span style={labelTextStyle}>Severity</span>
          <select value={severityFilter} onChange={e => setSeverityFilter(e.target.value)} style={selectStyle}>
            {SEVERITY_OPTIONS.map(s => <option key={s.value} value={s.value}>{s.label}</option>)}
          </select>
        </label>
      </div>

      <div style={{ flex: 1, overflow: 'auto' }}>
        {eventsLoading ? (
          <div style={{ padding: 60, textAlign: 'center', color: 'var(--muted)', fontSize: 14 }}>Loading security events...</div>
        ) : events.length === 0 ? (
          <div style={{ padding: 60, textAlign: 'center' }}>
            <div style={{ color: 'var(--muted)', fontSize: 14, marginBottom: 8 }}>No security events found</div>
            <div style={{ color: 'var(--muted)', fontSize: 12, opacity: 0.7 }}>
              {eventTypeFilter || severityFilter ? 'Try adjusting your filters' : 'System is clear -- no events recorded'}
            </div>
          </div>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={thStyle}>Time</th>
                <th style={thStyle}>Type</th>
                <th style={thStyle}>Severity</th>
                <th style={thStyle}>Source IP</th>
                <th style={thStyle}>Username</th>
                <th style={{ ...thStyle, minWidth: 240 }}>Details</th>
              </tr>
            </thead>
            <tbody>
              {events.map(evt => {
                const sev = severityColor(evt.severity)
                return (
                  <tr key={evt.id} style={{ transition: 'background 0.1s ease' }}
                    onMouseEnter={e => { (e.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,.02)' }}
                    onMouseLeave={e => { (e.currentTarget as HTMLElement).style.background = 'transparent' }}>
                    <td style={{ ...tdStyle, fontFamily: 'JetBrains Mono, monospace', fontSize: 12, whiteSpace: 'nowrap', color: 'var(--muted)' }}>
                      {formatTimestamp(evt.created_at)}
                    </td>
                    <td style={tdStyle}>
                      <span style={{
                        display: 'inline-block', padding: '2px 8px', borderRadius: 4,
                        fontSize: 11, fontWeight: 600, background: 'rgba(168,85,247,.12)',
                        color: '#a855f7', fontFamily: 'JetBrains Mono, monospace',
                      }}>{eventTypeLabel(evt.event_type)}</span>
                    </td>
                    <td style={tdStyle}>
                      <span style={{
                        display: 'inline-block', padding: '2px 10px', borderRadius: 4,
                        fontSize: 11, fontWeight: 700, textTransform: 'uppercase',
                        background: sev.bg, color: sev.color, letterSpacing: '.03em',
                      }}>{evt.severity}</span>
                    </td>
                    <td style={{ ...tdStyle, fontFamily: 'JetBrains Mono, monospace', fontSize: 12 }}>
                      {evt.source_ip ?? <span style={{ color: 'var(--muted)', opacity: 0.5 }}>--</span>}
                    </td>
                    <td style={tdStyle}>{evt.username ?? <span style={{ color: 'var(--muted)', opacity: 0.5 }}>--</span>}</td>
                    <td style={{ ...tdStyle, fontSize: 12, color: 'var(--muted)', maxWidth: 340, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {evt.details ?? <span style={{ opacity: 0.5 }}>--</span>}
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        )}
      </div>

      <div style={{ borderTop: '1px solid var(--border)', flexShrink: 0 }}>
        <button onClick={() => setShowThresholds(v => !v)} style={{
          width: '100%', display: 'flex', alignItems: 'center', gap: 8,
          padding: '12px 24px', background: 'transparent', border: 'none',
          color: 'var(--fg)', fontSize: 13, fontWeight: 600, cursor: 'pointer', textAlign: 'left',
        }} aria-expanded={showThresholds} aria-label="Toggle threshold configuration">
          <ChevronIcon open={showThresholds} />
          <span style={{ textTransform: 'uppercase', letterSpacing: '.04em', fontSize: 11, color: 'var(--muted)' }}>Threshold Configuration</span>
        </button>
        {showThresholds && (
          <div style={{ padding: '0 24px 18px', display: 'flex', flexDirection: 'column', gap: 14 }}>
            {!thresholdForm ? (
              <div style={{ color: 'var(--muted)', fontSize: 13, padding: '8px 0' }}>Loading thresholds...</div>
            ) : (
              <>
                <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap' }}>
                  <label style={{ ...labelStyle, flex: '1 1 200px' }}>
                    <span style={labelTextStyle}>Failed Login Threshold</span>
                    <input type="number" min={1} value={thresholdForm.failed_login_threshold}
                      onChange={e => setThresholdForm({ ...thresholdForm, failed_login_threshold: Number(e.target.value) })}
                      style={{ ...inputStyle, fontFamily: 'JetBrains Mono, monospace' }} />
                  </label>
                  <label style={{ ...labelStyle, flex: '1 1 200px' }}>
                    <span style={labelTextStyle}>Suspicious 401 Threshold</span>
                    <input type="number" min={1} value={thresholdForm.suspicious_401_threshold}
                      onChange={e => setThresholdForm({ ...thresholdForm, suspicious_401_threshold: Number(e.target.value) })}
                      style={{ ...inputStyle, fontFamily: 'JetBrains Mono, monospace' }} />
                  </label>
                  <label style={{ ...labelStyle, flex: '1 1 200px' }}>
                    <span style={labelTextStyle}>Rate Limit Threshold</span>
                    <input type="number" min={1} value={thresholdForm.rate_limit_threshold}
                      onChange={e => setThresholdForm({ ...thresholdForm, rate_limit_threshold: Number(e.target.value) })}
                      style={{ ...inputStyle, fontFamily: 'JetBrains Mono, monospace' }} />
                  </label>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                  <button onClick={handleSaveThresholds} disabled={updateThresholdsMut.isPending} style={{
                    ...btnStyle, borderColor: 'var(--cyan)', color: 'var(--cyan)',
                    opacity: updateThresholdsMut.isPending ? 0.6 : 1,
                  }}>
                    {updateThresholdsMut.isPending ? 'Saving...' : 'Save Thresholds'}
                  </button>
                  {updateThresholdsMut.isSuccess && <span style={{ fontSize: 12, color: 'var(--ok)' }}>Saved</span>}
                  {updateThresholdsMut.isError && <span style={{ fontSize: 12, color: 'var(--crit)' }}>{updateThresholdsMut.error?.message || 'Failed to save'}</span>}
                </div>
              </>
            )}
          </div>
        )}
      </div>
    </>
  )

  if (embedded) {
    return <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>{content}</div>
  }

  return (
    <div role="dialog" aria-label="Security Audit Panel" style={{
      position: 'fixed', inset: 0, zIndex: 1000, background: 'rgba(0,0,0,.6)',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
    }} onClick={onClose}>
      <div style={{
        position: 'relative', width: '92%', maxWidth: 1100, maxHeight: '88vh',
        background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 12,
        display: 'flex', flexDirection: 'column', overflow: 'hidden',
      }} onClick={e => e.stopPropagation()}>
        {content}
      </div>
    </div>
  )
}
