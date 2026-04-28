var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { status, stColor, SOURCE_CATEGORY } from '../types.js';
import './chart-sparkline.js';
let SensorCard = class SensorCard extends LitElement {
    constructor() {
        super(...arguments);
        this.selected = false;
    }
    static { this.styles = css `
    :host { display: block; height: 100%; }
    .card {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px;
      overflow: hidden; height: 100%; display: flex; flex-direction: column;
      box-sizing: border-box;
    }
    .card.selected { border-color: var(--cyan); box-shadow: 0 0 0 2px var(--cyan); }
    .header {
      display: flex; align-items: center; justify-content: space-between;
      padding: 14px 16px 10px;
    }
    .title-row {
      display: flex; align-items: center; gap: 8px;
      font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em;
    }
    .status-dot { width: 8px; height: 8px; border-radius: 50%; }
    .source-tag {
      font-size: 9px; padding: 1px 5px; border-radius: 3px; font-weight: 700;
      letter-spacing: 0.03em; text-transform: none;
    }
    .right {
      display: flex; align-items: center; gap: 6px;
    }
    .shortcut {
      font-family: 'JetBrains Mono', monospace; font-size: 10px; font-weight: 600;
      padding: 2px 6px; border-radius: 3px; background: var(--bg); border: 1px solid var(--border);
      color: var(--muted); line-height: 1;
    }
    .status-badge {
      font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em;
      padding: 2px 8px; border-radius: 4px;
    }
    .value {
      font-family: 'JetBrains Mono', monospace; font-size: 22px; font-weight: 700;
      padding: 0 16px;
    }
    .unit { font-size: 13px; color: var(--muted); font-weight: 400; margin-left: 4px; }
    .chart-wrap { flex: 1; min-height: 0; height: 140px; }
  `; }
    _click() {
        this.dispatchEvent(new CustomEvent('sensor-click', { detail: this.sensor.config.id, bubbles: true, composed: true }));
    }
    render() {
        const s = this.sensor.config;
        const cv = this.sensor.data;
        const isOffline = cv && (cv.value == null || Number.isNaN(cv.value));
        const st = isOffline ? 'offline' : cv ? status(cv.value, cv.warn, cv.crit) : 'normal';
        const ct = s.chart_type || 'sparkline';
        const hideStatus = ct === 'image' || ct === 'text';
        const sourceTag = SOURCE_CATEGORY[s.source_type]?.tag ?? s.source_type;
        const sourceTagColor = SOURCE_CATEGORY[s.source_type]?.color ?? 'var(--muted)';
        const valueText = isOffline ? 'OFFLINE' : cv ? cv.value.toFixed(s.decimals) : s.base_val.toFixed(s.decimals);
        const valueColor = st === 'normal' ? s.color : stColor(st);
        return html `
      <div class="card ${this.selected ? 'selected' : ''}" @click="${this._click}" style="cursor:pointer">
        ${!hideStatus ? html `
          <div class="header">
            <div class="title-row">
              <div class="status-dot" style="background:${stColor(st)}"></div>
              ${s.name}
              <span class="source-tag" style="background:${sourceTagColor}18;color:${sourceTagColor}">${sourceTag}</span>
            </div>
            <div class="right">
              <div class="status-badge" style="background:${st === 'critical' ? 'rgba(239,68,68,.15)' : st === 'warning' ? 'rgba(234,179,8,.15)' : st === 'offline' ? 'rgba(100,100,100,.15)' : 'rgba(34,197,94,.15)'};color:${stColor(st)}">${st}</div>
            </div>
          </div>
          <div class="value" style="color:${valueColor}">${valueText}<span class="unit">${s.unit}</span></div>
        ` : ''}
        <div class="chart-wrap">
          ${ct === 'sparkline'
            ? html `<vlt-chart-sparkline .data="${this.sensor.chartData}" .color="${s.color}" .warn="${s.warn_val}" .crit="${s.crit_val}"></vlt-chart-sparkline>`
            : html `<div style="display:flex;align-items:center;justify-content:center;height:100%;color:var(--muted);font-size:12px">${ct} chart</div>`}
        </div>
      </div>
    `;
    }
};
__decorate([
    property({ type: Object })
], SensorCard.prototype, "sensor", void 0);
__decorate([
    property({ type: Boolean })
], SensorCard.prototype, "selected", void 0);
SensorCard = __decorate([
    customElement('vlt-sensor-card')
], SensorCard);
export { SensorCard };
//# sourceMappingURL=sensor-card.js.map