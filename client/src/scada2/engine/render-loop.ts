import { Canvas, FabricObject, Group, FabricText } from 'fabric'
import { evalExpression } from './expression'
import { resolveContext, resolveTemplate } from './bindings'
import type { VisualRule } from './types'
import type { SensorConfig, SensorData } from '../../types'

export function startRenderLoop(
  canvas: Canvas,
  getState: () => { currentValues: Record<string, SensorData> },
  sensors: SensorConfig[],
): () => void {
  let rafId: number | null = null
  let lastValues: Record<string, SensorData> = {}

  function tick() {
    const { currentValues } = getState()
    const changed = currentValues !== lastValues
    lastValues = currentValues

    if (changed) {
      canvas.getObjects().forEach((obj: FabricObject) => {
        const scada = (obj as any)._scada
        if (!scada) return

        // Evaluate visual rules
        if (scada.visual_rules?.length > 0 && scada.bindings?.length > 0) {
          const contexts = resolveContext(scada.bindings, currentValues, sensors)
          for (const rule of scada.visual_rules as VisualRule[]) {
            for (const ctx of contexts) {
              const result = evalExpression(rule.expression, ctx)
              applyRule(obj, rule.target, result)
            }
          }
        }

        // Update attached labels
        if (scada.attached_labels?.length > 0 && (obj as any)._labelObjects) {
          for (const labelObj of (obj as any)._labelObjects as FabricText[]) {
            const tmpl = (labelObj as any)._template
            if (tmpl) {
              labelObj.set('text', resolveTemplate(tmpl, scada.bindings || [], currentValues, sensors))
            }
          }
        }

        // Update standalone labels
        if (scada.template && (obj as any)._scada?.type === 'ScadaLabel') {
          const text = resolveTemplate(scada.template, scada.bindings || [], currentValues, sensors)
          obj.set('text', text)
        }
      })
      canvas.renderAll()
    }

    rafId = requestAnimationFrame(tick)
  }

  rafId = requestAnimationFrame(tick)
  return () => { if (rafId !== null) cancelAnimationFrame(rafId) }
}

function applyRule(obj: FabricObject, target: string, value: string | number | boolean) {
  switch (target) {
    case 'fill':
      obj.set('fill', value as string)
      if ((obj as any)._objects) {
        (obj as Group).getObjects().forEach((child: FabricObject) => {
          if (child.type === 'path' || child.type === 'circle' || child.type === 'rect') {
            child.set('stroke', value as string)
          }
        })
      }
      break
    case 'stroke':
      obj.set('stroke', value as string)
      break
    case 'opacity':
      obj.set('opacity', value as number)
      break
    case 'visible':
      obj.set('visible', value as boolean)
      break
    case 'strokeWidth':
      obj.set('strokeWidth', value as number)
      break
  }
}
