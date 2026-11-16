/**
 * The white-label theme in the browser: contrast checks (the same rules the server
 * enforces in enterprise/src/branding/theme.rs), the font stacks, and the CSS that
 * puts a theme into the core client's variables (--bg, --card, --text, --cyan,
 * --font-ui ...). Pure functions, tested in tests/theme.test.ts.
 */

export interface Palette {
  background?: string
  surface?: string
  text?: string
  accent?: string
}

export interface Theme {
  dark?: Palette
  light?: Palette
  /** One of FONT_STACKS' ids; empty = system. */
  font?: string
}

export type Mode = 'dark' | 'light'

/** Minimum contrast of text on the background and on surfaces (WCAG AA). */
export const TEXT_CONTRAST = 4.5
/** Minimum contrast of the accent on the background and on surfaces (WCAG 1.4.11). */
export const ACCENT_CONTRAST = 3

/** The Voltrus colours of core's index.css, which apply where a theme is empty. */
export const DEFAULTS: Record<Mode, Required<Palette>> = {
  dark: { background: '#09090b', surface: '#111113', text: '#e4e4e7', accent: '#22c55e' },
  light: { background: '#ffffff', surface: '#f9fafb', text: '#111827', accent: '#16a34a' },
}

/** System font stacks: nothing is loaded from the network. `custom` is the uploaded file. */
export const FONT_STACKS: Record<string, { label: string; stack: string }> = {
  system: { label: 'Voltrus default (system UI)', stack: "-apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif" },
  sans: { label: 'Helvetica / Arial', stack: "'Helvetica Neue', Helvetica, Arial, 'Liberation Sans', sans-serif" },
  humanist: { label: 'Segoe UI / Noto Sans', stack: "'Segoe UI', 'Noto Sans', Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif" },
  rounded: { label: 'Rounded', stack: "ui-rounded, 'SF Pro Rounded', 'Arial Rounded MT', 'Arial Rounded MT Bold', 'Varela Round', Nunito, Quicksand, system-ui, sans-serif" },
  serif: { label: 'Serif (Charter / Georgia)', stack: "Charter, 'Bitstream Charter', 'Sitka Text', Cambria, Georgia, 'Times New Roman', serif" },
  mono: { label: 'Monospace', stack: "ui-monospace, 'Cascadia Code', 'Source Code Pro', Menlo, Consolas, 'DejaVu Sans Mono', monospace" },
  custom: { label: 'Uploaded font', stack: "'VoltrusBrandFont', -apple-system, BlinkMacSystemFont, system-ui, sans-serif" },
}

/** Largest uploaded font file (the server's limit too). */
export const MAX_FONT_FILE_BYTES = 256 * 1024

const HEX = /^#(?:[0-9a-f]{3}|[0-9a-f]{6})$/i

/** `#rgb` / `#rrggbb` as lower-case `#rrggbb`; '' for empty; null when invalid. */
export function normalizeColour(value: string | undefined): string | null {
  const v = (value ?? '').trim()
  if (!v) return ''
  if (!HEX.test(v)) return null
  const h = v.slice(1).toLowerCase()
  return '#' + (h.length === 3 ? h.split('').map(c => c + c).join('') : h)
}

function channel(hex: string, i: number): number {
  const c = parseInt(hex.slice(1 + i * 2, 3 + i * 2), 16) / 255
  return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4
}

export function luminance(hex: string): number {
  return 0.2126 * channel(hex, 0) + 0.7152 * channel(hex, 1) + 0.0722 * channel(hex, 2)
}

/** WCAG contrast ratio of two `#rrggbb` colours, 1 to 21. */
export function contrast(a: string, b: string): number {
  const [la, lb] = [luminance(a), luminance(b)]
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05)
}

/**
 * The brand accent as it is painted in one mode: unchanged when it reads at 3:1 on
 * the background and on surfaces, else darkened (light mode) or lightened (dark mode)
 * just enough. One brand colour rarely reads on both white and near-black.
 */
export function readableAccent(accent: string, background: string, surface: string): string {
  const ok = (c: string) => contrast(c, background) >= ACCENT_CONTRAST && contrast(c, surface) >= ACCENT_CONTRAST
  if (ok(accent)) return accent
  const target = contrast('#000000', background) >= contrast('#ffffff', background) ? '#000000' : '#ffffff'
  for (let w = 0.95; w > 0; w -= 0.05) {
    const c = mix(accent, target, w)
    if (ok(c)) return c
  }
  return target
}

/**
 * The colours a mode ends up with: the theme's, else Voltrus. The accent is the
 * theme's own (checked, never adjusted), else the brand accent made readable in
 * this mode, else the Voltrus green.
 */
export function effectivePalette(theme: Theme | undefined, mode: Mode, brandAccent = ''): Required<Palette> {
  const p = theme?.[mode] ?? {}
  const pick = (v: string | undefined, d: string) => normalizeColour(v) || d
  const background = pick(p.background, DEFAULTS[mode].background)
  const surface = pick(p.surface, DEFAULTS[mode].surface)
  const own = normalizeColour(p.accent)
  const accent = own || readableAccent(pick(brandAccent, DEFAULTS[mode].accent), background, surface)
  return { background, surface, text: pick(p.text, DEFAULTS[mode].text), accent }
}

export interface ContrastCheck {
  mode: Mode
  what: string
  fg: string
  bg: string
  ratio: number
  min: number
  ok: boolean
}

/** The four contrast checks of one mode. */
export function contrastChecks(theme: Theme | undefined, mode: Mode, brandAccent = ''): ContrastCheck[] {
  const p = effectivePalette(theme, mode, brandAccent)
  return ([
    ['Text on background', p.text, p.background, TEXT_CONTRAST],
    ['Text on surfaces', p.text, p.surface, TEXT_CONTRAST],
    ['Accent on background', p.accent, p.background, ACCENT_CONTRAST],
    ['Accent on surfaces', p.accent, p.surface, ACCENT_CONTRAST],
  ] as const).map(([what, fg, bg, min]) => {
    const ratio = contrast(fg, bg)
    return { mode, what, fg, bg, ratio, min, ok: ratio + 1e-9 >= min }
  })
}

/** Why the theme cannot be saved, or null. Mirrors the server's check. */
export function themeProblem(theme: Theme | undefined, brandAccent = ''): string | null {
  if (normalizeColour(brandAccent) === null) return 'The accent colour must look like #0055aa.'
  for (const mode of ['dark', 'light'] as const) {
    for (const key of ['background', 'surface', 'text', 'accent'] as const) {
      if (normalizeColour(theme?.[mode]?.[key]) === null) return `${mode === 'dark' ? 'Dark' : 'Light'} theme, ${key}: use a colour like #0055aa.`
    }
    const bad = contrastChecks(theme, mode, brandAccent).find(c => !c.ok)
    if (bad) return `${mode === 'dark' ? 'Dark' : 'Light'} theme: ${bad.what.toLowerCase()} is ${bad.ratio.toFixed(2)}:1, at least ${bad.min}:1 is needed.`
  }
  if (theme?.font && !(theme.font in FONT_STACKS)) return 'Unknown font.'
  return null
}

/** Mix two colours in sRGB, like CSS color-mix(in srgb, a w, b). */
export function mix(a: string, b: string, weight: number): string {
  const part = (h: string, i: number) => parseInt(h.slice(1 + i * 2, 3 + i * 2), 16)
  return '#' + [0, 1, 2].map(i => Math.round(part(a, i) * weight + part(b, i) * (1 - weight)).toString(16).padStart(2, '0')).join('')
}

/**
 * The dimmed text colours for a custom text/background pair: as faint as the
 * Voltrus theme makes them, but never below 4.5:1 on the surface.
 */
export function dimmedText(p: Required<Palette>): { dim: string; muted: string } {
  const readable = (start: number) => {
    for (let w = start; w < 1; w += 0.05) {
      const c = mix(p.text, p.background, w)
      if (contrast(c, p.surface) >= TEXT_CONTRAST && contrast(c, p.background) >= TEXT_CONTRAST) return c
    }
    return p.text
  }
  return { dim: readable(0.75), muted: readable(0.6) }
}

/**
 * CSS declarations of one mode: the variables the theme changes, or with `full`
 * every themed variable (for a preview that must not inherit the page's mode).
 */
export function modeDeclarations(theme: Theme | undefined, mode: Mode, brandAccent = '', full = false): string[] {
  const own = theme?.[mode] ?? {}
  const p = effectivePalette(theme, mode, brandAccent)
  const set = (v: string | undefined) => full || !!normalizeColour(v)
  const out: string[] = []
  if (set(own.background)) out.push(`--bg: ${p.background}`, `--input-bg: ${p.background}`)
  if (set(own.surface)) out.push(`--card: ${p.surface}`, `--code-bg: ${p.surface}`)
  if (set(own.text)) out.push(`--text: ${p.text}`, `--fg: ${p.text}`)
  if (full || normalizeColour(own.text) || normalizeColour(own.background) || normalizeColour(own.surface)) {
    const { dim, muted } = dimmedText(p)
    out.push(`--text-dim: ${dim}`, `--text-muted: ${muted}`, `--muted: ${muted}`, `--border: ${mix(p.text, p.background, 0.16)}`, `--border-light: ${mix(p.text, p.background, 0.26)}`)
  }
  if (full || normalizeColour(own.accent) || normalizeColour(brandAccent)) out.push(`--cyan: ${p.accent}`)
  return out
}

export interface BrandTheme {
  accent?: string
  theme?: Theme
  font_url?: string | null
}

/**
 * The stylesheet for a brand: both modes (selected by core's data-theme on <html>)
 * and the font. Every value is a validated colour, a fixed font stack or a
 * same-origin URL, so nothing from the server is pasted into CSS unchecked.
 * Returns '' when the brand changes nothing.
 */
export function themeCss(brand: BrandTheme | null | undefined, scope = 'html', full = false): string {
  if (!brand) return ''
  const accent = normalizeColour(brand.accent) ?? ''
  const theme = brand.theme
  const rules: string[] = []
  // `html[data-theme=...]` outranks core's `[data-theme=...]` rules.
  for (const mode of ['dark', 'light'] as const) {
    const decl = modeDeclarations(theme, mode, accent, full)
    if (decl.length) rules.push(`${scope}[data-theme="${mode}"] { ${decl.join('; ')}; }`)
  }
  const font = theme?.font && FONT_STACKS[theme.font] ? theme.font : ''
  const fontUrl = brand.font_url && FONT_URL.test(brand.font_url) ? brand.font_url : ''
  if (font === 'custom' && fontUrl) {
    rules.unshift(`@font-face { font-family: 'VoltrusBrandFont'; src: url('${fontUrl}'); font-display: swap; }`)
    rules.push(`${scope}[data-theme] { --font-ui: ${FONT_STACKS.custom.stack}; }`)
  } else if (font && font !== 'custom' && font !== 'system') {
    rules.push(`${scope}[data-theme] { --font-ui: ${FONT_STACKS[font].stack}; }`)
  }
  return rules.join('\n')
}

/** The only font URL a theme may load: the server's own, versioned by hash. */
const FONT_URL = /^\/api\/v1\/settings\/branding\/font\?v=[0-9a-f]{1,64}$/
