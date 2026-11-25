# 21 CFR Part 11

What the Enterprise add-on gives a site that keeps electronic records under
21 CFR Part 11, how to set it up, and what it does not do yet. Part 11 is
about procedures as much as software: validation (IQ/OQ/PQ), SOPs and training
stay with the site. Licence: Enterprise (`cfr_part_11` on
`GET /api/license/status`). Code: `enterprise/src/compliance/` (policy,
hooks, exports); the audit log and the signing prompt are in the core
(`core/src/api/audit_log.rs`, `core/src/api/signature.rs`, contract in
`core/docs/edition-hooks.md`).

## What you get

| Control | Where | Status |
| --- | --- | --- |
| Electronic signature with re-authentication | `POST /api/v1/compliance/sign`, web: Data Manager › Insights › E-Signatures | Works. The signer types their own password every time (11.200(a)); a wrong password counts towards the same lockout as sign-in (10 failures, 15 minutes). |
| Signatures on control actions | Compliance › Policy › Electronic signatures | Works. An admin chooses which of tag writes, alarm acknowledge, alarm shelve, recipe load, mode switches and design publishing need a signature, and the meaning of each. The person is asked for password and reason before the action runs; the action never runs unsigned or unrecorded. |
| Signature manifestation (11.50) | the signature record, the list, the CSV and PDF exports | Works. User name, full printed name (recorded at signing), time with time zone, meaning, reason, and for a signed action what was done (old and new value of a setpoint). |
| Signature/record linking (11.70) | SHA-256 hash over all fields; the core audit entry of every signed action carries `signature=<id>` | Works. `GET /api/v1/compliance/signatures/verify` recomputes every hash (and the checksum of the action details). The exports name the audit entry of each signed action. |
| Password policy | Compliance › Policy › Password Policy | **Enforced** on self-service change, admin create and admin reset: minimum length, uppercase, number, special character, not the user name, not one of the last N passwords. |
| Password expiry | Compliance › Policy › Password expires after | **Enforced** at sign-in: the user must choose a new password. |
| Auto-logout and session length | Compliance › Policy › Session | **Enforced**: idle timeout and a maximum session length from sign-in. |
| Audit trail | core audit log, hash-chained | Works. Tag writes record the previous and the new value. `GET /api/v1/audit/verify` checks the chain. |
| Retention | core retention | Works. With a Part 11 or IEC 62443 licence the audit and security logs are kept whatever `retention.*_keep_secs` says. |
| Export for inspection | Compliance › E-Signatures, Compliance › Audit trail | Works (CSV and PDF). |
| Account lockout | core sign-in and signatures | Works: 10 wrong passwords lock the account for 15 minutes. |
| "Authority to change" | Compliance › Policy | Stored only; there is no approval workflow. |

## Before you start

1. An Enterprise licence is active: web **More actions (⋯) › Data Manager ›
   Admin › License** shows tier Enterprise, or `GET /api/license/status`
   returns `"cfr_part_11": true`. Without it none of the rules below apply.
2. Every person has their own local account (no shared logins). Create them in
   **Data Manager › Admin › Users › + Add User**.
3. Accounts that sign in through single sign-on (SAML, OIDC, LDAP) have no local
   password and cannot sign. When an action they start needs a signature, the
   server answers `403 signature_not_possible`.

There is no `compliance:` key in `config.yaml`. **Nothing is enforced until an
admin saves the Compliance policy once**: a licensed site that never opened the
screen keeps the core's behaviour. The Policy tab says which state you are in.

## Set up, step by step

Sign in as an admin and open **More actions (⋯) › Data Manager › Admin ›
Compliance › Policy**.

### 1. Session

| Field | What it does | What the user sees |
| --- | --- | --- |
| Auto-logout after inactivity (minutes, 1 to 1440) | Signs a person out after this long without keyboard, mouse or touch input. Screens that only refresh, and the WebSocket, do not count as activity. | The sign-in screen with "You were signed out after 15 minutes of inactivity." |
| Maximum session length (hours, 0 to 744; 0 = no limit) | Signs a person out this long after signing in, active or not. | The sign-in screen with "Your session reached its maximum length. Please sign in again." |

The limits apply from the next sign-in or token refresh (at most a few minutes).
API clients that refresh tokens get `401 {"code":"session_idle"}` or
`{"code":"session_expired"}`.

### 2. Password policy

| Field | Enforced when |
| --- | --- |
| Minimum length (8 to 128) | Any new password: the user's own change, an admin creating a user or setting a password. |
| Require an uppercase letter / a number / a special character | Same. |
| Password history (0 to 24) | The user's own change and an admin reset: the new password may not be the current one or one of the last N. Only Argon2 hashes are kept (`compliance_password_history`). |
| Password expires after (days, 0 = never) | At sign-in. |
| (always) | A password equal to the user name (any case) is refused. |

What users see: a refused password answers `400` with the reason, shown under
the password field ("Password must contain at least one number", "Password was
used recently: choose one that is not among your last 5", "Password cannot be
the same as the user name"). An expired password signs the person in to the
change-password screen with "Your password has expired. Set a new one to continue."; they choose a new one
that meets the policy.

Accounts that existed before the policy was saved have no recorded change date:
their age counts from the first sign-in after the policy was saved, so nobody is
locked out on the day you switch it on.

### 3. Electronic signatures

Switch on each action that needs a signature and set its **Meaning** (the
defaults are Performed for tag writes, Reviewed for alarm acknowledge and shelve,
Approved for recipe loads, mode switches and design publishing; any text up to 80 characters).

| Action | Covers |
| --- | --- |
| Tag writes (setpoints) | Writes from screens, faceplates, the REST API, the WebSocket and MCP. **Which tags**: every writable tag, only tags whose write policy has "requires a reason" set (the core's per-tag flag), or a list of tag ids. |
| Acknowledge alarms | One alarm and "Ack All". |
| Shelve and unshelve alarms | Both directions. |
| Load recipes | One signature covers every write of the load. If loads are not signed but tag writes are, a load that writes a signed tag asks once with the tag-write meaning. |
| Mode switches | Simulation on/off, go-live and rollback, flow start/stop/run once, changing a tag's write policy. |
| Publish screen designs | Publishing a screen design, or requesting and approving a publish when publishing needs approval (the request is signed by the author, the approval by the reviewer). |

What the person sees: the action is answered `428`, and the web client opens
**Electronic signature required** showing the meaning ("You are signing:
Reviewed"), **Your password** and **Reason**. **Sign** repeats the action with the
signature; a wrong password asks again (and counts towards the lockout); **Cancel**
leaves the action undone. The native console has its own signing prompt for the
actions it offers (see the core user guide).

### 4. Unattended callers

API keys, MCP clients (AI agents) and flows cannot type a password. When an
action they start needs a signature it is **refused** (`403
{"code":"signature_not_possible"}`; a flow's write node fails with "needs an
electronic signature … which a flow cannot give").

Under **Unattended callers › Add exception** an admin can allow named callers to
act unsigned:

| Kind | Caller | Example |
| --- | --- | --- |
| API key | `api_key:<id>` (the id on Admin › API Keys) or `*` | the MES integration key |
| Flow | `flow:<id>` or `*` | a validated CIP sequence |
| MCP | the user name the agent acts for, `api_key:<id>`, or `*` | an agent that may acknowledge alarms |

An exception covers one kind only (an exception for `api_key:2` does not let the
same key act through MCP). Write in **Why** which validation covers it. People
are always asked to sign; exceptions never apply to them. Their actions are still
in the audit trail under the key or flow name.

### 5. Signer names and save

1. **Admin › Compliance › Signer names**: type each person's full legal name.
   It is stored with every signature at the moment of signing.
2. Back on **Policy**, check **What operators will see**, then **Save policy**.
   The tab now says the server enforces it. Each save is in the audit trail
   (`update_compliance_config`, with every value).

API (admin): `GET /api/v1/compliance/config` returns the policy plus
`"enforced": true|false`; `PUT` takes any subset:

```json
{
  "auto_logout_minutes": 15, "session_max_hours": 8,
  "password_min_length": 10, "password_require_uppercase": true,
  "password_require_number": true, "password_require_special": true,
  "password_history": 5, "password_rotation_days": 90,
  "signatures": {
    "actions": {
      "tag_write":   {"enabled": true,  "meaning": "Performed"},
      "alarm_ack":   {"enabled": true,  "meaning": "Reviewed"},
      "alarm_shelve":{"enabled": false, "meaning": "Reviewed"},
      "recipe_load": {"enabled": true,  "meaning": "Approved"},
      "mode_switch": {"enabled": false, "meaning": "Approved"},
      "design_publish": {"enabled": false, "meaning": "Approved"}
    },
    "tag_scope": "requires_reason", "tag_ids": [],
    "unattended_exceptions": [{"origin": "api_key", "principal": "api_key:2", "note": "MES, VAL-014"}]
  }
}
```

`tag_scope` is `all`, `requires_reason` or `listed` (with `tag_ids`).

## Sign a record (not tied to an action)

Web: **Data Manager › Insights › E-Signatures › + Sign**, fill in Resource Type,
Resource ID, Reason, Meaning and **Your password**, then **Sign**.

```sh
curl -b jar -H 'Content-Type: application/json' \
  -d '{"resource_type":"batch","resource_id":"B-17","reason":"Release after QA",
       "meaning":"Approved","password":"<the signer'"'"'s own password>"}' \
  https://scada.example.com/api/v1/compliance/sign
```

Signed actions and signed records are kept in the same records
(`electronic_signatures`); a signed action has the action as `action`
(`tag_write`, `alarm_ack`, …), what it acted on as `resource_type`/`resource_id`
(`alarm` / `41`, `data_source` / `7`, `device` / `3:go_live`) and its details.

## Check it works

1. Save a policy with **Acknowledge alarms** on.
2. **Users › + Add User** with password `password1234`: refused, "Password must
   contain at least one uppercase letter". With `Strong-Pass1`: created.
3. Sign in as that operator, open the alarm panel, **Acknowledge**: the signature
   dialog shows "Reviewed". Sign with the password and a reason: the alarm is
   acknowledged.
4. **Admin › Compliance › E-Signatures › Export PDF**: the block for the signature
   shows the full name, `alarm_ack (signed in a session as operator)`, the reason,
   `alarm / <id>`, the details and **Audit entry #N (action signature)**. The CSV
   has the same in `signer_full_name`, `origin`, `details`, `audit_entry_id`.
5. **Audit Log**: entry N is `signature` (`action=alarm_ack signature=<id>
   meaning="Reviewed" reason=…`) and the next is `alarm_acknowledge` with the same
   `signature=<id>`.
6. An API key acknowledging an alarm: `403 signature_not_possible`; after adding
   it as an exception: `200`.
7. `GET /api/v1/compliance/signatures/verify` returns `"ok":true`.

Verified on 2026-11-16 on a stub-licence node (`voltrus-ha-test-node`), in the
web client and over the API; the same steps run in CI as
`enterprise/tests/compliance_hooks.rs` (plus password history, expiry at sign-in
and the idle and absolute timeouts).

## Export for an inspection

Admin only. Both exports take the same filter for CSV and PDF; times in the PDF are
printed in the time zone of the browser that asked for it.

**Signature records**: **Admin › Compliance › E-Signatures**. Pick **From** and
**To** dates and optionally a **User**, then **Export CSV** or **Export PDF**. The
PDF has one block per signature:

* **Signed by**: the full name recorded at signing and the user name, e.g.
  `Alice Kusuma (alice)` (marked "not recorded at signing" or "no full name on
  file" when that applies).
* **Date and time** with the time zone, **Meaning**, **Reason**.
* **Action**: `sign` for a record signed on the E-Signatures page, or the signed
  action with how it was signed (`tag_write (signed in a session as operator)`).
* **Record** (`resource_type / resource_id`), **Details** (for a tag write
  `{"key", "device_id", "value", "previous"}`), **Audit entry** (the core audit
  entry written with the signature).
* **Checksum**: the SHA-256 signature hash and `verified` or `DOES NOT VERIFY`
  (also when the stored details were changed after signing).

**Audit trail**: **Admin › Compliance › Audit trail**, same filters plus
**Action** and **Contains**; the PDF lists every entry with its chain hash and the
result of verifying the whole chain.

Every page of both PDFs has the site name, the title and the filter in its
header, and `page n of m` in its footer. One PDF holds at most 10,000 records.
Each export is written to the audit trail.

```sh
curl -b jar -o signatures.pdf \
  'https://scada.example.com/api/v1/compliance/signatures/export.pdf?from=1793466000&to=1794848399&utc_offset_minutes=420'
```

The PDFs use the standard Helvetica and Courier fonts: characters outside Latin-1
print as `?`; the CSV export keeps them.

## Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `403 CFR Part 11 requires an Enterprise licence` | The licence is Integrator or missing. Check **Admin › License**. |
| Policy saved but nothing changes | Check the Policy tab says the server enforces it. Session limits apply from the next refresh (minutes); passwords from the next change. |
| `403 signature_not_possible` | An API key, MCP client, flow or SSO account started an action that needs a signature. Sign in with a local account, or add a named exception. |
| `428 signature_required` from your own integration | The action needs a signature: send `"signature": {"password", "meaning", "reason"}` with the meaning from the 428, from a session (not a key). |
| `403 account_locked` | 10 wrong passwords (sign-in or signature). Wait 15 minutes or have an admin reset the password. |
| `invalid_ids` not empty on verify | A signature row or its details were changed in the database after signing. Compare with the audit entry carrying the same `signature=<id>`. |
| Writes answer `503 standby_read_only` | You are on the standby of a hot standby pair; act on the active node. |

## Not done yet

* **Expired password and the API**: an expired password sends the web client to
  the change-password screen, but the core does not refuse other API requests of
  that session until it is changed (core gap, see `core/docs/edition-hooks.md`).
* **Signature scope per tag group or role**: tag writes are scoped by all / the
  "requires a reason" flag / a tag id list. The core has no "critical tag" flag
  or tag groups to scope by.
* **Two-person rule** (a second signer): not available.
* **Authority to change**: stored, not enforced; there is no approval workflow.
* **Non-Latin names in PDFs** print as `?`; use the CSV export.
