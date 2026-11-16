import assert from 'node:assert/strict'
import { test } from 'node:test'
import { brandingPayload, isValidAccent } from '../src/panels/brandingPayload.ts'

const form = { title: 'Acme', subtitle: 'Plant 4', logo: 'https://example.com/logo.png', accent: '#0055aa' }

test('a full theme licence sends every branding field', () => {
  assert.deepEqual(brandingPayload(form, false), form)
})

test('a form without an accent resets it to the theme default', () => {
  const { accent: _, ...noAccent } = form
  assert.deepEqual(brandingPayload(noAccent, false), { ...noAccent, accent: '' })
})

test('a logo-only licence sends the logo and nothing else', () => {
  assert.deepEqual(brandingPayload(form, true), { logo: 'https://example.com/logo.png' })
})

test('only #rrggbb accents or an empty reset are valid', () => {
  for (const ok of ['', '#0055aa', '#A1B2C3']) assert.equal(isValidAccent(ok), true, ok)
  for (const bad of ['red', '#fff', '0055aa', '#0055aa;x', '#gggggg']) assert.equal(isValidAccent(bad), false, bad)
})
