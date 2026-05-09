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
export { valveSymbol } from './valve'
export { pumpSymbol } from './pump'
export { tankSymbol } from './tank'
export { heatExchangerSymbol } from './heat_exchanger'
export { compressorSymbol } from './compressor'
export { pipeSymbol } from './pipe'
export { pipeElbowSymbol } from './pipe_elbow'
export { pipeTeeSymbol } from './pipe_tee'
export { sensorSymbol } from './sensor'
export { transmitterSymbol } from './transmitter'
export { controllerSymbol } from './controller'
export { indicatorSymbol } from './indicator'
export { analyzerSymbol } from './analyzer'
export { rectangleSymbol } from './rectangle'
export { circleSymbol } from './circle'
export { diamondSymbol } from './diamond'
export { arrowSymbol } from './arrow'
export { labelSymbol } from './label'

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
import { valveSymbol } from './valve'
import { pumpSymbol } from './pump'
import { tankSymbol } from './tank'
import { heatExchangerSymbol } from './heat_exchanger'
import { compressorSymbol } from './compressor'
import { pipeSymbol } from './pipe'
import { pipeElbowSymbol } from './pipe_elbow'
import { pipeTeeSymbol } from './pipe_tee'
import { sensorSymbol } from './sensor'
import { transmitterSymbol } from './transmitter'
import { controllerSymbol } from './controller'
import { indicatorSymbol } from './indicator'
import { analyzerSymbol } from './analyzer'
import { rectangleSymbol } from './rectangle'
import { circleSymbol } from './circle'
import { diamondSymbol } from './diamond'
import { arrowSymbol } from './arrow'
import { labelSymbol } from './label'

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
  valve: valveSymbol,
  pump: pumpSymbol,
  tank: tankSymbol,
  heat_exchanger: heatExchangerSymbol,
  compressor: compressorSymbol,
  pipe: pipeSymbol,
  pipe_elbow: pipeElbowSymbol,
  pipe_tee: pipeTeeSymbol,
  sensor: sensorSymbol,
  transmitter: transmitterSymbol,
  controller: controllerSymbol,
  indicator: indicatorSymbol,
  analyzer: analyzerSymbol,
  rectangle: rectangleSymbol,
  circle: circleSymbol,
  diamond: diamondSymbol,
  arrow: arrowSymbol,
  label: labelSymbol,
}
