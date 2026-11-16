import assert from 'node:assert/strict'
import { test } from 'node:test'
import { parseHubStatus } from '../src/queries/hubStatus.ts'

test('reads the server shape { sites: [{ id }] } and keys rows by site_id', () => {
  const rows = parseHubStatus({
    total: 1,
    sites: [{ id: 7, name: 'Plant A', running: true, connected: true, tags: 12, last_message_at: 100, last_seen: 90, last_error: null, device_id: 'Plant A:default' }],
  })
  assert.equal(rows.length, 1)
  assert.equal(rows[0].site_id, 7)
  assert.equal(rows[0].connected, true)
  assert.equal(rows[0].tags, 12)
  assert.equal(rows[0].device_id, 'Plant A:default')
})

test('tolerates older servers and empty answers', () => {
  assert.deepEqual(parseHubStatus(null), [])
  const [row] = parseHubStatus([{ site_id: 3, name: 'B', running: true, last_seen: null }])
  assert.equal(row.site_id, 3)
  assert.equal(row.connected, false)
  assert.equal(row.device_id, 'B:default')
})
