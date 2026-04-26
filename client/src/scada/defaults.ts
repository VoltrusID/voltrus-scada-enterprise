import type { ScadaEquipmentType } from '../types'

export const DEFAULT_SIZES: Record<ScadaEquipmentType, { width: number; height: number }> = {
  solar_panel: { width: 100, height: 60 },
  inverter: { width: 80, height: 60 },
  gateway: { width: 80, height: 70 },
  transformer: { width: 90, height: 80 },
  battery: { width: 70, height: 80 },
  meter: { width: 60, height: 60 },
  circuit_breaker: { width: 60, height: 70 },
}

export const PORT_POSITIONS: Record<ScadaEquipmentType, { x: number; y: number; side: 'top' | 'bottom' | 'left' | 'right' }[]> = {
  solar_panel: [
    { x: 0.2, y: 1, side: 'bottom' },
    { x: 0.8, y: 1, side: 'bottom' },
  ],
  inverter: [
    { x: 0, y: 0.3, side: 'left' },
    { x: 1, y: 0.3, side: 'right' },
  ],
  gateway: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  transformer: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  battery: [
    { x: 0.3, y: 0, side: 'top' },
    { x: 0.7, y: 0, side: 'top' },
  ],
  meter: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  circuit_breaker: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
}

export const CONNECTION_COLORS: Record<string, string> = {
  power: '#f59e0b',
  data: '#3b82f6',
  flow: '#22c55e',
}
