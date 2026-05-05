import React from 'react'
import type { ScadaTool } from './widgets/types'

interface Props {
  editMode: boolean
  activeTool: ScadaTool
  onToggleEdit: () => void
  onToolChange: (tool: ScadaTool) => void
  onSave: () => void
  hasChanges: boolean
}

const btn = (active: boolean): React.CSSProperties => ({
  padding: '6px 12px',
  borderRadius: 6,
  border: `1px solid ${active ? 'var(--cyan)' : 'var(--border)'}`,
  background: active ? 'rgba(34,197,94,.1)' : 'var(--card)',
  color: active ? 'var(--cyan)' : 'var(--fg)',
  cursor: 'pointer',
  fontSize: 12,
  fontWeight: 500,
  transition: 'border-color 0.15s, color 0.15s',
})

export const ScadaToolbar: React.FC<Props> = ({
  editMode, activeTool, onToggleEdit, onToolChange, onSave, hasChanges,
}) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
    <button onClick={onToggleEdit} style={btn(editMode)}>
      {editMode ? 'Done' : 'Edit'}
    </button>
    {editMode && (
      <>
        <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
        <button style={btn(activeTool === 'select')} onClick={() => onToolChange('select')}>Select</button>
        <button style={btn(activeTool === 'connect')} onClick={() => onToolChange('connect')}>Connect</button>
        <button style={btn(activeTool === 'delete')} onClick={() => onToolChange('delete')}>Delete</button>
        <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
        <button onClick={onSave} disabled={!hasChanges} style={{
          padding: '6px 14px', borderRadius: 6,
          border: `1px solid ${hasChanges ? 'var(--cyan)' : 'var(--border)'}`,
          background: hasChanges ? 'var(--cyan)' : 'var(--card)',
          color: hasChanges ? '#fff' : 'var(--muted)',
          cursor: hasChanges ? 'pointer' : 'not-allowed',
          fontSize: 12, fontWeight: 600,
        }}>Save Layout</button>
      </>
    )}
  </div>
)
