import assert from 'node:assert/strict'
import { test } from 'node:test'
import { formatSmsTargets, parseSmsTargets, summarizeSmsTargets } from '../src/channels/smsConfig.ts'

test('parseSmsTargets tolerates invalid or non-array configs', () => {
  assert.deepEqual(parseSmsTargets('not json'), [])
  assert.deepEqual(parseSmsTargets('{"phone":"1"}'), [])
  assert.deepEqual(parseSmsTargets('[{"phone":"5551234567","carrier":"att"}]'), [{ phone: '5551234567', carrier: 'att' }])
})

test('formatSmsTargets drops blank rows', () => {
  assert.equal(
    formatSmsTargets([{ phone: '5551234567', carrier: 'att' }, { phone: '  ', carrier: 'verizon' }]),
    '[{"phone":"5551234567","carrier":"att"}]',
  )
})

test('summarizeSmsTargets describes none, one and many numbers', () => {
  assert.equal(summarizeSmsTargets('[]'), 'none')
  assert.equal(summarizeSmsTargets('[{"phone":"1","carrier":"att"}]'), '1 (att)')
  assert.equal(summarizeSmsTargets('[{"phone":"1","carrier":"att"},{"phone":"2","carrier":"att"}]'), '2 phone numbers')
})
