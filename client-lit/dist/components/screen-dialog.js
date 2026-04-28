var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
let PromptDialog = class PromptDialog extends LitElement {
    constructor() {
        super(...arguments);
        this._value = '';
        this.title = 'Prompt';
        this.label = 'Value';
        this.placeholder = '';
        this.initialValue = '';
        this.okLabel = 'OK';
    }
    static { this.styles = css `
    :host { position: fixed; inset: 0; background: rgba(0,0,0,.6); display: flex; align-items: center; justify-content: center; z-index: 60; }
    .modal { background: var(--card); border: 1px solid var(--border); border-radius: 10px; width: 380px; padding: 20px; }
    .title { font-size: 15px; font-weight: 600; margin-bottom: 16px; color: var(--fg); }
    label { display: flex; flex-direction: column; gap: 4px; }
    .label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
    input { background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 6px 8px; color: var(--fg); font-size: 13px; font-family: inherit; outline: none; width: 100%; box-sizing: border-box; }
    .footer { display: flex; gap: 8; justify-content: flex-end; margin-top: 16px; }
    .footer button { padding: 7px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; font-family: inherit; }
    .ok { border: 1px solid var(--cyan); background: transparent; color: var(--cyan); font-weight: 600; }
    .cancel { border: 1px solid var(--border); background: transparent; color: var(--fg); }
  `; }
    connectedCallback() {
        super.connectedCallback();
        this._value = this.initialValue || '';
    }
    _submit() {
        const v = this._value.trim();
        if (!v)
            return;
        this.dispatchEvent(new CustomEvent('submit', { detail: v }));
    }
    _close() {
        this.dispatchEvent(new CustomEvent('close'));
    }
    render() {
        return html `
      <div @click="${this._close}">
        <div class="modal" @click="${(e) => e.stopPropagation()}">
          <div class="title">${this.title}</div>
          <label>
            <span class="label">${this.label}</span>
            <input .value="${this._value}" @input="${(e) => this._value = e.target.value}" placeholder="${this.placeholder}" autofocus @keydown="${(e) => e.key === 'Enter' && this._submit()}" />
          </label>
          <div class="footer">
            <button class="cancel" @click="${this._close}">Cancel</button>
            <button class="ok" @click="${this._submit}">${this.okLabel}</button>
          </div>
        </div>
      </div>
    `;
    }
};
__decorate([
    state()
], PromptDialog.prototype, "_value", void 0);
PromptDialog = __decorate([
    customElement('vlt-prompt-dialog')
], PromptDialog);
export { PromptDialog };
let ConfirmDialog = class ConfirmDialog extends LitElement {
    constructor() {
        super(...arguments);
        this.title = 'Confirm';
        this.message = 'Are you sure?';
        this.confirmLabel = 'Delete';
    }
    static { this.styles = css `
    :host { position: fixed; inset: 0; background: rgba(0,0,0,.6); display: flex; align-items: center; justify-content: center; z-index: 60; }
    .modal { background: var(--card); border: 1px solid var(--border); border-radius: 10px; width: 380px; padding: 20px; }
    .title { font-size: 15px; font-weight: 600; margin-bottom: 12px; color: var(--fg); }
    .msg { font-size: 13px; color: var(--muted); margin-bottom: 20px; }
    .footer { display: flex; gap: 8; justify-content: flex-end; }
    .footer button { padding: 7px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; font-family: inherit; }
    .confirm { border: 1px solid var(--crit); background: var(--crit); color: #fff; font-weight: 600; }
    .cancel { border: 1px solid var(--border); background: transparent; color: var(--fg); }
  `; }
    _confirm() {
        this.dispatchEvent(new CustomEvent('confirm'));
    }
    _close() {
        this.dispatchEvent(new CustomEvent('close'));
    }
    render() {
        return html `
      <div @click="${this._close}">
        <div class="modal" @click="${(e) => e.stopPropagation()}">
          <div class="title">${this.title}</div>
          <div class="msg">${this.message}</div>
          <div class="footer">
            <button class="cancel" @click="${this._close}">Cancel</button>
            <button class="confirm" @click="${this._confirm}">${this.confirmLabel}</button>
          </div>
        </div>
      </div>
    `;
    }
};
ConfirmDialog = __decorate([
    customElement('vlt-confirm-dialog')
], ConfirmDialog);
export { ConfirmDialog };
//# sourceMappingURL=screen-dialog.js.map