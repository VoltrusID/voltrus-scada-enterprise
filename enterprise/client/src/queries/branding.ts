import { useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import type { BrandingSettings } from '@voltrus/core/edition/branding'
import { hasFeature, type LicenseInfo } from '@voltrus/core/edition/license'
import { enterpriseQueryKeys } from './keys'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'
import { isValidAccent } from '../panels/brandingPayload'

/** Core branding plus the accent colour the white-label theme adds. */
export type EnterpriseBranding = BrandingSettings & { accent?: string }

/** CSS variable the core theme uses for its accent (buttons, links, active tabs). */
const ACCENT_VAR = '--cyan'

/** Server branding, only while the licence includes white_label (otherwise the Voltrus default applies). */
export function useBranding(license: LicenseInfo): EnterpriseBranding | null {
  const enabled = hasFeature(license, 'white_label')
  const { data } = useQuery({
    queryKey: enterpriseQueryKeys.branding,
    queryFn: () => fetchJson<EnterpriseBranding>('/api/v1/settings/branding'),
    enabled,
    retry: false,
  })
  const branding = enabled ? data ?? null : null
  const accent = branding?.accent ?? ''

  // An inline style on <html> wins over both the dark and the light theme, so one
  // brand colour serves both. Called from the app root, so the login page gets it too.
  useEffect(() => {
    const root = document.documentElement.style
    if (accent && isValidAccent(accent)) root.setProperty(ACCENT_VAR, accent)
    else root.removeProperty(ACCENT_VAR)
  }, [accent])

  return branding
}

export function useSaveBranding() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (settings: Partial<EnterpriseBranding>) => {
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
