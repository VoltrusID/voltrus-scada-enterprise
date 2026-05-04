var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, state, property } from 'lit/decorators.js';
import { api, fetchOk } from '../services/api.js';
import { LIVE_PTS } from '../types.js';
import { dataManager } from '../services/api.js';
import '../components/sensor-card.js';
let CanvasView = class CanvasView extends LitElement {
    constructor() {
        super(...arguments);
        this._sensors = [];
        this._layouts = [];
        this._dataMap = {};
        this._currentValues = {};
        this._selectedId = null;
        this._dragging = null;
        this._resizing = null;
        this._loading = true;
        this.screenId = 'all';
        this._onData = (e) => {
            const batch = e.detail;
            if (!batch?.length)
                return;
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
        this._onMouseMove = (e) => {
            if (this._dragging) {
                const d = this._dragging;
                this._updateLayout(d.id, { x: d.origX + e.clientX - d.startX, y: d.origY + e.clientY - d.startY });
            }
            if (this._resizing) {
                const d = this._resizing;
                this._updateLayout(d.id, { width: Math.max(200, d.origW + e.clientX - d.startX), height: Math.max(150, d.origH + e.clientY - d.startY) });
            }
        };
        this._onMouseUp = () => {
            this._dragging = null;
            this._resizing = null;
            window.removeEventListener('mousemove', this._onMouseMove);
            window.removeEventListener('mouseup', this._onMouseUp);
        };
    }
    static { this.styles = css `
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
  `; }
    connectedCallback() {
        super.connectedCallback();
        dataManager.addEventListener('data', this._onData);
        dataManager.connect();
        this._load();
    }
    disconnectedCallback() {
        super.disconnectedCallback();
        dataManager.removeEventListener('data', this._onData);
    }
    updated(changed) {
        if (changed.has('screenId')) {
            this._load();
        }
    }
    async _load() {
        this._loading = true;
        try {
            const path = this.screenId === 'all' ? '/data-sources' : `/screens/${this.screenId}/data-sources`;
            this._sensors = await api(path);
            const actualId = this.screenId === 'all' ? '__overview' : this.screenId;
            this._layouts = await api(`/screens/${actualId}/canvas-layouts`);
        }
        catch { /* ignore */ }
        this._loading = false;
    }
    _layoutFor(id) {
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
    _onMouseDown(id, e) {
        if (e.target?.classList?.contains('resize-handle'))
            return;
        this._selectedId = id;
        this._dragging = { id, startX: e.clientX, startY: e.clientY, origX: this._layoutFor(id).x, origY: this._layoutFor(id).y };
        window.addEventListener('mousemove', this._onMouseMove);
        window.addEventListener('mouseup', this._onMouseUp);
    }
    _onResizeDown(id, e) {
        e.stopPropagation();
        const l = this._layoutFor(id);
        this._resizing = { id, startX: e.clientX, startY: e.clientY, origW: l.width, origH: l.height };
        window.addEventListener('mousemove', this._onMouseMove);
        window.addEventListener('mouseup', this._onMouseUp);
    }
    _updateLayout(id, patch) {
        this._layouts = this._layouts.map(l => l.data_source_id === id ? { ...l, ...patch } : l);
    }
    async saveLayouts() {
        try {
            const actualId = this.screenId === 'all' ? '__overview' : this.screenId;
            await fetchOk(`/api/v1/screens/${actualId}/canvas-layouts`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(this._layouts) });
        }
        catch { /* ignore */ }
    }
    render() {
        if (this._loading && this._sensors.length === 0) {
            return html `<div style="display:flex;align-items:center;justify-content:center;height:200px;color:var(--muted)">Loading sensors...</div>`;
        }
        return html `
      <div class="container">
        ${this._sensors.map(s => {
            const l = this._layoutFor(s.id);
            const chartData = (this._dataMap[String(s.id)] || []).map(d => d.value).filter(v => v != null && !Number.isNaN(v));
            return html `
            <div class="card-wrap ${this._selectedId === s.id ? 'selected' : ''}"
              style="left:${l.x}px;top:${l.y}px;width:${l.width}px;height:${l.height}px"
              @mousedown="${(e) => this._onMouseDown(s.id, e)}"
            >
              <div class="drag-bar"></div>
              <vlt-sensor-card .sensor="${{ config: s, data: this._currentValues[String(s.id)], chartData }}"></vlt-sensor-card>
              <div class="resize-handle" @mousedown="${(e) => this._onResizeDown(s.id, e)}"></div>
            </div>
          `;
        })}
      </div>
    `;
    }
};
__decorate([
    state()
], CanvasView.prototype, "_sensors", void 0);
__decorate([
    state()
], CanvasView.prototype, "_layouts", void 0);
__decorate([
    state()
], CanvasView.prototype, "_dataMap", void 0);
__decorate([
    state()
], CanvasView.prototype, "_currentValues", void 0);
__decorate([
    state()
], CanvasView.prototype, "_selectedId", void 0);
__decorate([
    state()
], CanvasView.prototype, "_dragging", void 0);
__decorate([
    state()
], CanvasView.prototype, "_resizing", void 0);
__decorate([
    state()
], CanvasView.prototype, "_loading", void 0);
__decorate([
    property({ type: String })
], CanvasView.prototype, "screenId", void 0);
CanvasView = __decorate([
    customElement('vlt-canvas-view')
], CanvasView);
export { CanvasView };
//# sourceMappingURL=canvas-view.js.map