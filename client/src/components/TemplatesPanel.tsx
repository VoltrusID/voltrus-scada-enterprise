import { useState, useEffect, useCallback } from 'react'
import { fetchJson } from '../queries'

// ── Types ──

interface TemplateSummary {
  id: string; name: string; industry: string; description: string
  tag_count: number; alarm_count: number; flow_count: number; has_oee: boolean
}
interface IndustryInfo { id: string; name: string }
interface TemplateListResponse { templates: TemplateSummary[]; industries: IndustryInfo[] }

interface TemplateDetail {
  id: string; name: string; industry: string; description: string; equipment_type: string
  tags: { name_pattern: string; description: string; unit: string; data_type: string; group: string; widget: string }[]
  alarms: { tag_pattern: string; severity: string; comparison: string; threshold: number; message_template: string }[]
  flows: { name: string; description: string; trigger_type: string }[]
  oee_config: any
  dashboard: { screen_name: string }
}

// ── Component ──

export function TemplatesPanel() {
  const [tab, setTab] = useState<'browse' | 'detail' | 'instantiate'>('browse')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  const [templates, setTemplates] = useState<TemplateSummary[]>([])
  const [industries, setIndustries] = useState<IndustryInfo[]>([])
  const [selectedIndustry, setSelectedIndustry] = useState('')
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [detail, setDetail] = useState<TemplateDetail | null>(null)

  // Instantiate form
  const [instanceName, setInstanceName] = useState('')
  const [instanceCount, setInstanceCount] = useState('2')
  const [instantiating, setInstantiating] = useState(false)
  // Preview
  const [previewTags, setPreviewTags] = useState<any[]>([])
  const [previewAlarms, setPreviewAlarms] = useState<any[]>([])

  const load = useCallback(async () => {
    setLoading(true); setError('')
    try {
      const d = await fetchJson<TemplateListResponse>('/templates' + (selectedIndustry ? `?industry=${selectedIndustry}` : ''))
      setTemplates(d.templates || []); setIndustries(d.industries || [])
    } catch (e: any) { setError(e.message) }
    setLoading(false)
  }, [selectedIndustry])

  useEffect(() => { load() }, [load])

  const selectTemplate = async (id: string) => {
    setSelectedId(id); setTab('detail'); setLoading(true)
    try { setDetail(await fetchJson<TemplateDetail>(`/templates/${id}`)) } catch (e: any) { setError(e.message) }
    setLoading(false)
  }

  const preview = async () => {
    if (!selectedId) return
    setLoading(true)
    try {
      const r = await fetch(`/templates/preview`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ template_id: selectedId, instance_counts: { n: parseInt(instanceCount) || 2 } }),
        credentials: 'include',
      })
      const data = await r.json()
      if (!r.ok) throw new Error(data.error || 'preview failed')
      setPreviewTags(data.expanded_tags || []); setPreviewAlarms(data.expanded_alarms || [])
      setTab('instantiate')
    } catch (e: any) { setError(e.message) }
    setLoading(false)
  }

  const instantiate = async () => {
    if (!selectedId || !instanceName.trim()) return
    setInstantiating(true); setError(''); setSuccess('')
    try {
      const r = await fetch(`/templates/${selectedId}/instantiate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ template_id: selectedId, name: instanceName.trim(), instance_counts: { n: parseInt(instanceCount) || 2 } }),
        credentials: 'include',
      })
      const data = await r.json()
      if (!r.ok) throw new Error(data.error || 'instantiate failed')
      setSuccess(`Created: ${data.tags_created} tags, ${data.alarms_created} alarms, ${data.flows_created} flows on device "${data.device_id}"`)
    } catch (e: any) { setError(e.message) }
    setInstantiating(false)
  }

  const industryName = (id: string) => industries.find(i => i.id === id)?.name || id

  const tabStyle = (active: boolean) => ({
    padding: '8px 16px', cursor: 'pointer', fontSize: 13,
    color: active ? 'var(--fg)' : 'var(--muted)',
    borderBottom: active ? '2px solid #f97316' : '2px solid transparent',
  })

  const cardBase = { background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20, cursor: 'pointer', transition: 'border-color 0.2s' } as React.CSSProperties
  const badge = (bg: string, c: string) => ({ display: 'inline-block', padding: '2px 8px', borderRadius: 4, fontSize: 11, fontWeight: 600, background: bg, color: c })

  return (
    <div style={{ padding: 16 }}>
      <h2 style={{ fontSize: 18, fontWeight: 600, margin: '0 0 4px' }}>Vertical Templates</h2>
      <p style={{ color: 'var(--muted)', fontSize: 13, margin: '0 0 16px' }}>
        Pre-built configurations for specific industries. Select → Preview → Instantiate.
      </p>

      <div style={{ display: 'flex', gap: 0, borderBottom: '1px solid var(--border)', marginBottom: 16 }}>
        <div style={tabStyle(tab === 'browse')} onClick={() => { setTab('browse'); setSelectedId(null) }}>Browse</div>
        <div style={tabStyle(tab === 'detail')} onClick={() => { if (selectedId) setTab('detail') }}>Detail</div>
        <div style={tabStyle(tab === 'instantiate')} onClick={() => { if (selectedId) setTab('instantiate') }}>Instantiate</div>
      </div>

      {error && <div style={{ color: '#ef4444', fontSize: 13, marginBottom: 12 }}>{error}</div>}
      {success && <div style={{ color: '#16a34a', fontSize: 13, fontWeight: 500, marginBottom: 12 }}>{success}</div>}
      {loading && <div style={{ color: 'var(--muted)', fontSize: 13 }}>Loading…</div>}

      {!loading && tab === 'browse' && (
        <>
          <select value={selectedIndustry} onChange={e => setSelectedIndustry(e.target.value)} style={{
            padding: '6px 10px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', fontSize: 13, marginBottom: 12,
          }}>
            <option value="">All Industries</option>
            {industries.map(i => <option key={i.id} value={i.id}>{i.name}</option>)}
          </select>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(340px, 1fr))', gap: 16 }}>
            {templates.map(t => (
              <div key={t.id} onClick={() => selectTemplate(t.id)} style={{
                ...cardBase,
                borderColor: selectedId === t.id ? '#f97316' : undefined,
              }}>
                <div style={{ fontSize: 16, fontWeight: 600, marginBottom: 6 }}>{t.name}</div>
                <div style={{ fontSize: 13, color: 'var(--muted)', lineHeight: 1.4, marginBottom: 12 }}>{t.description}</div>
                <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
                  <span style={badge('rgba(249,115,22,.12)', '#f97316')}>{industryName(t.industry)}</span>
                  <span style={badge('rgba(22,163,74,.12)', '#16a34a')}>{t.tag_count} tags</span>
                  <span style={badge('rgba(59,130,246,.12)', '#3b82f6')}>{t.alarm_count} alarms</span>
                  <span style={badge('rgba(139,92,246,.12)', '#8b5cf6')}>{t.flow_count} flows</span>
                  {t.has_oee && <span style={badge('rgba(249,115,22,.12)', '#f97316')}>OEE</span>}
                </div>
              </div>
            ))}
          </div>
        </>
      )}

      {!loading && tab === 'detail' && detail && (
        <>
          <div style={cardBase}>
            <div style={{ fontSize: 18, fontWeight: 600 }}>{detail.name}</div>
            <div style={{ fontSize: 13, color: 'var(--muted)', margin: '6px 0 12px' }}>{detail.description}</div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
              {[
                ['Industry', industryName(detail.industry)],
                ['Equipment', detail.equipment_type],
                ['Tags', `${detail.tags.length} patterns`],
                ['Alarms', `${detail.alarms.length} rules`],
                ['Flows', `${detail.flows.length} rules`],
                ['OEE', detail.oee_config ? 'Yes' : 'No'],
              ].map(([l, v]) => (
                <div key={l}>
                  <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>{l}</div>
                  <div style={{ fontSize: 14, fontWeight: 500 }}>{v}</div>
                </div>
              ))}
            </div>
          </div>

          <h3 style={{ fontSize: 15, fontWeight: 600, margin: '16px 0 8px' }}>Tag Patterns (ISA-5.1)</h3>
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
              <thead><tr>{['Pattern', 'Description', 'Unit', 'Type', 'Group'].map(h => (
                <th key={h} style={{ textAlign: 'left', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', padding: '6px 8px', borderBottom: '1px solid var(--border)' }}>{h}</th>
              ))}</tr></thead>
              <tbody>{detail.tags.map((t, i) => (
                <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                  <td><code>{t.name_pattern}</code></td><td>{t.description}</td><td>{t.unit}</td><td>{t.data_type}</td><td>{t.group}</td>
                </tr>
              ))}</tbody>
            </table>
          </div>

          <h3 style={{ fontSize: 15, fontWeight: 600, margin: '16px 0 8px' }}>Alarm Rules</h3>
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
              <thead><tr>{['Tag', 'Severity', 'Condition', 'Threshold', 'Message'].map(h => (
                <th key={h} style={{ textAlign: 'left', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', padding: '6px 8px', borderBottom: '1px solid var(--border)' }}>{h}</th>
              ))}</tr></thead>
              <tbody>{detail.alarms.map((a, i) => (
                <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                  <td><code>{a.tag_pattern}</code></td>
                  <td><span style={badge(a.severity === 'critical' ? 'rgba(139,92,246,.12)' : 'rgba(249,115,22,.12)', a.severity === 'critical' ? '#8b5cf6' : '#f97316')}>{a.severity}</span></td>
                  <td>{a.comparison}</td><td>{a.threshold}</td><td>{a.message_template}</td>
                </tr>
              ))}</tbody>
            </table>
          </div>

          <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
            <button onClick={preview} style={{ padding: '8px 16px', borderRadius: 6, border: 'none', background: '#f97316', color: '#fff', cursor: 'pointer', fontSize: 13, fontWeight: 600 }}>Preview &amp; Instantiate</button>
            <button onClick={() => setTab('browse')} style={{ padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Back</button>
          </div>
        </>
      )}

      {!loading && tab === 'instantiate' && (
        <>
          <div style={cardBase}>
            <h3 style={{ margin: '0 0 12px' }}>Instantiate Template</h3>
            <div style={{ display: 'grid', gridTemplateColumns: '120px 1fr', gap: '8px 12px', alignItems: 'center' }}>
              <span style={{ fontSize: 12, color: 'var(--muted)', textAlign: 'right' }}>Name</span>
              <input value={instanceName} onChange={e => setInstanceName(e.target.value)} placeholder="e.g. My Plant Pump Station" style={{
                padding: '6px 10px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', fontSize: 13,
              }} />
              <span style={{ fontSize: 12, color: 'var(--muted)', textAlign: 'right' }}>Instances</span>
              <input type="number" min={1} max={20} value={instanceCount} onChange={e => setInstanceCount(e.target.value)} style={{
                padding: '6px 10px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', fontSize: 13, width: 80,
              }} />
            </div>
          </div>

          {previewTags.length > 0 && (
            <>
              <h3 style={{ fontSize: 15, fontWeight: 600, margin: '16px 0 8px' }}>Preview: {previewTags.length} Tags</h3>
              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
                  <thead><tr>{['Name', 'Description', 'Unit', 'Type', 'Widget'].map(h => (
                    <th key={h} style={{ textAlign: 'left', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', padding: '6px 8px', borderBottom: '1px solid var(--border)' }}>{h}</th>
                  ))}</tr></thead>
                  <tbody>{previewTags.slice(0, 30).map((t: any, i: number) => (
                    <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                      <td><code>{t.name}</code></td><td>{t.description}</td><td>{t.unit}</td><td>{t.data_type}</td><td>{t.widget}</td>
                    </tr>
                  ))}</tbody>
                </table>
                {previewTags.length > 30 && <div style={{ color: 'var(--muted)', fontSize: 13, padding: 16, textAlign: 'center' }}>…and {previewTags.length - 30} more</div>}
              </div>
            </>
          )}

          <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
            <button onClick={instantiate} disabled={instantiating || !instanceName.trim()} style={{
              padding: '8px 16px', borderRadius: 6, border: 'none',
              background: instantiating || !instanceName.trim() ? '#666' : '#f97316',
              color: '#fff', cursor: instantiating || !instanceName.trim() ? 'not-allowed' : 'pointer', fontSize: 13, fontWeight: 600,
            }}>{instantiating ? 'Creating…' : 'Instantiate Template'}</button>
            <button onClick={() => setTab('detail')} style={{ padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Back</button>
          </div>
        </>
      )}
    </div>
  )
}
