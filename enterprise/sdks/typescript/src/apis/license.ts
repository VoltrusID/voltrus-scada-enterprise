import type { HttpClient } from 'voltrus';
import type {
  ActivateLicenseRequest,
  LicenseActionResponse,
  LicenseRequestFile,
  LicenseStatus,
} from '../types';

/**
 * Licence status and activation. `status()` is served by Core as well; the
 * other calls need the Enterprise add-on and an admin session.
 */
export class LicenseApi {
  constructor(private readonly http: HttpClient) {}

  /** Current edition, licence status and feature flags. */
  async status(): Promise<LicenseStatus> {
    return this.http.get('/api/license/status');
  }

  /** Activate online with a licence key, or offline with an activation file. */
  async activate(input: ActivateLicenseRequest): Promise<LicenseActionResponse> {
    return this.http.post('/api/license/activate', input);
  }

  /** Offline step 1: build the request file to upload at activate.voltrus.id. */
  async request(licenseKey: string): Promise<LicenseRequestFile> {
    return this.http.post('/api/license/request', { license_key: licenseKey });
  }

  /** Renew a monthly key online. */
  async refresh(): Promise<LicenseActionResponse> {
    return this.http.post('/api/license/refresh');
  }
}
