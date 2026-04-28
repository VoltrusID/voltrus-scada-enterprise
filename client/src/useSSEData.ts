import { useEffect, useRef, useCallback, useState } from 'react'
import type { SensorData } from './types'

interface SSEDataState {
  data: SensorData[] | null
  connected: boolean
}

export function useSSEData(enabled: boolean) {
  const [state, setState] = useState<SSEDataState>({ data: null, connected: false })
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const abortRef = useRef<AbortController | null>(null)

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
        setState({ data: parsed, connected: true })
      } catch { /* ignore parse errors */ }
    }

    evtSource.onerror = () => {
      evtSource.close()
      if (!abort.signal.aborted) {
        setState(s => ({ ...s, connected: false }))
        // Reconnect after 2s
        reconnectTimer.current = setTimeout(connect, 2000)
      }
    }
  }, [enabled])

  useEffect(() => {
    if (!enabled) {
      abortRef.current?.abort()
      setState({ data: null, connected: false })
      return
    }
    connect()
    return () => {
      abortRef.current?.abort()
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current)
    }
  }, [enabled, connect])

  return state
}
