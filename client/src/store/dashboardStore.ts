import { createContext, useContext, useRef, useCallback, useSyncExternalStore } from 'react'
import type { DataPoint, SensorData } from '../types'

interface DashboardState {
  dataMap: Record<string, DataPoint[]>
  currentValues: Record<string, SensorData>
  historyMap: Record<string, DataPoint[]> | null
  historyReady: boolean
  sseConnected: boolean
  systemInfo: { hostname: string; os: string; cpu_usage: number; memory_used_mb: number; memory_total_mb: number; memory_percent: number } | null
}

const initialState: DashboardState = {
  dataMap: {},
  currentValues: {},
  historyMap: null,
  historyReady: false,
  sseConnected: false,
  systemInfo: null,
}

type Listener = () => void

function createDashboardStore() {
  let state = initialState
  const listeners = new Set<Listener>()

  const getState = () => state

  const setState = (updater: (prev: DashboardState) => DashboardState) => {
    const next = updater(state)
    if (next !== state) {
      state = next
      listeners.forEach(l => l())
    }
  }

  const subscribe = (listener: Listener) => {
    listeners.add(listener)
    return () => listeners.delete(listener)
  }

  // Batch-friendly updates: merge sensor data without replacing entire maps
  const mergeSensorData = (updates: { id: number | string; points: DataPoint[]; value: SensorData }[]) => {
    if (updates.length === 0) return
    setState(prev => {
      const newDataMap = prev.dataMap
      const newValues = prev.currentValues
      let dataChanged = false
      let valuesChanged = false

      for (const { id, points, value } of updates) {
        const k = String(id)
        if (points !== newDataMap[k]) {
          if (!dataChanged) {
            // shallow clone on first change
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            ;(prev as any).dataMap = { ...newDataMap }
            dataChanged = true
          }
          ;(state as any).dataMap[k] = points
        }
        if (value !== newValues[k]) {
          if (!valuesChanged) {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            ;(prev as any).currentValues = { ...newValues }
            valuesChanged = true
          }
          ;(state as any).currentValues[k] = value
        }
      }
      return prev
    })
  }

  const setHistoryMap = (map: Record<string, DataPoint[]> | null) => {
    setState(prev => ({ ...prev, historyMap: map }))
  }

  const setHistoryReady = (ready: boolean) => {
    setState(prev => ({ ...prev, historyReady: ready }))
  }

  const setSseConnected = (connected: boolean) => {
    setState(prev => ({ ...prev, sseConnected: connected }))
  }

  const setSystemInfo = (info: DashboardState['systemInfo']) => {
    setState(prev => ({ ...prev, systemInfo: info }))
  }

  const clearData = () => {
    setState(() => initialState)
  }

  return {
    getState,
    subscribe,
    setState,
    mergeSensorData,
    setHistoryMap,
    setHistoryReady,
    setSseConnected,
    setSystemInfo,
    clearData,
  }
}

type Store = ReturnType<typeof createDashboardStore>

const DashboardStoreContext = createContext<Store | null>(null)

export const DashboardStoreProvider = DashboardStoreContext.Provider

function useStore() {
  const store = useContext(DashboardStoreContext)
  if (!store) throw new Error('useDashboardStore must be used within DashboardStoreProvider')
  return store
}

// Generic selector hook using useSyncExternalStore
export function useDashboardSelector<T>(selector: (state: DashboardState) => T): T {
  const store = useStore()
  const selectorRef = useRef(selector)
  selectorRef.current = selector

  const getSnapshot = useCallback(() => selectorRef.current(store.getState()), [store])
  const getServerSnapshot = useCallback(() => selectorRef.current(initialState), [])

  return useSyncExternalStore(store.subscribe, getSnapshot, getServerSnapshot)
}

// Public API
export function useSensorDataPoints(sensorId: number | string): DataPoint[] {
  const k = String(sensorId)
  return useDashboardSelector(s => s.dataMap[k] ?? [])
}

export function useSensorCurrentValue(sensorId: number | string): SensorData | undefined {
  const k = String(sensorId)
  return useDashboardSelector(s => s.currentValues[k])
}

export function useHistoryDataPoints(sensorId: number | string): DataPoint[] | undefined {
  const k = String(sensorId)
  return useDashboardSelector(s => s.historyMap?.[k])
}

export function useHistoryReady(): boolean {
  return useDashboardSelector(s => s.historyReady)
}

export function useSseConnected(): boolean {
  return useDashboardSelector(s => s.sseConnected)
}

export function useChartData(sensorId: number | string, mode: 'live' | 'history'): number[] {
  const k = String(sensorId)
  return useDashboardSelector(s => {
    if (mode === 'history' && s.historyMap && s.historyMap[k]) {
      return s.historyMap[k].map(d => d.value)
    }
    return (s.dataMap[k] ?? []).map(d => d.value)
  })
}

export function useDetailData(sensorId: number | string, mode: 'live' | 'history'): DataPoint[] {
  const k = String(sensorId)
  return useDashboardSelector(s => {
    if (mode === 'history' && s.historyMap && s.historyMap[k]) {
      return s.historyMap[k]
    }
    return s.dataMap[k] ?? []
  })
}

export function useDashboardStoreActions() {
  const store = useStore()
  return useRef({
    mergeSensorData: store.mergeSensorData,
    setHistoryMap: store.setHistoryMap,
    setHistoryReady: store.setHistoryReady,
    setSseConnected: store.setSseConnected,
    setSystemInfo: store.setSystemInfo,
    clearData: store.clearData,
    getState: store.getState,
  }).current
}

export function useDashboardSystemInfo() {
  return useDashboardSelector(s => s.systemInfo)
}

export { createDashboardStore, type DashboardState }
