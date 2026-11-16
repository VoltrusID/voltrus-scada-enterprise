/**
 * Links of the Part 11 exports (CSV and PDF) with the filter the operator chose.
 * Pure functions, tested in tests/compliance.test.ts.
 */

export interface ExportFilter {
  /** `YYYY-MM-DD` from a date input, in the browser's time zone; empty = open. */
  from?: string
  to?: string
  username?: string
  action?: string
  search?: string
}

/** Minutes east of UTC of the browser at `at` (WIB = 420). */
export function browserOffsetMinutes(at: Date = new Date()): number {
  return -at.getTimezoneOffset()
}

/**
 * Unix seconds of a `YYYY-MM-DD` day in the browser's zone: its first second, or
 * with `endOfDay` its last. Null for an empty or malformed value.
 */
export function dayToUnix(day: string | undefined, endOfDay = false): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec((day ?? '').trim())
  if (!m) return null
  const d = endOfDay
    ? new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]), 23, 59, 59)
    : new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]), 0, 0, 0)
  const t = d.getTime()
  return Number.isNaN(t) ? null : Math.floor(t / 1000)
}

/** The query string the export endpoints read (`from`, `to`, `username`, `action`, `search`, `utc_offset_minutes`). */
export function exportQuery(f: ExportFilter, offsetMinutes = browserOffsetMinutes()): string {
  const q = new URLSearchParams()
  const from = dayToUnix(f.from)
  const to = dayToUnix(f.to, true)
  if (from !== null) q.set('from', String(from))
  if (to !== null) q.set('to', String(to))
  for (const key of ['username', 'action', 'search'] as const) {
    const v = f[key]?.trim()
    if (v) q.set(key, v)
  }
  q.set('utc_offset_minutes', String(offsetMinutes))
  return q.toString()
}

/** The signature records export, `csv` or `pdf`. */
export function signaturesExportUrl(format: 'csv' | 'pdf', f: ExportFilter): string {
  const base = format === 'pdf' ? '/api/v1/compliance/signatures/export.pdf' : '/api/v1/compliance/signatures/export'
  return `${base}?${exportQuery({ from: f.from, to: f.to, username: f.username })}`
}

/** The audit trail export: CSV from the core endpoint, PDF from the Part 11 one. */
export function auditExportUrl(format: 'csv' | 'pdf', f: ExportFilter): string {
  const base = format === 'pdf' ? '/api/v1/compliance/audit/export.pdf' : '/api/v1/audit/export'
  return `${base}?${exportQuery(f)}`
}

/** A range that ends before it starts cannot be exported. */
export function rangeProblem(f: ExportFilter): string | null {
  const from = dayToUnix(f.from)
  const to = dayToUnix(f.to, true)
  if (from !== null && to !== null && to < from) return 'The end date is before the start date.'
  return null
}
