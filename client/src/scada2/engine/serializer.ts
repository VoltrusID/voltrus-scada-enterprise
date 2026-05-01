import { Canvas, Polyline, FabricObject } from 'fabric'
import type { ScadaEquipmentItem, ScadaConnectionItem } from '../../types'
import type { ScadaEquipmentProps, ScadaConnectionProps } from './types'
import { DEFAULT_SIZES } from '../defaults'
import { createEquipmentObject } from '../objects/ScadaEquipment'
import { createConnectionObject } from '../objects/ScadaConnection'

export async function loadFromAPI(
  canvas: Canvas,
  equipment: ScadaEquipmentItem[],
  connections: ScadaConnectionItem[],
): Promise<void> {
  const toRemove = canvas.getObjects().filter((o: FabricObject) => (o as any)._scada)
  toRemove.forEach((o: FabricObject) => canvas.remove(o))

  for (const eq of equipment) {
    const obj = await createEquipmentObject(eq.equipment_type, {
      id: eq.equipment_id,
      name: eq.equipment_name,
      x: eq.x,
      y: eq.y,
      width: eq.width,
      height: eq.height,
      rotation: eq.rotation,
      bindings: JSON.parse(eq.sensor_bindings || '[]').map((id: number) => ({ sensor_id: id })),
    })
    canvas.add(obj)
  }

  for (const conn of connections) {
    const points: [number, number][] = JSON.parse(conn.waypoints || '[]')
    const obj = createConnectionObject({
      id: conn.connection_id,
      from_id: conn.from_equipment,
      to_id: conn.to_equipment,
      connection_type: conn.connection_type,
      points,
      visual_rules: [],
      attached_labels: [],
    }, canvas)
    if (obj) canvas.add(obj)
  }

  canvas.renderAll()
}

export function exportToAPI(canvas: Canvas): {
  equipment: ScadaEquipmentItem[]
  connections: ScadaConnectionItem[]
} {
  const equipment: ScadaEquipmentItem[] = []
  const connections: ScadaConnectionItem[] = []

  canvas.getObjects().forEach((obj: FabricObject) => {
    const scada = (obj as any)._scada
    if (!scada) return

    if (scada.type === 'ScadaEquipment') {
      const props = scada as ScadaEquipmentProps
      equipment.push({
        equipment_id: props.id,
        equipment_type: props.equipment_type,
        equipment_name: props.name,
        x: Math.round((obj.left ?? 0) * 100) / 100,
        y: Math.round((obj.top ?? 0) * 100) / 100,
        width: Math.round((obj.width ?? DEFAULT_SIZES[props.equipment_type].width) * (obj.scaleX ?? 1)),
        height: Math.round((obj.height ?? DEFAULT_SIZES[props.equipment_type].height) * (obj.scaleY ?? 1)),
        rotation: Math.round((obj.angle ?? 0) * 10) / 10,
        sensor_bindings: JSON.stringify(props.bindings.map((b: { sensor_id: number }) => b.sensor_id)),
      })
    }

    if (scada.type === 'ScadaConnection') {
      const props = scada as ScadaConnectionProps
      const polyline = obj as Polyline
      const pts = polyline.points?.map((p: any) => [Math.round(p.x), Math.round(p.y)] as [number, number]) ?? []
      connections.push({
        connection_id: props.id,
        from_equipment: props.from_id,
        to_equipment: props.to_id,
        waypoints: JSON.stringify(pts),
        connection_type: props.connection_type,
      })
    }
  })

  return { equipment, connections }
}
