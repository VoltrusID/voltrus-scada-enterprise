# Licensing

Voltrus Core needs no licence and never phones home. A licence key unlocks the
Integrator or Enterprise add-on features of the official binary on **one machine**.
An Enterprise licence also covers that machine's partner in a
[hot standby pair](#one-licence-for-a-hot-standby-pair): activate it once, on one of
the two. A missing, expired or invalid licence never limits Core: the server keeps
running as Community and only the add-on features switch off.

Contents: [What a key unlocks](#what-a-key-unlocks) · [How it works](#how-it-works) ·
[Activate in the web UI](#activate-in-the-web-ui) · [Activate from the command line](#activate-from-the-command-line) ·
[Offline (air-gapped) activation](#offline-air-gapped-activation) · [Check the status](#check-the-status) ·
[One licence for a hot standby pair](#one-licence-for-a-hot-standby-pair) ·
[Monthly keys, renewal and grace](#monthly-keys-renewal-and-grace) · [Moving to another machine](#moving-to-another-machine) ·
[What happens when the licence is not valid](#what-happens-when-the-licence-is-not-valid) · [Troubleshooting](#troubleshooting) ·
[For developers](#for-developers-testing-without-a-production-key)

## What a key unlocks

| Flag (`/api/license/status` › `feature_flags`) | Feature | Integrator | Enterprise |
|---|---|:-:|:-:|
| `white_label`, `theme_full` | White-label branding: title, subtitle, logo, colours of the light and dark theme, font ([branding.md](branding.md)) | yes | yes |
| `pdf_reporting` | Scheduled PDF reports | yes | yes |
| `chat_notifications` | Telegram and WhatsApp alarm notifications | yes | yes |
| `multi_site` | Multi-site hub | yes | yes |
| `sso`, `sso_saml` | Single sign-on: OIDC; SAML 2.0 and LDAP/AD ([sso.md](sso.md)) | | yes |
| `cfr_part_11` | 21 CFR Part 11 mode | | yes |
| `iec_62443` | IEC 62443 security audit mode | | yes |
| `ha` | Hot standby with automatic failover; one licence covers both nodes | | yes |
| `rtsp_video` | RTSP video cameras | | yes |

Keys sold before the open-core split keep working: `professional`/`pro` resolve to
Integrator, `enterprise` stays Enterprise, `starter`/`edge` resolve to Community but
keep the logo-only white-label they were sold with.

## How it works

1. The server collects a **machine fingerprint**: host name, CPU model, disk serial,
   MAC address and machine id (`/etc/machine-id` on Linux).
2. Activation sends the licence key and the fingerprint to the activation server
   (`https://activate.voltrus.id/activate`), directly (online) or through a file you
   carry (offline).
3. The activation server checks the key, binds it to that fingerprint (one machine per
   key) and returns an **activation file** (`license-activation.bin`) signed with
   Ed25519. It carries the key, the fingerprint hash, the tier, the expiry (monthly
   keys), and for per-site keys the site and the support end date.
4. The server checks the signature against the public keys built into the binary,
   refuses a file issued for another machine or already expired past grace, and
   stores it in its database. From then on every check is **offline**: no licence
   server, no internet needed.
5. At every start the stored activation is checked again: signature, expiry and
   grace, and the fingerprint. The fingerprint check is fuzzy: 3 of the 5
   components must still match, so replacing a disk or a network card, or renaming
   the host, keeps the licence; moving to a new machine does not.

The licence lives in the server's database (`db_path` in `config.yaml`), not in a
file you need to keep. A backup restored on the same machine restores it too.

Where keys come from: the purchase receipt email, or
[console.voltrus.id](https://console.voltrus.id) (keys start with `VLT-`). Plans and
prices: <https://voltrus.id/voltrus-scada>.

## Activate in the web UI

Needs an `admin` account (the License tab and its API are admin only).

1. Sign in, open **More actions (⋯, top right) › Data Manager**.
2. In the sidebar, under **Admin**, click **License** (direct link:
   `https://<server>/data?tab=license`). The **License** card shows Edition, Status,
   Key and Expires.
3. **Online** (the server can reach `https://activate.voltrus.id`): paste the key into
   **Activate with a license key** (`VLT-...`), optionally type the **Site name** (only
   for a per-site key that has no site yet), click **Activate online**. "License
   activated." appears and the card shows the new edition. Features unlock at once;
   no restart.
4. **Offline**: see [Offline activation](#offline-air-gapped-activation), web UI steps.

## Activate from the command line

Run as the user the service runs as, with the same `config.yaml` (it holds `db_path`
and, if set, `db_encryption_key`). The CLI writes the database; **restart the service
afterwards** so the running server picks the licence up (the web UI path needs no
restart).

```sh
# online
sudo -u voltrus voltrus --online-activate VLT-XXXX-XXXX /etc/voltrus/config.yaml
# a per-site key that has no site yet
sudo -u voltrus voltrus --online-activate VLT-XXXX-XXXX /etc/voltrus/config.yaml --site "Plant 3"
sudo systemctl restart voltrus
```

Output on success: `License activated successfully: VLT-XXXX-XXXX` (plus `Site:` and
`Support until` when the key has them). On Windows run `voltrus.exe` the same way from
the install folder, then restart the Voltrus service.

## Offline (air-gapped) activation

The server never needs internet access. You carry two small files.

**Web UI**

1. **More actions › Data Manager › Admin › License**. Type the key in **Activate with a
   license key** (and the site name if needed), click **Save request file**. The browser
   downloads `license-request.bin`.
2. On any computer with internet, open <https://voltrus.id/activate/>, choose
   `license-request.bin`, click **Generate Activation File**. The page downloads
   `license-activation.bin` (the key is inside the request file; you do not type it again).
3. Back on the Voltrus server: **License › Import activation file**, pick
   `license-activation.bin`. "Activation file imported." appears and the features
   unlock at once.

**Command line**

```sh
voltrus --generate-request VLT-XXXX-XXXX [--site "Plant 3"]
#   License request written to: license-request.bin
#   Upload it at https://voltrus.id/activate/ ...
# carry license-request.bin to a connected PC, upload it at https://voltrus.id/activate/,
# carry license-activation.bin back, then:
sudo -u voltrus voltrus --import-activation license-activation.bin /etc/voltrus/config.yaml
sudo systemctl restart voltrus
```

`--generate-request` must run **on the machine that will be licensed** (it reads that
machine's fingerprint). `https://activate.voltrus.id` itself is the API, not a page;
upload at `voltrus.id/activate`.

## Check the status

* Web UI: **Data Manager › Admin › License**. The native console shows the edition as a
  badge in its navbar and on its sign-in card (read only; it cannot activate).
* API (public, no sign-in): `curl -s http://localhost:3000/api/license/status`
* CLI (exit code 0 when valid, 1 otherwise):

```sh
voltrus --license-status /etc/voltrus/config.yaml
```

```json
{
  "status": "Valid",
  "license_key": "VLT-XXXX-XXXX",
  "expires_at": null,
  "days_remaining": null,
  "tier": "enterprise",
  "grace": false,
  "support": "none"
}
```

| `status` | Meaning | Add-on features |
|---|---|---|
| `Valid` | Signed, this machine, not expired (or within grace: `"grace": true`) | on |
| `Unactivated` | No licence stored | off |
| `Expired` | Monthly key past expiry + 7 days | off |
| `InvalidSignature` | The stored file does not verify (tampered, or signed with a retired key) | off |
| `InvalidFingerprint` | Fewer than 3 of 5 fingerprint components match: the database moved to another machine | off |
| `Corrupted` | The licence row cannot be read | off |

## One licence for a hot standby pair

One Enterprise licence covers both machines of a [hot standby](hot-standby.md) pair.
You activate it on **one** machine (the licensed node); its partner needs no key, no
activation and no second purchase, and keeps every Enterprise feature when it takes
over.

### Set it up

1. Install the same Voltrus version on both machines and configure the pair as in
   [hot-standby.md](hot-standby.md#set-up-a-pair-step-by-step) (same
   `shared_secret`, different `node_id`).
2. On the machine that will be the primary, activate the Enterprise key (web UI,
   CLI or offline, as above). Its License card shows Enterprise.
3. On the second machine activate **nothing**. Give it `role: standby` in its `ha:`
   block (a machine without a licence of its own joins a pair only as the standby).
4. Start the primary, then the standby. The standby takes its copy of the data, and
   with the first heartbeat of the replication session it receives the licence.
5. Check: on the standby, **Data Manager › Admin › License** (or
   `GET /api/license/status`) shows the same key and Enterprise, and
   `GET /api/v1/ha/status` shows
   `"license": {"source": "peer", "holder_node_id": "node-a", "confirmed_at": "…", "valid_until": "…"}`.
   On the licensed node `license.source` is `own`.

Nothing else is needed for failover: when the primary dies the standby takes over
with the licence it holds, and when the old primary comes back (as the new standby)
it confirms the licence again. A switchover (`POST /api/v1/ha/force-failover`) works
the same way in both directions.

### How it works

* The licence stays bound to the licensed machine's fingerprint, exactly as for a
  single node. Over the pair link (TLS 1.3 and the shared-secret proof bound to the
  session) that machine sends its signed activation file with every heartbeat of a
  replication session, in whichever direction the session runs.
* The partner checks the Ed25519 signature against the keys built into its own
  binary, the expiry, and that the tier includes hot standby, then runs with that
  licence. It stores the file and the time it was last confirmed in `ha_license_share`
  (a node-local table that never replicates), so a restart keeps it.
* **Grace**: the partner keeps the licence for **30 days** after the licensed machine
  was last heard. That covers a primary that died and is being repaired. After 30
  days without it the partner's add-on features switch off (Core keeps running, it
  stays in the pair, `license.source` turns `expired`), and they come back as soon as
  the licensed machine rejoins. To run on the surviving machine alone for longer,
  [move the licence](#moving-to-another-machine) to it and activate it there.
* A monthly key is renewed by the licensed machine; the new file reaches the partner
  with the next heartbeat.

### What one licence does not allow

* **A third machine.** Copying the activation file or the whole database to another
  machine gives `This activation file was issued for a different machine.` or status
  `InvalidFingerprint`, as before.
* **Passing it on.** Only a machine's own, fingerprint-bound activation is ever sent:
  a partner never forwards a licence it received, so a licence reaches one partner at
  a time.
* **Two independent sites.** Only the peer of a replication session confirms the
  licence, and that peer shares the partner's database. Two machines split apart run
  independently for the 30 days at most; joining them again makes one of them take a
  full copy from the other (its own data is replaced). Which node is active is decided
  by the pair's epochs and witness as always (see
  [Failover safety](hot-standby.md#failover-safety)); configure an arbiter witness to
  rule out two active nodes during a link failure.
* **Taking over before the licence arrived.** A standby that has no licence of its own
  and has not received one never promotes; `GET /api/v1/ha/status` says so in
  `promotion_blocked`.

## Monthly keys, renewal and grace

* **Lifetime keys** have no expiry.
* **Monthly keys** carry the end of the paid period. An online server renews by itself:
  a background task checks every 6 hours and asks the activation server for a fresh
  file when the key ends within 10 days (it never binds a new machine and never uses up
  an activation). You can force it with **License › Refresh from activation server**
  (shown for monthly keys) or `POST /api/license/refresh`.
* **Grace**: after the period ends the features stay on for **7 days** (the License
  card says "expired, grace period running"). After that the status is `Expired` and
  the server runs as Community until a renewed file is installed.
* **Offline servers** with a monthly key: before the period ends, repeat the
  [offline activation](#offline-air-gapped-activation) (same key, same machine) to get
  a file with the new end date.
* **Support contract** (per-site keys): `support_until` is informational. A lapsed
  contract never switches anything off; it is re-read on the same renewal path, at most
  once a day while it is near its end.

## Moving to another machine

A key is bound to one machine. Activating it on a second machine is refused with
"License already bound to a different device. Contact support to transfer." Email
support@voltrus.id with the key; once the binding is moved, activate on the new
machine. Copying the database or the activation file to another machine does not move
the licence (`InvalidFingerprint`, or "This activation file was issued for a different
machine." on import).

## What happens when the licence is not valid

The server always starts and Core always works: devices, tags, dashboards, alarms,
historian, local accounts and the REST API are never limited. Add-on features
switch off:

* **SSO**: the SSO buttons disappear and the SSO routes answer 403. Local accounts,
  including `admin`, still sign in with their passwords; keep one local admin.
* **White-label**: the UI shows the Voltrus name and logo again (your settings are
  kept and come back with a valid licence).
* Scheduled reports, chat notifications, hub, compliance and audit modes, hot standby
  and cameras stop as described in their own documents.

A licence state change made through the web UI (activate, import, refresh) applies at
once. A change made with the CLI applies at the next start.

## Troubleshooting

| Message | Cause and fix |
|---|---|
| `Failed to reach the activation server. Check the internet connection, or activate offline.` | No route or proxy to `https://activate.voltrus.id` (HTTPS, port 443). Use the offline flow. |
| `Activation failed: License key not found` (or `... not found in Polar.sh`), `License is not active`, `License has been revoked`, `License is expired` | The key is mistyped, refunded, disabled, or its subscription ended. Check it at console.voltrus.id or with support. |
| `Activation failed: Polar.sh validation failed: ...` | The licence store could not be reached from the activation server; try again later. |
| `Activation failed: License already bound to a different device...` | The key is active on another machine; see [Moving](#moving-to-another-machine). Reinstalling the OS on the same hardware can change the machine id; support can rebind. |
| `This activation file was issued for a different machine.` | The request file was generated on another machine, or the fingerprint changed between request and import. Generate the request on this machine and import the file it produces. |
| `Invalid activation signature` / `Invalid activation file: bad magic header` | The file is damaged (edited, truncated, or saved as text). Download it again; transfer it as a binary file. |
| `This licence has expired...` on import | The monthly period ended more than 7 days ago. Renew the subscription, then activate again. |
| Status `InvalidFingerprint` after hardware work | More than 2 of host name, CPU, disk serial, MAC, machine id changed. Activate again on this machine (online or offline); if the key is still bound to the old fingerprint, ask support to rebind. |
| Status `InvalidSignature` after an update | The stored file was signed with a retired key. An online server fetches a re-signed file by itself within minutes of starting; offline, repeat the offline activation. |
| CLI activation worked but the UI still shows Community | Restart the service; the running server reads the licence at start. |
| `admin access required` on the License tab or `/api/license/*` | Only the `admin` role may activate. |

## For developers: testing without a production key

Only the production public keys are compiled into the binary, so only the
activation server can issue a file the official binary accepts. Nothing in this
repository can sign one, by design.

* Unit tests sign with throwaway keys (`enterprise/src/licensing/tests.rs`,
  `activation.rs`); `activation_signed_by_the_worker_verifies` checks a file produced
  by the real worker code (`worker/license-activate`, test key) against the Rust
  verifier, so the two cannot drift apart.
* `cargo build --features ha-test-node --bin voltrus-ha-test-node` builds the real
  add-on with a stub Enterprise licence for local testing (never released). The SSO
  live suite uses it. With `VOLTRUS_TEST_LICENSE=real` it runs the real licence check
  instead; that build also trusts the worker test key (seed `0x11`, see
  `activation::test_support`), reads the machine from `VOLTRUS_TEST_MACHINE` and the
  pair grace period from `VOLTRUS_TEST_PAIR_GRACE_SECS`. The pair licensing test
  (`one_licence_covers_the_pair` in `enterprise/tests/ha_pair.rs`) runs three
  "machines" that way. None of this exists in a release build.
* `VOLTRUS_ACTIVATION_URL` points online activation and renewal at another activation
  server (e.g. `wrangler dev` of `worker/license-activate`); its files only verify if
  it signs with a production key.
