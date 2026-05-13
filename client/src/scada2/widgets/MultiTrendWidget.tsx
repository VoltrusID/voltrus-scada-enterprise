import React, { useRef, useEffect } from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const MultiTrendWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensors, currentValues } = props
  const config = instance.config ?? {}
  const maxTags = config.maxTags ?? 4
  const pointsCount = config.pointsCount ?? 60

  const canvasRef = useRef<HTMLCanvasElement>(null)
  const bufferRef = useRef<Map<string, number[]>>(new Map())

  const boundSensors = instance.sensorIds.length > 0
    ? sensors.filter(s => instance.sensorIds.includes(s.id))
    : instance.sensorId
      ? sensors.filter(s => s.id === instance.sensorId)
      : sensors.slice(0, maxTags)

  const displaySensors = boundSensors.length > 0 ? boundSensors.slice(0, maxTags) : sensors.slice(0, maxTags)

  const palette = ['var(--cyan)', 'var(--ok)', 'var(--warn)', '#8b5cf6', '#f97316']

  useEffect(() => {
    for (const sensor of displaySensors) {
      const d = currentValues[sensor.key]
      if (d?.value !== undefined) {
        let buf = bufferRef.current.get(sensor.key)
        if (!buf) {
          buf = []
          bufferRef.current.set(sensor.key, buf)
        }
        buf.push(d.value)
        if (buf.length > pointsCount) buf.shift()
      }
    }
  }, [currentValues, displaySensors, pointsCount])

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
    ctx.clearRect(0, 0, w, h)

    if (displaySensors.length === 0) {
      ctx.strokeStyle = 'var(--muted)'
      ctx.lineWidth = 1
      ctx.setLineDash([4, 4])
      ctx.beginPath()
      ctx.moveTo(0, h / 2)
      ctx.lineTo(w, h / 2)
      ctx.stroke()
      return
    }

    let yMin = Infinity
    let yMax = -Infinity
    for (const sensor of displaySensors) {
      const buf = bufferRef.current.get(sensor.key) ?? []
      for (const v of buf) {
        if (v < yMin) yMin = v
        if (v > yMax) yMax = v
      }
    }
    if (yMin === Infinity) return

    const yRange = yMax - yMin || 1
    yMin -= yRange * 0.05
    yMax += yRange * 0.05
    const totalRange = yMax - yMin || 1

    const pad = 4
    const plotW = w - pad * 2
    const plotH = h - pad * 2

    for (let si = 0; si < displaySensors.length; si++) {
      const sensor = displaySensors[si]
      const buf = bufferRef.current.get(sensor.key) ?? []
      if (buf.length < 2) continue

      const color = resolveColor(palette[si % palette.length])

      const points = buf.map((v, i) => ({
        x: pad + (i / (buf.length - 1)) * plotW,
        y: pad + plotH - ((v - yMin) / totalRange) * plotH,
      }))

      ctx.beginPath()
      ctx.moveTo(points[0].x, points[0].y)
      for (let i = 1; i < points.length; i++) {
        ctx.lineTo(points[i].x, points[i].y)
      }
      ctx.strokeStyle = color
      ctx.lineWidth = 1.5
      ctx.lineJoin = 'round'
      ctx.stroke()

      const last = points[points.length - 1]
      ctx.beginPath()
      ctx.arc(last.x, last.y, 3, 0, Math.PI * 2)
      ctx.fillStyle = color
      ctx.fill()
    }

    const legendY = h - 8
    let legendX = 4
    ctx.font = '9px sans-serif'
    for (let si = 0; si < displaySensors.length; si++) {
      const color = resolveColor(palette[si % palette.length])
      const name = displaySensors[si].name.length > 8
        ? displaySensors[si].name.slice(0, 7) + '..'
        : displaySensors[si].name

      ctx.fillStyle = color
      ctx.fillRect(legendX, legendY - 6, 8, 8)
      legendX += 10
      ctx.fillStyle = 'var(--muted)'
      ctx.fillText(name, legendX, legendY + 1)
      legendX += ctx.measureText(name).width + 8
    }
  }, [currentValues, displaySensors, pointsCount])

  return (
    <div style={{ width: '100%', height: '100%', position: 'relative', overflow: 'hidden' }}>
      {displaySensors.length === 0 ? (
        <div style={{
          position: 'absolute', inset: 0, display: 'flex',
          alignItems: 'center', justifyContent: 'center',
          color: 'var(--muted)', fontSize: 11,
        }}>
          Drop tags here
        </div>
      ) : null}
      <canvas ref={canvasRef} style={{ width: '100%', height: '100%', display: 'block' }} />
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.multi_trend,
  component: MultiTrendWidget,
})

export default MultiTrendWidget
