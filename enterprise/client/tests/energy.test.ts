import assert from 'node:assert/strict'
import { test } from 'node:test'
import {
  dailyScale, deviceTags, excessFactor, formatRupiah, metricName, numOrNull, showUnconfirmedNote,
} from '../src/energy/model.ts'

test('Rupiah uses the language thousands separator', () => {
  assert.equal(formatRupiah(86_949_720, 'id'), 'Rp 86.949.720')
  assert.equal(formatRupiah(86_949_720.4, 'en'), 'Rp 86,949,720')
  assert.equal(formatRupiah(-1500, 'id-ID'), 'Rp -1.500')
  assert.equal(formatRupiah(null, 'id'), '-')
})

test('the default excess factor is tan(acos 0.85)', () => {
  assert.ok(Math.abs(excessFactor(0.85) - 0.6197) < 1e-4)
})

test('the to-be-confirmed note stays until a month reconciles', () => {
  assert.equal(showUnconfirmedNote({ confirmed: false }, { confirmed: false }), true)
  assert.equal(showUnconfirmedNote(null, null), true)
  assert.equal(showUnconfirmedNote({ confirmed: true }, null), false)
  assert.equal(showUnconfirmedNote({ confirmed: false }, { confirmed: true }), false)
})

test('form numbers accept a decimal comma and blank is null', () => {
  assert.equal(numOrNull('1035,78'), 1035.78)
  assert.equal(numOrNull('  '), null)
  assert.equal(numOrNull('abc'), null)
})

test('data source keys become historian metrics per device', () => {
  assert.equal(metricName('pm5560.Energy_Import', 'pm5560'), 'Energy_Import')
  assert.equal(metricName('Energy_Import', 'pm5560'), 'Energy_Import')
  const tags = deviceTags([
    { key: 'pm.kWh', device_id: 'pm', name: 'Energy' },
    { key: 'pm.kVArh', device_id: 'pm', name: 'Reactive' },
    { key: 'calc', device_id: '', name: 'Virtual' },
  ])
  assert.deepEqual([...tags.keys()], ['pm'])
  assert.deepEqual(tags.get('pm')?.map(t => t.metric), ['kVArh', 'kWh'])
})

test('the daily chart axis rounds up to 1, 2 or 5 × 10^n', () => {
  assert.equal(dailyScale([{ wbp_premium: 1_657_248, kvarh_charge: 2_898_324 }]), 5_000_000)
  assert.equal(dailyScale([{ wbp_premium: 120, kvarh_charge: null }]), 200)
  assert.equal(dailyScale([]), 1)
})
