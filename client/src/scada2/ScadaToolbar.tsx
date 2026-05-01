import React from 'react'
import type { ScadaTool } from './engine/types'

interface Props {
  editMode: boolean
  activeTool: ScadaTool
  onToggleEdit: () => void
  onToolChange: (tool: ScadaTool) => void
  onSave: () => void
  hasChanges: boolean
  onZoomIn: () => void
  onZoomOut: () => void
  onFitView: () => void
}

const btn = (active: boolean): React.CSSProperties => ({
  padding: '6px 12px', borderRadius: 4,
  border: `1px solid ${active ? 'var(--cyan)' : 'var(--border)'}`,
  background: active ? 'rgba(34,197,94,.1)' : 'var(--card)',
  color: active ? 'var(--cyan)' : 'var(--fg)',
  cursor: 'pointer', fontSize: 12, fontWeight: 500,
})

export const ScadaToolbar: React.FC<Props> = ({
  editMode, activeTool, onToggleEdit, onToolChange, onSave, hasChanges, onZoomIn, onZoomOut, onFitView,
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
        <button style={btn(activeTool === 'label')} onClick={() => onToolChange('label')}>Label</button>
        <button style={btn(activeTool === 'delete')} onClick={() => onToolChange('delete')}>Delete</button>
        <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
        <button onClick={onSave} disabled={!hasChanges} style={{
          padding: '6px 14px', borderRadius: 4,
          border: `1px solid ${hasChanges ? 'var(--cyan)' : 'var(--border)'}`,
          background: hasChanges ? 'var(--cyan)' : 'var(--card)',
          color: hasChanges ? '#fff' : 'var(--muted)',
          cursor: hasChanges ? 'pointer' : 'not-allowed',
          fontSize: 12, fontWeight: 600,
        }}>Save Layout</button>
      </>
    )}
    <div style={{ flex: 1 }} />
    <button onClick={onZoomIn} style={btn(false)}>+</button>
    <button onClick={onZoomOut} style={btn(false)}>-</button>
    <button onClick={onFitView} style={btn(false)}>Fit</button>
  </div>
)
