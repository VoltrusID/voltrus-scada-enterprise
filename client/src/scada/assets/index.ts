export { SolarPanelSvg } from './SolarPanelSvg'
export { InverterSvg } from './InverterSvg'
export { GatewaySvg } from './GatewaySvg'
export { TransformerSvg } from './TransformerSvg'
export { BatterySvg } from './BatterySvg'
export { MeterSvg } from './MeterSvg'
export { CircuitBreakerSvg } from './CircuitBreakerSvg'
export { BusbarSvg } from './BusbarSvg'
export { IsolatorSvg } from './IsolatorSvg'
export { EarthSvg } from './EarthSvg'
export { FuseSvg } from './FuseSvg'
export { CapacitorSvg } from './CapacitorSvg'
export { GeneratorSvg } from './GeneratorSvg'
export { MotorSvg } from './MotorSvg'
export { SurgeArresterSvg } from './SurgeArresterSvg'
export { CurrentTransformerSvg } from './CurrentTransformerSvg'
export { VoltageTransformerSvg } from './VoltageTransformerSvg'
export { ContactorSvg } from './ContactorSvg'
export { RelaySvg } from './RelaySvg'
export { LoadSvg } from './LoadSvg'

import { SolarPanelSvg } from './SolarPanelSvg'
import { InverterSvg } from './InverterSvg'
import { GatewaySvg } from './GatewaySvg'
import { TransformerSvg } from './TransformerSvg'
import { BatterySvg } from './BatterySvg'
import { MeterSvg } from './MeterSvg'
import { CircuitBreakerSvg } from './CircuitBreakerSvg'
import { BusbarSvg } from './BusbarSvg'
import { IsolatorSvg } from './IsolatorSvg'
import { EarthSvg } from './EarthSvg'
import { FuseSvg } from './FuseSvg'
import { CapacitorSvg } from './CapacitorSvg'
import { GeneratorSvg } from './GeneratorSvg'
import { MotorSvg } from './MotorSvg'
import { SurgeArresterSvg } from './SurgeArresterSvg'
import { CurrentTransformerSvg } from './CurrentTransformerSvg'
import { VoltageTransformerSvg } from './VoltageTransformerSvg'
import { ContactorSvg } from './ContactorSvg'
import { RelaySvg } from './RelaySvg'
import { LoadSvg } from './LoadSvg'
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
  busbar: BusbarSvg,
  isolator: IsolatorSvg,
  earth: EarthSvg,
  fuse: FuseSvg,
  capacitor: CapacitorSvg,
  generator: GeneratorSvg,
  motor: MotorSvg,
  surge_arrester: SurgeArresterSvg,
  current_transformer: CurrentTransformerSvg,
  voltage_transformer: VoltageTransformerSvg,
  contactor: ContactorSvg,
  relay: RelaySvg,
  load: LoadSvg,
}
