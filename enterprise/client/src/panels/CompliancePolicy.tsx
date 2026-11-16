import { useEffect, useState } from 'react'
import { btnStyle, inputStyle, labelStyle, labelTextStyle } from '@voltrus/core/styles'
import { useComplianceConfig, useUpdateComplianceConfig } from '../queries/compliance'
import {
  type ComplianceConfig, type SignedActionName, type TagScope, type UnattendedException, type UnattendedOrigin,
  configProblem, DEFAULT_CONFIG, exceptionProblem, MEANING_SUGGESTIONS, parseTagIds, SIGNED_ACTIONS, signatureSummary, withDefaults,
} from './complianceModel'

const sectionHeader: React.CSSProperties = {
  fontSize: 13, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '.05em',
  color: 'var(--fg)', marginBottom: 16, paddingLeft: 12, borderLeft: '4px solid var(--cyan)',
}
const hint: React.CSSProperties = { fontSize: 12, color: 'var(--muted)', lineHeight: 1.5 }
const body: React.CSSProperties = { display: 'flex', flexDirection: 'column', gap: 16, paddingLeft: 12 }

function Toggle({ on, label, onChange }: { on: boolean; label: string; onChange: (v: boolean) => void }) {
  return (
    <button
      type="button" role="switch" aria-checked={on} aria-label={label} onClick={() => onChange(!on)}
      style={{
        width: 40, height: 22, borderRadius: 11, background: on ? 'var(--cyan)' : 'var(--border)',
        position: 'relative', cursor: 'pointer', border: 'none', transition: 'background .15s', flexShrink: 0,
      }}
    >
      <span style={{ width: 16, height: 16, borderRadius: '50%', background: '#fff', position: 'absolute', top: 3, left: on ? 21 : 3, transition: 'left .15s' }} />
    </button>
  )
}

function NumberField({ label, value, min, max, onChange, note }: {
  label: string; value: number; min: number; max: number; onChange: (n: number) => void; note?: string
}) {
  return (
    <label style={labelStyle}>
      <span style={labelTextStyle}>{label}</span>
      <input
        type="number" min={min} max={max} value={value} style={inputStyle}
        onChange={e => onChange(Math.min(max, Math.max(min, Math.round(Number(e.target.value) || 0))))}
      />
      {note && <span style={hint}>{note}</span>}
    </label>
  )
}

export function PolicyTab() {
  const { data, isLoading } = useComplianceConfig()
  const updateMut = useUpdateComplianceConfig()
  const [form, setForm] = useState<ComplianceConfig>(DEFAULT_CONFIG)
  const [tagText, setTagText] = useState('')
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    if (!data) return
    const c = withDefaults(data)
    setForm(c)
    setTagText(c.signatures.tag_ids.join(', '))
  }, [data])

  const set = (patch: Partial<ComplianceConfig>) => { setForm(f => ({ ...f, ...patch })); setSaved(false) }
  const setSig = (patch: Partial<ComplianceConfig['signatures']>) => set({ signatures: { ...form.signatures, ...patch } })
  const setRule = (key: SignedActionName, patch: Partial<{ enabled: boolean; meaning: string }>) =>
    setSig({ actions: { ...form.signatures.actions, [key]: { ...form.signatures.actions[key], ...patch } } })
  const setException = (i: number, patch: Partial<UnattendedException>) =>
    setSig({ unattended_exceptions: form.signatures.unattended_exceptions.map((e, j) => (j === i ? { ...e, ...patch } : e)) })

  const tagIds = parseTagIds(tagText)
  const problem = tagIds === null ? 'Tag ids are numbers separated by commas.' : configProblem({ ...form, signatures: { ...form.signatures, tag_ids: tagIds } })

  const save = () => {
    if (problem || tagIds === null) return
    const { enforced: _enforced, ...payload } = { ...form, signatures: { ...form.signatures, tag_ids: tagIds } }
    updateMut.mutate(payload, {
      onSuccess: () => { setSaved(true); setTimeout(() => setSaved(false), 2500) },
    })
  }

  if (isLoading) return <div style={{ color: 'var(--muted)', textAlign: 'center', padding: 40 }}>Loading...</div>

  const summary = signatureSummary(form)

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 32, maxWidth: 620 }}>
      <p role="note" style={{ ...hint, color: data?.enforced ? 'var(--muted)' : 'var(--warn)' }}>
        {data?.enforced
          ? 'The server enforces these settings at sign-in, on every password change and before each signed action.'
          : 'Nothing is enforced until you save these settings once. Until then sign-in, passwords and actions follow the core defaults.'}
      </p>

      <section>
        <h3 style={sectionHeader}>Session</h3>
        <div style={body}>
          <NumberField label="Auto-logout after inactivity (minutes)" value={form.auto_logout_minutes} min={1} max={1440}
            onChange={n => set({ auto_logout_minutes: n })}
            note="Keyboard, mouse and touch count as activity; screens that only refresh do not." />
          <NumberField label="Maximum session length (hours)" value={form.session_max_hours} min={0} max={744}
            onChange={n => set({ session_max_hours: n })}
            note="Signed out this long after signing in, active or not. 0: no limit." />
        </div>
      </section>

      <section>
        <h3 style={sectionHeader}>Password Policy</h3>
        <div style={body}>
          <NumberField label="Minimum length" value={form.password_min_length} min={8} max={128} onChange={n => set({ password_min_length: n })} />
          {([
            ['password_require_uppercase', 'Require an uppercase letter'],
            ['password_require_number', 'Require a number'],
            ['password_require_special', 'Require a special character'],
          ] as const).map(([key, label]) => (
            <div key={key} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 13 }}>{label}</span>
              <Toggle on={form[key]} label={label} onChange={v => set({ [key]: v } as Partial<ComplianceConfig>)} />
            </div>
          ))}
          <NumberField label="Password history" value={form.password_history} min={0} max={24}
            onChange={n => set({ password_history: n })} note="A new password cannot be one of the last N. 0: no history." />
          <NumberField label="Password expires after (days)" value={form.password_rotation_days} min={0} max={3650}
            onChange={n => set({ password_rotation_days: n })}
            note="At sign-in after this many days the user must choose a new password. 0: never." />
          <span style={hint}>A password can never be the same as the user name.</span>
        </div>
      </section>

      <section>
        <h3 style={sectionHeader}>Electronic signatures</h3>
        <div style={body}>
          <span style={hint}>
            Actions switched on here ask the person for their password, a reason and the meaning below before they run.
            The signature is stored with the e-signature records and linked to the audit trail.
          </span>
          {SIGNED_ACTIONS.map(a => {
            const rule = form.signatures.actions[a.key]
            return (
              <div key={a.key} style={{ display: 'flex', flexDirection: 'column', gap: 6, paddingBottom: 8, borderBottom: '1px solid var(--border)' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
                  <span style={{ fontSize: 13 }}>{a.label}{a.hint && <span style={{ ...hint, display: 'block' }}>{a.hint}</span>}</span>
                  <Toggle on={rule.enabled} label={`Sign ${a.label}`} onChange={v => setRule(a.key, { enabled: v })} />
                </div>
                {rule.enabled && (
                  <label style={labelStyle}>
                    <span style={labelTextStyle}>Meaning</span>
                    <input list="part11-meanings" value={rule.meaning} maxLength={80} style={inputStyle}
                      onChange={e => setRule(a.key, { meaning: e.target.value })} />
                  </label>
                )}
                {rule.enabled && a.key === 'tag_write' && (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                    <label style={labelStyle}>
                      <span style={labelTextStyle}>Which tags</span>
                      <select value={form.signatures.tag_scope} style={inputStyle}
                        onChange={e => setSig({ tag_scope: e.target.value as TagScope })}>
                        <option value="all">Every writable tag</option>
                        <option value="requires_reason">Tags whose write policy requires a reason</option>
                        <option value="listed">Only the tags listed</option>
                      </select>
                    </label>
                    {form.signatures.tag_scope === 'listed' && (
                      <label style={labelStyle}>
                        <span style={labelTextStyle}>Tag ids</span>
                        <input value={tagText} placeholder="12, 15, 40" style={inputStyle}
                          onChange={e => { setTagText(e.target.value); setSaved(false) }} />
                      </label>
                    )}
                  </div>
                )}
              </div>
            )
          })}
          <datalist id="part11-meanings">{MEANING_SUGGESTIONS.map(m => <option key={m} value={m} />)}</datalist>
        </div>
      </section>

      <section>
        <h3 style={sectionHeader}>Unattended callers</h3>
        <div style={body}>
          <span style={hint}>
            API keys, MCP (AI agents) and flows cannot sign. When an action needs a signature they are refused,
            unless they are listed here. List only callers whose behaviour your validation covers.
          </span>
          {form.signatures.unattended_exceptions.map((e, i) => {
            const p = exceptionProblem(e)
            return (
              <div key={i} style={{ display: 'grid', gridTemplateColumns: '110px 1fr 1fr auto', gap: 8, alignItems: 'start' }}>
                <select aria-label="Kind of caller" value={e.origin} style={inputStyle}
                  onChange={ev => setException(i, { origin: ev.target.value as UnattendedOrigin })}>
                  <option value="api_key">API key</option>
                  <option value="flow">Flow</option>
                  <option value="mcp">MCP</option>
                </select>
                <div>
                  <input aria-label="Caller" value={e.principal} style={inputStyle}
                    placeholder={e.origin === 'api_key' ? 'api_key:3' : e.origin === 'flow' ? 'flow:12' : 'user name or api_key:3'}
                    onChange={ev => setException(i, { principal: ev.target.value })} />
                  {p && e.principal && <span style={{ fontSize: 11, color: 'var(--crit)' }}>{p}</span>}
                </div>
                <input aria-label="Why it is allowed" value={e.note} placeholder="Why (validation reference)" style={inputStyle}
                  onChange={ev => setException(i, { note: ev.target.value })} />
                <button type="button" style={btnStyle} aria-label="Remove exception"
                  onClick={() => setSig({ unattended_exceptions: form.signatures.unattended_exceptions.filter((_, j) => j !== i) })}>Remove</button>
              </div>
            )
          })}
          <div>
            <button type="button" style={btnStyle}
              onClick={() => setSig({ unattended_exceptions: [...form.signatures.unattended_exceptions, { origin: 'api_key', principal: '', note: '' }] })}>
              Add exception
            </button>
          </div>
        </div>
      </section>

      {summary.length > 0 && (
        <section>
          <h3 style={sectionHeader}>What operators will see</h3>
          <ul style={{ ...hint, paddingLeft: 28, margin: 0 }}>{summary.map(s => <li key={s}>{s}</li>)}</ul>
        </section>
      )}

      <div style={{ display: 'flex', gap: 8, alignItems: 'center', paddingLeft: 12, flexWrap: 'wrap' }}>
        <button style={{ ...btnStyle, borderColor: 'var(--cyan)', color: 'var(--cyan)', fontWeight: 600 }}
          onClick={save} disabled={updateMut.isPending || !!problem}>
          {updateMut.isPending ? 'Saving...' : 'Save policy'}
        </button>
        {problem && <span style={{ fontSize: 12, color: 'var(--warn)' }}>{problem}</span>}
        {updateMut.error && <span style={{ fontSize: 12, color: 'var(--crit)' }}>{updateMut.error.message}</span>}
        {saved && <span style={{ fontSize: 12, color: 'var(--ok)' }}>Policy saved and enforced</span>}
      </div>
    </div>
  )
}
