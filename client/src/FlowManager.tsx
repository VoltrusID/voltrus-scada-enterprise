import { useState, useMemo, useRef, useEffect } from 'react'
import { useFlows, useCreateFlow, useUpdateFlow, useDeleteFlow, useStartFlow, useStopFlow, useTestFlow } from './queries'
import type { Flow, FlowNodeConfig, FlowNodeType } from './types'
import { btnStyle } from './styles'

/* ── Node type definitions ── */
const NODE_META: Record<string, { label: string; color: string; icon: string; desc: string; defaultConfig: Record<string, any> }> = {
  filter:        { label: 'Filter',        color: '#22d3ee', icon: 'M3 6h18M3 12h18M3 18h18', desc: 'Pass only if condition is true', defaultConfig: { condition: 'payload.metrics.temp > 0' } },
  transform:     { label: 'Transform',     color: '#60a5fa', icon: 'M23 4l-11 16-5-5-5 5M23 4h-6', desc: 'Map fields to new schema', defaultConfig: { map: { status: "if(payload.metrics.temp > 80, 'CRITICAL', 'OK')" } } },
  threshold:     { label: 'Threshold',     color: '#fbbf24', icon: 'M12 2v20M2 12h20', desc: 'Check values against rules', defaultConfig: { field: 'payload.metrics.temp', rules: [{ gt: 80, set: { alarm: 'HIGH' } }] } },
  lookup:        { label: 'Lookup',        color: '#c084fc', icon: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z', desc: 'Map values via lookup table', defaultConfig: { field: 'payload.machine_code', output_field: 'payload.machine_name', table: { M1: 'Cutting' }, default: 'Unknown' } },
  expression:    { label: 'Expression',    color: '#4ade80', icon: 'M9 7h6M9 11h6M9 15h6M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z', desc: 'Compute new field from formula', defaultConfig: { output_field: 'payload.power_kw', expr: 'payload.metrics.voltage * payload.metrics.current * 0.001' } },
  data_source_out: { label: 'Data Source',  color: '#a3e635', icon: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5', desc: 'Write to virtual sensor', defaultConfig: { value_field: 'payload.power_kw', source_key: 'flow_power', name: 'Power', unit: 'kW', decimals: 2 } },
  http_out:      { label: 'HTTP Webhook',  color: '#818cf8', icon: 'M10 13a5 5 0 007 0M4 15l3-3-3-3M20 15l-3-3 3-3', desc: 'POST to external URL', defaultConfig: { url: 'https://example.com/webhook', headers: {}, timeout_ms: 5000, retry_count: 3 } },
  alarm_out:     { label: 'Alarm',         color: '#f87171', icon: 'M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0zM12 9v4M12 17h.01', desc: 'Trigger an alarm', defaultConfig: { severity: 'critical', source_key: 'flow_alert', message: 'Alert triggered' } },
}

const TRIGGER_META: Record<string, { label: string; color: string }> = {
  modbus: { label: 'Modbus', color: '#22d3ee' },
  timer:  { label: 'Timer',  color: '#fbbf24' },
  mqtt:   { label: 'MQTT',   color: '#c084fc' },
}

const TRIGGER_DEFAULTS: Record<string, Record<string, any>> = {
  modbus: { type: 'modbus', device_id: '*', metric: '', debounce_ms: 1000, on_change_only: false },
  timer:  { type: 'timer', interval_ms: 5000 },
  mqtt:   { type: 'mqtt', topic: 'sensors/+/temp', debounce_ms: 1000 },
}

/* ── SVG Edge between two node positions ── */
function Edge({ from, to }: { from: { x: number; y: number }; to: { x: number; y: number } }) {
  const mx = (from.x + to.x) / 2
  const d = `M ${from.x} ${from.y} C ${mx} ${from.y}, ${mx} ${to.y}, ${to.x} ${to.y}`
  return (
    <svg style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', pointerEvents: 'none', zIndex: 0 }}>
      <defs>
        <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
          <polygon points="0 0, 8 3, 0 6" fill="var(--border)" />
        </marker>
      </defs>
      <path d={d} fill="none" stroke="var(--border)" strokeWidth={2} markerEnd="url(#arrowhead)" />
    </svg>
  )
}

/* ── Flow List View ── */
function FlowList({ flows, onEdit, onDelete, onStart, onStop, onCreate }: {
  flows: Flow[]
  onEdit: (f: Flow) => void
  onDelete: (id: string) => void
  onStart: (id: string) => void
  onStop: (id: string) => void
  onCreate: () => void
}) {
  return (
    <div style={{ padding: '24px 32px', maxWidth: 1200, margin: '0 auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 28 }}>
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, margin: '0 0 4px', letterSpacing: '-0.5px' }}>Flows</h2>
          <p style={{ margin: 0, fontSize: 13, color: 'var(--muted)' }}>Data pipelines that transform, filter, and route sensor data</p>
        </div>
        <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)', fontSize: 13 }} onClick={onCreate}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          New Flow
        </button>
      </div>

      {flows.length === 0 ? (
        <div style={{ border: '1px dashed var(--border)', borderRadius: 12, padding: 60, textAlign: 'center' }}>
          <div style={{ width: 48, height: 48, margin: '0 auto 16px', borderRadius: 12, background: 'var(--card)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--muted)" strokeWidth="1.5"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
          </div>
          <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 6 }}>No flows configured</div>
          <div style={{ fontSize: 13, color: 'var(--muted)' }}>Create a flow to transform data, compute metrics, or send alerts.</div>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {flows.map(flow => {
            let triggerLabel = 'Unknown'
            let triggerColor = 'var(--muted)'
            let nodeCount = 0
            try {
              const t = JSON.parse(flow.trigger_config)
              triggerLabel = TRIGGER_META[t.type]?.label || t.type
              triggerColor = TRIGGER_META[t.type]?.color || 'var(--muted)'
              nodeCount = JSON.parse(flow.nodes).length
            } catch {}
            return (
              <div key={flow.id} style={{
                display: 'flex', alignItems: 'center', gap: 16, padding: '14px 20px', borderRadius: 10,
                border: '1px solid var(--border)', background: 'var(--card)',
                transition: 'border-color .15s', cursor: 'pointer',
              }} onClick={() => onEdit(flow)}>
                <div style={{
                  width: 10, height: 10, borderRadius: '50%', flexShrink: 0,
                  background: flow.enabled ? 'var(--green)' : 'var(--muted)',
                  boxShadow: flow.enabled ? '0 0 8px rgba(34,197,94,.4)' : 'none',
                }} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 3 }}>{flow.name}</div>
                  <div style={{ fontSize: 11, color: 'var(--muted)', fontFamily: "'JetBrains Mono', monospace", display: 'flex', gap: 12 }}>
                    <span>{flow.id}</span>
                    <span style={{ color: triggerColor }}>{triggerLabel}</span>
                    <span>{nodeCount} node{nodeCount !== 1 ? 's' : ''}</span>
                  </div>
                </div>
                <div style={{ display: 'flex', gap: 6, flexShrink: 0 }} onClick={e => e.stopPropagation()}>
                  {flow.enabled ? (
                    <button style={{ ...btnStyle, padding: '5px 10px', fontSize: 12, color: 'var(--warn)' }} onClick={() => onStop(flow.id)}>Stop</button>
                  ) : (
                    <button style={{ ...btnStyle, padding: '5px 10px', fontSize: 12, color: 'var(--green)' }} onClick={() => onStart(flow.id)}>Start</button>
                  )}
                  <button style={{ ...btnStyle, padding: '5px 10px', fontSize: 12 }} onClick={() => onEdit(flow)}>Edit</button>
                  <button style={{ ...btnStyle, padding: '5px 10px', fontSize: 12, color: 'var(--crit)' }} onClick={() => onDelete(flow.id)}>Delete</button>
                </div>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}

/* ── Visual Flow Editor ── */
function FlowEditor({ flow, onSave, onCancel }: {
  flow: Flow
  onSave: (f: Flow) => void
  onCancel: () => void
}) {
  const [form, setForm] = useState<Flow>({ ...flow })
  const [showJson, setShowJson] = useState(false)
  const [testPayload, setTestPayload] = useState('{"metrics": {"temp": 85}}')
  const [testResult, setTestResult] = useState<any>(null)
  const [testLoading, setTestLoading] = useState(false)
  const [error, setError] = useState('')
  const [selectedNode, setSelectedNode] = useState<number | null>(null)
  const canvasRef = useRef<HTMLDivElement>(null)
  const testFlow = useTestFlow()

  const parsedNodes = useMemo(() => {
    try { return JSON.parse(form.nodes) as FlowNodeConfig[] } catch { return [] }
  }, [form.nodes])

  const parsedTrigger = useMemo(() => {
    try { return JSON.parse(form.trigger_config) as any } catch { return { type: 'modbus' } }
  }, [form.trigger_config])

  const handleSave = () => {
    setError('')
    try { JSON.parse(form.trigger_config); JSON.parse(form.nodes) } catch { setError('Invalid JSON'); return }
    if (!form.id || !form.name) { setError('ID and Name required'); return }
    onSave({ ...form, updated_at: Math.floor(Date.now() / 1000) })
  }

  const addNode = (type: FlowNodeType, afterIndex?: number) => {
    const meta = NODE_META[type]
    if (!meta) return
    const newNode: FlowNodeConfig = { id: `n${Date.now()}`, type, ...meta.defaultConfig }
    const idx = afterIndex !== undefined ? afterIndex + 1 : parsedNodes.length
    const updated = [...parsedNodes.slice(0, idx), newNode, ...parsedNodes.slice(idx)]
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
    setSelectedNode(idx)
  }

  const removeNode = (index: number) => {
    const updated = parsedNodes.filter((_, i) => i !== index)
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
    if (selectedNode === index) setSelectedNode(null)
  }

  const updateNode = (index: number, key: string, value: any) => {
    const updated = parsedNodes.map((n, i) => i === index ? { ...n, [key]: value } : n)
    setForm(f => ({ ...f, nodes: JSON.stringify(updated, null, 2) }))
  }

  const updateTrigger = (key: string, value: any) => {
    const updated = { ...parsedTrigger, [key]: value }
    setForm(f => ({ ...f, trigger_config: JSON.stringify(updated, null, 2) }))
  }

  const handleTest = async () => {
    if (!form.id) return
    setTestLoading(true)
    setTestResult(null)
    try {
      const payload = JSON.parse(testPayload)
      const result = await testFlow.mutateAsync({ id: form.id, payload })
      setTestResult(result)
    } catch (e: any) {
      setTestResult({ error: e.message })
    }
    setTestLoading(false)
  }

  const nodePositions = useMemo(() => {
    // Calculate approximate positions for SVG edges
    const positions: { x: number; y: number }[] = []
    const gap = 220
    const startX = 120
    const y = 60
    positions.push({ x: startX, y }) // trigger
    parsedNodes.forEach((_, i) => {
      positions.push({ x: startX + (i + 1) * gap, y })
    })
    return positions
  }, [parsedNodes.length])

  const inputStyle: React.CSSProperties = {
    background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6,
    padding: '6px 10px', color: 'var(--fg)', fontSize: 12,
    fontFamily: "'JetBrains Mono', monospace", width: '100%',
  }

  const labelStyle: React.CSSProperties = {
    fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' as const,
    letterSpacing: 0.5, marginBottom: 4, display: 'block', fontWeight: 600,
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Toolbar */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 24px', borderBottom: '1px solid var(--border)', flexShrink: 0, background: 'var(--card)' }}>
        <button style={btnStyle} onClick={onCancel}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        </button>
        <div style={{ flex: 1 }}>
          <input
            style={{ background: 'transparent', border: 'none', color: 'var(--fg)', fontSize: 16, fontWeight: 700, padding: 0, fontFamily: 'inherit', width: '100%' }}
            value={form.name}
            onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
            placeholder="Flow name"
          />
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontSize: 12, color: 'var(--muted)' }}>{form.id}</span>
          <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
          <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, cursor: 'pointer' }}>
            <input type="checkbox" checked={form.enabled} onChange={e => setForm(f => ({ ...f, enabled: e.target.checked }))} />
            Enabled
          </label>
          <button style={{ ...btnStyle, borderColor: showJson ? 'var(--cyan)' : 'var(--border)', color: showJson ? 'var(--cyan)' : 'var(--fg)' }} onClick={() => setShowJson(!showJson)}>
            {showJson ? 'Visual' : 'JSON'}
          </button>
          <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={handleSave}>Save</button>
        </div>
      </div>

      {error && (
        <div style={{ background: 'rgba(239,68,68,.1)', borderBottom: '1px solid var(--crit)', color: 'var(--crit)', padding: '10px 24px', fontSize: 13, flexShrink: 0 }}>
          {error}
        </div>
      )}

      {showJson ? (
        <div style={{ flex: 1, padding: 24, overflow: 'auto' }}>
          <textarea
            style={{ ...inputStyle, minHeight: '80%', fontSize: 12, lineHeight: 1.6 }}
            value={JSON.stringify({ id: form.id, name: form.name, enabled: form.enabled, trigger: parsedTrigger, nodes: parsedNodes }, null, 2)}
            onChange={e => {
              try {
                const p = JSON.parse(e.target.value)
                setForm(f => ({ ...f, id: p.id, name: p.name, enabled: p.enabled ?? true, trigger_config: JSON.stringify(p.trigger || {}), nodes: JSON.stringify(p.nodes || []) }))
                setError('')
              } catch { setError('Invalid JSON') }
            }}
          />
        </div>
      ) : (
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          {/* Node Canvas */}
          <div style={{ flex: 1, overflow: 'auto', padding: '32px 24px', position: 'relative' }} ref={canvasRef}>
            <div style={{ display: 'flex', alignItems: 'flex-start', gap: 0, minWidth: 'max-content', position: 'relative', paddingBottom: 40 }}>
              {/* SVG Layer for edges */}
              <svg style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', pointerEvents: 'none', zIndex: 0 }}>
                <defs>
                  <marker id="arrow" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
                    <polygon points="0 0, 8 3, 0 6" fill="#4b5563" />
                  </marker>
                </defs>
                {/* Edges from trigger to nodes and between nodes */}
                {nodePositions.map((pos, i) => {
                  if (i === 0) return null
                  const prev = nodePositions[i - 1]
                  const fromX = prev.x + 140 // right edge of prev card
                  const toX = pos.x - 12     // left edge of next card (accounting for margin)
                  const my = (prev.y + pos.y) / 2 + 28
                  const d = `M ${fromX} ${prev.y + 28} C ${fromX + 40} ${prev.y + 28}, ${toX - 40} ${pos.y + 28}, ${toX} ${pos.y + 28}`
                  return <path key={i} d={d} fill="none" stroke="#4b5563" strokeWidth={2} markerEnd="url(#arrow)" />
                })}
              </svg>

              {/* Trigger Node */}
              <div style={{ position: 'relative', zIndex: 1, marginRight: 24 }}>
                <div style={{
                  width: 160, borderRadius: 10, border: '1px solid var(--border)',
                  background: 'var(--card)', overflow: 'hidden',
                  boxShadow: '0 2px 8px rgba(0,0,0,.15)',
                }}>
                  <div style={{
                    height: 4, background: TRIGGER_META[parsedTrigger.type]?.color || '#9ca3af',
                  }} />
                  <div style={{ padding: '12px 14px' }}>
                    <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: 0.5, fontWeight: 600, marginBottom: 6 }}>Trigger</div>
                    <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 8, display: 'flex', alignItems: 'center', gap: 6 }}>
                      <span style={{
                        width: 8, height: 8, borderRadius: '50%',
                        background: TRIGGER_META[parsedTrigger.type]?.color || '#9ca3af',
                      }} />
                      {TRIGGER_META[parsedTrigger.type]?.label || parsedTrigger.type}
                    </div>
                    <select style={{ ...inputStyle, fontSize: 11 }} value={parsedTrigger.type} onChange={e => {
                      const t = e.target.value
                      setForm(f => ({ ...f, trigger_config: JSON.stringify(TRIGGER_DEFAULTS[t] || { type: t }, null, 2) }))
                    }}>
                      <option value="modbus">Modbus</option>
                      <option value="timer">Timer</option>
                      <option value="mqtt">MQTT</option>
                    </select>
                    {parsedTrigger.type === 'modbus' && (
                      <div style={{ marginTop: 8, display: 'flex', flexDirection: 'column', gap: 6 }}>
                        <input style={{ ...inputStyle, fontSize: 11 }} placeholder="device_id" value={parsedTrigger.device_id || ''} onChange={e => updateTrigger('device_id', e.target.value)} />
                        <input style={{ ...inputStyle, fontSize: 11 }} placeholder="metric (opt)" value={parsedTrigger.metric || ''} onChange={e => updateTrigger('metric', e.target.value)} />
                      </div>
                    )}
                    {parsedTrigger.type === 'timer' && (
                      <div style={{ marginTop: 8 }}>
                        <input style={{ ...inputStyle, fontSize: 11 }} type="number" value={parsedTrigger.interval_ms || 5000} onChange={e => updateTrigger('interval_ms', Number(e.target.value))} />
                      </div>
                    )}
                    {parsedTrigger.type === 'mqtt' && (
                      <div style={{ marginTop: 8 }}>
                        <input style={{ ...inputStyle, fontSize: 11 }} placeholder="topic" value={parsedTrigger.topic || ''} onChange={e => updateTrigger('topic', e.target.value)} />
                      </div>
                    )}
                  </div>
                </div>
                {/* Add node after trigger */}
                <div style={{ display: 'flex', justifyContent: 'center', marginTop: 12 }}>
                  <NodeAddButton onSelect={type => addNode(type, -1)} />
                </div>
              </div>

              {/* Processing Nodes */}
              {parsedNodes.map((node, i) => {
                const meta = NODE_META[node.type] || { label: node.type, color: '#9ca3af', icon: '', desc: '' }
                const isSelected = selectedNode === i
                return (
                  <div key={i} style={{ position: 'relative', zIndex: 1, marginRight: 24 }}>
                    <div
                      style={{
                        width: 180, borderRadius: 10, border: `1px solid ${isSelected ? meta.color : 'var(--border)'}`,
                        background: 'var(--card)', overflow: 'hidden', cursor: 'pointer',
                        boxShadow: isSelected ? `0 0 0 2px ${meta.color}33, 0 4px 12px rgba(0,0,0,.2)` : '0 2px 8px rgba(0,0,0,.15)',
                        transition: 'box-shadow .15s, border-color .15s',
                      }}
                      onClick={() => setSelectedNode(isSelected ? null : i)}
                    >
                      <div style={{ height: 4, background: meta.color }} />
                      <div style={{ padding: '12px 14px' }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={meta.color} strokeWidth="2">
                            <path d={meta.icon} />
                          </svg>
                          <span style={{ fontSize: 12, fontWeight: 700 }}>{meta.label}</span>
                          <button
                            style={{ marginLeft: 'auto', background: 'none', border: 'none', color: 'var(--crit)', cursor: 'pointer', fontSize: 14, padding: 0, lineHeight: 1 }}
                            onClick={e => { e.stopPropagation(); removeNode(i) }}
                            title="Remove"
                          >&times;</button>
                        </div>
                        <div style={{ fontSize: 10, color: 'var(--muted)', marginBottom: 8 }}>{meta.desc}</div>
                        {/* Mini config preview */}
                        <div style={{ fontSize: 10, color: 'var(--muted)', fontFamily: "'JetBrains Mono', monospace", whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                          {node.condition || node.field || node.output_field || node.url || node.message || JSON.stringify(node.map || node.table || {}).slice(0, 40)}
                        </div>
                      </div>
                    </div>
                    {/* Add node after */}
                    <div style={{ display: 'flex', justifyContent: 'center', marginTop: 12 }}>
                      <NodeAddButton onSelect={type => addNode(type, i)} />
                    </div>

                    {/* Expanded config panel */}
                    {isSelected && (
                      <div style={{
                        position: 'absolute', top: '100%', left: 0, marginTop: 8,
                        width: 280, borderRadius: 10, border: '1px solid var(--border)',
                        background: 'var(--bg)', padding: 16, zIndex: 10,
                        boxShadow: '0 8px 24px rgba(0,0,0,.3)',
                      }}>
                        <div style={{ fontSize: 12, fontWeight: 700, marginBottom: 12, color: meta.color }}>{meta.label} Configuration</div>
                        <NodeForm node={node} onChange={(k, v) => updateNode(i, k, v)} />
                      </div>
                    )}
                  </div>
                )
              })}

              {/* Empty state */}
              {parsedNodes.length === 0 && (
                <div style={{ display: 'flex', alignItems: 'center', padding: '20px 40px', color: 'var(--muted)', fontSize: 13 }}>
                  Add a node to build your flow
                </div>
              )}
            </div>
          </div>

          {/* Right Sidebar: Test & Properties */}
          <div style={{ width: 320, borderLeft: '1px solid var(--border)', background: 'var(--card)', overflow: 'auto', flexShrink: 0, display: 'flex', flexDirection: 'column' }}>
            <div style={{ padding: '16px 20px', borderBottom: '1px solid var(--border)' }}>
              <h3 style={{ fontSize: 13, fontWeight: 700, margin: 0, textTransform: 'uppercase', letterSpacing: 0.5 }}>Test Flow</h3>
            </div>
            <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
              <div>
                <label style={labelStyle}>Payload JSON</label>
                <textarea style={{ ...inputStyle, minHeight: 100, resize: 'vertical' }} value={testPayload} onChange={e => setTestPayload(e.target.value)} />
              </div>
              <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)', width: '100%' }} onClick={handleTest} disabled={testLoading || !form.id}>
                {testLoading ? 'Running...' : 'Run Test'}
              </button>
              {testResult && (
                <div style={{ background: 'var(--bg)', borderRadius: 8, padding: 12, fontSize: 11, fontFamily: "'JetBrains Mono', monospace", overflow: 'auto', maxHeight: 300 }}>
                  {testResult.error ? (
                    <div style={{ color: 'var(--crit)' }}>{testResult.error}</div>
                  ) : (
                    <>
                      <div style={{ color: testResult.success ? 'var(--green)' : 'var(--crit)', marginBottom: 8, fontWeight: 600 }}>
                        {testResult.success ? 'SUCCESS' : 'FAILED'}
                      </div>
                      <div style={{ color: 'var(--muted)', marginBottom: 4 }}>Output:</div>
                      <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>{JSON.stringify(testResult.output, null, 2)}</pre>
                      {testResult.node_results?.length > 0 && (
                        <>
                          <div style={{ color: 'var(--muted)', marginTop: 12, marginBottom: 4 }}>Nodes:</div>
                          {testResult.node_results.map((nr: any, idx: number) => (
                            <div key={idx} style={{ marginBottom: 6, padding: 8, borderRadius: 6, background: 'var(--card)', border: '1px solid var(--border)' }}>
                              <div style={{ fontSize: 10, color: 'var(--muted)' }}>#{idx + 1} {nr.node_id} → {nr.result || nr.error}</div>
                              {nr.output && <pre style={{ margin: '4px 0 0', fontSize: 10, whiteSpace: 'pre-wrap' }}>{JSON.stringify(nr.output, null, 2)}</pre>}
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
        </div>
      )}
    </div>
  )
}

/* ── Node Add Button (dropdown) ── */
function NodeAddButton({ onSelect }: { onSelect: (type: FlowNodeType) => void }) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false) }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [])

  return (
    <div ref={ref} style={{ position: 'relative' }}>
      <button
        style={{
          width: 28, height: 28, borderRadius: '50%', border: '2px dashed var(--border)',
          background: 'var(--bg)', color: 'var(--muted)', cursor: 'pointer',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          fontSize: 18, lineHeight: 1, transition: 'all .15s',
        }}
        onMouseEnter={e => { e.currentTarget.style.borderColor = 'var(--green)'; e.currentTarget.style.color = 'var(--green)' }}
        onMouseLeave={e => { e.currentTarget.style.borderColor = 'var(--border)'; e.currentTarget.style.color = 'var(--muted)' }}
        onClick={() => setOpen(!open)}
      >+</button>
      {open && (
        <div style={{
          position: 'absolute', top: 'calc(100% + 8px)', left: '50%', transform: 'translateX(-50%)',
          background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10,
          padding: '8px 0', minWidth: 180, zIndex: 20,
          boxShadow: '0 8px 24px rgba(0,0,0,.3)',
        }}>
          <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: 0.5, padding: '4px 12px 8px', fontWeight: 600 }}>Add Node</div>
          {Object.entries(NODE_META).map(([type, meta]) => (
            <button
              key={type}
              style={{
                display: 'flex', alignItems: 'center', gap: 10, width: '100%', padding: '8px 14px',
                background: 'none', border: 'none', color: 'var(--fg)', cursor: 'pointer',
                fontSize: 12, textAlign: 'left',
              }}
              onMouseEnter={e => { e.currentTarget.style.background = 'var(--bg)' }}
              onMouseLeave={e => { e.currentTarget.style.background = 'none' }}
              onClick={() => { onSelect(type as FlowNodeType); setOpen(false) }}
            >
              <span style={{ width: 8, height: 8, borderRadius: '50%', background: meta.color, flexShrink: 0 }} />
              <div>
                <div style={{ fontWeight: 600 }}>{meta.label}</div>
                <div style={{ fontSize: 10, color: 'var(--muted)' }}>{meta.desc}</div>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

/* ── Node Form (expanded config) ── */
function NodeForm({ node, onChange }: { node: FlowNodeConfig; onChange: (k: string, v: any) => void }) {
  const inputStyle: React.CSSProperties = {
    background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6,
    padding: '6px 10px', color: 'var(--fg)', fontSize: 12,
    fontFamily: "'JetBrains Mono', monospace", width: '100%',
  }
  const labelStyle: React.CSSProperties = {
    fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' as const,
    letterSpacing: 0.5, marginBottom: 4, display: 'block', fontWeight: 600,
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      {Object.entries(node).filter(([k]) => k !== 'id' && k !== 'type').map(([key, value]) => {
        if (typeof value === 'boolean') {
          return (
            <label key={key} style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12, cursor: 'pointer' }}>
              <input type="checkbox" checked={value} onChange={e => onChange(key, e.target.checked)} />
              {key}
            </label>
          )
        }
        if (typeof value === 'number') {
          return (
            <div key={key}>
              <label style={labelStyle}>{key}</label>
              <input style={inputStyle} type="number" value={value} onChange={e => onChange(key, Number(e.target.value))} />
            </div>
          )
        }
        if (typeof value === 'object') {
          return (
            <div key={key}>
              <label style={labelStyle}>{key}</label>
              <textarea
                style={{ ...inputStyle, minHeight: 80, resize: 'vertical' }}
                value={JSON.stringify(value, null, 2)}
                onChange={e => { try { onChange(key, JSON.parse(e.target.value)) } catch { onChange(key, e.target.value) } }}
              />
            </div>
          )
        }
        return (
          <div key={key}>
            <label style={labelStyle}>{key}</label>
            <input style={inputStyle} value={value} onChange={e => onChange(key, e.target.value)} />
          </div>
        )
      })}
    </div>
  )
}

/* ── Main FlowManager ── */
export function FlowManager({ isAdmin }: { isAdmin: boolean }) {
  const [editingFlow, setEditingFlow] = useState<Flow | null>(null)
  const { data: flows = [] } = useFlows(isAdmin)
  const createFlow = useCreateFlow()
  const updateFlow = useUpdateFlow()
  const deleteFlow = useDeleteFlow()
  const startFlow = useStartFlow()
  const stopFlow = useStopFlow()

  const handleCreate = () => {
    setEditingFlow({
      id: `flow_${Date.now()}`,
      name: 'New Flow',
      enabled: true,
      trigger_config: JSON.stringify(TRIGGER_DEFAULTS.modbus, null, 2),
      nodes: JSON.stringify([], null, 2),
      created_at: 0,
      updated_at: 0,
    })
  }

  const handleSave = async (flow: Flow) => {
    if (flows.find(f => f.id === flow.id)) {
      await updateFlow.mutateAsync(flow)
    } else {
      await createFlow.mutateAsync(flow)
    }
    setEditingFlow(null)
  }

  if (editingFlow) {
    return <FlowEditor flow={editingFlow} onSave={handleSave} onCancel={() => setEditingFlow(null)} />
  }

  return (
    <FlowList
      flows={flows}
      onEdit={setEditingFlow}
      onDelete={id => deleteFlow.mutate(id)}
      onStart={id => startFlow.mutate(id)}
      onStop={id => stopFlow.mutate(id)}
      onCreate={handleCreate}
    />
  )
}
