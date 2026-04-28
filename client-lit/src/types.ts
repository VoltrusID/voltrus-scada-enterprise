export interface SensorConfig {
  id: number; key: string; name: string; unit: string; color: string;
  min_val: number; max_val: number; warn_val: number; crit_val: number;
  base_val: number; sort_order: number; decimals: number;
  source_type: string; source_config: string;
  device_id: string;
  equipment_id: string | null;
  chart_type: string;
}

export interface SensorData {
  id: number; key: string; value: number; warn: number; crit: number; source_type: string;
}

export interface DataPoint { value: number; ts: number }

export interface Screen { id: string; name: string; type: string; sort_order: number }

export interface LayoutItem { i: string; x: number; y: number; w: number; h: number }
export type ScreenLayouts = { lg: LayoutItem[]; md: LayoutItem[]; sm: LayoutItem[] }

export interface CanvasLayoutItem {
  data_source_id: number
  x: number; y: number; width: number; height: number
}

export interface BrandingSettings { title: string; subtitle: string; logo: string }

export const LIVE_PTS = 180;

export const SOURCE_CATEGORY: Record<string, { label: string; tag: string; color: string }> = {
  modbus:      { label: 'Modbus',        tag: 'REAL', color: 'var(--ok)' },
  random_walk: { label: 'Random Walk',   tag: 'SIM',  color: '#f59e0b' },
  sine:        { label: 'Sine Wave',     tag: 'SIM',  color: '#f59e0b' },
  step:        { label: 'Step / Spike',  tag: 'SIM',  color: '#f59e0b' },
  sawtooth:    { label: 'Sawtooth',      tag: 'SIM',  color: '#f59e0b' },
  constant:    { label: 'Constant',      tag: 'SIM',  color: '#f59e0b' },
  noise:       { label: 'Noise',         tag: 'SIM',  color: '#f59e0b' },
  api:         { label: 'API',           tag: 'API',  color: '#a78bfa' },
  none:        { label: 'None (static)', tag: 'NONE', color: 'var(--muted)' },
};

export const CHART_TYPES = ['sparkline', 'bignumber', 'gauge', 'image', 'text'] as const;
export const SIMULATE_TYPES = ['random_walk', 'sine', 'step', 'sawtooth', 'constant', 'noise'] as const;

export function status(val: number, warn: number, crit: number) {
  return val >= crit ? 'critical' : val >= warn ? 'warning' : 'normal';
}

export function stColor(s: string) {
  return s === 'critical' ? 'var(--crit)' : s === 'warning' ? 'var(--warn)' : s === 'offline' ? 'var(--muted)' : 'var(--ok)';
}

export function safeMinMax(arr: number[]): [number, number] {
  if (!arr.length) return [0, 0];
  let min = Infinity, max = -Infinity;
  for (const v of arr) { if (v < min) min = v; if (v > max) max = v; }
  return [min, max];
}

export function parseConfig(cfg: string): Record<string, any> {
  try { return JSON.parse(cfg); } catch { return {}; }
}

export function fmtTs(ts: number) {
  return new Date(ts).toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

const _colorCtx = typeof document !== 'undefined' ? document.createElement('canvas').getContext('2d') : null;
export function toHexColor(color: string): string {
  if (!_colorCtx) return color;
  _colorCtx.fillStyle = '#000000';
  _colorCtx.fillStyle = color;
  return _colorCtx.fillStyle;
}

export interface Alarm {
  id: number
  source_key: string
  source_name: string
  severity: 'warning' | 'critical'
  value: number
  threshold: number
  message: string
  triggered_at: number
  resolved_at: number | null
  acknowledged_at: number | null
  acknowledged_by: string | null
  cooldown_until: number | null
}
