/** What the server shows in place of a stored token. Sending it back keeps the stored one. */
export const SECRET_MASK = '********'

export interface TelegramConfig {
  bot_token: string
  chat_ids: string[]
}

const BOT_TOKEN = /^\d{3,}:[A-Za-z0-9_-]{20,}$/
const CHAT_ID = /^(-?\d{1,20}|@[A-Za-z][A-Za-z0-9_]{4,31})$/

export const EMPTY_TELEGRAM: TelegramConfig = { bot_token: '', chat_ids: [] }

export function parseTelegram(config: string): TelegramConfig {
  try {
    const v = JSON.parse(config)
    if (v && typeof v === 'object' && !Array.isArray(v)) {
      return {
        bot_token: typeof v.bot_token === 'string' ? v.bot_token : '',
        chat_ids: Array.isArray(v.chat_ids) ? v.chat_ids.filter((c: unknown): c is string => typeof c === 'string') : [],
      }
    }
  } catch {
    // fall through
  }
  return { ...EMPTY_TELEGRAM, chat_ids: [] }
}

export function parseChatIds(text: string): string[] {
  return text.split(/[\n,]/).map(t => t.trim()).filter(Boolean)
}

/** The config as persisted: trimmed, blank and comma separated ids split out. */
export function formatTelegram(c: TelegramConfig): string {
  return JSON.stringify({
    bot_token: c.bot_token.trim(),
    chat_ids: c.chat_ids.flatMap(parseChatIds),
  })
}

/** Why the config cannot be saved, or null. Mirrors the server's checks. */
export function validateTelegram(config: string): string | null {
  const c = parseTelegram(config)
  const token = c.bot_token.trim()
  if (!token) return 'Enter the bot token from @BotFather.'
  if (token !== SECRET_MASK && !BOT_TOKEN.test(token)) return 'The bot token should look like 123456789:AA... (copy it from @BotFather).'
  const ids = c.chat_ids.flatMap(parseChatIds)
  if (ids.length === 0) return 'Add at least one chat id.'
  const bad = ids.find(id => !CHAT_ID.test(id))
  if (bad) return `"${bad}" is not a chat id. Use a number such as 123456789 or -1001234567890, or @channelname.`
  return null
}

export function summarizeTelegram(config: string): string {
  const ids = parseTelegram(config).chat_ids.flatMap(parseChatIds)
  if (ids.length === 0) return 'none'
  if (ids.length === 1) return `chat ${ids[0]}`
  return `${ids.length} chats`
}
