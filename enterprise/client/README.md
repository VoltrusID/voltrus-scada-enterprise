# Voltrus Enterprise client module

Proprietary UI that the public core client (`client/`, AGPL-3.0) loads through its edition seam
(`client/src/edition/`). The module contributes: Hub, Cameras (RTSP), scheduled Reports, IEC 62443
Security audit and 21 CFR Part 11 Compliance tabs, the License panel, SSO sign-in options, white-label
branding, the HA banner and the Telegram and WhatsApp notification channels. Each feature checks its licence flag and shows
an upgrade notice when the flag is off.

The module's default export is an `EnterpriseClientModule` (`client/src/edition/types.ts`), see `src/module.tsx`.

## Build the official client

From the core client directory:

```sh
cd <core client dir>            # core/client in the private repo
VOLTRUS_ENTERPRISE_CLIENT=<abs path to enterprise/client/src/index.ts> npm run build
```

Without `VOLTRUS_ENTERPRISE_CLIENT` the core build resolves `@voltrus/enterprise-client` to the
Community stub (`src/edition/community.ts`) and contains none of this code.

Or from this directory (finds the core client itself, see below):

```sh
npm run build -- --outDir /tmp/voltrus-client   # extra args go to vite
npm run dev
```

Note that core's `npm run build` copies the output into `../static`; the `npm run build` here only runs
`vite build` into core's `dist` (or `--outDir`).

## How the module reaches core

- Enterprise code imports core as `@voltrus/core/...` (for example `@voltrus/core/queries/fetch`,
  `@voltrus/core/styles`, `@voltrus/core/edition/license`). Never use relative paths into core.
- `@voltrus/core` is a Vite alias (and a TS `paths` entry) for the core client `src` directory. The core
  `vite.config.ts` reads `VOLTRUS_CORE_CLIENT_SRC` (default `./src` next to that config).
- `vite.config.ts` here and `scripts/resolve-core.mjs` set `VOLTRUS_CORE_CLIENT_SRC` for you. Without the
  variable they look for core at `../../core/client/src` (private repo layout), then `../../client/src`
  (the current monorepo layout). Set `VOLTRUS_CORE_CLIENT_SRC=<abs path to core/client/src>` to override.
- The module has no `node_modules`. The core Vite config resolves the module's bare imports (`react`,
  `@tanstack/react-query`...) from the core client, so there is one copy of each dependency.

## Type-check

```sh
npm run typecheck
```

`tsconfig.json` has the `paths` for the private repo layout. `scripts/typecheck.mjs` regenerates them for
whichever core checkout it finds (temporary `tsconfig.generated.json`) and runs core's `tsc`.

## Tests

```sh
npm test    # node --experimental-strip-types, pure logic only
```

## i18n

The locale strings that belonged to enterprise features (`license`, `reports`, `security`, `compliance`)
moved to `src/locales/{en,id,zh,de}.json`. The module registers them at load under the `enterprise`
namespace (`useTranslation('enterprise')`). The current panels use plain English text and do not read them yet.

## Server endpoints used

`/api/license/{status,activate,request,refresh}`, `/api/v1/settings/branding`, `/api/v1/auth/providers`,
`/api/v1/auth/ldap/login`, `/api/v1/auth/{oidc,saml}/login` (assumed browser-redirect endpoints that start OIDC/SAML sign-in; the current server does not route them), `/api/v1/hub/*`,
`/api/v1/cameras/*`, `/api/v1/reports/*`, `/api/v1/security/*`, `/api/v1/compliance/*`, `/api/v1/ha/status`.
