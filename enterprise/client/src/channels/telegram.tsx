import { inputStyle } from '@voltrus/core/styles'
import { hasFeature } from '@voltrus/core/edition/license'
import type { NotificationChannel } from '@voltrus/core/edition/types'
import { FeatureBadge } from '../components/FeatureBadge'
import {
  EMPTY_TELEGRAM,
  SECRET_MASK,
  formatTelegram,
  parseTelegram,
  summarizeTelegram,
  validateTelegram,
} from './telegramConfig'

const labelStyle = { fontSize: 12, color: 'var(--muted)', marginBottom: 4, display: 'block' } as const
const helpStyle = { fontSize: 11, color: 'var(--muted)', marginTop: 4 } as const

function TelegramEditor({ config, onChange, locked }: { config: string; onChange: (config: string) => void; locked: boolean }) {
  const c = parseTelegram(config)
  // Kept as typed while editing; `serialize` trims and splits on save.
  const write = (next: Partial<typeof c>) => onChange(JSON.stringify({ ...c, ...next }))
  const masked = c.bot_token === SECRET_MASK

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <div>
        <label style={labelStyle}>Bot token</label>
        <input
          type="password"
          autoComplete="off"
          placeholder={masked ? 'Saved. Type to replace' : '123456789:AA...'}
          value={masked ? '' : c.bot_token}
          onChange={e => write({ bot_token: e.target.value })}
          disabled={locked}
          style={{ ...inputStyle, fontFamily: 'var(--font-mono)', fontSize: 12 }}
        />
        <p style={helpStyle}>Create a bot with @BotFather and paste its token. It is stored on this server and never shown again.</p>
      </div>
      <div>
        <label style={labelStyle}>Chat ids (one per line)</label>
        <textarea
          placeholder={'123456789\n-1001234567890\n@plant_ops'}
          value={c.chat_ids.join('\n')}
          onChange={e => write({ chat_ids: e.target.value.split('\n') })}
          disabled={locked}
          rows={3}
          style={{ ...inputStyle, minHeight: 70, resize: 'vertical', fontFamily: 'var(--font-mono)', fontSize: 12 }}
        />
        <p style={helpStyle}>A person must press Start on the bot first. Add the bot to a group to alert the group; group ids start with a minus sign.</p>
      </div>
      {locked && (
        <p style={{ fontSize: 11, color: 'var(--muted)' }}>
          <FeatureBadge tier="Integrator" /> Required for Telegram notifications
        </p>
      )}
    </div>
  )
}

export const telegramChannel: NotificationChannel = {
  type: 'telegram',
  label: 'Telegram',
  pill: { background: 'rgba(56,189,248,.15)', color: '#38bdf8' },
  available: license => hasFeature(license, 'chat_notifications'),
  lockedHint: 'Integrator',
  emptyConfig: formatTelegram(EMPTY_TELEGRAM),
  isValid: config => validateTelegram(config) === null,
  validate: validateTelegram,
  serialize: config => formatTelegram(parseTelegram(config)),
  summarize: summarizeTelegram,
  Editor: TelegramEditor,
}
