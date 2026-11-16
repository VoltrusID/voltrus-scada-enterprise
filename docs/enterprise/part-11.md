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
| Signature manifestation (11.50) | the signature record, the list, the CSV export | Works. Each record has the signer's user name, the UTC time, the meaning (Created, Approved, Reviewed, Verified or any text) and the reason. |
| Signature/record linking (11.70) | `resource_type` + `resource_id` on the record, SHA-256 hash over all fields | Works. `GET /api/v1/compliance/signatures/verify` recomputes every hash. |
| Audit trail | core audit log, hash-chained | Works. Every signature, refused signature, user change and configuration write is logged with user and time; `GET /api/v1/audit/verify` checks the chain. |
| Retention | core retention | Works. With a Part 11 or IEC 62443 licence the audit and security logs are kept whatever `retention.*_keep_secs` says (a compliance hold). |
| Export for inspection | Audit Log › Export CSV, Compliance › E-Signatures › Export CSV | Works, CSV. There is no PDF export. |
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
3. Tell signers where to sign: **More actions (⋯) › Data Manager › Insights ›
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
5. Export: **Admin › Compliance › E-Signatures › Export CSV**
   (`GET /api/v1/compliance/signatures/export`, admin) has one row per
   signature with `hash_valid`. **More actions (⋯) › Audit Log › Export CSV**
   (`GET /api/v1/audit/export`) exports the audit trail with the filters on
   screen. The native console exports the audit trail from **More › Audit log**;
   it has no signature screen.

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
* **PDF export** of the audit trail.
