import type { HttpClient } from 'voltrus';
import type { SsoProvider } from '../types';

/** Single sign-on providers advertised by the server. */
export class SsoApi {
  constructor(private readonly http: HttpClient) {}

  /**
   * SSO providers from `GET /api/v1/auth/providers`: OIDC, SAML and LDAP with
   * their enabled flag. The built-in local provider is left out; use
   * `client.auth.listProviders()` for the full list. A Core-only server returns
   * an empty array.
   */
  async listProviders(): Promise<SsoProvider[]> {
    const res = await this.http.get<{ providers: SsoProvider[] }>('/api/v1/auth/providers');
    return res.providers.filter((p) => p.provider !== 'local');
  }
}
