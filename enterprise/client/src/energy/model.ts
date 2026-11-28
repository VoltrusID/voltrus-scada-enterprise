// Voltrus Energy (PLN bill shadow): types of /api/v1/energy and pure helpers.
// Kept free of React so node tests can import it.

export type Golongan = 'I-3' | 'I-4'
export type Band = 'LWBP' | 'WBP'
export type Quality = 'ok' | 'estimated' | 'gap'
export type MeterRole = 'pln_incomer' | 'sub'

export interface TariffValues {
  golongan: Golongan
  k_factor?: number | null
  rate_lwbp: number
  rate_wbp?: number | null
  rate_kvarh: number
  pf_limit: number
  kvarh_factor?: number | null
}

export interface Tariff extends TariffValues {
  id: number
  site_id: number
  valid_from: string
  note: string
}

export interface WbpWindow { start: string; end: string }

export interface AlertConfig {
  pf_warn: number
  pf_window_hours: number
  wbp_kw_target: number | null
  daily_summary: boolean
  lang: 'id' | 'en'
}

export interface Site {
  id: number
  name: string
  timezone: string
  wbp_windows: WbpWindow[]
  contracted_kva: number | null
  alerts: AlertConfig
}

export interface Meter {
  id: number
  site_id: number
  name: string
  device_id: string
  kwh_metric: string
  kvarh_metric: string | null
  kvar_metric: string | null
  role: MeterRole
  multiplier: number
  rollover: number | null
  max_gap_secs: number
  enabled: boolean
}

export interface NotificationKeys { pf: string; wbp_kw: string; daily: string }

export interface SiteDetail {
  site: Site
  current_tariff: Tariff | null
  tariffs: Tariff[]
  meters: Meter[]
  confirmed: boolean
  notification_keys: NotificationKeys
}

export interface SiteSummary {
  site: Site
  current_tariff: Tariff | null
  meter_count: number
  confirmed: boolean
}

export interface Bill {
  kwh_lwbp: number
  kwh_wbp: number
  kwh_total: number
  kvarh: number | null
  cos_phi: number | null
  rate_lwbp: number
  rate_wbp: number
  rate_kvarh: number
  k_factor: number | null
  kvarh_factor: number
  kvarh_allowed: number
  kvarh_excess: number | null
  cost_lwbp: number
  cost_wbp: number
  wbp_premium: number
  cost_kvarh: number | null
  total: number
}

export interface PlnFigures {
  kwh_lwbp: number
  kwh_wbp: number
  kvarh: number
  kvarh_excess?: number | null
  cost_lwbp?: number | null
  cost_wbp?: number | null
  cost_kvarh?: number | null
  total?: number | null
  note?: string
}

export interface Diff {
  line: string
  computed: number | null
  pln: number
  difference: number | null
  difference_pct: number | null
}

export interface HourCounts { lwbp: number; wbp: number }
export interface QualityCounts { ok: number; estimated: number; gap: number }

export interface MonthReport {
  site_id: number
  site_name: string
  month: string
  timezone: string
  complete: boolean
  hours_in_month: HourCounts
  hours_with_data: HourCounts
  quality: QualityCounts
  incomer_meters: number[]
  tariff: Tariff | null
  month_to_date: Bill | null
  projected: Bill | null
  pln: PlnFigures | null
  reconciliation: { lines: Diff[]; within_tolerance: boolean } | null
  confirmed: boolean
  excluded: string[]
  notification_keys?: NotificationKeys
}

export interface DayRow {
  date: string
  kwh_lwbp: number
  kwh_wbp: number
  kvarh: number | null
  cos_phi: number | null
  wbp_premium: number
  kvarh_charge: number | null
  cost_total: number
  quality: QualityCounts
}

/** PLN 2025 defaults, mirrored from the server for the form before /defaults loads. */
export const DEFAULT_TARIFFS: Record<Golongan, TariffValues> = {
  'I-3': { golongan: 'I-3', k_factor: 1.4, rate_lwbp: 1035.78, rate_wbp: null, rate_kvarh: 1114.74, pf_limit: 0.85, kvarh_factor: null },
  'I-4': { golongan: 'I-4', k_factor: null, rate_lwbp: 996.74, rate_wbp: 996.74, rate_kvarh: 996.74, pf_limit: 0.85, kvarh_factor: null },
}

export const TIMEZONES = ['Asia/Jakarta', 'Asia/Makassar', 'Asia/Jayapura'] as const

/** `Rp 86.949.720` (id) or `Rp 86,949,720` (en). */
export function formatRupiah(v: number | null | undefined, lang: string): string {
  if (v == null || !Number.isFinite(v)) return '-'
  const n = Math.round(v)
  const digits = Math.abs(n).toString()
  const sep = lang.startsWith('id') ? '.' : ','
  let out = ''
  for (let i = 0; i < digits.length; i++) {
    if (i > 0 && (digits.length - i) % 3 === 0) out += sep
    out += digits[i]
  }
  return `Rp ${n < 0 ? '-' : ''}${out}`
}

/** A quantity with thousands separators and up to `decimals` decimals. */
export function formatNumber(v: number | null | undefined, lang: string, decimals = 0): string {
  if (v == null || !Number.isFinite(v)) return '-'
  return v.toLocaleString(lang.startsWith('id') ? 'id-ID' : 'en-US', { maximumFractionDigits: decimals, minimumFractionDigits: 0 })
}

export function formatPct(v: number | null | undefined, lang: string): string {
  if (v == null || !Number.isFinite(v)) return '-'
  return `${(v * 100).toLocaleString(lang.startsWith('id') ? 'id-ID' : 'en-US', { maximumFractionDigits: 1, minimumFractionDigits: 1, signDisplay: 'exceptZero' })}%`
}

/** The local `YYYY-MM` of a date (the browser's clock; the server defaults to the site's). */
export function monthOf(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

/** `tan(acos(pf))`, the default allowed kVArh per kWh. */
export function excessFactor(pfLimit: number): number {
  return Math.tan(Math.acos(pfLimit))
}

/**
 * The "to be confirmed with your PLN bill" note stays until one month
 * reconciled within 2% (spec §7).
 */
export function showUnconfirmedNote(report: Pick<MonthReport, 'confirmed'> | null | undefined, site?: Pick<SiteDetail, 'confirmed'> | null): boolean {
  return !(report?.confirmed || site?.confirmed)
}

/** Parse a form number; blank is null. */
export function numOrNull(s: string | null | undefined): number | null {
  if (s == null) return null
  const t = s.trim().replace(',', '.')
  if (t === '') return null
  const n = Number(t)
  return Number.isFinite(n) ? n : null
}

/** A data source key (`device.metric`) as the historian metric of its device. */
export function metricName(key: string, deviceId: string): string {
  const prefix = `${deviceId}.`
  return key.startsWith(prefix) ? key.slice(prefix.length) : key
}

/** Devices and their tags from the core data sources, for the meter form. */
export function deviceTags(sources: { key: string; device_id: string; name: string }[]): Map<string, { metric: string; name: string }[]> {
  const out = new Map<string, { metric: string; name: string }[]>()
  for (const s of sources) {
    if (!s.device_id) continue
    const list = out.get(s.device_id) ?? []
    list.push({ metric: metricName(s.key, s.device_id), name: s.name })
    out.set(s.device_id, list)
  }
  for (const list of out.values()) list.sort((a, b) => a.metric.localeCompare(b.metric))
  return out
}

/** Bar heights for the daily chart: one shared Rupiah axis for both series. */
export function dailyScale(days: Pick<DayRow, 'wbp_premium' | 'kvarh_charge'>[]): number {
  let max = 0
  for (const d of days) max = Math.max(max, d.wbp_premium, d.kvarh_charge ?? 0)
  if (max <= 0) return 1
  // Round the axis top up to 1, 2 or 5 × 10^n.
  const p = Math.pow(10, Math.floor(Math.log10(max)))
  for (const m of [1, 2, 5, 10]) if (m * p >= max) return m * p
  return 10 * p
}
