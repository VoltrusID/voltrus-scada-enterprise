export { SolarPanelSvg } from './SolarPanelSvg'
export { InverterSvg } from './InverterSvg'
export { GatewaySvg } from './GatewaySvg'
export { TransformerSvg } from './TransformerSvg'
export { BatterySvg } from './BatterySvg'
export { MeterSvg } from './MeterSvg'
export { CircuitBreakerSvg } from './CircuitBreakerSvg'

import { SolarPanelSvg } from './SolarPanelSvg'
import { InverterSvg } from './InverterSvg'
import { GatewaySvg } from './GatewaySvg'
import { TransformerSvg } from './TransformerSvg'
import { BatterySvg } from './BatterySvg'
import { MeterSvg } from './MeterSvg'
import { CircuitBreakerSvg } from './CircuitBreakerSvg'
import type { ScadaEquipmentType } from '../../types'
import type { FC } from 'react'

export const ASSET_MAP: Record<ScadaEquipmentType, FC<{ statusColor?: string }>> = {
  solar_panel: SolarPanelSvg,
  inverter: InverterSvg,
  gateway: GatewaySvg,
  transformer: TransformerSvg,
  battery: BatterySvg,
  meter: MeterSvg,
  circuit_breaker: CircuitBreakerSvg,
}
