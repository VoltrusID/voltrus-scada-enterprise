import { useEffect, useRef, useCallback, useState } from 'react'
import type { SensorData, SystemInfo } from './types'

interface SSEMessage {
  sensors?: SensorData[]
  system?: SystemInfo
}

interface SSEState {
  sensors: SensorData[] | null
  system: SystemInfo | null
  connected: boolean
}

const FLUSH_INTERVAL_MS = 1000 // batch updates to 1Hz max
const STALL_TIMEOUT_MS = 5000  // reconnect if no message for 5s

export function useSSEData(enabled: boolean, turbo: boolean) {
  const [state, setState] = useState<SSEState>({ sensors: null, system: null, connected: false })
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

  const flush = useCallback(() => {
    const batch = bufferRef.current
    if (batch && batch.length > 0) {
      setState({ sensors: batch, system: systemRef.current, connected: true })
      bufferRef.current = null
    } else if (systemRef.current) {
      // Even if no new sensors, system info might have updated
      setState(s => (s.system !== systemRef.current ? { ...s, system: systemRef.current, connected: true } : s))
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

    // Clean up previous connection
    teardown()
    const abort = new AbortController()
    abortRef.current = abort

    const evtSource = new EventSource('/api/stream')
    evtSourceRef.current = evtSource
    lastMessageAtRef.current = Date.now()

    evtSource.onopen = () => {
      if (abort.signal.aborted) { evtSource.close(); return }
      setState(s => ({ ...s, connected: true }))
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
          // Immediate flush — render at SSE rate
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
        setState(s => ({ ...s, connected: false }))
        // Flush any pending buffered data before reconnect
        flush()
        // Reconnect after 2s
        reconnectTimer.current = setTimeout(connect, 2000)
      }
    }

    // Stall watchdog: if no message arrives within STALL_TIMEOUT_MS, force reconnect
    stallTimerRef.current = setInterval(() => {
      if (abort.signal.aborted) return
      if (Date.now() - lastMessageAtRef.current > STALL_TIMEOUT_MS) {
        // Connection is hung — tear down and reconnect
        evtSource.close()
        setState(s => ({ ...s, connected: false }))
        flush()
        reconnectTimer.current = setTimeout(connect, 1000)
      }
    }, 2000) // check every 2 seconds
  }, [enabled, scheduleFlush, flush, teardown])

  useEffect(() => {
    if (!enabled) {
      teardown()
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current)
      bufferRef.current = null
      systemRef.current = null
      setState({ sensors: null, system: null, connected: false })
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

  return state
}
