import assert from 'node:assert/strict'
import { test } from 'node:test'
import {
  contrast, contrastChecks, dimmedText, effectivePalette, FONT_STACKS, normalizeColour, readableAccent, themeCss, themeProblem,
} from '../src/panels/themeModel.ts'

test('contrast follows WCAG', () => {
  assert.equal(contrast('#000000', '#ffffff'), 21)
  assert.equal(contrast('#ffffff', '#ffffff'), 1)
  assert.ok(Math.abs(contrast('#777777', '#ffffff') - 4.48) < 0.01)
})

test('colours are normalised like the server does', () => {
  assert.equal(normalizeColour('#0AF'), '#00aaff')
  assert.equal(normalizeColour(' #0055AA '), '#0055aa')
  assert.equal(normalizeColour(''), '')
  for (const bad of ['red', '#0055aa;x', 'url(x)', '#12345']) assert.equal(normalizeColour(bad), null, bad)
})

test('the Voltrus defaults pass every check', () => {
  assert.equal(themeProblem({}, ''), null)
  assert.equal(themeProblem(undefined, '#22c55e'), null)
  for (const mode of ['dark', 'light'] as const) assert.ok(contrastChecks({}, mode).every(c => c.ok))
})

test('low contrast is refused with the ratio, as the server refuses it', () => {
  const problem = themeProblem({ light: { background: '#ffffff', text: '#aaaaaa' } })
  assert.match(problem ?? '', /Light theme: text on background is 2\.32:1, at least 4\.5:1/)
  // A theme's own accent is checked; the brand accent is adjusted per theme instead.
  assert.match(themeProblem({ light: { accent: '#ffee58' } }) ?? '', /Light theme: accent on background/)
  assert.equal(themeProblem({}, '#ffee58'), null)
  assert.equal(themeProblem({ light: { accent: '#8a6d00' } }, '#ffee58'), null)
  assert.match(themeProblem({ dark: { text: 'nope' } }) ?? '', /Dark theme, text/)
})

test('a theme only sets the variables it changes', () => {
  const css = themeCss({ accent: '#0055aa', theme: { light: { background: '#fdfdfd' } } })
  // #0055aa is too dark for the dark theme (2.7:1): it is lightened there.
  assert.match(css, /html\[data-theme="dark"\] \{ --cyan: #[0-9a-f]{6}; \}/)
  assert.doesNotMatch(css, /html\[data-theme="dark"\] \{ --cyan: #0055aa/)
  assert.match(css, /html\[data-theme="light"\] \{ --bg: #fdfdfd; --input-bg: #fdfdfd; .*--cyan: #0055aa; \}/)
  assert.doesNotMatch(css, /--card/)
  assert.equal(themeCss({}), '')
  assert.equal(themeCss(null), '')
})

test('derived dim text stays readable on a custom palette', () => {
  const p = effectivePalette({ dark: { background: '#101820', surface: '#1b2633', text: '#d8e2ee' } }, 'dark')
  const { dim, muted } = dimmedText(p)
  for (const c of [dim, muted]) {
    assert.ok(contrast(c, p.surface) >= 4.5, c)
    assert.ok(contrast(c, p.background) >= 4.5, c)
  }
})

test('fonts come from the fixed list or the server URL only', () => {
  const serif = themeCss({ theme: { font: 'serif' } })
  assert.ok(serif.includes(`html[data-theme] { --font-ui: ${FONT_STACKS.serif.stack}; }`))
  const custom = themeCss({ theme: { font: 'custom' }, font_url: '/api/v1/settings/branding/font?v=0a1b2c3d' })
  assert.match(custom, /@font-face \{ font-family: 'VoltrusBrandFont'; src: url\('\/api\/v1\/settings\/branding\/font\?v=0a1b2c3d'\)/)
  // Anything else is ignored: no remote CSS, no injection.
  for (const url of ['https://evil.example/font.woff2', "/api/v1/settings/branding/font?v=1'); } body { x: url('", '//x/font']) {
    assert.doesNotMatch(themeCss({ theme: { font: 'custom' }, font_url: url }), /@font-face/, url)
  }
  assert.equal(themeCss({ theme: { font: 'Comic Sans; } body {' } }), '')
  assert.equal(themeCss({ theme: { font: 'system' } }), '')
})

test('one brand accent is made readable in each theme', () => {
  for (const accent of ['#22c55e', '#0055aa', '#ffee58', '#ff0000']) {
    for (const mode of ['dark', 'light'] as const) {
      const p = effectivePalette({}, mode, accent)
      assert.ok(contrast(p.accent, p.background) >= 3, `${accent} ${mode} ${p.accent}`)
      assert.ok(contrast(p.accent, p.surface) >= 3, `${accent} ${mode} ${p.accent}`)
    }
  }
  // A colour that already reads is left alone.
  assert.equal(readableAccent('#0055aa', '#ffffff', '#f9fafb'), '#0055aa')
})

test('a preview gets every variable so it does not inherit the page theme', () => {
  const css = themeCss({ theme: {} }, '.preview', true)
  assert.match(css, /\.preview\[data-theme="dark"\] \{ --bg: #09090b;/)
  assert.match(css, /\.preview\[data-theme="light"\] \{ --bg: #ffffff;/)
})
