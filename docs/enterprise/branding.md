# White-label branding

Show your own name, logo and accent colour instead of Voltrus, on the login page and
in the header of the web client. Needs an **Integrator** or **Enterprise** licence
(flags `white_label` and `theme_full`; see [licensing.md](licensing.md)). Keys from the
retired Starter/Edge plans may change the logo only.

| Setting | Where it shows | Limits |
|---|---|---|
| Title | Login page heading, header next to the logo | 200 bytes, HTML stripped |
| Subtitle | Under the title on the login page, beside it in the header (desktop) | 200 bytes, HTML stripped |
| Logo | Login page and header (shown at 32 and 22 px, square works best) | URL, or an uploaded image of at most 36 KB |
| Accent colour | Highlights in the app: active tabs, links, Edit/Done and Save buttons and other accent-coloured controls; dark and light theme alike | `#rrggbb`, empty = Voltrus default |

## Set it in the web UI

Needs an `admin` account on a desktop-width browser window (the Edit button is hidden
on phones).

1. Sign in and click **Edit** in the header (top right). The title in the top-left
   corner gets a small pencil.
2. Click the **logo or title** (top left). **Branding Settings** opens.
3. Fill in:
   * **Title** and **Subtitle**.
   * **Accent colour**: pick a colour or type `#0055aa`; **Default** clears it.
   * **Logo**: **URL** tab, paste an `https://` address; or **Upload** tab ›
     **Choose File** (PNG, SVG, JPEG; at most 36 KB, it is stored inside Voltrus).
     **Reset** removes the logo. The preview shows what will be used.
4. Click **Save**, then **Done** to leave edit mode. The header changes at once; the
   login page shows the new branding to everyone at their next visit (sign out to see it).

Prefer an uploaded logo for plants without internet access: a URL logo is loaded by
every browser from that address.

Without a licence that includes white-label, the same click opens a notice
("White-label branding ... Requires the Integrator licence") and the server refuses
changes with 403.

## Set it through the API

`GET /api/v1/settings/branding` is public (the login page reads it).
`PUT /api/v1/settings/branding` needs a session or API key with the `admin`, `demo`
or `integration` role. Send only the fields you change:

```sh
curl -s -b cookies.txt -X PUT -H 'Content-Type: application/json' \
  -d '{"title":"Acme Water","subtitle":"Plant 3 HMI","accent":"#0055aa","logo":"https://cdn.acme.example/logo.svg"}' \
  https://scada.example.com/api/v1/settings/branding
# {"ok":true}

curl -s https://scada.example.com/api/v1/settings/branding
# {"title":"Acme Water","subtitle":"Plant 3 HMI","logo":"https://...","accent":"#0055aa","theme_level":"full"}
```

`theme_level` is `full` (Integrator/Enterprise), `basic` (logo only) or `none` (no
white-label: the Voltrus defaults are returned and your stored values are kept for
when the licence is back). An uploaded logo is a `data:image/...;base64,...` URL of at
most 50,000 bytes. Every change is written to the audit log (`update_branding`).

## What is not branded (yet)

* The browser tab title and the favicon stay "Voltrus".
* On the login page the **Sign In** button and the header tint stay Voltrus green; the
  accent colour applies after sign-in.
* There is no font or layout editor; the theme follows the user's dark/light choice.
* The native desktop console (`voltrus-native`) always shows the Voltrus name and
  logo.
* Alarm emails, PDF reports and the API reference page are outside the scope of this
  setting.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| Clicking the logo does nothing | You are not in edit mode (click **Edit** first), the window is phone-width, or the account is not an admin. |
| "White-label branding requires an Integrator or Enterprise licence" (403) | No valid Integrator/Enterprise licence; check **Data Manager › Admin › License**. |
| "Full theme customization needs an Integrator or Enterprise licence." | A logo-only (Starter/Edge) key: only the logo can change. |
| "File too large (max 36 KB)" or "Logo is too large" (400) | Shrink the image (an SVG or a 128 px PNG is plenty) or use a URL. |
| "accent must be a colour like #0055aa" (400) | Use six hex digits with `#`, or an empty string to reset. |
| The logo is a broken image | The URL is not reachable from the operator's browser (offline plant, intranet-only CDN, mixed http on an https page). Upload the file instead. |
| Branding disappeared after the licence lapsed | Expected: the defaults show while the licence is not valid. Your settings come back with a valid licence. |
