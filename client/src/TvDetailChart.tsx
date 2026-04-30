import { useEffect, useRef } from 'react'
import {
  createChart,
  LineSeries,
  LineType,
  CrosshairMode,
  type IChartApi,
  type ISeriesApi,
  type LineData,
  type LineStyleOptions,
  type SeriesOptionsCommon,
  type Time,
} from 'lightweight-charts'
import { DataPoint } from './types'
import { toHexColor } from './styles'

export interface OverlaySeries {
  data: DataPoint[]
  color: string
  name: string
  unit: string
}

function toTvData(points: DataPoint[]): LineData<Time>[] {
  return points.map(p => ({
    time: p.ts / 1000 as Time,
    value: p.value,
  }))
}

interface Props {
  data: DataPoint[]
  color: string
  warn: number
  crit: number
  unit: string
  overlays?: OverlaySeries[]
}

export function TvDetailChart({
  data: rawData,
  color: rawColor,
  warn,
  crit,
  unit,
  overlays = [],
}: Props) {
  const containerRef = useRef<HTMLDivElement>(null)
  const chartRef = useRef<IChartApi | null>(null)
  const seriesRef = useRef<ISeriesApi<'Line'> | null>(null)
  const overlayRefs = useRef<ISeriesApi<'Line'>[]>([])
  const warnLineRef = useRef<ReturnType<ISeriesApi<'Line'>['createPriceLine']> | null>(null)
  const critLineRef = useRef<ReturnType<ISeriesApi<'Line'>['createPriceLine']> | null>(null)

  const color = toHexColor(rawColor)

  // Initialise chart once
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const chart = createChart(container, {
      layout: {
        background: { color: 'transparent' },
        textColor: '#d1d5db',
        attributionLogo: false,
      },
      grid: {
        vertLines: { color: '#1a1a1e' },
        horzLines: { color: '#1a1a1e' },
      },
      crosshair: {
        mode: CrosshairMode.Normal,
        vertLine: { color: 'rgba(255,255,255,0.12)', width: 1, style: 2 },
        horzLine: { color: 'rgba(255,255,255,0.12)', width: 1, style: 2 },
      },
      rightPriceScale: {
        borderColor: '#27272a',
        scaleMargins: { top: 0.1, bottom: 0.1 },
      },
      timeScale: {
        borderColor: '#27272a',
        timeVisible: true,
        secondsVisible: true,
        rightOffset: 12,
        barSpacing: 6,
        fixLeftEdge: false,
        fixRightEdge: false,
      },
      autoSize: true,
      handleScroll: { vertTouchDrag: false },
    })
    chartRef.current = chart

    const series = chart.addSeries(LineSeries, {
      color,
      lineWidth: 2,
      lineType: LineType.Curved,
      lastValueVisible: true,
      priceLineVisible: false,
      title: unit || undefined,
    } as LineStyleOptions & SeriesOptionsCommon)
    seriesRef.current = series

    // Threshold lines
    if (warn > 0) {
      warnLineRef.current = series.createPriceLine({
        price: warn,
        color: 'rgba(234,179,8,0.5)',
        lineWidth: 1,
        lineStyle: 2,
        title: 'WARN',
        axisLabelVisible: true,
      })
    }
    if (crit > 0) {
      critLineRef.current = series.createPriceLine({
        price: crit,
        color: 'rgba(239,68,68,0.5)',
        lineWidth: 1,
        lineStyle: 2,
        title: 'CRIT',
        axisLabelVisible: true,
      })
    }

    // Legend via tooltip formatter
    chart.applyOptions({
      localization: {
        priceFormatter: (p: number) => p.toFixed(2) + (unit ? ' ' + unit : ''),
      },
    })

    const handleResize = () => {
      chart.applyOptions({ width: container.clientWidth, height: container.clientHeight })
    }
    const ro = new ResizeObserver(handleResize)
    ro.observe(container)

    return () => {
      ro.disconnect()
      chart.remove()
      chartRef.current = null
      seriesRef.current = null
      overlayRefs.current = []
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Update primary data
  useEffect(() => {
    const series = seriesRef.current
    if (!series) return
    const tvData = toTvData(rawData)
    if (tvData.length === 0) return
    series.setData(tvData)

    const chart = chartRef.current!
    chart.timeScale().fitContent()
  }, [rawData])

  // Update overlays
  useEffect(() => {
    const chart = chartRef.current
    if (!chart) return
    // Remove old overlay series
    overlayRefs.current.forEach(s => chart.removeSeries(s))
    overlayRefs.current = []

    // Create new overlay series
    overlayRefs.current = overlays.map(ov => {
      const s = chart.addSeries(LineSeries, {
        color: toHexColor(ov.color),
        lineWidth: 1,
        lineType: LineType.Curved,
        lastValueVisible: false,
        priceLineVisible: false,
        title: ov.name,
      } as LineStyleOptions & SeriesOptionsCommon)
      s.setData(toTvData(ov.data))
      return s
    })
  }, [overlays])

  // Update threshold lines when warn/crit changes
  useEffect(() => {
    const series = seriesRef.current
    if (!series) return
    if (warnLineRef.current) series.removePriceLine(warnLineRef.current)
    if (critLineRef.current) series.removePriceLine(critLineRef.current)

    if (warn > 0) {
      warnLineRef.current = series.createPriceLine({
        price: warn,
        color: 'rgba(234,179,8,0.5)',
        lineWidth: 1,
        lineStyle: 2,
        title: 'WARN',
        axisLabelVisible: true,
      })
    }
    if (crit > 0) {
      critLineRef.current = series.createPriceLine({
        price: crit,
        color: 'rgba(239,68,68,0.5)',
        lineWidth: 1,
        lineStyle: 2,
        title: 'CRIT',
        axisLabelVisible: true,
      })
    }
  }, [warn, crit])

  return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />
}
