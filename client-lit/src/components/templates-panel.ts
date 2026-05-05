import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { api } from '../services/api.js';

interface TemplateSummary {
  id: string;
  name: string;
  industry: string;
  description: string;
  equipment_type: string;
  tag_count: number;
  alarm_count: number;
  flow_count: number;
  has_oee: boolean;
}

interface IndustryInfo {
  id: string;
  name: string;
  description: string;
}

interface TemplateDetail {
  id: string;
  name: string;
  industry: string;
  description: string;
  equipment_type: string;
  tags: any[];
  alarms: any[];
  flows: any[];
  oee_config: any;
  dashboard: any;
}

@customElement('vlt-templates-panel')
export class TemplatesPanel extends LitElement {
  static styles = css`
    :host { display: block; padding: 16px; overflow-y: auto; height: 100%; box-sizing: border-box; }
    h2 { font-size: 18px; font-weight: 600; color: var(--fg); margin: 0 0 16px; }
    h3 { font-size: 15px; font-weight: 600; color: var(--fg); margin: 16px 0 8px; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 16px; }
    .card {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 20px;
      cursor: pointer; transition: border-color 0.2s;
    }
    .card:hover { border-color: #f97316; }
    .card.selected { border-color: #f97316; box-shadow: 0 0 0 1px #f97316; }
    .card-title { font-size: 16px; font-weight: 600; color: var(--fg); margin-bottom: 6px; }
    .card-desc { font-size: 13px; color: var(--muted); line-height: 1.4; margin-bottom: 12px; }
    .tag-row { display: flex; gap: 8px; flex-wrap: wrap; }
    .badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; }
    .badge.orange { background: #f9731622; color: #f97316; }
    .badge.green { background: #16a34a22; color: #16a34a; }
    .badge.blue { background: #3b82f622; color: #3b82f6; }
    .badge.purple { background: #8b5cf622; color: #8b5cf6; }
    .tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 1px solid var(--border); }
    .tab { padding: 8px 16px; cursor: pointer; font-size: 13px; color: var(--muted); border-bottom: 2px solid transparent; }
    .tab.active { color: var(--fg); border-bottom-color: #f97316; }
    .btn { padding: 8px 16px; border-radius: 6px; border: 1px solid var(--border); background: var(--card); color: var(--fg); cursor: pointer; font-size: 13px; }
    .btn:hover { background: var(--border); }
    .btn.primary { background: #f97316; color: #fff; border-color: #f97316; }
    .btn.primary:hover { background: #ea580c; }
    .btn:disabled { opacity: 0.5; cursor: not-allowed; }
    .empty { color: var(--muted); font-size: 13px; padding: 20px; text-align: center; }
    .detail-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 16px; }
    .detail-item { }
    .detail-label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 2px; }
    .detail-value { font-size: 14px; color: var(--fg); font-weight: 500; }
    .preview-table { width: 100%; border-collapse: collapse; margin-top: 8px; }
    .preview-table th { text-align: left; font-size: 11px; color: var(--muted); text-transform: uppercase; padding: 6px 8px; border-bottom: 1px solid var(--border); }
    .preview-table td { font-size: 12px; padding: 6px 8px; border-bottom: 1px solid var(--border); color: var(--fg); }
    .preview-table tr:hover { background: var(--border); }
    .form-grid { display: grid; grid-template-columns: 120px 1fr; gap: 8px 12px; align-items: center; margin-bottom: 16px; }
    .form-label { font-size: 12px; color: var(--muted); text-align: right; }
    input, select { padding: 6px 10px; border-radius: 6px; border: 1px solid var(--border); background: var(--card); color: var(--fg); font-size: 13px; font-family: inherit; }
    input:focus, select:focus { outline: none; border-color: #f97316; }
    .actions { display: flex; gap: 8px; margin-top: 16px; }
    .success-msg { color: #16a34a; font-size: 13px; font-weight: 500; margin-top: 12px; }
    .error-msg { color: #ef4444; font-size: 13px; font-weight: 500; margin-top: 12px; }
    .loading { display: flex; align-items: center; gap: 8px; color: var(--muted); font-size: 13px; }
  `;

  @state() private _templates: TemplateSummary[] = [];
  @state() private _industries: IndustryInfo[] = [];
  @state() private _selectedIndustry = '';
  @state() private _selectedId: string | null = null;
  @state() private _detail: TemplateDetail | null = null;
  @state() private _tab: 'browse' | 'detail' | 'instantiate' = 'browse';
  @state() private _loading = false;
  @state() private _error = '';
  @state() private _success = '';
  // Instantiate form
  @state() private _instanceName = '';
  @state() private _instanceCount = '2';
  @state() private _instantiating = false;
  // Preview
  @state() private _previewTags: any[] = [];
  @state() private _previewAlarms: any[] = [];

  connectedCallback() {
    super.connectedCallback();
    this._load();
  }

  private async _load() {
    this._loading = true;
    this._error = '';
    try {
      const data = await api<any>('/templates' + (this._selectedIndustry ? `?industry=${this._selectedIndustry}` : ''));
      this._templates = data.templates || [];
      this._industries = data.industries || [];
    } catch (e: any) {
      this._error = e.message || 'Failed to load templates';
    }
    this._loading = false;
  }

  private async _selectTemplate(id: string) {
    this._selectedId = id;
    this._tab = 'detail';
    this._loading = true;
    try {
      this._detail = await api<TemplateDetail>(`/templates/${id}`);
    } catch (e: any) {
      this._error = e.message;
    }
    this._loading = false;
  }

  private async _preview() {
    if (!this._selectedId) return;
    this._loading = true;
    try {
      const result = await api<any>('/templates/preview', {
        method: 'POST',
        body: JSON.stringify({
          template_id: this._selectedId,
          instance_counts: { 'n': parseInt(this._instanceCount) || 2 },
        }),
      });
      this._previewTags = result.expanded_tags || [];
      this._previewAlarms = result.expanded_alarms || [];
      this._tab = 'instantiate';
    } catch (e: any) {
      this._error = e.message;
    }
    this._loading = false;
  }

  private async _instantiate() {
    if (!this._selectedId || !this._instanceName.trim()) return;
    this._instantiating = true;
    this._error = '';
    this._success = '';
    try {
      const result = await api<any>(`/templates/${this._selectedId}/instantiate`, {
        method: 'POST',
        body: JSON.stringify({
          template_id: this._selectedId,
          name: this._instanceName.trim(),
          instance_counts: { 'n': parseInt(this._instanceCount) || 2 },
        }),
      });
      this._success = `Template instantiated: ${result.tags_created} tags, ${result.alarms_created} alarms, ${result.flows_created} flows created on device "${result.device_id}"`;
    } catch (e: any) {
      this._error = e.message;
    }
    this._instantiating = false;
  }

  private _industryName(id: string): string {
    return this._industries.find(i => i.id === id)?.name || id;
  }

  render() {
    return html`
      <h2>Vertical Templates</h2>
      <p style="color:var(--muted);font-size:13px;margin-bottom:16px">
        Pre-built configurations for specific industries. Select a template, preview the tags and alarms, then instantiate to create devices, tags, screens, and flow rules automatically.
      </p>

      <div class="tabs">
        <div class="tab ${this._tab === 'browse' ? 'active' : ''}" @click=${() => { this._tab = 'browse'; this._selectedId = null; }}>Browse</div>
        <div class="tab ${this._tab === 'detail' ? 'active' : ''}" @click=${() => { if (this._selectedId) this._tab = 'detail'; }}>Detail</div>
        <div class="tab ${this._tab === 'instantiate' ? 'active' : ''}" @click=${() => { if (this._selectedId) this._tab = 'instantiate'; }}>Instantiate</div>
      </div>

      ${this._error ? html`<div class="error-msg">${this._error}</div>` : ''}
      ${this._success ? html`<div class="success-msg">${this._success}</div>` : ''}

      ${this._loading ? html`<div class="loading">Loading...</div>` : ''}
      ${this._tab === 'browse' ? this._renderBrowse() : ''}
      ${this._tab === 'detail' ? this._renderDetail() : ''}
      ${this._tab === 'instantiate' ? this._renderInstantiate() : ''}
    `;
  }

  private _renderBrowse() {
    const filtered = this._selectedIndustry
      ? this._templates.filter(t => t.industry === this._selectedIndustry)
      : this._templates;

    return html`
      <div style="margin-bottom:12px">
        <select .value=${this._selectedIndustry} @change=${(e: any) => { this._selectedIndustry = e.target.value; }}>
          <option value="">All Industries</option>
          ${this._industries.map(i => html`
            <option value=${i.id}>${i.name}</option>
          `)}
        </select>
      </div>

      ${filtered.length === 0 ? html`<div class="empty">No templates available for this industry.</div>` : ''}

      <div class="grid">
        ${filtered.map(t => html`
          <div class="card ${this._selectedId === t.id ? 'selected' : ''}" @click=${() => this._selectTemplate(t.id)}>
            <div class="card-title">${t.name}</div>
            <div class="card-desc">${t.description}</div>
            <div class="tag-row">
              <span class="badge orange">${this._industryName(t.industry)}</span>
              <span class="badge green">${t.tag_count} tags</span>
              <span class="badge blue">${t.alarm_count} alarms</span>
              <span class="badge purple">${t.flow_count} flows</span>
              ${t.has_oee ? html`<span class="badge orange">OEE</span>` : ''}
            </div>
          </div>
        `)}
      </div>
    `;
  }

  private _renderDetail() {
    if (!this._detail) return html`<div class="empty">Select a template from Browse.</div>`;
    const d = this._detail;
    return html`
      <div class="card" style="margin-bottom:16px">
        <div class="card-title" style="font-size:18px">${d.name}</div>
        <div class="card-desc">${d.description}</div>
        <div class="detail-grid">
          <div class="detail-item">
            <div class="detail-label">Industry</div>
            <div class="detail-value">${this._industryName(d.industry)}</div>
          </div>
          <div class="detail-item">
            <div class="detail-label">Equipment Type</div>
            <div class="detail-value">${d.equipment_type}</div>
          </div>
          <div class="detail-item">
            <div class="detail-label">Tags</div>
            <div class="detail-value">${d.tags.length} tag patterns</div>
          </div>
          <div class="detail-item">
            <div class="detail-label">Alarms</div>
            <div class="detail-value">${d.alarms.length} alarm rules</div>
          </div>
          <div class="detail-item">
            <div class="detail-label">Flow Rules</div>
            <div class="detail-value">${d.flows.length} automation rules</div>
          </div>
          <div class="detail-item">
            <div class="detail-label">OEE Tracking</div>
            <div class="detail-value">${d.oee_config ? 'Yes' : 'No'}</div>
          </div>
        </div>
      </div>

      <h3>Tag Patterns (ISA-5.1)</h3>
      <table class="preview-table">
        <tr><th>Pattern</th><th>Description</th><th>Unit</th><th>Type</th><th>Group</th></tr>
        ${d.tags.map(t => html`
          <tr>
            <td><code>${t.name_pattern}</code></td>
            <td>${t.description}</td>
            <td>${t.unit}</td>
            <td>${t.data_type}</td>
            <td>${t.group}</td>
          </tr>
        `)}
      </table>

      <h3>Alarm Rules</h3>
      <table class="preview-table">
        <tr><th>Tag</th><th>Severity</th><th>Condition</th><th>Threshold</th><th>Message</th></tr>
        ${d.alarms.map(a => html`
          <tr>
            <td><code>${a.tag_pattern}</code></td>
            <td><span class="badge ${a.severity === 'critical' ? 'purple' : 'orange'}">${a.severity}</span></td>
            <td>${a.comparison}</td>
            <td>${a.threshold}</td>
            <td>${a.message_template}</td>
          </tr>
        `)}
      </table>

      ${d.flows.length > 0 ? html`
        <h3>Automation Rules</h3>
        <table class="preview-table">
          <tr><th>Name</th><th>Trigger</th><th>Description</th></tr>
          ${d.flows.map(f => html`
            <tr>
              <td>${f.name}</td>
              <td>${f.trigger_type}</td>
              <td>${f.description}</td>
            </tr>
          `)}
        </table>
      ` : ''}

      <div class="actions">
        <button class="btn primary" @click=${() => this._preview()}>Preview &amp; Instantiate</button>
        <button class="btn" @click=${() => { this._tab = 'browse'; }}>Back</button>
      </div>
    `;
  }

  private _renderInstantiate() {
    return html`
      <div class="card" style="margin-bottom:16px">
        <h3 style="margin-top:0">Instantiate Template</h3>
        <div class="form-grid">
          <span class="form-label">Name</span>
          <input type="text" placeholder="e.g. My Plant Pump Station" .value=${this._instanceName} @input=${(e: any) => { this._instanceName = e.target.value; }} />
          <span class="form-label">Instances</span>
          <input type="number" min="1" max="20" .value=${this._instanceCount} @input=${(e: any) => { this._instanceCount = e.target.value; }} />
          <span class="form-label" style="font-size:11px;color:var(--muted)">Multi-instance tags (e.g. PMP{n}_RUN) will be expanded 1..N</span>
          <span></span>
        </div>
      </div>

      ${this._previewTags.length > 0 ? html`
        <h3>Preview: ${this._previewTags.length} Tags</h3>
        <table class="preview-table">
          <tr><th>Tag Name</th><th>Description</th><th>Unit</th><th>Type</th><th>Widget</th></tr>
          ${this._previewTags.slice(0, 30).map(t => html`
            <tr>
              <td><code>${t.name}</code></td>
              <td>${t.description}</td>
              <td>${t.unit}</td>
              <td>${t.data_type}</td>
              <td>${t.widget}</td>
            </tr>
          `)}
        </table>
        ${this._previewTags.length > 30 ? html`<div class="empty">... and ${this._previewTags.length - 30} more tags</div>` : ''}
      ` : ''}

      ${this._previewAlarms.length > 0 ? html`
        <h3>Preview: ${this._previewAlarms.length} Alarms</h3>
        <table class="preview-table">
          <tr><th>Tag</th><th>Severity</th><th>Condition</th><th>Threshold</th></tr>
          ${this._previewAlarms.map(a => html`
            <tr>
              <td><code>${a.tag_pattern}</code></td>
              <td><span class="badge ${a.severity === 'critical' ? 'purple' : 'orange'}">${a.severity}</span></td>
              <td>${a.comparison}</td>
              <td>${a.threshold}</td>
            </tr>
          `)}
        </table>
      ` : ''}

      <div class="actions">
        <button class="btn primary" ?disabled=${this._instantiating || !this._instanceName.trim()} @click=${() => this._instantiate()}>
          ${this._instantiating ? 'Creating...' : 'Instantiate Template'}
        </button>
        <button class="btn" @click=${() => { this._tab = 'detail'; }}>Back to Detail</button>
        <button class="btn" @click=${() => { this._tab = 'browse'; this._selectedId = null; }}>Back to Browse</button>
      </div>
    `;
  }
}
