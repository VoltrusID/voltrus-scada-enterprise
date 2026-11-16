/** Branding fields as saved by the dialog. */
export interface BrandingForm {
  title: string
  subtitle: string
  logo: string
  /** Accent colour `#rrggbb`; empty keeps the theme's own. */
  accent?: string
}

/**
 * Largest logo file the dialog accepts, the core limit (`MAX_LOGO_FILE_BYTES`, 100 KB). The
 * file is sent base64 encoded (4/3 of its size plus the `data:` prefix).
 */
export const MAX_LOGO_FILE_BYTES = 100_000

/** True for an accent the server accepts: `#rgb` or `#rrggbb`, or empty to reset it. */
export function isValidAccent(value: string): boolean {
  return value === '' || /^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.test(value)
}

/**
 * What the dialog sends to PUT /api/v1/settings/branding. A logo-only licence (white_label
 * without theme_full) must send the logo alone: the server refuses a body with a title,
 * subtitle or accent, even an unchanged one.
 */
export function brandingPayload(form: BrandingForm, logoOnly: boolean): Partial<BrandingForm> {
  if (logoOnly) return { logo: form.logo }
  return { title: form.title, subtitle: form.subtitle, logo: form.logo, accent: form.accent ?? '' }
}
