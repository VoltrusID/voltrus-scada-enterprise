import { useQuery } from '@tanstack/react-query'
import { fetchJson } from './fetch'

export interface AggregatedMetric {
  metric: string
  min: number
  max: number
  avg: number
  stddev: number
  count: number
  first_ts: number
  last_ts: number
}

export interface ComparisonResult {
  metric: string
  current: AggregatedMetric
  previous: AggregatedMetric
  change_percent: number
}

export function useAnalyticsAggregate(
  deviceId: string | undefined,
  metric: string | undefined,
  from: number | undefined,
  to: number | undefined,
  interval: number = 3600,
) {
  const params = new URLSearchParams()
  if (deviceId) params.set('device_id', deviceId)
  if (metric) params.set('metric', metric)
  if (from) params.set('from', String(from))
  if (to) params.set('to', String(to))
  params.set('interval', String(interval))
  const qs = params.toString()
  return useQuery({
    queryKey: ['analytics', 'aggregate', deviceId, metric, from, to, interval],
    queryFn: () => fetchJson<[number, AggregatedMetric][]>(`/api/v1/analytics/aggregate?${qs}`),
    enabled: !!deviceId && !!metric && !!from && !!to,
  })
}

export function useAnalyticsCompare(
  deviceId: string | undefined,
  metric: string | undefined,
  period1From: number | undefined,
  period1To: number | undefined,
  period2From: number | undefined,
  period2To: number | undefined,
) {
  const params = new URLSearchParams()
  if (deviceId) params.set('device_id', deviceId)
  if (metric) params.set('metric', metric)
  if (period1From) params.set('period1_from', String(period1From))
  if (period1To) params.set('period1_to', String(period1To))
  if (period2From) params.set('period2_from', String(period2From))
  if (period2To) params.set('period2_to', String(period2To))
  const qs = params.toString()
  return useQuery({
    queryKey: ['analytics', 'compare', deviceId, metric, period1From, period1To, period2From, period2To],
    queryFn: () => fetchJson<ComparisonResult>(`/api/v1/analytics/compare?${qs}`),
    enabled: !!deviceId && !!metric && !!period1From && !!period1To && !!period2From && !!period2To,
  })
}

// ── SPC ──
export interface SpcPoint { index: number; value: number; timestamp: number; subgroup_values?: number[] }
export interface ControlLimits { ucl: number; cl: number; lcl: number }
export interface SpcResult { chart_type: string; data_points: SpcPoint[]; control_limits: ControlLimits; violations: number[]; in_control_percent: number }

export function useSpcChart(
  deviceId: string | undefined,
  metric: string | undefined,
  from: number | undefined,
  to: number | undefined,
  chartType: string,
  subgroupSize?: number,
  target?: number,
  lambda?: number,
  k?: number,
  h?: number,
) {
  const params = new URLSearchParams()
  if (deviceId) params.set('device_id', deviceId)
  if (metric) params.set('metric', metric)
  if (from) params.set('from', String(from))
  if (to) params.set('to', String(to))
  params.set('chart_type', chartType)
  if (subgroupSize) params.set('subgroup_size', String(subgroupSize))
  if (target !== undefined) params.set('target', String(target))
  if (lambda !== undefined) params.set('lambda', String(lambda))
  if (k !== undefined) params.set('k', String(k))
  if (h !== undefined) params.set('h', String(h))
  const qs = params.toString()
  return useQuery({
    queryKey: ['analytics', 'spc', deviceId, metric, from, to, chartType, subgroupSize, target, lambda, k, h],
    queryFn: () => fetchJson<SpcResult>(`/api/v1/analytics/spc?${qs}`),
    enabled: !!deviceId && !!metric && !!from && !!to,
  })
}

// ── Histogram ──
export interface HistogramBin { lower: number; upper: number; count: number; frequency: number }
export interface CurvePoint { x: number; y: number }
export interface DistributionStats { mean: number; stddev: number; skewness: number; kurtosis: number; count: number }
export interface CapabilityIndices { cp: number; cpk: number; pp: number; ppk: number }
export interface HistogramResult { bins: HistogramBin[]; normal_curve: CurvePoint[]; stats: DistributionStats; capability: CapabilityIndices | null }

export function useHistogram(
  deviceId: string | undefined, metric: string | undefined,
  from: number | undefined, to: number | undefined,
  bins?: number, lsl?: number, usl?: number,
) {
  const params = new URLSearchParams()
  if (deviceId) params.set('device_id', deviceId)
  if (metric) params.set('metric', metric)
  if (from) params.set('from', String(from))
  if (to) params.set('to', String(to))
  if (bins) params.set('bins', String(bins))
  if (lsl !== undefined) params.set('lsl', String(lsl))
  if (usl !== undefined) params.set('usl', String(usl))
  const qs = params.toString()
  return useQuery({
    queryKey: ['analytics', 'histogram', deviceId, metric, from, to, bins, lsl, usl],
    queryFn: () => fetchJson<HistogramResult>(`/api/v1/analytics/histogram?${qs}`),
    enabled: !!deviceId && !!metric && !!from && !!to,
  })
}

// ── Run Chart ──
export interface RunChartPoint { timestamp: number; value: number; above_median: boolean }
export interface Signal { rule: string; start_idx: number; end_idx: number; description: string }
export interface RunChartResult { points: RunChartPoint[]; median: number; signals: Signal[]; runs_count: number }

export function useRunChart(
  deviceId: string | undefined, metric: string | undefined,
  from: number | undefined, to: number | undefined,
) {
  const params = new URLSearchParams()
  if (deviceId) params.set('device_id', deviceId)
  if (metric) params.set('metric', metric)
  if (from) params.set('from', String(from))
  if (to) params.set('to', String(to))
  const qs = params.toString()
  return useQuery({
    queryKey: ['analytics', 'run-chart', deviceId, metric, from, to],
    queryFn: () => fetchJson<RunChartResult>(`/api/v1/analytics/run-chart?${qs}`),
    enabled: !!deviceId && !!metric && !!from && !!to,
  })
}

// ── Custom Metrics ──
export interface CustomMetric { id: number; name: string; description: string; formula: string; unit: string | null; tag_refs: string; validation_error: string | null; created_at: number; updated_at: number }
export interface CustomMetricValue { id: number; name: string; value: number; unit: string | null; computed_at: number; tag_values: Record<string, number> }

export function useCustomMetrics() {
  return useQuery({
    queryKey: ['analytics', 'custom-metrics'],
    queryFn: () => fetchJson<CustomMetric[]>(`/api/v1/analytics/custom-metrics`),
  })
}

export function useCustomMetricValues() {
  return useQuery({
    queryKey: ['analytics', 'custom-metrics', 'values'],
    queryFn: () => fetchJson<CustomMetricValue[]>(`/api/v1/analytics/custom-metrics/evaluate-all`),
  })
}
