import { useState, useRef, useEffect, useCallback, useMemo } from 'react'
import { SensorConfig, SensorData, DataPoint, SystemInfo, LIVE_PTS } from '../types'
import { expandCompactData } from '../utils'
import { useSSEData } from '../useSSEData'

export function useDashboardData(authed: boolean, mode: 'live' | 'history' | 'system', activeScreen: string, sensors: SensorConfig[], screenSensors: SensorConfig[], historyRange: string, historyDetail: 'auto' | 'full', historyAutoAgg: string, turbo: boolean) {
  const [dataMap, setDataMap] = useState<Record<string, DataPoint[]>>({});
  const [historyMap, setHistoryMap] = useState<Record<string, DataPoint[]> | null>(null);
  const [historyReady, setHistoryReady] = useState(false);
  const [currentValues, setCurrentValues] = useState<Record<string, SensorData>>({});
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const initialLoadDone = useRef(false);

  const sseData = useSSEData(authed, turbo);

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
    if (!sseData.sensors || !historyReady) return;
    const d = sseData.sensors;
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
  }, [sseData.sensors, historyReady]);

  // ── System info from SSE ─────────────────────────────────────────
  useEffect(() => {
    if (sseData.system) {
      setSystemInfo(sseData.system);
    }
  }, [sseData.system]);

  // ── History fetch ────────────────────────────────────────────────
  useEffect(() => {
    if (mode !== 'history') { setHistoryMap(null); return; }
    const dp = activeScreen === 'all' ? '' : `&screen=${activeScreen}`;
    const detailParam = historyDetail === 'full' ? '&detail=full' : '';
    const aggParam = historyDetail === 'auto' ? `&agg=${historyAutoAgg}` : '';
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
  }, [mode, historyRange, historyDetail, activeScreen, sensors, screenSensors, historyAutoAgg]);

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

  return {
    dataMap, currentValues, historyMap, historyReady, sseConnected: sseData.connected, systemInfo,
    chartDataFor, detailDataFor,
  };
}
