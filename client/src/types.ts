export interface SensorConfig {
  id: number; key: string; name: string; unit: string; color: string;
  min_val: number; max_val: number; warn_val: number; crit_val: number;
  base_val: number; sort_order: number; decimals: number;
  source_type: string; source_config: string;
  device_id: string;
  equipment_id: string | null;
  chart_type: string;
}

export interface SensorData {
  id: number; key: string; value: number; warn: number; crit: number; source_type: string;
}

export interface DataPoint { value: number; ts: number }

export interface Screen { id: string; name: string; type: string; sort_order: number }

export interface LayoutItem { i: string; x: number; y: number; w: number; h: number }

export type ScreenLayouts = { lg: LayoutItem[]; md: LayoutItem[]; sm: LayoutItem[] }

export interface CanvasLayoutItem {
  data_source_id: number
  x: number; y: number; width: number; height: number
}

export const MAX_PTS = 300;
export const NORM_PTS = 60;
export const SOURCE_TYPES = ['random_walk', 'sine', 'step', 'sawtooth', 'constant', 'noise', 'api', 'modbus', 'none'] as const;

export const CHART_TYPES = ['sparkline', 'bignumber', 'gauge', 'image', 'text'] as const;

export type EquipmentType = 'solar_panel' | 'inverter' | 'gateway'

export interface TwinEquipment {
  id: string
  type: EquipmentType
  name: string
  position: [number, number, number]
  rotation: number
}

export interface TwinLayout {
  screen_id: string
  equipment_id: string
  equipment_type: EquipmentType
  position_x: number
  position_z: number
  rotation_y: number
}

export const EQUIPMENT_TYPES: { value: EquipmentType; label: string }[] = [
  { value: 'solar_panel', label: 'Solar Panel' },
  { value: 'inverter', label: 'Inverter' },
  { value: 'gateway', label: 'Gateway (PLC/HMI)' },
]

export interface Alarm {
  id: number
  source_key: string
  source_name: string
  severity: 'warning' | 'critical'
  value: number
  threshold: number
  message: string
  triggered_at: number
  resolved_at: number | null
  acknowledged_at: number | null
  acknowledged_by: string | null
}

export interface ModbusDevice {
  id: string
  name: string
  ip: string
  port: number
  unit_id: number
  interval_ms: number
  endianness: string
  enabled: boolean
  created_at: number
  updated_at: number
  register_count: number
}

export interface ModbusRegister {
  id: number
  device_id: string
  name: string
  address: number
  register_type: string
  scale: number
  sort_order: number
}
