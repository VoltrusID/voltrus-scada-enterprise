# White-label branding

Show your own name, logo, colours and font instead of Voltrus, on the login page and
throughout the web client. Needs an **Integrator** or **Enterprise** licence (flags
`white_label` and `theme_full`; see [licensing.md](licensing.md)). Keys from the
retired Starter/Edge plans may change the logo only.

| Setting | Where it shows | Limits |
|---|---|---|
| Title | Login page heading, header next to the logo, browser tab title | 200 characters, HTML stripped |
| Subtitle | Under the title on the login page, beside it in the header (desktop) | 200 characters, HTML stripped |
| Logo | Login page, header, browser tab icon | URL, or an uploaded image of at most 100 KB |
| Accent colour | The login page's Sign In button; highlights in the app: active tabs, links, Save and other accent-coloured controls | `#rgb` or `#rrggbb`; empty = Voltrus green |
| Dark and light theme colours | Background, surfaces (cards, dialogs, header), text, and an accent per theme; everywhere, login page included | `#rrggbb`, contrast-checked (below) |
| Font | All interface text, login page included | A fixed list of system fonts, or one uploaded WOFF2/WOFF/TTF/OTF file of at most 256 KB |

The user still chooses dark or light (or system) as before; the theme sets the colours
of each. Layouts are not part of the theme.

## Set it in the web UI

Needs an `admin` account.

1. **More actions (⋯) › Data Manager › Admin › Branding** (direct link:
   `https://<server>/data?tab=branding`).
2. Fill in:
   * **Title**, **Subtitle**, **Logo** (paste an `https://` URL, or **Upload**: PNG,
     SVG, JPEG or WebP, at most 100 KB, stored inside Voltrus).
   * **Accent colour (both themes)**: your brand colour. Where it does not reach 3:1
     against a theme's background (a light yellow on white, a dark blue on near-black),
     that theme shows it darkened or lightened just enough.
   * **Dark theme** and **Light theme**: Background, Surface, Text and an optional
     Accent of their own. **Default** keeps the Voltrus colour. Under each theme the
     contrast checks show the ratio of text and accent on background and surfaces.
   * **Font**: one of the system fonts, or **Upload font** to add your own file and
     select it (**Remove** deletes it and falls back to the default).
3. The **Live preview** shows the header, a card, text, a link, an input and a
   button in both themes as you type.
4. **Save** (it stays disabled while a check fails). Everyone sees the new branding at
   their next page load, the login page included.

The quick editor in the header (click **Edit**, then the logo or title) still changes
the title, subtitle, logo and accent; it links here for the rest.

Prefer an uploaded logo and a system font or uploaded font for plants without
internet access: nothing in the theme is loaded from another site.

### Contrast rules

Checked in the browser while you edit and again by the server, which refuses a theme
that fails (`400` with the ratio, nothing is saved):

* text on the background and text on surfaces: at least **4.5:1** (WCAG AA);
* a theme's own accent on the background and on surfaces: at least **3:1**
  (WCAG 1.4.11; the accent marks controls, active tabs and links);
* dimmed text (labels, hints) is derived from your text and background and kept at
  4.5:1 or more.

Status colours (alarm red, warning amber, OK green) are never themed: they keep their
meaning on every site.

## Set it through the API

`GET /api/v1/settings/branding` is public (the login page reads it).
`PUT /api/v1/settings/branding` needs a session or API key with the `admin`, `demo`
or `integration` role. Send only the fields you change:

```sh
curl -s -b cookies.txt -X PUT -H 'Content-Type: application/json' -d '{
  "title": "Acme Water", "subtitle": "Plant 3 HMI", "accent": "#0055aa",
  "logo": "https://cdn.acme.example/logo.svg",
  "theme": {
    "dark":  {"background": "#0b1220", "surface": "#131c2e", "text": "#e6edf7", "accent": "#4ea1ff"},
    "light": {"background": "#ffffff", "surface": "#f3f6fa", "text": "#0b1220"},
    "font": "humanist"
  }}' https://scada.example.com/api/v1/settings/branding
# {"ok":true}
```

`theme` replaces the whole stored theme (`null` resets it); every colour is optional
(empty = Voltrus). `font` is one of `system`, `sans`, `humanist`, `rounded`, `serif`,
`mono`, or `custom` (after an upload). Unknown keys are refused.

Upload a font (it is selected at once), or remove it:

```sh
curl -s -b cookies.txt -X PUT -H 'Content-Type: application/json' \
  -d "{\"name\":\"Acme Sans\",\"data\":\"$(base64 -w0 AcmeSans.woff2)\"}" \
  https://scada.example.com/api/v1/settings/branding/font
# {"ok":true,"font_url":"/api/v1/settings/branding/font?v=3f9c…","mime":"font/woff2"}
curl -s -b cookies.txt -X DELETE https://scada.example.com/api/v1/settings/branding/font
```

The file is accepted only if its first bytes are a WOFF2, WOFF, TrueType or OpenType
signature, and served back only with that font type (`X-Content-Type-Options:
nosniff`), from the same server, so the default content security policy
(`font-src 'self'`) covers it.

`GET` answers `theme_level` `full` (Integrator/Enterprise: everything above, plus
`theme`, `font_url`, `font_name` and the list of `fonts`), `basic` (logo only) or
`none` (no white-label: the Voltrus defaults are returned and your stored values are
kept for when the licence is back). Every change is written to the audit log
(`update_branding`, `update_branding_font`, `delete_branding_font`).

## How it is applied

The web client writes the theme into the CSS variables of the core client
(`--bg`, `--card`, `--text`, `--text-dim`, `--text-muted`, `--border`, `--cyan`,
`--font-ui` and a few derived ones) in a `<style>` element, one rule per theme
(`html[data-theme="dark"]`, `html[data-theme="light"]`). Every value written there is
a checked `#rrggbb` colour, one of the fixed font stacks, or the server's own font
URL. The core reads the accent for the Sign In button and the title and logo for the
browser tab itself.

## What is not branded

* The native desktop console (`voltrus-native`) always shows the Voltrus name, logo
  and colours.
* Alarm emails, PDF reports and the API reference page.
* Charts and HMI symbols use their own palettes (status colours, process graphics).
* Layouts: the arrangement of screens is the same for every site.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| The Branding tab shows "Requires the Integrator licence" | No valid Integrator/Enterprise licence; check **Data Manager › Admin › License**. |
| "Full theme customization needs an Integrator or Enterprise licence." | A logo-only (Starter/Edge) key: only the logo can change. |
| `400 Light theme: contrast of text on the background (#aaaaaa on #ffffff) is 2.32:1, it must be at least 4.5:1` | Pick a darker text or lighter background; the Branding tab shows each ratio. |
| "Logo is too large" (400) | The image file is over 100 KB. An SVG or a 128 px PNG is plenty, or use a URL. |
| "Not a font file" / "Font file is too large" (400) | Upload a WOFF2, WOFF, TTF or OTF file of at most 256 KB. WOFF2 subsets of the Latin glyphs are usually 20 to 60 KB. |
| The logo is a broken image | The URL is not reachable from the operator's browser (offline plant, intranet-only CDN, mixed http on an https page). Upload the file instead. |
| A system font looks like the default | That font is not installed on the viewing computer; the stack falls back to the next one. Upload the font file for the same look everywhere. |
| Branding disappeared after the licence lapsed | Expected: the defaults show while the licence is not valid. Your settings come back with a valid licence. |
