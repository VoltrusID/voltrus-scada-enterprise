import React, { useState } from 'react'
import type { SensorConfig } from '../types'
import type { ScadaEquipmentProps, ScadaConnectionProps, ScadaLabelProps, VisualRule, AttachedLabel } from './engine/types'

type SelectedObject =
  | { kind: 'equipment'; props: ScadaEquipmentProps }
  | { kind: 'connection'; props: ScadaConnectionProps }
  | { kind: 'label'; props: ScadaLabelProps }

interface Props {
  selected: SelectedObject | null
  sensors: SensorConfig[]
  onUpdate: (updates: Record<string, any>) => void
  onClose: () => void
}

const inputStyle: React.CSSProperties = {
  width: '100%', padding: '6px 8px', borderRadius: 4,
  border: '1px solid var(--border)', background: 'var(--bg)',
  color: 'var(--fg)', fontSize: 13, boxSizing: 'border-box',
}

const labelStyle: React.CSSProperties = {
  fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase',
  letterSpacing: '.05em', display: 'block', marginBottom: 4,
}

export const ScadaEditPanel: React.FC<Props> = ({ selected, sensors, onUpdate, onClose }) => {
  if (!selected) return null

  return (
    <div style={{ position: 'absolute', top: 0, right: 0, bottom: 0, width: 300, background: 'var(--card)', borderLeft: '1px solid var(--border)', padding: 16, overflowY: 'auto', zIndex: 20 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <span style={{ fontWeight: 600, fontSize: 14 }}>
          {selected.kind === 'equipment' ? 'Equipment' : selected.kind === 'connection' ? 'Connection' : 'Label'}
        </span>
        <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18 }}>x</button>
      </div>
      {selected.kind === 'equipment' && <EquipmentEditor props={selected.props} sensors={sensors} onUpdate={onUpdate} />}
      {selected.kind === 'connection' && <ConnectionEditor props={selected.props} onUpdate={onUpdate} />}
      {selected.kind === 'label' && <LabelEditor props={selected.props} sensors={sensors} onUpdate={onUpdate} />}
    </div>
  )
}

function EquipmentEditor({ props, sensors, onUpdate }: { props: ScadaEquipmentProps; sensors: SensorConfig[]; onUpdate: (u: Record<string, any>) => void }) {
  const [bindingSearch, setBindingSearch] = useState('')
  const filteredSensors = bindingSearch
    ? sensors.filter(s => s.name.toLowerCase().includes(bindingSearch.toLowerCase()))
    : sensors

  const toggleBinding = (sensorId: number) => {
    const has = props.bindings.some(b => b.sensor_id === sensorId)
    const next = has
      ? props.bindings.filter(b => b.sensor_id !== sensorId)
      : [...props.bindings, { sensor_id: sensorId }]
    onUpdate({ bindings: next })
  }

  const addRule = () => {
    onUpdate({ visual_rules: [...props.visual_rules, { target: 'fill', expression: "if(value > warn, 'red', 'green')" }] })
  }
  const updateRule = (idx: number, patch: Partial<VisualRule>) => {
    const next = [...props.visual_rules]
    next[idx] = { ...next[idx], ...patch }
    onUpdate({ visual_rules: next })
  }
  const removeRule = (idx: number) => {
    onUpdate({ visual_rules: props.visual_rules.filter((_, i) => i !== idx) })
  }
  const addLabel = () => {
    onUpdate({ attached_labels: [...props.attached_labels, { template: '${name}: ${value} ${unit}', offset: [0, -20] as [number, number] }] })
  }
  const updateLabel = (idx: number, patch: Partial<AttachedLabel>) => {
    const next = [...props.attached_labels]
    next[idx] = { ...next[idx], ...patch }
    onUpdate({ attached_labels: next })
  }
  const removeLabel = (idx: number) => {
    onUpdate({ attached_labels: props.attached_labels.filter((_, i) => i !== idx) })
  }

  return (
    <>
      <label style={{ display: 'block', marginBottom: 12 }}>
        <span style={labelStyle}>Name</span>
        <input value={props.name} onChange={e => onUpdate({ name: e.target.value })} style={{ ...inputStyle, marginTop: 0 }} />
      </label>
      <div style={{ marginBottom: 12 }}>
        <span style={labelStyle}>Type</span>
        <div style={{ fontSize: 13, textTransform: 'capitalize' }}>{props.equipment_type.replace(/_/g, ' ')}</div>
      </div>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12, marginBottom: 12 }}>
        <span style={labelStyle}>Sensor Bindings ({props.bindings.length})</span>
        <input placeholder="Search sensors..." value={bindingSearch} onChange={e => setBindingSearch(e.target.value)} style={{ ...inputStyle, fontSize: 12 }} />
        <div style={{ maxHeight: 150, overflowY: 'auto', marginTop: 4 }}>
          {filteredSensors.map(s => {
            const bound = props.bindings.some(b => b.sensor_id === s.id)
            return (
              <div key={s.id} onClick={() => toggleBinding(s.id)} style={{ padding: '3px 6px', borderRadius: 4, cursor: 'pointer', fontSize: 12, display: 'flex', justifyContent: 'space-between', background: bound ? 'rgba(34,197,94,.1)' : 'transparent', marginBottom: 1 }}>
                <span style={{ color: bound ? 'var(--cyan)' : 'var(--fg)' }}>{s.name}</span>
                <span style={{ fontSize: 10, color: 'var(--muted)' }}>{s.unit}</span>
              </div>
            )
          })}
        </div>
      </div>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12, marginBottom: 12 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <span style={labelStyle}>Visual Rules</span>
          <button onClick={addRule} style={{ fontSize: 11, padding: '2px 8px', borderRadius: 3, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer' }}>+ Add</button>
        </div>
        {props.visual_rules.map((rule, i) => (
          <div key={i} style={{ marginBottom: 8, padding: 8, background: 'var(--bg)', borderRadius: 4 }}>
            <select value={rule.target} onChange={e => updateRule(i, { target: e.target.value as VisualRule['target'] })} style={{ ...inputStyle, fontSize: 11, marginBottom: 4 }}>
              <option value="fill">Fill</option>
              <option value="stroke">Stroke</option>
              <option value="opacity">Opacity</option>
              <option value="visible">Visible</option>
            </select>
            <input value={rule.expression} onChange={e => updateRule(i, { expression: e.target.value })} placeholder="if(value > 220, 'red', 'green')" style={{ ...inputStyle, fontSize: 11 }} />
            <button onClick={() => removeRule(i)} style={{ fontSize: 10, color: '#ef4444', background: 'none', border: 'none', cursor: 'pointer', marginTop: 2 }}>Remove</button>
          </div>
        ))}
      </div>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <span style={labelStyle}>Attached Labels</span>
          <button onClick={addLabel} style={{ fontSize: 11, padding: '2px 8px', borderRadius: 3, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer' }}>+ Add</button>
        </div>
        {props.attached_labels.map((lbl, i) => (
          <div key={i} style={{ marginBottom: 8, padding: 8, background: 'var(--bg)', borderRadius: 4 }}>
            <input value={lbl.template} onChange={e => updateLabel(i, { template: e.target.value })} placeholder="${name}: ${value} ${unit}" style={{ ...inputStyle, fontSize: 11, marginBottom: 4 }} />
            <button onClick={() => removeLabel(i)} style={{ fontSize: 10, color: '#ef4444', background: 'none', border: 'none', cursor: 'pointer' }}>Remove</button>
          </div>
        ))}
      </div>
    </>
  )
}

function ConnectionEditor({ props, onUpdate }: { props: ScadaConnectionProps; onUpdate: (u: Record<string, any>) => void }) {
  return (
    <>
      <div style={{ marginBottom: 12 }}>
        <span style={labelStyle}>Type</span>
        <select value={props.connection_type} onChange={e => onUpdate({ connection_type: e.target.value })} style={inputStyle}>
          <option value="power">Power</option>
          <option value="data">Data</option>
          <option value="flow">Flow</option>
        </select>
      </div>
      <div style={{ fontSize: 12, color: 'var(--muted)' }}>
        From: {props.from_id}<br />To: {props.to_id}
      </div>
    </>
  )
}

function LabelEditor({ props, sensors, onUpdate }: { props: ScadaLabelProps; sensors: SensorConfig[]; onUpdate: (u: Record<string, any>) => void }) {
  const addRule = () => {
    onUpdate({ visual_rules: [...props.visual_rules, { target: 'fill', expression: "if(value > warn, 'red', 'green')" }] })
  }
  const updateRule = (idx: number, patch: Partial<VisualRule>) => {
    const next = [...props.visual_rules]
    next[idx] = { ...next[idx], ...patch }
    onUpdate({ visual_rules: next })
  }
  const removeRule = (idx: number) => {
    onUpdate({ visual_rules: props.visual_rules.filter((_, i) => i !== idx) })
  }

  return (
    <>
      <label style={{ display: 'block', marginBottom: 12 }}>
        <span style={labelStyle}>Template</span>
        <input value={props.template} onChange={e => onUpdate({ template: e.target.value })} style={{ ...inputStyle, marginTop: 0 }} placeholder="${name}: ${value} ${unit}" />
      </label>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12, marginBottom: 12 }}>
        <span style={labelStyle}>Sensor Bindings</span>
        {sensors.slice(0, 20).map(s => {
          const bound = props.bindings.some(b => b.sensor_id === s.id)
          return (
            <div key={s.id} onClick={() => {
              const next = props.bindings.some(b => b.sensor_id === s.id)
                ? props.bindings.filter(b => b.sensor_id !== s.id)
                : [...props.bindings, { sensor_id: s.id }]
              onUpdate({ bindings: next })
            }} style={{ padding: '3px 6px', borderRadius: 4, cursor: 'pointer', fontSize: 12, display: 'flex', justifyContent: 'space-between', background: bound ? 'rgba(34,197,94,.1)' : 'transparent', marginBottom: 1 }}>
              <span style={{ color: bound ? 'var(--cyan)' : 'var(--fg)' }}>{s.name}</span>
              <span style={{ fontSize: 10, color: 'var(--muted)' }}>{s.unit}</span>
            </div>
          )
        })}
      </div>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <span style={labelStyle}>Visual Rules</span>
          <button onClick={addRule} style={{ fontSize: 11, padding: '2px 8px', borderRadius: 3, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer' }}>+ Add</button>
        </div>
        {props.visual_rules.map((rule, i) => (
          <div key={i} style={{ marginBottom: 8, padding: 8, background: 'var(--bg)', borderRadius: 4 }}>
            <select value={rule.target} onChange={e => updateRule(i, { target: e.target.value as VisualRule['target'] })} style={{ ...inputStyle, fontSize: 11, marginBottom: 4 }}>
              <option value="fill">Fill</option>
              <option value="stroke">Stroke</option>
              <option value="opacity">Opacity</option>
              <option value="visible">Visible</option>
              <option value="text">Text</option>
            </select>
            <input value={rule.expression} onChange={e => updateRule(i, { expression: e.target.value })} style={{ ...inputStyle, fontSize: 11 }} />
            <button onClick={() => removeRule(i)} style={{ fontSize: 10, color: '#ef4444', background: 'none', border: 'none', cursor: 'pointer', marginTop: 2 }}>Remove</button>
          </div>
        ))}
      </div>
    </>
  )
}
