import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { enterpriseQueryKeys } from './keys'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'

import type { ComplianceConfig } from '../panels/complianceModel'
export type { ComplianceConfig }

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
    mutationFn: async (config: Partial<ComplianceConfig>): Promise<{ ok: boolean; config: ComplianceConfig }> => {
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

/** What the signer enters. `password` is their own, checked again by the server. */
export interface SignatureInput {
  resource_type: string
  resource_id: string
  reason: string
  meaning: string
  password: string
}

export function useElectronicSignatures(params: { resource_type?: string; limit?: number } = {}, enabled = true) {
  const sp = new URLSearchParams()
  if (params.resource_type) sp.set('resource_type', params.resource_type)
  if (params.limit) sp.set('limit', String(params.limit))
  const qs = sp.toString()
  return useQuery({
    queryKey: [...enterpriseQueryKeys.complianceSignatures, params],
    // The server answers a bare array (older builds were documented as { signatures }).
    queryFn: async () => {
      const data = await fetchJson<ElectronicSignature[] | { signatures: ElectronicSignature[] }>(`/api/v1/compliance/signatures${qs ? '?' + qs : ''}`)
      return Array.isArray(data) ? data : (data?.signatures ?? [])
    },
    enabled,
  })
}

export interface SignerName {
  username: string
  role: string
  /** Printed with this person's signatures; empty when none is on file. */
  full_name: string
}

export function useSignerNames() {
  return useQuery({
    queryKey: enterpriseQueryKeys.signerNames,
    queryFn: () => fetchJson<SignerName[]>('/api/v1/compliance/signer-names'),
  })
}

export function useSaveSignerName() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ username, full_name }: { username: string; full_name: string }) => {
      await fetchOk(`/api/v1/compliance/signer-names/${encodeURIComponent(username)}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ full_name }),
      })
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: enterpriseQueryKeys.signerNames }),
  })
}

export function useCreateElectronicSignature() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (sig: SignatureInput) => {
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
