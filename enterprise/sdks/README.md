# Enterprise SDK extensions

PRIVATE. Official client packages for the endpoints that only the Enterprise
add-on serves. They extend the public Core SDKs; they do not replace them.

| Package | Path | Import |
| --- | --- | --- |
| `@voltrus/enterprise-sdk` (npm, private) | `typescript/` | `import { VoltrusEnterpriseClient } from '@voltrus/enterprise-sdk'` |
| `voltrus-enterprise-sdk` (PyPI name, private) | `python/` | `from voltrus_enterprise import VoltrusEnterpriseClient` |

## How it fits together

* The public Core SDKs live in `core/sdks/` (source of truth: the public repo
  VoltrusID/voltrus-scada, `sdks/`). They cover Core endpoints only.
* `VoltrusEnterpriseClient` subclasses the public `VoltrusClient`. Login, API
  keys, cookies, error types and every Core call work unchanged, and the new
  groups share the same session. In TypeScript the public client exposes
  `client.http` for this.
* Both packages pin the public SDK to the exact same version (peer dependency
  in `package.json`, `voltrus==` in `pyproject.toml`). Versions move in
  lockstep: bump the public SDK, bump these two to match, in the same change.
* Do not publish these to npmjs or PyPI. `private: true` and the
  `Private :: Do Not Upload` classifier guard against accidents.
* Against a Core-only server every Enterprise endpoint answers 404
  (`VoltrusApiError.status` / `VoltrusError.status`). A too-low licence tier
  answers 403.

## What is covered

Request and response shapes come from `enterprise/src/**/handlers.rs`,
`enterprise/src/licensing/routes.rs` and `enterprise/src/routes.rs`.

| Group (TypeScript) | Python methods | Endpoints |
| --- | --- | --- |
| `license` | `get_license_status` (public client), `activate_license`, `request_license`, `refresh_license` | `/api/license/status`, `activate`, `request`, `refresh` |
| `compliance` | `get_compliance_config`, `update_compliance_config`, `sign_record`, `list_signatures` | `/api/v1/compliance/config`, `sign`, `signatures` |
| `security` | `list_security_events`, `export_security_events_csv`, `get_security_thresholds`, `update_security_thresholds` | `/api/v1/security/events`, `events/export`, `thresholds` |
| `hub` | `list_hub_sites`, `create_hub_site`, `update_hub_site`, `delete_hub_site`, `start_hub_site`, `stop_hub_site`, `get_hub_status` | `/api/v1/hub/sites`, `sites/{id}`, `start`, `stop`, `/api/v1/hub/status` |
| `cameras` | `list_cameras`, `create_camera`, `update_camera`, `delete_camera`, `get_camera_stream`, `trigger_snapshot`, `list_snapshots`, `delete_snapshot` | `/api/v1/cameras...` |
| `reports` | `list_reports`, `create_report`, `update_report`, `delete_report`, `generate_report` | `/api/v1/reports...` |
| `branding` | `get_branding`, `update_branding` | `/api/v1/settings/branding` |
| `ha` | `get_ha_status`, `force_ha_sync`, `force_ha_failover` | `/api/v1/ha/status`, `force-sync`, `force-failover` |
| `sso` | `list_sso_providers` | `/api/v1/auth/providers` (non-local entries) |

Not wrapped: `POST /api/v1/auth/saml/metadata` (browser SAML flow) and the SSO
login redirects.

## Develop

TypeScript (the public SDK must be built once, the package links to it):

```bash
cd enterprise/sdks/typescript
npm run build:core      # installs and builds core/sdks/typescript
npm install
npm run typecheck
npm run build
```

Python:

```bash
cd enterprise/sdks/python
PYTHONPATH=../../../core/sdks/python:. python3 -m py_compile voltrus_enterprise/*.py
```

## Example

```typescript
import { VoltrusEnterpriseClient } from '@voltrus/enterprise-sdk';

const client = new VoltrusEnterpriseClient({ basePath: 'http://plant-01.local:3000' });
await client.auth.login('admin', 'secret');

const licence = await client.license.status();
if (licence.feature_flags.multi_site) {
  console.log(await client.hub.getStatus());
}
```

```python
from voltrus_enterprise import VoltrusEnterpriseClient

client = VoltrusEnterpriseClient("http://plant-01.local:3000")
client.login("admin", "secret")
if client.get_license_status()["feature_flags"]["multi_site"]:
    print(client.get_hub_status())
```
