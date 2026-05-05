import type { ComponentType } from 'react'
import type { SensorConfig, SensorData } from '../../types'

// ── Widget type identifiers ──

export type ScadaWidgetType =
  | 'gauge' | 'indicator' | 'bar' | 'status_light'
  | 'tank_level' | 'on_off' | 'label' | 'trend'

// ── Config field schema ──

export type ConfigFieldType = 'string' | 'number' | 'boolean' | 'select' | 'color'

export interface ConfigFieldDef {
  key: string
  label: string
  type: ConfigFieldType
  default: any
  options?: { value: string; label: string }[]
  min?: number
  max?: number
}

// ── Widget manifest ──

export type DataTypeCategory = 'analog' | 'digital' | 'any'

export interface WidgetManifest {
  type: ScadaWidgetType
  displayName: string
  icon: string
  defaultWidth: number
  defaultHeight: number
  minBindings: number
  maxBindings: number
  compatibleDataTypes: DataTypeCategory[]
  configSchema: ConfigFieldDef[]
}

// ── Widget instance (stored in state & API) ──

export interface WidgetInstance {
  id: string
  type: ScadaWidgetType
  sensorId: number | null
  x: number
  y: number
  width: number
  height: number
  config: Record<string, any>
}

// ── Connection instance ──

export type ConnectionType = 'power' | 'data' | 'flow'

export interface ConnectionInstance {
  id: string
  fromWidgetId: string
  toWidgetId: string
  connectionType: ConnectionType
  waypoints: [number, number][]
}

// ── Widget component props ──

export interface WidgetProps {
  instance: WidgetInstance
  sensor: SensorConfig | undefined
  data: SensorData | undefined
  editMode: boolean
  selected: boolean
}

// ── Registry entry ──

export interface WidgetRegistryEntry {
  manifest: WidgetManifest
  component: ComponentType<WidgetProps>
}

// ── Canvas state ──

export type ScadaTool = 'select' | 'connect' | 'label' | 'delete'

export interface ScadaCanvasState {
  editMode: boolean
  activeTool: ScadaTool
  selectedId: string | null
  connectFrom: string | null
  pendingSensor: { sensorId: number; x: number; y: number } | null
}

// ── Visual rule (kept from old engine) ──

export interface VisualRule {
  target: 'fill' | 'stroke' | 'opacity' | 'visible' | 'strokeWidth'
  expression: string
}
