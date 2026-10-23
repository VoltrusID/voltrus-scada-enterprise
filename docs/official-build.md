# Official build (core + Enterprise): layout and sync procedure

PRIVATE. This repository builds the paid product. The open-source Core lives in
<https://github.com/VoltrusID/voltrus-scada> and is consumed here as the `core/`
git submodule. Public is upstream; this repo adds the Enterprise add-on.

```
core/         submodule: VoltrusID/voltrus-scada (AGPL-3.0-or-later)
enterprise/   Rust crate voltrus-enterprise (proprietary), depends on ../core
  client/     enterprise web client module (loaded by core via @voltrus/enterprise-client)
  sdks/       private SDK extensions (typescript, python) for the Enterprise endpoints
worker/       licence activation worker (activate.voltrus.id)
deploy*.sh, release.sh, scripts/   internal deploy and release tooling
docs/         internal docs (this file, plans, old specs)
```

## Build

`scripts/build-official.sh` builds the client with the enterprise module, then
`enterprise/` in release mode. The enterprise crate's `voltrus` binary calls
`voltrus::run(Arc::new(Enterprise))`. It is Core plus: licensing (offline
signed keys, 7 day grace, online renew), SSO, 21 CFR Part 11, IEC 62443 audit,
hot standby, multi-site hub, RTSP video, white-label, scheduled PDF/SMS
reports, hosted Voltrus AI provider.

`release.sh` and `scripts/release-local.sh` still assume the old single-crate
layout (`cargo build --bin voltrus --bin modbus-sim` at the repo root). Adapt
them to `scripts/build-official.sh` output before the next official release.

## Where changes go

* Core feature or fix: do it in `core/` on a branch of the public repo (a normal
  clone of VoltrusID/voltrus-scada), open a PR there, let public CI pass, merge.
  Never put internal names, customer data, pricing or secrets in core. Run
  `open-core/audit/leak-scan.sh core` before pushing.
* Enterprise feature: in `enterprise/` (server) and `enterprise/client/` (UI).
  Core exposes only the seam in `core/src/edition.rs` and
  `core/client/src/edition/`. If the add-on needs a new hook, add a minimal
  generic hook to core (no enterprise logic), release core, then use it here.
* Bump core here: `git -C core pull origin main && git add core && git commit -m "Bump core"`,
  then run `scripts/build-official.sh` and the smoke test.

## Core snapshot procedure (initial, already done for v0.47.0)

`open-core/scripts/make-snapshot.sh SRC DST` copies tracked files minus
`open-core/audit/publish-exclude.txt`; `leak-scan.sh` must report clean; the
snapshot became a single-commit public repo. Do not copy history into core.

## Licence signing key

The activation signing key is NOT in this tree. The audit found that
`.license-private-key` was committed once in history (tag v0.17.5 and later) of
this private repo. Rotate it before treating this repo's history as safe to
share, and never push this history to any public remote.

## SDKs

Two layers, both versioned in lockstep with the server.

* **Public Core SDKs** (`core/sdks/typescript`, `core/sdks/python`, MIT, source
  of truth is the public repo). They cover Core endpoints only. They are
  hand-written clients modelled on the OpenAPI document of the Core build:
  `core/sdks/generate.sh` dumps that spec (`voltrus --dump-openapi`) and
  generates reference clients into `core/sdks/generated/` (git-ignored) to diff
  against. It does not overwrite the hand-written packages. Enterprise-only
  endpoints are not in that spec, so they never appear in the public SDKs.
* **Enterprise SDK extensions** (`enterprise/sdks/typescript`,
  `enterprise/sdks/python`, private). `VoltrusEnterpriseClient` subclasses the
  public `VoltrusClient` and adds licence activation, compliance, security
  audit, hub, cameras, scheduled reports, branding, HA and SSO providers. See
  `enterprise/sdks/README.md`.

Rules:

* Version lockstep: the public SDK version, both extension package versions
  and the peer pin (`voltrus` in `package.json`, `voltrus==` in
  `pyproject.toml`) are the same number. Bump them in one change, together with
  the `core/` submodule bump.
* The extensions must not be published to npmjs or PyPI.
* On a Core-only server every Enterprise endpoint answers 404, and the client
  raises `VoltrusApiError` / `VoltrusError` with `status` 404. A licence tier
  that is too low answers 403.
* New Enterprise endpoint: add the route and handler in `enterprise/src`, then
  add the shapes and call to both extension packages. Do not add it to the
  public SDKs.
