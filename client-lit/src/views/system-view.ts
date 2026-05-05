import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { api } from '../services/api.js';

interface SystemInfo {
  hostname: string;
  os: string;
  uptime_secs: number;
  cpu_usage: number;
  memory_used_mb: number;
  memory_total_mb: number;
  memory_percent: number;
  disk_used_gb: number;
  disk_total_gb: number;
  disk_percent: number;
  process: { pid: number; memory_mb: number; cpu_percent: number; uptime_secs: number; binary_size_mb: number };
}

@customElement('vlt-system-view')
export class SystemView extends LitElement {
  static styles = css`
    :host { display: block; padding: 8px; overflow-y: auto; height: 100%; box-sizing: border-box; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; }
    .card {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 20px;
    }
    .card.wide { grid-column: span 2; }
    .label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; }
    .value { font-size: 28px; font-weight: 700; color: var(--fg); }
    .value.ok { color: var(--ok); }
    .value.warn { color: var(--warn); }
    .value.crit { color: var(--crit); }
    .sub { font-size: 13px; color: var(--muted); margin-top: 4px; }
    .bar { margin-top: 8px; height: 6px; border-radius: 3px; background: var(--border); }
    .bar-fill { height: 100%; border-radius: 3px; transition: width 0.3s; }
    .process-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; }
    .process-card { text-align: center; }
    .process-value { font-size: 24px; font-weight: 700; color: #22c55e; }
    .process-label { font-size: 11px; color: var(--muted); margin-top: 4px; }
  `;

  @state() private _info: SystemInfo | null = null;
  private _interval?: ReturnType<typeof setInterval>;

  connectedCallback() {
    super.connectedCallback();
    this._load();
    this._interval = setInterval(() => this._load(), 5000);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    if (this._interval) clearInterval(this._interval);
  }

  private async _load() {
    try {
      this._info = await api<SystemInfo>('/system');
    } catch {
      this._info = null;
    }
  }

  private _barColor(pct: number) {
    return pct > 90 ? 'var(--crit)' : pct > 70 ? 'var(--warn)' : 'var(--ok)';
  }

  private _fmtUptime(s: number) {
    const d = Math.floor(s / 86400);
    const h = Math.floor((s % 86400) / 3600);
    const m = Math.floor((s % 3600) / 60);
    return `${d}d ${h}h ${m}m`;
  }

  render() {
    if (!this._info) {
      return html`<div style="display:flex;align-items:center;justify-content:center;height:300px;color:var(--muted)">Connecting to server...</div>`;
    }
    const i = this._info;
    return html`
      <div class="grid">
        <div class="card">
          <div class="label">Host</div>
          <div class="value">${i.hostname}</div>
          <div class="sub">${i.os} — up ${this._fmtUptime(i.uptime_secs)}</div>
        </div>
        <div class="card">
          <div class="label">CPU Usage</div>
          <div class="value ${i.cpu_usage > 80 ? 'crit' : i.cpu_usage > 50 ? 'warn' : 'ok'}">${i.cpu_usage.toFixed(1)}%</div>
          <div class="bar"><div class="bar-fill" style="background:${this._barColor(i.cpu_usage)};width:${Math.min(i.cpu_usage, 100)}%"></div></div>
        </div>
        <div class="card">
          <div class="label">Memory</div>
          <div class="value">${i.memory_used_mb.toFixed(0)} <span style="font-size:14px;color:var(--muted);font-weight:400">/ ${i.memory_total_mb.toFixed(0)} MB</span></div>
          <div class="sub">${i.memory_percent.toFixed(1)}%</div>
          <div class="bar"><div class="bar-fill" style="background:${this._barColor(i.memory_percent)};width:${Math.min(i.memory_percent, 100)}%"></div></div>
        </div>
        <div class="card">
          <div class="label">Disk</div>
          <div class="value">${i.disk_used_gb.toFixed(1)} <span style="font-size:14px;color:var(--muted);font-weight:400">/ ${i.disk_total_gb.toFixed(1)} GB</span></div>
          <div class="sub">${i.disk_percent.toFixed(1)}%</div>
          <div class="bar"><div class="bar-fill" style="background:${this._barColor(i.disk_percent)};width:${Math.min(i.disk_percent, 100)}%"></div></div>
        </div>
        <div class="card wide" style="background:linear-gradient(135deg,#1a1a2e 0%,#16213e 100%);border-color:#22c55e">
          <div class="label" style="color:#22c55e;font-weight:700;letter-spacing:0.08em">Rust Service (PID ${i.process.pid})</div>
          <div class="process-grid">
            <div class="process-card">
              <div class="process-value">${i.process.memory_mb.toFixed(1)} MB</div>
              <div class="process-label">RSS Memory</div>
            </div>
            <div class="process-card">
              <div class="process-value">${i.process.cpu_percent.toFixed(1)}%</div>
              <div class="process-label">CPU Usage</div>
            </div>
            <div class="process-card">
              <div class="process-value">${this._fmtUptime(i.process.uptime_secs)}</div>
              <div class="process-label">Uptime</div>
            </div>
            <div class="process-card">
              <div class="process-value">${i.process.binary_size_mb.toFixed(1)} MB</div>
              <div class="process-label">Binary Size</div>
            </div>
          </div>
        </div>
      </div>

      <vlt-integration-panel></vlt-integration-panel>

      <vlt-templates-panel></vlt-templates-panel>

      <vlt-docs-panel></vlt-docs-panel>
    `;
  }
}
