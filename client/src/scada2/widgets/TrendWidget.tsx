import React, { useRef, useEffect } from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const TrendWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, data } = props
  const config = instance.config ?? {}
  const lineColor = config.lineColor ?? 'var(--cyan)'
  const pointsCount = config.pointsCount ?? 60

  const canvasRef = useRef<HTMLCanvasElement>(null)
  const bufferRef = useRef<number[]>([])

  // Resolve CSS variable to actual color for canvas
  const resolveColor = (color: string): string => {
    if (color.startsWith('var(')) {
      const varName = color.slice(4, -1)
      if (typeof window !== 'undefined') {
        return getComputedStyle(document.documentElement).getPropertyValue(varName).trim() || '#22d3ee'
      }
      return '#22d3ee'
    }
    return color
  }

  useEffect(() => {
    // Push current value into buffer
    const val = data?.value
    if (val !== undefined && val !== null) {
      bufferRef.current.push(val)
      if (bufferRef.current.length > pointsCount) {
        bufferRef.current.shift()
      }
    }
  }, [data?.value, pointsCount])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const dpr = window.devicePixelRatio || 1
    const rect = canvas.getBoundingClientRect()
    const w = rect.width
    const h = rect.height

    if (w === 0 || h === 0) return

    canvas.width = w * dpr
    canvas.height = h * dpr
    ctx.scale(dpr, dpr)

    // Clear
    ctx.clearRect(0, 0, w, h)

    const buf = bufferRef.current
    if (buf.length < 2) {
      // Not enough data - draw placeholder
      ctx.strokeStyle = 'var(--muted)'
      ctx.lineWidth = 1
      ctx.setLineDash([4, 4])
      ctx.beginPath()
      ctx.moveTo(0, h / 2)
      ctx.lineTo(w, h / 2)
      ctx.stroke()
      ctx.setLineDash([])
      return
    }

    const resolvedLineColor = resolveColor(lineColor)

    // Determine y range from buffer
    let yMin = Infinity
    let yMax = -Infinity
    for (const v of buf) {
      if (v < yMin) yMin = v
      if (v > yMax) yMax = v
    }
    // Add padding to range
    const yRange = yMax - yMin || 1
    yMin -= yRange * 0.05
    yMax += yRange * 0.05
    const totalRange = yMax - yMin || 1

    const pad = 4
    const plotW = w - pad * 2
    const plotH = h - pad * 2

    // Map data points to canvas coordinates
    const points = buf.map((v, i) => ({
      x: pad + (i / (buf.length - 1)) * plotW,
      y: pad + plotH - ((v - yMin) / totalRange) * plotH,
    }))

    // Gradient fill under line
    const gradient = ctx.createLinearGradient(0, pad, 0, pad + plotH)
    gradient.addColorStop(0, resolvedLineColor + '40')
    gradient.addColorStop(1, resolvedLineColor + '05')

    ctx.beginPath()
    ctx.moveTo(points[0].x, points[0].y)
    for (let i = 1; i < points.length; i++) {
      ctx.lineTo(points[i].x, points[i].y)
    }
    // Close path for fill
    ctx.lineTo(points[points.length - 1].x, pad + plotH)
    ctx.lineTo(points[0].x, pad + plotH)
    ctx.closePath()
    ctx.fillStyle = gradient
    ctx.fill()

    // Draw line
    ctx.beginPath()
    ctx.moveTo(points[0].x, points[0].y)
    for (let i = 1; i < points.length; i++) {
      ctx.lineTo(points[i].x, points[i].y)
    }
    ctx.strokeStyle = resolvedLineColor
    ctx.lineWidth = 1.5
    ctx.lineJoin = 'round'
    ctx.stroke()

    // Draw current value dot
    const last = points[points.length - 1]
    ctx.beginPath()
    ctx.arc(last.x, last.y, 3, 0, Math.PI * 2)
    ctx.fillStyle = resolvedLineColor
    ctx.fill()

  }, [data?.value, lineColor, pointsCount])

  const containerStyle: React.CSSProperties = {
    width: '100%',
    height: '100%',
    position: 'relative',
    overflow: 'hidden',
  }

  const canvasStyle: React.CSSProperties = {
    width: '100%',
    height: '100%',
    display: 'block',
  }

  return (
    <div style={containerStyle}>
      <canvas ref={canvasRef} style={canvasStyle} />
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.trend,
  component: TrendWidget,
})

export default TrendWidget
