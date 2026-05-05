import React, { useState, useMemo } from 'react'
import type { SensorConfig } from '../types'

interface Props {
  visible: boolean
  sensors: SensorConfig[]
  onSelect: (sensorId: number) => void
}

export const ScadaPalette: React.FC<Props> = ({ visible, sensors, onSelect }) => {
  const [search, setSearch] = useState('')

  const filtered = useMemo(() => {
    if (!search) return sensors
    const q = search.toLowerCase()
    return sensors.filter(s =>
      s.name.toLowerCase().includes(q) || s.key.toLowerCase().includes(q),
    )
  }, [sensors, search])

  if (!visible) return null

  return (
    <div style={{
      position: 'absolute', top: 8, left: 8, zIndex: 10,
      background: 'var(--card)', border: '1px solid var(--border)',
      borderRadius: 8, padding: 8, width: 220, maxHeight: 300,
      overflowY: 'auto', boxShadow: '0 4px 12px rgba(0,0,0,.15)',
    }}>
      <input
        type="text"
        placeholder="Search tags..."
        value={search}
        onChange={e => setSearch(e.target.value)}
        style={{
          width: '100%', padding: '6px 8px', borderRadius: 6,
          border: '1px solid var(--border)', background: 'var(--bg)',
          color: 'var(--fg)', fontSize: 12, marginBottom: 6,
          outline: 'none', boxSizing: 'border-box',
        }}
      />
      <div style={{ fontSize: 10, color: 'var(--muted)', marginBottom: 6, textTransform: 'uppercase', letterSpacing: '.05em' }}>
        Drag a tag to the canvas
      </div>
      {filtered.map(s => (
        <div
          key={s.id}
          draggable
          onDragStart={e => {
            e.dataTransfer.setData('text/plain', String(s.id))
            e.dataTransfer.effectAllowed = 'copy'
          }}
          onClick={() => onSelect(s.id)}
          style={{
            display: 'flex', alignItems: 'center', gap: 8,
            padding: '5px 6px', borderRadius: 6, cursor: 'grab',
            marginBottom: 2,
            transition: 'background 0.1s',
          }}
          onMouseEnter={e => e.currentTarget.style.background = 'var(--hover-overlay)'}
          onMouseLeave={e => e.currentTarget.style.background = 'transparent'}
        >
          <span style={{
            width: 8, height: 8, borderRadius: '50%', flexShrink: 0,
            background: s.color || 'var(--muted)',
          }} />
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ fontSize: 12, color: 'var(--fg)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {s.name}
            </div>
            <div style={{ fontSize: 10, color: 'var(--muted)' }}>
              {s.unit || '—'}
            </div>
          </div>
        </div>
      ))}
      {filtered.length === 0 && (
        <div style={{ fontSize: 12, color: 'var(--muted)', padding: '8px 0', textAlign: 'center' }}>
          No tags found
        </div>
      )}
    </div>
  )
}
