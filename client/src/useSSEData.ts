import { useEffect, useRef, useCallback } from 'react'
import type { SensorData, SystemInfo } from './types'

interface SSEMessage {
  sensors?: SensorData[]
  system?: SystemInfo
}

const FLUSH_INTERVAL_MS = 1000 // batch updates to 1Hz max
const STALL_TIMEOUT_MS = 5000  // reconnect if no message for 5s

export function useSSEData(
  enabled: boolean,
  turbo: boolean,
  onBatch: (sensors: SensorData[], system: SystemInfo | null) => void,
  onConnected: (connected: boolean) => void,
) {
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const bufferRef = useRef<SensorData[] | null>(null)
  const systemRef = useRef<SystemInfo | null>(null)
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const evtSourceRef = useRef<EventSource | null>(null)
  const lastMessageAtRef = useRef<number>(0)
  const stallTimerRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const turboRef = useRef(turbo)
  turboRef.current = turbo

  // Keep callbacks fresh without triggering effect re-runs
  const onBatchRef = useRef(onBatch)
  onBatchRef.current = onBatch
  const onConnectedRef = useRef(onConnected)
  onConnectedRef.current = onConnected

  const flush = useCallback(() => {
    const batch = bufferRef.current
    if (batch && batch.length > 0) {
      onBatchRef.current(batch, systemRef.current)
      bufferRef.current = null
    } else if (systemRef.current) {
      onBatchRef.current([], systemRef.current)
    }
    flushTimerRef.current = null
  }, [])

  const scheduleFlush = useCallback(() => {
    if (!flushTimerRef.current) {
      flushTimerRef.current = setTimeout(flush, FLUSH_INTERVAL_MS)
    }
  }, [flush])

  const teardown = useCallback(() => {
    abortRef.current?.abort()
    evtSourceRef.current?.close()
    evtSourceRef.current = null
    if (reconnectTimer.current) { clearTimeout(reconnectTimer.current); reconnectTimer.current = null }
    if (stallTimerRef.current) { clearInterval(stallTimerRef.current); stallTimerRef.current = null }
  }, [])

  const connect = useCallback(() => {
    if (!enabled) return

    teardown()
    const abort = new AbortController()
    abortRef.current = abort

    const evtSource = new EventSource('/api/v1/stream')
    evtSourceRef.current = evtSource
    lastMessageAtRef.current = Date.now()

    evtSource.onopen = () => {
      if (abort.signal.aborted) { evtSource.close(); return }
      onConnectedRef.current(true)
    }

    evtSource.onmessage = (e) => {
      if (abort.signal.aborted) { evtSource.close(); return }
      lastMessageAtRef.current = Date.now()
      try {
        const parsed: SSEMessage = JSON.parse(e.data)
        if (parsed.sensors) {
          if (bufferRef.current) {
            bufferRef.current.push(...parsed.sensors)
          } else {
            bufferRef.current = parsed.sensors
          }
        }
        if (parsed.system) {
          systemRef.current = parsed.system
        }
        if (turboRef.current) {
          if (flushTimerRef.current) {
            clearTimeout(flushTimerRef.current)
            flushTimerRef.current = null
          }
          flush()
        } else {
          scheduleFlush()
        }
      } catch { /* ignore parse errors */ }
    }

    evtSource.onerror = () => {
      evtSource.close()
      if (!abort.signal.aborted) {
        onConnectedRef.current(false)
        flush()
        reconnectTimer.current = setTimeout(connect, 2000)
      }
    }

    stallTimerRef.current = setInterval(() => {
      if (abort.signal.aborted) return
      if (Date.now() - lastMessageAtRef.current > STALL_TIMEOUT_MS) {
        evtSource.close()
        onConnectedRef.current(false)
        flush()
        reconnectTimer.current = setTimeout(connect, 1000)
      }
    }, 2000)
  }, [enabled, scheduleFlush, flush, teardown])

  useEffect(() => {
    if (!enabled) {
      teardown()
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current)
      bufferRef.current = null
      systemRef.current = null
      onConnectedRef.current(false)
      return
    }
    connect()
    return () => {
      teardown()
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current)
      bufferRef.current = null
      systemRef.current = null
    }
  }, [enabled, connect, teardown])
}
