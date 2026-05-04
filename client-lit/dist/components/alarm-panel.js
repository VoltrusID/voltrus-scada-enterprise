var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { api } from '../services/api.js';
import { fmtTs } from '../types.js';
const FILTER_TABS = ['all', 'active', 'acknowledged'];
let AlarmPanel = class AlarmPanel extends LitElement {
    constructor() {
        super(...arguments);
        this._filter = 'active';
        this._alarms = [];
        this._loading = false;
    }
    static { this.styles = css `
    :host {
      position: fixed; inset: 0; z-index: 1000;
      display: flex; justify-content: flex-end;
    }
    .backdrop { flex: 1; background: rgba(0,0,0,.5); }
    .panel {
      width: 360px; height: 100%; background: var(--bg);
      border-left: 1px solid var(--border); display: flex; flex-direction: column; overflow: hidden;
    }
    .header {
      padding: 16px 16px 12px; border-bottom: 1px solid var(--border);
      display: flex; justify-content: space-between; align-items: center;
    }
    .header span { font-size: 15px; font-weight: 700; color: var(--fg); }
    .header button {
      padding: 4px 12px; border-radius: 6px; border: 1px solid var(--warn);
      background: transparent; color: var(--warn); cursor: pointer; font-size: 11px; font-weight: 600;
    }
    .header .close {
      background: none; border: none; color: var(--muted); font-size: 18px; padding: 0 4px; line-height: 1;
    }
    .filters {
      display: flex; border-bottom: 1px solid var(--border);
    }
    .filters button {
      flex: 1; padding: 8px 0; font-size: 11px; font-weight: 600; text-transform: uppercase;
      letter-spacing: 0.04em; cursor: pointer; border: none; background: transparent;
      border-bottom: 2px solid transparent; color: var(--muted);
    }
    .filters button.active { border-bottom-color: var(--cyan); color: var(--cyan); }
    .list { flex: 1; overflow: auto; padding: 8px; }
    .alarm {
      padding: 12px; margin-bottom: 8px; border-radius: 8px;
      border: 1px solid var(--border); background: var(--card);
    }
    .alarm.active-critical { border-color: var(--crit); background: rgba(239,68,68,.15); }
    .alarm.active-warning { border-color: var(--warn); background: rgba(234,179,8,.15); }
    .alarm.resolved { opacity: 0.6; }
    .alarm-header {
      display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;
    }
    .alarm-title { font-size: 13px; font-weight: 600; color: var(--fg); margin-bottom: 4px; }
    .alarm-msg { font-size: 12px; color: var(--muted); margin-bottom: 8px; }
    .pulse { width: 8px; height: 8px; border-radius: 50%; background: var(--crit); animation: pulse 1.5s infinite; }
    .sev { padding: 2px 8px; border-radius: 4px; font-size: 10px; font-weight: 700; text-transform: uppercase; }
    .ack-btn {
      padding: 4px 12px; border-radius: 6px; font-size: 11px; font-weight: 600; cursor: pointer;
      background: transparent; border: 1px solid currentColor;
    }
    @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
  `; }
    connectedCallback() {
        super.connectedCallback();
        this._loadAlarms();
        const interval = setInterval(() => this._loadAlarms(), 5000);
        this._cleanup = () => clearInterval(interval);
    }
    disconnectedCallback() {
        super.disconnectedCallback();
        this._cleanup?.();
    }
    async _loadAlarms() {
        this._loading = true;
        try {
            this._alarms = await api(`/alarms?status=${this._filter}&limit=50`);
        }
        catch {
            this._alarms = [];
        }
        this._loading = false;
    }
    async _ack(id) {
        try {
            await fetch(`http://localhost:3100/api/v1/alarms/${id}/acknowledge`, { method: 'POST', credentials: 'include' });
            this._loadAlarms();
        }
        catch { /* ignore */ }
    }
    async _ackAll() {
        try {
            await fetch('http://localhost:3100/api/v1/alarms', { method: 'POST', credentials: 'include' });
            this._loadAlarms();
        }
        catch { /* ignore */ }
    }
    _setFilter(f) {
        this._filter = f;
        this._loadAlarms();
    }
    _close() {
        this.dispatchEvent(new CustomEvent('close'));
    }
    render() {
        return html `
      <div class="backdrop" @click="${this._close}"></div>
      <div class="panel">
        <div class="header">
          <span>Alarms</span>
          <div style="display:flex;gap:8px">
            <button @click="${this._ackAll}">Ack All</button>
            <button class="close" @click="${this._close}">&times;</button>
          </div>
        </div>
        <div class="filters">
          ${FILTER_TABS.map(tab => html `
            <button class="${this._filter === tab ? 'active' : ''}" @click="${() => this._setFilter(tab)}">${tab}</button>
          `)}
        </div>
        <div class="list">
          ${this._loading && this._alarms.length === 0 ? html `<div style="padding:20px;color:var(--muted);text-align:center">Loading...</div>` : ''}
          ${!this._loading && this._alarms.length === 0 ? html `<div style="padding:20px;color:var(--muted);text-align:center">No alarms</div>` : ''}
          ${this._alarms.map(alarm => {
            const isCritical = alarm.severity === 'critical';
            const isActive = !alarm.acknowledged_at;
            const isResolved = !!alarm.resolved_at;
            const sevColor = isCritical ? 'var(--crit)' : 'var(--warn)';
            const cls = isActive ? (isCritical ? 'active-critical' : 'active-warning') : (isResolved ? 'resolved' : '');
            return html `
              <div class="alarm ${cls}">
                <div class="alarm-header">
                  <div style="display:flex;align-items:center;gap:6px">
                    ${isActive ? html `<div class="pulse" style="background:${sevColor}"></div>` : ''}
                    <span class="sev" style="background:${sevColor}18;color:${sevColor}">${alarm.severity}</span>
                  </div>
                  <span style="font-size:11px;color:var(--muted)">${fmtTs(alarm.triggered_at * 1000)}</span>
                </div>
                <div class="alarm-title">${alarm.source_name}</div>
                <div class="alarm-msg">${alarm.message}</div>
                ${isActive ? html `
                  <button class="ack-btn" style="color:${sevColor};border-color:${sevColor}" @click="${() => this._ack(alarm.id)}">Acknowledge</button>
                ` : html `
                  <span style="font-size:11px;color:var(--ok)">&#10003; Acknowledged${alarm.acknowledged_by ? ` by ${alarm.acknowledged_by}` : ''}</span>
                `}
              </div>
            `;
        })}
        </div>
      </div>
    `;
    }
};
__decorate([
    state()
], AlarmPanel.prototype, "_filter", void 0);
__decorate([
    state()
], AlarmPanel.prototype, "_alarms", void 0);
__decorate([
    state()
], AlarmPanel.prototype, "_loading", void 0);
AlarmPanel = __decorate([
    customElement('vlt-alarm-panel')
], AlarmPanel);
export { AlarmPanel };
//# sourceMappingURL=alarm-panel.js.map