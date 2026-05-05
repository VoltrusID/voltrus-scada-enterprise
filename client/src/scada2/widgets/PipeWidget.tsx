import React, { useRef, useEffect } from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const PipeWidget: React.FC<WidgetProps> = ({ instance, data }) => {
  const config = instance.config ?? {}
  const direction = config.direction ?? 'horizontal'
  const pipeColor = config.pipeColor ?? '#3b82f6'
  const flowSpeed = config.flowSpeed ?? 2
  const thickness = config.thickness ?? 8

  const isFlowing = data?.value != null && data.value > 0

  const containerRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    const container = containerRef.current
    if (!canvas || !container) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const dpr = window.devicePixelRatio || 1
    const w = container.clientWidth
    const h = container.clientHeight
    if (w === 0 || h === 0) return

    canvas.width = w * dpr
    canvas.height = h * dpr
    ctx.scale(dpr, dpr)

    ctx.clearRect(0, 0, w, h)

    const isH = direction === 'horizontal'
    const cx = w / 2
    const cy = h / 2
    const halfT = thickness / 2

    // Outer pipe (dark border)
    ctx.fillStyle = 'var(--border)'
    if (isH) {
      ctx.fillRect(0, cy - halfT - 2, w, thickness + 4)
    } else {
      ctx.fillRect(cx - halfT - 2, 0, thickness + 4, h)
    }

    // Inner pipe (lighter)
    ctx.fillStyle = isFlowing ? pipeColor : 'var(--muted)'
    if (isH) {
      ctx.fillRect(0, cy - halfT, w, thickness)
    } else {
      ctx.fillRect(cx - halfT, 0, thickness, h)
    }

    // Flow animation dashes
    if (isFlowing && flowSpeed > 0) {
      const time = Date.now() / 1000 * flowSpeed * 30
      const dashLen = 12
      const gapLen = 8
      const period = dashLen + gapLen
      const totalLen = isH ? w : h

      ctx.fillStyle = 'rgba(255,255,255,0.2)'
      for (let pos = (time % period) - period; pos < totalLen; pos += period) {
        const len = Math.min(dashLen, totalLen - pos)
        if (pos + len <= 0) continue
        const start = Math.max(0, pos)
        const drawLen = Math.min(len, totalLen - start)
        if (drawLen <= 0) continue

        if (isH) {
          ctx.fillRect(start, cy - halfT, drawLen, thickness)
        } else {
          ctx.fillRect(cx - halfT, start, thickness, drawLen)
        }
      }
    }

    // End caps (flanges)
    ctx.fillStyle = 'var(--fg)'
    ctx.globalAlpha = 0.3
    if (isH) {
      ctx.fillRect(0, cy - halfT - 4, 4, thickness + 8)
      ctx.fillRect(w - 4, cy - halfT - 4, 4, thickness + 8)
    } else {
      ctx.fillRect(cx - halfT - 4, 0, thickness + 8, 4)
      ctx.fillRect(cx - halfT - 4, h - 4, thickness + 8, 4)
    }
    ctx.globalAlpha = 1

  }, [direction, pipeColor, flowSpeed, thickness, data?.value, isFlowing])

  // Animate flow
  useEffect(() => {
    if (!isFlowing || flowSpeed <= 0) return
    let raf: number
    const tick = () => {
      // Trigger re-render by updating canvas
      const canvas = canvasRef.current
      const container = containerRef.current
      if (!canvas || !container) return

      const ctx = canvas.getContext('2d')
      if (!ctx) return

      const dpr = window.devicePixelRatio || 1
      const w = container.clientWidth
      const h = container.clientHeight
      if (w === 0 || h === 0) return

      ctx.clearRect(0, 0, canvas.width, canvas.height)
      canvas.width = w * dpr
      canvas.height = h * dpr
      ctx.scale(dpr, dpr)

      const isH = direction === 'horizontal'
      const cx = w / 2
      const cy = h / 2
      const halfT = thickness / 2

      // Outer pipe
      ctx.fillStyle = 'var(--border)'
      if (isH) ctx.fillRect(0, cy - halfT - 2, w, thickness + 4)
      else ctx.fillRect(cx - halfT - 2, 0, thickness + 4, h)

      // Inner pipe
      ctx.fillStyle = pipeColor
      if (isH) ctx.fillRect(0, cy - halfT, w, thickness)
      else ctx.fillRect(cx - halfT, 0, thickness, h)

      // Flow dashes
      const time = Date.now() / 1000 * flowSpeed * 30
      const dashLen = 12
      const gapLen = 8
      const period = dashLen + gapLen
      const totalLen = isH ? w : h

      ctx.fillStyle = 'rgba(255,255,255,0.25)'
      for (let pos = (time % period) - period; pos < totalLen; pos += period) {
        const len = Math.min(dashLen, totalLen - pos)
        if (pos + len <= 0) continue
        const start = Math.max(0, pos)
        const drawLen = Math.min(len, totalLen - start)
        if (drawLen <= 0) continue

        if (isH) ctx.fillRect(start, cy - halfT, drawLen, thickness)
        else ctx.fillRect(cx - halfT, start, thickness, drawLen)
      }

      // Flanges
      ctx.fillStyle = 'var(--fg)'
      ctx.globalAlpha = 0.3
      if (isH) {
        ctx.fillRect(0, cy - halfT - 4, 4, thickness + 8)
        ctx.fillRect(w - 4, cy - halfT - 4, 4, thickness + 8)
      } else {
        ctx.fillRect(cx - halfT - 4, 0, thickness + 8, 4)
        ctx.fillRect(cx - halfT - 4, h - 4, thickness + 8, 4)
      }
      ctx.globalAlpha = 1

      raf = requestAnimationFrame(tick)
    }
    raf = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(raf)
  }, [isFlowing, flowSpeed, direction, pipeColor, thickness])

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', position: 'relative' }}>
      <canvas ref={canvasRef} style={{ width: '100%', height: '100%', display: 'block' }} />
    </div>
  )
}

registerWidget({ manifest: WIDGET_MANIFESTS.pipe, component: PipeWidget })
