import { inputStyle, btnStyle } from '@voltrus/core/styles'
import { hasFeature } from '@voltrus/core/edition/license'
import type { NotificationChannel } from '@voltrus/core/edition/types'
import { FeatureBadge } from '../components/FeatureBadge'
import { SECRET_MASK } from './telegramConfig'
import {
  EMPTY_WHATSAPP,
  MAX_PARAMETERS,
  TEMPLATE_FIELDS,
  formatWhatsApp,
  parseWhatsApp,
  summarizeWhatsApp,
  validateWhatsApp,
} from './whatsappConfig'

const labelStyle = { fontSize: 12, color: 'var(--muted)', marginBottom: 4, display: 'block' } as const
const helpStyle = { fontSize: 11, color: 'var(--muted)', marginTop: 4 } as const
const monoInput = { ...inputStyle, fontFamily: 'var(--font-mono)', fontSize: 12 }

function WhatsAppEditor({ config, onChange, locked }: { config: string; onChange: (config: string) => void; locked: boolean }) {
  const c = parseWhatsApp(config)
  const write = (next: Partial<typeof c>) => onChange(JSON.stringify({ ...c, ...next }))
  const masked = c.access_token === SECRET_MASK

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <div>
        <label style={labelStyle}>Access token</label>
        <input
          type="password"
          autoComplete="off"
          placeholder={masked ? 'Saved. Type to replace' : 'EAAG...'}
          value={masked ? '' : c.access_token}
          onChange={e => write({ access_token: e.target.value })}
          disabled={locked}
          style={monoInput}
        />
        <p style={helpStyle}>Permanent token of a Meta system user with whatsapp_business_messaging access. Stored on this server and never shown again.</p>
      </div>
      <div>
        <label style={labelStyle}>Phone number id</label>
        <input
          placeholder="106540352242922"
          value={c.phone_number_id}
          onChange={e => write({ phone_number_id: e.target.value })}
          disabled={locked}
          style={monoInput}
        />
        <p style={helpStyle}>The id of the business number in the Meta app dashboard (WhatsApp, API Setup), not the number itself.</p>
      </div>
      <div>
        <label style={labelStyle}>Recipients (one per line, with country code)</label>
        <textarea
          placeholder={'+6281234567890\n+14155550100'}
          value={c.recipients.join('\n')}
          onChange={e => write({ recipients: e.target.value.split('\n') })}
          disabled={locked}
          rows={3}
          style={{ ...monoInput, minHeight: 70, resize: 'vertical' }}
        />
      </div>
      <div style={{ display: 'flex', gap: 8 }}>
        <div style={{ flex: 1 }}>
          <label style={labelStyle}>Template name</label>
          <input
            placeholder="alarm_notice"
            value={c.template_name}
            onChange={e => write({ template_name: e.target.value })}
            disabled={locked}
            style={monoInput}
          />
        </div>
        <div style={{ width: 90 }}>
          <label style={labelStyle}>Language</label>
          <input
            placeholder="id"
            value={c.language}
            onChange={e => write({ language: e.target.value })}
            disabled={locked}
            style={monoInput}
          />
        </div>
      </div>
      <p style={{ ...helpStyle, marginTop: -6 }}>
        WhatsApp only lets a business start a chat with an approved message template. Create the template in Meta Business Manager, then map
        each placeholder below to an alarm field.
      </p>
      <div>
        <label style={labelStyle}>Template parameters</label>
        {c.parameters.length === 0 && <p style={helpStyle}>No parameters: the template has no placeholders.</p>}
        {c.parameters.map((p, idx) => (
          <div key={idx} style={{ display: 'flex', gap: 6, marginBottom: 6, alignItems: 'center' }}>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--muted)', width: 44 }}>{`{{${idx + 1}}}`}</span>
            <select
              value={p}
              onChange={e => write({ parameters: c.parameters.map((x, i) => (i === idx ? e.target.value : x)) })}
              disabled={locked}
              style={{ ...inputStyle, flex: 1, fontSize: 12 }}
            >
              {TEMPLATE_FIELDS.map(f => (
                <option key={f.value} value={f.value}>{f.label}</option>
              ))}
            </select>
            <button
              type="button"
              disabled={locked}
              onClick={() => write({ parameters: c.parameters.filter((_, i) => i !== idx) })}
              style={{ ...btnStyle, fontSize: 11, padding: '4px 8px', borderColor: 'var(--crit)', color: 'var(--crit)', flexShrink: 0 }}
            >X</button>
          </div>
        ))}
        <button
          type="button"
          disabled={locked || c.parameters.length >= MAX_PARAMETERS}
          onClick={() => write({ parameters: [...c.parameters, 'message'] })}
          style={{ ...btnStyle, fontSize: 11, padding: '4px 10px', borderColor: 'var(--cyan)', color: 'var(--cyan)', marginTop: 4 }}
        >+ Add Parameter</button>
      </div>
      {locked && (
        <p style={{ fontSize: 11, color: 'var(--muted)' }}>
          <FeatureBadge tier="Integrator" /> Required for WhatsApp notifications
        </p>
      )}
    </div>
  )
}

export const whatsappChannel: NotificationChannel = {
  type: 'whatsapp',
  label: 'WhatsApp',
  pill: { background: 'rgba(34,197,94,.15)', color: '#22c55e' },
  available: license => hasFeature(license, 'chat_notifications'),
  lockedHint: 'Integrator',
  emptyConfig: formatWhatsApp(EMPTY_WHATSAPP),
  isValid: config => validateWhatsApp(config) === null,
  validate: validateWhatsApp,
  serialize: config => formatWhatsApp(parseWhatsApp(config)),
  summarize: summarizeWhatsApp,
  Editor: WhatsAppEditor,
}
