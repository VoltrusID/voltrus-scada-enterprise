import assert from 'node:assert/strict'
import { test } from 'node:test'
import {
  SECRET_MASK,
  formatTelegram,
  parseTelegram,
  summarizeTelegram,
  validateTelegram,
} from '../src/channels/telegramConfig.ts'
import {
  EMPTY_WHATSAPP,
  formatWhatsApp,
  parseWhatsApp,
  summarizeWhatsApp,
  validateWhatsApp,
} from '../src/channels/whatsappConfig.ts'

const TOKEN = '123456789:AAEhBP0av28mG8Wv9oQxkXq1Zt2yVbcD3_4'

test('telegram config tolerates garbage and round-trips', () => {
  assert.deepEqual(parseTelegram('not json'), { bot_token: '', chat_ids: [] })
  assert.deepEqual(parseTelegram('["1"]'), { bot_token: '', chat_ids: [] })
  const stored = JSON.stringify({ bot_token: TOKEN, chat_ids: ['1', '-100'] })
  assert.equal(formatTelegram(parseTelegram(stored)), stored)
})

test('telegram save splits and trims chat ids typed in the editor', () => {
  const typed = JSON.stringify({ bot_token: ` ${TOKEN} `, chat_ids: ['12, 34', '', ' @plant_ops '] })
  assert.deepEqual(JSON.parse(formatTelegram(parseTelegram(typed))), { bot_token: TOKEN, chat_ids: ['12', '34', '@plant_ops'] })
})

test('telegram validation names the problem and accepts a masked token', () => {
  const cfg = (bot_token: string, chat_ids: string[]) => JSON.stringify({ bot_token, chat_ids })
  assert.match(validateTelegram(cfg('', ['1']))!, /bot token/)
  assert.match(validateTelegram(cfg('abc', ['1']))!, /BotFather/)
  assert.match(validateTelegram(cfg(TOKEN, []))!, /at least one/)
  assert.match(validateTelegram(cfg(TOKEN, ['bob']))!, /"bob" is not a chat id/)
  assert.equal(validateTelegram(cfg(TOKEN, ['123', '-1001', '@plant_ops'])), null)
  assert.equal(validateTelegram(cfg(SECRET_MASK, ['123'])), null)
})

test('telegram summary', () => {
  assert.equal(summarizeTelegram('{}'), 'none')
  assert.equal(summarizeTelegram(JSON.stringify({ bot_token: TOKEN, chat_ids: ['5'] })), 'chat 5')
  assert.equal(summarizeTelegram(JSON.stringify({ bot_token: TOKEN, chat_ids: ['5', '6'] })), '2 chats')
})

const wa = (patch: object = {}) =>
  JSON.stringify({
    ...EMPTY_WHATSAPP,
    access_token: 'EAAG',
    phone_number_id: '106540352242922',
    recipients: ['+6281234567890'],
    template_name: 'alarm_notice',
    ...patch,
  })

test('whatsapp validation mirrors the server rules', () => {
  assert.equal(validateWhatsApp(wa()), null)
  assert.equal(validateWhatsApp(wa({ access_token: SECRET_MASK })), null)
  assert.match(validateWhatsApp(wa({ access_token: '' }))!, /access token/)
  assert.match(validateWhatsApp(wa({ phone_number_id: '+62812' }))!, /phone number id/)
  assert.match(validateWhatsApp(wa({ recipients: [] }))!, /at least one/)
  assert.match(validateWhatsApp(wa({ recipients: ['081234'] }))!, /international/)
  assert.match(validateWhatsApp(wa({ template_name: 'Alarm Notice' }))!, /template name/)
  assert.match(validateWhatsApp(wa({ language: 'indonesian' }))!, /language/)
  assert.match(validateWhatsApp(wa({ parameters: ['severity', 'colour'] }))!, /Unknown template parameter "colour"/)
  assert.equal(validateWhatsApp(wa({ parameters: [] })), null)
})

test('whatsapp save normalises numbers typed with spaces and dashes', () => {
  const typed = wa({ recipients: ['+62 812-3456-7890\n+1 415 555 0100'] })
  assert.deepEqual(parseWhatsApp(formatWhatsApp(parseWhatsApp(typed))).recipients, ['+6281234567890', '+14155550100'])
})

test('whatsapp summary names the template', () => {
  assert.equal(summarizeWhatsApp(wa()), '+6281234567890 (alarm_notice)')
  assert.equal(summarizeWhatsApp(wa({ recipients: ['+1', '+2'] })), '2 numbers (alarm_notice)')
  assert.equal(summarizeWhatsApp('[]'), 'none')
})
