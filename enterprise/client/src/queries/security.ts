import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { enterpriseQueryKeys } from './keys'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'

export interface SecurityEvent {
  id: number
  event_type: string
  severity: string
  source_ip: string | null
  user_agent: string | null
  username: string | null
  details: string | null
  created_at: number
}

export interface SecurityThresholds {
  failed_login_threshold: number
  failed_login_window_secs: number
  suspicious_401_threshold: number
  suspicious_401_window_secs: number
  rate_limit_threshold: number
  rate_limit_window_secs: number
}

export function useSecurityEvents(params: { event_type?: string; severity?: string; from?: number; to?: number; limit?: number; offset?: number } = {}, enabled = true) {
  const sp = new URLSearchParams()
  if (params.event_type) sp.set('event_type', params.event_type)
  if (params.severity) sp.set('severity', params.severity)
  if (params.from) sp.set('from', String(params.from))
  if (params.to) sp.set('to', String(params.to))
  if (params.limit) sp.set('limit', String(params.limit))
  if (params.offset) sp.set('offset', String(params.offset))
  const qs = sp.toString()
  return useQuery({
    queryKey: [...enterpriseQueryKeys.securityEvents, params],
    queryFn: () => fetchJson<{ events: SecurityEvent[]; count: number }>(`/api/v1/security/events${qs ? '?' + qs : ''}`),
    enabled,
  })
}

export function useSecurityThresholds(enabled = true) {
  return useQuery({
    queryKey: enterpriseQueryKeys.securityThresholds,
    queryFn: () => fetchJson<SecurityThresholds>('/api/v1/security/thresholds'),
    enabled,
  })
}

export function useUpdateSecurityThresholds() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (thresholds: SecurityThresholds) => {
      const res = await fetchOk('/api/v1/security/thresholds', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(thresholds),
      })
      return res.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: enterpriseQueryKeys.securityThresholds })
    },
  })
}

export function useExportSecurityCsv() {
  return useMutation({
    mutationFn: async (params: { from?: number; to?: number } = {}) => {
      const sp = new URLSearchParams()
      if (params.from) sp.set('from', String(params.from))
      if (params.to) sp.set('to', String(params.to))
      const qs = sp.toString()
      const res = await fetch(`/api/v1/security/events/export${qs ? '?' + qs : ''}`, { credentials: 'include' })
      if (!res.ok) throw new Error('Export failed')
      return res.text()
    },
  })
}
