import type { WidgetManifest, ScadaWidgetType, WidgetRegistryEntry } from './types'

const manifests: Record<ScadaWidgetType, WidgetManifest> = {
  gauge: {
    type: 'gauge',
    displayName: 'Gauge',
    icon: '◬',
    defaultWidth: 180,
    defaultHeight: 180,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog'],
    configSchema: [
      { key: 'showLabels', label: 'Show Labels', type: 'boolean', default: true },
      { key: 'arcColor', label: 'Arc Color', type: 'color', default: 'var(--ok)' },
    ],
  },
  indicator: {
    type: 'indicator',
    displayName: 'Indicator',
    icon: '#',
    defaultWidth: 140,
    defaultHeight: 80,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog'],
    configSchema: [
      { key: 'fontSize', label: 'Font Size', type: 'number', default: 28, min: 12, max: 72 },
      { key: 'showUnit', label: 'Show Unit', type: 'boolean', default: true },
      { key: 'showStatus', label: 'Show Status', type: 'boolean', default: true },
    ],
  },
  bar: {
    type: 'bar',
    displayName: 'Bar',
    icon: '▬',
    defaultWidth: 200,
    defaultHeight: 60,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog'],
    configSchema: [
      { key: 'fillColor', label: 'Fill Color', type: 'color', default: 'var(--ok)' },
      { key: 'showValue', label: 'Show Value', type: 'boolean', default: true },
    ],
  },
  status_light: {
    type: 'status_light',
    displayName: 'Status Light',
    icon: '●',
    defaultWidth: 60,
    defaultHeight: 60,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog', 'digital'],
    configSchema: [
      { key: 'onThreshold', label: 'On Threshold', type: 'number', default: 0.5 },
    ],
  },
  tank_level: {
    type: 'tank_level',
    displayName: 'Tank Level',
    icon: '▤',
    defaultWidth: 100,
    defaultHeight: 160,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog'],
    configSchema: [
      { key: 'fillColor', label: 'Fill Color', type: 'color', default: '#3b82f6' },
      { key: 'showPercentage', label: 'Show %', type: 'boolean', default: true },
    ],
  },
  on_off: {
    type: 'on_off',
    displayName: 'On/Off',
    icon: '⏻',
    defaultWidth: 100,
    defaultHeight: 60,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog', 'digital'],
    configSchema: [
      { key: 'onThreshold', label: 'On Threshold', type: 'number', default: 0.5 },
      { key: 'onLabel', label: 'On Label', type: 'string', default: 'ON' },
      { key: 'offLabel', label: 'Off Label', type: 'string', default: 'OFF' },
    ],
  },
  label: {
    type: 'label',
    displayName: 'Label',
    icon: 'T',
    defaultWidth: 160,
    defaultHeight: 40,
    minBindings: 0,
    maxBindings: 99,
    compatibleDataTypes: ['any'],
    configSchema: [
      { key: 'text', label: 'Text', type: 'string', default: 'Label' },
      { key: 'fontSize', label: 'Font Size', type: 'number', default: 14, min: 8, max: 48 },
    ],
  },
  trend: {
    type: 'trend',
    displayName: 'Trend',
    icon: '📈',
    defaultWidth: 240,
    defaultHeight: 100,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog'],
    configSchema: [
      { key: 'lineColor', label: 'Line Color', type: 'color', default: 'var(--cyan)' },
      { key: 'pointsCount', label: 'Points', type: 'number', default: 60, min: 10, max: 300 },
    ],
  },
  big_number: {
    type: 'big_number',
    displayName: 'Big Number',
    icon: '⢀',
    defaultWidth: 180,
    defaultHeight: 120,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog', 'digital'],
    configSchema: [
      { key: 'fontSize', label: 'Font Size', type: 'number', default: 48, min: 20, max: 120 },
      { key: 'showUnit', label: 'Show Unit', type: 'boolean', default: true },
      { key: 'showName', label: 'Show Name', type: 'boolean', default: true },
      { key: 'decimals', label: 'Decimals', type: 'number', default: 1, min: 0, max: 6 },
    ],
  },
  data_table: {
    type: 'data_table',
    displayName: 'Data Table',
    icon: '⊞',
    defaultWidth: 320,
    defaultHeight: 200,
    minBindings: 0,
    maxBindings: 99,
    compatibleDataTypes: ['any'],
    configSchema: [
      { key: 'showStatus', label: 'Show Status', type: 'boolean', default: true },
      { key: 'compact', label: 'Compact', type: 'boolean', default: false },
    ],
  },
  pipe: {
    type: 'pipe',
    displayName: 'Pipe',
    icon: '━',
    defaultWidth: 200,
    defaultHeight: 30,
    minBindings: 0,
    maxBindings: 1,
    compatibleDataTypes: ['any'],
    configSchema: [
      { key: 'direction', label: 'Direction', type: 'select', default: 'horizontal', options: [
        { value: 'horizontal', label: 'Horizontal' },
        { value: 'vertical', label: 'Vertical' },
      ] },
      { key: 'pipeColor', label: 'Pipe Color', type: 'color', default: '#3b82f6' },
      { key: 'flowSpeed', label: 'Flow Speed', type: 'number', default: 2, min: 0, max: 10 },
      { key: 'thickness', label: 'Thickness', type: 'number', default: 8, min: 4, max: 24 },
    ],
  },
  slider: {
    type: 'slider',
    displayName: 'Slider',
    icon: '◱',
    defaultWidth: 220,
    defaultHeight: 80,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog'],
    configSchema: [
      { key: 'showValue', label: 'Show Value', type: 'boolean', default: true },
      { key: 'step', label: 'Step', type: 'number', default: 1, min: 0.01, max: 1000 },
      { key: 'sliderColor', label: 'Slider Color', type: 'color', default: 'var(--cyan)' },
    ],
  },
  number_input: {
    type: 'number_input',
    displayName: 'Number Input',
    icon: '⌨',
    defaultWidth: 180,
    defaultHeight: 80,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog'],
    configSchema: [
      { key: 'showButtons', label: 'Show +/- Buttons', type: 'boolean', default: true },
      { key: 'step', label: 'Step', type: 'number', default: 1, min: 0.01, max: 1000 },
    ],
  },
  toggle: {
    type: 'toggle',
    displayName: 'Toggle',
    icon: '⏻',
    defaultWidth: 120,
    defaultHeight: 80,
    minBindings: 1,
    maxBindings: 1,
    compatibleDataTypes: ['analog', 'digital'],
    configSchema: [
      { key: 'onThreshold', label: 'On Threshold', type: 'number', default: 0.5 },
      { key: 'onValue', label: 'Write On Value', type: 'number', default: 1 },
      { key: 'offValue', label: 'Write Off Value', type: 'number', default: 0 },
      { key: 'onLabel', label: 'On Label', type: 'string', default: 'ON' },
      { key: 'offLabel', label: 'Off Label', type: 'string', default: 'OFF' },
    ],
  },
}

export const WIDGET_MANIFESTS = manifests

export function getCompatibleWidgetTypes(isAnalog: boolean): ScadaWidgetType[] {
  const dt: 'analog' | 'digital' = isAnalog ? 'analog' : 'digital'
  return (Object.values(manifests) as WidgetManifest[]).filter(
    m => m.compatibleDataTypes.includes(dt) || m.compatibleDataTypes.includes('any'),
  ).map(m => m.type)
}

// Registry will be populated by each widget file
export const WIDGET_REGISTRY = new Map<ScadaWidgetType, WidgetRegistryEntry>()

export function registerWidget(entry: WidgetRegistryEntry) {
  WIDGET_REGISTRY.set(entry.manifest.type, entry)
}

export function getWidgetComponent(type: ScadaWidgetType) {
  return WIDGET_REGISTRY.get(type)?.component
}
