export interface SmsTarget {
  phone: string
  carrier: string
}

export function parseSmsTargets(config: string): SmsTarget[] {
  try {
    const parsed = JSON.parse(config)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

/** Persisted form: blank phone rows are dropped. */
export function formatSmsTargets(targets: SmsTarget[]): string {
  return JSON.stringify(targets.filter(t => t.phone.trim() !== ''))
}

export function summarizeSmsTargets(config: string): string {
  const targets = parseSmsTargets(config).filter(t => t.phone.trim() !== '')
  if (targets.length === 0) return 'none'
  if (targets.length === 1) return `${targets[0].phone} (${targets[0].carrier})`
  return `${targets.length} phone numbers`
}
