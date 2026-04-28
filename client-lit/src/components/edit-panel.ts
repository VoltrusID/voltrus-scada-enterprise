import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { api, fetchOk } from '../services/api.js';
import type { SensorConfig } from '../types.js';
import { SOURCE_CATEGORY, CHART_TYPES, SIMULATE_TYPES } from '../types.js';

@customElement('vlt-edit-panel')
export class EditPanel extends LitElement {
  static styles = css`
    :host {
      position: fixed; inset: 0; background: rgba(0,0,0,.6);
      display: flex; align-items: center; justify-content: center; z-index: 60;
    }
    .modal {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px;
      width: 480px; max-height: 80vh; display: flex; flex-direction: column;
    }
    .header {
      display: flex; align-items: center; justify-content: space-between;
      padding: 12px 16px; border-bottom: 1px solid var(--border);
    }
    .header span { font-size: 13px; font-weight: 600; color: var(--fg); }
    .close { background: none; border: none; color: var(--muted); cursor: pointer; font-size: 18; line-height: 1; }
    .tabs { display: flex; border-bottom: 1px solid var(--border); }
    .tabs button {
      padding: 8px 16px; font-size: 12px; font-weight: 600; cursor: pointer; border: none;
      background: transparent; color: var(--muted); text-transform: uppercase; letter-spacing: 0.04em;
      border-bottom: 2px solid transparent;
    }
    .tabs button.active { border-bottom-color: var(--cyan); color: var(--cyan); }
    .body { overflow: auto; flex: 1; padding: 16px; }
    .grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
    .field { display: flex; flex-direction: column; gap: 4px; }
    .field.wide { grid-column: 1 / -1; }
    .field label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
    .field input, .field select {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px;
      padding: 6px 8px; color: var(--fg); font-size: 13px; font-family: inherit; outline: none; width: 100%; box-sizing: border-box;
    }
    .field input:focus, .field select:focus { border-color: var(--border-light); }
    .footer { display: flex; gap: 8px; padding: 0 16px 16px; }
    .footer button {
      padding: 6px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; font-family: inherit;
    }
    .save { border: 1px solid var(--cyan); background: transparent; color: var(--cyan); }
    .delete { border: 1px solid var(--crit); background: transparent; color: var(--crit); }
  `;

  @state() private _tab: 'general' | 'source' = 'general';
  @state() private _form: Partial<SensorConfig> = {};
  @state() private _srcConfig: Record<string, any> = {};

  sensor?: SensorConfig;

  connectedCallback() {
    super.connectedCallback();
    if (this.sensor) {
      this._form = { ...this.sensor };
      try { this._srcConfig = JSON.parse(this.sensor.source_config || '{}'); } catch { this._srcConfig = {}; }
    }
  }

  private _save() {
    if (!this.sensor) return;
    const s = { ...this.sensor, ...this._form, source_config: JSON.stringify(this._srcConfig) } as SensorConfig;
    this.dispatchEvent(new CustomEvent('save', { detail: s }));
  }

  private _delete() {
    if (!this.sensor) return;
    this.dispatchEvent(new CustomEvent('delete', { detail: this.sensor.id }));
  }

  private _close() {
    this.dispatchEvent(new CustomEvent('close'));
  }

  render() {
    if (!this.sensor) return html``;
    const f = this._form;
    const isImage = (f.chart_type || 'sparkline') === 'image';
    const isText = (f.chart_type || 'sparkline') === 'text';
    const hideNumberFields = isImage || isText;
    const sourceTag = SOURCE_CATEGORY[f.source_type || 'none'];

    return html`
      <div @click="${this._close}">
        <div class="modal" @click="${(e: Event) => e.stopPropagation()}">
          <div class="header">
            <span>Edit: ${this.sensor.name}</span>
            <div style="display:flex;align-items:center;gap:8px">
              <span style="font-size:10px;padding:2px 8px;border-radius:4px;background:${sourceTag?.color ?? 'var(--muted)'}18;color:${sourceTag?.color ?? 'var(--muted)'};font-weight:700">${sourceTag?.tag ?? f.source_type} — ${sourceTag?.label ?? f.source_type}</span>
              <button class="close" @click="${this._close}">&times;</button>
            </div>
          </div>
          <div class="tabs">
            <button class="${this._tab === 'general' ? 'active' : ''}" @click="${() => this._tab = 'general'}">General</button>
            <button class="${this._tab === 'source' ? 'active' : ''}" @click="${() => this._tab = 'source'}">Source</button>
          </div>
          <div class="body">
            ${this._tab === 'general' ? html`
              <div class="grid2">
                <div class="field"><label>Name</label><input .value="${f.name || ''}" @input="${(e: InputEvent) => this._form = { ...this._form, name: (e.target as HTMLInputElement).value }}"></div>
                <div class="field"><label>Key</label><input .value="${f.key || ''}" @input="${(e: InputEvent) => this._form = { ...this._form, key: (e.target as HTMLInputElement).value }}"></div>
                ${!hideNumberFields ? html`
                  <div class="field"><label>Unit</label><input .value="${f.unit || ''}" @input="${(e: InputEvent) => this._form = { ...this._form, unit: (e.target as HTMLInputElement).value }}"></div>
                  <div class="field"><label>Min</label><input type="number" step="any" .value="${String(f.min_val ?? 0)}" @input="${(e: InputEvent) => this._form = { ...this._form, min_val: parseFloat((e.target as HTMLInputElement).value) || 0 }}"></div>
                  <div class="field"><label>Max</label><input type="number" step="any" .value="${String(f.max_val ?? 0)}" @input="${(e: InputEvent) => this._form = { ...this._form, max_val: parseFloat((e.target as HTMLInputElement).value) || 0 }}"></div>
                  <div class="field"><label>Warn</label><input type="number" step="any" .value="${String(f.warn_val ?? 0)}" @input="${(e: InputEvent) => this._form = { ...this._form, warn_val: parseFloat((e.target as HTMLInputElement).value) || 0 }}"></div>
                  <div class="field"><label>Crit</label><input type="number" step="any" .value="${String(f.crit_val ?? 0)}" @input="${(e: InputEvent) => this._form = { ...this._form, crit_val: parseFloat((e.target as HTMLInputElement).value) || 0 }}"></div>
                  <div class="field"><label>Base</label><input type="number" step="any" .value="${String(f.base_val ?? 0)}" @input="${(e: InputEvent) => this._form = { ...this._form, base_val: parseFloat((e.target as HTMLInputElement).value) || 0 }}"></div>
                  <div class="field"><label>Decimals</label><input type="number" .value="${String(f.decimals ?? 0)}" @input="${(e: InputEvent) => this._form = { ...this._form, decimals: parseInt((e.target as HTMLInputElement).value) || 0 }}"></div>
                ` : ''}
                <div class="field"><label>Color</label><input type="color" .value="${f.color || '#22c55e'}" @input="${(e: InputEvent) => this._form = { ...this._form, color: (e.target as HTMLInputElement).value }}"></div>
                <div class="field"><label>Chart Type</label>
                  <select .value="${f.chart_type || 'sparkline'}" @change="${(e: InputEvent) => this._form = { ...this._form, chart_type: (e.target as HTMLSelectElement).value }}">
                    ${CHART_TYPES.map(t => html`<option value="${t}">${t}</option>`)}
                  </select>
                </div>
              </div>
            ` : html`
              <div class="field wide"><label>Data Source</label>
                <select .value="${f.source_type || 'none'}" @change="${(e: InputEvent) => { this._form = { ...this._form, source_type: (e.target as HTMLSelectElement).value }; this._srcConfig = {}; }}">
                  <optgroup label="Real Data"><option value="modbus">Modbus (Live Device)</option></optgroup>
                  <optgroup label="Simulate">${SIMULATE_TYPES.map(t => html`<option value="${t}">${SOURCE_CATEGORY[t]?.label ?? t}</option>`)}</optgroup>
                  <optgroup label="Other"><option value="api">API (HTTP Poll)</option><option value="none">None (Static)</option></optgroup>
                </select>
              </div>
              <div class="grid2" style="margin-top:8px">
                ${f.source_type === 'api' ? html`
                  <div class="field wide"><label>URL</label><input .value="${this._srcConfig.url || ''}" @input="${(e: InputEvent) => this._srcConfig = { ...this._srcConfig, url: (e.target as HTMLInputElement).value }}"></div>
                ` : ''}
                ${f.source_type === 'modbus' ? html`
                  <div class="field"><label>Device ID</label><input .value="${this._srcConfig.modbus_device_id || ''}" @input="${(e: InputEvent) => this._srcConfig = { ...this._srcConfig, modbus_device_id: (e.target as HTMLInputElement).value }}"></div>
                  <div class="field"><label>Register</label><input .value="${this._srcConfig.modbus_register_name || ''}" @input="${(e: InputEvent) => this._srcConfig = { ...this._srcConfig, modbus_register_name: (e.target as HTMLInputElement).value }}"></div>
                ` : ''}
              </div>
            `}
          </div>
          <div class="footer">
            <button class="save" @click="${this._save}">Save</button>
            <button class="delete" @click="${this._delete}">Delete</button>
          </div>
        </div>
      </div>
    `;
  }
}
