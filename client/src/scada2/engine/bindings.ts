import type { SensorBinding, ExprContext } from './types'
import type { SensorConfig, SensorData } from '../../types'

export function resolveContext(
  bindings: SensorBinding[],
  currentValues: Record<string, SensorData>,
  sensors: SensorConfig[],
): ExprContext[] {
  return bindings.map(b => {
    const val = currentValues[String(b.sensor_id)]
    const cfg = sensors.find(s => s.id === b.sensor_id)
    const ctx: ExprContext = {
      value: val?.value ?? 0,
      warn: cfg?.warn_val ?? 0,
      crit: cfg?.crit_val ?? 0,
      min: cfg?.min_val ?? 0,
      max: cfg?.max_val ?? 0,
      status: val?.status ?? 'normal',
    }
    if (b.alias) {
      ctx[b.alias] = val?.value ?? 0
    }
    return ctx
  })
}

export function resolveTemplate(
  template: string,
  bindings: SensorBinding[],
  currentValues: Record<string, SensorData>,
  sensors: SensorConfig[],
): string {
  return template.replace(/\$\{([^}]+)\}/g, (_match, expr: string) => {
    const parts = expr.trim().split('.')
    if (parts.length === 2) {
      const [sensorRef, prop] = parts
      const binding = bindings.find(b => b.alias === sensorRef || String(b.sensor_id) === sensorRef)
      if (binding) {
        const cfg = sensors.find(s => s.id === binding.sensor_id)
        const val = currentValues[String(binding.sensor_id)]
        switch (prop) {
          case 'value': return val ? val.value.toFixed(cfg?.decimals ?? 1) : '--'
          case 'name': return cfg?.name ?? ''
          case 'unit': return cfg?.unit ?? ''
          case 'status': return val?.status ?? ''
        }
      }
    }
    // Simple ${value}, ${name}, ${unit} for single-binding labels
    if (bindings.length > 0) {
      const b = bindings[0]
      const cfg = sensors.find(s => s.id === b.sensor_id)
      const val = currentValues[String(b.sensor_id)]
      switch (expr.trim()) {
        case 'value': return val ? val.value.toFixed(cfg?.decimals ?? 1) : '--'
        case 'name': return cfg?.name ?? ''
        case 'unit': return cfg?.unit ?? ''
      }
    }
    return _match
  })
}
