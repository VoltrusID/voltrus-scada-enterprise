import { useMemo } from 'react'
import { TransitionState, FORWARD_DURATION, REVERSE_DURATION } from '../hooks/useChartTransition'

interface Props {
  state: TransitionState
}

export function ChartTransitionOverlay({ state }: Props) {
  const { phase, fromRect, toRect } = state

  const isActive = phase === 'animating-forward' || phase === 'animating-back' || phase === 'capturing' || phase === 'capturing-back'
  const showClone = phase !== 'idle' && phase !== 'settled' && fromRect != null

  const cloneStyle = useMemo((): React.CSSProperties => {
    if (!fromRect || !showClone) return { display: 'none' }

    const base: React.CSSProperties = {
      position: 'fixed',
      left: fromRect.left,
      top: fromRect.top,
      width: fromRect.width,
      height: fromRect.height,
      borderRadius: 10,
      background: 'var(--card)',
      border: '1px solid var(--border)',
      boxShadow: '0 24px 48px rgba(0,0,0,0.4)',
      zIndex: 101,
      willChange: 'transform',
      transformOrigin: 'top left',
    }

    if (phase === 'capturing' || phase === 'capturing-back') {
      return { ...base, transform: 'none', transition: 'none' }
    }

    if (phase === 'animating-forward' && toRect) {
      const sx = toRect.width / fromRect.width
      const sy = toRect.height / fromRect.height
      const tx = toRect.left - fromRect.left
      const ty = toRect.top - fromRect.top
      return {
        ...base,
        transform: `translate(${tx}px, ${ty}px) scale(${sx}, ${sy})`,
        transition: `transform ${FORWARD_DURATION}ms cubic-bezier(0.32, 0.72, 0, 1)`,
      }
    }

    if (phase === 'animating-back') {
      if (toRect) {
        const sx = toRect.width / fromRect.width
        const sy = toRect.height / fromRect.height
        const tx = toRect.left - fromRect.left
        const ty = toRect.top - fromRect.top
        return {
          ...base,
          transform: `translate(${tx}px, ${ty}px) scale(${sx}, ${sy})`,
          transition: `transform ${REVERSE_DURATION}ms cubic-bezier(0.32, 0.72, 0, 1)`,
        }
      } else {
        // Fallback: shrink to center + fade
        const cx = window.innerWidth / 2 - fromRect.left - fromRect.width / 2
        const cy = window.innerHeight / 2 - fromRect.top - fromRect.height / 2
        return {
          ...base,
          transform: `translate(${cx}px, ${cy}px) scale(0.1)`,
          opacity: 0,
          transition: `transform ${REVERSE_DURATION}ms cubic-bezier(0.32, 0.72, 0, 1), opacity ${REVERSE_DURATION}ms ease`,
        }
      }
    }

    return { display: 'none' }
  }, [phase, fromRect, toRect, showClone])

  const backdropStyle: React.CSSProperties = {
    position: 'fixed',
    inset: 0,
    zIndex: 100,
    background: isActive ? 'rgba(0,0,0,0.5)' : 'rgba(0,0,0,0)',
    backdropFilter: isActive ? 'blur(8px)' : 'blur(0px)',
    WebkitBackdropFilter: isActive ? 'blur(8px)' : 'blur(0px)',
    transition: `background ${FORWARD_DURATION}ms ease, backdrop-filter ${FORWARD_DURATION}ms ease`,
    pointerEvents: isActive ? 'auto' : 'none',
  }

  if (phase === 'idle') return null

  return (
    <div style={backdropStyle}>
      <div style={cloneStyle} />
    </div>
  )
}
