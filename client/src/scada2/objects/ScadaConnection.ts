import { Canvas, Polyline, Point, FabricObject } from 'fabric'
import type { ScadaConnectionProps } from '../engine/types'
import { CONNECTION_COLORS } from '../defaults'

export function createConnectionObject(
  props: ScadaConnectionProps,
  canvas: Canvas,
): Polyline | null {
  const fromObj = canvas.getObjects().find((o: FabricObject) => (o as any)._scada?.id === props.from_id)
  const toObj = canvas.getObjects().find((o: FabricObject) => (o as any)._scada?.id === props.to_id)
  if (!fromObj || !toObj) return null

  const fromCenter = fromObj.getCenterPoint()
  const toCenter = toObj.getCenterPoint()

  const points = props.points.length > 0
    ? props.points.map(p => new Point(p[0], p[1]))
    : [fromCenter, toCenter]

  const color = CONNECTION_COLORS[props.connection_type] ?? CONNECTION_COLORS.power

  const polyline = new Polyline(points, {
    fill: 'transparent',
    stroke: color,
    strokeWidth: props.connection_type === 'power' ? 3 : 2,
    strokeDashArray: props.connection_type === 'data' ? [8, 4] : undefined,
    strokeLineCap: 'round',
    strokeLineJoin: 'round',
    selectable: true,
    hasControls: true,
    hasBorders: true,
    borderColor: '#06b6d4',
    evented: true,
    objectCaching: false,
  })

  ;(polyline as any)._scada = { ...props, type: 'ScadaConnection' }
  return polyline
}

export function updateConnectionPoints(
  polyline: Polyline,
  canvas: Canvas,
): void {
  const scada = (polyline as any)._scada
  if (!scada) return

  const fromObj = canvas.getObjects().find((o: FabricObject) => (o as any)._scada?.id === scada.from_id)
  const toObj = canvas.getObjects().find((o: FabricObject) => (o as any)._scada?.id === scada.to_id)
  if (!fromObj || !toObj) return

  if (!scada.points || scada.points.length === 0) {
    polyline.set({ points: [fromObj.getCenterPoint(), toObj.getCenterPoint()] } as any)
  }
}
