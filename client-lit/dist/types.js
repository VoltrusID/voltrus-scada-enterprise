export const LIVE_PTS = 180;
export const SOURCE_CATEGORY = {
    modbus: { label: 'Modbus', tag: 'REAL', color: 'var(--ok)' },
    random_walk: { label: 'Random Walk', tag: 'SIM', color: '#f59e0b' },
    sine: { label: 'Sine Wave', tag: 'SIM', color: '#f59e0b' },
    step: { label: 'Step / Spike', tag: 'SIM', color: '#f59e0b' },
    sawtooth: { label: 'Sawtooth', tag: 'SIM', color: '#f59e0b' },
    constant: { label: 'Constant', tag: 'SIM', color: '#f59e0b' },
    noise: { label: 'Noise', tag: 'SIM', color: '#f59e0b' },
    api: { label: 'API', tag: 'API', color: '#a78bfa' },
    none: { label: 'None (static)', tag: 'NONE', color: 'var(--muted)' },
};
export const CHART_TYPES = ['sparkline', 'bignumber', 'gauge', 'image', 'text'];
export const SIMULATE_TYPES = ['random_walk', 'sine', 'step', 'sawtooth', 'constant', 'noise'];
export function status(val, warn, crit) {
    return val >= crit ? 'critical' : val >= warn ? 'warning' : 'normal';
}
export function stColor(s) {
    return s === 'critical' ? 'var(--crit)' : s === 'warning' ? 'var(--warn)' : s === 'offline' ? 'var(--muted)' : 'var(--ok)';
}
export function safeMinMax(arr) {
    if (!arr.length)
        return [0, 0];
    let min = Infinity, max = -Infinity;
    for (const v of arr) {
        if (v < min)
            min = v;
        if (v > max)
            max = v;
    }
    return [min, max];
}
export function parseConfig(cfg) {
    try {
        return JSON.parse(cfg);
    }
    catch {
        return {};
    }
}
export function fmtTs(ts) {
    return new Date(ts).toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
}
const _colorCtx = typeof document !== 'undefined' ? document.createElement('canvas').getContext('2d') : null;
export function toHexColor(color) {
    if (!_colorCtx)
        return color;
    _colorCtx.fillStyle = '#000000';
    _colorCtx.fillStyle = color;
    return _colorCtx.fillStyle;
}
//# sourceMappingURL=types.js.map