import type { ScadaEquipmentType } from '../../types'

export interface SymbolDef {
  svg: string
}

// Export each symbol
export { transformerSymbol } from './transformer'
export { meterSymbol } from './meter'
export { inverterSymbol } from './inverter'
export { solarPanelSymbol } from './solar_panel'
export { gatewaySymbol } from './gateway'
export { batterySymbol } from './battery'
export { circuitBreakerSymbol } from './circuit_breaker'
export { busbarSymbol } from './busbar'
export { isolatorSymbol } from './isolator'
export { earthSymbol } from './earth'
export { fuseSymbol } from './fuse'
export { capacitorSymbol } from './capacitor'
export { generatorSymbol } from './generator'
export { motorSymbol } from './motor'
export { surgeArresterSymbol } from './surge_arrester'
export { currentTransformerSymbol } from './current_transformer'
export { voltageTransformerSymbol } from './voltage_transformer'
export { contactorSymbol } from './contactor'
export { relaySymbol } from './relay'
export { loadSymbol } from './load'

// Import for registry
import { transformerSymbol } from './transformer'
import { meterSymbol } from './meter'
import { inverterSymbol } from './inverter'
import { solarPanelSymbol } from './solar_panel'
import { gatewaySymbol } from './gateway'
import { batterySymbol } from './battery'
import { circuitBreakerSymbol } from './circuit_breaker'
import { busbarSymbol } from './busbar'
import { isolatorSymbol } from './isolator'
import { earthSymbol } from './earth'
import { fuseSymbol } from './fuse'
import { capacitorSymbol } from './capacitor'
import { generatorSymbol } from './generator'
import { motorSymbol } from './motor'
import { surgeArresterSymbol } from './surge_arrester'
import { currentTransformerSymbol } from './current_transformer'
import { voltageTransformerSymbol } from './voltage_transformer'
import { contactorSymbol } from './contactor'
import { relaySymbol } from './relay'
import { loadSymbol } from './load'

export const SYMBOLS: Record<ScadaEquipmentType, SymbolDef> = {
  transformer: transformerSymbol,
  meter: meterSymbol,
  inverter: inverterSymbol,
  solar_panel: solarPanelSymbol,
  gateway: gatewaySymbol,
  battery: batterySymbol,
  circuit_breaker: circuitBreakerSymbol,
  busbar: busbarSymbol,
  isolator: isolatorSymbol,
  earth: earthSymbol,
  fuse: fuseSymbol,
  capacitor: capacitorSymbol,
  generator: generatorSymbol,
  motor: motorSymbol,
  surge_arrester: surgeArresterSymbol,
  current_transformer: currentTransformerSymbol,
  voltage_transformer: voltageTransformerSymbol,
  contactor: contactorSymbol,
  relay: relaySymbol,
  load: loadSymbol,
}
