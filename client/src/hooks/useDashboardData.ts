import { useRef, useEffect, useCallback } from 'react'
import { SensorConfig, SensorData, DataPoint, LIVE_PTS } from '../types'
import { expandCompactData } from '../utils'
import { useSSEData } from '../useSSEData'
import { useSystemInfo } from '../queries'
import { useDashboardStoreActions, useHistoryReady } from '../store/dashboardStore'

export function useDashboardData(
  authed: boolean,
  mode: 'live' | 'history' | 'system' | 'flows' | 'assets',
  activeScreen: string,
  sensors: SensorConfig[],
  screenSensors: SensorConfig[],
  historyRange: string,
  historyDetail: 'auto' | 'full',
  historyAutoAgg: string,
  turbo: boolean,
) {
  const initialLoadDone = useRef(false)
  const historyReady = useHistoryReady()

  const actions = useDashboardStoreActions()
  const { data: systemInfo } = useSystemInfo(authed && mode === 'system')

  // Sync systemInfo to store
  useEffect(() => {
    actions.setSystemInfo(systemInfo ?? null)
  }, [systemInfo, actions])

  // SSE batch callback — stable reference so useSSEData doesn't reconnect
  const onBatch = useCallback((batch: SensorData[], system: import('../types').SystemInfo | null) => {
    if (system) actions.setSystemInfo(system)
    if (!batch || batch.length === 0) return

    const updates: { id: string; points: DataPoint[]; value: SensorData }[] = []
    const state = actions.getState()

    for (const item of batch) {
      const k = String(item.id)
      const isOffline = item.value == null || Number.isNaN(item.value)
      if (isOffline) {
        updates.push({ id: k, points: state.dataMap[k] ?? [], value: item })
        continue
      }
      const prevArr = state.dataMap[k] ?? []
      const newArr = [...prevArr, { value: item.value, ts: Date.now() }]
      const trimmed = newArr.length > LIVE_PTS ? newArr.slice(-LIVE_PTS) : newArr
      updates.push({ id: k, points: trimmed, value: item })
    }

    if (updates.length > 0) {
      actions.mergeSensorData(updates)
    }
  }, [actions])

  const onConnected = useCallback((connected: boolean) => {
    actions.setSseConnected(connected)
  }, [actions])

  useSSEData(authed, turbo, onBatch, onConnected)

  // ── Initial data load ────────────────────────────────────────────
  useEffect(() => {
    if (!authed || sensors.length === 0 || initialLoadDone.current) return
    initialLoadDone.current = true
    fetch('/api/v1/history?range=30m').then(r => r.json()).then(result => {
      if (!result.data) return
      const expanded = expandCompactData(result.data)
      const updates: { id: string; points: DataPoint[]; value: SensorData }[] = []
      for (const s of sensors) {
        const pts = expanded[s.key]
        if (pts) {
          const trimmed = pts.length > LIVE_PTS ? pts.slice(-LIVE_PTS) : pts
          const last = trimmed[trimmed.length - 1]
          const fakeValue: SensorData = {
            id: s.id, key: s.key,
            value: last?.value ?? s.base_val,
            warn: s.warn_val, crit: s.crit_val,
            source_type: s.source_type,
            status: 'normal',
          }
          updates.push({ id: String(s.id), points: trimmed, value: fakeValue })
        }
      }
      if (updates.length > 0) actions.mergeSensorData(updates)
      actions.setHistoryReady(true)
    }).catch(() => {}).finally(() => {
      if (!historyReady) actions.setHistoryReady(true)
    })
  }, [authed, sensors, actions, historyReady])

  // ── History fetch ────────────────────────────────────────────────
  useEffect(() => {
    if (mode !== 'history') { actions.setHistoryMap(null); return }
    const dp = activeScreen === 'all' ? '' : `&screen=${activeScreen}`
    const detailParam = historyDetail === 'full' ? '&detail=full' : ''
    const aggParam = historyDetail === 'auto' ? `&agg=${historyAutoAgg}` : ''
    fetch(`/api/v1/history?range=${historyRange}${dp}${detailParam}${aggParam}`).then(r => r.json()).then(result => {
      if (!result.data) return
      const expanded = expandCompactData(result.data)
      const remapped: Record<string, DataPoint[]> = {}
      for (const s of screenSensors) {
        const pts = expanded[s.key]
        if (pts) remapped[String(s.id)] = pts
      }
      actions.setHistoryMap(remapped)
    })
  }, [mode, historyRange, historyDetail, activeScreen, screenSensors, historyAutoAgg, actions])

  return {
    historyReady,
    systemInfo,
  }
}
