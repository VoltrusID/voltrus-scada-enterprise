import type { ScadaEquipmentType, ScadaConnectionType } from '../../types'

export interface SensorBinding {
  sensor_id: number
  alias?: string
}

export interface VisualRule {
  target: 'fill' | 'stroke' | 'opacity' | 'visible' | 'text' | 'strokeWidth'
  expression: string
}

export interface AttachedLabel {
  template: string
  offset: [number, number]
  style?: { fontSize?: number; fill?: string; fontWeight?: string }
}

export interface PortDef {
  x: number
  y: number
  side: 'top' | 'bottom' | 'left' | 'right'
}

export interface ScadaEquipmentProps {
  id: string
  equipment_type: ScadaEquipmentType
  name: string
  bindings: SensorBinding[]
  visual_rules: VisualRule[]
  attached_labels: AttachedLabel[]
  ports: PortDef[]
}

export interface ScadaConnectionProps {
  id: string
  from_id: string
  to_id: string
  connection_type: ScadaConnectionType
  visual_rules: VisualRule[]
  attached_labels: AttachedLabel[]
}

export interface ScadaLabelProps {
  id: string
  template: string
  bindings: SensorBinding[]
  visual_rules: VisualRule[]
}

export interface ExprContext {
  value: number
  warn: number
  crit: number
  min: number
  max: number
  status: string
  [key: string]: number | string
}

export type ScadaTool = 'select' | 'connect' | 'label' | 'delete'

export interface ScadaCanvasState {
  editMode: boolean
  activeTool: ScadaTool
  selectedId: string | null
  pendingType: ScadaEquipmentType | null
  connectFrom: { eqId: string; port: PortDef } | null
}
