var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import '../components/sensor-card.js';
import { dataManager } from '../services/api.js';
import { api, fetchOk } from '../services/api.js';
import { LIVE_PTS } from '../types.js';
const BREAKPOINTS = { lg: 1200, md: 768, sm: 0 };
const COLS = { lg: 24, md: 12, sm: 4 };
const ROW_HEIGHT = 50;
let GridView = class GridView extends LitElement {
    constructor() {
        super(...arguments);
        this._sensors = [];
        this._dataMap = {};
        this._currentValues = {};
        this._screenId = 'all';
        this._loading = true;
        this._layouts = { lg: [], md: [], sm: [] };
        this._editMode = false;
        this._draggingId = null;
        this._overId = null;
        this._bp = 'lg';
        this._containerWidth = 1200;
        this._onData = (e) => {
            const batch = e.detail;
            if (!batch?.length)
                return;
            const newMap = { ...this._dataMap };
            const newVals = { ...this._currentValues };
            for (const item of batch) {
                const k = String(item.id);
                const isOffline = item.value == null || Number.isNaN(item.value);
                if (isOffline) {
                    newVals[k] = item;
                    continue;
                }
                const arr = newMap[k] ? [...newMap[k]] : [];
                arr.push({ value: item.value, ts: Date.now() });
                newMap[k] = arr.length > LIVE_PTS ? arr.slice(-LIVE_PTS) : arr;
                newVals[k] = item;
            }
            this._dataMap = newMap;
            this._currentValues = newVals;
        };
    }
    static { this.styles = css `
    :host { display: block; padding: 0 20px 20px; overflow-y: auto; height: 100%; box-sizing: border-box; }
    .grid {
      display: grid;
      gap: 12px;
      grid-auto-rows: 50px;
      align-items: stretch;
    }
    .grid.lg { grid-template-columns: repeat(24, 1fr); }
    .grid.md { grid-template-columns: repeat(12, 1fr); }
    .grid.sm { grid-template-columns: repeat(4, 1fr); }
    .card-wrap {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px;
      overflow: hidden; display: flex; flex-direction: column;
      min-height: 0;
    }
    .card-wrap.edit { cursor: grab; }
    .card-wrap.edit.dragging { opacity: 0.5; cursor: grabbing; }
    .card-wrap.edit.over { outline: 2px dashed var(--cyan); outline-offset: 2px; }
    .drag-handle {
      height: 6px; background: var(--cyan); opacity: 0; cursor: grab; border-radius: 10px 10px 0 0; flex-shrink: 0;
    }
    .card-wrap.edit .drag-handle { opacity: 0.4; }
    .empty { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--muted); }
  `; }
    set screenId(id) {
        if (this._screenId === id)
            return;
        this._screenId = id;
        this._loadSensors();
        this._loadLayouts();
    }
    get screenId() { return this._screenId; }
    set editMode(v) { this._editMode = v; }
    get editMode() { return this._editMode; }
    connectedCallback() {
        super.connectedCallback();
        dataManager.addEventListener('data', this._onData);
        dataManager.connect();
        this._loadSensors();
        this._loadLayouts();
        this._seedHistory();
        this._resizeObs = new ResizeObserver(entries => {
            const w = entries[0]?.contentRect.width ?? 1200;
            this._containerWidth = w;
            this._bp = w >= BREAKPOINTS.lg ? 'lg' : w >= BREAKPOINTS.md ? 'md' : 'sm';
        });
        this._resizeObs.observe(this);
    }
    disconnectedCallback() {
        super.disconnectedCallback();
        dataManager.removeEventListener('data', this._onData);
        this._resizeObs?.disconnect();
    }
    async _loadSensors() {
        this._loading = true;
        try {
            const path = this._screenId === 'all' ? '/data-sources' : `/screens/${this._screenId}/data-sources`;
            this._sensors = await api(path);
        }
        catch {
            this._sensors = [];
        }
        this._loading = false;
    }
    async _loadLayouts() {
        try {
            const actualId = this._screenId === 'all' ? '__overview' : this._screenId;
            this._layouts = await api(`/screens/${actualId}/layouts`);
        }
        catch {
            this._layouts = { lg: [], md: [], sm: [] };
        }
    }
    async saveLayouts() {
        try {
            const actualId = this._screenId === 'all' ? '__overview' : this._screenId;
            await fetchOk(`/api/screens/${actualId}/layouts`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(this._layouts) });
        }
        catch { /* ignore */ }
    }
    async _seedHistory() {
        try {
            const dp = this._screenId === 'all' ? '' : `&screen=${this._screenId}`;
            const result = await api(`/history?range=30m${dp}`);
            if (!result.data)
                return;
            const expanded = {};
            for (const key of Object.keys(result.data)) {
                const s = result.data[key];
                if (s && Array.isArray(s.values)) {
                    const pts = [];
                    for (let i = 0; i < s.values.length; i++)
                        pts.push({ ts: s.start + i * s.interval, value: s.values[i] });
                    expanded[key] = pts;
                }
                else if (Array.isArray(s)) {
                    expanded[key] = s;
                }
            }
            const remapped = {};
            for (const s of this._sensors) {
                const pts = expanded[s.key];
                if (pts)
                    remapped[String(s.id)] = pts.length > LIVE_PTS ? pts.slice(-LIVE_PTS) : pts;
            }
            this._dataMap = remapped;
        }
        catch { /* ignore */ }
    }
    _cardSensors() {
        return this._sensors.map(s => ({
            config: s,
            data: this._currentValues[String(s.id)],
            chartData: (this._dataMap[String(s.id)] || []).map(d => d.value).filter(v => v != null && !Number.isNaN(v)),
        }));
    }
    _layoutFor(id) {
        const items = this._layouts[this._bp] || [];
        return items.find(l => l.i === String(id));
    }
    _gridStyleFor(l) {
        if (!l)
            return '';
        const cols = COLS[this._bp];
        const x = Math.min(l.x, cols - 1);
        const w = Math.min(l.w, cols - x);
        return `grid-column:${x + 1} / span ${w};grid-row:${l.y + 1} / span ${l.h};`;
    }
    _onSensorClick(e) {
        e.stopPropagation();
        this.dispatchEvent(new CustomEvent('sensor-click', { detail: e.detail }));
    }
    _onDragStart(id, e) {
        if (!this._editMode) {
            e.preventDefault();
            return;
        }
        this._draggingId = id;
        e.dataTransfer.effectAllowed = 'move';
        e.dataTransfer.setData('text/plain', String(id));
    }
    _onDragOver(id, e) {
        if (!this._editMode || this._draggingId === null || this._draggingId === id)
            return;
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
        this._overId = id;
    }
    _onDragLeave() {
        this._overId = null;
    }
    _onDrop(id, e) {
        if (!this._editMode || this._draggingId === null)
            return;
        e.preventDefault();
        const fromId = this._draggingId;
        const toId = id;
        this._draggingId = null;
        this._overId = null;
        if (fromId === toId)
            return;
        // Swap positions in all breakpoints
        const swapIn = (items) => {
            const from = items.find(l => l.i === String(fromId));
            const to = items.find(l => l.i === String(toId));
            if (!from || !to)
                return items;
            return items.map(l => {
                if (l.i === String(fromId))
                    return { ...l, x: to.x, y: to.y };
                if (l.i === String(toId))
                    return { ...l, x: from.x, y: from.y };
                return l;
            });
        };
        this._layouts = {
            lg: swapIn(this._layouts.lg || []),
            md: swapIn(this._layouts.md || []),
            sm: swapIn(this._layouts.sm || []),
        };
    }
    render() {
        if (this._loading && this._sensors.length === 0) {
            return html `<div class="empty">Loading sensors...</div>`;
        }
        const cards = this._cardSensors();
        if (!cards.length) {
            return html `<div class="empty">No sensors on this screen</div>`;
        }
        return html `
      <div class="grid ${this._bp} ${this._editMode ? 'edit' : ''}">
        ${cards.map(cs => {
            const l = this._layoutFor(cs.config.id);
            const style = this._gridStyleFor(l);
            return html `
            <div
              class="card-wrap ${this._editMode ? 'edit' : ''} ${this._draggingId === cs.config.id ? 'dragging' : ''} ${this._overId === cs.config.id ? 'over' : ''}"
              style="${style}"
              draggable="${this._editMode}"
              @dragstart="${(e) => this._onDragStart(cs.config.id, e)}"
              @dragover="${(e) => this._onDragOver(cs.config.id, e)}"
              @dragleave="${this._onDragLeave}"
              @drop="${(e) => this._onDrop(cs.config.id, e)}"
            >
              ${this._editMode ? html `<div class="drag-handle"></div>` : ''}
              <vlt-sensor-card .sensor="${cs}" @sensor-click="${this._onSensorClick}"></vlt-sensor-card>
            </div>
          `;
        })}
      </div>
    `;
    }
};
__decorate([
    state()
], GridView.prototype, "_sensors", void 0);
__decorate([
    state()
], GridView.prototype, "_dataMap", void 0);
__decorate([
    state()
], GridView.prototype, "_currentValues", void 0);
__decorate([
    state()
], GridView.prototype, "_screenId", void 0);
__decorate([
    state()
], GridView.prototype, "_loading", void 0);
__decorate([
    state()
], GridView.prototype, "_layouts", void 0);
__decorate([
    state()
], GridView.prototype, "_editMode", void 0);
__decorate([
    state()
], GridView.prototype, "_draggingId", void 0);
__decorate([
    state()
], GridView.prototype, "_overId", void 0);
__decorate([
    state()
], GridView.prototype, "_bp", void 0);
__decorate([
    state()
], GridView.prototype, "_containerWidth", void 0);
GridView = __decorate([
    customElement('vlt-grid-view')
], GridView);
export { GridView };
//# sourceMappingURL=grid-view.js.map