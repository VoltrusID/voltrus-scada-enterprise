import { useState, useEffect } from 'react'
import {
  useNotificationSettings,
  useCreateNotificationSetting,
  useUpdateNotificationSetting,
  useDeleteNotificationSetting,
  useTestNotification,
} from './queries'
import { inputStyle, btnStyle } from './styles'
import { useLicense } from './LicenseContext'
import { FeatureBadge } from './components/FeatureBadge'
import type { NotificationSetting } from './types'

interface SmsTarget {
  phone: string
  carrier: string
}

const CARRIER_OPTIONS = [
  { value: 'att', label: 'AT&T' },
  { value: 'verizon', label: 'Verizon' },
  { value: 'tmobile', label: 'T-Mobile' },
  { value: 'sprint', label: 'Sprint' },
  { value: 'uscellular', label: 'US Cellular' },
]

function parseConfigTargets(config: string): string[] {
  try {
    const parsed = JSON.parse(config)
    if (Array.isArray(parsed)) return parsed
    return []
  } catch {
    return []
  }
}

function parseSmsTargets(config: string): SmsTarget[] {
  try {
    const parsed = JSON.parse(config)
    if (Array.isArray(parsed)) return parsed
    return []
  } catch {
    return []
  }
}

function formatConfigTargets(targets: string[]): string {
  return JSON.stringify(targets.filter(t => t.trim() !== ''))
}

function formatSmsTargets(targets: SmsTarget[]): string {
  return JSON.stringify(targets.filter(t => t.phone.trim() !== ''))
}

export function NotificationSettingsPanel({ onClose, embedded }: { onClose: () => void; embedded?: boolean }) {
  const { data: settings = [], isLoading } = useNotificationSettings()
  const createMut = useCreateNotificationSetting()
  const updateMut = useUpdateNotificationSetting()
  const deleteMut = useDeleteNotificationSetting()
  const testMut = useTestNotification()

  const [showAdd, setShowAdd] = useState(false)
  const [editItem, setEditItem] = useState<NotificationSetting | null>(null)
  const [confirmDelete, setConfirmDelete] = useState<NotificationSetting | null>(null)
  const [testResult, setTestResult] = useState<{ id: number; success: boolean; message: string } | null>(null)

  useEffect(() => {
    if (embedded) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose, embedded])

  const thStyle: React.CSSProperties = { textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600, borderBottom: '1px solid var(--border)' }
  const tdStyle: React.CSSProperties = { padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)', verticalAlign: 'middle' }

  const handleCreate = (data: { source_key: string; channel_type: string; config: string; enabled: boolean }) => {
    createMut.mutate(data, { onSuccess: () => setShowAdd(false) })
  }

  const handleUpdate = (id: number, data: { source_key?: string; channel_type?: string; config?: string; enabled?: boolean }) => {
    updateMut.mutate({ id, ...data }, { onSuccess: () => setEditItem(null) })
  }

  const handleDelete = (id: number) => {
    deleteMut.mutate(id, { onSuccess: () => setConfirmDelete(null) })
  }

  const handleTest = (setting: NotificationSetting) => {
    testMut.mutate({ channel_type: setting.channel_type, config: setting.config }, {
      onSuccess: (res) => {
        setTestResult({ id: setting.id, success: res.success, message: res.success ? 'Test notification sent successfully' : 'Test notification failed' })
        setTimeout(() => setTestResult(null), 4000)
      },
      onError: (err) => {
        setTestResult({ id: setting.id, success: false, message: err.message })
        setTimeout(() => setTestResult(null), 4000)
      },
    })
  }

  const tableContent = (
    <>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
        <span style={{ color: 'var(--muted)', fontSize: 13 }}>{settings.length} notification rule{settings.length !== 1 ? 's' : ''}</span>
        <button style={{ ...btnStyle, borderColor: 'var(--ok)', color: 'var(--ok)' }} onClick={() => setShowAdd(true)}>+ Add Notification</button>
      </div>
      <div style={{ flex: 1, overflow: 'auto', background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10 }}>
        {isLoading ? (
          <div style={{ color: 'var(--muted)', textAlign: 'center', padding: 40 }}>Loading...</div>
        ) : settings.length === 0 ? (
          <div style={{ color: 'var(--muted)', textAlign: 'center', padding: 40 }}>No notification settings found.</div>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={thStyle}>Source Key</th>
                <th style={thStyle}>Channel</th>
                <th style={thStyle}>Targets</th>
                <th style={thStyle}>Enabled</th>
                <th style={{ ...thStyle, width: 220 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {settings.map(s => {
                const targets = parseConfigTargets(s.config)
                const testRes = testResult?.id === s.id ? testResult : null
                return (
                  <tr key={s.id}>
                    <td style={tdStyle}>
                      <span style={{ fontWeight: 600, fontFamily: 'var(--font-mono)', fontSize: 12 }}>{s.source_key || '*'}</span>
                    </td>
                    <td style={tdStyle}>
                      <span style={{
                        display: 'inline-block', padding: '2px 8px', borderRadius: 4, fontSize: 11, fontWeight: 600,
                        background: s.channel_type === 'email' ? 'rgba(168,85,247,.15)' : s.channel_type === 'sms' ? 'rgba(59,130,246,.15)' : 'rgba(34,197,94,.1)',
                        color: s.channel_type === 'email' ? '#a855f7' : s.channel_type === 'sms' ? '#3b82f6' : 'var(--ok)',
                      }}>
                        {s.channel_type}
                      </span>
                    </td>
                    <td style={tdStyle}>
                      <span style={{ fontSize: 12, color: 'var(--muted)' }}>
                        {s.channel_type === 'sms'
                          ? (() => {
                              const smsTargets = parseSmsTargets(s.config)
                              return smsTargets.length === 0 ? 'none' : smsTargets.length === 1
                                ? `${smsTargets[0].phone} (${smsTargets[0].carrier})`
                                : `${smsTargets.length} phone numbers`
                            })()
                          : targets.length === 0 ? 'none' : targets.length === 1 ? targets[0] : `${targets.length} targets`
                        }
                      </span>
                    </td>
                    <td style={tdStyle}>
                      <span style={{
                        display: 'inline-block', padding: '2px 8px', borderRadius: 4, fontSize: 11, fontWeight: 600,
                        background: s.enabled ? 'rgba(34,197,94,.1)' : 'rgba(239,68,68,.1)',
                        color: s.enabled ? 'var(--ok)' : 'var(--crit)',
                      }}>
                        {s.enabled ? 'ON' : 'OFF'}
                      </span>
                    </td>
                    <td style={tdStyle}>
                      <div style={{ display: 'flex', gap: 6, alignItems: 'center', flexWrap: 'wrap' }}>
                        <button
                          style={{ ...btnStyle, fontSize: 11, padding: '4px 10px' }}
                          onClick={() => setEditItem(s)}
                        >Edit</button>
                        <button
                          style={{ ...btnStyle, fontSize: 11, padding: '4px 10px', borderColor: 'var(--cyan)', color: 'var(--cyan)' }}
                          onClick={() => handleTest(s)}
                          disabled={testMut.isPending}
                        >Test</button>
                        <button
                          style={{ ...btnStyle, fontSize: 11, padding: '4px 10px', borderColor: 'var(--crit)', color: 'var(--crit)' }}
                          onClick={() => setConfirmDelete(s)}
                        >Delete</button>
                        {testRes && (
                          <span style={{ fontSize: 11, color: testRes.success ? 'var(--ok)' : 'var(--crit)' }}>
                            {testRes.message}
                          </span>
                        )}
                      </div>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        )}
      </div>

      {showAdd && <NotificationDialog onSave={handleCreate} onClose={() => setShowAdd(false)} error={createMut.error?.message} />}
      {editItem && <NotificationDialog setting={editItem} onSave={(data) => handleUpdate(editItem.id, data)} onClose={() => setEditItem(null)} error={updateMut.error?.message} />}
      {confirmDelete && (
        <div style={{ position: 'fixed', inset: 0, zIndex: 60, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 12, padding: 24, maxWidth: 400, width: '90%' }}>
            <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 8 }}>Delete Notification Rule</h3>
            <p style={{ fontSize: 13, color: 'var(--muted)', marginBottom: 16 }}>
              Delete notification rule for <strong style={{ color: 'var(--fg)' }}>{confirmDelete.source_key || '*'}</strong> ({confirmDelete.channel_type})? This cannot be undone.
            </p>
            {deleteMut.error && <p style={{ fontSize: 12, color: 'var(--crit)', marginBottom: 8 }}>{deleteMut.error.message}</p>}
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button style={btnStyle} onClick={() => setConfirmDelete(null)}>Cancel</button>
              <button style={{ ...btnStyle, borderColor: 'var(--crit)', color: 'var(--crit)' }} onClick={() => handleDelete(confirmDelete.id)}>Delete</button>
            </div>
          </div>
        </div>
      )}
    </>
  )

  if (embedded) return <div style={{ display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0 }}>{tableContent}</div>

  return (
    <div style={{ position: 'fixed', inset: 0, zIndex: 50, background: 'var(--bg)', display: 'flex', flexDirection: 'column', fontFamily: 'var(--font-ui)' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '16px 24px', borderBottom: '1px solid var(--border)', flexShrink: 0 }}>
        <button onClick={onClose} style={{ ...btnStyle, padding: '4px 8px' }}>&larr; Back</button>
        <h2 style={{ fontSize: 18, fontWeight: 700 }}>Notification Settings</h2>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: '16px 24px', display: 'flex', flexDirection: 'column' }}>
        {tableContent}
      </div>
    </div>
  )
}

function NotificationDialog({ setting, onSave, onClose, error }: {
  setting?: NotificationSetting
  onSave: (data: { source_key: string; channel_type: string; config: string; enabled: boolean }) => void
  onClose: () => void
  error?: string
}) {
  const isEdit = !!setting
  const license = useLicense()
  const isEngineeringNotificationsDisabled = license?.tier === 'engineering' && license.feature_flags?.email_notifications === false
  const smsEnabled = license?.feature_flags?.sms_notifications === true
  const [sourceKey, setSourceKey] = useState(setting?.source_key || '')
  const [channelType, setChannelType] = useState<'email' | 'webhook' | 'push' | 'sms'>(setting?.channel_type || 'email')
  const [targetsText, setTargetsText] = useState(
    setting?.channel_type === 'sms' ? '' : parseConfigTargets(setting?.config || '[]').join('\n')
  )
  const [smsTargets, setSmsTargets] = useState<SmsTarget[]>(
    setting?.channel_type === 'sms' ? parseSmsTargets(setting?.config || '[]') : [{ phone: '', carrier: 'att' }]
  )
  const [enabled, setEnabled] = useState(setting?.enabled ?? true)

  const handleSave = () => {
    if (channelType === 'sms') {
      onSave({
        source_key: sourceKey,
        channel_type: channelType,
        config: formatSmsTargets(smsTargets),
        enabled,
      })
    } else {
      const targets = targetsText.split('\n').map(t => t.trim()).filter(Boolean)
      onSave({
        source_key: sourceKey,
        channel_type: channelType,
        config: formatConfigTargets(targets),
        enabled,
      })
    }
  }

  const addSmsTarget = () => {
    setSmsTargets([...smsTargets, { phone: '', carrier: 'att' }])
  }

  const removeSmsTarget = (index: number) => {
    setSmsTargets(smsTargets.filter((_, i) => i !== index))
  }

  const updateSmsTarget = (index: number, field: keyof SmsTarget, value: string) => {
    const updated = [...smsTargets]
    updated[index] = { ...updated[index], [field]: value }
    setSmsTargets(updated)
  }

  const canSave = channelType === 'sms'
    ? smsTargets.some(t => t.phone.trim() !== '')
    : targetsText.trim() !== ''

  return (
    <div style={{ position: 'fixed', inset: 0, zIndex: 60, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 12, padding: 24, maxWidth: 480, width: '90%' }}>
        <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 16 }}>{isEdit ? 'Edit' : 'Add'} Notification Rule</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <div>
            <label style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 4, display: 'block' }}>Source Key (leave empty for all)</label>
            <input
              placeholder="e.g. solar_voltage"
              value={sourceKey}
              onChange={e => setSourceKey(e.target.value)}
              style={inputStyle}
              autoFocus
            />
          </div>
          <div>
            <label style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 4, display: 'flex', alignItems: 'center', gap: 6 }}>
              Channel Type
              {isEngineeringNotificationsDisabled && <FeatureBadge tier="Starter" />}
            </label>
            <select value={channelType} onChange={e => setChannelType(e.target.value as 'email' | 'webhook' | 'push' | 'sms')} style={inputStyle}>
              <option value="email">Email</option>
              <option value="webhook">Webhook</option>
              <option value="push">Push Notification</option>
              {smsEnabled ? (
                <option value="sms">SMS</option>
              ) : (
                <option value="sms" disabled style={{ color: 'var(--muted)' }}>SMS (Professional)</option>
              )}
            </select>
            {!smsEnabled && channelType === 'sms' && (
              <p style={{ fontSize: 11, color: 'var(--muted)', marginTop: 4 }}>
                SMS notifications require a Professional tier license.
              </p>
            )}
          </div>
          {channelType === 'sms' ? (
            <div>
              <label style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 4, display: 'block' }}>
                SMS Recipients
              </label>
              {smsTargets.map((target, idx) => (
                <div key={idx} style={{ display: 'flex', gap: 6, marginBottom: 6, alignItems: 'center' }}>
                  <input
                    placeholder="5551234567"
                    value={target.phone}
                    onChange={e => updateSmsTarget(idx, 'phone', e.target.value)}
                    style={{ ...inputStyle, flex: 1, fontFamily: 'var(--font-mono)', fontSize: 12 }}
                  />
                  <select
                    value={target.carrier}
                    onChange={e => updateSmsTarget(idx, 'carrier', e.target.value)}
                    style={{ ...inputStyle, width: 130, fontSize: 12 }}
                  >
                    {CARRIER_OPTIONS.map(c => (
                      <option key={c.value} value={c.value}>{c.label}</option>
                    ))}
                  </select>
                  {smsTargets.length > 1 && (
                    <button
                      type="button"
                      onClick={() => removeSmsTarget(idx)}
                      style={{ ...btnStyle, fontSize: 11, padding: '4px 8px', borderColor: 'var(--crit)', color: 'var(--crit)', flexShrink: 0 }}
                    >
                      X
                    </button>
                  )}
                </div>
              ))}
              <button
                type="button"
                onClick={addSmsTarget}
                style={{ ...btnStyle, fontSize: 11, padding: '4px 10px', borderColor: 'var(--cyan)', color: 'var(--cyan)', marginTop: 4 }}
              >
                + Add Number
              </button>
              {!smsEnabled && (
                <p style={{ fontSize: 11, color: 'var(--muted)', marginTop: 8 }}>
                  <FeatureBadge tier="Professional" /> Required for SMS notifications
                </p>
              )}
            </div>
          ) : (
            <div>
              <label style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 4, display: 'block' }}>
                Targets ({channelType === 'email' ? 'one email per line' : channelType === 'webhook' ? 'one URL per line' : 'one username per line'})
              </label>
              <textarea
                placeholder={channelType === 'email' ? 'user@example.com\nadmin@example.com' : channelType === 'webhook' ? 'https://hooks.example.com/notify' : 'admin\noperator'}
                value={targetsText}
                onChange={e => setTargetsText(e.target.value)}
                style={{ ...inputStyle, minHeight: 80, resize: 'vertical', fontFamily: 'var(--font-mono)', fontSize: 12 }}
                rows={3}
              />
            </div>
          )}
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <label style={{ fontSize: 13, color: 'var(--fg)' }}>Enabled</label>
            <button
              type="button"
              onClick={() => setEnabled(!enabled)}
              style={{
                width: 40, height: 22, borderRadius: 11, border: 'none', cursor: 'pointer',
                background: enabled ? 'var(--ok)' : 'var(--border)',
                position: 'relative', transition: 'background 0.2s',
              }}
            >
              <span style={{
                position: 'absolute', top: 2,
                left: enabled ? 20 : 2,
                width: 18, height: 18, borderRadius: '50%', background: '#fff',
                transition: 'left 0.2s',
              }} />
            </button>
          </div>
          {error && <p style={{ fontSize: 12, color: 'var(--crit)' }}>{error}</p>}
          <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
            <button style={btnStyle} onClick={onClose}>Cancel</button>
            <button
              style={{ ...btnStyle, borderColor: 'var(--ok)', color: 'var(--ok)' }}
              onClick={handleSave}
              disabled={!canSave}
            >{isEdit ? 'Save' : 'Create'}</button>
          </div>
        </div>
      </div>
    </div>
  )
}
