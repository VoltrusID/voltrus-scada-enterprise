// Pure conversion of the server's report rows, kept free of React so tests can import it.

export interface Report {
  id: number
  name: string
  schedule: string
  enabled: boolean
  /** Device ids, or ['all'] for every device. */
  device_ids: string[]
  email_recipients: string[]
  /** ISO time of the last scheduled run, null before the first. */
  last_generated: string | null
  created_at: string
}

/** The server stores lists as JSON text and times as Unix seconds. */
function list(v: unknown): string[] {
  if (Array.isArray(v)) return v.map(String)
  if (typeof v !== 'string' || v === '') return []
  if (v === 'all') return ['all']
  try {
    const parsed = JSON.parse(v)
    return Array.isArray(parsed) ? parsed.map(String) : []
  } catch {
    return []
  }
}

function time(v: unknown): string | null {
  if (v === null || v === undefined || v === '') return null
  if (typeof v === 'number') return new Date(v * 1000).toISOString()
  return String(v)
}

export function normalizeReport(raw: any): Report {
  return {
    id: raw.id,
    name: raw.name ?? '',
    schedule: raw.schedule ?? 'daily',
    enabled: !!raw.enabled,
    device_ids: list(raw.device_ids),
    email_recipients: list(raw.email_recipients),
    last_generated: time(raw.last_generated),
    created_at: time(raw.created_at) ?? '',
  }
}
