import { LitElement, html, css } from 'lit';
import { customElement, state, property } from 'lit/decorators.js';
import { api, fetchOk } from '../services/api.js';
import type { SensorConfig, SensorData, DataPoint, CanvasLayoutItem } from '../types.js';
import { LIVE_PTS } from '../types.js';
import { dataManager } from '../services/api.js';
import '../components/sensor-card.js';

@customElement('vlt-canvas-view')
export class CanvasView extends LitElement {
  static styles = css`
    :host { display: block; height: 100%; overflow: auto; position: relative; }
    .container { position: relative; min-height: 600px; padding: 20px; }
    .card-wrap {
      position: absolute; background: var(--card); border: 1px solid var(--border);
      border-radius: 10px; overflow: hidden; display: flex; flex-direction: column;
      user-select: none;
    }
    .card-wrap.selected { border-color: var(--cyan); box-shadow: 0 0 0 2px var(--cyan); }
    .card-wrap .drag-bar {
      height: 6px; background: var(--cyan); opacity: 0; cursor: grab; border-radius: 10px 10px 0 0;
    }
    .card-wrap:hover .drag-bar, .card-wrap.selected .drag-bar { opacity: 0.6; }
    .resize-handle {
      position: absolute; width: 12px; height: 12px; bottom: 0; right: 0; cursor: nwse-resize;
      background: var(--cyan); border-radius: 6px 0 6px 0; opacity: 0;
    }
    .card-wrap:hover .resize-handle, .card-wrap.selected .resize-handle { opacity: 0.7; }
  `;

  @state() private _sensors: SensorConfig[] = [];
  @state() private _layouts: CanvasLayoutItem[] = [];
  @state() private _dataMap: Record<string, DataPoint[]> = {};
  @state() private _currentValues: Record<string, SensorData> = {};
  @state() private _selectedId: number | null = null;
  @state() private _dragging: { id: number; startX: number; startY: number; origX: number; origY: number } | null = null;
  @state() private _resizing: { id: number; startX: number; startY: number; origW: number; origH: number } | null = null;
  @state() private _loading = true;

  @property({ type: String }) screenId = 'all';

  connectedCallback() {
    super.connectedCallback();
    dataManager.addEventListener('data', this._onData as EventListener);
    dataManager.connect();
    this._load();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    dataManager.removeEventListener('data', this._onData as EventListener);
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('screenId')) {
      this._load();
    }
  }

  private async _load() {
    this._loading = true;
    try {
      const path = this.screenId === 'all' ? '/data-sources' : `/screens/${this.screenId}/data-sources`;
      this._sensors = await api<SensorConfig[]>(path);
      const actualId = this.screenId === 'all' ? '__overview' : this.screenId;
      this._layouts = await api<CanvasLayoutItem[]>(`/screens/${actualId}/canvas-layouts`);
    } catch { /* ignore */ }
    this._loading = false;
  }

  private _onData = (e: CustomEvent<SensorData[]>) => {
    const batch = e.detail;
    if (!batch?.length) return;
    const newMap = { ...this._dataMap };
    const newVals = { ...this._currentValues };
    for (const item of batch) {
      const k = String(item.id);
      const arr = newMap[k] ? [...newMap[k]] : [];
      arr.push({ value: item.value, ts: Date.now() });
      newMap[k] = arr.length > LIVE_PTS ? arr.slice(-LIVE_PTS) : arr;
      newVals[k] = item;
    }
    this._dataMap = newMap;
    this._currentValues = newVals;
  };

  private _layoutFor(id: number): CanvasLayoutItem {
    const saved = this._layouts.find(l => l.data_source_id === id);
    const idx = this._sensors.findIndex(s => s.id === id);
    const cols = Math.max(1, Math.floor(1200 / 320));
    return saved || {
      data_source_id: id,
      x: (idx % cols) * 320,
      y: Math.floor(idx / cols) * 220,
      width: 300,
      height: 200,
    };
  }

  private _onMouseDown(id: number, e: MouseEvent) {
    if ((e.target as HTMLElement)?.classList?.contains('resize-handle')) return;
    this._selectedId = id;
    this._dragging = { id, startX: e.clientX, startY: e.clientY, origX: this._layoutFor(id).x, origY: this._layoutFor(id).y };
    window.addEventListener('mousemove', this._onMouseMove);
    window.addEventListener('mouseup', this._onMouseUp);
  }

  private _onResizeDown(id: number, e: MouseEvent) {
    e.stopPropagation();
    const l = this._layoutFor(id);
    this._resizing = { id, startX: e.clientX, startY: e.clientY, origW: l.width, origH: l.height };
    window.addEventListener('mousemove', this._onMouseMove);
    window.addEventListener('mouseup', this._onMouseUp);
  }

  private _onMouseMove = (e: MouseEvent) => {
    if (this._dragging) {
      const d = this._dragging;
      this._updateLayout(d.id, { x: d.origX + e.clientX - d.startX, y: d.origY + e.clientY - d.startY });
    }
    if (this._resizing) {
      const d = this._resizing;
      this._updateLayout(d.id, { width: Math.max(200, d.origW + e.clientX - d.startX), height: Math.max(150, d.origH + e.clientY - d.startY) });
    }
  };

  private _onMouseUp = () => {
    this._dragging = null;
    this._resizing = null;
    window.removeEventListener('mousemove', this._onMouseMove);
    window.removeEventListener('mouseup', this._onMouseUp);
  };

  private _updateLayout(id: number, patch: Partial<CanvasLayoutItem>) {
    this._layouts = this._layouts.map(l => l.data_source_id === id ? { ...l, ...patch } : l);
  }

  async saveLayouts() {
    try {
      const actualId = this.screenId === 'all' ? '__overview' : this.screenId;
      await fetchOk(`/api/v1/screens/${actualId}/canvas-layouts`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(this._layouts) });
    } catch { /* ignore */ }
  }

  render() {
    if (this._loading && this._sensors.length === 0) {
      return html`<div style="display:flex;align-items:center;justify-content:center;height:200px;color:var(--muted)">Loading sensors...</div>`;
    }
    return html`
      <div class="container">
        ${this._sensors.map(s => {
          const l = this._layoutFor(s.id);
          const chartData = (this._dataMap[String(s.id)] || []).map(d => d.value).filter(v => v != null && !Number.isNaN(v));
          return html`
            <div class="card-wrap ${this._selectedId === s.id ? 'selected' : ''}"
              style="left:${l.x}px;top:${l.y}px;width:${l.width}px;height:${l.height}px"
              @mousedown="${(e: MouseEvent) => this._onMouseDown(s.id, e)}"
            >
              <div class="drag-bar"></div>
              <vlt-sensor-card .sensor="${{ config: s, data: this._currentValues[String(s.id)], chartData }}"></vlt-sensor-card>
              <div class="resize-handle" @mousedown="${(e: MouseEvent) => this._onResizeDown(s.id, e)}"></div>
            </div>
          `;
        })}
      </div>
    `;
  }
}
