import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { api } from '../services/api.js';

interface TopologyDevice {
  id: string; name: string; protocol: string; address: string; enabled: boolean; tag_count: number; online: boolean;
}
interface TopologySummary {
  total_devices: number; total_tags: number; protocols: {protocol: string; device_count: number; tag_count: number}[]; online_count: number; offline_count: number;
}
interface TopologyConnection { from: string; to: string; protocol: string; address: string; }

interface TagXrefEntry {
  key: string; name: string; device_id: string; device_name: string; protocol: string;
  unit: string; has_alarm: boolean; alarm_severity?: string; warn_val: number; crit_val: number;
}

interface ScriptDoc {
  id: number; name: string; enabled: boolean; description: string; cpu_limit_ms: number; syntax_error?: string;
}
interface FlowDoc {
  id: string; name: string; enabled: boolean; trigger_description: string; node_descriptions: string[]; description: string;
}
interface AutomationSummary {
  total_scripts: number; enabled_scripts: number; total_flows: number; enabled_flows: number;
}

interface ConfigSnapshot {
  id: number; label?: string; trigger: string; created_at: number;
}

@customElement('vlt-docs-panel')
export class DocsPanel extends LitElement {
  static styles = css`
    :host { display: block; padding: 16px; overflow-y: auto; height: 100%; box-sizing: border-box; }
    h2 { font-size: 18px; font-weight: 600; color: var(--fg); margin: 0 0 16px; }
    h3 { font-size: 15px; font-weight: 600; color: var(--fg); margin: 16px 0 8px; }
    .tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 1px solid var(--border); }
    .tab { padding: 8px 16px; cursor: pointer; font-size: 13px; color: var(--muted); border-bottom: 2px solid transparent; }
    .tab.active { color: var(--fg); border-bottom-color: #f97316; }
    .card { background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 16px; margin-bottom: 12px; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 12px; margin-bottom: 16px; }
    .stat-card { background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 16px; text-align: center; }
    .stat-num { font-size: 28px; font-weight: 700; color: #f97316; }
    .stat-label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-top: 4px; }
    .badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; }
    .badge.on { background: #16a34a22; color: #16a34a; }
    .badge.off { background: #ef444422; color: #ef4444; }
    .badge.ok { background: #16a34a22; color: #16a34a; }
    .badge.warn { background: #f59e0b22; color: #f59e0b; }
    .table-wrap { overflow-x: auto; }
    table { width: 100%; border-collapse: collapse; font-size: 12px; }
    th { text-align: left; font-size: 11px; color: var(--muted); text-transform: uppercase; padding: 6px 8px; border-bottom: 1px solid var(--border); }
    td { padding: 6px 8px; border-bottom: 1px solid var(--border); color: var(--fg); }
    tr:hover { background: var(--border); }
    code { background: var(--border); padding: 1px 4px; border-radius: 3px; font-size: 11px; }
    .btn { padding: 8px 16px; border-radius: 6px; border: 1px solid var(--border); background: var(--card); color: var(--fg); cursor: pointer; font-size: 13px; }
    .btn:hover { background: var(--border); }
    .btn.primary { background: #f97316; color: #fff; border-color: #f97316; }
    .empty { color: var(--muted); font-size: 13px; padding: 20px; text-align: center; }
    .loading { display: flex; align-items: center; gap: 8px; color: var(--muted); font-size: 13px; }
    .actions { display: flex; gap: 8px; margin-top: 16px; }
    .protocol-row { display: flex; align-items: center; gap: 8px; padding: 6px 0; border-bottom: 1px solid var(--border); }
    .protocol-dot { width: 10px; height: 10px; border-radius: 50%; }
    select { padding: 6px 10px; border-radius: 6px; border: 1px solid var(--border); background: var(--card); color: var(--fg); font-size: 13px; }
    select:focus { outline: none; border-color: #f97316; }
    .error-msg { color: #ef4444; font-size: 13px; font-weight: 500; margin-top: 12px; }
    .device-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px; margin-top: 12px; }
    .device-card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 12px; }
    .device-name { font-size: 14px; font-weight: 600; color: var(--fg); }
    .device-meta { font-size: 12px; color: var(--muted); margin-top: 4px; }
  `;

  @state() private _tab = 'topology';
  @state() private _loading = false;
  @state() private _error = '';
  // Topology
  @state() private _devices: TopologyDevice[] = [];
  @state() private _connections: TopologyConnection[] = [];
  @state() private _summary: TopologySummary | null = null;
  // Tag Xref
  @state() private _tags: TagXrefEntry[] = [];
  @state() private _tagTotal = 0;
  // Automation
  @state() private _scripts: ScriptDoc[] = [];
  @state() private _flows: FlowDoc[] = [];
  @state() private _automationSummary: AutomationSummary | null = null;
  // Config diff
  @state() private _snapshots: ConfigSnapshot[] = [];
  @state() private _diffFrom = '';
  @state() private _diffTo = '';
  @state() private _diffResult: any = null;

  connectedCallback() {
    super.connectedCallback();
    this._loadTopology();
  }

  private async _loadTopology() {
    this._loading = true; this._error = '';
    try {
      const data = await api<any>('/docs/topology');
      this._devices = data.devices || [];
      this._connections = data.connections || [];
      this._summary = data.summary;
    } catch (e: any) { this._error = e.message; }
    this._loading = false;
  }

  private async _loadTagXref() {
    this._loading = true; this._error = '';
    try {
      const data = await api<any>('/docs/tag-xref');
      this._tags = data.tags || [];
      this._tagTotal = data.total_tags || 0;
    } catch (e: any) { this._error = e.message; }
    this._loading = false;
  }

  private async _loadAutomation() {
    this._loading = true; this._error = '';
    try {
      const data = await api<any>('/docs/automation');
      this._scripts = data.scripts || [];
      this._flows = data.flows || [];
      this._automationSummary = data.summary;
    } catch (e: any) { this._error = e.message; }
    this._loading = false;
  }

  private async _loadSnapshots() {
    this._loading = true; this._error = '';
    try {
      const data = await api<any>('/config/snapshots');
      this._snapshots = data || [];
    } catch (e: any) { this._error = e.message; }
    this._loading = false;
  }

  private async _loadDiff() {
    if (!this._diffFrom || !this._diffTo) return;
    this._loading = true; this._error = '';
    try {
      const data = await api<any>(`/docs/config-diff?from=${this._diffFrom}&to=${this._diffTo}`);
      this._diffResult = data;
    } catch (e: any) { this._error = e.message; }
    this._loading = false;
  }

  private async _exportDocs(format: string) {
    try {
      window.open(`/api/v1/docs/export?format=${format}`, '_blank');
    } catch (e: any) { this._error = e.message; }
  }

  private _switchTab(tab: string) {
    this._tab = tab;
    this._error = '';
    if (tab === 'topology' && !this._summary) this._loadTopology();
    if (tab === 'tags' && this._tags.length === 0) this._loadTagXref();
    if (tab === 'automation' && !this._automationSummary) this._loadAutomation();
    if (tab === 'diff' && this._snapshots.length === 0) this._loadSnapshots();
  }

  private _fmtTime(ts: number) {
    if (!ts) return '—';
    return new Date(ts * 1000).toLocaleString();
  }

  private _protocolColor(proto: string): string {
    const colors: Record<string, string> = {
      'Modbus TCP': '#f97316', 'OPC-UA': '#3b82f6', 'Siemens S7': '#8b5cf6',
      'EtherNet/IP': '#06b6d4', 'DNP3': '#22c55e', 'BACnet/IP': '#ec4899',
      'MQTT': '#eab308', 'Simulation': '#6b7280',
    };
    return colors[proto] || '#94a3b8';
  }

  render() {
    return html`
      <h2>System Documentation</h2>
      <p style="color:var(--muted);font-size:13px;margin-bottom:16px">
        Auto-generated system documentation for handover, audits, and compliance. Reverse-engineer your own system.
      </p>

      <div class="actions" style="margin-bottom:16px">
        <button class="btn primary" @click=${() => this._exportDocs('pdf')}>Export PDF</button>
        <button class="btn" @click=${() => this._exportDocs('html')}>Export HTML</button>
      </div>

      <div class="tabs">
        <div class="tab ${this._tab === 'topology' ? 'active' : ''}" @click=${() => this._switchTab('topology')}>Topology</div>
        <div class="tab ${this._tab === 'tags' ? 'active' : ''}" @click=${() => this._switchTab('tags')}>Tag Xref</div>
        <div class="tab ${this._tab === 'automation' ? 'active' : ''}" @click=${() => this._switchTab('automation')}>Automation</div>
        <div class="tab ${this._tab === 'diff' ? 'active' : ''}" @click=${() => this._switchTab('diff')}>Config Diff</div>
      </div>

      ${this._error ? html`<div class="error-msg">${this._error}</div>` : ''}
      ${this._loading ? html`<div class="loading">Loading...</div>` : ''}

      ${this._tab === 'topology' ? this._renderTopology() : ''}
      ${this._tab === 'tags' ? this._renderTags() : ''}
      ${this._tab === 'automation' ? this._renderAutomation() : ''}
      ${this._tab === 'diff' ? this._renderDiff() : ''}
    `;
  }

  private _renderTopology() {
    if (!this._summary) return html`<div class="empty">No topology data available.</div>`;
    const s = this._summary;
    return html`
      <div class="grid">
        <div class="stat-card"><div class="stat-num">${s.total_devices}</div><div class="stat-label">Devices</div></div>
        <div class="stat-card"><div class="stat-num">${s.total_tags}</div><div class="stat-label">Tags</div></div>
        <div class="stat-card"><div class="stat-num" style="color:#16a34a">${s.online_count}</div><div class="stat-label">Online</div></div>
        <div class="stat-card"><div class="stat-num" style="color:#ef4444">${s.offline_count}</div><div class="stat-label">Offline</div></div>
      </div>

      <h3>Protocols</h3>
      ${(s.protocols || []).map((p: any) => html`
        <div class="protocol-row">
          <div class="protocol-dot" style="background:${this._protocolColor(p.protocol)}"></div>
          <strong>${p.protocol}</strong>
          <span style="color:var(--muted)">${p.device_count} devices, ${p.tag_count} tags</span>
        </div>
      `)}

      <h3>Device Inventory</h3>
      <div class="device-grid">
        ${this._devices.map(d => html`
          <div class="device-card">
            <div style="display:flex;justify-content:space-between;align-items:center">
              <span class="device-name">${d.name}</span>
              <span class="badge ${d.online ? 'on' : 'off'}">${d.online ? 'Online' : 'Offline'}</span>
            </div>
            <div class="device-meta">
              <div class="protocol-dot" style="background:${this._protocolColor(d.protocol)};display:inline-block;width:8px;height:8px;border-radius:50%;margin-right:4px"></div>
              ${d.protocol} ${d.address ? html`· <code>${d.address}</code>` : ''}
            </div>
            <div class="device-meta">${d.tag_count} tags${!d.enabled ? ' · <span style="color:#ef4444">Disabled</span>' : ''}</div>
          </div>
        `)}
      </div>
    `;
  }

  private _renderTags() {
    if (this._tags.length === 0) return html`<div class="empty">No tag data. Click the tab to load.</div>`;
    return html`
      <div class="card">
        <strong>${this._tagTotal}</strong> tags across all devices
      </div>
      <div class="table-wrap">
        <table>
          <tr><th>Key</th><th>Name</th><th>Device</th><th>Protocol</th><th>Unit</th><th>Alarm</th><th>Warn</th><th>Crit</th></tr>
          ${this._tags.slice(0, 200).map(t => html`
            <tr>
              <td><code>${t.key}</code></td>
              <td>${t.name}</td>
              <td>${t.device_name}</td>
              <td>${t.protocol}</td>
              <td>${t.unit}</td>
              <td>${t.has_alarm
                ? html`<span class="badge ${t.alarm_severity === 'critical' ? 'off' : 'warn'}">${t.alarm_severity || 'yes'}</span>`
                : '—'}</td>
              <td>${t.warn_val}</td>
              <td>${t.crit_val}</td>
            </tr>
          `)}
        </table>
        ${this._tags.length > 200 ? html`<div class="empty">Showing first 200 of ${this._tagTotal} tags</div>` : ''}
      </div>
    `;
  }

  private _renderAutomation() {
    if (!this._automationSummary) return html`<div class="empty">No automation data. Click the tab to load.</div>`;
    const s = this._automationSummary;
    return html`
      <div class="grid">
        <div class="stat-card"><div class="stat-num">${s.total_scripts}</div><div class="stat-label">Scripts</div></div>
        <div class="stat-card"><div class="stat-num">${s.total_flows}</div><div class="stat-label">Flow Rules</div></div>
        <div class="stat-card"><div class="stat-num" style="color:#16a34a">${s.enabled_scripts + s.enabled_flows}</div><div class="stat-label">Enabled</div></div>
      </div>

      ${this._flows.length > 0 ? html`
        <h3>Flow Rules</h3>
        <div class="table-wrap">
          <table>
            <tr><th>Name</th><th>Trigger</th><th>Actions</th><th>Status</th></tr>
            ${this._flows.map(f => html`
              <tr>
                <td><strong>${f.name}</strong></td>
                <td>${f.trigger_description}</td>
                <td style="max-width:300px">${f.node_descriptions.join('; ') || '—'}</td>
                <td><span class="badge ${f.enabled ? 'on' : 'off'}">${f.enabled ? 'ON' : 'OFF'}</span></td>
              </tr>
            `)}
          </table>
        </div>
      ` : ''}

      ${this._scripts.length > 0 ? html`
        <h3>Scripts</h3>
        <div class="table-wrap">
          <table>
            <tr><th>Name</th><th>Description</th><th>Timeout</th><th>Status</th></tr>
            ${this._scripts.map(s => html`
              <tr>
                <td><strong>${s.name}</strong></td>
                <td style="max-width:400px">${s.description}</td>
                <td>${s.cpu_limit_ms}ms</td>
                <td>
                  ${s.syntax_error
                    ? html`<span class="badge off">Error</span>`
                    : html`<span class="badge ${s.enabled ? 'on' : 'off'}">${s.enabled ? 'ON' : 'OFF'}</span>`}
                </td>
              </tr>
            `)}
          </table>
        </div>
      ` : ''}
    `;
  }

  private _renderDiff() {
    return html`
      <div class="card">
        <h3 style="margin-top:0">Compare Config Versions</h3>
        <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap">
          <div>
            <div style="font-size:11px;color:var(--muted);margin-bottom:4px">From</div>
            <select .value=${this._diffFrom} @change=${(e: any) => { this._diffFrom = e.target.value; }}>
              <option value="">Select snapshot...</option>
              ${this._snapshots.map(s => html`
                <option value=${s.id}>#${s.id} ${s.label || s.trigger} (${this._fmtTime(s.created_at)})</option>
              `)}
            </select>
          </div>
          <div>
            <div style="font-size:11px;color:var(--muted);margin-bottom:4px">To</div>
            <select .value=${this._diffTo} @change=${(e: any) => { this._diffTo = e.target.value; }}>
              <option value="">Select snapshot...</option>
              ${this._snapshots.map(s => html`
                <option value=${s.id}>#${s.id} ${s.label || s.trigger} (${this._fmtTime(s.created_at)})</option>
              `)}
            </select>
          </div>
          <button class="btn primary" ?disabled=${!this._diffFrom || !this._diffTo} @click=${() => this._loadDiff()}>Compare</button>
        </div>
      </div>

      ${this._diffResult ? this._renderDiffResult() : ''}
    `;
  }

  private _renderDiffResult() {
    const diff = this._diffResult?.diff;
    if (!diff || typeof diff !== 'object') return html`<div class="empty">No differences found.</div>`;

    const tables = Object.entries(diff);
    if (tables.length === 0) return html`<div class="empty">Configs are identical — no differences found.</div>`;

    return html`
      <div class="card" style="margin-top:12px">
        <div style="font-size:12px;color:var(--muted)">
          Comparing #${this._diffResult.from?.id} → #${this._diffResult.to?.id}
        </div>
      </div>

      ${tables.map(([table, data]: [string, any]) => html`
        <div class="card">
          <h3 style="margin-top:0">${table}</h3>
          ${(data.added?.length > 0) ? html`
            <div style="margin-bottom:8px"><span class="badge ok">+${data.added.length} added</span></div>
            <div class="table-wrap"><table>
              ${data.added.map((r: any) => html`<tr><td colspan="4"><code>${JSON.stringify(r).substring(0, 200)}</code></td></tr>`)}
            </table></div>
          ` : ''}
          ${(data.removed?.length > 0) ? html`
            <div style="margin-bottom:8px"><span class="badge off">-${data.removed.length} removed</span></div>
            <div class="table-wrap"><table>
              ${data.removed.map((r: any) => html`<tr><td colspan="4" style="color:#ef4444"><code>${JSON.stringify(r).substring(0, 200)}</code></td></tr>`)}
            </table></div>
          ` : ''}
          ${(data.changed?.length > 0) ? html`
            <div style="margin-bottom:8px"><span class="badge warn">~${data.changed.length} changed</span></div>
            <div class="table-wrap"><table>
              ${data.changed.slice(0, 50).map((c: any) => html`
                <tr>
                  <td><code>ID ${c.id}</code></td>
                  <td style="max-width:300px;color:#ef4444;text-decoration:line-through">${JSON.stringify(c.from).substring(0, 150)}</td>
                  <td style="max-width:300px;color:#16a34a">${JSON.stringify(c.to).substring(0, 150)}</td>
                </tr>
              `)}
            </table></div>
          ` : ''}
          ${(!data.added?.length && !data.removed?.length && !data.changed?.length) ? html`<div class="empty">No changes</div>` : ''}
        </div>
      `)}
    `;
  }
}
