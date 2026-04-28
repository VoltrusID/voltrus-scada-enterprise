import { useState, useRef, useEffect, useCallback, useMemo, lazy, Suspense } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { SensorConfig, SensorData, DataPoint, Screen, LIVE_PTS, CanvasLayoutItem } from './types'
import { status, stColor, fmtTime, btnStyle, tabStyle, normalizeKeyEvent, displayShortcut } from './styles'
import { LoginScreen } from './LoginScreen'
import { Chart } from './Chart'
import { EditPanel } from './EditPanel'
import { DashboardSettings } from './DashboardSettings'
import { SensorDetail } from './SensorDetail'
import { DataManager } from './DataManager'
import { AddSensorDialog } from './AddSensorDialog'
import { ConfirmDialog, PromptDialog } from './ScreenDialog'
import { ShortcutsConfigPanel } from './ShortcutsConfigPanel'
import { useAlarmAudio } from './useAlarmAudio'
import { useSSEData } from './useSSEData'
import { AlarmPanel } from './AlarmPanel'
import { HistoryRangeBar } from './CustomRangePicker'
import { SensorGrid } from './SensorGrid'
import { CanvasView } from './CanvasView'
import { ScreenLayouts, LayoutItem } from './types'
import { BrandingDialog, DEFAULT_BRANDING, type BrandingSettings } from './BrandingDialog'
import { useSensors, useScreenSensors, useScreens, useLayouts, useTwinLayouts, useTwinConnections, useCanvasLayouts, useShortcuts, useNavShortcuts, useBranding, useSystemInfo, useUpdateSensor, useDeleteSensor, useAddSensor, useSaveLayout, useSaveTwinLayout, useSaveTwinConnections, useSaveCanvasLayout, useSaveShortcuts, useSaveNavShortcuts, useSaveBranding, useCreateScreen, useRenameScreen, useDeleteScreen, useAddSensorToScreen, useRemoveSensorFromScreen, useScadaEquipment, useScadaConnections, useSaveScadaEquipment, useSaveScadaConnections, fetchOk } from './queries'
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
  const [alerts, setAlerts] = useState(0);
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

  function expandCompactData(raw: Record<string, any>): Record<string, DataPoint[]> {
    const out: Record<string, DataPoint[]> = {};
    for (const key of Object.keys(raw)) {
      const s = raw[key];
      if (s && Array.isArray(s.values)) {
        const pts: DataPoint[] = [];
        for (let i = 0; i < s.values.length; i++) {
          pts.push({ ts: s.start + i * s.interval, value: s.values[i] });
        }
        out[key] = pts;
      } else if (Array.isArray(s)) {
        out[key] = s; // legacy format fallback
      }
    }
    return out;
  }

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

  useEffect(() => {
    if (authed) fetch('/api/seed', { method: 'POST' });
  }, [authed]);

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
    let alertCount = 0;
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
      if (item.source_type === 'modbus' && ((item.crit > 0 && item.value >= item.crit) || (item.warn > 0 && item.value >= item.warn))) {
        alertCount++;
      }
    }
    setDataMap(newMap);
    setCurrentValues(newVals);
    setAlerts(alertCount);
    if (alertCount > 0) {
      startAlarm();
    } else {
      stopAlarm();
    }
  }, [sseData.data, paused, historyReady]);

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

  // ── Render gates ─────────────────────────────────────────────────

  if (authLoading) return <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)', color: 'var(--muted)', fontFamily: 'Inter,sans-serif' }}>Loading...</div>;
  if (!authed) return <LoginScreen onLogin={handleLogin} error={loginError} branding={branding} />;

  const chartDataFor = (id: number) => {
    const k = String(id);
    if (mode === 'history' && historyMap && historyMap[k]) return historyMap[k].map(d => d.value);
    return (dataMap[k] || []).map(d => d.value);
  };
  const detailDataFor = (id: number): DataPoint[] => {
    const k = String(id);
    if (mode === 'history' && historyMap && historyMap[k]) return historyMap[k];
    return dataMap[k] || [];
  };

  return (
    <div style={{ height: '100vh', display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '12px 20px', flexShrink: 0, flexWrap: 'wrap', gap: 16 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: editMode ? 'pointer' : 'default' }}
          onClick={() => { if (editMode) setShowBrandingDialog(true); }}>
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
            <button onClick={() => setShowAlarmPanel(v => !v)} style={{
              ...btnStyle, position: 'relative', borderColor: alerts > 0 ? 'var(--crit)' : 'var(--border)',
              color: alerts > 0 ? 'var(--crit)' : 'var(--fg)',
            }}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
              {effectiveNavShortcuts['action:alarms'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['action:alarms'])}</kbd> : null}
              {alerts > 0 && <span style={{
                position: 'absolute', top: -4, right: -4, background: 'var(--crit)',
                color: '#fff', borderRadius: '50%', width: 16, height: 16, fontSize: 10,
                display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 700,
              }}>{alerts}</span>}
            </button>
          {/* View mode toggle */}
          <div style={{ display: 'flex', borderRadius: 6, overflow: 'hidden', border: '1px solid var(--border)' }}>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'cards' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'cards' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setViewMode('cards')}>Grids{effectiveNavShortcuts['view:cards'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:cards'])}</kbd> : null}</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'canvas' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'canvas' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setViewMode('canvas')}>Canvas{effectiveNavShortcuts['view:canvas'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:canvas'])}</kbd> : null}</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'twin' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'twin' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setViewMode('twin')}>3D{effectiveNavShortcuts['view:twin'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:twin'])}</kbd> : null}</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'scada' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'scada' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setViewMode('scada')}>2D{effectiveNavShortcuts['view:scada'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['view:scada'])}</kbd> : null}</button>
          </div>
          <div style={{ position: 'relative' }}>
            <button onClick={() => setShowShortcutsPanel(!showShortcutsPanel)} style={{ ...btnStyle, padding: '6px 8px', fontSize: 12, color: showShortcutsPanel ? 'var(--cyan)' : 'var(--muted)', background: showShortcutsPanel ? 'rgba(34,197,94,.1)' : 'transparent', border: showShortcutsPanel ? '1px solid var(--cyan)' : '1px solid var(--border)' }} title="Keyboard shortcuts">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M8 16h8"/></svg>
            </button>
            {showShortcutsPanel && <ShortcutsConfigPanel effectiveNavShortcuts={effectiveNavShortcuts} navShortcuts={navShortcuts} saveNavShortcutsMut={saveNavShortcutsMut} onClose={() => setShowShortcutsPanel(false)} />}
          </div>
          {/* Live/History toggle (hidden in 3D mode) */}
          {viewMode !== 'twin' && viewMode !== 'scada' && (<>
            <div style={{ display: 'flex', borderRadius: 6, overflow: 'hidden', border: '1px solid var(--border)' }}>
              <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'live' ? 'var(--cyan)' : 'var(--fg)', background: mode === 'live' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setMode('live')}>Live{effectiveNavShortcuts['mode:live'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['mode:live'])}</kbd> : null}</button>
              <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'history' ? 'var(--warn)' : 'var(--fg)', background: mode === 'history' ? 'rgba(234,179,8,.1)' : 'var(--card)' }} onClick={() => setMode('history')}>History{effectiveNavShortcuts['mode:history'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '0 3px', marginLeft: 4, borderRadius: 2, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['mode:history'])}</kbd> : null}</button>
            </div>
            {mode === 'history' && (
              <HistoryRangeBar range={historyRange} onChange={setHistoryRange} detail={historyDetail} onDetailChange={setHistoryDetail} agg={dashSettings.historyAutoAgg} />
            )}
          </>)}
          {mode === 'live' && (<>
            <button style={btnStyle} onClick={() => setPaused(!paused)}>
              {paused ? <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="5,3 19,12 5,21"/></svg> : <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>}
              <span>{paused ? 'Resume' : 'Pause'}</span>
            </button>
            <button style={{ ...btnStyle, borderColor: turbo ? 'var(--warn)' : 'var(--border)', color: turbo ? 'var(--warn)' : 'var(--fg)' }} onClick={() => setTurbo(!turbo)}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10"/></svg>
              <span>{turbo ? 'Turbo ON' : 'Turbo'}</span>
            </button>
          </>)}
          <button style={btnStyle} onClick={() => setShowDashSettings(true)}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            <span>Settings</span>
          </button>
          {isAdmin && (
          <button style={{ ...btnStyle, borderColor: editMode ? 'var(--cyan)' : 'var(--border)', color: editMode ? 'var(--cyan)' : 'var(--fg)' }} onClick={() => {
            if (editMode) {
              if (localLayouts) {
                saveLayoutMut.mutate({ screenId: activeScreen, layouts: localLayouts });
                setLocalLayouts(null);
              }
              if (localCanvasLayouts) {
                saveCanvasLayoutMut.mutate({ screenId: activeScreen, layouts: localCanvasLayouts });
                setLocalCanvasLayouts(null);
              }
            }
            setEditMode(!editMode);
            setEditingId(null);
            setLayoutBreakpoint(null);
          }}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
            <span>{editMode ? 'Done' : 'Edit'}</span>
          </button>
          )}
          {editMode && activeScreen !== 'all' && <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={() => setShowAddDialog(true)}>+ Add Source</button>}
          <button style={btnStyle} onClick={() => setShowDataManager(true)}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
            <span>Data</span>
          </button>
          <div style={{ marginLeft: 'auto' }} />
          <button style={btnStyle} onClick={handleLogout}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
            <span>Logout</span>
          </button>
        </div>
      </div>

      {/* Screen navigation + content */}
      <div style={{ flex: 1, overflow: 'auto', padding: '0 20px 20px' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 0, marginBottom: 20, borderBottom: '1px solid var(--border)' }}>
        <button style={tabStyle(mode !== 'system' && activeScreen === 'all')} onClick={() => { setActiveScreen('all'); if (mode === 'system') setMode('live'); }}>Overview{effectiveNavShortcuts['screen:all'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '1px 4px', marginLeft: 6, borderRadius: 3, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['screen:all'])}</kbd> : null}</button>
        {screens.map(s => (
          <div key={s.id} style={{ position: 'relative' }}>
            <button style={tabStyle(mode !== 'system' && activeScreen === s.id)} onClick={() => { setActiveScreen(s.id); if (mode === 'system') setMode('live'); }}>{s.name}{effectiveNavShortcuts[`screen:${s.id}`] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '1px 4px', marginLeft: 6, borderRadius: 3, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts[`screen:${s.id}`])}</kbd> : null}</button>
            {editMode && (
              <div style={{ position: 'absolute', top: 2, right: 2, display: 'flex', gap: 2 }}>
                <button onClick={e => { e.stopPropagation(); handleRenameScreen(s.id, s.name); }} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 10, padding: '0 2px', lineHeight: 1 }}>&#9998;</button>
                <button onClick={e => { e.stopPropagation(); handleDeleteScreen(s.id, s.name); }} style={{ background: 'none', border: 'none', color: 'var(--crit)', cursor: 'pointer', fontSize: 10, padding: '0 2px', lineHeight: 1 }}>&times;</button>
              </div>
            )}
          </div>
        ))}
        {editMode && <button onClick={handleAddScreen} style={{ ...tabStyle(false), color: 'var(--green)' }}>+</button>}
        <button style={{
          ...tabStyle(mode === 'system'),
          color: mode === 'system' ? '#a78bfa' : 'var(--muted)',
          borderBottom: mode === 'system' ? '2px solid #a78bfa' : '2px solid transparent',
        }} onClick={() => { setMode('system'); setActiveScreen('all'); }}>System{effectiveNavShortcuts['screen:system'] ? <kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '1px 4px', marginLeft: 6, borderRadius: 3, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>{displayShortcut(effectiveNavShortcuts['screen:system'])}</kbd> : null}</button>
      </div>

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

      {/* System Resource Panel */}
      {mode === 'system' && systemInfo && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: 16, padding: 8 }}>
          <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
            <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>Host</div>
            <div style={{ fontSize: 18, fontWeight: 700 }}>{systemInfo.hostname}</div>
            <div style={{ fontSize: 13, color: 'var(--muted)' }}>{systemInfo.os} — up {Math.floor(systemInfo.uptime_secs / 86400)}d {Math.floor((systemInfo.uptime_secs % 86400) / 3600)}h {Math.floor((systemInfo.uptime_secs % 3600) / 60)}m</div>
          </div>
          <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
            <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>CPU Usage</div>
            <div style={{ fontSize: 28, fontWeight: 700, color: systemInfo.cpu_usage > 80 ? 'var(--crit)' : systemInfo.cpu_usage > 50 ? 'var(--warn)' : 'var(--ok)' }}>{systemInfo.cpu_usage.toFixed(1)}%</div>
            <div style={{ marginTop: 8, height: 6, borderRadius: 3, background: 'var(--border)' }}>
              <div style={{ height: '100%', borderRadius: 3, background: systemInfo.cpu_usage > 80 ? 'var(--crit)' : systemInfo.cpu_usage > 50 ? 'var(--warn)' : 'var(--ok)', width: `${Math.min(systemInfo.cpu_usage, 100)}%`, transition: 'width 0.3s' }} />
            </div>
          </div>
          <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
            <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>Memory</div>
            <div style={{ fontSize: 28, fontWeight: 700 }}>{systemInfo.memory_used_mb.toFixed(0)} <span style={{ fontSize: 14, color: 'var(--muted)' }}>/ {systemInfo.memory_total_mb.toFixed(0)} MB</span></div>
            <div style={{ fontSize: 13, color: 'var(--muted)', marginTop: 4 }}>{systemInfo.memory_percent.toFixed(1)}%</div>
            <div style={{ marginTop: 8, height: 6, borderRadius: 3, background: 'var(--border)' }}>
              <div style={{ height: '100%', borderRadius: 3, background: systemInfo.memory_percent > 90 ? 'var(--crit)' : systemInfo.memory_percent > 70 ? 'var(--warn)' : 'var(--ok)', width: `${Math.min(systemInfo.memory_percent, 100)}%`, transition: 'width 0.3s' }} />
            </div>
          </div>
          <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 20 }}>
            <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', marginBottom: 8 }}>Disk</div>
            <div style={{ fontSize: 28, fontWeight: 700 }}>{systemInfo.disk_used_gb.toFixed(1)} <span style={{ fontSize: 14, color: 'var(--muted)' }}>/ {systemInfo.disk_total_gb.toFixed(1)} GB</span></div>
            <div style={{ fontSize: 13, color: 'var(--muted)', marginTop: 4 }}>{systemInfo.disk_percent.toFixed(1)}%</div>
            <div style={{ marginTop: 8, height: 6, borderRadius: 3, background: 'var(--border)' }}>
              <div style={{ height: '100%', borderRadius: 3, background: systemInfo.disk_percent > 90 ? 'var(--crit)' : systemInfo.disk_percent > 70 ? 'var(--warn)' : 'var(--ok)', width: `${Math.min(systemInfo.disk_percent, 100)}%`, transition: 'width 0.3s' }} />
            </div>
          </div>
          <div style={{ background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)', border: '1px solid #22c55e', borderRadius: 10, padding: 20, gridColumn: 'span 2' }}>
            <div style={{ fontSize: 11, color: '#22c55e', textTransform: 'uppercase', letterSpacing: '.08em', marginBottom: 12, fontWeight: 700 }}>Rust Service (PID {systemInfo.process.pid})</div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16 }}>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{systemInfo.process.memory_mb.toFixed(1)} MB</div>
                <div style={{ fontSize: 11, color: 'var(--muted)' }}>RSS Memory</div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{systemInfo.process.cpu_percent.toFixed(1)}%</div>
                <div style={{ fontSize: 11, color: 'var(--muted)' }}>CPU Usage</div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{Math.floor(systemInfo.process.uptime_secs / 86400)}d {Math.floor((systemInfo.process.uptime_secs % 86400) / 3600)}h {Math.floor((systemInfo.process.uptime_secs % 3600) / 60)}m</div>
                <div style={{ fontSize: 11, color: 'var(--muted)' }}>Uptime</div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{systemInfo.process.binary_size_mb.toFixed(1)} MB</div>
                <div style={{ fontSize: 11, color: 'var(--muted)' }}>Binary Size</div>
              </div>
            </div>
          </div>
        </div>
      )}
      {mode === 'system' && !systemInfo && (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 300, color: 'var(--muted)' }}>Connecting to server...</div>
      )}

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

      {showDashSettings && <DashboardSettings settings={dashSettings} onChange={setDashSettings} onClose={() => setShowDashSettings(false)} alarmMuted={alarmMuted} onToggleMute={() => { toggleMute(); setAlarmMuted(!alarmMuted); }} />}

      {selectedSensor != null && (() => {
        const s = sensors.find(x => x.id === selectedSensor);
        return s ? <SensorDetail sensor={s} data={detailDataFor(selectedSensor)} currentValue={currentValues[selectedSensor]} onBack={() => setSelectedSensor(null)} historyRange={historyRange} onRangeChange={setHistoryRange} historyDetail={historyDetail} onDetailChange={setHistoryDetail} historyAutoAgg={dashSettings.historyAutoAgg} shortcuts={shortcuts} mode={mode === 'history' ? 'history' : 'live'} onModeChange={setMode} allSensors={sensors} getDataFor={detailDataFor} currentValues={currentValues} /> : null;
      })()}

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
