export interface WhatsAppConfig {
  access_token: string
  phone_number_id: string
  recipients: string[]
  template_name: string
  language: string
  /** Alarm field for each template placeholder, in order ({{1}}, {{2}}...). */
  parameters: string[]
}

export const TEMPLATE_FIELDS = [
  { value: 'severity', label: 'Severity' },
  { value: 'source', label: 'Source' },
  { value: 'value', label: 'Value' },
  { value: 'message', label: 'Message' },
  { value: 'time', label: 'Time' },
] as const

const E164 = /^\+[1-9]\d{6,14}$/
const TEMPLATE_NAME = /^[a-z0-9_]{1,512}$/
const LANGUAGE = /^[a-z]{2,3}(_[A-Z]{2})?$/
const PHONE_NUMBER_ID = /^\d{5,32}$/
export const MAX_PARAMETERS = 10

export const EMPTY_WHATSAPP: WhatsAppConfig = {
  access_token: '',
  phone_number_id: '',
  recipients: [],
  template_name: '',
  language: 'id',
  parameters: ['severity', 'source', 'value', 'message', 'time'],
}

const strings = (v: unknown): string[] => (Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : [])

export function parseWhatsApp(config: string): WhatsAppConfig {
  try {
    const v = JSON.parse(config)
    if (v && typeof v === 'object' && !Array.isArray(v)) {
      return {
        access_token: typeof v.access_token === 'string' ? v.access_token : '',
        phone_number_id: typeof v.phone_number_id === 'string' ? v.phone_number_id : '',
        recipients: strings(v.recipients),
        template_name: typeof v.template_name === 'string' ? v.template_name : '',
        language: typeof v.language === 'string' ? v.language : '',
        parameters: strings(v.parameters),
      }
    }
  } catch {
    // fall through
  }
  return { ...EMPTY_WHATSAPP, recipients: [], parameters: [...EMPTY_WHATSAPP.parameters] }
}

/** One number per entry, spaces and dashes removed, lines and commas split. */
export function parseRecipients(text: string): string[] {
  return text.split(/[\n,]/).map(t => t.replace(/[\s-]/g, '')).filter(Boolean)
}

/** The config as persisted. */
export function formatWhatsApp(c: WhatsAppConfig): string {
  return JSON.stringify({
    access_token: c.access_token.trim(),
    phone_number_id: c.phone_number_id.trim(),
    recipients: c.recipients.flatMap(parseRecipients),
    template_name: c.template_name.trim(),
    language: c.language.trim(),
    parameters: c.parameters,
  })
}

/** Why the config cannot be saved, or null. Mirrors the server's checks. */
export function validateWhatsApp(config: string): string | null {
  const c = parseWhatsApp(config)
  if (!c.access_token.trim()) return 'Enter the access token of the system user.'
  if (!PHONE_NUMBER_ID.test(c.phone_number_id.trim())) return 'The phone number id is the numeric id shown in the Meta app dashboard (not the phone number).'
  const recipients = c.recipients.flatMap(parseRecipients)
  if (recipients.length === 0) return 'Add at least one recipient.'
  const bad = recipients.find(r => !E164.test(r))
  if (bad) return `"${bad}" is not an international number. Use + and the country code, for example +6281234567890.`
  if (!TEMPLATE_NAME.test(c.template_name.trim())) return 'The template name is lowercase letters, digits and underscores, as approved in Meta Business Manager.'
  if (!LANGUAGE.test(c.language.trim())) return 'The language is a code such as id or en_US.'
  if (c.parameters.length > MAX_PARAMETERS) return `A template can bind at most ${MAX_PARAMETERS} parameters.`
  const unknown = c.parameters.find(p => !TEMPLATE_FIELDS.some(f => f.value === p))
  if (unknown) return `Unknown template parameter "${unknown}".`
  return null
}

export function summarizeWhatsApp(config: string): string {
  const c = parseWhatsApp(config)
  const recipients = c.recipients.flatMap(parseRecipients)
  if (recipients.length === 0) return 'none'
  const who = recipients.length === 1 ? recipients[0] : `${recipients.length} numbers`
  return c.template_name ? `${who} (${c.template_name})` : who
}
