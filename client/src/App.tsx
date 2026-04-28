import { useState, useRef, useEffect, useCallback, useMemo, lazy, Suspense } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { SensorConfig, SensorData, DataPoint, Screen, LIVE_PTS, CanvasLayoutItem } from './types'
import { normalizeKeyEvent } from './styles'
import { expandCompactData } from './utils'
import { LoginScreen } from './LoginScreen'
import { DashboardSettings } from './DashboardSettings'
import { SensorDetail } from './SensorDetail'
import { DataManager } from './DataManager'
import { AddSensorDialog } from './AddSensorDialog'
import { ConfirmDialog, PromptDialog } from './ScreenDialog'
import { ShortcutsConfigPanel } from './ShortcutsConfigPanel'
import { useAlarmAudio } from './useAlarmAudio'
import { useSSEData } from './useSSEData'
import { AlarmPanel } from './AlarmPanel'
import { SensorGrid } from './SensorGrid'
import { CanvasView } from './CanvasView'
import { ScreenLayouts, LayoutItem } from './types'
import { BrandingDialog, DEFAULT_BRANDING, type BrandingSettings } from './BrandingDialog'
import { AppHeader } from './components/AppHeader'
import { ScreenTabs } from './components/ScreenTabs'
import { SystemPanel } from './components/SystemPanel'
import { useSensors, useScreenSensors, useScreens, useLayouts, useTwinLayouts, useTwinConnections, useCanvasLayouts, useShortcuts, useNavShortcuts, useBranding, useSystemInfo, useActiveAlarmCount, useUpdateSensor, useDeleteSensor, useAddSensor, useSaveLayout, useSaveTwinLayout, useSaveTwinConnections, useSaveCanvasLayout, useSaveShortcuts, useSaveNavShortcuts, useSaveBranding, useCreateScreen, useRenameScreen, useDeleteScreen, useAddSensorToScreen, useRemoveSensorFromScreen, useScadaEquipment, useScadaConnections, useSaveScadaEquipment, useSaveScadaConnections, fetchOk } from './queries'
const DigitalTwinView = lazy(() => import('./DigitalTwinView').then(m => ({ default: m.DigitalTwinView })))
const ScadaView = lazy(() => import('./ScadaView').then(m => ({ default: m.ScadaView })))

export default function App() {
  const queryClient = useQueryClient()

  const [authed, setAuthed] = useState(false);
  const [authRole, setAuthRole] = useState<string>('user');
  const [authLoading, setAuthLoading] = useState(true);
  const [loginError, setLoginError] = useState('');

  const [dataMap, setDataMap] = useState<Record<string, DataPoint[]>>({});
  const [historyMap, setHistoryMap] = useState<Record<string, DataPoint[]> | null>(null);
  const [historyReady, setHistoryReady] = useState(false);
  const [currentValues, setCurrentValues] = useState<Record<string, SensorData>>({});
  const [showAlarmPanel, setShowAlarmPanel] = useState(false);
  const [alarmMuted, setAlarmMuted] = useState(false);
  const { startAlarm, stopAlarm, toggleMute } = useAlarmAudio();
  const [paused, setPaused] = useState(false);
  const [turbo, setTurbo] = useState(false);
  const [editMode, setEditMode] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [dashSettings, setDashSettings] = useState({ gap: 12, refreshInterval: 1000, historyAutoAgg: 'sampling' as const });
  const [showDashSettings, setShowDashSettings] = useState(false);
  const [selectedSensor, setSelectedSensor] = useState<number | null>(null);
  const [activeScreen, setActiveScreen] = useState<string>('all');
  const [localLayouts, setLocalLayouts] = useState<ScreenLayouts | null>(null);
  const [mode, setMode] = useState<'live' | 'history' | 'system'>('live');
  const [historyRange, setHistoryRange] = useState('30m');
  const [historyDetail, setHistoryDetail] = useState<'auto' | 'full'>('auto');
  const [showDataManager, setShowDataManager] = useState(false);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [screenPrompt, setScreenPrompt] = useState<{ title: string; label: string; placeholder?: string; initialValue?: string; shortcutValue?: string; okLabel?: string; onSubmit: (v: string, shortcut?: string | null) => void } | null>(null);
  const [screenConfirm, setScreenConfirm] = useState<{ title: string; message: string; onConfirm: () => void } | null>(null);
  const [showBrandingDialog, setShowBrandingDialog] = useState(false);
  const [layoutBreakpoint, setLayoutBreakpoint] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'cards' | 'canvas' | 'twin' | 'scada'>('cards');
  const [localCanvasLayouts, setLocalCanvasLayouts] = useState<CanvasLayoutItem[] | null>(null);
  const [showShortcutsPanel, setShowShortcutsPanel] = useState(false);

  const startTime = useRef(Date.now());
  const initialLoadDone = useRef(false);

  // ── Query hooks ──────────────────────────────────────────────────

  const { data: apiSensors = [] } = useSensors(authed);
  const sensors = apiSensors;

  const { data: apiScreenSensors = [] } = useScreenSensors(activeScreen, authed);
  const screenSensors = apiScreenSensors;

  const { data: screens = [] } = useScreens(authed);

  const { data: serverLayouts } = useLayouts(activeScreen, authed);
  const screenLayouts = localLayouts ?? (serverLayouts ?? { lg: [], md: [], sm: [] });

  const { data: twinLayoutsData } = useTwinLayouts(activeScreen, authed);
  const twinLayouts = twinLayoutsData?.items ?? [];
  const twinCamera = twinLayoutsData?.camera;

  const { data: twinConnections = [] } = useTwinConnections(activeScreen, authed);

  const { data: scadaEquipment = [] } = useScadaEquipment(activeScreen, authed && viewMode === 'scada')
  const { data: scadaConnections = [] } = useScadaConnections(activeScreen, authed && viewMode === 'scada')

  const { data: serverCanvasLayouts = [] } = useCanvasLayouts(activeScreen, authed);
  const canvasLayouts = localCanvasLayouts ?? serverCanvasLayouts;

  const { data: shortcuts = {} } = useShortcuts(authed);
  const { data: navShortcuts = {} } = useNavShortcuts(authed);

  const effectiveNavShortcuts = useMemo(() => {
    const defaults: Record<string, string> = {
      'screen:all': 'Mod+1',
      'screen:system': 'Mod+0',
      'view:cards': '',
      'view:canvas': '',
      'view:twin': '',
      'view:scada': '',
      'mode:live': '',
      'mode:history': '',
      'action:alarms': '',
    };
    screens.forEach((s, i) => { defaults[`screen:${s.id}`] = `Mod+${i + 2}`; });
    // User overrides take precedence
    Object.entries(navShortcuts).forEach(([k, v]) => { if (v) defaults[k] = v; });
    return defaults;
  }, [screens, navShortcuts]);

  const { data: branding = DEFAULT_BRANDING } = useBranding();

  const { data: systemInfo } = useSystemInfo(authed);
  const { data: activeAlarms = { count: 0 } } = useActiveAlarmCount(authed);

  // SSE for live data
  const sseData = useSSEData(authed && mode === 'live');

  // ── Mutation hooks ───────────────────────────────────────────────

  const updateSensorMut = useUpdateSensor();
  const deleteSensorMut = useDeleteSensor();
  const addSensorMut = useAddSensor();
  const saveLayoutMut = useSaveLayout();
  const saveTwinLayoutMut = useSaveTwinLayout();
  const saveTwinConnectionsMut = useSaveTwinConnections();
  const saveCanvasLayoutMut = useSaveCanvasLayout();
  const saveScadaEquipmentMut = useSaveScadaEquipment()
  const saveScadaConnectionsMut = useSaveScadaConnections()
  const saveShortcutsMut = useSaveShortcuts();
  const saveNavShortcutsMut = useSaveNavShortcuts();
  const saveBrandingMut = useSaveBranding();
  const createScreenMut = useCreateScreen();
  const renameScreenMut = useRenameScreen();
  const deleteScreenMut = useDeleteScreen();
  const addSensorToScreenMut = useAddSensorToScreen();
  const removeSensorFromScreenMut = useRemoveSensorFromScreen();

  // ── Helpers ──────────────────────────────────────────────────────

  const isAdmin = authRole === 'admin';

  const DEFAULT_COLS: Record<string, number> = { lg: 24, md: 12, sm: 4 };

  function generateDefaultLayout(sensorIds: number[], bp: string): LayoutItem[] {
    const cols = DEFAULT_COLS[bp] || 24;
    const defaultW = bp === 'lg' ? 6 : bp === 'md' ? 6 : 4;
    const defaultH = 4;
    let x = 0, y = 0;
    return sensorIds.map(id => {
      if (x + defaultW > cols) { x = 0; y += defaultH; }
      const item = { i: String(id), x, y, w: defaultW, h: defaultH };
      x += defaultW;
      return item;
    });
  }

  const defaultLayouts = useMemo(() => {
    if (screenLayouts.lg.length > 0) return null;
    const ids = screenSensors.map(s => s.id);
    if (ids.length === 0) return null;
    return { lg: generateDefaultLayout(ids, 'lg'), md: generateDefaultLayout(ids, 'md'), sm: generateDefaultLayout(ids, 'sm') };
  }, [screenSensors, screenLayouts]);

  const effectiveLayouts = defaultLayouts ?? screenLayouts;

  // ── Auth ─────────────────────────────────────────────────────────

  useEffect(() => {
    fetch('/api/auth/me').then(r => r.json()).then(d => { setAuthed(d.authenticated === true); setAuthRole(d.role || 'user'); setAuthLoading(false); }).catch(() => { setAuthed(false); setAuthRole('user'); setAuthLoading(false); });
  }, []);

  const handleLogin = async (username: string, password: string) => {
    setLoginError('');
    const r = await fetch('/api/auth/login', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ username, password }) });
    if (r.ok) { const d = await r.json(); setAuthed(true); setAuthRole(d.role || 'user'); }
    else { const d = await r.json(); setLoginError(d.error || 'Login failed'); }
  };

  const handleLogout = async () => { await fetch('/api/auth/logout', { method: 'POST' }); setAuthed(false); setAuthRole('user'); };

  // ── Seed ─────────────────────────────────────────────────────────

  // Request notification permission once on login
  useEffect(() => {
    if (!authed) return;
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }
  }, [authed]);

  // Register service worker for push notifications
  useEffect(() => {
    if ('serviceWorker' in navigator) {
      navigator.serviceWorker.register('/sw.js');
    }
  }, []);

  // ── Branding handler ─────────────────────────────────────────────

  const handleSaveBranding = (settings: BrandingSettings) => saveBrandingMut.mutate(settings);

  // ── Handlers ─────────────────────────────────────────────────────

  const handleSave = async (s: SensorConfig, shortcut?: string | null) => {
    await updateSensorMut.mutateAsync(s);
    if (shortcut !== undefined) {
      const prev = shortcuts[s.id] ?? null;
      if (shortcut !== prev) {
        await saveShortcutsMut.mutateAsync({ ...shortcuts, [s.id]: shortcut });
      }
    }
    setEditingId(null);
  };

  const handleDelete = async (id: number) => {
    await deleteSensorMut.mutateAsync(id);
    setEditingId(null);
  };

  const handleAddSensorToManager = async (form: Partial<SensorConfig>) => {
    await addSensorMut.mutateAsync(form);
  };

  const handleDuplicateSensor = async (s: SensorConfig) => {
    const dup = {
      key: s.key + '_copy',
      name: s.name + ' (Copy)',
      unit: s.unit,
      min_val: s.min_val,
      max_val: s.max_val,
      warn_val: s.warn_val,
      crit_val: s.crit_val,
      base_val: s.base_val,
      decimals: s.decimals,
      source_type: s.source_type,
      color: s.color,
      chart_type: s.chart_type,
      source_config: s.source_config,
    };
    await addSensorMut.mutateAsync(dup);
  };

  const handleLayoutChange = useCallback((currentLayout: any[], allLayouts: { [bp: string]: any[] }) => {
    const normalize = (items: any[]) => items.map((l: any) => ({ i: String(l.i), x: l.x, y: l.y, w: l.w, h: l.h }));
    const newLayouts: ScreenLayouts = {
      lg: allLayouts.lg?.length > 0 ? normalize(allLayouts.lg) : [],
      md: allLayouts.md?.length > 0 ? normalize(allLayouts.md) : [],
      sm: allLayouts.sm?.length > 0 ? normalize(allLayouts.sm) : [],
    };
    setLocalLayouts(prev => {
      const next = JSON.stringify(newLayouts);
      if (prev && JSON.stringify(prev) === next) return prev;
      return newLayouts;
    });
  }, []);

  const handleCanvasLayoutChange = useCallback((items: CanvasLayoutItem[]) => {
    setLocalCanvasLayouts(items);
  }, []);

  const handleAddScreen = () => {
    const defaultShortcut = `Mod+${screens.length + 2}`;
    setScreenPrompt({
      title: 'New Screen', label: 'Screen name', placeholder: 'My Screen', shortcutValue: defaultShortcut, okLabel: 'Create',
      onSubmit: async (name, shortcut) => {
        const created = await createScreenMut.mutateAsync(name);
        if (shortcut) {
          await saveNavShortcutsMut.mutateAsync({ ...navShortcuts, [`screen:${created.id}`]: shortcut });
        }
      }
    });
  };

  const handleRenameScreen = (id: string, oldName: string) => {
    const currentShortcut = navShortcuts[`screen:${id}`] || effectiveNavShortcuts[`screen:${id}`] || '';
    setScreenPrompt({
      title: 'Rename Screen', label: 'Screen name', initialValue: oldName, shortcutValue: currentShortcut, okLabel: 'Rename',
      onSubmit: async (name, shortcut) => {
        if (name !== oldName) renameScreenMut.mutate({ id, name });
        const target = `screen:${id}`;
        if (shortcut !== undefined) {
          const updated = { ...navShortcuts };
          if (shortcut) updated[target] = shortcut; else delete updated[target];
          await saveNavShortcutsMut.mutateAsync(updated);
        }
      }
    });
  };

  const handleDeleteScreen = (id: string, name: string) => {
    setScreenConfirm({
      title: 'Delete Screen', message: `Delete screen "${name}"? This cannot be undone.`,
      onConfirm: () => { deleteScreenMut.mutate(id); if (activeScreen === id) setActiveScreen('all') }
    });
  };

  // Sensor removal from screen (used by SensorGrid's "Remove" button)
  const handleRemoveSensorFromScreen = (sensorId: number) => {
    removeSensorFromScreenMut.mutate({ screenId: activeScreen, sensorId });
  };

  // ── Initial data load ────────────────────────────────────────────

  useEffect(() => {
    if (!authed || sensors.length === 0 || initialLoadDone.current) return;
    initialLoadDone.current = true;
    fetch('/api/history?range=30m').then(r => r.json()).then(result => {
      if (!result.data) return;
      const expanded = expandCompactData(result.data);
      const remapped: Record<string, DataPoint[]> = {};
      for (const s of sensors) {
        const pts = expanded[s.key];
        if (pts) remapped[String(s.id)] = pts.length > LIVE_PTS ? pts.slice(-LIVE_PTS) : pts;
      }
      setDataMap(remapped);
    }).catch(() => {}).finally(() => setHistoryReady(true));
  }, [authed, sensors]);

  // ── SSE live data processing ─────────────────────────────────────

  const dataMapRef = useRef(dataMap);
  dataMapRef.current = dataMap;
  const currentValuesRef = useRef(currentValues);
  currentValuesRef.current = currentValues;

  useEffect(() => {
    if (paused || !sseData.data || !historyReady) return;
    const d = sseData.data;
    const newMap = { ...dataMapRef.current };
    const newVals = { ...currentValuesRef.current };
    for (const item of d) {
      const k = String(item.id);
      const isOffline = item.value == null || Number.isNaN(item.value);
      if (isOffline) {
        newVals[k] = item;
        continue;
      }
      const arr = newMap[k] ? [...newMap[k]] : [];
      arr.push({ value: item.value, ts: Date.now() });
      newMap[k] = arr.length > LIVE_PTS ? arr.slice(-LIVE_PTS) : arr;
      newVals[k] = item;
    }
    setDataMap(newMap);
    setCurrentValues(newVals);
  }, [sseData.data, paused, historyReady]);

  // Alarm sound driven by server-side active alarm count
  useEffect(() => {
    if (activeAlarms.count > 0) {
      startAlarm();
    } else {
      stopAlarm();
    }
  }, [activeAlarms.count, startAlarm, stopAlarm]);

  // ── History fetch ────────────────────────────────────────────────

  useEffect(() => {
    if (mode !== 'history') { setHistoryMap(null); return; }
    const dp = activeScreen === 'all' ? '' : `&screen=${activeScreen}`;
    const detailParam = historyDetail === 'full' ? '&detail=full' : '';
    const aggParam = historyDetail === 'auto' ? `&agg=${dashSettings.historyAutoAgg}` : '';
    fetch(`/api/history?range=${historyRange}${dp}${detailParam}${aggParam}`).then(r => r.json()).then(result => {
      if (!result.data) return;
      const expanded = expandCompactData(result.data);
      const remapped: Record<string, DataPoint[]> = {};
      for (const s of screenSensors) {
        const pts = expanded[s.key];
        if (pts) remapped[String(s.id)] = pts;
      }
      setHistoryMap(remapped);
    });
  }, [mode, historyRange, historyDetail, activeScreen, sensors, screenSensors, dashSettings.historyAutoAgg]);

  // Re-seed live dataMap with 30m history when switching back to live
  useEffect(() => {
    if (mode !== 'live' || !authed || sensors.length === 0) return;
    const dp = activeScreen === 'all' ? '' : `&screen=${activeScreen}`;
    fetch(`/api/history?range=30m${dp}`).then(r => r.json()).then(result => {
      if (!result.data) return;
      const expanded = expandCompactData(result.data);
      const remapped: Record<string, DataPoint[]> = {};
      for (const s of screenSensors) {
        const pts = expanded[s.key];
        if (pts) remapped[String(s.id)] = pts.length > LIVE_PTS ? pts.slice(-LIVE_PTS) : pts;
      }
      setDataMap(remapped);
    });
  }, [mode]);

  // ── Keyboard shortcuts ───────────────────────────────────────────

  const GAP = dashSettings.gap;

  const shortcutToSensorId = useMemo(() => {
    const map = new Map<string, number>();
    const normalize = (s: string) => s.replace(/\bCtrl\b/gi, 'Mod').replace(/\bMeta\b/gi, 'Mod').toLowerCase();
    for (const [id, sc] of Object.entries(shortcuts)) {
      const sensor = sensors.find(s => s.id === Number(id));
      if (sensor) map.set(normalize(sc), sensor.id);
    }
    return map;
  }, [shortcuts, sensors]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (!el) return;
      if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT' || el.isContentEditable) return;
      const combo = normalizeKeyEvent(e).toLowerCase();

      // Nav shortcuts
      const navEntry = Object.entries(effectiveNavShortcuts).find(([, v]) => v && v.replace(/\bCtrl\b/gi, 'Mod').replace(/\bMeta\b/gi, 'Mod').toLowerCase() === combo);
      if (navEntry) {
        e.preventDefault();
        setSelectedSensor(null);
        const [target] = navEntry;
        if (target.startsWith('screen:')) {
          const screenId = target.slice(7);
          if (screenId === 'system') {
            setMode('system');
            setActiveScreen('all');
          } else {
            setActiveScreen(screenId);
            if (mode === 'system') setMode('live');
          }
        } else if (target.startsWith('view:')) {
          setViewMode(target.slice(5) as typeof viewMode);
        } else if (target.startsWith('mode:')) {
          setMode(target.slice(5) as 'live' | 'history');
        } else if (target === 'action:alarms') {
          setShowAlarmPanel(v => !v);
        }
        return;
      }

      // Sensor detail shortcuts
      const sensorId = shortcutToSensorId.get(combo);
      if (sensorId != null) {
        e.preventDefault();
        setSelectedSensor(sensorId);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [shortcutToSensorId, effectiveNavShortcuts, mode]);

  const chartDataFor = useCallback((id: number) => {
    const k = String(id);
    if (mode === 'history' && historyMap && historyMap[k]) return historyMap[k].map(d => d.value);
    return (dataMap[k] || []).map(d => d.value);
  }, [mode, historyMap, dataMap]);
  const detailDataFor = useCallback((id: number): DataPoint[] => {
    const k = String(id);
    if (mode === 'history' && historyMap && historyMap[k]) return historyMap[k];
    return dataMap[k] || [];
  }, [mode, historyMap, dataMap]);

  // ── Render gates ─────────────────────────────────────────────────

  if (authLoading) return <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)', color: 'var(--muted)', fontFamily: 'Inter,sans-serif' }}>Loading...</div>;
  if (!authed) return <LoginScreen onLogin={handleLogin} error={loginError} branding={branding} />;

  return (
    <div style={{ height: '100vh', display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      <AppHeader
        branding={branding}
        editMode={editMode}
        onBrandingClick={() => { if (editMode) setShowBrandingDialog(true); }}
        activeAlarmCount={activeAlarms.count}
        effectiveNavShortcuts={effectiveNavShortcuts}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
        showShortcutsPanel={showShortcutsPanel}
        onToggleShortcutsPanel={() => setShowShortcutsPanel(v => !v)}
        shortcutsPanel={showShortcutsPanel ? <ShortcutsConfigPanel effectiveNavShortcuts={effectiveNavShortcuts} navShortcuts={navShortcuts} saveNavShortcutsMut={saveNavShortcutsMut} onClose={() => setShowShortcutsPanel(false)} /> : null}
        mode={mode}
        onModeChange={setMode}
        historyRange={historyRange}
        onHistoryRangeChange={setHistoryRange}
        historyDetail={historyDetail}
        onHistoryDetailChange={setHistoryDetail}
        historyAutoAgg={dashSettings.historyAutoAgg}
        onSettingsClick={() => setShowDashSettings(true)}
        onDataManagerClick={() => setShowDataManager(true)}
        onLogout={handleLogout}
        onAlarmPanelToggle={() => setShowAlarmPanel(v => !v)}
      />

      {/* Screen navigation + content */}
      <div style={{ flex: 1, overflow: 'auto', padding: '0 20px 20px' }}>
      <ScreenTabs
        mode={mode}
        activeScreen={activeScreen}
        screens={screens}
        effectiveNavShortcuts={effectiveNavShortcuts}
        editMode={editMode}
        onTabClick={(id) => { setActiveScreen(id); if (mode === 'system') setMode('live'); }}
        onSystemClick={() => { setMode('system'); setActiveScreen('all'); }}
        onRenameScreen={handleRenameScreen}
        onDeleteScreen={handleDeleteScreen}
        onAddScreen={handleAddScreen}
        isAdmin={isAdmin}
        onToggleEditMode={() => {
          if (editMode) {
            if (localLayouts) { saveLayoutMut.mutate({ screenId: activeScreen, layouts: localLayouts }); setLocalLayouts(null); }
            if (localCanvasLayouts) { saveCanvasLayoutMut.mutate({ screenId: activeScreen, layouts: localCanvasLayouts }); setLocalCanvasLayouts(null); }
          }
          setEditMode(!editMode); setEditingId(null); setLayoutBreakpoint(null);
        }}
        editLabel={editMode ? 'Done' : 'Edit'}
        onAddSource={() => setShowAddDialog(true)}
        canAddSource={activeScreen !== 'all'}
      />

      {/* Layout breakpoint selector (edit mode) */}
      {editMode && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 12 }}>
          <span style={{ fontSize: 11, color: 'var(--muted)', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '.05em' }}>Edit layout:</span>
          {([['lg', 'Desktop', 24], ['md', 'Tablet', 12], ['sm', 'Mobile', 4]] as const).map(([bp, label, cols]) => (
            <button key={bp} onClick={() => setLayoutBreakpoint(layoutBreakpoint === bp ? null : bp)} style={{
              padding: '4px 10px', borderRadius: 6, fontSize: 11, fontWeight: 600, cursor: 'pointer',
              border: `1px solid ${layoutBreakpoint === bp ? 'var(--cyan)' : 'var(--border)'}`,
              background: layoutBreakpoint === bp ? 'rgba(34,197,94,.1)' : 'var(--card)',
              color: layoutBreakpoint === bp ? 'var(--cyan)' : 'var(--muted)',
            }}>{label} ({cols}col)</button>
          ))}
          {layoutBreakpoint && <span style={{ fontSize: 10, color: 'var(--cyan)', marginLeft: 4 }}>Editing {layoutBreakpoint.toUpperCase()} layout — drag/resize to customize</span>}
        </div>
      )}

      {selectedSensor != null ? (() => {
        const s = sensors.find(x => x.id === selectedSensor);
        return s ? <SensorDetail sensor={s} data={detailDataFor(selectedSensor)} currentValue={currentValues[selectedSensor]} onBack={() => setSelectedSensor(null)} historyRange={historyRange} onRangeChange={setHistoryRange} historyDetail={historyDetail} onDetailChange={setHistoryDetail} historyAutoAgg={dashSettings.historyAutoAgg} shortcuts={shortcuts} mode={mode === 'history' ? 'history' : 'live'} onModeChange={setMode} allSensors={sensors} getDataFor={detailDataFor} currentValues={currentValues} /> : null;
      })() : (<>
      {mode === 'system' && <SystemPanel systemInfo={systemInfo} />}

      {/* Sensors Grid / Canvas / Digital Twin */}
      {mode !== 'system' && (viewMode === 'cards' ? (
        <SensorGrid
          screenLayouts={effectiveLayouts}
          screenSensors={screenSensors}
          editMode={editMode}
          activeScreen={activeScreen}
          gap={GAP}
          editingId={editingId}
          currentValues={currentValues}
          setEditingId={setEditingId}
          setSelectedSensor={setSelectedSensor}
          chartDataFor={chartDataFor}
          loadSensorsForScreen={() => {
            queryClient.invalidateQueries({ queryKey: ['screenDataSources', activeScreen] });
            queryClient.invalidateQueries({ queryKey: ['sensors'] });
          }}
          onLayoutChange={handleLayoutChange}
          onSave={handleSave}
          onDelete={handleDelete}
          forcedBreakpoint={layoutBreakpoint}
          shortcuts={shortcuts}
        />
      ) : viewMode === 'canvas' ? (
        <CanvasView
          screenSensors={screenSensors}
          canvasLayouts={canvasLayouts}
          editMode={editMode}
          activeScreen={activeScreen}
          editingId={editingId}
          currentValues={currentValues}
          setEditingId={setEditingId}
          setSelectedSensor={setSelectedSensor}
          chartDataFor={chartDataFor}
          loadSensorsForScreen={() => {
            queryClient.invalidateQueries({ queryKey: ['screenDataSources', activeScreen] });
            queryClient.invalidateQueries({ queryKey: ['sensors'] });
          }}
          onLayoutChange={handleCanvasLayoutChange}
          onSave={handleSave}
          onDelete={handleDelete}
          shortcuts={shortcuts}
        />
      ) : viewMode === 'scada' ? (
        <Suspense fallback={<div style={{ height: 400, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--muted)' }}>Loading 2D View...</div>}>
          <ScadaView
            sensors={screenSensors}
            currentValues={currentValues}
            activeScreen={activeScreen}
            scadaEquipment={scadaEquipment}
            scadaConnections={scadaConnections}
            onSaveEquipment={(items) => saveScadaEquipmentMut.mutate({ screenId: activeScreen, items })}
            onSaveConnections={(items) => saveScadaConnectionsMut.mutate({ screenId: activeScreen, items })}
            onSensorClick={(id) => setSelectedSensor(id)}
          />
        </Suspense>
      ) : (
        <Suspense fallback={<div style={{ height: 400, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--muted)' }}>Loading 3D View...</div>}>
          <DigitalTwinView
            sensors={screenSensors}
            currentValues={currentValues}
            activeScreen={activeScreen}
            twinLayouts={twinLayouts}
            twinCamera={twinCamera}
            twinConnections={twinConnections}
            onSwitchToCards={() => setViewMode('cards')}
            onEquipmentClick={(eqId) => {
              const sensor = sensors.find(s => s.equipment_id === eqId)
              if (sensor) setSelectedSensor(sensor.id)
            }}
            onSensorUpdate={(s) => updateSensorMut.mutate(s)}
            onSaveLayouts={(equipment) => {
              saveTwinLayoutMut.mutate({
                screenId: activeScreen,
                items: equipment.map(e => ({
                  equipment_id: e.id, equipment_type: e.type, equipment_name: e.name,
                  position_x: e.position[0], position_z: e.position[2], rotation_y: e.rotation,
                }))
              })
            }}
            onSaveConnections={(connections) => {
              saveTwinConnectionsMut.mutate({
                screenId: activeScreen,
                connections: connections.map(c => ({ from_id: c.from, to_id: c.to })),
              })
            }}
            onSaveCamera={(camera) => {
              const actualId = activeScreen === 'all' ? '__overview' : activeScreen
              fetchOk(`/api/screens/${actualId}/twin-camera`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(camera) })
            }}
          />
        </Suspense>
      ))}
      </>)}

      {showDashSettings && <DashboardSettings settings={dashSettings} onChange={setDashSettings} onClose={() => setShowDashSettings(false)} alarmMuted={alarmMuted} onToggleMute={() => { toggleMute(); setAlarmMuted(!alarmMuted); }} />}

      {showDataManager && <DataManager sensors={sensors} onSave={handleSave} onDelete={handleDelete} onAdd={handleAddSensorToManager} onDuplicate={handleDuplicateSensor} onClose={() => setShowDataManager(false)} shortcuts={shortcuts} isAdmin={isAdmin} />}

      {showAddDialog && activeScreen !== 'all' && (
        <AddSensorDialog screenId={activeScreen} sensors={sensors} screenSensorIds={screenSensors.map(s => s.id)} onClose={() => setShowAddDialog(false)} onDone={() => {
          setShowAddDialog(false);
          queryClient.invalidateQueries({ queryKey: ['screenDataSources', activeScreen] });
          queryClient.invalidateQueries({ queryKey: ['sensors'] });
        }} />
      )}

      {screenPrompt && <PromptDialog {...screenPrompt} onClose={() => setScreenPrompt(null)} />}
      {screenConfirm && <ConfirmDialog {...screenConfirm} onClose={() => setScreenConfirm(null)} />}
      {showBrandingDialog && <BrandingDialog branding={branding} onSave={handleSaveBranding} onClose={() => setShowBrandingDialog(false)} />}

      {showAlarmPanel && <AlarmPanel onClose={() => setShowAlarmPanel(false)} />}
      </div>{/* end scrollable content */}

      <div style={{ position: 'fixed', bottom: 12, right: 12, display: 'flex', alignItems: 'center', gap: 8, zIndex: 51 }}>
        <div style={{ fontFamily: 'Inter,sans-serif', fontSize: 11, color: 'var(--muted)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 10px', borderRadius: 4, display: 'flex', alignItems: 'center', gap: 6 }}>
          <span style={{ color: 'var(--cyan)', fontWeight: 600 }}>Voltrus Lite</span>
        </div>
        <a href="https://joyodigitama.com" target="_blank" rel="noopener noreferrer" style={{ fontFamily: 'Inter,sans-serif', fontSize: 11, color: 'var(--muted)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 10px', borderRadius: 4, textDecoration: 'none' }}>
          by <span style={{ color: 'var(--cyan)', fontWeight: 600 }}>JoyoDigitama</span>
        </a>
        <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 11, color: sseData.connected ? 'var(--ok)' : 'var(--warn)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 8px', borderRadius: 4, display: 'flex', alignItems: 'center', gap: 4 }}>
          <span style={{ width: 6, height: 6, borderRadius: '50%', background: sseData.connected ? 'var(--ok)' : 'var(--warn)' }} />
          {sseData.connected ? 'LIVE' : 'SSE...'}
        </div>
      </div>
    </div>
  );
}
