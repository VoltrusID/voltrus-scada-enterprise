var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { authManager } from '../services/auth.js';
import { api, fetchOk } from '../services/api.js';
import { alarmAudio } from '../services/alarm-audio.js';
import { store } from '../services/store.js';
import '../components/login-screen.js';
import '../components/alarm-panel.js';
import '../components/sensor-detail.js';
import '../components/dashboard-settings.js';
import '../components/screen-dialog.js';
import '../views/grid-view.js';
import '../views/canvas-view.js';
import '../views/scada-view.js';
import '../views/twin-view.js';
import '../views/system-view.js';
import '../components/data-manager.js';
let AppShell = class AppShell extends LitElement {
    constructor() {
        super(...arguments);
        this._authed = false;
        this._authLoading = true;
        this._view = 'grid';
        this._activeScreen = 'all';
        this._screens = [];
        this._sseConnected = false;
        this._showAlarmPanel = false;
        this._selectedSensorId = null;
        this._showSettings = false;
        this._showDataManager = false;
        this._allSensors = [];
        this._currentValues = {};
        this._alerts = 0;
        this._branding = { title: 'Voltrus Lite', subtitle: 'Industrial monitoring', logo: '' };
        this._editMode = false;
        this._dialog = null;
        this._authRole = 'user';
        this._systemInfo = {};
        this._onAuth = () => {
            const s = authManager.state;
            this._authed = s.authed;
            this._authLoading = s.loading;
            this._authRole = s.role;
            if (s.authed) {
                this._loadScreens();
                this._loadAllSensors();
                this._loadBranding();
                this._loadShortcuts();
            }
        };
        this._onKeyDown = (e) => {
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target?.isContentEditable)
                return;
            const key = e.key.toLowerCase();
            const nav = store.navShortcuts;
            for (const [action, shortcut] of Object.entries(nav)) {
                if (shortcut.toLowerCase() === key) {
                    e.preventDefault();
                    if (['grid', 'canvas', 'scada', 'twin', 'system'].includes(action))
                        this._setView(action);
                    if (action === 'alarms')
                        this._showAlarmPanel = true;
                    if (action === 'settings')
                        this._showSettings = true;
                    if (action === 'data')
                        this._showDataManager = true;
                    return;
                }
            }
            const shortcuts = store.shortcuts;
            for (const [sid, shortcut] of Object.entries(shortcuts)) {
                if (shortcut.toLowerCase() === key) {
                    e.preventDefault();
                    this._selectedSensorId = Number(sid);
                    return;
                }
            }
        };
    }
    static { this.styles = css `
    :host { display: flex; flex-direction: column; height: 100vh; }
    .header {
      display: flex; align-items: center; justify-content: space-between;
      padding: 12px 20px; flex-shrink: 0; flex-wrap: wrap; gap: 16px;
    }
    .brand { display: flex; align-items: center; gap: 12px; }
    .brand img { width: 32px; height: 32px; border-radius: 6px; object-fit: contain; }
    .brand h1 { font-size: 20px; font-weight: 700; margin: 0; color: var(--fg); }
    .brand span { color: var(--muted); font-weight: 400; font-size: 14px; }
    .toolbar { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
    .toolbar button {
      font-family: Inter, sans-serif; font-size: 13px; font-weight: 500; padding: 7px 14px;
      border-radius: 6px; border: 1px solid var(--border); background: var(--card);
      color: var(--fg); cursor: pointer; display: flex; align-items: center; gap: 6px;
    }
    .toolbar button.active { background: rgba(34,197,94,.1); color: var(--cyan); border-color: var(--cyan); }
    .screens {
      display: flex; align-items: center; gap: 0; margin: 0 20px 20px;
      border-bottom: 1px solid var(--border);
    }
    .screens button {
      padding: 8px 16px; font-size: 12px; font-weight: 600; cursor: pointer; border: none;
      border-bottom: 2px solid transparent; color: var(--muted); background: transparent;
      text-transform: uppercase; letter-spacing: 0.04em;
      display: flex; align-items: center; gap: 6px;
    }
    .screens button.active { border-bottom-color: var(--cyan); color: var(--cyan); }
    .screens button .actions { display: none; gap: 4px; margin-left: 4px; }
    .screens button:hover .actions, .screens button.active .actions { display: flex; }
    .screens button .actions span { font-size: 10px; opacity: 0.6; cursor: pointer; }
    .screens button .actions span:hover { opacity: 1; color: var(--cyan); }
    .screen-add { padding: 8px 12px; font-size: 16px; font-weight: 700; cursor: pointer; border: none; background: transparent; color: var(--muted); }
    .screen-add:hover { color: var(--cyan); }
    main { flex: 1; overflow: hidden; position: relative; }
    main > * { position: absolute; inset: 0; }
    .loading {
      height: 100vh; display: flex; align-items: center; justify-content: center;
      background: var(--bg); color: var(--muted); font-family: Inter, sans-serif;
    }
    .sse-badge {
      font-family: 'JetBrains Mono', monospace; font-size: 11px;
      background: var(--card); border: 1px solid var(--border); padding: 4px 8px;
      border-radius: 4px; display: flex; align-items: center; gap: 4px;
    }
    .sse-dot { width: 6px; height: 6px; border-radius: 50%; }
    .footer {
      display: flex; align-items: center; justify-content: space-between;
      padding: 8px 20px; border-top: 1px solid var(--border);
      font-size: 11px; color: var(--muted); flex-shrink: 0;
      font-family: 'JetBrains Mono', monospace;
    }
    .footer .left { display: flex; align-items: center; gap: 12px; }
    .footer .right { display: flex; align-items: center; gap: 12px; }
    .footer .dot { width: 6px; height: 6px; border-radius: 50%; display: inline-block; margin-right: 4px; }
  `; }
    connectedCallback() {
        super.connectedCallback();
        authManager.addEventListener('change', this._onAuth);
        authManager.check();
        this._loadScreens();
        this._loadAllSensors();
        this._loadBranding();
        this._loadShortcuts();
        this._pollAlarms();
        this._pollSystem();
        document.addEventListener('keydown', this._onKeyDown);
    }
    disconnectedCallback() {
        super.disconnectedCallback();
        authManager.removeEventListener('change', this._onAuth);
        document.removeEventListener('keydown', this._onKeyDown);
    }
    async _loadScreens() {
        try {
            this._screens = await api('/screens');
        }
        catch {
            this._screens = [];
        }
    }
    async _loadAllSensors() {
        try {
            this._allSensors = await api('/data-sources');
        }
        catch {
            this._allSensors = [];
        }
    }
    async _loadBranding() {
        try {
            this._branding = await api('/settings/branding');
            store.setBranding(this._branding);
        }
        catch { /* ignore */ }
    }
    async _loadShortcuts() {
        try {
            const shortcuts = await api('/shortcuts');
            const nav = await api('/nav-shortcuts');
            store.setShortcuts(shortcuts);
            store.setNavShortcuts(nav);
        }
        catch { /* ignore */ }
    }
    async _pollAlarms() {
        const check = async () => {
            try {
                const alarms = await api('/alarms/active/count');
                const count = alarms.count ?? 0;
                if (count > this._alerts && count > 0)
                    alarmAudio.start();
                if (count === 0)
                    alarmAudio.stop();
                this._alerts = count;
            }
            catch { /* ignore */ }
        };
        check();
        setInterval(check, 5000);
    }
    async _pollSystem() {
        const check = async () => {
            try {
                this._systemInfo = await api('/system');
            }
            catch { /* ignore */ }
        };
        check();
        setInterval(check, 30000);
    }
    _setScreen(id) {
        this._activeScreen = id;
        this._editMode = false;
    }
    _setView(v) {
        this._view = v;
        this._editMode = false;
    }
    _handleLogout() {
        alarmAudio.stop();
        authManager.logout();
    }
    _onSensorClick(e) {
        this._selectedSensorId = e.detail;
    }
    _closeDetail() {
        this._selectedSensorId = null;
    }
    _openAddScreen() {
        this._dialog = { type: 'add-screen', title: 'Add Screen', label: 'Screen Name', okLabel: 'Create', initialValue: '' };
    }
    _openRenameScreen() {
        const s = this._screens.find(x => x.id === this._activeScreen);
        if (!s)
            return;
        this._dialog = { type: 'rename-screen', title: 'Rename Screen', label: 'Screen Name', okLabel: 'Save', initialValue: s.name };
    }
    _openDeleteScreen() {
        const s = this._screens.find(x => x.id === this._activeScreen);
        if (!s)
            return;
        this._dialog = { type: 'delete-screen', title: 'Delete Screen', label: '', okLabel: 'Delete', initialValue: '' };
    }
    async _onDialogSubmit(e) {
        const d = this._dialog;
        this._dialog = null;
        if (!d)
            return;
        const value = e.detail;
        try {
            if (d.type === 'add-screen') {
                const res = await api('/screens', { method: 'POST', body: JSON.stringify({ name: value }) });
                this._activeScreen = res.id;
                await this._loadScreens();
            }
            else if (d.type === 'rename-screen') {
                await fetchOk(`/api/screens/${this._activeScreen}`, { method: 'PUT', body: JSON.stringify({ name: value }) });
                await this._loadScreens();
            }
        }
        catch { /* ignore */ }
    }
    async _onDialogConfirm() {
        const d = this._dialog;
        this._dialog = null;
        if (d?.type === 'delete-screen') {
            try {
                await fetchOk(`/api/screens/${this._activeScreen}`, { method: 'DELETE' });
                this._activeScreen = 'all';
                await this._loadScreens();
            }
            catch { /* ignore */ }
        }
    }
    _toggleEditMode() {
        this._editMode = !this._editMode;
    }
    async _saveLayouts() {
        if (this._view === 'grid') {
            const grid = this.querySelector('vlt-grid-view');
            if (grid?.saveLayouts)
                await grid.saveLayouts();
        }
        else if (this._view === 'canvas') {
            const canvas = this.querySelector('vlt-canvas-view');
            if (canvas?.saveLayouts)
                await canvas.saveLayouts();
        }
        this._editMode = false;
    }
    render() {
        if (this._authLoading) {
            return html `<div class="loading">Loading...</div>`;
        }
        if (!this._authed) {
            return html `<vlt-login-screen></vlt-login-screen>`;
        }
        const selectedSensor = this._allSensors.find(s => s.id === this._selectedSensorId);
        const activeScreenObj = this._screens.find(s => s.id === this._activeScreen);
        const canEditScreen = this._activeScreen !== 'all';
        return html `
      <div class="header">
        <div class="brand">
          ${this._branding.logo ? html `<img src="${this._branding.logo}" alt="${this._branding.title}" />` : html `<img src="/favicon-32.png" alt="${this._branding.title}" />`}
          <h1>${this._branding.title} <span>${this._branding.subtitle}</span></h1>
        </div>
        <div class="toolbar">
          <button style="position:relative;border-color:${this._alerts > 0 ? 'var(--crit)' : 'var(--border)'};color:${this._alerts > 0 ? 'var(--crit)' : 'var(--fg)'}" @click="${() => this._showAlarmPanel = true}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
            ${this._alerts > 0 ? html `<span style="position:absolute;top:-4px;right:-4px;background:var(--crit);color:#fff;border-radius:50%;width:16px;height:16px;font-size:10px;display:flex;align-items:center;justify-content:center;font-weight:700">${this._alerts}</span>` : ''}
          </button>
          <button class="${this._view === 'grid' ? 'active' : ''}" @click="${() => this._setView('grid')}">Grid</button>
          <button class="${this._view === 'canvas' ? 'active' : ''}" @click="${() => this._setView('canvas')}">Canvas</button>
          <button class="${this._view === 'scada' ? 'active' : ''}" @click="${() => this._setView('scada')}">2D SCADA</button>
          <button class="${this._view === 'twin' ? 'active' : ''}" @click="${() => this._setView('twin')}">3D Twin</button>
          <button class="${this._view === 'system' ? 'active' : ''}" @click="${() => this._setView('system')}">System</button>
          ${(this._view === 'grid' || this._view === 'canvas') ? html `
            <button @click="${this._toggleEditMode}">${this._editMode ? 'Done' : 'Edit Layout'}</button>
            ${this._editMode ? html `<button @click="${this._saveLayouts}">Save Layout</button>` : ''}
          ` : ''}
          <div class="sse-badge">
            <span class="sse-dot" style="background:${this._sseConnected ? 'var(--ok)' : 'var(--warn)'}"></span>
            ${this._sseConnected ? 'LIVE' : 'SSE...'}
          </div>
          <button @click="${() => this._showSettings = true}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            Settings
          </button>
          <button @click="${() => this._showDataManager = true}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
            Data
          </button>
          <button @click="${this._handleLogout}">Logout</button>
        </div>
      </div>

      <div class="screens">
        <button class="${this._activeScreen === 'all' ? 'active' : ''}" @click="${() => this._setScreen('all')}">Overview</button>
        ${this._screens.map(s => html `
          <button class="${this._activeScreen === s.id ? 'active' : ''}" @click="${() => this._setScreen(s.id)}">
            ${s.name}
            ${this._activeScreen === s.id ? html `
              <span class="actions">
                <span title="Rename" @click="${(e) => { e.stopPropagation(); this._openRenameScreen(); }}">✎</span>
                <span title="Delete" @click="${(e) => { e.stopPropagation(); this._openDeleteScreen(); }}">✕</span>
              </span>
            ` : ''}
          </button>
        `)}
        <button class="screen-add" title="Add screen" @click="${this._openAddScreen}">+</button>
      </div>

      <main>
        ${this._view === 'grid' ? html `<vlt-grid-view .screenId="${this._activeScreen}" .editMode="${this._editMode}" @sensor-click="${this._onSensorClick}"></vlt-grid-view>` : ''}
        ${this._view === 'canvas' ? html `<vlt-canvas-view .screenId="${this._activeScreen}"></vlt-canvas-view>` : ''}
        ${this._view === 'scada' ? html `<vlt-scada-view></vlt-scada-view>` : ''}
        ${this._view === 'twin' ? html `<vlt-twin-view></vlt-twin-view>` : ''}
        ${this._view === 'system' ? html `<vlt-system-view></vlt-system-view>` : ''}
      </main>

      <div class="footer">
        <div class="left">
          <span><span class="dot" style="background:${this._sseConnected ? 'var(--ok)' : 'var(--warn)'}"></span>${this._sseConnected ? 'LIVE' : 'OFFLINE'}</span>
          <span>${this._systemInfo.hostname || '—'} · ${this._systemInfo.os || '—'}</span>
        </div>
        <div class="right">
          <span>${this._allSensors.length} sensors</span>
          <span>${this._screens.length} screens</span>
          <span>v1.0.0-lit</span>
        </div>
      </div>

      ${this._showAlarmPanel ? html `<vlt-alarm-panel @close="${() => this._showAlarmPanel = false}"></vlt-alarm-panel>` : ''}
      ${selectedSensor ? html `<vlt-sensor-detail .sensor="${selectedSensor}" .currentValue="${this._currentValues[String(selectedSensor.id)]}" .allSensors="${this._allSensors}" @back="${this._closeDetail}"></vlt-sensor-detail>` : ''}
      ${this._showSettings ? html `<vlt-dashboard-settings @close="${() => this._showSettings = false}"></vlt-dashboard-settings>` : ''}
      ${this._showDataManager ? html `<vlt-data-manager .isAdmin="${this._authRole === 'admin'}" @close="${() => this._showDataManager = false}"></vlt-data-manager>` : ''}
      ${this._dialog?.type === 'add-screen' || this._dialog?.type === 'rename-screen' ? html `
        <vlt-prompt-dialog
          .title="${this._dialog.title}"
          .label="${this._dialog.label}"
          .okLabel="${this._dialog.okLabel}"
          .initialValue="${this._dialog.initialValue}"
          @submit="${this._onDialogSubmit}"
          @close="${() => this._dialog = null}"
        ></vlt-prompt-dialog>
      ` : ''}
      ${this._dialog?.type === 'delete-screen' ? html `
        <vlt-confirm-dialog
          .title="${'Delete Screen'}"
          .message="${`Delete "${activeScreenObj?.name}"? Sensors will remain but be removed from this screen.`}"
          .confirmLabel="${'Delete'}"
          @confirm="${this._onDialogConfirm}"
          @close="${() => this._dialog = null}"
        ></vlt-confirm-dialog>
      ` : ''}
    `;
    }
};
__decorate([
    state()
], AppShell.prototype, "_authed", void 0);
__decorate([
    state()
], AppShell.prototype, "_authLoading", void 0);
__decorate([
    state()
], AppShell.prototype, "_view", void 0);
__decorate([
    state()
], AppShell.prototype, "_activeScreen", void 0);
__decorate([
    state()
], AppShell.prototype, "_screens", void 0);
__decorate([
    state()
], AppShell.prototype, "_sseConnected", void 0);
__decorate([
    state()
], AppShell.prototype, "_showAlarmPanel", void 0);
__decorate([
    state()
], AppShell.prototype, "_selectedSensorId", void 0);
__decorate([
    state()
], AppShell.prototype, "_showSettings", void 0);
__decorate([
    state()
], AppShell.prototype, "_showDataManager", void 0);
__decorate([
    state()
], AppShell.prototype, "_allSensors", void 0);
__decorate([
    state()
], AppShell.prototype, "_currentValues", void 0);
__decorate([
    state()
], AppShell.prototype, "_alerts", void 0);
__decorate([
    state()
], AppShell.prototype, "_branding", void 0);
__decorate([
    state()
], AppShell.prototype, "_editMode", void 0);
__decorate([
    state()
], AppShell.prototype, "_dialog", void 0);
__decorate([
    state()
], AppShell.prototype, "_authRole", void 0);
__decorate([
    state()
], AppShell.prototype, "_systemInfo", void 0);
AppShell = __decorate([
    customElement('vlt-app-shell')
], AppShell);
export { AppShell };
//# sourceMappingURL=app-shell.js.map