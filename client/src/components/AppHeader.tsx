import { useState, useEffect, useCallback } from 'react'
import { btnStyle, displayShortcut } from '../styles'
import { ShortcutsConfigPanel } from '../ShortcutsConfigPanel'
import { HistoryRangeBar } from '../CustomRangePicker'
import type { BrandingSettings } from '../BrandingDialog'
import { ThemeToggle } from './ThemeToggle'
import { LanguageSelector } from './LanguageSelector'
import { NotificationBell } from './NotificationBell'

const useIsMobile = () => {
  const [mobile, setMobile] = useState(window.innerWidth <= 768)
  useEffect(() => {
    const mq = window.matchMedia('(max-width: 768px)')
    const handler = (e: MediaQueryListEvent) => setMobile(e.matches)
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [])
  return mobile
}

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
  mode: 'live' | 'history' | 'system' | 'flows' | 'tags' | 'assets'
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
  onFlowsClick: () => void
  onTagsClick: () => void
  onAssetsClick: () => void
  isAdmin: boolean
  onLogout: () => void
  onAlarmPanelToggle: () => void
  onCommandPanelToggle: () => void
  onExportClick: () => void
  onAnnotationPanelToggle: () => void
  onShiftHandoverToggle: () => void
  onSnapshotsClick: () => void
  onAuditPanelToggle: () => void
  onImportWizardClick: () => void
  onNodeRedWizardClick: () => void
  onMqttPanelClick: () => void
  onEdrPanelClick: () => void
}

export function AppHeader({
  branding, editMode, onBrandingClick,
  activeAlarmCount, effectiveNavShortcuts,
  viewMode, onViewModeChange,
  showShortcutsPanel, onToggleShortcutsPanel, shortcutsPanel,
  mode, onModeChange,
  historyRange, onHistoryRangeChange, historyDetail, onHistoryDetailChange, historyAutoAgg,
  turbo, onToggleTurbo,
  onSettingsClick, onDataManagerClick, onFlowsClick, onTagsClick, isAdmin,
  onLogout, onAlarmPanelToggle, onCommandPanelToggle, onExportClick, onAnnotationPanelToggle, onShiftHandoverToggle,
  onSnapshotsClick, onAuditPanelToggle, onImportWizardClick, onNodeRedWizardClick, onMqttPanelClick, onEdrPanelClick, onAssetsClick,
}: Props) {
  const isMobile = useIsMobile()
  const [drawerOpen, setDrawerOpen] = useState(false)

  const closeDrawer = useCallback(() => setDrawerOpen(false), [])

  useEffect(() => {
    if (!drawerOpen) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') closeDrawer()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [drawerOpen, closeDrawer])

  const drawerItemStyle: React.CSSProperties = {
    display: 'flex', alignItems: 'center', gap: 12,
    width: '100%', padding: '14px 20px', fontSize: 14, fontWeight: 500,
    border: 'none', background: 'none', color: 'var(--text)',
    cursor: 'pointer', textAlign: 'left' as const,
    borderRadius: 0,
  }

  return (
    <>
      <nav role="navigation" aria-label="Main navigation" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '12px 20px', flexShrink: 0, flexWrap: 'wrap', gap: 16 }}>
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
          {/* Notification bell */}
          <NotificationBell authed={true} />
          {/* Alarm bell */}
          <button onClick={onAlarmPanelToggle} aria-label={`Alarms${activeAlarmCount > 0 ? ` (${activeAlarmCount} active)` : ''}`} style={{
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
          {/* Annotation button */}
          <button onClick={onAnnotationPanelToggle} aria-label="Annotations" style={{
            ...btnStyle, position: 'relative',
          }}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
            <span>Notes</span>
          </button>
          {/* Shift Handover button */}
          <button onClick={onShiftHandoverToggle} aria-label="Shift Handover" style={{
            ...btnStyle, position: 'relative',
          }}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>
            <span>Shift</span>
          </button>
          {/* Commands button */}
          <button onClick={onCommandPanelToggle} aria-label="Commands" style={{
            ...btnStyle, position: 'relative',
          }}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
            <span>Cmds</span>
          </button>
          {/* View mode toggle */}
          <div style={{ display: 'flex', borderRadius: 6, overflow: 'hidden', border: '1px solid var(--border)' }}>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'cards' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'cards' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onViewModeChange('cards')}>Grids{effectiveNavShortcuts['view:cards'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:cards'])}</kbd> : null}</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'canvas' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'canvas' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onViewModeChange('canvas')}>Canvas{effectiveNavShortcuts['view:canvas'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:canvas'])}</kbd> : null}</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'twin' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'twin' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onViewModeChange('twin')}>3D{effectiveNavShortcuts['view:twin'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:twin'])}</kbd> : null}</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'scada' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'scada' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onViewModeChange('scada')}>2D{effectiveNavShortcuts['view:scada'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:scada'])}</kbd> : null}</button>
          </div>
          <div style={{ position: 'relative' }}>
            <button onClick={onToggleShortcutsPanel} aria-label="Keyboard shortcuts" style={{ ...btnStyle, padding: '8px 14px', fontSize: 12, color: showShortcutsPanel ? 'var(--cyan)' : 'var(--muted)', background: showShortcutsPanel ? 'rgba(34,197,94,.1)' : 'transparent', border: showShortcutsPanel ? '1px solid var(--cyan)' : '1px solid var(--border)', minHeight: 44 }} title="Keyboard shortcuts">
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
              <button onClick={onToggleTurbo} aria-label={turbo ? 'Disable turbo mode' : 'Enable turbo mode'} style={{
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

          {/* Hamburger button — visible only on mobile */}
          {isMobile && (
            <button
              className="hamburger-menu"
              onClick={() => setDrawerOpen(true)}
              aria-label="Open menu"
              style={{ ...btnStyle, padding: '10px', minWidth: 44, minHeight: 44 }}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
                <line x1="3" y1="6" x2="21" y2="6" />
                <line x1="3" y1="12" x2="21" y2="12" />
                <line x1="3" y1="18" x2="21" y2="18" />
              </svg>
            </button>
          )}

          {/* Desktop-only action buttons */}
          <span className="header-actions-desktop" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <LanguageSelector />
            <ThemeToggle />
            <button style={btnStyle} onClick={onSettingsClick} aria-label="Settings">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
              <span>Settings</span>
            </button>
            <button style={btnStyle} onClick={onDataManagerClick} aria-label="Data manager">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
              <span>Data</span>
            </button>
            <button style={btnStyle} onClick={onExportClick} aria-label="Export data">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
              <span>Export</span>
            </button>
            <button style={btnStyle} onClick={onSnapshotsClick} aria-label="Config snapshots">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
              <span>Snapshots</span>
            </button>
            {isAdmin && (
              <button style={btnStyle} onClick={onAuditPanelToggle} aria-label="Audit log">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
                <span>Audit</span>
              </button>
            )}
            {isAdmin && (
              <button style={btnStyle} onClick={onImportWizardClick} aria-label="Import data">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                <span>Import</span>
              </button>
            )}
            {isAdmin && (
              <button style={btnStyle} onClick={onNodeRedWizardClick} aria-label="Node-RED import">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
                <span>Node-RED</span>
              </button>
            )}
            {isAdmin && (
              <button style={btnStyle} onClick={onMqttPanelClick} aria-label="MQTT broker">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>
                <span>MQTT</span>
              </button>
            )}
            {isAdmin && (
              <button style={btnStyle} onClick={onEdrPanelClick} aria-label="EDR compatibility">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                <span>EDR</span>
              </button>
            )}
            {isAdmin && (
              <button
                aria-label="Assets"
                style={{
                  ...btnStyle,
                  borderColor: mode === 'assets' ? '#a855f7' : 'var(--border)',
                  color: mode === 'assets' ? '#a855f7' : 'var(--fg)',
                  background: mode === 'assets' ? 'rgba(168,85,247,.1)' : 'var(--card)',
                }}
                onClick={onAssetsClick}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>
                <span>Assets</span>
              </button>
            )}
            {isAdmin && (
              <button
                aria-label="Tags"
                style={{
                  ...btnStyle,
                  borderColor: mode === 'tags' ? 'var(--cyan)' : 'var(--border)',
                  color: mode === 'tags' ? 'var(--cyan)' : 'var(--fg)',
                  background: mode === 'tags' ? 'rgba(34,197,94,.1)' : 'var(--card)',
                }}
                onClick={onTagsClick}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                <span>Tags</span>
              </button>
            )}
            {isAdmin && (
              <button
                aria-label="Flows"
                style={{
                  ...btnStyle,
                  borderColor: mode === 'flows' ? '#f59e0b' : 'var(--border)',
                  color: mode === 'flows' ? '#f59e0b' : 'var(--fg)',
                  background: mode === 'flows' ? 'rgba(245,158,11,.1)' : 'var(--card)',
                }}
                onClick={onFlowsClick}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
                <span>Flows</span>
              </button>
            )}
            <div style={{ marginLeft: 'auto' }} />
            <button style={btnStyle} onClick={onLogout} aria-label="Logout">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
              <span>Logout</span>
            </button>
          </span>
        </div>
      </nav>

      {/* Mobile drawer backdrop */}
      {drawerOpen && (
        <div
          role="presentation"
          onClick={closeDrawer}
          aria-hidden="true"
          style={{
            position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)',
            zIndex: 99,
          }}
        />
      )}

      {/* Mobile drawer */}
      {drawerOpen && (
        <div
          role="dialog"
          aria-label="Navigation menu"
          style={{
            position: 'fixed', top: 0, right: 0, bottom: 0, width: 280,
            background: 'var(--card)', borderLeft: '1px solid var(--border)',
            zIndex: 100, display: 'flex', flexDirection: 'column',
            boxShadow: '-4px 0 24px rgba(0,0,0,0.3)',
            animation: 'slideInRight 200ms ease-out',
          }}
        >
          {/* Drawer header */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '16px 20px', borderBottom: '1px solid var(--border)' }}>
            <span style={{ fontSize: 16, fontWeight: 700 }}>Menu</span>
            <button
              onClick={closeDrawer}
              aria-label="Close menu"
              style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', padding: 8, display: 'flex', alignItems: 'center', justifyContent: 'center', minWidth: 44, minHeight: 44 }}
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>

          {/* Drawer items */}
          <div style={{ flex: 1, overflowY: 'auto', paddingTop: 8 }}>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onSettingsClick(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
              Settings
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onDataManagerClick(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
              Data
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onExportClick(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
              Export
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onSnapshotsClick(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
              Snapshots
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onAnnotationPanelToggle(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
              Annotations
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onShiftHandoverToggle(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>
              Shift Handover
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onCommandPanelToggle(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
              Commands
            </button>
            {isAdmin && (
              <button style={drawerItemStyle} onClick={() => { closeDrawer(); onAssetsClick(); }}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>
                Assets
              </button>
            )}
            {isAdmin && (
              <button style={drawerItemStyle} onClick={() => { closeDrawer(); onFlowsClick(); }}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
                Flows
              </button>
            )}
          </div>

          {/* Drawer footer — logout */}
          <div style={{ borderTop: '1px solid var(--border)', padding: '12px 0' }}>
            <button style={{ ...drawerItemStyle, color: 'var(--crit)' }} onClick={() => { closeDrawer(); onLogout(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
              Logout
            </button>
          </div>
        </div>
      )}
    </>
  )
}
