import { useState, useEffect } from 'react'
import { useComplianceConfig, useUpdateComplianceConfig, useElectronicSignatures, useCreateElectronicSignature } from './queries'
import type { ComplianceConfig, ElectronicSignature } from './queries'
import { btnStyle, inputStyle, labelStyle, labelTextStyle } from './styles'

const TABS = ['Policy', 'E-Signatures'] as const
const MEANING_OPTIONS = ['Created', 'Approved', 'Reviewed', 'Verified'] as const

function formatDatetime(ts: number): string {
  return new Date(ts * 1000).toLocaleString('en-US', {
    hour12: false,
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

export function CompliancePanel({ onClose, embedded }: { onClose: () => void; embedded?: boolean }) {
  const [tab, setTab] = useState<string>('Policy')

  useEffect(() => {
    if (embedded) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose, embedded])

  const content = (
    <>
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '16px 24px', borderBottom: '1px solid var(--border)', flexShrink: 0,
      }}>
        <h2 style={{ fontSize: 18, fontWeight: 700 }}>FDA 21 CFR Part 11 Compliance</h2>
        {!embedded && <button onClick={onClose} style={{ ...btnStyle, padding: '4px 10px' }} aria-label="Close compliance panel">&times;</button>}
      </div>

      <div style={{ display: 'flex', borderBottom: '1px solid var(--border)', flexShrink: 0 }}>
        {TABS.map(t => (
          <button key={t} onClick={() => setTab(t)} style={{
            padding: '8px 20px', fontSize: 12, fontWeight: 600, cursor: 'pointer',
            border: 'none', background: 'transparent',
            borderBottom: tab === t ? '2px solid var(--cyan)' : '2px solid transparent',
            color: tab === t ? 'var(--cyan)' : 'var(--muted)',
            textTransform: 'uppercase', letterSpacing: '.04em',
          }}>{t}</button>
        ))}
      </div>

      <div style={{ flex: 1, overflow: 'auto', padding: '20px 24px' }}>
        {tab === 'Policy' && <PolicyTab />}
        {tab === 'E-Signatures' && <ESignaturesTab />}
      </div>
    </>
  )

  if (embedded) {
    return <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>{content}</div>
  }

  return (
    <div style={{
      position: 'fixed', inset: 0, zIndex: 50,
      background: 'rgba(0,0,0,.6)',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      fontFamily: 'var(--font-ui)',
    }}>
      <div style={{
        background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 12,
        width: '90%', maxWidth: 860, maxHeight: '90vh',
        display: 'flex', flexDirection: 'column', overflow: 'hidden',
      }}>
        {content}
      </div>
    </div>
  )
}

/* ── Policy Tab ─────────────────────────────────────────────────── */

function PolicyTab() {
  const { data: config, isLoading } = useComplianceConfig()
  const updateMut = useUpdateComplianceConfig()

  const [form, setForm] = useState<ComplianceConfig>({
    auto_logout_minutes: 30,
    password_min_length: 8,
    password_require_uppercase: true,
    password_require_number: true,
    password_require_special: false,
    password_rotation_days: 90,
    authority_to_change: false,
  })
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    if (config) setForm(config)
  }, [config])

  const handleSave = () => {
    updateMut.mutate(form, {
      onSuccess: () => {
        setSaved(true)
        setTimeout(() => setSaved(false), 2000)
      },
    })
  }

  const setField = <K extends keyof ComplianceConfig>(key: K, value: ComplianceConfig[K]) => {
    setForm(prev => ({ ...prev, [key]: value }))
    setSaved(false)
  }

  if (isLoading) {
    return <div style={{ color: 'var(--muted)', textAlign: 'center', padding: 40 }}>Loading...</div>
  }

  const sectionHeader: React.CSSProperties = {
    fontSize: 13, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '.05em',
    color: 'var(--fg)', marginBottom: 16, paddingLeft: 12,
    borderLeft: '4px solid var(--cyan)',
  }

  const toggleStyle = (on: boolean): React.CSSProperties => ({
    width: 40, height: 22, borderRadius: 11,
    background: on ? 'var(--cyan)' : 'var(--border)',
    position: 'relative', cursor: 'pointer', border: 'none',
    transition: 'background .15s', flexShrink: 0,
  })

  const toggleKnob: React.CSSProperties = {
    width: 16, height: 16, borderRadius: '50%', background: '#fff',
    position: 'absolute', top: 3, transition: 'left .15s',
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 32, maxWidth: 520 }}>
      {/* Session */}
      <section>
        <h3 style={sectionHeader}>Session</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16, paddingLeft: 12 }}>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Auto-logout (minutes)</span>
            <input
              type="number"
              min={1}
              value={form.auto_logout_minutes}
              onChange={e => setField('auto_logout_minutes', Math.max(1, Number(e.target.value)))}
              style={inputStyle}
            />
          </label>
        </div>
      </section>

      {/* Password Policy */}
      <section>
        <h3 style={sectionHeader}>Password Policy</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16, paddingLeft: 12 }}>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Minimum Length</span>
            <input
              type="number"
              min={4}
              max={128}
              value={form.password_min_length}
              onChange={e => setField('password_min_length', Math.max(4, Number(e.target.value)))}
              style={inputStyle}
            />
          </label>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {([
              ['password_require_uppercase', 'Require Uppercase Letter'],
              ['password_require_number', 'Require Number'],
              ['password_require_special', 'Require Special Character'],
            ] as const).map(([key, label]) => (
              <div key={key} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <span style={{ fontSize: 13, color: 'var(--fg)' }}>{label}</span>
                <button
                  type="button"
                  style={toggleStyle(form[key])}
                  onClick={() => setField(key, !form[key])}
                  aria-label={`${label}: ${form[key] ? 'on' : 'off'}`}
                >
                  <div style={{ ...toggleKnob, left: form[key] ? 21 : 3 }} />
                </button>
              </div>
            ))}
          </div>

          <label style={labelStyle}>
            <span style={labelTextStyle}>Password Rotation (days)</span>
            <input
              type="number"
              min={0}
              value={form.password_rotation_days}
              onChange={e => setField('password_rotation_days', Math.max(0, Number(e.target.value)))}
              style={inputStyle}
            />
          </label>
        </div>
      </section>

      {/* Authority */}
      <section>
        <h3 style={sectionHeader}>Authority</h3>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingLeft: 12 }}>
          <span style={{ fontSize: 13, color: 'var(--fg)' }}>Authority to Change</span>
          <button
            type="button"
            style={toggleStyle(form.authority_to_change)}
            onClick={() => setField('authority_to_change', !form.authority_to_change)}
            aria-label={`Authority to change: ${form.authority_to_change ? 'on' : 'off'}`}
          >
            <div style={{ ...toggleKnob, left: form.authority_to_change ? 21 : 3 }} />
          </button>
        </div>
      </section>

      {/* Actions */}
      <div style={{ display: 'flex', gap: 8, alignItems: 'center', paddingLeft: 12 }}>
        <button
          style={{ ...btnStyle, borderColor: 'var(--cyan)', color: 'var(--cyan)', fontWeight: 600 }}
          onClick={handleSave}
          disabled={updateMut.isPending}
        >
          {updateMut.isPending ? 'Saving...' : 'Save Policy'}
        </button>
        {updateMut.error && (
          <span style={{ fontSize: 12, color: 'var(--crit)' }}>{updateMut.error.message}</span>
        )}
        {saved && (
          <span style={{ fontSize: 12, color: 'var(--ok)' }}>Policy saved</span>
        )}
      </div>
    </div>
  )
}

/* ── E-Signatures Tab ───────────────────────────────────────────── */

function ESignaturesTab() {
  const { data: sigData, isLoading } = useElectronicSignatures({ limit: 100 })
  const signatures = sigData?.signatures ?? []
  const createMut = useCreateElectronicSignature()

  const [showSignForm, setShowSignForm] = useState(false)

  const thStyle: React.CSSProperties = {
    textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)',
    textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600,
    borderBottom: '1px solid var(--border)', whiteSpace: 'nowrap',
  }
  const tdStyle: React.CSSProperties = {
    padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)',
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      {/* Section header */}
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
      }}>
        <h3 style={{
          fontSize: 13, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '.05em',
          color: 'var(--fg)', paddingLeft: 12,
          borderLeft: '4px solid var(--cyan)',
        }}>
          Electronic Signatures
        </h3>
        <button
          style={{ ...btnStyle, borderColor: 'var(--cyan)', color: 'var(--cyan)' }}
          onClick={() => setShowSignForm(true)}
        >
          + Sign
        </button>
      </div>

      {/* Table */}
      <div style={{
        flex: 1, overflow: 'auto',
        background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10,
      }}>
        {isLoading ? (
          <div style={{ color: 'var(--muted)', textAlign: 'center', padding: 40 }}>Loading...</div>
        ) : signatures.length === 0 ? (
          <div style={{ color: 'var(--muted)', textAlign: 'center', padding: 40 }}>No electronic signatures recorded.</div>
        ) : (
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', minWidth: 700 }}>
              <thead>
                <tr>
                  <th style={thStyle}>Time</th>
                  <th style={thStyle}>User</th>
                  <th style={thStyle}>Action</th>
                  <th style={thStyle}>Resource Type</th>
                  <th style={thStyle}>Resource ID</th>
                  <th style={thStyle}>Reason</th>
                  <th style={thStyle}>Meaning</th>
                </tr>
              </thead>
              <tbody>
                {signatures.map(sig => (
                  <ESignatureRow key={sig.id} sig={sig} tdStyle={tdStyle} />
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Sign form dialog */}
      {showSignForm && (
        <SignDialog
          onSave={(data) => {
            createMut.mutate(data, { onSuccess: () => setShowSignForm(false) })
          }}
          onClose={() => setShowSignForm(false)}
          loading={createMut.isPending}
          error={createMut.error?.message}
        />
      )}
    </div>
  )
}

function ESignatureRow({ sig, tdStyle }: { sig: ElectronicSignature; tdStyle: React.CSSProperties }) {
  const meaningColors: Record<string, { bg: string; fg: string }> = {
    Created: { bg: 'rgba(34,197,94,.12)', fg: 'var(--ok)' },
    Approved: { bg: 'rgba(59,130,246,.12)', fg: '#3b82f6' },
    Reviewed: { bg: 'rgba(168,85,247,.12)', fg: '#a855f7' },
    Verified: { bg: 'rgba(234,179,8,.12)', fg: 'var(--warn)' },
  }
  const mc = meaningColors[sig.meaning] ?? { bg: 'rgba(161,161,170,.1)', fg: 'var(--muted)' }

  return (
    <tr>
      <td style={{ ...tdStyle, color: 'var(--muted)', fontSize: 12, whiteSpace: 'nowrap' }}>
        {formatDatetime(sig.created_at)}
      </td>
      <td style={tdStyle}>
        <span style={{ fontWeight: 600 }}>{sig.user_id}</span>
      </td>
      <td style={tdStyle}>{sig.action}</td>
      <td style={tdStyle}>{sig.resource_type}</td>
      <td style={{ ...tdStyle, fontFamily: 'var(--font-mono)', fontSize: 12 }}>
        {sig.resource_id}
      </td>
      <td style={{ ...tdStyle, maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
        {sig.reason}
      </td>
      <td style={tdStyle}>
        <span style={{
          display: 'inline-block', padding: '2px 8px', borderRadius: 4,
          fontSize: 11, fontWeight: 600, background: mc.bg, color: mc.fg,
        }}>
          {sig.meaning}
        </span>
      </td>
    </tr>
  )
}

function SignDialog({ onSave, onClose, loading, error }: {
  onSave: (d: { resource_type: string; resource_id: string; reason: string; meaning: string }) => void
  onClose: () => void
  loading?: boolean
  error?: string
}) {
  const [resourceType, setResourceType] = useState('')
  const [resourceId, setResourceId] = useState('')
  const [reason, setReason] = useState('')
  const [meaning, setMeaning] = useState<string>('Created')

  const canSubmit = resourceType.trim() && resourceId.trim() && reason.trim()

  return (
    <div style={{
      position: 'fixed', inset: 0, zIndex: 60,
      background: 'rgba(0,0,0,.6)',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
    }}>
      <div style={{
        background: 'var(--card)', border: '1px solid var(--border)',
        borderRadius: 12, padding: 24, maxWidth: 440, width: '90%',
      }}>
        <h3 style={{
          fontSize: 16, fontWeight: 700, marginBottom: 4,
        }}>
          Create Electronic Signature
        </h3>
        <p style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 20 }}>
          Record a new e-signature event per 21 CFR Part 11 requirements.
        </p>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Resource Type</span>
            <input
              placeholder="e.g. batch_record, sop"
              value={resourceType}
              onChange={e => setResourceType(e.target.value)}
              style={inputStyle}
              autoFocus
            />
          </label>

          <label style={labelStyle}>
            <span style={labelTextStyle}>Resource ID</span>
            <input
              placeholder="e.g. BR-2026-0042"
              value={resourceId}
              onChange={e => setResourceId(e.target.value)}
              style={inputStyle}
            />
          </label>

          <label style={labelStyle}>
            <span style={labelTextStyle}>Reason</span>
            <textarea
              placeholder="Describe the reason for this signature..."
              value={reason}
              onChange={e => setReason(e.target.value)}
              style={{ ...inputStyle, minHeight: 72, resize: 'vertical', fontFamily: 'inherit' }}
            />
          </label>

          <label style={labelStyle}>
            <span style={labelTextStyle}>Meaning</span>
            <select
              value={meaning}
              onChange={e => setMeaning(e.target.value)}
              style={inputStyle}
            >
              {MEANING_OPTIONS.map(m => (
                <option key={m} value={m}>{m}</option>
              ))}
            </select>
          </label>

          {error && <p style={{ fontSize: 12, color: 'var(--crit)' }}>{error}</p>}

          <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 4 }}>
            <button style={btnStyle} onClick={onClose} disabled={loading}>Cancel</button>
            <button
              style={{ ...btnStyle, borderColor: 'var(--cyan)', color: 'var(--cyan)', fontWeight: 600 }}
              onClick={() => onSave({
                resource_type: resourceType.trim(),
                resource_id: resourceId.trim(),
                reason: reason.trim(),
                meaning,
              })}
              disabled={!canSubmit || loading}
            >
              {loading ? 'Signing...' : 'Sign'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
