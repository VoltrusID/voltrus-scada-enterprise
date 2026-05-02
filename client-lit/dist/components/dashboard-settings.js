var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { alarmAudio } from '../services/alarm-audio.js';
import { api, fetchOk } from '../services/api.js';
import { store } from '../services/store.js';
let DashboardSettings = class DashboardSettings extends LitElement {
    constructor() {
        super(...arguments);
        this._tab = 'general';
        this._gap = 12;
        this._refresh = 1000;
        this._agg = 'sampling';
        this._muted = false;
        this._branding = { title: 'Voltrus Lite', subtitle: 'Industrial monitoring', logo: '' };
        this._sensors = [];
        this._shortcuts = {};
        this._navShortcuts = {};
        this._saving = false;
        this._navActions = [
            { key: 'grid', label: 'Grid View' },
            { key: 'canvas', label: 'Canvas View' },
            { key: 'scada', label: '2D SCADA' },
            { key: 'twin', label: '3D Twin' },
            { key: 'system', label: 'System' },
            { key: 'alarms', label: 'Alarms' },
            { key: 'settings', label: 'Settings' },
            { key: 'data', label: 'Data Manager' },
        ];
    }
    static { this.styles = css `
    :host {
      position: fixed; inset: 0; background: rgba(0,0,0,.6);
      display: flex; align-items: center; justify-content: center; z-index: 100;
    }
    .modal {
      background: var(--card); border: 1px solid var(--border); border-radius: 12px;
      padding: 0; min-width: 480px; max-width: 600px; max-height: 80vh; display: flex; flex-direction: column;
    }
    .header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; border-bottom: 1px solid var(--border); }
    .title { font-size: 15px; font-weight: 600; color: var(--fg); }
    .tabs { display: flex; gap: 0; border-bottom: 1px solid var(--border); }
    .tabs button {
      flex: 1; padding: 10px; font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em;
      background: transparent; border: none; border-bottom: 2px solid transparent; color: var(--muted); cursor: pointer; font-family: inherit;
    }
    .tabs button.active { border-bottom-color: var(--cyan); color: var(--cyan); }
    .body { padding: 20px; overflow-y: auto; flex: 1; }
    .row { display: flex; flex-direction: column; gap: 8px; margin-bottom: 14px; }
    .label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
    input[type="text"], input[type="url"] {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 6px 8px;
      color: var(--fg); font-size: 13px; font-family: inherit; outline: none; width: 100%; box-sizing: border-box;
    }
    input[type="range"] { width: 100%; }
    .value { font-size: 11px; font-family: 'JetBrains Mono', monospace; }
    .agg-btns { display: flex; gap: 4px; margin-top: 4px; }
    .agg-btns button {
      flex: 1; padding: 8px 6px; font-size: 11px; border-radius: 6px; border: 1px solid var(--border);
      background: transparent; color: var(--fg); cursor: pointer; font-family: inherit;
    }
    .agg-btns button.active { border-color: var(--warn); color: var(--warn); background: rgba(234,179,8,.08); font-weight: 600; }
    .desc { font-size: 11px; color: var(--muted); margin-top: 6px; line-height: 1.4; }
    .mute-btn {
      padding: 6px 14px; border-radius: 6px; border: 1px solid var(--border);
      background: transparent; color: var(--fg); cursor: pointer; font-family: inherit; font-size: 13px;
      display: flex; align-items: center; gap: 6px; width: fit-content;
    }
    .mute-btn.muted { border-color: var(--warn); background: rgba(234,179,8,.1); color: var(--warn); }
    .close-btn {
      margin: 0 20px 20px; padding: 8px 16px; border-radius: 6px; border: 1px solid var(--border);
      background: transparent; color: var(--fg); cursor: pointer; font-family: inherit; font-size: 13px;
    }
    .shortcut-row {
      display: grid; grid-template-columns: 1fr 80px; gap: 8px; align-items: center; margin-bottom: 8px;
    }
    .shortcut-row input {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 5px 8px;
      color: var(--fg); font-size: 13px; font-family: inherit; outline: none; text-align: center; text-transform: uppercase;
    }
    .shortcut-row span { font-size: 13px; color: var(--fg); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .save-btn {
      margin-top: 12px; padding: 8px 16px; border-radius: 6px; border: 1px solid var(--cyan);
      background: transparent; color: var(--cyan); cursor: pointer; font-family: inherit; font-size: 13px; font-weight: 600;
    }
    .section-title { font-size: 12px; font-weight: 600; color: var(--muted); margin: 16px 0 8px; text-transform: uppercase; letter-spacing: 0.04em; }
  `; }
    connectedCallback() {
        super.connectedCallback();
        this._muted = alarmAudio.isMuted;
        this._load();
    }
    async _load() {
        try {
            this._branding = await api('/settings/branding');
            this._sensors = await api('/data-sources');
            this._shortcuts = await api('/shortcuts');
            this._navShortcuts = await api('/nav-shortcuts');
        }
        catch { /* ignore */ }
    }
    _setAgg(a) {
        this._agg = a;
    }
    _toggleMute() {
        this._muted = alarmAudio.toggleMute();
    }
    async _saveBranding() {
        try {
            await fetchOk('/api/v1/settings/branding', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(this._branding) });
            store.setBranding(this._branding);
        }
        catch { /* ignore */ }
    }
    async _saveShortcuts() {
        this._saving = true;
        try {
            await fetchOk('/api/v1/shortcuts', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(this._shortcuts) });
            await fetchOk('/api/v1/nav-shortcuts', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(this._navShortcuts) });
            store.setShortcuts(this._shortcuts);
            store.setNavShortcuts(this._navShortcuts);
        }
        catch { /* ignore */ }
        this._saving = false;
    }
    _updateShortcut(id, val) {
        const v = val.trim().toLowerCase();
        if (!v) {
            const copy = { ...this._shortcuts };
            delete copy[id];
            this._shortcuts = copy;
        }
        else {
            this._shortcuts = { ...this._shortcuts, [id]: v };
        }
    }
    _updateNavShortcut(action, val) {
        const v = val.trim().toLowerCase();
        if (!v) {
            const copy = { ...this._navShortcuts };
            delete copy[action];
            this._navShortcuts = copy;
        }
        else {
            this._navShortcuts = { ...this._navShortcuts, [action]: v };
        }
    }
    _close() {
        this.dispatchEvent(new CustomEvent('close'));
    }
    render() {
        const aggOptions = [
            { value: 'sampling', label: 'Sampling', desc: 'Pick evenly-spaced points. Fastest, but skips data between intervals.' },
            { value: 'avg', label: 'Average', desc: 'Average all values within each interval bucket. Smooth curve, hides peaks.' },
            { value: 'minmax', label: 'Min/Max/Avg', desc: 'Shows average line with min/max band. Best overview — no data is lost.' },
        ];
        return html `
      <div @click="${this._close}">
        <div class="modal" @click="${(e) => e.stopPropagation()}">
          <div class="header">
            <div class="title">Dashboard Settings</div>
            <button style="background:transparent;border:none;color:var(--muted);cursor:pointer;font-size:18px;line-height:1" @click="${this._close}">×</button>
          </div>
          <div class="tabs">
            <button class="${this._tab === 'general' ? 'active' : ''}" @click="${() => this._tab = 'general'}">General</button>
            <button class="${this._tab === 'branding' ? 'active' : ''}" @click="${() => this._tab = 'branding'}">Branding</button>
            <button class="${this._tab === 'shortcuts' ? 'active' : ''}" @click="${() => this._tab = 'shortcuts'}">Shortcuts</button>
          </div>
          <div class="body">
            ${this._tab === 'general' ? html `
              <div class="row">
                <span class="label">Grid Gap (px)</span>
                <input type="range" min="4" max="24" .value="${this._gap}" @input="${(e) => this._gap = parseInt(e.target.value)}">
                <span class="value">${this._gap}px</span>
              </div>
              <div class="row">
                <span class="label">History Auto Mode</span>
                <div class="agg-btns">
                  ${aggOptions.map(opt => html `
                    <button class="${this._agg === opt.value ? 'active' : ''}" @click="${() => this._setAgg(opt.value)}">${opt.label}</button>
                  `)}
                </div>
                <div class="desc">${aggOptions.find(o => o.value === this._agg)?.desc}</div>
              </div>
              <div class="row">
                <span class="label">Mute Alarms</span>
                <button class="mute-btn ${this._muted ? 'muted' : ''}" @click="${this._toggleMute}">
                  ${this._muted ? 'Muted' : 'Active'}
                </button>
              </div>
            ` : ''}
            ${this._tab === 'branding' ? html `
              <div class="row">
                <span class="label">Title</span>
                <input type="text" .value="${this._branding.title}" @input="${(e) => this._branding = { ...this._branding, title: e.target.value }}" />
              </div>
              <div class="row">
                <span class="label">Subtitle</span>
                <input type="text" .value="${this._branding.subtitle}" @input="${(e) => this._branding = { ...this._branding, subtitle: e.target.value }}" />
              </div>
              <div class="row">
                <span class="label">Logo URL</span>
                <input type="url" .value="${this._branding.logo}" @input="${(e) => this._branding = { ...this._branding, logo: e.target.value }}" placeholder="https://..." />
              </div>
              <button class="save-btn" @click="${this._saveBranding}">Save Branding</button>
            ` : ''}
            ${this._tab === 'shortcuts' ? html `
              <div class="section-title">Navigation Shortcuts</div>
              ${this._navActions.map(a => html `
                <div class="shortcut-row">
                  <span title="${a.label}">${a.label}</span>
                  <input type="text" maxlength="1" .value="${this._navShortcuts[a.key] || ''}" @input="${(e) => this._updateNavShortcut(a.key, e.target.value)}" />
                </div>
              `)}
              <div class="section-title">Sensor Shortcuts</div>
              ${this._sensors.slice(0, 20).map(s => html `
                <div class="shortcut-row">
                  <span title="${s.name}">${s.name}</span>
                  <input type="text" maxlength="1" .value="${this._shortcuts[s.id] || ''}" @input="${(e) => this._updateShortcut(s.id, e.target.value)}" />
                </div>
              `)}
              ${this._sensors.length > 20 ? html `<div style="font-size:11px;color:var(--muted)">Showing first 20 sensors.</div>` : ''}
              <button class="save-btn" @click="${this._saveShortcuts}" .disabled="${this._saving}">${this._saving ? 'Saving...' : 'Save Shortcuts'}</button>
            ` : ''}
          </div>
          <button class="close-btn" @click="${this._close}">Close</button>
        </div>
      </div>
    `;
    }
};
__decorate([
    state()
], DashboardSettings.prototype, "_tab", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_gap", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_refresh", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_agg", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_muted", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_branding", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_sensors", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_shortcuts", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_navShortcuts", void 0);
__decorate([
    state()
], DashboardSettings.prototype, "_saving", void 0);
DashboardSettings = __decorate([
    customElement('vlt-dashboard-settings')
], DashboardSettings);
export { DashboardSettings };
//# sourceMappingURL=dashboard-settings.js.map