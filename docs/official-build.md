# Official build (core + Enterprise): layout and sync procedure

This repository builds the paid product and is public (source available, see
`enterprise/LICENSE`). The open-source Core lives in
<https://github.com/VoltrusID/voltrus-scada> and is consumed here as the `core/`
git submodule. Public is upstream; this repo adds the Enterprise add-on.

```
core/         submodule: VoltrusID/voltrus-scada (AGPL-3.0-or-later)
enterprise/   Rust crate voltrus-enterprise (Voltrus Enterprise License), depends on ../core
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

`release.sh` and `scripts/release-local.sh` build the official artifacts from this
layout: `voltrus` from `enterprise/`, `modbus-sim` from `core/`, the client from
`core/client` with the enterprise module (output `core/static`). Archive names and
layout are unchanged. `--dry-run` builds and packages, then prints the R2/D1 upload
plan without running any remote command; `--skip-upload` (`--build-only` in
`release.sh`) builds only. `BUILD_DIR` moves the output.

### Publishing a release

1. Bump `version` in `enterprise/Cargo.toml` (and `enterprise/Cargo.lock`), merge to
   `master`, wait for CI to pass.
2. Tag it (`git tag v0.47.1 && git push origin v0.47.1`). `.github/workflows/release.yml`
   builds the four archives with `release.sh --build-only --target <platform>` (the
   tag must equal the Cargo version) and attaches them, with `SHA256SUMS`, to a GitHub
   release.
3. Publish to R2 and D1, which is where customers download from
   (`dl-voltrus-scada.voltrus.id`): download the assets into `target/release-artifacts`
   (`gh release download v0.47.1 --pattern 'voltrus-v*' --dir target/release-artifacts`)
   and run `DESCRIPTION='...' ./release.sh --upload-only` (`--dry-run` first). Objects go to
   `voltrus/<tag>/<platform>/` and `voltrus/latest/<platform>/` in the bucket
   `voltrus-releases`; D1 `voltrus-releases-db` gets the records only after every
   upload succeeded, with a semver-ordered version (0.47.1 is 47001) that the worker's
   `/latest` and `/downloads` serve.
4. `deploy/deploy.sh [latest|VERSION]` installs a release on the VPS from the same
   download worker.

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

The activation signing key is NOT in this tree. The original key's private half
was committed once in the history of the former private repository
(`.license-private-key`, tags v0.17.5 to v0.45.0). Treat it as compromised. That
history was never published: this public repository starts from a rewritten
history with the key, local databases and backup archives removed. Keep
`.license-private-key` in `.gitignore`.

### Rotation (2026-10)

* New key id `2026-10`. The private half exists only in
  `/run/media/dzul/Data/voltrus-secrets/license-signing-key-2026-10.txt` (the drive
  is exFAT, so POSIX permissions cannot be enforced there: keep the drive offline
  when not needed) and as the Cloudflare Worker secret `ED25519_PRIVATE_KEY` of
  `voltrus-license-activate` (64 hex characters, the Ed25519 seed, which is what
  `worker/license-activate/src/index.ts` passes to `sign`). The console worker does
  not sign activations and holds no copy.
* Trusted public keys (`TRUSTED_PUBLIC_KEYS` in
  `enterprise/src/licensing/activation.rs`), newest first. Builds up to v0.47.0 trust
  only the original key. v0.47.1 and later trust only the 2026-10 key: the original key
  was retired in v0.47.1 (see "Retiring the old public key"). During a rotation the list
  holds the new key and the retiring one, so a file signed by either verifies. The
  worker signs everything new with the 2026-10 key.
* An install that is online and holds an activation its build cannot verify, or one
  signed with a key being retired, asks the worker for a fresh one (`/renew`, same
  machine, spends no activation) when it starts and then every six hours. Offline
  installs have to import a new file.
* A binary older than v0.47.1 does not know the 2026-10 key. Since the worker
  switched, an activation or renewal issued to such a binary fails signature
  verification there, so tell customers to update before they renew or activate.
* Signing a test activation: `cargo run --example sign_activation -- --key FILE
  --tier enterprise --out activation.bin` in `enterprise/` (binds to this machine's
  fingerprint, `--expires`, `--fingerprint`, `--license-key` and `--issued-at`
  override). It lives in `examples/`, so `cargo build --release` never builds it and it
  is not in the packaged binary. The official binary has no override of the trusted
  keys; tests inject keys only through the private `verify_with_keys`.

### Retiring the old public key

Do this once old-signed activations are gone, or the remaining holders were told. The
worker records the key id of every activation it signs in D1 table `activation_signings`
(database `voltrus-licenses`; a licence with no row was last signed before the
rotation, so with the old key). The worker stores no activation file, only that row:
each `/activate` and `/renew` signs a fresh file with the current key, so there is
nothing server-side to re-sign. A licence moves to the new key when its install next
asks.

1. Release a build that trusts both keys and let online installs re-sign (first start
   after the update). Allow a full monthly billing cycle so monthly keys renew once too.
2. Count bindings still on the old key (revoked ones do not matter):

   ```sh
   cd worker/license-activate
   bunx wrangler d1 execute voltrus-licenses --remote --command "
     SELECT COUNT(*) AS still_old FROM license_bindings b
     LEFT JOIN activation_signings s ON s.license_key = b.license_key
     WHERE b.revoked = 0 AND (s.key_id IS NULL OR s.key_id != '2026-10')"
   ```

   List them with `SELECT b.license_key, b.tier, b.activated_at ...` for outreach.
   Console licences (`VLT-*`) have no row in `license_bindings`; check
   `activation_signings WHERE license_key LIKE 'VLT-%'` against the console seats
   (`license_activations` in `voltrus-console`) the same way. A binding whose key
   Polar no longer knows (validate answers 404) can never be renewed.
3. Contact holders that remain (offline or lifetime installs that never came online).
   They either connect once, or import a new file from the console or support.
4. When the count is zero, or the remaining holders were told and accepted a date:
   delete the old entry of `TRUSTED_PUBLIC_KEYS`, regenerate the two fixtures in the
   `activation.rs` tests (one activation signed by the current key, one by the retired
   key, both bound to a fake fingerprint and long expired) and release. Rotate
   `SIGNING_KEY_ID` only on the next real rotation. Old-signed files then fail with
   "Invalid activation signature" and the install runs as Community until it renews;
   core keeps working.
5. After dropping the key from the binary and once every supported release has it,
   the old private key has no value to an attacker. Until then, an attacker with it
   can forge a licence that verifies on any binary that still trusts the old key, so
   customers should update.

#### Record: original key retired in v0.47.1 (2026-10-24)

No release with both keys was ever shipped, so v0.47.1 is the first build that knows
the 2026-10 key and it drops the original key at once. State when it was cut:

* `license_bindings` held 8 rows, none revoked, none with an `activation_signings` row,
  so all were last signed with the original key. 6 are valid in Polar (lifetime keys, no
  expiry). 2 are unknown to Polar (validate answers 404), so `/renew` refuses them.
* The console had 2 SCADA `VLT-*` licences: one revoked, one active that has no device
  seat. No console activation existed.
* Because the worker keeps no activation file, each of the 6 live installs moves to the
  new key the first time it runs v0.47.1 online: the build sees its stored file fail
  verification and renews at start-up. Offline installs, and the 2 unknown keys, run as
  Community on v0.47.1 until they import a new file.
* v0.47.0 and older keep trusting the original key. Customers should update.

### Monitoring

* Alert on a rising count of `activation_signings` rows with `key_id != '2026-10'`
  after the rotation (that means the secret was rolled back) and on worker 5xx
  rates (`wrangler tail voltrus-license-activate`, observability is enabled).
* Customer symptom for an unverifiable file is `Invalid activation signature` in
  the Licence panel or `InvalidSignature` in `GET /api/license/status`. A cluster
  of these after a release means a binary without the new key got a new-key
  activation.
* Legacy Starter and Edge keys resolve to Community plus logo-only white-label
  (`legacy_white_label` in the licence status); they are not affected by the rotation.

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
