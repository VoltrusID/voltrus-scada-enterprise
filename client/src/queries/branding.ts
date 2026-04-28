import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import type { BrandingSettings } from '../BrandingDialog'
import { queryKeys } from './keys'
import { fetchJson, fetchOk } from './fetch'

export function useBranding() {
  return useQuery({
    queryKey: queryKeys.branding,
    queryFn: () => fetchJson<BrandingSettings>('/api/settings/branding'),
  })
}

export function useSaveBranding() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (settings: BrandingSettings) => {
      await fetchOk('/api/settings/branding', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings),
      })
      return settings
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.branding })
    },
  })
}
