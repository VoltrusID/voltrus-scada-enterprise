import { btnStyle, displayShortcut } from '../styles'
import { ShortcutsConfigPanel } from '../ShortcutsConfigPanel'
import { HistoryRangeBar } from '../CustomRangePicker'
import type { BrandingSettings } from '../BrandingDialog'

interface Props {
  branding: BrandingSettings
  editMode: boolean
  onBrandingClick: () => void
  activeAlarmCount: number
  effectiveNavShortcuts: Record<string, string>
  viewMode: 'cards' | 'canvas' | 'twin' | 'scada'
  onViewModeChange: (mode: 'cards' | 'canvas' | 'twin' | 'scada') => void
  showShortcutsPanel: boolean
  onToggleShortcutsPanel: () => void
  shortcutsPanel: React.ReactNode
  mode: 'live' | 'history' | 'system' | 'flows'
  onModeChange: (mode: 'live' | 'history') => void
  historyRange: string
  onHistoryRangeChange: (range: string) => void
  historyDetail: 'auto' | 'full'
  onHistoryDetailChange: (d: 'auto' | 'full') => void
  historyAutoAgg: string
  turbo: boolean
  onToggleTurbo: () => void
  onSettingsClick: () => void
  onDataManagerClick: () => void
  onLogout: () => void
  onAlarmPanelToggle: () => void
}

export function AppHeader({
  branding, editMode, onBrandingClick,
  activeAlarmCount, effectiveNavShortcuts,
  viewMode, onViewModeChange,
  showShortcutsPanel, onToggleShortcutsPanel, shortcutsPanel,
  mode, onModeChange,
  historyRange, onHistoryRangeChange, historyDetail, onHistoryDetailChange, historyAutoAgg,
  turbo, onToggleTurbo,
  onSettingsClick, onDataManagerClick, onLogout, onAlarmPanelToggle,
}: Props) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '12px 20px', flexShrink: 0, flexWrap: 'wrap', gap: 16 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: editMode ? 'pointer' : 'default' }}
        onClick={onBrandingClick}>
        <img src={branding.logo || '/favicon-32.png'} alt={branding.title} style={{ width: 32, height: 32, borderRadius: 6, objectFit: 'contain' }} />
        <h1 style={{ fontSize: 20, fontWeight: 700 }}>{branding.title} <span style={{ color: 'var(--muted)', fontWeight: 400, fontSize: 14 }}>{branding.subtitle}</span></h1>
        {editMode && (
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--muted)" strokeWidth="2" style={{ marginLeft: -4 }}>
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
          </svg>
        )}
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
        {/* Alarm bell */}
        <button onClick={onAlarmPanelToggle} style={{
          ...btnStyle, position: 'relative', borderColor: activeAlarmCount > 0 ? 'var(--crit)' : 'var(--border)',
          color: activeAlarmCount > 0 ? 'var(--crit)' : 'var(--fg)',
        }}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
          {effectiveNavShortcuts['action:alarms'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['action:alarms'])}</kbd> : null}
          {activeAlarmCount > 0 && <span style={{
            position: 'absolute', top: -4, right: -4, background: 'var(--crit)',
            color: '#fff', borderRadius: '50%', width: 16, height: 16, fontSize: 10,
            display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 700,
          }}>{activeAlarmCount}</span>}
        </button>
        {/* View mode toggle */}
        <div style={{ display: 'flex', borderRadius: 6, overflow: 'hidden', border: '1px solid var(--border)' }}>
          <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'cards' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'cards' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onViewModeChange('cards')}>Grids{effectiveNavShortcuts['view:cards'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:cards'])}</kbd> : null}</button>
          <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'canvas' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'canvas' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onViewModeChange('canvas')}>Canvas{effectiveNavShortcuts['view:canvas'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:canvas'])}</kbd> : null}</button>
          <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'twin' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'twin' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onViewModeChange('twin')}>3D{effectiveNavShortcuts['view:twin'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:twin'])}</kbd> : null}</button>
          <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'scada' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'scada' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onViewModeChange('scada')}>2D{effectiveNavShortcuts['view:scada'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:scada'])}</kbd> : null}</button>
        </div>
        <div style={{ position: 'relative' }}>
          <button onClick={onToggleShortcutsPanel} style={{ ...btnStyle, padding: '6px 8px', fontSize: 12, color: showShortcutsPanel ? 'var(--cyan)' : 'var(--muted)', background: showShortcutsPanel ? 'rgba(34,197,94,.1)' : 'transparent', border: showShortcutsPanel ? '1px solid var(--cyan)' : '1px solid var(--border)' }} title="Keyboard shortcuts">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M8 16h8"/></svg>
          </button>
          {shortcutsPanel}
        </div>
        {/* Live/History toggle (hidden in 3D/2D mode) */}
        {viewMode !== 'twin' && viewMode !== 'scada' && (<>
          <div style={{ display: 'flex', borderRadius: 6, overflow: 'hidden', border: '1px solid var(--border)' }}>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'live' ? 'var(--cyan)' : 'var(--fg)', background: mode === 'live' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onModeChange('live')}>Live{effectiveNavShortcuts['mode:live'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['mode:live'])}</kbd> : null}</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'history' ? 'var(--warn)' : 'var(--fg)', background: mode === 'history' ? 'rgba(234,179,8,.1)' : 'var(--card)' }} onClick={() => onModeChange('history')}>History{effectiveNavShortcuts['mode:history'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['mode:history'])}</kbd> : null}</button>
          </div>
          {mode === 'history' && (
            <HistoryRangeBar range={historyRange} onChange={onHistoryRangeChange} detail={historyDetail} onDetailChange={onHistoryDetailChange} agg={historyAutoAgg} />
          )}
          {mode === 'live' && (
            <button onClick={onToggleTurbo} style={{
              ...btnStyle,
              borderColor: turbo ? 'var(--warn)' : 'var(--border)',
              color: turbo ? 'var(--warn)' : 'var(--fg)',
              background: turbo ? 'rgba(234,179,8,.1)' : 'var(--card)',
            }} title={turbo ? 'Turbo ON — rendering at SSE rate' : 'Turbo OFF — throttled to 1 Hz'}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 4 }}><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
              Turbo
            </button>
          )}
        </>)}
        <button style={btnStyle} onClick={onSettingsClick}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
          <span>Settings</span>
        </button>
        <button style={btnStyle} onClick={onDataManagerClick}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
          <span>Data</span>
        </button>
        <div style={{ marginLeft: 'auto' }} />
        <button style={btnStyle} onClick={onLogout}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
          <span>Logout</span>
        </button>
      </div>
    </div>
  )
}
