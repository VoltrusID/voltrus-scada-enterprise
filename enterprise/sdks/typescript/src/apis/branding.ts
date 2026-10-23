import type { HttpClient } from 'voltrus';
import type { Branding, UpdateBrandingRequest } from '../types';

/** White-label branding. Reading is public; writing needs an operator session and a licence. */
export class BrandingApi {
  constructor(private readonly http: HttpClient) {}

  /** Defaults ("Voltrus", theme_level "none") when the server has no white-label entitlement. */
  async get(): Promise<Branding> {
    return this.http.get('/api/v1/settings/branding');
  }

  /**
   * Integrator and Enterprise keys can set title, subtitle and logo. Retired
   * entitlements that only grant logo-only branding are rejected when they send
   * a title or subtitle.
   */
  async update(input: UpdateBrandingRequest): Promise<{ ok: boolean }> {
    return this.http.put('/api/v1/settings/branding', input);
  }
}
