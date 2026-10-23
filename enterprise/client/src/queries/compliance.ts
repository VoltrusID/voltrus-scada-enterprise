import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { enterpriseQueryKeys } from './keys'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'

export interface ComplianceConfig {
  auto_logout_minutes: number
  password_min_length: number
  password_require_uppercase: boolean
  password_require_number: boolean
  password_require_special: boolean
  password_rotation_days: number
  authority_to_change: boolean
}

export interface ElectronicSignature {
  id: number
  user_id: string
  action: string
  resource_type: string
  resource_id: string
  reason: string
  meaning: string
  signature_hash: string
  created_at: number
}

export function useComplianceConfig(enabled = true) {
  return useQuery({
    queryKey: enterpriseQueryKeys.complianceConfig,
    queryFn: () => fetchJson<ComplianceConfig>('/api/v1/compliance/config'),
    enabled,
  })
}

export function useUpdateComplianceConfig() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (config: Partial<ComplianceConfig>) => {
      const res = await fetchOk('/api/v1/compliance/config', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      })
      return res.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: enterpriseQueryKeys.complianceConfig })
    },
  })
}

export function useElectronicSignatures(params: { resource_type?: string; limit?: number } = {}, enabled = true) {
  const sp = new URLSearchParams()
  if (params.resource_type) sp.set('resource_type', params.resource_type)
  if (params.limit) sp.set('limit', String(params.limit))
  const qs = sp.toString()
  return useQuery({
    queryKey: [...enterpriseQueryKeys.complianceSignatures, params],
    queryFn: () => fetchJson<{ signatures: ElectronicSignature[] }>(`/api/v1/compliance/signatures${qs ? '?' + qs : ''}`),
    enabled,
  })
}

export function useCreateElectronicSignature() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (sig: { resource_type: string; resource_id: string; reason: string; meaning: string }) => {
      const res = await fetchOk('/api/v1/compliance/sign', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(sig),
      })
      return res.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: enterpriseQueryKeys.complianceSignatures })
    },
  })
}
