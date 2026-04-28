var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, property, query } from 'lit/decorators.js';
import { safeMinMax, toHexColor, fmtTs } from '../types.js';
let DetailChart = class DetailChart extends LitElement {
    constructor() {
        super(...arguments);
        this.data = [];
        this.color = '#22c55e';
        this.warn = 0;
        this.crit = 0;
        this.unit = '';
        this.overlays = [];
        this.pointWidth = 0;
        this._area = { pL: 60, pR: 20, pT: 20, pB: 44, cW: 0, cH: 0, yMin: 0, yMax: 0, dLen: 0 };
        this._hover = null;
    }
    static { this.styles = css `
    :host { display: block; width: 100%; height: 100%; position: relative; }
    .scroll { width: 100%; height: 100%; overflow-x: auto; overflow-y: hidden; position: relative; }
    .wrapper { position: relative; height: 100%; }
    canvas { display: block; width: 100%; height: 100%; cursor: crosshair; }
    .legend {
      position: absolute; top: 8px; right: 24px; display: flex; gap: 10; pointer-events: none; z-index: 1;
    }
    .legend-item {
      display: flex; align-items: center; gap: 4; font-size: 11px;
      font-family: 'JetBrains Mono', monospace; opacity: 0.85;
      background: rgba(17,17,19,0.8); padding: 2px 6px; border-radius: 4;
    }
    .tooltip {
      position: fixed; background: rgba(17,17,19,0.95); border: 1px solid var(--border);
      border-radius: 8px; padding: 8px 12px; pointer-events: none; z-index: 9999;
      font-family: 'JetBrains Mono', monospace; font-size: 12px; white-space: nowrap;
    }
    .tooltip-line {
      position: fixed; width: 1px; background: rgba(255,255,255,0.12);
      pointer-events: none; z-index: 9999;
    }
    .tooltip-dot {
      position: fixed; width: 10px; height: 10px; border-radius: 50%; border: 2px solid #fff;
      pointer-events: none; z-index: 9999;
    }
  `; }
    updated(changed) {
        if (changed.has('data') || changed.has('color') || changed.has('overlays') || changed.has('pointWidth')) {
            this._draw();
        }
    }
    _draw() {
        const cv = this.canvas;
        const wrapper = this.wrapperEl;
        if (!cv || !wrapper || this.data.length < 2)
            return;
        const ctx = cv.getContext('2d');
        const dpr = devicePixelRatio || 1;
        const wrapRect = wrapper.getBoundingClientRect();
        const wrapW = wrapRect.width;
        const wrapH = wrapRect.height;
        if (wrapW <= 0 || wrapH <= 0)
            return;
        const W = this.pointWidth > 0 ? Math.max(this.pointWidth * this.data.length + 80, wrapW) : wrapW;
        const H = wrapH;
        wrapper.style.width = W + 'px';
        cv.width = W * dpr;
        cv.height = H * dpr;
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        const pL = 60, pR = 20, pT = 20, pB = 44;
        const cW = W - pL - pR, cH = H - pT - pB;
        if (cW <= 0 || cH <= 0)
            return;
        const values = this.data.map(d => d.value);
        const [rawMin, rawMax] = safeMinMax(values);
        let yMin = rawMin - 2, yMax = rawMax + 2;
        if (yMax - yMin < 4) {
            yMin--;
            yMax++;
        }
        this._area = { pL, pR, pT, pB, cW, cH, yMin, yMax, dLen: this.data.length };
        const toX = (i, len) => pL + (i / (len - 1)) * cW;
        const toY = (v) => pT + cH - ((v - yMin) / (yMax - yMin)) * cH;
        ctx.clearRect(0, 0, W, H);
        // Grid
        ctx.strokeStyle = '#1a1a1e';
        ctx.lineWidth = 1;
        for (let i = 0; i <= 5; i++) {
            const y = pT + (cH / 5) * i;
            ctx.beginPath();
            ctx.moveTo(pL, y);
            ctx.lineTo(W - pR, y);
            ctx.stroke();
        }
        // Y labels
        ctx.fillStyle = '#52525b';
        ctx.font = '11px JetBrains Mono,monospace';
        ctx.textAlign = 'right';
        for (let i = 0; i <= 5; i++) {
            const v = yMin + ((yMax - yMin) / 5) * (5 - i);
            ctx.fillText(v.toFixed(1), pL - 8, pT + (cH / 5) * i + 4);
        }
        // X labels
        ctx.textAlign = 'center';
        const labelGap = Math.max(60, cW / 12);
        const pxStep = cW / (this.data.length - 1);
        const indexStep = Math.max(1, Math.ceil(labelGap / pxStep));
        for (let i = 0; i < this.data.length; i += indexStep) {
            ctx.fillText(fmtTs(this.data[i].ts), toX(i, this.data.length), H - pB + 16);
        }
        ctx.fillText(fmtTs(this.data[this.data.length - 1].ts), toX(this.data.length - 1, this.data.length), H - pB + 16);
        // Threshold lines
        ctx.setLineDash([6, 4]);
        if (this.warn >= yMin && this.warn <= yMax) {
            ctx.strokeStyle = 'rgba(234,179,8,0.4)';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(pL, toY(this.warn));
            ctx.lineTo(W - pR, toY(this.warn));
            ctx.stroke();
        }
        if (this.crit >= yMin && this.crit <= yMax) {
            ctx.strokeStyle = 'rgba(239,68,68,0.4)';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(pL, toY(this.crit));
            ctx.lineTo(W - pR, toY(this.crit));
            ctx.stroke();
        }
        ctx.setLineDash([]);
        // Overlays
        const hexColor = toHexColor(this.color);
        const validOverlays = this.overlays.map(o => ({
            ...o, data: o.data.filter(d => d.value != null && !Number.isNaN(d.value)),
            color: toHexColor(o.color)
        })).filter(o => o.data.length >= 2);
        for (const ov of validOverlays) {
            const ovVals = ov.data.map(d => d.value);
            const [ovMin, ovMax] = safeMinMax(ovVals);
            const ovRange = Math.max(ovMax - ovMin, 0.001);
            const ovToY = (v) => pT + cH - ((v - ovMin) / ovRange) * cH;
            ctx.beginPath();
            for (let i = 0; i < ov.data.length; i++) {
                const x = toX(i, ov.data.length), y = ovToY(ov.data[i].value);
                i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
            }
            ctx.strokeStyle = ov.color;
            ctx.lineWidth = 1.5;
            ctx.lineJoin = 'round';
            ctx.lineCap = 'round';
            ctx.stroke();
        }
        // Primary line
        ctx.beginPath();
        for (let i = 0; i < this.data.length; i++) {
            const x = toX(i, this.data.length), y = toY(this.data[i].value);
            i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
        }
        ctx.strokeStyle = hexColor;
        ctx.lineWidth = 2;
        ctx.lineJoin = 'round';
        ctx.lineCap = 'round';
        ctx.stroke();
        // Fill
        const lx = toX(this.data.length - 1, this.data.length);
        ctx.lineTo(lx, pT + cH);
        ctx.lineTo(toX(0, this.data.length), pT + cH);
        ctx.closePath();
        const g = ctx.createLinearGradient(0, pT, 0, pT + cH);
        g.addColorStop(0, hexColor + '25');
        g.addColorStop(1, hexColor + '00');
        ctx.fillStyle = g;
        ctx.fill();
        // Last value dot
        const lv = this.data[this.data.length - 1].value;
        const cy = toY(lv);
        ctx.beginPath();
        ctx.arc(lx, cy, 5, 0, Math.PI * 2);
        ctx.fillStyle = hexColor;
        ctx.fill();
        ctx.beginPath();
        ctx.arc(lx, cy, 9, 0, Math.PI * 2);
        ctx.strokeStyle = hexColor + '40';
        ctx.lineWidth = 2;
        ctx.stroke();
        ctx.fillStyle = hexColor;
        ctx.font = 'bold 13px JetBrains Mono,monospace';
        ctx.textAlign = 'right';
        ctx.fillText(lv.toFixed(1) + (this.unit ? ' ' + this.unit : ''), W - pR - 2, cy - 12);
    }
    _onMove(e) {
        const cv = this.canvas;
        if (!cv || this.data.length < 2) {
            this._hover = null;
            return;
        }
        const rect = cv.getBoundingClientRect();
        const mx = e.clientX - rect.left;
        const { pL, cW, dLen } = this._area;
        const idx = Math.round(((mx - pL) / cW) * (dLen - 1));
        if (idx < 0 || idx >= dLen) {
            this._hover = null;
            return;
        }
        this._hover = { idx, clientX: e.clientX, clientY: e.clientY };
        this.requestUpdate();
    }
    _onLeave() {
        this._hover = null;
        this.requestUpdate();
    }
    render() {
        const hover = this._hover;
        const data = this.data;
        const validOverlays = this.overlays.filter(o => o.data.length >= 2);
        let tooltipHtml = html ``;
        let lineHtml = html ``;
        let dotHtml = html ``;
        if (hover && data[hover.idx]) {
            const series = [];
            series.push({ name: 'Primary', val: data[hover.idx].value.toFixed(2) + (this.unit ? ' ' + this.unit : ''), color: toHexColor(this.color) });
            for (const ov of validOverlays) {
                const ovIdx = Math.min(Math.round((hover.idx / (data.length - 1)) * (ov.data.length - 1)), ov.data.length - 1);
                if (ov.data[ovIdx]) {
                    series.push({ name: ov.name, val: ov.data[ovIdx].value.toFixed(2) + (ov.unit ? ' ' + ov.unit : ''), color: toHexColor(ov.color) });
                }
            }
            const tipW = 190;
            const tipLeft = hover.clientX + 16 + tipW > window.innerWidth ? hover.clientX - tipW - 12 : hover.clientX + 16;
            const tipTop = Math.max(4, hover.clientY - 60);
            tooltipHtml = html `
        <div class="tooltip" style="left:${tipLeft}px;top:${tipTop}px">
          ${series.map((s, i) => html `
            <div style="color:${s.color};font-weight:${i === 0 ? 700 : 500};font-size:${i === 0 ? 14 : 11}px;margin-top:${i > 0 ? 2 : 0}px">
              ${series.length > 1 ? html `<span style="font-size:10px;opacity:0.7">${s.name}: </span>` : ''}${s.val}
            </div>
          `)}
          <div style="color:var(--muted);font-size:11px;margin-top:2px">${new Date(data[hover.idx].ts).toLocaleString('en-US', { hour12: false, month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' })}</div>
        </div>
      `;
            lineHtml = html `<div class="tooltip-line" style="left:${hover.clientX}px;top:0;height:100%"></div>`;
            dotHtml = html `<div class="tooltip-dot" style="left:${hover.clientX - 5}px;top:${hover.clientY - 5}px;background:${toHexColor(this.color)}"></div>`;
        }
        return html `
      <div class="scroll">
        <div class="wrapper">
          <canvas @mousemove="${this._onMove}" @mouseleave="${this._onLeave}"></canvas>
          ${validOverlays.length > 0 ? html `
            <div class="legend">
              ${validOverlays.map(ov => html `
                <div class="legend-item" style="color:${toHexColor(ov.color)}">
                  <div style="width:12px;height:2px;background:${toHexColor(ov.color)};border-radius:1px"></div>
                  ${ov.name} (${ov.unit})
                </div>
              `)}
            </div>
          ` : ''}
        </div>
      </div>
      ${lineHtml}
      ${dotHtml}
      ${tooltipHtml}
    `;
    }
};
__decorate([
    property({ type: Array })
], DetailChart.prototype, "data", void 0);
__decorate([
    property({ type: String })
], DetailChart.prototype, "color", void 0);
__decorate([
    property({ type: Number })
], DetailChart.prototype, "warn", void 0);
__decorate([
    property({ type: Number })
], DetailChart.prototype, "crit", void 0);
__decorate([
    property({ type: String })
], DetailChart.prototype, "unit", void 0);
__decorate([
    property({ type: Array })
], DetailChart.prototype, "overlays", void 0);
__decorate([
    property({ type: Number })
], DetailChart.prototype, "pointWidth", void 0);
__decorate([
    query('canvas')
], DetailChart.prototype, "canvas", void 0);
__decorate([
    query('.scroll')
], DetailChart.prototype, "scrollEl", void 0);
__decorate([
    query('.wrapper')
], DetailChart.prototype, "wrapperEl", void 0);
DetailChart = __decorate([
    customElement('vlt-detail-chart')
], DetailChart);
export { DetailChart };
//# sourceMappingURL=detail-chart.js.map