# Per-site licences and annual support (SCADA-4): what is built, and the deploy checklist

Plan: `docs/licensing-per-site-plan.md` in the open core (`VoltrusID/voltrus-scada`, `d5ff8f4` and later). Console side: `docs/scada-per-site-support.md` in
`voltrus-console`. Rule that does not change: Core and every licensed feature keep working forever on a perpetual key; a lapsed support contract only
ends updates and support.

## What the official build does

- **Signed fields.** An activation file may carry `site_id`, `site_name` and `support_until` (Unix seconds). They are part of the signed payload
  (`ActivationPayload` in `enterprise/src/licensing/activation.rs`, appended after `device_limit` in that order, omitted when absent), so tampering with
  any of them, or stripping them, breaks the signature. A file without them re-serialises to exactly the bytes signed before: every activation issued
  so far, under the current key or a retired one, still verifies. `resolve_license_status` fills `LicenseStatus::Valid.site` and `.support_until`
  (informational), and the row stored in `license_activation` gets `site_id`, `site_name`, `support_until`.
- **Support lapse policy: soft, report-only.** The plan leaves strict-versus-soft as an owner **DECISION** and recommends soft for the first release,
  so that is what is built. A lapsed contract changes nothing but what is reported (`support: "lapsed"` on `/api/license/status`, the footer notice in
  core). The binary does **not** compare its build date with `support_until`, refuses nothing and degrades nothing. If you later choose strict, it is a
  change in `evaluate_activation` (and needs an embedded release date, which official builds do not carry today).
- **Naming a site.** `voltrus --generate-request <KEY> --site "<NAME>"` and `voltrus --online-activate <KEY> [CONFIG] --site "<NAME>"` carry the name; the
  dashboard's licence panel and `POST /api/license/activate|request` take `site_name`. The issuer uses it only for a key that has no site yet; renaming
  later happens on the console and reaches the install at its next refresh. Names are trimmed, at most 120 characters, no control characters.
- **Picking up a renewal without reactivating.** The background renewal task (every 6 hours, `online.rs`) now also re-reads the signed activation when a
  support contract ends within 10 days or has lapsed, at most once a day, so a renewal paid on the console reaches an installed key by itself. The same
  path is the dashboard's "Refresh" (`POST /api/license/refresh`). A failed refresh changes nothing.
- **Compatibility gate.** A build that predates the new fields would re-serialise the payload without them and reject the signature. So the request
  carries `features: ["site_support"]` and the worker signs the new fields **only** for a client that lists it. Older installs keep receiving the old
  format and keep working; they just do not see the site or the support date until they update.

## What the activation worker does (`worker/license-activate`)

- Reads the console (`CONSOLE_DB`): `scada_sites` and the `scada_support` view, by `license_code` (the upper-cased `VLT-*` code or the Polar key string).
  `schema.sql` documents the contract; `src/site-support.ts` is the code. The console wins on every activation and renewal, so a rename or a paid
  support period there is signed on the next `/renew`. Support is signed only when the console has a contract for the key: a grandfathered lifetime key
  gets none (`support_until` NULL, no support line in the UI).
- One key is one site. The first activation that names a site for a key without one creates it (written to the console as source `activation`); after
  that the request cannot pick another site, id or name. This is how "reject activation for a different site" is met without a site id in the request:
  the site id is issuer-assigned and a key has exactly one.
- `license_bindings` gets `site_id`, `site_name`, `support_until` (migration `0001_per_site_support.sql`, index on `site_id`): a cache of what was last
  signed for a Polar key, used only when the console cannot be read. The `/admin` panel shows them. A console that is not migrated, or D1 trouble, never
  fails an activation: it signs what the binding remembers and mints nothing.
- Lapsed support never refuses `/activate` or `/renew`. `/renew` (same key, same fingerprint, no Polar usage spent) already re-signs; it now carries the
  updated `support_until` and the current site name.

## Deploy checklist (nothing has been deployed or pushed by this work)

Order matters only where marked.

1. **Review and merge** the branches `scada-4/per-site-support` of `voltrus-scada-enterprise` and `voltrus-console` (both local, unpushed). The enterprise
   branch bumps the `core` submodule to `d5ff8f4`.
2. **D1, licence DB** (additive, must precede the worker deploy): in `worker/license-activate`, `bunx wrangler d1 migrations apply voltrus-licenses --remote`
   applies `0001_per_site_support.sql`. `wrangler.toml` now sets `migrations_dir`; the database has no migration history, so only this file runs.
3. **D1, console DB** (should precede the worker deploy; the worker degrades without it): in `voltrus-console`,
   `bunx wrangler d1 migrations apply voltrus-console --remote` applies `0055_scada_per_site_support.sql`. It seeds no price and no product; the credit
   rule is inactive.
4. **Deploy the activation worker**: `bunx wrangler deploy` in `worker/license-activate` (activate.voltrus.id and dl-voltrus-scada.voltrus.id). Existing
   installs see no change: their requests lack `site_support`, so they receive the old format.
5. **Deploy the console**: `bun run deploy` in `voltrus-console` (Licenses page: site and support line; support webhook; admin API).
6. **Release the SCADA binary** (any time after step 4; `release.sh`, then the release procedure in `docs/official-build.md`). Installs that update start
   sending `site_support` and see site and support on their next activation or refresh.
7. **Polar** (owner decisions, see below): create the annual support subscription product(s), register them in the console
   (`PUT /api/licenses/admin/scada/support-products`), create the per-site licence products and their `products` rows. The existing Polar webhook already
   delivers `order.paid` and `subscription.*`; nothing new to subscribe to.
8. **Staging end to end** with a test key (plan section 8.3), before selling anything: for a throwaway SCADA key, grant a period by hand
   (`POST /api/licenses/admin/scada/support`) and check `voltrus --license-status` reports `support: "active"`; grant a past date and check `lapsed` with
   every feature still on; revoke the periods (`.../support/revoke`) and check no support fields; then run a real test purchase of a support product and a
   `POST /renew` (dashboard "Refresh") to see `support_until` move without reactivating.
9. **Landing and comms** (not part of this change): `voltrus-landing` pricing copy, the emails to lifetime and monthly buyers, the changelog (plan
   sections 7 and 8). Keep the old checkout links until the issuer can sell per-site.

The older copy of the issuer at `voltrus/voltrus-cloudflare-worker` is superseded by `worker/license-activate` and was not touched; do not deploy it.

## Owner decisions still open

Prices of the per-site licences and of annual support (plan section 7); soft versus strict after lapse (soft is built); machines per site (unchanged: Polar
keys bind one machine, console `seat_limit` governs `VLT-*` keys); the credit rule's details and the lifetime version fence (recorded as data on the
console, not enforced); whether managed multi-site monitoring is priced per site or per fleet (not built).

## Not implemented

- A strict policy (build date versus `support_until`): not needed for the soft policy the plan recommends.
- Per-site machine counts: still a decision.
- Redeeming an upgrade credit automatically: there is no per-site checkout yet; an admin marks a credit redeemed.
- Landing pricing copy, buyer emails, managed multi-site monitoring.

## Tests

`cargo test -p voltrus-enterprise` (activation verification with both key generations, old and new wire format, tamper cases, two activations issued by
the real worker code as shared vectors, request building, persistence of site and support, refresh timing), `cargo clippy -p voltrus-enterprise -- -D
warnings`, `cargo fmt --check`; `bun run test` and `bun run typecheck` in `worker/license-activate`. The two worker-issued vectors appear in both
`activation.rs` (`VECTOR_WORKER_*`) and `test/site-support.test.ts`; change one side and the other suite fails.
