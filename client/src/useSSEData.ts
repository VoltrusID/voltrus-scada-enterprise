import { useEffect, useRef, useCallback, useState } from 'react'
import type { SensorData } from './types'

interface SSEDataState {
  data: SensorData[] | null
  connected: boolean
}

const FLUSH_INTERVAL_MS = 1000 // batch updates to 1Hz max
const STALL_TIMEOUT_MS = 5000  // reconnect if no message for 5s

export function useSSEData(enabled: boolean) {
  const [state, setState] = useState<SSEDataState>({ data: null, connected: false })
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const bufferRef = useRef<SensorData[] | null>(null)
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const evtSourceRef = useRef<EventSource | null>(null)
  const lastMessageAtRef = useRef<number>(0)
  const stallTimerRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const flush = useCallback(() => {
    const batch = bufferRef.current
    if (batch && batch.length > 0) {
      setState({ data: batch, connected: true })
      bufferRef.current = null
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
        const parsed: SensorData[] = JSON.parse(e.data)
        // Buffer latest data — if buffer already has data, append
        if (bufferRef.current) {
          bufferRef.current.push(...parsed)
        } else {
          bufferRef.current = parsed
        }
        scheduleFlush()
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
      setState({ data: null, connected: false })
      return
    }
    connect()
    return () => {
      teardown()
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current)
      bufferRef.current = null
    }
  }, [enabled, connect, teardown])

  return state
}
