/** Branding fields as saved by the dialog. */
export interface BrandingForm {
  title: string
  subtitle: string
  logo: string
}

/**
 * What the dialog sends to PUT /api/v1/settings/branding. A logo-only licence (white_label
 * without theme_full) must send the logo alone: the server refuses a body with a title or
 * subtitle, even an unchanged one.
 */
export function brandingPayload(form: BrandingForm, logoOnly: boolean): Partial<BrandingForm> {
  return logoOnly ? { logo: form.logo } : form
}
