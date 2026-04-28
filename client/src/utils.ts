import type { DataPoint } from './types'

/**
 * Expand compact history data from the server into a map of DataPoint arrays.
 * The server returns: { sensor_key: { values: [f64], start: unix_ms, interval: ms } }
 */
export function expandCompactData(raw: Record<string, any>): Record<string, DataPoint[]> {
  const out: Record<string, DataPoint[]> = {};
  for (const key of Object.keys(raw)) {
    const s = raw[key];
    if (s && Array.isArray(s.values)) {
      const pts: DataPoint[] = [];
      for (let i = 0; i < s.values.length; i++) {
        pts.push({ ts: s.start + i * s.interval, value: s.values[i] });
      }
      out[key] = pts;
    } else if (Array.isArray(s)) {
      out[key] = s; // legacy format fallback
    }
  }
  return out;
}
