import { inputStyle, btnStyle } from '@voltrus/core/styles'
import { hasFeature } from '@voltrus/core/edition/license'
import type { NotificationChannel } from '@voltrus/core/edition/types'
import { FeatureBadge } from '../components/FeatureBadge'
import { formatSmsTargets, parseSmsTargets, summarizeSmsTargets, type SmsTarget } from './smsConfig'

const CARRIER_OPTIONS = [
  { value: 'att', label: 'AT&T' },
  { value: 'verizon', label: 'Verizon' },
  { value: 'tmobile', label: 'T-Mobile' },
  { value: 'sprint', label: 'Sprint' },
  { value: 'uscellular', label: 'US Cellular' },
]

const EMPTY_TARGET: SmsTarget = { phone: '', carrier: 'att' }

function SmsEditor({ config, onChange, locked }: { config: string; onChange: (config: string) => void; locked: boolean }) {
  // Blank rows are kept while editing; `serialize` drops them on save.
  const parsed = parseSmsTargets(config)
  const targets = parsed.length > 0 ? parsed : [EMPTY_TARGET]

  const update = (next: SmsTarget[]) => onChange(JSON.stringify(next))

  return (
    <div>
      <label style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 4, display: 'block' }}>
        SMS Recipients
      </label>
      {targets.map((target, idx) => (
        <div key={idx} style={{ display: 'flex', gap: 6, marginBottom: 6, alignItems: 'center' }}>
          <input
            placeholder="5551234567"
            value={target.phone}
            onChange={e => update(targets.map((t, i) => (i === idx ? { ...t, phone: e.target.value } : t)))}
            style={{ ...inputStyle, flex: 1, fontFamily: 'var(--font-mono)', fontSize: 12 }}
          />
          <select
            value={target.carrier}
            onChange={e => update(targets.map((t, i) => (i === idx ? { ...t, carrier: e.target.value } : t)))}
            style={{ ...inputStyle, width: 130, fontSize: 12 }}
          >
            {CARRIER_OPTIONS.map(c => (
              <option key={c.value} value={c.value}>{c.label}</option>
            ))}
          </select>
          {targets.length > 1 && (
            <button
              type="button"
              onClick={() => update(targets.filter((_, i) => i !== idx))}
              style={{ ...btnStyle, fontSize: 11, padding: '4px 8px', borderColor: 'var(--crit)', color: 'var(--crit)', flexShrink: 0 }}
            >
              X
            </button>
          )}
        </div>
      ))}
      <button
        type="button"
        onClick={() => update([...targets, EMPTY_TARGET])}
        style={{ ...btnStyle, fontSize: 11, padding: '4px 10px', borderColor: 'var(--cyan)', color: 'var(--cyan)', marginTop: 4 }}
      >
        + Add Number
      </button>
      {locked && (
        <p style={{ fontSize: 11, color: 'var(--muted)', marginTop: 8 }}>
          <FeatureBadge tier="Integrator" /> Required for SMS notifications
        </p>
      )}
    </div>
  )
}

export const smsChannel: NotificationChannel = {
  type: 'sms',
  label: 'SMS',
  pill: { background: 'rgba(59,130,246,.15)', color: '#3b82f6' },
  available: license => hasFeature(license, 'sms_notifications'),
  lockedHint: 'Integrator',
  emptyConfig: formatSmsTargets([]),
  isValid: config => parseSmsTargets(config).some(t => t.phone.trim() !== ''),
  serialize: config => formatSmsTargets(parseSmsTargets(config)),
  summarize: summarizeSmsTargets,
  Editor: SmsEditor,
}
