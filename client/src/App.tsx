import { useState, useCallback, useMemo, useEffect, useRef } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { SensorConfig, ScreenLayouts, CanvasLayoutItem, LayoutItem } from './types'
import { LoginScreen } from './LoginScreen'
import { DashboardSettings } from './DashboardSettings'
import { SensorDetail } from './SensorDetail'
import { DataManager } from './DataManager'
import { AddSensorDialog } from './AddSensorDialog'
import { ConfirmDialog, PromptDialog } from './ScreenDialog'
import { useAlarmAudio } from './useAlarmAudio'
import { AlarmPanel } from './AlarmPanel'
import { ShortcutsConfigPanel } from './ShortcutsConfigPanel'
import { BrandingDialog, DEFAULT_BRANDING, type BrandingSettings } from './BrandingDialog'
import { AppHeader } from './components/AppHeader'
import { ScreenTabs } from './components/ScreenTabs'
import { SystemPanel } from './components/SystemPanel'
import { LayoutBreakpointSelector } from './components/LayoutBreakpointSelector'
import { Footer } from './components/Footer'
import { MainContent } from './components/MainContent'
import { useDashboardData } from './hooks/useDashboardData'
import { useKeyboardNavigation } from './hooks/useKeyboardNavigation'
import { useDashboardSystemInfo } from './store/dashboardStore'
import { createDashboardStore, DashboardStoreProvider } from './store/dashboardStore'
import { useSensors, useScreenSensors, useScreens, useLayouts, useTwinLayouts, useTwinConnections, useCanvasLayouts, useShortcuts, useNavShortcuts, useBranding, useActiveAlarmCount, useUpdateSensor, useDeleteSensor, useAddSensor, useSaveLayout, useSaveTwinLayout, useSaveTwinConnections, useSaveCanvasLayout, useSaveShortcuts, useSaveNavShortcuts, useSaveBranding, useCreateScreen, useRenameScreen, useDeleteScreen, useAddSensorToScreen, useRemoveSensorFromScreen, useScadaEquipment, useScadaConnections, useSaveScadaEquipment, useSaveScadaConnections, fetchOk } from './queries'

function AppInner() {
  const queryClient = useQueryClient()

  // ── Auth state ───────────────────────────────────────────────────
  const [authed, setAuthed] = useState(false)
  const [authRole, setAuthRole] = useState<string>('user')
  const [authLoading, setAuthLoading] = useState(true)
  const [loginError, setLoginError] = useState('')

  // ── UI state ─────────────────────────────────────────────────────
  const [editMode, setEditMode] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [dashSettings, setDashSettings] = useState({ gap: 12, refreshInterval: 1000, historyAutoAgg: 'sampling' as const })
  const [showDashSettings, setShowDashSettings] = useState(false)
  const [selectedSensor, setSelectedSensor] = useState<number | null>(null)
  const [activeScreen, setActiveScreen] = useState<string>(() => new URLSearchParams(window.location.search).get('screen') ?? 'all')
  const [mode, setMode] = useState<'live' | 'history' | 'system'>(() => {
    const m = new URLSearchParams(window.location.search).get('mode')
    return (m === 'history' || m === 'system') ? m : 'live'
  })
  const [historyRange, setHistoryRange] = useState('30m')
  const [historyDetail, setHistoryDetail] = useState<'auto' | 'full'>('auto')
  const [showDataManager, setShowDataManager] = useState(false)
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [screenPrompt, setScreenPrompt] = useState<{ title: string; label: string; placeholder?: string; initialValue?: string; shortcutValue?: string; okLabel?: string; onSubmit: (v: string, shortcut?: string | null) => void } | null>(null)
  const [screenConfirm, setScreenConfirm] = useState<{ title: string; message: string; onConfirm: () => void } | null>(null)
  const [showBrandingDialog, setShowBrandingDialog] = useState(false)
  const [layoutBreakpoint, setLayoutBreakpoint] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<'cards' | 'canvas' | 'twin' | 'scada'>(() => {
    const v = new URLSearchParams(window.location.search).get('view')
    return (v === 'canvas' || v === 'twin' || v === 'scada') ? v : 'cards'
  })
  const [localLayouts, setLocalLayouts] = useState<ScreenLayouts | null>(null)
  const [localCanvasLayouts, setLocalCanvasLayouts] = useState<CanvasLayoutItem[] | null>(null)
  const [showShortcutsPanel, setShowShortcutsPanel] = useState(false)
  const [showAlarmPanel, setShowAlarmPanel] = useState(false)
  const [alarmMuted, setAlarmMuted] = useState(false)
  const [turbo, setTurbo] = useState(false)

  // ── URL sync ─────────────────────────────────────────────────────
  useEffect(() => {
    const p = new URLSearchParams(window.location.search)
    p.set('mode', mode)
    p.set('screen', activeScreen)
    p.set('view', viewMode)
    window.history.replaceState(null, '', `${window.location.pathname}?${p}`)
  }, [mode, activeScreen, viewMode])

  // ── Query hooks ──────────────────────────────────────────────────
  const { data: sensors = [] } = useSensors(authed)
  const { data: screenSensors = [] } = useScreenSensors(activeScreen, authed)
  const { data: screens = [] } = useScreens(authed)
  const { data: serverLayouts } = useLayouts(activeScreen, authed)
  const screenLayouts = localLayouts ?? (serverLayouts ?? { lg: [], md: [], sm: [] })
  const { data: twinLayoutsData } = useTwinLayouts(activeScreen, authed)
  const twinLayouts = twinLayoutsData?.items ?? []
  const twinCamera = twinLayoutsData?.camera
  const { data: twinConnections = [] } = useTwinConnections(activeScreen, authed)
  const { data: scadaEquipment = [] } = useScadaEquipment(activeScreen, authed && viewMode === 'scada')
  const { data: scadaConnections = [] } = useScadaConnections(activeScreen, authed && viewMode === 'scada')
  const { data: serverCanvasLayouts = [] } = useCanvasLayouts(activeScreen, authed)
  const canvasLayouts = localCanvasLayouts ?? serverCanvasLayouts
  const { data: shortcuts = {} } = useShortcuts(authed)
  const { data: navShortcuts = {} } = useNavShortcuts(authed)
  const { data: branding = DEFAULT_BRANDING } = useBranding()
  const { data: activeAlarms = { count: 0 } } = useActiveAlarmCount(authed)

  // ── Mutation hooks ───────────────────────────────────────────────
  const updateSensorMut = useUpdateSensor()
  const deleteSensorMut = useDeleteSensor()
  const addSensorMut = useAddSensor()
  const saveLayoutMut = useSaveLayout()
  const saveTwinLayoutMut = useSaveTwinLayout()
  const saveTwinConnectionsMut = useSaveTwinConnections()
  const saveCanvasLayoutMut = useSaveCanvasLayout()
  const saveScadaEquipmentMut = useSaveScadaEquipment()
  const saveScadaConnectionsMut = useSaveScadaConnections()
  const saveShortcutsMut = useSaveShortcuts()
  const saveNavShortcutsMut = useSaveNavShortcuts()
  const saveBrandingMut = useSaveBranding()
  const createScreenMut = useCreateScreen()
  const renameScreenMut = useRenameScreen()
  const deleteScreenMut = useDeleteScreen()
  const addSensorToScreenMut = useAddSensorToScreen()
  const removeSensorFromScreenMut = useRemoveSensorFromScreen()

  // ── Dashboard data hook (now store-backed) ───────────────────────
  const { historyReady, sseConnected, systemInfo } = useDashboardData(authed, mode, activeScreen, sensors, screenSensors, historyRange, historyDetail, dashSettings.historyAutoAgg, turbo)
  const storeSystemInfo = useDashboardSystemInfo()
  const effectiveSystemInfo = systemInfo ?? storeSystemInfo

  // ── Derived values ───────────────────────────────────────────────
  const isAdmin = authRole === 'admin'

  const effectiveNavShortcuts = useMemo(() => {
    const defaults: Record<string, string> = {
      'screen:all': 'Mod+1', 'screen:system': 'Mod+0',
      'view:cards': '', 'view:canvas': '', 'view:twin': '', 'view:scada': '',
      'mode:live': '', 'mode:history': '', 'action:alarms': '',
    }
    screens.forEach((s, i) => { defaults[`screen:${s.id}`] = `Mod+${i + 2}` })
    Object.entries(navShortcuts).forEach(([k, v]) => { if (v) defaults[k] = v })
    return defaults
  }, [screens, navShortcuts])

  const DEFAULT_COLS: Record<string, number> = { lg: 24, md: 12, sm: 4 }
  function generateDefaultLayout(sensorIds: number[], bp: string): LayoutItem[] {
    const cols = DEFAULT_COLS[bp] || 24
    const defaultW = bp === 'lg' ? 6 : bp === 'md' ? 6 : 4
    const defaultH = 4
    let x = 0, y = 0
    return sensorIds.map(id => {
      if (x + defaultW > cols) { x = 0; y += defaultH }
      const item = { i: String(id), x, y, w: defaultW, h: defaultH }
      x += defaultW
      return item
    })
  }

  const CHART_SIZES: Record<string, { w: number; h: number }> = {
    gauge: { w: 4, h: 4 },
    sparkline: { w: 6, h: 3 },
    bignumber: { w: 4, h: 3 },
    image: { w: 6, h: 4 },
    text: { w: 6, h: 4 },
  }

  function autoArrangeLayout(sensors: SensorConfig[], bp: string): LayoutItem[] {
    const cols = DEFAULT_COLS[bp] || 24
    const groups = new Map<string, SensorConfig[]>()
    for (const s of sensors) {
      const key = s.device_id || '_'
      if (!groups.has(key)) groups.set(key, [])
      groups.get(key)!.push(s)
    }

    const items: LayoutItem[] = []
    let globalY = 0

    for (const [, group] of groups) {
      let x = 0, rowY = globalY, maxRowH = 0
      for (const s of group) {
        const ct = s.chart_type || 'sparkline'
        let { w, h } = CHART_SIZES[ct] || { w: 6, h: 4 }
        if (bp === 'sm') { w = Math.min(w, 4); h = Math.max(h, 3) }
        if (x + w > cols) { x = 0; rowY += maxRowH; maxRowH = 0 }
        items.push({ i: String(s.id), x, y: rowY, w, h })
        maxRowH = Math.max(maxRowH, h)
        x += w
      }
      globalY = rowY + maxRowH
    }
    return items
  }

  function autoArrangeAll(sensors: SensorConfig[]): ScreenLayouts {
    return { lg: autoArrangeLayout(sensors, 'lg'), md: autoArrangeLayout(sensors, 'md'), sm: autoArrangeLayout(sensors, 'sm') }
  }

  const defaultLayouts = useMemo(() => {
    if (screenLayouts.lg.length > 0) return null
    const ids = screenSensors.map(s => s.id)
    if (ids.length === 0) return null
    return { lg: generateDefaultLayout(ids, 'lg'), md: generateDefaultLayout(ids, 'md'), sm: generateDefaultLayout(ids, 'sm') }
  }, [screenSensors, screenLayouts])

  const effectiveLayouts = defaultLayouts ?? screenLayouts

  // ── Auth ─────────────────────────────────────────────────────────
  useEffect(() => {
    fetch('/api/auth/me').then(r => r.json()).then(d => { setAuthed(d.authenticated === true); setAuthRole(d.role || 'user'); setAuthLoading(false) }).catch(() => { setAuthed(false); setAuthRole('user'); setAuthLoading(false) })
  }, [])

  const handleLogin = async (username: string, password: string) => {
    setLoginError('')
    const r = await fetch('/api/auth/login', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ username, password }) })
    if (r.ok) { const d = await r.json(); setAuthed(true); setAuthRole(d.role || 'user') }
    else { const d = await r.json(); setLoginError(d.error || 'Login failed') }
  }

  const handleLogout = async () => { await fetch('/api/auth/logout', { method: 'POST' }); setAuthed(false); setAuthRole('user') }

  // ── Handlers ─────────────────────────────────────────────────────
  const handleSaveBranding = (settings: BrandingSettings) => saveBrandingMut.mutate(settings)

  const handleSave = async (s: SensorConfig, shortcut?: string | null) => {
    await updateSensorMut.mutateAsync(s)
    if (shortcut !== undefined) {
      const prev = shortcuts[s.id] ?? null
      if (shortcut !== prev) {
        await saveShortcutsMut.mutateAsync({ ...shortcuts, [s.id]: shortcut })
      }
    }
    setEditingId(null)
  }

  const handleDelete = async (id: number) => {
    await deleteSensorMut.mutateAsync(id)
    setEditingId(null)
  }

  const handleAddSensorToManager = async (form: Partial<SensorConfig>) => {
    await addSensorMut.mutateAsync(form)
  }

  const handleDuplicateSensor = async (s: SensorConfig) => {
    const dup = {
      key: s.key + '_copy', name: s.name + ' (Copy)', unit: s.unit,
      min_val: s.min_val, max_val: s.max_val, warn_val: s.warn_val, crit_val: s.crit_val,
      base_val: s.base_val, decimals: s.decimals, source_type: s.source_type,
      color: s.color, chart_type: s.chart_type, source_config: s.source_config,
    }
    await addSensorMut.mutateAsync(dup)
  }

  const handleLayoutChange = useCallback((currentLayout: any[], allLayouts: { [bp: string]: any[] }) => {
    const normalize = (items: any[]) => items.map((l: any) => ({ i: String(l.i), x: l.x, y: l.y, w: l.w, h: l.h }))
    const newLayouts: ScreenLayouts = {
      lg: allLayouts.lg?.length > 0 ? normalize(allLayouts.lg) : [],
      md: allLayouts.md?.length > 0 ? normalize(allLayouts.md) : [],
      sm: allLayouts.sm?.length > 0 ? normalize(allLayouts.sm) : [],
    }
    setLocalLayouts(prev => {
      const next = JSON.stringify(newLayouts)
      if (prev && JSON.stringify(prev) === next) return prev
      return newLayouts
    })
  }, [])

  const handleCanvasLayoutChange = useCallback((items: CanvasLayoutItem[]) => {
    setLocalCanvasLayouts(items)
  }, [])

  const handleAutoArrange = useCallback(() => {
    setLocalLayouts(autoArrangeAll(screenSensors))
  }, [screenSensors])

  const handleAddScreen = () => {
    const defaultShortcut = `Mod+${screens.length + 2}`
    setScreenPrompt({
      title: 'New Screen', label: 'Screen name', placeholder: 'My Screen', shortcutValue: defaultShortcut, okLabel: 'Create',
      onSubmit: async (name, shortcut) => {
        const created = await createScreenMut.mutateAsync(name)
        if (shortcut) {
          await saveNavShortcutsMut.mutateAsync({ ...navShortcuts, [`screen:${created.id}`]: shortcut })
        }
      }
    })
  }

  const handleRenameScreen = (id: string, oldName: string) => {
    const currentShortcut = navShortcuts[`screen:${id}`] || effectiveNavShortcuts[`screen:${id}`] || ''
    setScreenPrompt({
      title: 'Rename Screen', label: 'Screen name', initialValue: oldName, shortcutValue: currentShortcut, okLabel: 'Rename',
      onSubmit: async (name, shortcut) => {
        if (name !== oldName) renameScreenMut.mutate({ id, name })
        const target = `screen:${id}`
        if (shortcut !== undefined) {
          const updated = { ...navShortcuts }
          if (shortcut) updated[target] = shortcut; else delete updated[target]
          await saveNavShortcutsMut.mutateAsync(updated)
        }
      }
    })
  }

  const handleDeleteScreen = (id: string, name: string) => {
    setScreenConfirm({
      title: 'Delete Screen', message: `Delete screen "${name}"? This cannot be undone.`,
      onConfirm: () => { deleteScreenMut.mutate(id); if (activeScreen === id) setActiveScreen('all') }
    })
  }

  const handleRemoveSensorFromScreen = (sensorId: number) => {
    removeSensorFromScreenMut.mutate({ screenId: activeScreen, sensorId })
  }

  const handleToggleEditMode = () => {
    if (editMode) {
      if (localLayouts) { saveLayoutMut.mutate({ screenId: activeScreen, layouts: localLayouts }); setLocalLayouts(null) }
      if (localCanvasLayouts) { saveCanvasLayoutMut.mutate({ screenId: activeScreen, layouts: localCanvasLayouts }); setLocalCanvasLayouts(null) }
    }
    setEditMode(!editMode); setEditingId(null); setLayoutBreakpoint(null)
  }

  // ── Keyboard navigation ──────────────────────────────────────────
  useKeyboardNavigation({
    shortcuts, sensors, screens, navShortcuts, effectiveNavShortcuts,
    setSelectedSensor, setActiveScreen, setViewMode, setMode, setShowAlarmPanel, mode,
  })

  // ── Alarm audio ──────────────────────────────────────────────────
  const { startAlarm, stopAlarm, toggleMute } = useAlarmAudio()
  useEffect(() => {
    if (activeAlarms.count > 0) startAlarm(); else stopAlarm()
  }, [activeAlarms.count, startAlarm, stopAlarm])

  // ── Render gates ─────────────────────────────────────────────────
  if (authLoading) return <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)', color: 'var(--muted)', fontFamily: 'Inter,sans-serif' }}>Loading...</div>
  if (!authed) return <LoginScreen onLogin={handleLogin} error={loginError} branding={branding} />

  return (
    <div style={{ height: '100vh', display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      <AppHeader
        branding={branding} editMode={editMode}
        onBrandingClick={() => { if (editMode) setShowBrandingDialog(true) }}
        activeAlarmCount={activeAlarms.count}
        effectiveNavShortcuts={effectiveNavShortcuts}
        viewMode={viewMode} onViewModeChange={setViewMode}
        showShortcutsPanel={showShortcutsPanel}
        onToggleShortcutsPanel={() => setShowShortcutsPanel(v => !v)}
        shortcutsPanel={showShortcutsPanel ? <ShortcutsConfigPanel effectiveNavShortcuts={effectiveNavShortcuts} navShortcuts={navShortcuts} saveNavShortcutsMut={saveNavShortcutsMut} onClose={() => setShowShortcutsPanel(false)} /> : null}
        mode={mode} onModeChange={setMode}
        historyRange={historyRange} onHistoryRangeChange={setHistoryRange}
        historyDetail={historyDetail} onHistoryDetailChange={setHistoryDetail}
        historyAutoAgg={dashSettings.historyAutoAgg}
        turbo={turbo} onToggleTurbo={() => setTurbo(v => !v)}
        onSettingsClick={() => setShowDashSettings(true)}
        onDataManagerClick={() => setShowDataManager(true)}
        onLogout={handleLogout}
        onAlarmPanelToggle={() => setShowAlarmPanel(v => !v)}
      />

      <div style={{ flex: 1, overflow: 'auto', padding: '0 20px 20px' }}>
        <ScreenTabs
          mode={mode} activeScreen={activeScreen} screens={screens}
          effectiveNavShortcuts={effectiveNavShortcuts} editMode={editMode}
          onTabClick={(id) => { setActiveScreen(id); if (mode === 'system') setMode('live') }}
          onSystemClick={() => { setMode('system'); setActiveScreen('all') }}
          onRenameScreen={handleRenameScreen}
          onDeleteScreen={handleDeleteScreen}
          onAddScreen={handleAddScreen}
          isAdmin={isAdmin}
          onToggleEditMode={handleToggleEditMode}
          editLabel={editMode ? 'Done' : 'Edit'}
          onAddSource={() => setShowAddDialog(true)}
          canAddSource={activeScreen !== 'all'}
          onAutoArrange={handleAutoArrange}
        />

        {editMode && (
          <LayoutBreakpointSelector layoutBreakpoint={layoutBreakpoint} onChange={setLayoutBreakpoint} />
        )}

        {selectedSensor != null ? (() => {
          const s = sensors.find(x => x.id === selectedSensor)
          return s ? <SensorDetail sensor={s} onBack={() => setSelectedSensor(null)} historyRange={historyRange} onRangeChange={setHistoryRange} historyDetail={historyDetail} onDetailChange={setHistoryDetail} historyAutoAgg={dashSettings.historyAutoAgg} shortcuts={shortcuts} mode={mode === 'history' ? 'history' : 'live'} onModeChange={setMode} allSensors={sensors} /> : null
        })() : (<>
          {mode === 'system' && <SystemPanel systemInfo={effectiveSystemInfo} />}
          <MainContent
            viewMode={viewMode} mode={mode}
            screenSensors={screenSensors}
            effectiveLayouts={effectiveLayouts}
            canvasLayouts={canvasLayouts}
            editMode={editMode} activeScreen={activeScreen}
            gap={dashSettings.gap}
            editingId={editingId}
            setEditingId={setEditingId}
            setSelectedSensor={setSelectedSensor}
            onLayoutChange={handleLayoutChange}
            onCanvasLayoutChange={handleCanvasLayoutChange}
            onSave={handleSave}
            onDelete={handleDelete}
            forcedBreakpoint={layoutBreakpoint}
            shortcuts={shortcuts}
            scadaEquipment={scadaEquipment}
            scadaConnections={scadaConnections}
            twinLayouts={twinLayouts}
            twinCamera={twinCamera}
            twinConnections={twinConnections}
            sensors={sensors}
            saveScadaEquipmentMut={saveScadaEquipmentMut}
            saveScadaConnectionsMut={saveScadaConnectionsMut}
            saveTwinLayoutMut={saveTwinLayoutMut}
            saveTwinConnectionsMut={saveTwinConnectionsMut}
            updateSensorMut={updateSensorMut}
            fetchOk={fetchOk}
          />
        </>)}

        {showDashSettings && <DashboardSettings settings={dashSettings} onChange={setDashSettings} onClose={() => setShowDashSettings(false)} alarmMuted={alarmMuted} onToggleMute={() => { toggleMute(); setAlarmMuted(!alarmMuted) }} />}
        {showDataManager && <DataManager sensors={sensors} onSave={handleSave} onDelete={handleDelete} onAdd={handleAddSensorToManager} onDuplicate={handleDuplicateSensor} onClose={() => setShowDataManager(false)} shortcuts={shortcuts} isAdmin={isAdmin} />}
        {showAddDialog && activeScreen !== 'all' && (
          <AddSensorDialog screenId={activeScreen} sensors={sensors} screenSensorIds={screenSensors.map(s => s.id)} onClose={() => setShowAddDialog(false)} onDone={() => { setShowAddDialog(false); queryClient.invalidateQueries({ queryKey: ['screenDataSources', activeScreen] }); queryClient.invalidateQueries({ queryKey: ['sensors'] }) }} />
        )}
        {screenPrompt && <PromptDialog {...screenPrompt} onClose={() => setScreenPrompt(null)} />}
        {screenConfirm && <ConfirmDialog {...screenConfirm} onClose={() => setScreenConfirm(null)} />}
        {showBrandingDialog && <BrandingDialog branding={branding} onSave={handleSaveBranding} onClose={() => setShowBrandingDialog(false)} />}
        {showAlarmPanel && <AlarmPanel onClose={() => setShowAlarmPanel(false)} />}
      </div>

      <Footer />
    </div>
  )
}

export default function App() {
  const storeRef = useRef(createDashboardStore())
  return (
    <DashboardStoreProvider value={storeRef.current}>
      <AppInner />
    </DashboardStoreProvider>
  )
}
