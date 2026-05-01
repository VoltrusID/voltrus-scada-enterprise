import { useState, useMemo, useCallback } from 'react'
import { useFlows, useFlow, useFlowRuns, useCreateFlow, useUpdateFlow, useDeleteFlow, useStartFlow, useStopFlow, useTestFlow } from './queries'
import type { Flow, FlowNodeConfig, FlowTriggerConfig } from './types'
import { btnStyle } from './styles'

const NODE_TYPES: { value: string; label: string; config: Record<string, any> }[] = [
  { value: 'filter', label: 'Filter', config: { condition: 'payload.metrics.temp > 0' } },
  { value: 'transform', label: 'Transform', config: { map: { status: "if(payload.metrics.temp > 80, 'CRITICAL', 'OK')" } } },
  { value: 'threshold', label: 'Threshold', config: { field: 'payload.metrics.temp', rules: [{ gt: 80, set: { alarm: 'HIGH' } }] } },
  { value: 'lookup', label: 'Lookup', config: { field: 'payload.machine_code', output_field: 'payload.machine_name', table: { M1: 'Cutting' }, default: 'Unknown' } },
  { value: 'expression', label: 'Expression', config: { output_field: 'payload.power_kw', expr: 'payload.metrics.voltage * payload.metrics.current * 0.001' } },
  { value: 'data_source_out', label: 'Data Source Output', config: { value_field: 'payload.power_kw', source_key: 'flow_power', name: 'Power', unit: 'kW', decimals: 2 } },
  { value: 'http_out', label: 'HTTP Webhook', config: { url: 'https://example.com/webhook', headers: {}, timeout_ms: 5000, retry_count: 3 } },
  { value: 'alarm_out', label: 'Alarm', config: { severity: 'critical', source_key: 'flow_alert', message: 'Alert: {{payload.metrics.temp}}' } },
]

const TRIGGER_TYPES: { value: string; label: string; config: Record<string, any> }[] = [
  { value: 'modbus', label: 'Modbus Device', config: { type: 'modbus', device_id: '*', metric: '', debounce_ms: 1000, on_change_only: false } },
  { value: 'timer', label: 'Timer Interval', config: { type: 'timer', interval_ms: 5000 } },
  { value: 'mqtt', label: 'MQTT Topic', config: { type: 'mqtt', topic: 'sensors/+/temp', debounce_ms: 1000 } },
]

export function FlowManager({ isAdmin }: { isAdmin: boolean }) {
  const [editingId, setEditingId] = useState<string | null>(null)
  const [showEditor, setShowEditor] = useState(false)
  const [showJson, setShowJson] = useState(false)
  const [testPayload, setTestPayload] = useState('{"metrics": {"temp": 85}}')
  const [testResult, setTestResult] = useState<any>(null)
  const [testLoading, setTestLoading] = useState(false)
  const [error, setError] = useState('')

  const { data: flows = [] } = useFlows(isAdmin)
  const createFlow = useCreateFlow()
  const updateFlow = useUpdateFlow()
  const deleteFlow = useDeleteFlow()
  const startFlow = useStartFlow()
  const stopFlow = useStopFlow()
  const testFlow = useTestFlow()

  const [form, setForm] = useState<Flow>({
    id: '',
    name: '',
    enabled: true,
    trigger_config: JSON.stringify(TRIGGER_TYPES[0].config),
    nodes: JSON.stringify([{ id: 'n1', type: 'filter', condition: 'payload.metrics.temp > 0' }]),
    created_at: 0,
    updated_at: 0,
  })

  const parsedNodes = useMemo(() => {
    try { return JSON.parse(form.nodes) as FlowNodeConfig[] } catch { return [] }
  }, [form.nodes])

  const parsedTrigger = useMemo(() => {
    try { return JSON.parse(form.trigger_config) as any } catch { return { type: 'modbus' } }
  }, [form.trigger_config])

  const handleSave = async () => {
    setError('')
    try {
      JSON.parse(form.trigger_config)
      JSON.parse(form.nodes)
    } catch (e) {
      setError('Invalid JSON in trigger config or nodes')
      return
    }
    if (!form.id || !form.name) {
      setError('ID and Name are required')
      return
    }
    const now = Math.floor(Date.now() / 1000)
    const payload = { ...form, created_at: form.created_at || now, updated_at: now }
    if (editingId) {
      await updateFlow.mutateAsync(payload)
    } else {
      await createFlow.mutateAsync(payload)
    }
    setShowEditor(false)
    setEditingId(null)
  }

  const handleEdit = (flow: Flow) => {
    setForm(flow)
    setEditingId(flow.id)
    setShowEditor(true)
    setTestResult(null)
  }

  const handleCreate = () => {
    setForm({
      id: `flow_${Date.now()}`,
      name: 'New Flow',
      enabled: true,
      trigger_config: JSON.stringify(TRIGGER_TYPES[0].config, null, 2),
      nodes: JSON.stringify([{ id: 'n1', type: 'filter', condition: 'payload.metrics.temp > 0' }], null, 2),
      created_at: 0,
      updated_at: 0,
    })
    setEditingId(null)
    setShowEditor(true)
    setTestResult(null)
  }

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this flow?')) return
    await deleteFlow.mutateAsync(id)
  }

  const handleTest = async () => {
    if (!editingId) return
    setTestLoading(true)
    setTestResult(null)
    try {
      const payload = JSON.parse(testPayload)
      const result = await testFlow.mutateAsync({ id: editingId, payload })
      setTestResult(result)
    } catch (e: any) {
      setTestResult({ error: e.message })
    }
    setTestLoading(false)
  }

  const addNode = (type: string) => {
    const template = NODE_TYPES.find(n => n.value === type)
    if (!template) return
    const newNode = { id: `n${parsedNodes.length + 1}`, type: template.value, ...template.config }
    const updated = [...parsedNodes, newNode]
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
  }

  const removeNode = (index: number) => {
    const updated = parsedNodes.filter((_, i) => i !== index)
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
  }

  const updateNode = (index: number, key: string, value: any) => {
    const updated = parsedNodes.map((n, i) => i === index ? { ...n, [key]: value } : n)
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
  }

  const updateTrigger = (key: string, value: any) => {
    const updated = { ...parsedTrigger, [key]: value }
    setForm(f => ({ ...f, trigger_config: JSON.stringify(updated, null, 2) }))
  }

  const inputStyle: React.CSSProperties = {
    background: 'var(--card)',
    border: '1px solid var(--border)',
    borderRadius: 4,
    padding: '6px 10px',
    color: 'var(--fg)',
    fontSize: 13,
    fontFamily: "'JetBrains Mono', monospace",
    width: '100%',
  }

  const labelStyle: React.CSSProperties = {
    fontSize: 11,
    color: 'var(--muted)',
    textTransform: 'uppercase' as const,
    letterSpacing: 0.5,
    marginBottom: 4,
    display: 'block',
  }

  if (showEditor) {
    return (
      <div style={{ padding: 20, maxWidth: 900, margin: '0 auto' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 20 }}>
          <button style={btnStyle} onClick={() => setShowEditor(false)}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
            Back
          </button>
          <h2 style={{ fontSize: 20, fontWeight: 700, margin: 0 }}>{editingId ? 'Edit Flow' : 'New Flow'}</h2>
          <div style={{ marginLeft: 'auto', display: 'flex', gap: 8 }}>
            <button style={{ ...btnStyle, borderColor: showJson ? 'var(--cyan)' : 'var(--border)', color: showJson ? 'var(--cyan)' : 'var(--fg)' }} onClick={() => setShowJson(!showJson)}>
              {showJson ? 'Form' : 'JSON'}
            </button>
            <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={handleSave}>
              Save
            </button>
          </div>
        </div>

        {error && <div style={{ background: 'rgba(239,68,68,.1)', border: '1px solid var(--crit)', color: 'var(--crit)', padding: '10px 14px', borderRadius: 6, marginBottom: 16, fontSize: 13 }}>{error}</div>}

        {showJson ? (
          <div style={{ display: 'grid', gap: 16 }}>
            <div>
              <label style={labelStyle}>Flow JSON</label>
              <textarea
                style={{ ...inputStyle, minHeight: 400, fontFamily: "'JetBrains Mono', monospace", fontSize: 12 }}
                value={JSON.stringify({ id: form.id, name: form.name, enabled: form.enabled, trigger: parsedTrigger, nodes: parsedNodes }, null, 2)}
                onChange={e => {
                  try {
                    const parsed = JSON.parse(e.target.value)
                    setForm(f => ({
                      ...f,
                      id: parsed.id || f.id,
                      name: parsed.name || f.name,
                      enabled: parsed.enabled ?? f.enabled,
                      trigger_config: JSON.stringify(parsed.trigger || parsed.trigger_config || {}),
                      nodes: JSON.stringify(parsed.nodes || parsed.node || []),
                    }))
                    setError('')
                  } catch {
                    setError('Invalid JSON')
                  }
                }}
              />
            </div>
          </div>
        ) : (
          <div style={{ display: 'grid', gap: 20 }}>
            {/* Basic Info */}
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
              <div>
                <label style={labelStyle}>Flow ID</label>
                <input style={inputStyle} value={form.id} onChange={e => setForm(f => ({ ...f, id: e.target.value }))} disabled={!!editingId} />
              </div>
              <div>
                <label style={labelStyle}>Name</label>
                <input style={inputStyle} value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} />
              </div>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <input type="checkbox" checked={form.enabled} onChange={e => setForm(f => ({ ...f, enabled: e.target.checked }))} />
              <span style={{ fontSize: 13 }}>Enabled</span>
            </div>

            {/* Trigger */}
            <div style={{ border: '1px solid var(--border)', borderRadius: 8, padding: 16 }}>
              <h3 style={{ fontSize: 14, fontWeight: 600, margin: '0 0 12px', color: 'var(--cyan)' }}>Trigger</h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
                <div>
                  <label style={labelStyle}>Type</label>
                  <select style={inputStyle} value={parsedTrigger.type} onChange={e => {
                    const t = TRIGGER_TYPES.find(t => t.value === e.target.value)
                    if (t) setForm(f => ({ ...f, trigger_config: JSON.stringify(t.config, null, 2) }))
                  }}>
                    {TRIGGER_TYPES.map(t => <option key={t.value} value={t.value}>{t.label}</option>)}
                  </select>
                </div>
                {parsedTrigger.type === 'modbus' && (
                  <>
                    <div>
                      <label style={labelStyle}>Device ID</label>
                      <input style={inputStyle} value={parsedTrigger.device_id || ''} onChange={e => updateTrigger('device_id', e.target.value)} />
                    </div>
                    <div>
                      <label style={labelStyle}>Metric (optional)</label>
                      <input style={inputStyle} value={parsedTrigger.metric || ''} onChange={e => updateTrigger('metric', e.target.value)} />
                    </div>
                    <div>
                      <label style={labelStyle}>Debounce (ms)</label>
                      <input style={inputStyle} type="number" value={parsedTrigger.debounce_ms || 0} onChange={e => updateTrigger('debounce_ms', Number(e.target.value))} />
                    </div>
                  </>
                )}
                {parsedTrigger.type === 'timer' && (
                  <div>
                    <label style={labelStyle}>Interval (ms)</label>
                    <input style={inputStyle} type="number" value={parsedTrigger.interval_ms || 5000} onChange={e => updateTrigger('interval_ms', Number(e.target.value))} />
                  </div>
                )}
                {parsedTrigger.type === 'mqtt' && (
                  <div>
                    <label style={labelStyle}>Topic</label>
                    <input style={inputStyle} value={parsedTrigger.topic || ''} onChange={e => updateTrigger('topic', e.target.value)} />
                  </div>
                )}
              </div>
            </div>

            {/* Nodes */}
            <div style={{ border: '1px solid var(--border)', borderRadius: 8, padding: 16 }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
                <h3 style={{ fontSize: 14, fontWeight: 600, margin: 0, color: 'var(--cyan)' }}>Nodes ({parsedNodes.length})</h3>
                <select
                  style={{ ...inputStyle, width: 'auto' }}
                  value=""
                  onChange={e => { if (e.target.value) { addNode(e.target.value); e.target.value = '' } }}
                >
                  <option value="">+ Add Node</option>
                  {NODE_TYPES.map(n => <option key={n.value} value={n.value}>{n.label}</option>)}
                </select>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {parsedNodes.map((node, i) => (
                  <div key={i} style={{ border: '1px solid var(--border)', borderRadius: 6, padding: 12, background: 'var(--bg)' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                      <span style={{ fontSize: 11, color: 'var(--muted)', fontFamily: "'JetBrains Mono', monospace" }}>#{i + 1}</span>
                      <span style={{ fontSize: 13, fontWeight: 600, textTransform: 'capitalize' }}>{node.type.replace(/_/g, ' ')}</span>
                      <span style={{ fontSize: 11, color: 'var(--muted)', fontFamily: "'JetBrains Mono', monospace" }}>{node.id}</span>
                      <button style={{ marginLeft: 'auto', ...btnStyle, padding: '2px 6px', fontSize: 11, color: 'var(--crit)', borderColor: 'var(--crit)' }} onClick={() => removeNode(i)}>Remove</button>
                    </div>
                    <NodeEditor node={node} onChange={(key, val) => updateNode(i, key, val)} />
                  </div>
                ))}
                {parsedNodes.length === 0 && (
                  <div style={{ color: 'var(--muted)', fontSize: 13, textAlign: 'center', padding: 20 }}>No nodes. Add one above.</div>
                )}
              </div>
            </div>

            {/* Test Panel */}
            {editingId && (
              <div style={{ border: '1px solid var(--border)', borderRadius: 8, padding: 16 }}>
                <h3 style={{ fontSize: 14, fontWeight: 600, margin: '0 0 12px', color: 'var(--green)' }}>Test Flow</h3>
                <div style={{ display: 'grid', gap: 12 }}>
                  <div>
                    <label style={labelStyle}>Test Payload (JSON)</label>
                    <textarea style={{ ...inputStyle, minHeight: 80 }} value={testPayload} onChange={e => setTestPayload(e.target.value)} />
                  </div>
                  <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)', width: 'fit-content' }} onClick={handleTest} disabled={testLoading}>
                    {testLoading ? 'Running...' : 'Run Test'}
                  </button>
                  {testResult && (
                    <div style={{ background: 'var(--bg)', borderRadius: 6, padding: 12, fontSize: 12, fontFamily: "'JetBrains Mono', monospace", overflow: 'auto', maxHeight: 400 }}>
                      {testResult.error ? (
                        <div style={{ color: 'var(--crit)' }}>{testResult.error}</div>
                      ) : (
                        <>
                          <div style={{ marginBottom: 8, color: testResult.success ? 'var(--green)' : 'var(--crit)' }}>
                            Success: {testResult.success ? 'Yes' : 'No'}
                          </div>
                          <div style={{ marginBottom: 8 }}>Output:</div>
                          <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>{JSON.stringify(testResult.output, null, 2)}</pre>
                          {testResult.node_results && (
                            <>
                              <div style={{ marginTop: 12, marginBottom: 8 }}>Node Results:</div>
                              {testResult.node_results.map((nr: any, i: number) => (
                                <div key={i} style={{ marginBottom: 6, padding: 6, borderRadius: 4, background: 'var(--card)' }}>
                                  <div style={{ fontSize: 11, color: 'var(--muted)' }}>{nr.node_id} → {nr.result || nr.error}</div>
                                  {nr.output && <pre style={{ margin: '4px 0 0', fontSize: 11, whiteSpace: 'pre-wrap' }}>{JSON.stringify(nr.output, null, 2)}</pre>}
                                </div>
                              ))}
                            </>
                          )}
                        </>
                      )}
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    )
  }

  return (
    <div style={{ padding: 20 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 20 }}>
        <h2 style={{ fontSize: 20, fontWeight: 700, margin: 0 }}>Flows</h2>
        {isAdmin && (
          <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={handleCreate}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
            New Flow
          </button>
        )}
      </div>

      {flows.length === 0 ? (
        <div style={{ color: 'var(--muted)', textAlign: 'center', padding: 60 }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>📊</div>
          <div style={{ fontSize: 16, marginBottom: 8 }}>No flows configured</div>
          <div style={{ fontSize: 13 }}>Create a flow to transform data, compute metrics, or send alerts.</div>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {flows.map(flow => {
            let triggerInfo = 'unknown'
            try {
              const t = JSON.parse(flow.trigger_config)
              triggerInfo = t.type === 'modbus' ? `Modbus: ${t.device_id || '*'}` : t.type === 'timer' ? `Timer: ${t.interval_ms}ms` : `MQTT: ${t.topic || ''}`
            } catch {}
            return (
              <div key={flow.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 16px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--card)' }}>
                <div style={{ width: 8, height: 8, borderRadius: '50%', background: flow.enabled ? 'var(--green)' : 'var(--muted)', flexShrink: 0 }} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 14, fontWeight: 600 }}>{flow.name}</div>
                  <div style={{ fontSize: 11, color: 'var(--muted)', fontFamily: "'JetBrains Mono', monospace" }}>{flow.id} • {triggerInfo} • {(() => { try { return `${JSON.parse(flow.nodes).length} nodes` } catch { return '0 nodes' } })()}</div>
                </div>
                <div style={{ display: 'flex', gap: 6, flexShrink: 0 }}>
                  {flow.enabled ? (
                    <button style={{ ...btnStyle, padding: '4px 8px', fontSize: 12, color: 'var(--warn)', borderColor: 'var(--warn)' }} onClick={() => stopFlow.mutate(flow.id)}>Stop</button>
                  ) : (
                    <button style={{ ...btnStyle, padding: '4px 8px', fontSize: 12, color: 'var(--green)', borderColor: 'var(--green)' }} onClick={() => startFlow.mutate(flow.id)}>Start</button>
                  )}
                  <button style={{ ...btnStyle, padding: '4px 8px', fontSize: 12 }} onClick={() => handleEdit(flow)}>Edit</button>
                  <button style={{ ...btnStyle, padding: '4px 8px', fontSize: 12, color: 'var(--crit)', borderColor: 'var(--crit)' }} onClick={() => handleDelete(flow.id)}>Delete</button>
                </div>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}

function NodeEditor({ node, onChange }: { node: FlowNodeConfig; onChange: (key: string, value: any) => void }) {
  const inputStyle: React.CSSProperties = {
    background: 'var(--card)',
    border: '1px solid var(--border)',
    borderRadius: 4,
    padding: '4px 8px',
    color: 'var(--fg)',
    fontSize: 12,
    fontFamily: "'JetBrains Mono', monospace",
    width: '100%',
  }

  const labelStyle: React.CSSProperties = {
    fontSize: 10,
    color: 'var(--muted)',
    textTransform: 'uppercase' as const,
    letterSpacing: 0.5,
    marginBottom: 2,
    display: 'block',
  }

  const renderField = (key: string, value: any) => {
    if (key === 'id' || key === 'type') return null
    if (typeof value === 'boolean') {
      return (
        <div key={key} style={{ display: 'flex', alignItems: 'center', gap: 6, marginTop: 4 }}>
          <input type="checkbox" checked={value} onChange={e => onChange(key, e.target.checked)} />
          <span style={{ fontSize: 12 }}>{key}</span>
        </div>
      )
    }
    if (typeof value === 'number') {
      return (
        <div key={key} style={{ marginTop: 4 }}>
          <label style={labelStyle}>{key}</label>
          <input style={inputStyle} type="number" value={value} onChange={e => onChange(key, Number(e.target.value))} />
        </div>
      )
    }
    if (typeof value === 'object') {
      return (
        <div key={key} style={{ marginTop: 4 }}>
          <label style={labelStyle}>{key}</label>
          <textarea
            style={{ ...inputStyle, minHeight: 60, fontSize: 11 }}
            value={typeof value === 'string' ? value : JSON.stringify(value, null, 2)}
            onChange={e => {
              try { onChange(key, JSON.parse(e.target.value)) } catch { onChange(key, e.target.value) }
            }}
          />
        </div>
      )
    }
    return (
      <div key={key} style={{ marginTop: 4 }}>
        <label style={labelStyle}>{key}</label>
        <input style={inputStyle} value={value} onChange={e => onChange(key, e.target.value)} />
      </div>
    )
  }

  return (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
      {Object.entries(node).filter(([k]) => k !== 'id' && k !== 'type').map(([k, v]) => (
        <div key={k} style={{ gridColumn: typeof v === 'object' ? 'span 2' : undefined }}>
          {renderField(k, v)}
        </div>
      ))}
    </div>
  )
}
