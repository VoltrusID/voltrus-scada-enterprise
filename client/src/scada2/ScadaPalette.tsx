import React from 'react'
import DOMPurify from 'dompurify'
import { SCADA_EQUIPMENT_TYPES, type ScadaEquipmentType } from '../types'
import { SYMBOLS } from './symbols'

interface Props {
  visible: boolean
  onSelect: (type: ScadaEquipmentType) => void
  selectedType: ScadaEquipmentType | null
}

export const ScadaPalette: React.FC<Props> = ({ visible, onSelect, selectedType }) => {
  if (!visible) return null
  return (
    <div style={{
      position: 'absolute', top: 8, left: 8, zIndex: 10,
      background: 'var(--card)', border: '1px solid var(--border)',
      borderRadius: 8, padding: 8, display: 'flex', gap: 6, flexWrap: 'wrap',
      maxWidth: 360, maxHeight: 200, overflowY: 'auto',
      boxShadow: '0 2px 8px rgba(0,0,0,.08)',
    }}>
      {SCADA_EQUIPMENT_TYPES.map(({ value, label }) => {
        const color = selectedType === value ? 'var(--cyan)' : '#999'
        const svgHtml = DOMPurify.sanitize(
          SYMBOLS[value].svg
            .replace(/currentColor/g, color)
            .replace(/width="[^"]*"/, 'width="28"')
            .replace(/height="[^"]*"/, 'height="20"'),
        )
        return (
          <div key={value} onClick={() => onSelect(value)} style={{
            display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4,
            padding: '4px 6px', borderRadius: 6, cursor: 'pointer', minWidth: 52,
            border: `1px solid ${selectedType === value ? 'var(--cyan)' : 'transparent'}`,
            background: selectedType === value ? 'rgba(34,197,94,.1)' : 'transparent',
          }}>
            <div style={{ width: 28, height: 20 }} dangerouslySetInnerHTML={{ __html: svgHtml }} />
            <span style={{ fontSize: 8, color: selectedType === value ? 'var(--cyan)' : 'var(--muted)', textAlign: 'center', lineHeight: 1.2 }}>{label}</span>
          </div>
        )
      })}
    </div>
  )
}
