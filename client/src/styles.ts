export function status(val: number, warn: number, crit: number) {
  return val >= crit ? 'critical' : val >= warn ? 'warning' : 'normal';
}
export const stColor = (s: string) => s === 'critical' ? 'var(--crit)' : s === 'warning' ? 'var(--warn)' : s === 'offline' ? 'var(--muted)' : 'var(--ok)';
export function fmtTime(ms: number) {
  const s = ms / 1000 | 0;
  return [s / 3600 | 0, (s % 3600) / 60 | 0, s % 60].map(v => String(v).padStart(2, '0')).join(':');
}
export function fmtTs(ts: number) {
  return new Date(ts).toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
}
export function parseConfig(cfg: string): Record<string, any> { try { return JSON.parse(cfg) } catch { return {} } }

export function safeMinMax(arr: number[]): [number, number] {
  if (!arr.length) return [0, 0];
  let min = Infinity, max = -Infinity;
  for (const v of arr) { if (v < min) min = v; if (v > max) max = v; }
  return [min, max];
}

const _colorCtx = typeof document !== 'undefined' ? document.createElement('canvas').getContext('2d') : null;
export function toHexColor(color: string): string {
  if (!_colorCtx) return color;
  _colorCtx.fillStyle = '#000000';
  _colorCtx.fillStyle = color;
  return _colorCtx.fillStyle;
}

export function normalizeKeyEvent(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey && e.key.length > 1) parts.push('Shift');
  parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
  return parts.join('+');
}

export const inputStyle: React.CSSProperties = {
  background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6, padding: '6px 8px',
  color: 'var(--fg)', fontSize: 13, fontFamily: 'inherit', width: '100%', boxSizing: 'border-box'
};
export const monoInputStyle: React.CSSProperties = { ...inputStyle, fontFamily: 'JetBrains Mono,monospace' };
export const labelStyle: React.CSSProperties = { display: 'flex', flexDirection: 'column', gap: 4 };
export const labelTextStyle: React.CSSProperties = { fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' as const };
export const tabStyle = (active: boolean): React.CSSProperties => ({
  padding: '8px 16px', fontSize: 12, fontWeight: 600, cursor: 'pointer', border: 'none',
  borderBottom: active ? '2px solid var(--cyan)' : '2px solid transparent',
  color: active ? 'var(--cyan)' : 'var(--muted)', background: 'transparent', textTransform: 'uppercase' as const,
  letterSpacing: '.04em'
});
export const btnStyle: React.CSSProperties = {
  fontFamily: 'Inter,sans-serif', fontSize: 13, fontWeight: 500, padding: '7px 14px', borderRadius: 6,
  border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer',
  display: 'flex', alignItems: 'center', gap: 6
};
