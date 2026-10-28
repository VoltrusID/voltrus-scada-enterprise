import assert from 'node:assert/strict'
import { test } from 'node:test'
import { describeHa } from '../src/components/haBannerModel.ts'
import type { HaStatus } from '../src/queries/ha.ts'

const base: HaStatus = {
  enabled: true, role: 'primary', active: true, node_id: 'node-a', epoch: 3,
  peer_id: 'node-b', peer_role: 'standby', peer_epoch: 3, peer_status: 'connected',
  sync_state: 'streaming', sync_progress: null, sync_lag_ms: 40, pending_changes: 0,
  last_sync_seq: 10, last_sync_at: null, sync_error: null, failover_ts: null, failover_reason: null,
  uptime_as_role_secs: 5, failover_after_ms: 3000, can_take_over: false, promotion_blocked: null,
  witness: null, link_auth_failures: 0,
}

test('hidden when HA is off', () => {
  assert.equal(describeHa({ ...base, enabled: false }), null)
})

test('a healthy pair is green, a lost standby is a warning', () => {
  assert.equal(describeHa(base)?.tone, 'ok')
  assert.equal(describeHa({ ...base, peer_status: 'disconnected' })?.tone, 'warn')
})

test('a standby shows what it is really doing', () => {
  const standby: HaStatus = { ...base, role: 'standby', active: false, can_take_over: true }
  assert.match(describeHa(standby)!.text, /follows node-b/)
  assert.match(describeHa({ ...standby, sync_state: 'full_sync', sync_progress: { tables_done: 2, tables_total: 9, rows: 50 } })!.text, /2\/9 tables/)
  assert.equal(describeHa({ ...standby, can_take_over: false })?.tone, 'warn')
  assert.equal(describeHa({ ...standby, sync_state: 'error', sync_error: 'boom' })?.tone, 'alert')
  assert.match(describeHa({ ...standby, peer_status: 'disconnected', promotion_blocked: 'witness unreachable' })!.text, /blocked: witness unreachable/)
})

test('a recent takeover is announced and dismissible, an old one is not', () => {
  const now = Date.parse('2026-10-29T12:00:00Z')
  const recent = describeHa({ ...base, failover_ts: '2026-10-29T11:00:00Z', failover_reason: 'primary silent' }, now)!
  assert.equal(recent.tone, 'alert')
  assert.equal(recent.dismissKey, '2026-10-29T11:00:00Z')
  assert.equal(describeHa({ ...base, failover_ts: '2026-10-20T11:00:00Z' }, now)?.tone, 'ok')
})

test('a fenced node says it has stopped', () => {
  const m = describeHa({ ...base, role: 'fenced', active: false })!
  assert.equal(m.tone, 'alert')
  assert.match(m.text, /stopped polling and writing/)
})
