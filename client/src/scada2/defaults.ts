import type { ScadaEquipmentType } from '../types'
import type { PortDef } from './engine/types'

export const DEFAULT_SIZES: Record<ScadaEquipmentType, { width: number; height: number }> = {
  solar_panel: { width: 100, height: 60 },
  inverter: { width: 80, height: 60 },
  gateway: { width: 80, height: 70 },
  transformer: { width: 90, height: 80 },
  battery: { width: 70, height: 80 },
  meter: { width: 60, height: 60 },
  circuit_breaker: { width: 60, height: 70 },
  busbar: { width: 120, height: 30 },
  isolator: { width: 60, height: 60 },
  earth: { width: 40, height: 60 },
  fuse: { width: 60, height: 50 },
  capacitor: { width: 60, height: 60 },
  generator: { width: 70, height: 70 },
  motor: { width: 70, height: 70 },
  surge_arrester: { width: 50, height: 70 },
  current_transformer: { width: 60, height: 50 },
  voltage_transformer: { width: 60, height: 50 },
  contactor: { width: 60, height: 60 },
  relay: { width: 60, height: 60 },
  load: { width: 60, height: 60 },
}

export const PORT_POSITIONS: Record<ScadaEquipmentType, PortDef[]> = {
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
  busbar: [
    { x: 0.15, y: 0, side: 'top' },
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.85, y: 0, side: 'top' },
    { x: 0.15, y: 1, side: 'bottom' },
    { x: 0.5, y: 1, side: 'bottom' },
    { x: 0.85, y: 1, side: 'bottom' },
  ],
  isolator: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  earth: [
    { x: 0.5, y: 0, side: 'top' },
  ],
  fuse: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  capacitor: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  generator: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  motor: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  surge_arrester: [
    { x: 0.5, y: 0, side: 'top' },
  ],
  current_transformer: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  voltage_transformer: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  contactor: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  relay: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  load: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
}

export const CONNECTION_COLORS: Record<string, string> = {
  power: '#f59e0b',
  data: '#3b82f6',
  flow: '#22c55e',
}
