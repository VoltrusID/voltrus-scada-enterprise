import { useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import type { BrandingSettings } from '@voltrus/core/edition/branding'
import { hasFeature, type LicenseInfo } from '@voltrus/core/edition/license'
import { enterpriseQueryKeys } from './keys'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'
import { themeCss, type Theme } from '../panels/themeModel'

/** Core branding plus the theme the white-label licence adds. */
export type EnterpriseBranding = BrandingSettings & {
  accent?: string
  theme_level?: 'none' | 'basic' | 'full'
  theme?: Theme
  font_url?: string | null
  font_name?: string
  fonts?: { id: string; label: string }[]
}

/** The <style> element that carries the brand theme. */
const STYLE_ID = 'voltrus-brand-theme'

/** Put `css` into the page (after core's stylesheet, whose rules it overrides), or remove it. */
export function applyThemeCss(css: string) {
  let el = document.getElementById(STYLE_ID) as HTMLStyleElement | null
  if (!css) {
    el?.remove()
    return
  }
  if (!el) {
    el = document.createElement('style')
    el.id = STYLE_ID
    document.head.appendChild(el)
  }
  if (el.textContent !== css) el.textContent = css
}

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
  const css = themeCss(branding)

  // Called from the app root, so the login page gets the theme too.
  useEffect(() => { applyThemeCss(css) }, [css])

  return branding
}

async function put(path: string, body: unknown) {
  await fetchOk(path, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

export function useSaveBranding() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (settings: Partial<EnterpriseBranding>) => {
      await put('/api/v1/settings/branding', settings)
      return settings
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: enterpriseQueryKeys.branding })
    },
  })
}

export function useUploadBrandFont() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (font: { name: string; data: string }) => put('/api/v1/settings/branding/font', font),
    onSuccess: () => qc.invalidateQueries({ queryKey: enterpriseQueryKeys.branding }),
  })
}

export function useDeleteBrandFont() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: () => fetchOk('/api/v1/settings/branding/font', { method: 'DELETE' }),
    onSuccess: () => qc.invalidateQueries({ queryKey: enterpriseQueryKeys.branding }),
  })
}
