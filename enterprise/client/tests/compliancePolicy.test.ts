import assert from 'node:assert/strict'
import { test } from 'node:test'
import {
  configProblem, DEFAULT_CONFIG, exceptionProblem, parseTagIds, signatureSummary, withDefaults,
} from '../src/panels/complianceModel.ts'

test('signatures are off by default with the usual meanings', () => {
  const c = withDefaults({})
  assert.equal(c.signatures.actions.tag_write.meaning, 'Performed')
  assert.equal(c.signatures.actions.alarm_ack.meaning, 'Reviewed')
  assert.equal(c.signatures.actions.recipe_load.meaning, 'Approved')
  assert.ok(Object.values(c.signatures.actions).every(r => !r.enabled))
  assert.deepEqual(signatureSummary(c), [])
})

test('a reply from an older server fills the missing fields', () => {
  const c = withDefaults({ auto_logout_minutes: 10, password_min_length: 12 } as never)
  assert.equal(c.auto_logout_minutes, 10)
  assert.equal(c.password_history, 5)
  assert.equal(c.session_max_hours, 12)
  assert.deepEqual(c.signatures.unattended_exceptions, [])
})

test('tag ids are parsed and checked', () => {
  assert.deepEqual(parseTagIds('15, 12 40;12'), [12, 15, 40])
  assert.deepEqual(parseTagIds(''), [])
  assert.equal(parseTagIds('12, abc'), null)
  assert.equal(parseTagIds('0'), null)
})

test('exceptions name one caller of their kind', () => {
  assert.equal(exceptionProblem({ origin: 'api_key', principal: 'api_key:3', note: '' }), null)
  assert.equal(exceptionProblem({ origin: 'flow', principal: '*', note: '' }), null)
  assert.equal(exceptionProblem({ origin: 'mcp', principal: 'alice', note: '' }), null)
  assert.match(exceptionProblem({ origin: 'flow', principal: 'api_key:3', note: '' })!, /flow:<id>/)
  assert.ok(exceptionProblem({ origin: 'api_key', principal: '', note: '' }))
  assert.ok(exceptionProblem({ origin: 'mcp', principal: 'two words', note: '' }))
})

test('the form explains why it cannot be saved', () => {
  assert.equal(configProblem(DEFAULT_CONFIG), null)
  assert.match(configProblem({ ...DEFAULT_CONFIG, password_min_length: 6 })!, /at least 8/)
  const listedNone = withDefaults({
    signatures: { ...DEFAULT_CONFIG.signatures, tag_scope: 'listed', actions: { ...DEFAULT_CONFIG.signatures.actions, tag_write: { enabled: true, meaning: 'Performed' } } },
  })
  assert.match(configProblem(listedNone)!, /at least one tag/)
  const noMeaning = withDefaults({
    signatures: { ...DEFAULT_CONFIG.signatures, actions: { ...DEFAULT_CONFIG.signatures.actions, alarm_ack: { enabled: true, meaning: ' ' } } },
  })
  assert.match(configProblem(noMeaning)!, /meaning cannot be empty/)
})

test('the summary says what a person will be asked', () => {
  const c = withDefaults({
    signatures: {
      ...DEFAULT_CONFIG.signatures,
      tag_scope: 'requires_reason',
      actions: { ...DEFAULT_CONFIG.signatures.actions, tag_write: { enabled: true, meaning: 'Performed' }, alarm_ack: { enabled: true, meaning: 'Reviewed' } },
    },
  })
  assert.deepEqual(signatureSummary(c), [
    'Tag writes (setpoints) (tags whose write policy requires a reason): password, reason, signed as "Performed"',
    'Acknowledge alarms: password, reason, signed as "Reviewed"',
  ])
})
