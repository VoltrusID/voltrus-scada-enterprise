# Multi-site hub

One Voltrus server (the hub) shows live values and keeps history of tags from
other Voltrus servers (the sites). Each site runs its own server and keeps
working on its own when the link is down. Licence: Integrator or Enterprise
(`multi_site`) **on the hub**; sites need no add-on licence. Code:
`enterprise/src/hub/`.

## How it works

* The hub opens a WebSocket to each site (`ws://` or `wss://<site>/api/v1/ws`)
  and authenticates with a site API key in the `X-API-Key` header. Use
  `https://` site URLs across untrusted networks; the hub checks the site's
  certificate against the public CA roots (a self-signed site certificate is
  refused).
* Every tag update of the site lands in one hub device, `<site name>:default`,
  under the site's tag key. The hub stores history for it like for its own
  devices, so trends and reports on the hub cover the sites.
* **Import tags** creates one hub tag per site tag, key
  `<site name>:default.<site tag key>`, copying name (prefixed with the site
  name), unit, range and warning/critical limits from the site. Hub alarms then
  work on those tags like on local ones. Run it again after a site gains tags;
  existing hub tags are left alone.
* When a site's link drops, its hub tags turn `offline` with quality
  `bad_comm_failure` at once; the hub retries with back-off (1 s doubling to
  60 s) and resumes on its own.
* What is **not** aggregated: the sites' own alarms, alarm acknowledgements,
  screens, users and configuration. The hub never writes to a site.

## Set up

On each site:

1. Sign in as admin, **More actions (⋯) › Data Manager › Admin › API Keys ›
   create a key** with the read-only role (`api`), named after the hub. Copy
   the key; it is shown once.
2. Make sure the hub can reach the site's HTTP(S) port.

On the hub:

3. Licence: **Data Manager › Admin › License** shows Integrator or Enterprise.
4. **Data Manager › Data › Hub › + Add Site**: name (a short label; it becomes
   part of every tag key, so do not rename it later), URL
   (`https://plant-a.example.com` or `http://10.1.0.10:3000`), API key. The
   site connects at once. API: `POST /api/v1/hub/sites`
   `{"name","url","api_key","enabled"}`.
5. In the site's row, **Import tags** (`POST /api/v1/hub/sites/<id>/import-tags`).
6. Put the new tags on hub screens as usual (they are ordinary tags).

There is no `hub:` section in `config.yaml`; sites live in the hub's database
and are added in the web client or through the API.

## Check it works

1. **Hub** shows the site **Connected, N tags** with a recent Last Seen
   (`GET /api/v1/hub/status`: `"connected": true`, `"tags": N`).
2. After **Import tags**, `GET /api/v1/data` on the hub lists
   `<site>:default.<tag>` with live values and quality `good`.
3. Stop the site: within seconds **Hub** shows the retry reason and the tags
   turn `offline`. Start it: they return to `good` without action on the hub.

Verified on 2026-11-16 with a hub and two sites (each polling a Modbus
simulator): both connected (3 and 2 tags), import created 2 + 2 hub tags (one
mapped by hand before was skipped), live values matched, history rows were
stored on the hub for both sites, a third site with a wrong key showed
`site refused the API key: Invalid or expired token`, and killing and
restarting a site turned its tags offline and back.

## Troubleshooting

| Status or error | Cause and fix |
| --- | --- |
| `site refused the API key: Invalid or expired token` | Wrong, revoked or expired key; create a new one on the site and edit the hub site. |
| `IO error: Connection refused` | The site server is down or the port is wrong. |
| `invalid peer certificate` | The site's TLS certificate is not from a public CA; use a public certificate or connect over a VPN with `http://`. |
| Connected but no values on the hub | Run **Import tags**, or create tags reading device `<site>:default`. |
| Two sites with the same tag names | Each site has its own prefix (`<site>:default.`), so they do not clash. Keep site names unique. |
| Hub links stop after a hot standby failover | Only the active node runs the links; they start on the node that takes over. |

The native console shows hub tags like any tag but has no Hub screen; manage
sites in the web client.

## Fixed in the audit of 2026-11-16

Before: the hub never connected to any site (its WebSocket request lacked the
handshake headers and every site refused it), the Hub screen always showed
"Disconnected" (it read a response shape the server never sent), a new site
did not connect until a restart, connection errors were invisible, values of a
lost site stayed `good` forever, and remote tags could only be shown by
creating hub tags through the API by hand.
