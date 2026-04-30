import { useState, useCallback, useRef, useEffect } from 'react'

export type TransitionPhase =
  | 'idle'
  | 'capturing'
  | 'animating-forward'
  | 'settled'
  | 'capturing-back'
  | 'animating-back'

export interface TransitionState {
  phase: TransitionPhase
  sensorId: number | null
  fromRect: DOMRect | null
  toRect: DOMRect | null
}

export const FORWARD_DURATION = 400
export const REVERSE_DURATION = 350

function computeDetailRect(): DOMRect {
  const vw = window.innerWidth
  const vh = window.innerHeight
  const width = Math.max(320, Math.min(1200, vw - 80))
  const height = Math.max(200, vh - 120)
  const left = (vw - width) / 2
  const top = Math.max(20, (vh - height) / 2 - 20)
  return new DOMRect(left, top, width, height)
}

export function useChartTransition() {
  const [state, setState] = useState<TransitionState>({
    phase: 'idle',
    sensorId: null,
    fromRect: null,
    toRect: null,
  })

  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const rafRef = useRef<number | null>(null)

  const clearTimers = useCallback(() => {
    if (timerRef.current) { clearTimeout(timerRef.current); timerRef.current = null }
    if (rafRef.current) { cancelAnimationFrame(rafRef.current); rafRef.current = null }
  }, [])

  const start = useCallback((sensorId: number, cardEl: HTMLElement) => {
    clearTimers()

    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    if (reduced) {
      setState({ phase: 'settled', sensorId, fromRect: null, toRect: null })
      return
    }

    const cardRect = cardEl.getBoundingClientRect()
    const detailRect = computeDetailRect()

    setState({
      phase: 'capturing',
      sensorId,
      fromRect: cardRect,
      toRect: detailRect,
    })

    rafRef.current = requestAnimationFrame(() => {
      rafRef.current = requestAnimationFrame(() => {
        setState(prev => ({ ...prev, phase: 'animating-forward' }))
        timerRef.current = setTimeout(() => {
          setState(prev => ({ ...prev, phase: 'settled' }))
        }, FORWARD_DURATION)
      })
    })
  }, [clearTimers])

  const reverse = useCallback((sensorId: number) => {
    clearTimers()

    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    if (reduced) {
      setState({ phase: 'idle', sensorId: null, fromRect: null, toRect: null })
      return
    }

    const cardEl = document.querySelector(`[data-sensor-id="${sensorId}"]`)
    const rawRect = cardEl?.getBoundingClientRect()
    const cardRect = rawRect && rawRect.width > 0 && rawRect.height > 0 ? rawRect : null
    const detailRect = computeDetailRect()

    setState({
      phase: 'capturing-back',
      sensorId,
      fromRect: detailRect,
      toRect: cardRect,
    })

    rafRef.current = requestAnimationFrame(() => {
      rafRef.current = requestAnimationFrame(() => {
        setState(prev => ({ ...prev, phase: 'animating-back' }))
        timerRef.current = setTimeout(() => {
          setState({ phase: 'idle', sensorId: null, fromRect: null, toRect: null })
        }, REVERSE_DURATION)
      })
    })
  }, [clearTimers])

  useEffect(() => {
    return () => clearTimers()
  }, [clearTimers])

  return { state, start, reverse }
}
