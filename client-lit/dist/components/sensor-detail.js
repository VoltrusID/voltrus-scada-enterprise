var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { api } from '../services/api.js';
import { dataManager } from '../services/api.js';
import { status, stColor, safeMinMax, LIVE_PTS } from '../types.js';
import './detail-chart.js';
const PRESETS = ['30m', '1h', '6h', '24h'];
let SensorDetail = class SensorDetail extends LitElement {
    constructor() {
        super(...arguments);
        this.allSensors = [];
        this._range = '30m';
        this._historyData = [];
        this._loading = false;
        this._overlayIds = [];
        this._overlayData = {};
        this._liveHistory = [];
        this._onKey = (e) => {
            if (e.key === 'Escape')
                this._back();
        };
        this._onLiveData = (e) => {
            const batch = e.detail;
            if (!batch?.length)
                return;
            const newLiveHistory = [...this._liveHistory];
            for (const item of batch) {
                if (item.id === this.sensor.id) {
                    this._liveValue = item;
                    if (item.value != null && !Number.isNaN(item.value)) {
                        newLiveHistory.push({ value: item.value, ts: Date.now() });
                    }
                }
                // Also update overlay data if this sensor is overlaid
                if (this._overlayIds.includes(item.id) && item.value != null && !Number.isNaN(item.value)) {
                    const arr = [...(this._overlayData[item.id] || [])];
                    arr.push({ value: item.value, ts: Date.now() });
                    this._overlayData = { ...this._overlayData, [item.id]: arr.length > LIVE_PTS ? arr.slice(-LIVE_PTS) : arr };
                }
            }
            this._liveHistory = newLiveHistory.length > LIVE_PTS ? newLiveHistory.slice(-LIVE_PTS) : newLiveHistory;
        };
    }
    static { this.styles = css `
    :host {
      position: fixed; inset: 0; background: var(--bg); z-index: 50;
      display: flex; flex-direction: column; padding: 16px;
    }
    .container {
      max-width: 1600px; width: 100%; margin: 0 auto;
      display: flex; flex-direction: column; height: 100%; gap: 12px;
    }
    .header {
      display: flex; align-items: center; justify-content: space-between; flex-shrink: 0;
    }
    .back-btn {
      display: flex; align-items: center; gap: 6px;
      padding: 7px 14px; border-radius: 6px; border: 1px solid var(--border);
      background: var(--card); color: var(--fg); cursor: pointer; font-family: inherit; font-size: 13px;
    }
    h1 { font-size: 20px; font-weight: 700; color: var(--fg); margin: 0; }
    .status-dot { width: 10px; height: 10px; border-radius: 50%; }
    .stats {
      display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; flex-shrink: 0;
    }
    .stat-card {
      background: var(--card); border: 1px solid var(--border); border-radius: 8px;
      padding: 12px 16px;
    }
    .stat-label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px; }
    .stat-value { font-family: 'JetBrains Mono', monospace; font-size: 18px; font-weight: 700; color: var(--fg); }
    .toolbar {
      display: flex; align-items: center; gap: 8px; flex-shrink: 0;
    }
    .toolbar button {
      padding: 6px 12px; border-radius: 6px; border: 1px solid var(--border);
      background: var(--card); color: var(--fg); cursor: pointer; font-family: inherit; font-size: 12px;
    }
    .toolbar button.active { background: rgba(34,197,94,.1); color: var(--cyan); border-color: var(--cyan); }
    .chart-wrap { flex: 1; min-height: 0; background: var(--card); border: 1px solid var(--border); border-radius: 10px; overflow: hidden; }
  `; }
    connectedCallback() {
        super.connectedCallback();
        this._liveValue = this.currentValue;
        this._loadHistory();
        window.addEventListener('keydown', this._onKey);
        dataManager.addEventListener('data', this._onLiveData);
        dataManager.connect();
    }
    disconnectedCallback() {
        super.disconnectedCallback();
        window.removeEventListener('keydown', this._onKey);
        dataManager.removeEventListener('data', this._onLiveData);
    }
    async _loadHistory() {
        this._loading = true;
        try {
            const result = await api(`/history?range=${this._range}`);
            if (!result.data) {
                this._historyData = [];
                this._overlayData = {};
                return;
            }
            const key = this.sensor.key;
            const s = result.data[key];
            if (s && Array.isArray(s.values)) {
                const pts = [];
                for (let i = 0; i < s.values.length; i++)
                    pts.push({ ts: s.start + i * s.interval, value: s.values[i] });
                this._historyData = pts;
            }
            else if (Array.isArray(s)) {
                this._historyData = s;
            }
            else {
                this._historyData = [];
            }
            // Load overlay histories
            await this._loadOverlayHistories(result.data);
        }
        catch {
            this._historyData = [];
            this._overlayData = {};
        }
        this._loading = false;
    }
    async _loadOverlayHistories(allData) {
        if (this._overlayIds.length === 0) {
            this._overlayData = {};
            return;
        }
        const result = {};
        for (const id of this._overlayIds) {
            const s = this.allSensors.find(x => x.id === id);
            if (!s)
                continue;
            if (allData && allData[s.key]) {
                const d = allData[s.key];
                if (d && Array.isArray(d.values)) {
                    const pts = [];
                    for (let i = 0; i < d.values.length; i++)
                        pts.push({ ts: d.start + i * d.interval, value: d.values[i] });
                    result[id] = pts;
                }
                else if (Array.isArray(d)) {
                    result[id] = d;
                }
            }
            else {
                // Fetch individually if not in batch
                try {
                    const res = await api(`/history?range=${this._range}`);
                    const d = res.data?.[s.key];
                    if (d && Array.isArray(d.values)) {
                        const pts = [];
                        for (let i = 0; i < d.values.length; i++)
                            pts.push({ ts: d.start + i * d.interval, value: d.values[i] });
                        result[id] = pts;
                    }
                    else if (Array.isArray(d)) {
                        result[id] = d;
                    }
                }
                catch { /* ignore */ }
            }
        }
        this._overlayData = result;
    }
    _back() {
        this.dispatchEvent(new CustomEvent('back'));
    }
    _setRange(r) {
        this._range = r;
        this._loadHistory();
    }
    async _toggleOverlay(id) {
        const next = this._overlayIds.includes(id)
            ? this._overlayIds.filter(x => x !== id)
            : [...this._overlayIds, id];
        this._overlayIds = next;
        await this._loadOverlayHistories();
    }
    _removeOverlay(id) {
        this._overlayIds = this._overlayIds.filter(x => x !== id);
        const copy = { ...this._overlayData };
        delete copy[id];
        this._overlayData = copy;
    }
    _overlays() {
        return this._overlayIds.map(id => {
            const s = this.allSensors.find(x => x.id === id);
            if (!s)
                return null;
            const data = this._overlayData[id] || [];
            return { data, color: s.color, name: s.name, unit: s.unit };
        }).filter((x) => x != null);
    }
    _chartData() {
        // Merge history with live data for seamless chart
        if (this._liveHistory.length > 0) {
            return [...this._historyData, ...this._liveHistory];
        }
        return this._historyData;
    }
    render() {
        const isOffline = this._liveValue && (this._liveValue.value == null || Number.isNaN(this._liveValue.value));
        const st = isOffline ? 'offline' : this._liveValue ? status(this._liveValue.value, this._liveValue.warn, this._liveValue.crit) : 'normal';
        const chartData = this._chartData();
        const safeValues = chartData.map(d => d.value).filter(v => v != null && !Number.isNaN(v));
        const [minV, maxV] = safeMinMax(safeValues);
        const avgV = safeValues.length ? safeValues.reduce((a, b) => a + b, 0) / safeValues.length : 0;
        const curV = isOffline ? null : this._liveValue ? this._liveValue.value : this.sensor.base_val;
        const sparklineSensors = this.allSensors.filter(s => s.id !== this.sensor.id && !this._overlayIds.includes(s.id));
        return html `
      <div class="container">
        <div class="header">
          <div style="display:flex;align-items:center;gap:12px">
            <button class="back-btn" @click="${this._back}">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
              Back
            </button>
            <h1>${this.sensor.name}</h1>
            <div class="status-dot" style="background:${stColor(st)}"></div>
          </div>
          <div style="font-family:'JetBrains Mono',monospace;font-size:28px;font-weight:700;color:${st === 'normal' ? this.sensor.color : stColor(st)}">
            ${curV != null ? curV.toFixed(this.sensor.decimals) : 'OFFLINE'}
            <span style="font-size:14px;color:var(--muted);font-weight:400">${this.sensor.unit}</span>
          </div>
        </div>

        <div class="stats">
          <div class="stat-card">
            <div class="stat-label">Minimum</div>
            <div class="stat-value" style="color:var(--cyan)">${minV.toFixed(this.sensor.decimals)}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Maximum</div>
            <div class="stat-value" style="color:var(--warn)">${maxV.toFixed(this.sensor.decimals)}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Average</div>
            <div class="stat-value">${avgV.toFixed(this.sensor.decimals)}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Points</div>
            <div class="stat-value">${safeValues.length}</div>
          </div>
        </div>

        <div class="toolbar">
          ${PRESETS.map(p => html `
            <button class="${this._range === p ? 'active' : ''}" @click="${() => this._setRange(p)}">${p}</button>
          `)}
          ${sparklineSensors.length > 0 ? html `
            <select @change="${(e) => this._toggleOverlay(Number(e.target.value))}" style="background:var(--bg);border:1px solid var(--border);color:var(--fg);border-radius:6px;padding:6px 10px;font-family:inherit;font-size:12px">
              <option value="">+ Overlay sensor...</option>
              ${sparklineSensors.map(s => html `<option value="${s.id}">${s.name}</option>`)}
            </select>
          ` : ''}
          ${this._overlayIds.length > 0 ? html `
            <div style="display:flex;gap:4px;flex-wrap:wrap">
              ${this._overlayIds.map(id => {
            const s = this.allSensors.find(x => x.id === id);
            return s ? html `<span style="font-size:11px;padding:2px 8px;border-radius:4px;background:${s.color}18;color:${s.color};display:flex;align-items:center;gap:4px;cursor:pointer" @click="${() => this._removeOverlay(id)}" title="Remove">${s.name} <span style="opacity:.6">×</span></span>` : '';
        })}
            </div>
          ` : ''}
        </div>

        <div class="chart-wrap">
          ${this._loading && this._historyData.length === 0
            ? html `<div style="display:flex;align-items:center;justify-content:center;height:100%;color:var(--muted)">Loading history...</div>`
            : html `<vlt-detail-chart .data="${chartData}" .color="${this.sensor.color}" .warn="${this.sensor.warn_val}" .crit="${this.sensor.crit_val}" .unit="${this.sensor.unit}" .overlays="${this._overlays()}"></vlt-detail-chart>`}
        </div>
      </div>
    `;
    }
};
__decorate([
    property({ type: Object })
], SensorDetail.prototype, "sensor", void 0);
__decorate([
    property({ type: Object })
], SensorDetail.prototype, "currentValue", void 0);
__decorate([
    property({ type: Array })
], SensorDetail.prototype, "allSensors", void 0);
__decorate([
    state()
], SensorDetail.prototype, "_range", void 0);
__decorate([
    state()
], SensorDetail.prototype, "_historyData", void 0);
__decorate([
    state()
], SensorDetail.prototype, "_loading", void 0);
__decorate([
    state()
], SensorDetail.prototype, "_overlayIds", void 0);
__decorate([
    state()
], SensorDetail.prototype, "_overlayData", void 0);
__decorate([
    state()
], SensorDetail.prototype, "_liveValue", void 0);
__decorate([
    state()
], SensorDetail.prototype, "_liveHistory", void 0);
SensorDetail = __decorate([
    customElement('vlt-sensor-detail')
], SensorDetail);
export { SensorDetail };
//# sourceMappingURL=sensor-detail.js.map