import assert from 'node:assert/strict'
import { test } from 'node:test'
import { auditExportUrl, dayToUnix, exportQuery, rangeProblem, signaturesExportUrl } from '../src/panels/exportUrl.ts'

test('a day is a whole day in the browser zone', () => {
  const from = dayToUnix('2026-11-16')!
  const to = dayToUnix('2026-11-16', true)!
  assert.equal(to - from, 86_399)
  assert.equal(new Date(from * 1000).getHours(), 0)
  assert.equal(dayToUnix(''), null)
  assert.equal(dayToUnix('16/11/2026'), null)
})

test('the query carries the range, the filters and the zone offset', () => {
  const q = new URLSearchParams(exportQuery({ from: '2026-11-01', to: '2026-11-16', username: ' alice ', action: '', search: 'batch' }, 420))
  assert.equal(q.get('from'), String(dayToUnix('2026-11-01')))
  assert.equal(q.get('to'), String(dayToUnix('2026-11-16', true)))
  assert.equal(q.get('username'), 'alice')
  assert.equal(q.get('action'), null)
  assert.equal(q.get('search'), 'batch')
  assert.equal(q.get('utc_offset_minutes'), '420')
})

test('the CSV and PDF buttons share one filter', () => {
  const f = { from: '2026-11-01', username: 'alice', action: 'login' }
  const csv = signaturesExportUrl('csv', f)
  const pdf = signaturesExportUrl('pdf', f)
  assert.ok(csv.startsWith('/api/v1/compliance/signatures/export?'))
  assert.ok(pdf.startsWith('/api/v1/compliance/signatures/export.pdf?'))
  assert.equal(csv.split('?')[1], pdf.split('?')[1])
  // Signatures have no action filter.
  assert.ok(!pdf.includes('action='))
  assert.ok(auditExportUrl('csv', f).startsWith('/api/v1/audit/export?'))
  assert.ok(auditExportUrl('pdf', f).startsWith('/api/v1/compliance/audit/export.pdf?'))
  assert.ok(auditExportUrl('pdf', f).includes('action=login'))
})

test('a range that ends before it starts is refused', () => {
  assert.equal(rangeProblem({ from: '2026-11-16', to: '2026-11-01' }), 'The end date is before the start date.')
  assert.equal(rangeProblem({ from: '2026-11-16', to: '2026-11-16' }), null)
  assert.equal(rangeProblem({}), null)
})
