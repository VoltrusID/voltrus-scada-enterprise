import { btnStyle, tabStyle, displayShortcut } from '../styles'
import type { Screen } from '../types'

interface Props {
  mode: 'live' | 'history' | 'system'
  activeScreen: string
  screens: Screen[]
  effectiveNavShortcuts: Record<string, string>
  editMode: boolean
  onTabClick: (screenId: string) => void
  onSystemClick: () => void
  onRenameScreen: (id: string, name: string) => void
  onDeleteScreen: (id: string, name: string) => void
  onAddScreen: () => void
  isAdmin: boolean
  onToggleEditMode: () => void
  editLabel: string
  onAddSource: () => void
  canAddSource: boolean
  onAutoArrange?: () => void
}

export function ScreenTabs({
  mode, activeScreen, screens, effectiveNavShortcuts,
  editMode, onTabClick, onSystemClick,
  onRenameScreen, onDeleteScreen, onAddScreen,
  isAdmin, onToggleEditMode, editLabel, onAddSource, canAddSource,
  onAutoArrange,
}: Props) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 0, marginBottom: 20, borderBottom: '1px solid var(--border)' }}>
      <button style={tabStyle(mode !== 'system' && activeScreen === 'all')} onClick={() => onTabClick('all')}>Overview{effectiveNavShortcuts['screen:all'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '1px 4px', marginLeft: 6, borderRadius: 3, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['screen:all'])}</kbd> : null}</button>
      {screens.map(s => (
        <div key={s.id} style={{ position: 'relative' }}>
          <button style={tabStyle(mode !== 'system' && activeScreen === s.id)} onClick={() => onTabClick(s.id)}>{s.name}{effectiveNavShortcuts[`screen:${s.id}`] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '1px 4px', marginLeft: 6, borderRadius: 3, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts[`screen:${s.id}`])}</kbd> : null}</button>
          {editMode && (
            <div style={{ position: 'absolute', top: 2, right: 2, display: 'flex', gap: 2 }}>
              <button onClick={e => { e.stopPropagation(); onRenameScreen(s.id, s.name); }} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 10, padding: '0 2px', lineHeight: 1 }}>&#9998;</button>
              <button onClick={e => { e.stopPropagation(); onDeleteScreen(s.id, s.name); }} style={{ background: 'none', border: 'none', color: 'var(--crit)', cursor: 'pointer', fontSize: 10, padding: '0 2px', lineHeight: 1 }}>&times;</button>
            </div>
          )}
        </div>
      ))}
      {editMode && <button onClick={onAddScreen} style={{ ...tabStyle(false), color: 'var(--green)' }}>+</button>}
      <button style={{
        ...tabStyle(mode === 'system'),
        color: mode === 'system' ? '#a78bfa' : 'var(--muted)',
        borderBottom: mode === 'system' ? '2px solid #a78bfa' : '2px solid transparent',
      }} onClick={onSystemClick}>System{effectiveNavShortcuts['screen:system'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '1px 4px', marginLeft: 6, borderRadius: 3, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['screen:system'])}</kbd> : null}</button>
      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8 }}>
        {isAdmin && (
        <button style={{ ...btnStyle, borderColor: editMode ? 'var(--cyan)' : 'var(--border)', color: editMode ? 'var(--cyan)' : 'var(--fg)' }} onClick={onToggleEditMode}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
          <span>{editLabel}</span>
        </button>
        )}
        {editMode && canAddSource && <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={onAddSource}>+ Add Source</button>}
        {editMode && onAutoArrange && <button style={{ ...btnStyle, borderColor: '#a78bfa', color: '#a78bfa' }} onClick={onAutoArrange}>Auto Arrange</button>}
      </div>
    </div>
  )
}
