var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
let ScadaView = class ScadaView extends LitElement {
    constructor() {
        super(...arguments);
        this._editMode = false;
        this._tool = 'select';
    }
    static { this.styles = css `
    :host { display: block; height: 100%; padding: 20px; overflow: auto; }
    .toolbar {
      display: flex; align-items: center; gap: 8px; margin-bottom: 12px;
    }
    .toolbar button {
      padding: 6px 12px; border-radius: 6px; border: 1px solid var(--border);
      background: var(--card); color: var(--fg); cursor: pointer; font-family: inherit; font-size: 12px;
    }
    .toolbar button.active { background: rgba(34,197,94,.1); color: var(--cyan); border-color: var(--cyan); }
    .canvas {
      position: relative; width: 100%; height: calc(100% - 60px);
      background: var(--card); border: 1px solid var(--border); border-radius: 10px;
    }
    .placeholder {
      display: flex; align-items: center; justify-content: center;
      height: 100%; color: var(--muted); font-size: 14px;
    }
  `; }
    render() {
        return html `
      <div class="toolbar">
        <button class="${this._tool === 'select' ? 'active' : ''}" @click="${() => this._tool = 'select'}">Select</button>
        <button class="${this._tool === 'connect' ? 'active' : ''}" @click="${() => this._tool = 'connect'}">Connect</button>
        <button class="${this._tool === 'delete' ? 'active' : ''}" @click="${() => this._tool = 'delete'}">Delete</button>
        <button @click="${() => this._editMode = !this._editMode}">${this._editMode ? 'Done' : 'Edit'}</button>
      </div>
      <div class="canvas">
        <div class="placeholder">
          2D SCADA Diagram Editor — equipment palette, drag/drop, and connections coming soon.
          <br>Use the React client for full SCADA editing.
        </div>
      </div>
    `;
    }
};
__decorate([
    state()
], ScadaView.prototype, "_editMode", void 0);
__decorate([
    state()
], ScadaView.prototype, "_tool", void 0);
ScadaView = __decorate([
    customElement('vlt-scada-view')
], ScadaView);
export { ScadaView };
//# sourceMappingURL=scada-view.js.map