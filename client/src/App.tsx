import { useState, useRef, useEffect, useCallback, useMemo, lazy, Suspense } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { SensorConfig, SensorData, DataPoint, Screen, MAX_PTS, NORM_PTS, CanvasLayoutItem } from './types'
import { status, stColor, fmtTime, btnStyle, tabStyle, normalizeKeyEvent } from './styles'
import { LoginScreen } from './LoginScreen'
import { Chart } from './Chart'
import { EditPanel } from './EditPanel'
import { DashboardSettings } from './DashboardSettings'
import { SensorDetail } from './SensorDetail'
import { SensorManager } from './SensorManager'
import { AddSensorDialog } from './AddSensorDialog'
import { ConfirmDialog, PromptDialog } from './ScreenDialog'
import { useAlarmAudio } from './useAlarmAudio'
import { AlarmPanel } from './AlarmPanel'
import { HistoryRangeBar } from './CustomRangePicker'
import { SensorGrid } from './SensorGrid'
import { CanvasView } from './CanvasView'
import { ScreenLayouts, LayoutItem } from './types'
import { BrandingDialog, DEFAULT_BRANDING, type BrandingSettings } from './BrandingDialog'
import { useSensors, useScreenSensors, useScreens, useLayouts, useTwinLayouts, useTwinConnections, useCanvasLayouts, useShortcuts, useBranding, useSystemInfo, useUpdateSensor, useDeleteSensor, useAddSensor, useSaveLayout, useSaveTwinLayout, useSaveTwinConnections, useSaveCanvasLayout, useSaveShortcuts, useSaveBranding, useCreateScreen, useRenameScreen, useDeleteScreen, useAddSensorToScreen, useRemoveSensorFromScreen } from './queries'
const DigitalTwinView = lazy(() => import('./DigitalTwinView').then(m => ({ default: m.DigitalTwinView })))

export default function App() {
  const queryClient = useQueryClient()

  const [authed, setAuthed] = useState(false);
  const [authLoading, setAuthLoading] = useState(true);
  const [loginError, setLoginError] = useState('');

  const [dataMap, setDataMap] = useState<Record<string, DataPoint[]>>({});
  const [historyMap, setHistoryMap] = useState<Record<string, DataPoint[]> | null>(null);
  const [currentValues, setCurrentValues] = useState<Record<string, SensorData>>({});
  const [alerts, setAlerts] = useState(0);
  const [showAlarmPanel, setShowAlarmPanel] = useState(false);
  const [alarmMuted, setAlarmMuted] = useState(false);
  const { startAlarm, stopAlarm, toggleMute } = useAlarmAudio();
  const [paused, setPaused] = useState(false);
  const [turbo, setTurbo] = useState(false);
  const [editMode, setEditMode] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [fps, setFps] = useState(0);
  const [dashSettings, setDashSettings] = useState({ gap: 12, refreshInterval: 1000 });
  const [showDashSettings, setShowDashSettings] = useState(false);
  const [selectedSensor, setSelectedSensor] = useState<string | null>(null);
  const [activeScreen, setActiveScreen] = useState<string>('all');
  const [localLayouts, setLocalLayouts] = useState<ScreenLayouts | null>(null);
  const [mode, setMode] = useState<'live' | 'history' | 'system'>('live');
  const [historyRange, setHistoryRange] = useState('30m');
  const [showSensorManager, setShowSensorManager] = useState(false);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [screenPrompt, setScreenPrompt] = useState<{ title: string; label: string; placeholder?: string; initial?: string; okLabel?: string; onSubmit: (v: string) => void } | null>(null);
  const [screenConfirm, setScreenConfirm] = useState<{ title: string; message: string; onConfirm: () => void } | null>(null);
  const [showBrandingDialog, setShowBrandingDialog] = useState(false);
  const [layoutBreakpoint, setLayoutBreakpoint] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'cards' | 'canvas' | 'twin'>('cards');
  const [localCanvasLayouts, setLocalCanvasLayouts] = useState<CanvasLayoutItem[] | null>(null);

  const startTime = useRef(Date.now());
  const lastFetch = useRef(0);
  const fetching = useRef(false);
  const pendingData = useRef<SensorData[] | null>(null);
  const rafRef = useRef(0);
  const frameCount = useRef(0);
  const lastFpsTime = useRef(performance.now());
  const initialLoadDone = useRef(false);

  // ── Query hooks ──────────────────────────────────────────────────

  const { data: apiSensors = [] } = useSensors(authed);
  const sensors = apiSensors;

  const { data: apiScreenSensors = [] } = useScreenSensors(activeScreen, authed);
  const screenSensors = apiScreenSensors;

  const { data: screens = [] } = useScreens(authed);

  const { data: serverLayouts } = useLayouts(activeScreen, authed);
  const screenLayouts = localLayouts ?? (serverLayouts ?? { lg: [], md: [], sm: [] });

  const { data: twinLayouts = [] } = useTwinLayouts(activeScreen, authed);

  const { data: twinConnections = [] } = useTwinConnections(activeScreen, authed);

  const { data: serverCanvasLayouts = [] } = useCanvasLayouts(activeScreen, authed);
  const canvasLayouts = localCanvasLayouts ?? serverCanvasLayouts;

  const { data: shortcuts = {} } = useShortcuts(authed);

  const { data: branding = DEFAULT_BRANDING } = useBranding();

  const { data: systemInfo } = useSystemInfo(authed);

  // ── Mutation hooks ───────────────────────────────────────────────

  const updateSensorMut = useUpdateSensor();
  const deleteSensorMut = useDeleteSensor();
  const addSensorMut = useAddSensor();
  const saveLayoutMut = useSaveLayout();
  const saveTwinLayoutMut = useSaveTwinLayout();
  const saveTwinConnectionsMut = useSaveTwinConnections();
  const saveCanvasLayoutMut = useSaveCanvasLayout();
  const saveShortcutsMut = useSaveShortcuts();
  const saveBrandingMut = useSaveBranding();
  const createScreenMut = useCreateScreen();
  const renameScreenMut = useRenameScreen();
  const deleteScreenMut = useDeleteScreen();
  const addSensorToScreenMut = useAddSensorToScreen();
  const removeSensorFromScreenMut = useRemoveSensorFromScreen();

  // ── Helpers ──────────────────────────────────────────────────────

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
    fetch('/api/auth/me').then(r => r.json()).then(d => { setAuthed(d.authenticated === true); setAuthLoading(false); }).catch(() => { setAuthed(false); setAuthLoading(false); });
  }, []);

  const handleLogin = async (username: string, password: string) => {
    setLoginError('');
    const r = await fetch('/api/auth/login', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ username, password }) });
    if (r.ok) setAuthed(true);
    else { const d = await r.json(); setLoginError(d.error || 'Login failed'); }
  };

  const handleLogout = async () => { await fetch('/api/auth/logout', { method: 'POST' }); setAuthed(false); };

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
        await saveShortcutsMut.mutateAsync({ [s.id]: shortcut });
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
    setScreenPrompt({
      title: 'New Screen', label: 'Screen name', placeholder: 'My Screen', okLabel: 'Create',
      onSubmit: (name) => createScreenMut.mutate(name)
    });
  };

  const handleRenameScreen = (id: string, oldName: string) => {
    setScreenPrompt({
      title: 'Rename Screen', label: 'Screen name', initial: oldName, okLabel: 'Rename',
      onSubmit: (name) => { if (name !== oldName) renameScreenMut.mutate({ id, name }) }
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
      if (result.data) setDataMap(result.data);
    });
  }, [authed, sensors]);

  const fetchData = useCallback(async () => {
    if (fetching.current) return;
    fetching.current = true;
    try {
      const url = activeScreen === 'all' ? '/api/data' : `/api/data?screen=${activeScreen}`;
      const r = await fetch(url);
      if (r.status === 401) { setAuthed(false); return; }
      const d: SensorData[] = await r.json();
      pendingData.current = d;
    } catch { /* ignore */ }
    fetching.current = false;
  }, [activeScreen]);

  // ── rAF polling loop ─────────────────────────────────────────────

  const dataMapRef = useRef(dataMap);
  dataMapRef.current = dataMap;
  const currentValuesRef = useRef(currentValues);
  currentValuesRef.current = currentValues;

  useEffect(() => {
    if (!authed) return;
    const loop = (now: number) => {
      rafRef.current = requestAnimationFrame(loop);
      frameCount.current++;
      if (now - lastFpsTime.current >= 1000) { setFps(frameCount.current); frameCount.current = 0; lastFpsTime.current = now; }
      if (paused || mode !== 'live') return;
      const interval = turbo ? 1000 / 120 : dashSettings.refreshInterval;
      if (now - lastFetch.current >= interval) { lastFetch.current = now; fetchData(); }
      if (pendingData.current) {
        const d = pendingData.current;
        pendingData.current = null;
        const newMap = { ...dataMapRef.current };
        const newVals = { ...currentValuesRef.current };
        let alertCount = 0;
        for (const item of d) {
          const arr = newMap[item.key] ? [...newMap[item.key]] : [];
          arr.push({ value: item.value, ts: Date.now() });
          if (arr.length > (turbo ? MAX_PTS : NORM_PTS)) arr.shift();
          newMap[item.key] = arr;
          newVals[item.key] = item;
          if (item.source_type === 'modbus' && ((item.crit > 0 && item.value >= item.crit) || (item.warn > 0 && item.value >= item.warn))) {
            alertCount++;
          }
        }
        setDataMap(newMap);
        setCurrentValues(newVals);
        setAlerts(alertCount);
        // Alarm audio: start on new breaches, stop when all clear
        if (alertCount > 0) {
          startAlarm();
        } else {
          stopAlarm();
        }
      }
    };
    rafRef.current = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(rafRef.current);
  }, [authed, paused, turbo, dashSettings.refreshInterval, fetchData, mode]);

  // ── History fetch ────────────────────────────────────────────────

  useEffect(() => {
    if (mode !== 'history') { setHistoryMap(null); return; }
    const dp = activeScreen === 'all' ? '' : `&screen=${activeScreen}`;
    fetch(`/api/history?range=${historyRange}${dp}`).then(r => r.json()).then(result => { if (result.data) setHistoryMap(result.data); });
  }, [mode, historyRange, activeScreen, sensors, screenSensors]);

  // ── Keyboard shortcuts ───────────────────────────────────────────

  const GAP = dashSettings.gap;

  const shortcutToSensorKey = useMemo(() => {
    const map = new Map<string, string>();
    for (const [id, sc] of Object.entries(shortcuts)) {
      const sensor = sensors.find(s => s.id === Number(id));
      if (sensor) map.set(sc.toLowerCase(), sensor.key);
    }
    return map;
  }, [shortcuts, sensors]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (!el) return;
      if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT' || el.isContentEditable) return;
      const combo = normalizeKeyEvent(e).toLowerCase();
      const sensorKey = shortcutToSensorKey.get(combo);
      if (sensorKey) {
        e.preventDefault();
        setSelectedSensor(sensorKey);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [shortcutToSensorKey]);

  // ── Render gates ─────────────────────────────────────────────────

  if (authLoading) return <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)', color: 'var(--muted)', fontFamily: 'Inter,sans-serif' }}>Loading...</div>;
  if (!authed) return <LoginScreen onLogin={handleLogin} error={loginError} branding={branding} />;

  const chartDataFor = (key: string) => {
    if (mode === 'history' && historyMap && historyMap[key]) return historyMap[key].map(d => d.value);
    return (dataMap[key] || []).map(d => d.value);
  };
  const detailDataFor = (key: string): DataPoint[] => {
    if (mode === 'history' && historyMap && historyMap[key]) return historyMap[key];
    return dataMap[key] || [];
  };

  return (
    <div style={{ maxWidth: 1600, margin: '0 auto', padding: 24 }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexWrap: 'wrap', gap: 16 }}>
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
            <button onClick={() => setShowAlarmPanel(true)} style={{
              ...btnStyle, position: 'relative', borderColor: alerts > 0 ? 'var(--crit)' : 'var(--border)',
              color: alerts > 0 ? 'var(--crit)' : 'var(--fg)',
            }}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
              {alerts > 0 && <span style={{
                position: 'absolute', top: -4, right: -4, background: 'var(--crit)',
                color: '#fff', borderRadius: '50%', width: 16, height: 16, fontSize: 10,
                display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 700,
              }}>{alerts}</span>}
            </button>
            {/* Mute toggle */}
            <button onClick={() => { toggleMute(); setAlarmMuted(!alarmMuted); }} style={{
              ...btnStyle, borderColor: alarmMuted ? 'var(--warn)' : 'var(--border)',
              color: alarmMuted ? 'var(--warn)' : 'var(--fg)',
            }}>
              {alarmMuted ? (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/><line x1="1" y1="1" x2="23" y2="23" stroke="currentColor" strokeWidth="2"/></svg>
              ) : (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
              )}
            </button>
          {/* View mode toggle */}
          <div style={{ display: 'flex', borderRadius: 6, overflow: 'hidden', border: '1px solid var(--border)' }}>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'cards' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'cards' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setViewMode('cards')}>Grids</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'canvas' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'canvas' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setViewMode('canvas')}>Canvas</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: viewMode === 'twin' ? 'var(--cyan)' : 'var(--fg)', background: viewMode === 'twin' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setViewMode('twin')}>3D</button>
          </div>
          {/* Live/History toggle */}
          <div style={{ display: 'flex', borderRadius: 6, overflow: 'hidden', border: '1px solid var(--border)' }}>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'live' ? 'var(--cyan)' : 'var(--fg)', background: mode === 'live' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => setMode('live')}>Live</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'history' ? 'var(--warn)' : 'var(--fg)', background: mode === 'history' ? 'rgba(234,179,8,.1)' : 'var(--card)' }} onClick={() => setMode('history')}>History</button>
            <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'system' ? '#a78bfa' : 'var(--fg)', background: mode === 'system' ? 'rgba(167,139,250,.1)' : 'var(--card)' }} onClick={() => setMode('system')}>System</button>
          </div>
          {mode === 'history' && (
            <HistoryRangeBar range={historyRange} onChange={setHistoryRange} />
          )}
          {mode === 'live' && (<>
            <button style={btnStyle} onClick={() => setShowDashSettings(true)}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
              <span>Settings</span>
            </button>
            <button style={btnStyle} onClick={() => setPaused(!paused)}>
              {paused ? <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="5,3 19,12 5,21"/></svg> : <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>}
              <span>{paused ? 'Resume' : 'Pause'}</span>
            </button>
            <button style={{ ...btnStyle, borderColor: turbo ? 'var(--warn)' : 'var(--border)', color: turbo ? 'var(--warn)' : 'var(--fg)' }} onClick={() => setTurbo(!turbo)}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10"/></svg>
              <span>{turbo ? 'Turbo ON' : 'Turbo'}</span>
            </button>
          </>)}
          <button style={{ ...btnStyle, borderColor: editMode ? 'var(--cyan)' : 'var(--border)', color: editMode ? 'var(--cyan)' : 'var(--fg)' }} onClick={() => {
            if (editMode) {
              // Persist card layouts if changed
              if (localLayouts) {
                saveLayoutMut.mutate({ screenId: activeScreen, layouts: localLayouts });
                setLocalLayouts(null);
              }
              // Persist canvas layouts if changed
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
          {editMode && activeScreen !== 'all' && <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={() => setShowAddDialog(true)}>+ Add Source</button>}
          <button style={btnStyle} onClick={() => setShowSensorManager(true)}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
            <span>Sources</span>
          </button>
          <button style={btnStyle} onClick={handleLogout}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
            <span>Logout</span>
          </button>
        </div>
      </div>

      {/* Screen navigation */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 0, marginBottom: 20, borderBottom: '1px solid var(--border)' }}>
        <button style={tabStyle(activeScreen === 'all')} onClick={() => setActiveScreen('all')}>Overview</button>
        {screens.map(s => (
          <div key={s.id} style={{ position: 'relative' }}>
            <button style={tabStyle(activeScreen === s.id)} onClick={() => setActiveScreen(s.id)}>{s.name}</button>
            {editMode && (
              <div style={{ position: 'absolute', top: 2, right: 2, display: 'flex', gap: 2 }}>
                <button onClick={e => { e.stopPropagation(); handleRenameScreen(s.id, s.name); }} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 10, padding: '0 2px', lineHeight: 1 }}>&#9998;</button>
                <button onClick={e => { e.stopPropagation(); handleDeleteScreen(s.id, s.name); }} style={{ background: 'none', border: 'none', color: 'var(--crit)', cursor: 'pointer', fontSize: 10, padding: '0 2px', lineHeight: 1 }}>&times;</button>
              </div>
            )}
          </div>
        ))}
        {editMode && <button onClick={handleAddScreen} style={{ ...tabStyle(false), color: 'var(--green)' }}>+</button>}
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
      ) : (
        <Suspense fallback={<div style={{ height: 400, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--muted)' }}>Loading 3D View...</div>}>
          <DigitalTwinView
            sensors={screenSensors}
            currentValues={currentValues}
            activeScreen={activeScreen}
            twinLayouts={twinLayouts}
            twinConnections={twinConnections}
            onSwitchToCards={() => setViewMode('cards')}
            onEquipmentClick={(eqId) => {
              const sensor = sensors.find(s => s.equipment_id === eqId)
              if (sensor) setSelectedSensor(sensor.key)
            }}
            onSensorUpdate={(s) => updateSensorMut.mutate(s)}
            onSaveLayouts={(equipment) => {
              saveTwinLayoutMut.mutate({
                screenId: activeScreen,
                items: equipment.map(e => ({
                  equipment_id: e.id, equipment_type: e.type,
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
          />
        </Suspense>
      ))}

      {showDashSettings && <DashboardSettings settings={dashSettings} onChange={setDashSettings} onClose={() => setShowDashSettings(false)} />}

      {selectedSensor && (() => {
        const s = sensors.find(x => x.key === selectedSensor);
        return s ? <SensorDetail sensor={s} data={detailDataFor(selectedSensor)} currentValue={currentValues[selectedSensor]} onBack={() => setSelectedSensor(null)} historyRange={historyRange} onRangeChange={setHistoryRange} shortcuts={shortcuts} /> : null;
      })()}

      {showSensorManager && <SensorManager sensors={sensors} onSave={handleSave} onDelete={handleDelete} onAdd={handleAddSensorToManager} onDuplicate={handleDuplicateSensor} onClose={() => setShowSensorManager(false)} shortcuts={shortcuts} />}

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

      <div style={{ position: 'fixed', bottom: 12, right: 12, display: 'flex', alignItems: 'center', gap: 8, zIndex: 51 }}>
        <div style={{ fontFamily: 'Inter,sans-serif', fontSize: 11, color: 'var(--muted)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 10px', borderRadius: 4, display: 'flex', alignItems: 'center', gap: 6 }}>
          <span style={{ color: 'var(--cyan)', fontWeight: 600 }}>Voltrus Lite</span>
        </div>
        <a href="https://joyodigitama.com" target="_blank" rel="noopener noreferrer" style={{ fontFamily: 'Inter,sans-serif', fontSize: 11, color: 'var(--muted)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 10px', borderRadius: 4, textDecoration: 'none' }}>
          by <span style={{ color: 'var(--cyan)', fontWeight: 600 }}>JoyoDigitama</span>
        </a>
        <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 11, color: 'var(--muted)', background: 'var(--card)', border: '1px solid var(--border)', padding: '4px 8px', borderRadius: 4 }}>
          {fps} FPS
        </div>
      </div>
    </div>
  );
}
