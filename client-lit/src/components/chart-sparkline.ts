import { LitElement, html, css } from 'lit';
import { customElement, property, query } from 'lit/decorators.js';
import { safeMinMax, toHexColor } from '../types.js';

@customElement('vlt-chart-sparkline')
export class ChartSparkline extends LitElement {
  static styles = css`
    :host { display: block; width: 100%; height: 100%; min-height: 0; }
    canvas { display: block; width: 100%; height: 100%; min-height: 0; }
  `;

  @property({ type: Array })
  data: number[] = [];

  @property({ type: String })
  color = '#22c55e';

  @property({ type: Number })
  warn = 0;

  @property({ type: Number })
  crit = 0;

  @query('canvas')
  canvas!: HTMLCanvasElement;

  updated(changed: Map<string, any>) {
    if (changed.has('data') || changed.has('color') || changed.has('warn') || changed.has('crit')) {
      this._draw();
    }
  }

  private _draw() {
    const cv = this.canvas;
    if (!cv || this.data.length < 2) return;
    const ctx = cv.getContext('2d')!;
    const dpr = devicePixelRatio || 1;
    const rect = cv.getBoundingClientRect();
    cv.width = rect.width * dpr;
    cv.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const W = rect.width, H = rect.height;
    const pL = 40, pR = 10, pT = 10, pB = 24;
    const cW = W - pL - pR, cH = H - pT - pB;

    ctx.clearRect(0, 0, W, H);

    // Grid lines
    ctx.strokeStyle = '#1a1a1e'; ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i++) {
      const y = pT + (cH / 4) * i;
      ctx.beginPath(); ctx.moveTo(pL, y); ctx.lineTo(W - pR, y); ctx.stroke();
    }

    const [rawMin, rawMax] = safeMinMax(this.data);
    let yMin = rawMin - 2, yMax = rawMax + 2;
    if (yMax - yMin < 4) { yMin--; yMax++; }

    const toX = (i: number) => pL + (i / (this.data.length - 1)) * cW;
    const toY = (v: number) => pT + cH - ((v - yMin) / (yMax - yMin)) * cH;

    // Y-axis labels
    ctx.fillStyle = '#52525b'; ctx.font = '10px JetBrains Mono,monospace'; ctx.textAlign = 'right';
    for (let i = 0; i <= 4; i++) {
      const v = yMin + ((yMax - yMin) / 4) * (4 - i);
      ctx.fillText(v.toFixed(1), pL - 6, pT + (cH / 4) * i + 3);
    }

    // Threshold lines
    ctx.setLineDash([4, 4]);
    if (this.warn >= yMin && this.warn <= yMax) {
      ctx.strokeStyle = 'rgba(234,179,8,0.3)'; ctx.beginPath();
      ctx.moveTo(pL, toY(this.warn)); ctx.lineTo(W - pR, toY(this.warn)); ctx.stroke();
    }
    if (this.crit >= yMin && this.crit <= yMax) {
      ctx.strokeStyle = 'rgba(239,68,68,0.3)'; ctx.beginPath();
      ctx.moveTo(pL, toY(this.crit)); ctx.lineTo(W - pR, toY(this.crit)); ctx.stroke();
    }
    ctx.setLineDash([]);

    // Line
    const hexColor = toHexColor(this.color.replace(/"/g, ''));
    ctx.beginPath();
    for (let i = 0; i < this.data.length; i++) {
      const x = toX(i), y = toY(this.data[i]);
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    }
    ctx.strokeStyle = hexColor; ctx.lineWidth = 2; ctx.lineJoin = 'round'; ctx.lineCap = 'round';
    ctx.stroke();

    // Fill gradient
    const lx = toX(this.data.length - 1), fx = toX(0);
    ctx.lineTo(lx, pT + cH); ctx.lineTo(fx, pT + cH); ctx.closePath();
    const g = ctx.createLinearGradient(0, pT, 0, pT + cH);
    g.addColorStop(0, hexColor + '30'); g.addColorStop(1, hexColor + '00');
    ctx.fillStyle = g; ctx.fill();

    // Last value dot
    const lv = this.data[this.data.length - 1], cy = toY(lv);
    ctx.beginPath(); ctx.arc(lx, cy, 4, 0, Math.PI * 2); ctx.fillStyle = hexColor; ctx.fill();
    ctx.beginPath(); ctx.arc(lx, cy, 7, 0, Math.PI * 2); ctx.strokeStyle = hexColor + '40'; ctx.lineWidth = 2; ctx.stroke();

    // Last value label
    ctx.fillStyle = hexColor; ctx.font = 'bold 11px JetBrains Mono,monospace'; ctx.textAlign = 'right';
    ctx.fillText(lv.toFixed(1), W - pR - 2, cy - 8);
  }

  render() {
    return html`<canvas></canvas>`;
  }
}
