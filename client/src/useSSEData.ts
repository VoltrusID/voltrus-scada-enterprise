import { useEffect, useRef, useCallback, useState } from 'react'
import type { SensorData } from './types'

interface SSEDataState {
  data: SensorData[] | null
  connected: boolean
}

const FLUSH_INTERVAL_MS = 1000 // batch updates to 1Hz max

export function useSSEData(enabled: boolean) {
  const [state, setState] = useState<SSEDataState>({ data: null, connected: false })
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const bufferRef = useRef<SensorData[] | null>(null)
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

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

  const connect = useCallback(() => {
    if (!enabled) return

    // Clean up previous connection
    abortRef.current?.abort()
    const abort = new AbortController()
    abortRef.current = abort

    const evtSource = new EventSource('/api/stream')

    evtSource.onopen = () => {
      if (abort.signal.aborted) { evtSource.close(); return }
      setState(s => ({ ...s, connected: true }))
    }

    evtSource.onmessage = (e) => {
      if (abort.signal.aborted) { evtSource.close(); return }
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
  }, [enabled, scheduleFlush, flush])

  useEffect(() => {
    if (!enabled) {
      abortRef.current?.abort()
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current)
      bufferRef.current = null
      setState({ data: null, connected: false })
      return
    }
    connect()
    return () => {
      abortRef.current?.abort()
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current)
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current)
      bufferRef.current = null
    }
  }, [enabled, connect])

  return state
}
