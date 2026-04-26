import React, { useState } from 'react'
import type { ScadaEquipmentItem, SensorConfig } from '../types'

interface Props {
  equipment: ScadaEquipmentItem | null
  sensors: SensorConfig[]
  onUpdate: (updates: Partial<ScadaEquipmentItem>) => void
  onClose: () => void
}

export const ScadaEditPanel: React.FC<Props> = ({ equipment, sensors, onUpdate, onClose }) => {
  const [bindingSearch, setBindingSearch] = useState('')

  if (!equipment) return null

  const currentBindings: string[] = JSON.parse(equipment.sensor_bindings || '[]')
  const filteredSensors = bindingSearch
    ? sensors.filter(s => s.name.toLowerCase().includes(bindingSearch.toLowerCase()) || s.key.toLowerCase().includes(bindingSearch.toLowerCase()))
    : sensors

  const toggleBinding = (sensorId: number) => {
    const id = String(sensorId)
    const next = currentBindings.includes(id) ? currentBindings.filter(b => b !== id) : [...currentBindings, id]
    onUpdate({ sensor_bindings: JSON.stringify(next) })
  }

  const inputStyle: React.CSSProperties = { width: '100%', padding: '6px 8px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', fontSize: 13, boxSizing: 'border-box' as const }

  return (
    <div style={{ position: 'absolute', top: 0, right: 0, bottom: 0, width: 280, background: 'var(--card)', borderLeft: '1px solid var(--border)', padding: 16, overflowY: 'auto', zIndex: 20 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <span style={{ fontWeight: 600, fontSize: 14 }}>Properties</span>
        <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18 }}>x</button>
      </div>

      <label style={{ display: 'block', marginBottom: 12 }}>
        <span style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em' }}>Name</span>
        <input value={equipment.equipment_name} onChange={e => onUpdate({ equipment_name: e.target.value })} style={{ ...inputStyle, marginTop: 4 }} />
      </label>

      <div style={{ marginBottom: 12 }}>
        <span style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em' }}>Type</span>
        <div style={{ marginTop: 4, fontSize: 13, textTransform: 'capitalize' as const }}>{equipment.equipment_type.replace(/_/g, ' ')}</div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 12 }}>
        <label>
          <span style={{ fontSize: 11, color: 'var(--muted)' }}>X</span>
          <input type="number" value={Math.round(equipment.x)} onChange={e => onUpdate({ x: +e.target.value })} style={{ ...inputStyle, fontSize: 12, padding: '4px 6px' }} />
        </label>
        <label>
          <span style={{ fontSize: 11, color: 'var(--muted)' }}>Y</span>
          <input type="number" value={Math.round(equipment.y)} onChange={e => onUpdate({ y: +e.target.value })} style={{ ...inputStyle, fontSize: 12, padding: '4px 6px' }} />
        </label>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 16 }}>
        <label>
          <span style={{ fontSize: 11, color: 'var(--muted)' }}>Width</span>
          <input type="number" value={Math.round(equipment.width)} onChange={e => onUpdate({ width: Math.max(40, +e.target.value) })} style={{ ...inputStyle, fontSize: 12, padding: '4px 6px' }} />
        </label>
        <label>
          <span style={{ fontSize: 11, color: 'var(--muted)' }}>Height</span>
          <input type="number" value={Math.round(equipment.height)} onChange={e => onUpdate({ height: Math.max(30, +e.target.value) })} style={{ ...inputStyle, fontSize: 12, padding: '4px 6px' }} />
        </label>
      </div>

      <label style={{ display: 'block', marginBottom: 16 }}>
        <span style={{ fontSize: 11, color: 'var(--muted)' }}>Rotation: {equipment.rotation}deg</span>
        <input type="range" min="0" max="360" step="15" value={equipment.rotation} onChange={e => onUpdate({ rotation: +e.target.value })} style={{ width: '100%', marginTop: 4 }} />
      </label>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12 }}>
        <span style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em' }}>Sensor Bindings ({currentBindings.length})</span>
        <input placeholder="Search sensors..." value={bindingSearch} onChange={e => setBindingSearch(e.target.value)} style={{ ...inputStyle, marginTop: 8, fontSize: 12 }} />
        <div style={{ maxHeight: 250, overflowY: 'auto', marginTop: 8 }}>
          {filteredSensors.map(s => {
            const bound = currentBindings.includes(String(s.id))
            return (
              <div key={s.id} onClick={() => toggleBinding(s.id)} style={{ padding: '4px 6px', borderRadius: 4, cursor: 'pointer', fontSize: 12, display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: bound ? 'rgba(34,197,94,.1)' : 'transparent', marginBottom: 2 }}>
                <span style={{ color: bound ? 'var(--cyan)' : 'var(--fg)' }}>{s.name}</span>
                <span style={{ fontSize: 10, color: 'var(--muted)' }}>{s.unit}</span>
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
