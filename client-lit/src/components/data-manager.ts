import { LitElement, html, css } from 'lit';
import { customElement, state, property } from 'lit/decorators.js';
import { api, fetchOk } from '../services/api.js';
import type { SensorConfig } from '../types.js';
import { SOURCE_CATEGORY } from '../types.js';
import './edit-panel.js';

type Tab = 'sources' | 'modbus' | 'users';

interface ModbusDevice {
  id: string; name: string; ip: string; port: number; unit_id: number;
  interval_ms: number; endianness: string; register_count: number;
  enabled: boolean;
}

interface UserItem {
  id: number; username: string; role: string;
}

@customElement('vlt-data-manager')
export class DataManager extends LitElement {
  static styles = css`
    :host {
      position: fixed; inset: 0; background: var(--bg); z-index: 50;
      display: flex; flex-direction: column; padding: 24px;
    }
    .container { max-width: 1200px; width: 100%; margin: 0 auto; display: flex; flex-direction: column; height: 100%; }
    .header { display: flex; align-items: center; gap: 12px; margin-bottom: 0; flex-shrink: 0; }
    .header h1 { font-size: 20px; font-weight: 700; margin: 0; color: var(--fg); }
    .header button {
      display: flex; align-items: center; gap: 6px;
      padding: 7px 14px; border-radius: 6px; border: 1px solid var(--border);
      background: var(--card); color: var(--fg); cursor: pointer; font-family: inherit; font-size: 13px;
    }
    .tabs { display: flex; border-bottom: 1px solid var(--border); margin-bottom: 16px; }
    .tabs button {
      padding: 8px 16px; font-size: 12px; font-weight: 600; cursor: pointer; border: none;
      background: transparent; color: var(--muted); border-bottom: 2px solid transparent;
      text-transform: uppercase; letter-spacing: 0.04em;
    }
    .tabs button.active { border-bottom-color: var(--cyan); color: var(--cyan); background: rgba(34,197,94,.1); }
    .table-wrap { flex: 1; overflow: auto; border: 1px solid var(--border); border-radius: 8px; }
    table { width: 100%; border-collapse: collapse; font-size: 13px; }
    th { text-align: left; padding: 10px 12px; background: var(--card); font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.04em; border-bottom: 1px solid var(--border); position: sticky; top: 0; }
    td { padding: 10px 12px; border-bottom: 1px solid var(--border); color: var(--fg); }
    tr:hover td { background: rgba(255,255,255,0.02); }
    .tag { font-size: 9px; padding: 1px 5px; border-radius: 3px; font-weight: 700; letter-spacing: 0.03em; }
    .actions { display: flex; gap: 6px; }
    .actions button {
      padding: 3px 8px; border-radius: 4px; border: 1px solid var(--border);
      background: transparent; color: var(--fg); cursor: pointer; font-size: 11px; font-family: inherit;
    }
    .actions button.edit { color: var(--cyan); border-color: var(--cyan); }
    .actions button.del { color: var(--crit); border-color: var(--crit); }
    .add-btn {
      padding: 7px 14px; border-radius: 6px; border: 1px solid var(--green);
      background: transparent; color: var(--green); cursor: pointer; font-family: inherit; font-size: 13px;
    }
    .toolbar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-shrink: 0; }
    .toolbar span { color: var(--muted); font-size: 13px; }
    .toolbar input {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px;
      color: var(--fg); font-size: 13px; font-family: inherit; width: 200px;
    }
    .empty { padding: 40px; text-align: center; color: var(--muted); }
  `;

  @property({ type: Boolean }) isAdmin = false;

  @state() private _tab: Tab = 'sources';
  @state() private _sensors: SensorConfig[] = [];
  @state() private _editingId: number | null = null;
  @state() private _filter = '';
  @state() private _modbusDevices: ModbusDevice[] = [];
  @state() private _modbusFilter = '';
  @state() private _users: UserItem[] = [];
  @state() private _userFilter = '';

  connectedCallback() {
    super.connectedCallback();
    this._loadSensors();
    this._loadModbus();
    this._loadUsers();
    window.addEventListener('keydown', this._onKey);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    window.removeEventListener('keydown', this._onKey);
  }

  private _onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') this._close(); };

  private async _loadSensors() {
    try { this._sensors = await api<SensorConfig[]>('/data-sources'); } catch { this._sensors = []; }
  }

  private async _loadModbus() {
    try { this._modbusDevices = await api<ModbusDevice[]>('/modbus-devices'); } catch { this._modbusDevices = []; }
  }

  private async _loadUsers() {
    try { this._users = await api<UserItem[]>('/users'); } catch { this._users = []; }
  }

  private async _saveSensor(e: CustomEvent<SensorConfig>) {
    const s = e.detail;
    try {
      await fetchOk(`/api/v1/data-sources/${s.id}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(s) });
      this._editingId = null;
      this._loadSensors();
    } catch { /* ignore */ }
  }

  private async _deleteSensor(id: number) {
    if (!confirm('Delete this sensor?')) return;
    try {
      await fetchOk(`/api/v1/data-sources/${id}`, { method: 'DELETE' });
      this._editingId = null;
      this._loadSensors();
    } catch { /* ignore */ }
  }

  private async _duplicateSensor(s: SensorConfig) {
    try {
      await fetchOk('/api/v1/data-sources', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          key: s.key + '_copy', name: s.name + ' (Copy)', unit: s.unit,
          min_val: s.min_val, max_val: s.max_val, warn_val: s.warn_val, crit_val: s.crit_val,
          base_val: s.base_val, decimals: s.decimals, source_type: s.source_type,
          color: s.color, chart_type: s.chart_type, source_config: s.source_config,
        }),
      });
      this._loadSensors();
    } catch { /* ignore */ }
  }

  private async _deleteUser(id: number) {
    if (!confirm('Delete this user?')) return;
    try { await fetchOk(`/api/v1/users/${id}`, { method: 'DELETE' }); this._loadUsers(); } catch { /* ignore */ }
  }

  private async _deleteModbus(id: string) {
    if (!confirm('Delete this Modbus device?')) return;
    try { await fetchOk(`/api/v1/modbus-devices/${id}`, { method: 'DELETE' }); this._loadModbus(); } catch { /* ignore */ }
  }

  private _close() {
    this.dispatchEvent(new CustomEvent('close'));
  }

  private _sourcesTab() {
    const filtered = this._sensors.filter(s => s.name.toLowerCase().includes(this._filter.toLowerCase()) || s.key.toLowerCase().includes(this._filter.toLowerCase()));
    return html`
      <div class="toolbar">
        <span>${this._sensors.length} sources</span>
        <div style="display:flex;gap:8px">
          <input placeholder="Search..." .value="${this._filter}" @input="${(e: InputEvent) => this._filter = (e.target as HTMLInputElement).value}" />
          <button class="add-btn" @click="${() => this._duplicateSensor({ ...this._sensors[0], id: 0, key: 'new', name: 'New Sensor' } as SensorConfig)}">+ Create Source</button>
        </div>
      </div>
      <div class="table-wrap">
        <table>
          <thead><tr>
            <th>Name</th><th>Key</th><th>Type</th><th>Unit</th><th>Source</th><th>Chart</th><th>Actions</th>
          </tr></thead>
          <tbody>
            ${filtered.map(s => {
              const tag = SOURCE_CATEGORY[s.source_type];
              return html`
                <tr>
                  <td><strong>${s.name}</strong></td>
                  <td style="font-family:'JetBrains Mono',monospace;font-size:12px">${s.key}</td>
                  <td>${s.unit || '—'}</td>
                  <td><span class="tag" style="background:${tag?.color ?? 'var(--muted)'}18;color:${tag?.color ?? 'var(--muted)'}">${tag?.tag ?? s.source_type}</span></td>
                  <td><span class="tag" style="background:rgba(34,197,94,.1);color:var(--cyan)">${s.source_type}</span></td>
                  <td><span class="tag" style="background:rgba(139,92,246,.1);color:#a78bfa">${s.chart_type || 'sparkline'}</span></td>
                  <td>
                    <div class="actions">
                      <button class="edit" @click="${() => this._editingId = s.id}">Edit</button>
                      <button @click="${() => this._duplicateSensor(s)}">Dup</button>
                      <button class="del" @click="${() => this._deleteSensor(s.id)}">Del</button>
                    </div>
                  </td>
                </tr>
              `;
            })}
          </tbody>
        </table>
      </div>
    `;
  }

  private _modbusTab() {
    const filtered = this._modbusDevices.filter(d => (d.name || d.id).toLowerCase().includes(this._modbusFilter.toLowerCase()) || d.ip.toLowerCase().includes(this._modbusFilter.toLowerCase()));
    return html`
      <div class="toolbar">
        <span>${this._modbusDevices.length} devices</span>
        <input placeholder="Search..." .value="${this._modbusFilter}" @input="${(e: InputEvent) => this._modbusFilter = (e.target as HTMLInputElement).value}" />
      </div>
      <div class="table-wrap">
        <table>
          <thead><tr>
            <th>Name</th><th>IP</th><th>Port</th><th>Unit</th><th>Interval</th><th>Endian</th><th>Status</th><th>Actions</th>
          </tr></thead>
          <tbody>
            ${filtered.map(d => html`
              <tr>
                <td>${d.name || d.id}</td>
                <td style="font-family:'JetBrains Mono',monospace;font-size:12px">${d.ip}</td>
                <td>${d.port}</td>
                <td>${d.unit_id}</td>
                <td>${d.interval_ms}ms</td>
                <td><span class="tag" style="background:rgba(139,92,246,.1);color:#a78bfa">${d.endianness}</span></td>
                <td><span class="tag" style="background:${d.enabled ? 'rgba(34,197,94,.1)' : 'rgba(239,68,68,.1)'};color:${d.enabled ? 'var(--cyan)' : 'var(--crit)'}">${d.enabled ? 'enabled' : 'disabled'}</span></td>
                <td>
                  <div class="actions">
                    <button class="del" @click="${() => this._deleteModbus(d.id)}">Del</button>
                  </div>
                </td>
              </tr>
            `)}
          </tbody>
        </table>
      </div>
    `;
  }

  private _usersTab() {
    const filtered = this._users.filter(u => u.username.toLowerCase().includes(this._userFilter.toLowerCase()));
    return html`
      <div class="toolbar">
        <span>${this._users.length} users</span>
        <input placeholder="Search..." .value="${this._userFilter}" @input="${(e: InputEvent) => this._userFilter = (e.target as HTMLInputElement).value}" />
      </div>
      <div class="table-wrap">
        <table>
          <thead><tr><th>Username</th><th>Role</th><th>Actions</th></tr></thead>
          <tbody>
            ${filtered.map(u => html`
              <tr>
                <td><strong>${u.username}</strong></td>
                <td><span class="tag" style="background:${u.role === 'admin' ? 'rgba(168,85,247,.15)' : 'rgba(34,197,94,.1)'};color:${u.role === 'admin' ? '#a855f7' : 'var(--green)'}">${u.role}</span></td>
                <td>
                  <div class="actions">
                    <button class="del" @click="${() => this._deleteUser(u.id)}">Del</button>
                  </div>
                </td>
              </tr>
            `)}
          </tbody>
        </table>
      </div>
    `;
  }

  render() {
    const tabs: { key: Tab; label: string }[] = [
      { key: 'sources', label: 'Sources' },
      { key: 'modbus', label: 'Modbus' },
    ];
    if (this.isAdmin) tabs.push({ key: 'users', label: 'Users' });

    return html`
      <div class="container">
        <div class="header">
          <button @click="${this._close}">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
            Back
          </button>
          <h1>Data Manager</h1>
        </div>
        <div class="tabs">
          ${tabs.map(t => html`
            <button class="${this._tab === t.key ? 'active' : ''}" @click="${() => this._tab = t.key}">${t.label}</button>
          `)}
        </div>
        ${this._tab === 'sources' ? this._sourcesTab() : ''}
        ${this._tab === 'modbus' ? this._modbusTab() : ''}
        ${this._tab === 'users' ? this._usersTab() : ''}
      </div>
      ${this._editingId != null ? html`
        <vlt-edit-panel
          .sensor="${this._sensors.find(s => s.id === this._editingId)}"
          @save="${this._saveSensor}"
          @delete="${(e: CustomEvent<number>) => this._deleteSensor(e.detail)}"
          @close="${() => this._editingId = null}"
        ></vlt-edit-panel>
      ` : ''}
    `;
  }
}
