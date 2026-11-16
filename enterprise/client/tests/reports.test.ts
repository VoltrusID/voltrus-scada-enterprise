import assert from 'node:assert/strict'
import { test } from 'node:test'
import { normalizeReport } from '../src/queries/reportModel.ts'

test('server rows (JSON text lists, Unix seconds) become arrays and ISO times', () => {
  const r = normalizeReport({
    id: 1, name: 'Daily', schedule: 'daily', enabled: true,
    device_ids: 'all', email_recipients: '["ops@example.com","qa@example.com"]',
    last_generated: 1_700_000_000, created_at: 1_699_000_000,
  })
  assert.deepEqual(r.device_ids, ['all'])
  assert.deepEqual(r.email_recipients, ['ops@example.com', 'qa@example.com'])
  assert.equal(r.last_generated, '2023-11-14T22:13:20.000Z')
  assert.equal(r.email_recipients.join('\n'), 'ops@example.com\nqa@example.com')
})

test('never run and bad text are safe', () => {
  const r = normalizeReport({ id: 2, name: 'W', schedule: 'weekly', enabled: false, device_ids: '["plc1"]', email_recipients: 'not json', last_generated: null, created_at: 0 })
  assert.equal(r.last_generated, null)
  assert.deepEqual(r.email_recipients, [])
  assert.deepEqual(r.device_ids, ['plc1'])
})
