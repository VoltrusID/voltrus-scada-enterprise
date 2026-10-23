import assert from 'node:assert/strict'
import { test } from 'node:test'
import { brandingPayload } from '../src/panels/brandingPayload.ts'

const form = { title: 'Acme', subtitle: 'Plant 4', logo: 'https://example.com/logo.png' }

test('a full theme licence sends every branding field', () => {
  assert.deepEqual(brandingPayload(form, false), form)
})

test('a logo-only licence sends the logo and nothing else', () => {
  assert.deepEqual(brandingPayload(form, true), { logo: 'https://example.com/logo.png' })
})
