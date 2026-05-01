import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { authManager } from '../services/auth.js';
import { api, fetchOk } from '../services/api.js';
import { alarmAudio } from '../services/alarm-audio.js';
import { store } from '../services/store.js';
import type { Screen, SensorConfig, SensorData, BrandingSettings } from '../types.js';
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

type ViewName = 'grid' | 'canvas' | 'scada' | 'twin' | 'system';

@customElement('vlt-app-shell')
export class AppShell extends LitElement {
  static styles = css`
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
  `;

  @state() private _authed = false;
  @state() private _authLoading = true;
  @state() private _mustChangePassword = false;
  @state() private _view: ViewName = 'grid';
  @state() private _activeScreen = 'all';

  private _readUrlParams() {
    const p = new URLSearchParams(window.location.search);
    const v = p.get('view');
    if (v && ['grid', 'canvas', 'scada', 'twin', 'system'].includes(v)) this._view = v as ViewName;
    const s = p.get('screen');
    if (s) this._activeScreen = s;
  }

  private _syncUrl() {
    const p = new URLSearchParams(window.location.search);
    p.set('view', this._view);
    p.set('screen', this._activeScreen);
    window.history.replaceState(null, '', `${window.location.pathname}?${p}${window.location.hash}`);
  }
  @state() private _screens: Screen[] = [];
  @state() private _sseConnected = false;
  @state() private _showAlarmPanel = false;
  @state() private _selectedSensorId: number | null = null;
  @state() private _showSettings = false;
  @state() private _showDataManager = false;
  @state() private _allSensors: SensorConfig[] = [];
  @state() private _currentValues: Record<string, SensorData> = {};
  @state() private _alerts = 0;
  @state() private _branding: BrandingSettings = { title: 'Voltrus Lite', subtitle: 'Industrial monitoring', logo: '' };
  @state() private _version = '';
  @state() private _editMode = false;
  @state() private _dialog: null | { type: 'add-screen' | 'rename-screen' | 'delete-screen'; title: string; label: string; okLabel: string; initialValue: string } = null;
  @state() private _authRole = 'user';
  @state() private _systemInfo: { hostname?: string; os?: string; uptime_secs?: number } = {};

  connectedCallback() {
    super.connectedCallback();
    this._readUrlParams();
    authManager.addEventListener('change', this._onAuth as EventListener);
    authManager.check();
    this._loadScreens();
    this._loadAllSensors();
    this._loadBranding();
    this._loadShortcuts();
    this._pollAlarms();
    this._pollSystem();
    this._fetchVersion();
    document.addEventListener('keydown', this._onKeyDown);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    authManager.removeEventListener('change', this._onAuth as EventListener);
    document.removeEventListener('keydown', this._onKeyDown);
  }

  private _onAuth = () => {
    const s = authManager.state;
    this._authed = s.authed;
    this._authLoading = s.loading;
    this._authRole = s.role;
    this._mustChangePassword = s.mustChangePassword;
    if (s.authed) { this._loadScreens(); this._loadAllSensors(); this._loadBranding(); this._loadShortcuts(); }
  };

  private async _handleChangePassword(e: Event) {
    e.preventDefault();
    const form = e.target as HTMLFormElement;
    const pw = (form.querySelector('[name=pw]') as HTMLInputElement).value;
    const pw2 = (form.querySelector('[name=pw2]') as HTMLInputElement).value;
    if (pw !== pw2) { alert('Passwords do not match'); return; }
    if (pw.length < 8) { alert('Password must be at least 8 characters'); return; }
    const ok = await authManager.changePassword(pw);
    if (!ok) alert('Failed to change password');
  }

  private async _loadScreens() {
    try { this._screens = await api<Screen[]>('/screens'); } catch { this._screens = []; }
  }

  private async _loadAllSensors() {
    try { this._allSensors = await api<SensorConfig[]>('/data-sources'); } catch { this._allSensors = []; }
  }

  private async _loadBranding() {
    try { this._branding = await api<BrandingSettings>('/settings/branding'); store.setBranding(this._branding); } catch { /* ignore */ }
  }

  private async _loadShortcuts() {
    try {
      const shortcuts = await api<Record<number, string>>('/shortcuts');
      const nav = await api<Record<string, string>>('/nav-shortcuts');
      store.setShortcuts(shortcuts);
      store.setNavShortcuts(nav);
    } catch { /* ignore */ }
  }

  private async _pollAlarms() {
    const check = async () => {
      try {
        const alarms = await api<{ count: number }>('/alarms/active/count');
        const count = (alarms as any).count ?? 0;
        if (count > this._alerts && count > 0) alarmAudio.start();
        if (count === 0) alarmAudio.stop();
        this._alerts = count;
      } catch { /* ignore */ }
    };
    check();
    setInterval(check, 5000);
  }

  private async _pollSystem() {
    const check = async () => {
      try { this._systemInfo = await api('/system'); } catch { /* ignore */ }
    };
    check();
    setInterval(check, 30000);
  }

  private async _fetchVersion() {
    try {
      const res = await fetch(`${window.location.origin}/api/version`);
      const data = await res.json();
      this._version = data.version || '';
    } catch { /* ignore */ }
  }

  private _setScreen(id: string) {
    this._activeScreen = id;
    this._editMode = false;
    this._syncUrl();
  }

  private _setView(v: ViewName) {
    this._view = v;
    this._editMode = false;
    this._syncUrl();
  }

  private _handleLogout() {
    alarmAudio.stop();
    authManager.logout();
  }

  private _onSensorClick(e: CustomEvent<number>) {
    this._selectedSensorId = e.detail;
  }

  private _closeDetail() {
    this._selectedSensorId = null;
  }

  private _onKeyDown = (e: KeyboardEvent) => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || (e.target as HTMLElement)?.isContentEditable) return;
    const key = e.key.toLowerCase();
    const nav = store.navShortcuts;
    for (const [action, shortcut] of Object.entries(nav)) {
      if (shortcut.toLowerCase() === key) {
        e.preventDefault();
        if (['grid','canvas','scada','twin','system'].includes(action)) this._setView(action as ViewName);
        if (action === 'alarms') this._showAlarmPanel = true;
        if (action === 'settings') this._showSettings = true;
        if (action === 'data') this._showDataManager = true;
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

  private _openAddScreen() {
    this._dialog = { type: 'add-screen', title: 'Add Screen', label: 'Screen Name', okLabel: 'Create', initialValue: '' };
  }

  private _openRenameScreen() {
    const s = this._screens.find(x => x.id === this._activeScreen);
    if (!s) return;
    this._dialog = { type: 'rename-screen', title: 'Rename Screen', label: 'Screen Name', okLabel: 'Save', initialValue: s.name };
  }

  private _openDeleteScreen() {
    const s = this._screens.find(x => x.id === this._activeScreen);
    if (!s) return;
    this._dialog = { type: 'delete-screen', title: 'Delete Screen', label: '', okLabel: 'Delete', initialValue: '' };
  }

  private async _onDialogSubmit(e: CustomEvent<string>) {
    const d = this._dialog;
    this._dialog = null;
    if (!d) return;
    const value = e.detail;
    try {
      if (d.type === 'add-screen') {
        const res = await api<{ id: string }>('/screens', { method: 'POST', body: JSON.stringify({ name: value }) });
        this._activeScreen = res.id;
        await this._loadScreens();
      } else if (d.type === 'rename-screen') {
        await fetchOk(`/api/screens/${this._activeScreen}`, { method: 'PUT', body: JSON.stringify({ name: value }) });
        await this._loadScreens();
      }
    } catch { /* ignore */ }
  }

  private async _onDialogConfirm() {
    const d = this._dialog;
    this._dialog = null;
    if (d?.type === 'delete-screen') {
      try {
        await fetchOk(`/api/screens/${this._activeScreen}`, { method: 'DELETE' });
        this._activeScreen = 'all';
        await this._loadScreens();
      } catch { /* ignore */ }
    }
  }

  private _toggleEditMode() {
    this._editMode = !this._editMode;
  }

  private async _saveLayouts() {
    if (this._view === 'grid') {
      const grid = this.querySelector('vlt-grid-view') as any;
      if (grid?.saveLayouts) await grid.saveLayouts();
    } else if (this._view === 'canvas') {
      const canvas = this.querySelector('vlt-canvas-view') as any;
      if (canvas?.saveLayouts) await canvas.saveLayouts();
    }
    this._editMode = false;
  }

  render() {
    if (this._authLoading) {
      return html`<div class="loading">Loading...</div>`;
    }
    if (!this._authed) {
      return html`<vlt-login-screen></vlt-login-screen>`;
    }

    if (this._mustChangePassword) {
      return html`
        <div style="height:100vh;display:flex;align-items:center;justify-content:center;background:var(--bg);font-family:Inter,sans-serif">
          <form @submit="${this._handleChangePassword}" style="background:var(--card);border-radius:12px;padding:32px;width:360px;display:flex;flex-direction:column;gap:16px;border:1px solid var(--border)">
            <h2 style="margin:0;font-size:18px;color:var(--fg)">Change Your Password</h2>
            <p style="margin:0;font-size:13px;color:var(--muted)">You must set a new password before continuing.</p>
            <input name="pw" type="password" placeholder="New password (min 8 chars)" required minlength="8" style="padding:10px 12px;border-radius:8px;border:1px solid var(--border);background:var(--bg);color:var(--fg);font-size:14px" />
            <input name="pw2" type="password" placeholder="Confirm new password" required style="padding:10px 12px;border-radius:8px;border:1px solid var(--border);background:var(--bg);color:var(--fg);font-size:14px" />
            <button type="submit" style="padding:10px;border-radius:8px;border:none;background:var(--accent);color:#fff;font-weight:600;cursor:pointer;font-size:14px">Change Password</button>
          </form>
        </div>
      `;
    }

    const selectedSensor = this._allSensors.find(s => s.id === this._selectedSensorId);
    const activeScreenObj = this._screens.find(s => s.id === this._activeScreen);
    const canEditScreen = this._activeScreen !== 'all';

    return html`
      <div class="header">
        <div class="brand">
          ${this._branding.logo ? html`<img src="${this._branding.logo}" alt="${this._branding.title}" />` : html`<img src="favicon-32.png" alt="${this._branding.title}" />`}
          <h1>${this._branding.title} <span>${this._branding.subtitle}</span></h1>
        </div>
        <div class="toolbar">
          <button style="position:relative;border-color:${this._alerts > 0 ? 'var(--crit)' : 'var(--border)'};color:${this._alerts > 0 ? 'var(--crit)' : 'var(--fg)'}" @click="${() => this._showAlarmPanel = true}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
            ${this._alerts > 0 ? html`<span style="position:absolute;top:-4px;right:-4px;background:var(--crit);color:#fff;border-radius:50%;width:16px;height:16px;font-size:10px;display:flex;align-items:center;justify-content:center;font-weight:700">${this._alerts}</span>` : ''}
          </button>
          <button class="${this._view === 'grid' ? 'active' : ''}" @click="${() => this._setView('grid')}">Grid</button>
          <button class="${this._view === 'canvas' ? 'active' : ''}" @click="${() => this._setView('canvas')}">Canvas</button>
          <button class="${this._view === 'scada' ? 'active' : ''}" @click="${() => this._setView('scada')}">2D SCADA</button>
          <button class="${this._view === 'twin' ? 'active' : ''}" @click="${() => this._setView('twin')}">3D Twin</button>
          <button class="${this._view === 'system' ? 'active' : ''}" @click="${() => this._setView('system')}">System</button>
          ${(this._view === 'grid' || this._view === 'canvas') ? html`
            <button @click="${this._toggleEditMode}">${this._editMode ? 'Done' : 'Edit Layout'}</button>
            ${this._editMode ? html`<button @click="${this._saveLayouts}">Save Layout</button>` : ''}
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
        ${this._screens.map(s => html`
          <button class="${this._activeScreen === s.id ? 'active' : ''}" @click="${() => this._setScreen(s.id)}">
            ${s.name}
            ${this._activeScreen === s.id ? html`
              <span class="actions">
                <span title="Rename" @click="${(e: Event) => { e.stopPropagation(); this._openRenameScreen(); }}">✎</span>
                <span title="Delete" @click="${(e: Event) => { e.stopPropagation(); this._openDeleteScreen(); }}">✕</span>
              </span>
            ` : ''}
          </button>
        `)}
        <button class="screen-add" title="Add screen" @click="${this._openAddScreen}">+</button>
      </div>

      <main>
        ${this._view === 'grid' ? html`<vlt-grid-view .screenId="${this._activeScreen}" .editMode="${this._editMode}" @sensor-click="${this._onSensorClick}"></vlt-grid-view>` : ''}
        ${this._view === 'canvas' ? html`<vlt-canvas-view .screenId="${this._activeScreen}"></vlt-canvas-view>` : ''}
        ${this._view === 'scada' ? html`<vlt-scada-view></vlt-scada-view>` : ''}
        ${this._view === 'twin' ? html`<vlt-twin-view></vlt-twin-view>` : ''}
        ${this._view === 'system' ? html`<vlt-system-view></vlt-system-view>` : ''}
      </main>

      <div class="footer">
        <div class="left">
          <span><span class="dot" style="background:${this._sseConnected ? 'var(--ok)' : 'var(--warn)'}"></span>${this._sseConnected ? 'LIVE' : 'OFFLINE'}</span>
          <span>${this._systemInfo.hostname || '—'} · ${this._systemInfo.os || '—'}</span>
        </div>
        <div class="right">
          <span>${this._allSensors.length} sensors</span>
          <span>${this._screens.length} screens</span>
          ${this._version ? html`<span>v${this._version}</span>` : ''}
        </div>
      </div>

      ${this._showAlarmPanel ? html`<vlt-alarm-panel @close="${() => this._showAlarmPanel = false}"></vlt-alarm-panel>` : ''}
      ${selectedSensor ? html`<vlt-sensor-detail .sensor="${selectedSensor}" .currentValue="${this._currentValues[String(selectedSensor.id)]}" .allSensors="${this._allSensors}" @back="${this._closeDetail}"></vlt-sensor-detail>` : ''}
      ${this._showSettings ? html`<vlt-dashboard-settings @close="${() => this._showSettings = false}"></vlt-dashboard-settings>` : ''}
      ${this._showDataManager ? html`<vlt-data-manager .isAdmin="${this._authRole === 'admin'}" @close="${() => this._showDataManager = false}"></vlt-data-manager>` : ''}
      ${this._dialog?.type === 'add-screen' || this._dialog?.type === 'rename-screen' ? html`
        <vlt-prompt-dialog
          .title="${this._dialog.title}"
          .label="${this._dialog.label}"
          .okLabel="${this._dialog.okLabel}"
          .initialValue="${this._dialog.initialValue}"
          @submit="${this._onDialogSubmit}"
          @close="${() => this._dialog = null}"
        ></vlt-prompt-dialog>
      ` : ''}
      ${this._dialog?.type === 'delete-screen' ? html`
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
}
