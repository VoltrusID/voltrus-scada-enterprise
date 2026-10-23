import { VoltrusClient } from 'voltrus';
import type { Configuration, ConfigurationParameters } from 'voltrus';
import { BrandingApi } from './apis/branding';
import { CamerasApi } from './apis/cameras';
import { ComplianceApi } from './apis/compliance';
import { HaApi } from './apis/ha';
import { HubApi } from './apis/hub';
import { LicenseApi } from './apis/license';
import { ReportsApi } from './apis/reports';
import { SecurityApi } from './apis/security';
import { SsoApi } from './apis/sso';

export * from 'voltrus';
export { BrandingApi } from './apis/branding';
export { CamerasApi } from './apis/cameras';
export { ComplianceApi } from './apis/compliance';
export { HaApi } from './apis/ha';
export { HubApi } from './apis/hub';
export { LicenseApi } from './apis/license';
export { ReportsApi } from './apis/reports';
export { SecurityApi } from './apis/security';
export { SsoApi } from './apis/sso';
export * from './types';

/**
 * The public `VoltrusClient` plus the Enterprise endpoint groups, on one
 * shared HTTP session. Every Core group (`auth`, `devices`, `alarms`, ...) works
 * unchanged.
 *
 * Against a Core-only server the Enterprise endpoints answer 404, which
 * surfaces as a `VoltrusApiError` with `status === 404`. Licence-gated
 * endpoints answer 403 when the licence tier is too low.
 */
export class VoltrusEnterpriseClient extends VoltrusClient {
  readonly license: LicenseApi;
  readonly compliance: ComplianceApi;
  readonly security: SecurityApi;
  readonly hub: HubApi;
  readonly cameras: CamerasApi;
  readonly reports: ReportsApi;
  readonly branding: BrandingApi;
  readonly ha: HaApi;
  readonly sso: SsoApi;

  constructor(configuration: Configuration | ConfigurationParameters = {}) {
    super(configuration);
    this.license = new LicenseApi(this.http);
    this.compliance = new ComplianceApi(this.http);
    this.security = new SecurityApi(this.http);
    this.hub = new HubApi(this.http);
    this.cameras = new CamerasApi(this.http);
    this.reports = new ReportsApi(this.http);
    this.branding = new BrandingApi(this.http);
    this.ha = new HaApi(this.http);
    this.sso = new SsoApi(this.http);
  }
}

export default VoltrusEnterpriseClient;
