import { FabricText } from 'fabric'
import type { ScadaLabelProps } from '../engine/types'

export function createLabelObject(
  props: Partial<ScadaLabelProps & { x: number; y: number }>,
): FabricText {
  const text = new FabricText(props.template ?? 'Label', {
    left: props.x ?? 100,
    top: props.y ?? 100,
    fontSize: 14,
    fill: '#ccc',
    fontFamily: 'monospace',
    selectable: true,
    hasControls: true,
    hasBorders: true,
    borderColor: '#06b6d4',
    cornerColor: '#06b6d4',
    cornerSize: 8,
    editable: true,
  })

  ;(text as any)._scada = {
    type: 'ScadaLabel',
    id: props.id ?? `label_${Date.now()}`,
    template: props.template ?? 'Label',
    bindings: props.bindings ?? [],
    visual_rules: props.visual_rules ?? [],
  }

  return text
}
