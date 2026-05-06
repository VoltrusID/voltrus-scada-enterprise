import { useState, useEffect, useCallback, useRef } from 'react'
import { btnStyle, displayShortcut } from '../styles'
import { ShortcutsConfigPanel } from '../ShortcutsConfigPanel'
import { HistoryRangeBar } from '../CustomRangePicker'
import type { BrandingSettings } from '../BrandingDialog'
import { UnifiedBell } from './UnifiedBell'

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

/* ── Shared styles ── */
const SEGMENT_BTN: React.CSSProperties = {
  fontFamily: 'Inter,sans-serif',
  fontSize: 12,
  fontWeight: 600,
  padding: '5px 12px',
  border: 'none',
  cursor: 'pointer',
  display: 'flex',
  alignItems: 'center',
  gap: 4,
  background: 'transparent',
  color: 'var(--muted)',
  transition: 'color 0.15s, background 0.15s',
  whiteSpace: 'nowrap',
}

const SEGMENT_ACTIVE_CYAN: React.CSSProperties = {
  ...SEGMENT_BTN,
  color: 'var(--cyan)',
  background: 'var(--card)',
  boxShadow: 'inset 0 0 0 1px var(--border)',
  borderRadius: 6,
}

const SEGMENT_ACTIVE_WARN: React.CSSProperties = {
  ...SEGMENT_BTN,
  color: 'var(--warn)',
  background: 'var(--card)',
  boxShadow: 'inset 0 0 0 1px var(--border)',
  borderRadius: 6,
}

const SEGMENT_GROUP: React.CSSProperties = {
  display: 'flex',
  borderRadius: 8,
  overflow: 'hidden',
  border: '1px solid var(--border)',
  background: 'var(--bg)',
}

const ICON_BTN: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: 34,
  height: 34,
  borderRadius: 8,
  border: '1px solid var(--border)',
  background: 'var(--card)',
  color: 'var(--muted)',
  cursor: 'pointer',
  position: 'relative' as const,
  transition: 'border-color 0.15s, color 0.15s, background 0.15s',
}

const DIVIDER_V: React.CSSProperties = {
  width: 1,
  height: 20,
  background: 'var(--border)',
  margin: '0 6px',
  flexShrink: 0,
}

interface MenuItem {
  label: string
  icon: React.ReactNode
  onClick: () => void
  shortcut?: string
  danger?: boolean
  active?: boolean
  activeColor?: string
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
  onShiftHandoverToggle: () => void
  onSnapshotsClick: () => void
  onAuditPanelToggle: () => void
  onImportWizardClick: () => void
  onNodeRedWizardClick: () => void
  onMqttPanelClick: () => void
  onEdrPanelClick: () => void
  onSystemClick: () => void
}

/* ── Action Menu (dropdown) ── */
function ActionMenu({ items, trigger }: { items: MenuItem[]; trigger: React.ReactNode }) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [open])

  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') setOpen(false) }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open])

  return (
    <div style={{ position: 'relative' }} ref={ref}>
      <div onClick={() => setOpen(o => !o)}>{trigger}</div>
      {open && (
        <div style={{
          position: 'absolute',
          top: 'calc(100% + 6px)',
          right: 0,
          minWidth: 200,
          background: 'var(--card)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          boxShadow: '0 8px 32px rgba(0,0,0,0.35)',
          zIndex: 200,
          padding: '4px 0',
          overflow: 'hidden',
        }}>
          {items.map((item, i) => (
            <button
              key={i}
              onClick={() => { setOpen(false); item.onClick() }}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 10,
                width: '100%',
                padding: '9px 14px',
                fontSize: 13,
                fontWeight: 500,
                border: 'none',
                background: item.active ? `rgba(${item.activeColor === 'purple' ? '168,85,247' : item.activeColor === 'amber' ? '245,158,11' : '34,197,94'},.1)` : 'none',
                color: item.danger ? 'var(--crit)' : item.active
                  ? item.activeColor === 'purple' ? '#a855f7'
                  : item.activeColor === 'amber' ? '#f59e0b'
                  : 'var(--cyan)'
                  : 'var(--fg)',
                cursor: 'pointer',
                textAlign: 'left' as const,
                fontFamily: 'Inter,sans-serif',
                transition: 'background 0.1s',
              }}
              onMouseEnter={e => { if (!item.active) (e.currentTarget.style.background = 'var(--hover-overlay)') }}
              onMouseLeave={e => { if (!item.active) (e.currentTarget.style.background = 'none') }}
            >
              <span style={{ width: 16, height: 16, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>{item.icon}</span>
              <span style={{ flex: 1 }}>{item.label}</span>
              {item.shortcut && (
                <kbd style={{
                  fontFamily: "'JetBrains Mono',monospace", fontSize: 10, padding: '1px 5px',
                  borderRadius: 3, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)',
                }}>{displayShortcut(item.shortcut)}</kbd>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

/* ── Inline SVG icons (small, 14px) ── */
const Icons = {
  bell: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>,
  tag: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>,
  layers: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>,
  db: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>,
  download: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>,
  upload: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>,
  save: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>,
  audit: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>,
  settings: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>,
  mail: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>,
  shield: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>,
  briefcase: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>,
  pen: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>,
  tagAlt: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>,
  clipboard: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>,
  logout: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>,
  bolt: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>,
  grid: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>,
  more: <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></svg>,
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
  onLogout, onAlarmPanelToggle, onCommandPanelToggle, onExportClick, onShiftHandoverToggle,
  onSnapshotsClick, onAuditPanelToggle, onImportWizardClick, onNodeRedWizardClick, onMqttPanelClick, onEdrPanelClick, onAssetsClick, onSystemClick,
}: Props) {
  const isMobile = useIsMobile()
  const [drawerOpen, setDrawerOpen] = useState(false)
  const closeDrawer = useCallback(() => setDrawerOpen(false), [])

  useEffect(() => {
    if (!drawerOpen) return
    const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') closeDrawer() }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [drawerOpen, closeDrawer])

  /* ── Build action menu items ── */
  const actionItems: MenuItem[] = [
    { label: 'Data Manager', icon: Icons.db, onClick: onDataManagerClick },
    { label: 'Export Data', icon: Icons.download, onClick: onExportClick },
    { label: 'Config Snapshots', icon: Icons.save, onClick: onSnapshotsClick },
    { label: 'Shift Handover', icon: Icons.clipboard, onClick: onShiftHandoverToggle },
    { label: 'Commands', icon: Icons.pen, onClick: onCommandPanelToggle },
  ]

  if (isAdmin) {
    actionItems.push(
      { label: 'Tags', icon: Icons.tag, onClick: onTagsClick, active: true, activeColor: 'green' },
      { label: 'Assets', icon: Icons.briefcase, onClick: onAssetsClick, active: true, activeColor: 'purple' },
      { label: 'Flows', icon: Icons.layers, onClick: onFlowsClick, active: true, activeColor: 'amber' },
      { label: 'System', icon: Icons.settings, onClick: onSystemClick, active: true, activeColor: 'green' },
      { label: 'Import Data', icon: Icons.upload, onClick: onImportWizardClick },
      { label: 'Node-RED Import', icon: Icons.layers, onClick: onNodeRedWizardClick },
      { label: 'MQTT Broker', icon: Icons.mail, onClick: onMqttPanelClick },
      { label: 'EDR Compat', icon: Icons.shield, onClick: onEdrPanelClick },
      { label: 'Audit Log', icon: Icons.audit, onClick: onAuditPanelToggle },
    )
  }

  const drawerItemStyle: React.CSSProperties = {
    display: 'flex', alignItems: 'center', gap: 12,
    width: '100%', padding: '14px 20px', fontSize: 14, fontWeight: 500,
    border: 'none', background: 'none', color: 'var(--text)',
    cursor: 'pointer', textAlign: 'left' as const,
    borderRadius: 0,
  }

  /* ── Shared sub-bar style (mobile mode/toggle strip) ── */
  const SUB_BAR: React.CSSProperties = {
    display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8,
    padding: '6px 12px',
    borderBottom: '1px solid var(--border)',
    background: 'var(--card)',
  }

  return (
    <>
      {/* ── Main header bar ── */}
      <nav role="navigation" aria-label="Main navigation" style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: isMobile ? '0 10px' : '0 20px',
        height: isMobile ? 48 : 52,
        flexShrink: 0,
        borderBottom: '1px solid var(--border)',
        background: 'var(--card)',
      }}>
        {/* ── Left: Branding ── */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 0, overflow: 'hidden' }}>
          <div
            style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: editMode ? 'pointer' : 'default' }}
            onClick={onBrandingClick}
          >
            <img src={branding.logo || '/favicon-32.png'} alt={branding.title} style={{ width: isMobile ? 20 : 24, height: isMobile ? 20 : 24, borderRadius: 4, objectFit: 'contain', flexShrink: 0 }} />
            <span style={{ fontSize: isMobile ? 14 : 15, fontWeight: 700, color: 'var(--fg)', whiteSpace: 'nowrap', letterSpacing: '-.01em', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {branding.title}
            </span>
            {!isMobile && branding.subtitle && (
              <span style={{ fontSize: 12, color: 'var(--muted)', fontWeight: 400, whiteSpace: 'nowrap' }}>
                {branding.subtitle}
              </span>
            )}
            {editMode && (
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="var(--muted)" strokeWidth="2" style={{ marginLeft: -2, flexShrink: 0 }}>
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
              </svg>
            )}
          </div>
          {/* Turbo (desktop only — mobile goes in sub-bar) */}
          {!isMobile && viewMode !== 'twin' && viewMode !== 'scada' && mode === 'live' && (
            <button
              onClick={onToggleTurbo}
              title={turbo ? 'Turbo ON' : 'Turbo OFF'}
              style={{
                ...SEGMENT_BTN,
                fontWeight: 600,
                padding: '5px 12px',
                border: '1px solid ' + (turbo ? 'var(--warn)' : 'var(--border)'),
                borderRadius: 6,
                color: turbo ? 'var(--warn)' : 'var(--muted)',
                background: turbo ? 'rgba(234,179,8,.1)' : 'var(--card)',
                marginLeft: 12,
              }}
            >
              Turbo
            </button>
          )}
        </div>

        {/* ── Center: View mode (desktop only) ── */}
        {!isMobile && (
          <div style={SEGMENT_GROUP}>
            <button
              style={viewMode === 'cards' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN}
              onClick={() => onViewModeChange('cards')}
              title="Grid view"
            >Grid</button>
            <button
              style={viewMode === 'canvas' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN}
              onClick={() => onViewModeChange('canvas')}
              title="Canvas view"
            >Canvas</button>
            <button
              style={viewMode === 'twin' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN}
              onClick={() => onViewModeChange('twin')}
              title="3D digital twin"
            >3D</button>
            <button
              style={viewMode === 'scada' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN}
              onClick={() => onViewModeChange('scada')}
              title="2D SCADA diagram"
            >SCADA</button>
          </div>
        )}

        {/* ── Right: actions ── */}
        <div style={{ display: 'flex', alignItems: 'center', gap: isMobile ? 4 : 6, justifyContent: 'flex-end', flexShrink: 0 }}>
          {/* Live/History toggle (desktop only) */}
          {!isMobile && viewMode !== 'twin' && viewMode !== 'scada' && (
            <>
              <div style={SEGMENT_GROUP}>
                <button
                  style={mode === 'live' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN}
                  onClick={() => onModeChange('live')}
                >Live</button>
                <button
                  style={mode === 'history' ? SEGMENT_ACTIVE_WARN : SEGMENT_BTN}
                  onClick={() => onModeChange('history')}
                >History</button>
              </div>

              {/* History range — appears right of Live/History */}
              {mode === 'history' && (
                <HistoryRangeBar range={historyRange} onChange={onHistoryRangeChange} detail={historyDetail} onDetailChange={onHistoryDetailChange} agg={historyAutoAgg} />
              )}
            </>
          )}

          {!isMobile && <div style={DIVIDER_V} />}

          {/* Unified bell */}
          <UnifiedBell authed={true} activeAlarmCount={activeAlarmCount} onOpenAlarmPanel={onAlarmPanelToggle} />

          {/* Keyboard shortcuts (desktop only) */}
          {!isMobile && (
            <div style={{ position: 'relative' }}>
              <button
                onClick={onToggleShortcutsPanel}
                aria-label="Keyboard shortcuts"
                title="Keyboard shortcuts"
                style={{
                  ...ICON_BTN,
                  color: showShortcutsPanel ? 'var(--cyan)' : 'var(--muted)',
                  borderColor: showShortcutsPanel ? 'var(--cyan)' : 'var(--border)',
                  background: showShortcutsPanel ? 'rgba(34,197,94,.1)' : 'var(--card)',
                }}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M8 16h8"/></svg>
              </button>
              {shortcutsPanel}
            </div>
          )}

          {!isMobile && <div style={DIVIDER_V} />}

          {/* Settings (desktop only) */}
          {!isMobile && (
            <button
              onClick={onSettingsClick}
              aria-label="Settings"
              title="Settings"
              style={ICON_BTN}
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            </button>
          )}

          {/* Action Menu (desktop only) */}
          {!isMobile && (
            <ActionMenu
              items={actionItems}
              trigger={
                <button
                  aria-label="More actions"
                  title="More actions"
                  style={ICON_BTN}
                >
                  {Icons.more}
                </button>
              }
            />
          )}

          {/* Hamburger (mobile only) */}
          {isMobile && (
            <button
              className="hamburger-menu"
              onClick={() => setDrawerOpen(true)}
              aria-label="Open menu"
              style={{ ...ICON_BTN, width: 38, height: 38 }}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
                <line x1="3" y1="6" x2="21" y2="6" />
                <line x1="3" y1="12" x2="21" y2="12" />
                <line x1="3" y1="18" x2="21" y2="18" />
              </svg>
            </button>
          )}
        </div>
      </nav>

      {/* ── Mobile sub-bar: view mode + Live/History + Turbo ── */}
      {isMobile && (
        <div style={SUB_BAR}>
          {/* View mode */}
          <div style={{ ...SEGMENT_GROUP, borderRadius: 6 }}>
            <button
              style={{ ...(viewMode === 'cards' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN), padding: '4px 8px', fontSize: 11 }}
              onClick={() => onViewModeChange('cards')}
              title="Grid view"
            >Grid</button>
            <button
              style={{ ...(viewMode === 'canvas' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN), padding: '4px 8px', fontSize: 11 }}
              onClick={() => onViewModeChange('canvas')}
              title="Canvas view"
            >Canvas</button>
            <button
              style={{ ...(viewMode === 'twin' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN), padding: '4px 8px', fontSize: 11 }}
              onClick={() => onViewModeChange('twin')}
              title="3D digital twin"
            >3D</button>
            <button
              style={{ ...(viewMode === 'scada' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN), padding: '4px 8px', fontSize: 11 }}
              onClick={() => onViewModeChange('scada')}
              title="2D SCADA diagram"
            >SCADA</button>
          </div>

          {/* Live/History toggle */}
          {viewMode !== 'twin' && viewMode !== 'scada' && (
            <div style={{ ...SEGMENT_GROUP, borderRadius: 6 }}>
              <button
                style={{ ...(mode === 'live' ? SEGMENT_ACTIVE_CYAN : SEGMENT_BTN), padding: '4px 8px', fontSize: 11 }}
                onClick={() => onModeChange('live')}
              >Live</button>
              <button
                style={{ ...(mode === 'history' ? SEGMENT_ACTIVE_WARN : SEGMENT_BTN), padding: '4px 8px', fontSize: 11 }}
                onClick={() => onModeChange('history')}
              >History</button>
            </div>
          )}

          {/* Turbo */}
          {viewMode !== 'twin' && viewMode !== 'scada' && mode === 'live' && (
            <button
              onClick={onToggleTurbo}
              title={turbo ? 'Turbo ON' : 'Turbo OFF'}
              style={{
                ...SEGMENT_BTN,
                fontSize: 11,
                padding: '4px 10px',
                border: '1px solid ' + (turbo ? 'var(--warn)' : 'var(--border)'),
                borderRadius: 6,
                color: turbo ? 'var(--warn)' : 'var(--muted)',
                background: turbo ? 'rgba(234,179,8,.1)' : 'var(--card)',
              }}
            >
              ⚡
            </button>
          )}
        </div>
      )}

      {/* ── History range bar (mobile) ── */}
      {viewMode !== 'twin' && viewMode !== 'scada' && mode === 'history' && isMobile && (
        <div style={{ padding: '6px 12px', borderBottom: '1px solid var(--border)', background: 'var(--card)', display: 'flex', justifyContent: 'center' }}>
          <HistoryRangeBar range={historyRange} onChange={onHistoryRangeChange} detail={historyDetail} onDetailChange={onHistoryDetailChange} agg={historyAutoAgg} />
        </div>
      )}

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

          <div style={{ flex: 1, overflowY: 'auto', paddingTop: 8 }}>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onToggleShortcutsPanel(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M8 16h8"/></svg>
              <span>Shortcuts</span>
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onSettingsClick(); }}>
              {Icons.settings}<span>Settings</span>
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onDataManagerClick(); }}>
              {Icons.db}<span>Data Manager</span>
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onExportClick(); }}>
              {Icons.download}<span>Export</span>
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onSnapshotsClick(); }}>
              {Icons.save}<span>Snapshots</span>
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onShiftHandoverToggle(); }}>
              {Icons.clipboard}<span>Shift Handover</span>
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onCommandPanelToggle(); }}>
              {Icons.pen}<span>Commands</span>
            </button>
            <button style={drawerItemStyle} onClick={() => { closeDrawer(); onAlarmPanelToggle(); }}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
              <span>Alarms &amp; Notifications</span>
            </button>
            <div style={{ height: 1, background: 'var(--border)', margin: '8px 16px' }} />
            {isAdmin && (
              <>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onAssetsClick(); }}>
                  {Icons.briefcase}<span>Assets</span>
                </button>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onTagsClick(); }}>
                  {Icons.tag}<span>Tags</span>
                </button>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onFlowsClick(); }}>
                  {Icons.layers}<span>Flows</span>
                </button>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onSystemClick(); }}>
                  {Icons.settings}<span>System</span>
                </button>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onImportWizardClick(); }}>
                  {Icons.upload}<span>Import Data</span>
                </button>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onNodeRedWizardClick(); }}>
                  {Icons.layers}<span>Node-RED Import</span>
                </button>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onMqttPanelClick(); }}>
                  {Icons.mail}<span>MQTT Broker</span>
                </button>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onEdrPanelClick(); }}>
                  {Icons.shield}<span>EDR Compat</span>
                </button>
                <button style={drawerItemStyle} onClick={() => { closeDrawer(); onAuditPanelToggle(); }}>
                  {Icons.audit}<span>Audit Log</span>
                </button>
              </>
            )}
          </div>

          <div style={{ borderTop: '1px solid var(--border)', padding: '4px 0' }}>
            <button style={{ ...drawerItemStyle, color: 'var(--crit)' }} onClick={() => { closeDrawer(); onLogout(); }}>
              {Icons.logout}<span>Logout</span>
            </button>
          </div>
        </div>
      )}
    </>
  )
}
