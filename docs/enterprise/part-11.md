# 21 CFR Part 11

What the Enterprise add-on gives a site that keeps electronic records under
21 CFR Part 11, how to set it up, and what it does not do yet. Part 11 is
about procedures as much as software: validation (IQ/OQ/PQ), SOPs and training
stay with the site. Licence: Enterprise (`cfr_part_11` on
`GET /api/license/status`). Code: `enterprise/src/compliance/`, the audit log
is in the core (`core/src/api/audit_log.rs`).

## What you get

| Control | Where | Status |
| --- | --- | --- |
| Electronic signature with re-authentication | `POST /api/v1/compliance/sign`, web: Data Manager › Insights › E-Signatures | Works (fixed in this audit: before, signing was admin only and took no password). The signer types their own password every time (11.200(a)); a wrong password counts towards the same lockout as sign-in (10 failures, 15 minutes) and writes a `signature_auth_failed` security event. |
| Signature manifestation (11.50) | the signature record, the list, the CSV and PDF exports | Works. Each record has the signer's user name and, once an admin has recorded it, the signer's full printed name (stored with the signature when it is given), the time, the meaning (Created, Approved, Reviewed, Verified or any text) and the reason. The PDF prints the time with its time zone. |
| Signature/record linking (11.70) | `resource_type` + `resource_id` on the record, SHA-256 hash over all fields | Works. `GET /api/v1/compliance/signatures/verify` recomputes every hash. |
| Audit trail | core audit log, hash-chained | Works. Every signature, refused signature, user change and configuration write is logged with user and time; `GET /api/v1/audit/verify` checks the chain. |
| Retention | core retention | Works. With a Part 11 or IEC 62443 licence the audit and security logs are kept whatever `retention.*_keep_secs` says (a compliance hold). |
| Export for inspection | Compliance › E-Signatures › Export CSV / Export PDF, Compliance › Audit trail › Export CSV / Export PDF, Audit Log › Export CSV | Works. The PDFs carry the signature manifestation, every record's checksum and whether it verifies, the result of verifying the whole audit hash chain, the filter on every page and page numbers. |
| Account lockout | core sign-in and signatures | Works: 10 wrong passwords lock the account for 15 minutes. |
| Password policy, auto-logout, rotation, "authority to change" | Compliance › Policy | **Stored only, not enforced.** See [Not done yet](#not-done-yet). |
| Signature required for setpoint changes, alarm acknowledgement, mode switches | none | **Not available.** Writes can require a reason per tag (core `write_requires_reason`), but not a signature. |

## Before you start

1. An Enterprise licence is active: web **More actions (⋯) › Data Manager ›
   Admin › License** shows tier Enterprise, or `GET /api/license/status`
   returns `"cfr_part_11": true`.
2. Every person has their own local account (no shared logins). Create them in
   **Data Manager › Admin › Users › + Add User**. Any role can sign (Admin,
   Demo, Operator, Viewer): signing changes no process value, and a read-only
   QA reviewer may need to sign a review.
3. Accounts that sign in through single sign-on (SAML, OIDC, LDAP) have no local
   password and cannot sign: the server answers `403 electronic signatures need
   a local user account with a password`.

There is no `compliance:` key in `config.yaml`. The mode is on when the licence
includes it; nothing else needs switching on.

## Set up

1. Sign in as an admin. Open **More actions (⋯) › Data Manager › Admin ›
   Compliance**. The **Policy** tab holds the policy values; save them so the
   project export carries your site's policy, but read the notice on that tab:
   the server does not enforce them yet.
2. Decide which records need a signature and what you call them: the
   `resource_type` is free text (for example `batch_record`, `recipe`, `sop`)
   and `resource_id` names the record (for example `BR-2026-0042`). Write this
   down in your SOP; the software does not enforce a list.
3. Record each signer's full name: **Admin › Compliance › Signer names**, type the
   person's full legal name next to their user name, **Save**. The name on file is
   stored with every signature at the moment of signing (so renaming someone later
   does not change signatures already given) and printed with it. API:
   `PUT /api/v1/compliance/signer-names/<username>` with `{"full_name": "Alice Kusuma"}`
   (admin; `GET /api/v1/compliance/signer-names` lists them). Each change is in the
   audit trail (`set_signer_name`).
4. Tell signers where to sign: **More actions (⋯) › Data Manager › Insights ›
   E-Signatures › + Sign** (every signed-in user). Admins can also use **Admin ›
   Compliance › E-Signatures**.

## Sign a record

Web: **Data Manager › Insights › E-Signatures › + Sign**, fill in Resource Type,
Resource ID, Reason, Meaning and **Your password**, then **Sign**.

API (a session cookie from `POST /api/v1/auth/login`; API keys cannot sign):

```sh
curl -b jar -H 'Content-Type: application/json' \
  -d '{"resource_type":"batch","resource_id":"B-17","reason":"Release after QA",
       "meaning":"Approved","password":"<the signer'"'"'s own password>"}' \
  https://scada.example.com/api/v1/compliance/sign
# {"ok":true,"id":2,"signer":"op2","meaning":"Approved","signature_hash":"0443…","created_at":1794826855}
```

The SDKs: `compliance.sign({..., password})` (TypeScript),
`sign_record(resource_type, resource_id, reason, meaning, password)` (Python).

## Check it works

1. Sign once without a password: `400 password is required`. With a wrong
   password: `401 password is incorrect`. With the right one: `200` and an id.
2. **E-Signatures** lists the record with user, time, reason and meaning.
3. **More actions (⋯) › Audit Log** shows `electronic_signature` (with meaning,
   reason, id and hash) and, for the wrong password, `electronic_signature_refused`.
   **Data Manager › Admin › Security** shows a `signature_auth_failed` event.
4. As admin: `GET /api/v1/compliance/signatures/verify` returns
   `{"ok":true,"checked":N,"invalid_ids":[]}` and `GET /api/v1/audit/verify`
   returns `"ok":true`.
5. Export: see [Export for an inspection](#export-for-an-inspection). The native
   console exports the audit trail as CSV from **More › Audit log**; it has no
   signature screen.

## Export for an inspection

Admin only. Both exports take the same filter for CSV and PDF; times in the PDF are
printed in the time zone of the browser that asked for it (for example
`2026-11-16 14:03:22 UTC+07:00`).

**Signature records**: **Admin › Compliance › E-Signatures**. Pick **From** and
**To** dates and optionally a **User**, then **Export CSV** or **Export PDF**. The
PDF has one block per signature:

* **Signed by**: the full name recorded at signing and the user name, e.g.
  `Alice Kusuma (alice)`. A signature given before a name was on file shows the name
  on file now, marked "not recorded at signing"; with no name at all it says "no
  full name on file".
* **Date and time** with the time zone, **Meaning**, **Reason**, **Record**
  (`resource_type / resource_id`).
* **Checksum**: the SHA-256 signature hash and `verified` or `DOES NOT VERIFY`.

**Audit trail**: **Admin › Compliance › Audit trail**. **From**, **To**, **User**,
**Action**, **Contains** (searches details, resource, user and action), then
**Export CSV** (the core export) or **Export PDF**. The PDF lists every matching
entry, oldest first, with its time, user, action, resource, details and checksum (its
hash in the audit chain), and on the first page the result of verifying the whole
chain (`verified: N entries intact`, or `BROKEN at entry N`).

Every page of both PDFs has the site name, the export title and the filter in its
header, and `page n of m` in its footer; the first page says who generated it and
when. One PDF holds at most 10,000 records: a wider filter answers `400` asking to
narrow it. Each export is written to the audit trail (`export_signatures_pdf`,
`export_audit_pdf`).

API (admin session or API key):

```sh
# 1 to 16 November 2026 in WIB (UTC+7), signer alice
curl -b jar -o signatures.pdf \
  'https://scada.example.com/api/v1/compliance/signatures/export.pdf?from=1793466000&to=1794848399&username=alice&utc_offset_minutes=420'
curl -b jar -o audit.pdf \
  'https://scada.example.com/api/v1/compliance/audit/export.pdf?from=1793466000&to=1794848399&action=electronic_signature&utc_offset_minutes=420'
```

`from` and `to` are Unix seconds (inclusive); `utc_offset_minutes` is minutes east
of UTC (-720 to 840, default 0). The CSV endpoint
`/api/v1/compliance/signatures/export` takes the same `from`, `to` and `username`.
The PDFs use the standard Helvetica and Courier fonts: characters outside Latin-1
(for example Chinese names) print as `?`; the CSV export keeps them.

Verified on 2026-11-16 against a stub-licence build (`voltrus-ha-test-node`):
steps 1 to 5 with an operator account, plus 10 wrong passwords locking the
account for signing and for sign-in.

## Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `403 CFR Part 11 requires an Enterprise licence` | The licence is Integrator or missing. Check **Admin › License**. |
| `403 electronic signatures need a local user account` | SSO or API key. Sign in with a local account. |
| `403 this account is locked or disabled` | 10 wrong passwords (sign-in or signature) within the lock window. Wait 15 minutes or have an admin reset the password. |
| `invalid_ids` not empty on verify | A signature row was changed in the database after signing. Compare with the audit log entry of the same id, which carries the original hash. |
| Writes answer `503 standby_read_only` | You are on the standby of a hot standby pair; sign on the active node. |

## Not done yet

Recorded as gaps in the audit of 2026-11-16; each needs a hook in the open core
(`core/src/edition.rs`) before the add-on can enforce it:

* **Password policy** (length, character classes, rotation): the core checks
  only an 8 character minimum when a password is set or changed.
* **Auto-logout**: sessions follow the core (15 minute access token renewed
  from a long-lived refresh cookie), not `auto_logout_minutes`.
* **Authority to change**: stored, not enforced; there is no approval workflow.
* **Signatures on control actions**: setpoint writes, alarm acknowledgement and
  mode changes cannot require a signature. Use the per-tag mandatory reason
  (`write_requires_reason`) for the "why", and sign batch or review records
  separately.
* **Previous value** of a tag write is not recorded in the audit trail.
* **Non-Latin names in PDFs**: the PDF exports print characters outside Latin-1 as
  `?` (no embedded fonts yet); use the CSV export for those.
