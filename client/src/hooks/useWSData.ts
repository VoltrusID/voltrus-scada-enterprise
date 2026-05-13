import { useEffect, useRef, useCallback } from 'react'
import type { SensorConfig, SensorData } from '../types'
import { getWsInstance, type TagUpdate } from '../api/websocket'
import { useDashboardStoreActions } from '../store/dashboardStore'

const LIVE_PTS = 300

export function useWSData(authed: boolean, sensors: SensorConfig[]) {
  const actions = useDashboardStoreActions()
  const sensorsRef = useRef(sensors)
  sensorsRef.current = sensors

  const handleBatch = useCallback((updates: TagUpdate[]) => {
    const sensorMap = new Map<string, SensorConfig>()
    for (const s of sensorsRef.current) {
      sensorMap.set(s.key, s)
      sensorMap.set(String(s.id), s)
    }

    const state = actions.getState()
    const dataUpdates: { id: string; points: { value: number; ts: number }[]; value: SensorData }[] = []

    for (const u of updates) {
      const sensor = sensorMap.get(u.tag_id)
      if (!sensor) continue

      const k = String(sensor.id)
      const isOffline = u.value == null || Number.isNaN(u.value)
      const value: SensorData = {
        id: sensor.id,
        key: sensor.key,
        value: u.value,
        warn: sensor.warn_val,
        crit: sensor.crit_val,
        source_type: sensor.source_type,
        status: u.quality || 'normal',
      }

      if (isOffline) {
        dataUpdates.push({ id: k, points: state.dataMap[k] ?? [], value })
      } else {
        const prevArr = state.dataMap[k] ?? []
        const newArr = [...prevArr, { value: u.value, ts: Date.now() }]
        const trimmed = newArr.length > LIVE_PTS ? newArr.slice(-LIVE_PTS) : newArr
        dataUpdates.push({ id: k, points: trimmed, value })
      }
    }

    if (dataUpdates.length > 0) {
      actions.mergeSensorData(dataUpdates)
    }
  }, [actions])

  useEffect(() => {
    if (!authed) return

    const ws = getWsInstance()
    if (!ws) {
      actions.setSseConnected(false)
      return
    }

    ws.onTagBatch(handleBatch)
    actions.setSseConnected(true)

    return () => {
      actions.setSseConnected(false)
    }
  }, [authed, handleBatch, actions])
}
