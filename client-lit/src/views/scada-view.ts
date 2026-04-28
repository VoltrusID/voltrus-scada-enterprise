import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { api } from '../services/api.js';

@customElement('vlt-scada-view')
export class ScadaView extends LitElement {
  static styles = css`
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
  `;

  @state() private _editMode = false;
  @state() private _tool: 'select' | 'connect' | 'delete' = 'select';

  render() {
    return html`
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
}
