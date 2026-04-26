import React from 'react'

type Tool = 'select' | 'connect' | 'delete'

interface Props {
  editMode: boolean
  activeTool: Tool
  onToggleEdit: () => void
  onToolChange: (tool: Tool) => void
  onSave: () => void
  hasChanges: boolean
}

const toolBtnStyle = (active: boolean): React.CSSProperties => ({
  padding: '6px 12px', borderRadius: 4,
  border: `1px solid ${active ? 'var(--cyan)' : 'var(--border)'}`,
  background: active ? 'rgba(34,197,94,.1)' : 'var(--card)',
  color: active ? 'var(--cyan)' : 'var(--fg)',
  cursor: 'pointer', fontSize: 12, fontWeight: 500,
})

export const ScadaEditToolbar: React.FC<Props> = ({ editMode, activeTool, onToggleEdit, onToolChange, onSave, hasChanges }) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
    <button onClick={onToggleEdit} style={{ ...toolBtnStyle(editMode), borderColor: editMode ? 'var(--cyan)' : 'var(--border)' }}>
      {editMode ? 'Done' : 'Edit'}
    </button>
    {editMode && (
      <>
        <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
        <button style={toolBtnStyle(activeTool === 'select')} onClick={() => onToolChange('select')}>Select</button>
        <button style={toolBtnStyle(activeTool === 'connect')} onClick={() => onToolChange('connect')}>Connect</button>
        <button style={toolBtnStyle(activeTool === 'delete')} onClick={() => onToolChange('delete')}>Delete</button>
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
  </div>
)
