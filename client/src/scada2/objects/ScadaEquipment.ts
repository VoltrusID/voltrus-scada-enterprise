import { Group, FabricObject, loadSVGFromString } from 'fabric'
import type { ScadaEquipmentType } from '../../types'
import type { ScadaEquipmentProps } from '../engine/types'
import { SYMBOLS } from '../symbols'
import { DEFAULT_SIZES, PORT_POSITIONS } from '../defaults'

export async function createEquipmentObject(
  type: ScadaEquipmentType,
  overrides: Partial<ScadaEquipmentProps & { x: number; y: number; width: number; height: number; rotation: number }> = {},
): Promise<Group> {
  const symbol = SYMBOLS[type]
  const defaults = DEFAULT_SIZES[type]
  const ports = PORT_POSITIONS[type]

  const { objects: svgObjects } = await loadSVGFromString(symbol.svg)
  const validObjects = svgObjects.filter((o): o is FabricObject => o !== null)

  const group = new Group(validObjects, {
    left: overrides.x ?? 100,
    top: overrides.y ?? 100,
    angle: overrides.rotation ?? 0,
  })

  const targetW = overrides.width ?? defaults.width
  const targetH = overrides.height ?? defaults.height
  const groupW = group.width || defaults.width
  const groupH = group.height || defaults.height

  group.set({
    scaleX: targetW / groupW,
    scaleY: targetH / groupH,
    hasControls: true,
    hasBorders: true,
    borderColor: '#06b6d4',
    cornerColor: '#06b6d4',
    cornerSize: 8,
    transparentCorners: false,
  })

  const props: ScadaEquipmentProps & { type: string } = {
    type: 'ScadaEquipment',
    id: overrides.id ?? `scada_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
    equipment_type: type,
    name: overrides.name ?? type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
    bindings: overrides.bindings ?? [],
    visual_rules: [],
    attached_labels: [],
    ports,
  }
  ;(group as any)._scada = props

  return group
}
