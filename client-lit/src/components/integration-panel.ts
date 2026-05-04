import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { api } from '../services/api.js';

interface EgressJob {
  id: number;
  name: string;
  enabled: boolean;
  schedule: string;
  format: string;
  destination: string;
  device_ids: string;
  time_range: string;
  compression: boolean;
  last_run: number | null;
  last_status: string | null;
  last_error: string | null;
}

interface OutboundWebhook {
  id: number;
  name: string;
  url: string;
  events: string;
  enabled: boolean;
  retry_count: number;
  timeout_ms: number;
  last_triggered: number | null;
  last_status: string | null;
}

@customElement('vlt-integration-panel')
export class IntegrationPanel extends LitElement {
  static styles = css`
    :host { display: block; padding: 16px; overflow-y: auto; height: 100%; box-sizing: border-box; }
    h2 { font-size: 18px; font-weight: 600; color: var(--fg); margin: 0 0 16px; }
    h3 { font-size: 15px; font-weight: 600; color: var(--fg); margin: 24px 0 12px; }
    .card { background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 16px; margin-bottom: 12px; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 12px; }
    .label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px; }
    .value { font-size: 14px; color: var(--fg); font-family: 'JetBrains Mono', monospace; word-break: break-all; }
    .badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; }
    .badge.on { background: #16a34a22; color: #16a34a; }
    .badge.off { background: #ef444422; color: #ef4444; }
    .badge.success { background: #16a34a22; color: #16a34a; }
    .badge.error { background: #ef444422; color: #ef4444; }
    .badge.never { background: var(--border); color: var(--muted); }
    .row { display: flex; justify-content: space-between; align-items: center; padding: 8px 0; border-bottom: 1px solid var(--border); }
    .row:last-child { border-bottom: none; }
    .btn { padding: 6px 14px; border-radius: 6px; border: 1px solid var(--border); background: var(--card); color: var(--fg); cursor: pointer; font-size: 12px; }
    .btn:hover { background: var(--border); }
    .btn.primary { background: #f97316; color: #fff; border-color: #f97316; }
    .btn.danger { color: #ef4444; border-color: #ef4444; }
    .endpoint { font-family: 'JetBrains Mono', monospace; font-size: 12px; background: var(--border); padding: 8px 12px; border-radius: 6px; margin: 4px 0; word-break: break-all; }
    .tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 1px solid var(--border); }
    .tab { padding: 8px 16px; cursor: pointer; font-size: 13px; color: var(--muted); border-bottom: 2px solid transparent; }
    .tab.active { color: var(--fg); border-bottom-color: #f97316; }
    .empty { color: var(--muted); font-size: 13px; padding: 20px; text-align: center; }
    .actions { display: flex; gap: 8px; margin-top: 8px; }
  `;

  @state() private _tab = 'overview';
  @state() private _egressJobs: EgressJob[] = [];
  @state() private _webhooks: OutboundWebhook[] = [];
  @state() private _connectionInfo: any = null;
  @state() private _loading = false;

  connectedCallback() {
    super.connectedCallback();
    this._load();
  }

  private async _load() {
    this._loading = true;
    try {
      const [jobs, webhooks, info] = await Promise.all([
        api<EgressJob[]>('/egress/jobs').catch(() => []),
        api<OutboundWebhook[]>('/integration/webhooks/outbound').catch(() => []),
        api<any>('/integration/connection-info').catch(() => null),
      ]);
      this._egressJobs = jobs;
      this._webhooks = webhooks;
      this._connectionInfo = info;
    } catch { /* tier gated */ }
    this._loading = false;
  }

  private async _runEgressJob(id: number) {
    try {
      await api(`/egress/jobs/${id}/run`, { method: 'POST' });
      this._load();
    } catch (e: any) { alert('Failed: ' + e.message); }
  }

  private async _deleteEgressJob(id: number) {
    if (!confirm('Delete this egress job?')) return;
    try {
      await api(`/egress/jobs/${id}`, { method: 'DELETE' });
      this._load();
    } catch (e: any) { alert('Failed: ' + e.message); }
  }

  private async _testWebhook(id: number) {
    try {
      const result = await api<any>(`/integration/webhooks/outbound/${id}/test`, { method: 'POST' });
      alert(result.ok ? `Success (HTTP ${result.status})` : `Failed: ${result.error}`);
    } catch (e: any) { alert('Test failed: ' + e.message); }
  }

  private async _deleteWebhook(id: number) {
    if (!confirm('Delete this webhook?')) return;
    try {
      await api(`/integration/webhooks/outbound/${id}`, { method: 'DELETE' });
      this._load();
    } catch (e: any) { alert('Failed: ' + e.message); }
  }

  private _fmtTime(ts: number | null) {
    if (!ts) return 'never';
    return new Date(ts * 1000).toLocaleString();
  }

  render() {
    return html`
      <h2>External Integration</h2>
      <div class="tabs">
        <div class="tab ${this._tab === 'overview' ? 'active' : ''}" @click=${() => { this._tab = 'overview'; }}>Connection Info</div>
        <div class="tab ${this._tab === 'webhooks' ? 'active' : ''}" @click=${() => { this._tab = 'webhooks'; }}>Webhooks</div>
        <div class="tab ${this._tab === 'egress' ? 'active' : ''}" @click=${() => { this._tab = 'egress'; }}>Data Egress</div>
      </div>

      ${this._tab === 'overview' ? this._renderOverview() : ''}
      ${this._tab === 'webhooks' ? this._renderWebhooks() : ''}
      ${this._tab === 'egress' ? this._renderEgress() : ''}
    `;
  }

  private _renderOverview() {
    if (!this._connectionInfo) return html`<div class="empty">Connection info unavailable (Professional tier required)</div>`;
    const info = this._connectionInfo;
    return html`
      <div class="card">
        <h3>Prometheus Exporter</h3>
        <div class="label">Endpoint</div>
        <div class="endpoint">GET ${info.prometheus?.endpoint || '/metrics'}</div>
        <div class="label">Metrics</div>
        ${(info.prometheus?.metrics || []).map((m: string) => html`<div class="value">${m}</div>`)}
      </div>

      <div class="card">
        <h3>REST API</h3>
        <div class="label">Base URL</div>
        <div class="endpoint">${info.rest_api?.base_url || ''}</div>
        <div class="label">Authentication</div>
        <div class="value">${info.rest_api?.authentication || ''}</div>
      </div>

      <div class="card">
        <h3>Grafana</h3>
        <div class="label">Query Endpoint</div>
        <div class="endpoint">${info.grafana?.query_endpoint || ''}</div>
        <div class="label">Search Endpoint</div>
        <div class="endpoint">${info.grafana?.search_endpoint || ''}</div>
      </div>

      <div class="card">
        <h3>Power BI</h3>
        <div class="label">Connector</div>
        <div class="value">${info.power_bi?.connector || ''}</div>
        <div class="label">URL</div>
        <div class="endpoint">${info.power_bi?.url || ''}</div>
      </div>

      ${info.sample_queries ? html`
        <div class="card">
          <h3>Sample Queries</h3>
          ${Object.entries(info.sample_queries).map(([k, v]) => html`
            <div class="row">
              <span class="label">${k.replace(/_/g, ' ')}</span>
              <span class="value" style="font-size:11px">${v}</span>
            </div>
          `)}
        </div>
      ` : ''}
    `;
  }

  private _renderWebhooks() {
    return html`
      <div class="card">
        <div style="display:flex;justify-content:space-between;align-items:center">
          <h3 style="margin:0">Outbound Webhooks</h3>
        </div>
      </div>
      ${this._webhooks.length === 0 ? html`<div class="empty">No outbound webhooks configured. Create one via the API.</div>` : ''}
      ${this._webhooks.map(w => html`
        <div class="card">
          <div class="row">
            <div>
              <strong>${w.name}</strong>
              <span class="badge ${w.enabled ? 'on' : 'off'}">${w.enabled ? 'ON' : 'OFF'}</span>
            </div>
            <div class="value" style="font-size:12px">${w.url}</div>
          </div>
          <div class="row">
            <div class="label">Events: ${w.events}</div>
            <div class="label">Last: ${this._fmtTime(w.last_triggered)} <span class="badge ${w.last_status === 'success' ? 'success' : w.last_status ? 'error' : 'never'}">${w.last_status || 'never'}</span></div>
          </div>
          <div class="actions">
            <button class="btn" @click=${() => this._testWebhook(w.id)}>Test</button>
            <button class="btn danger" @click=${() => this._deleteWebhook(w.id)}>Delete</button>
          </div>
        </div>
      `)}
    `;
  }

  private _renderEgress() {
    return html`
      <div class="card">
        <div style="display:flex;justify-content:space-between;align-items:center">
          <h3 style="margin:0">Scheduled Data Exports</h3>
        </div>
      </div>
      ${this._egressJobs.length === 0 ? html`<div class="empty">No egress jobs configured. Create one via the API.</div>` : ''}
      ${this._egressJobs.map(j => html`
        <div class="card">
          <div class="row">
            <div>
              <strong>${j.name}</strong>
              <span class="badge ${j.enabled ? 'on' : 'off'}">${j.enabled ? 'ON' : 'OFF'}</span>
              <span style="color:var(--muted);font-size:11px;margin-left:8px">${j.format.toUpperCase()}${j.compression ? ' (gz)' : ''}</span>
            </div>
          </div>
          <div class="row">
            <div class="label">Schedule</div>
            <div class="value" style="font-size:12px">${j.schedule} | Range: ${j.time_range}</div>
          </div>
          <div class="row">
            <div class="label">Destination</div>
            <div class="value" style="font-size:12px">${j.destination}</div>
          </div>
          <div class="row">
            <div class="label">Last Run</div>
            <div>
              <span style="font-size:12px">${this._fmtTime(j.last_run)}</span>
              <span class="badge ${j.last_status === 'success' ? 'success' : j.last_status ? 'error' : 'never'}">${j.last_status || 'never'}</span>
            </div>
          </div>
          ${j.last_error ? html`<div style="color:#ef4444;font-size:11px;margin-top:4px">${j.last_error}</div>` : ''}
          <div class="actions">
            <button class="btn primary" @click=${() => this._runEgressJob(j.id)}>Run Now</button>
            <button class="btn danger" @click=${() => this._deleteEgressJob(j.id)}>Delete</button>
          </div>
        </div>
      `)}
    `;
  }
}
