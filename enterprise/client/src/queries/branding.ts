import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import type { BrandingSettings } from '@voltrus/core/edition/branding'
import { hasFeature, type LicenseInfo } from '@voltrus/core/edition/license'
import { enterpriseQueryKeys } from './keys'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'

/** Server branding, only while the licence includes white_label (otherwise the Voltrus default applies). */
export function useBranding(license: LicenseInfo): BrandingSettings | null {
  const enabled = hasFeature(license, 'white_label')
  const { data } = useQuery({
    queryKey: enterpriseQueryKeys.branding,
    queryFn: () => fetchJson<BrandingSettings>('/api/v1/settings/branding'),
    enabled,
    retry: false,
  })
  return enabled ? data ?? null : null
}

export function useSaveBranding() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (settings: BrandingSettings) => {
      await fetchOk('/api/v1/settings/branding', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings),
      })
      return settings
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: enterpriseQueryKeys.branding })
    },
  })
}
