import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';

@customElement('vlt-prompt-dialog')
export class PromptDialog extends LitElement {
  static styles = css`
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
  `;

  @state() private _value = '';
  title = 'Prompt';
  label = 'Value';
  placeholder = '';
  initialValue = '';
  okLabel = 'OK';

  connectedCallback() {
    super.connectedCallback();
    this._value = this.initialValue || '';
  }

  private _submit() {
    const v = this._value.trim();
    if (!v) return;
    this.dispatchEvent(new CustomEvent('submit', { detail: v }));
  }

  private _close() {
    this.dispatchEvent(new CustomEvent('close'));
  }

  render() {
    return html`
      <div @click="${this._close}">
        <div class="modal" @click="${(e: Event) => e.stopPropagation()}">
          <div class="title">${this.title}</div>
          <label>
            <span class="label">${this.label}</span>
            <input .value="${this._value}" @input="${(e: InputEvent) => this._value = (e.target as HTMLInputElement).value}" placeholder="${this.placeholder}" autofocus @keydown="${(e: KeyboardEvent) => e.key === 'Enter' && this._submit()}" />
          </label>
          <div class="footer">
            <button class="cancel" @click="${this._close}">Cancel</button>
            <button class="ok" @click="${this._submit}">${this.okLabel}</button>
          </div>
        </div>
      </div>
    `;
  }
}

@customElement('vlt-confirm-dialog')
export class ConfirmDialog extends LitElement {
  static styles = css`
    :host { position: fixed; inset: 0; background: rgba(0,0,0,.6); display: flex; align-items: center; justify-content: center; z-index: 60; }
    .modal { background: var(--card); border: 1px solid var(--border); border-radius: 10px; width: 380px; padding: 20px; }
    .title { font-size: 15px; font-weight: 600; margin-bottom: 12px; color: var(--fg); }
    .msg { font-size: 13px; color: var(--muted); margin-bottom: 20px; }
    .footer { display: flex; gap: 8; justify-content: flex-end; }
    .footer button { padding: 7px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; font-family: inherit; }
    .confirm { border: 1px solid var(--crit); background: var(--crit); color: #fff; font-weight: 600; }
    .cancel { border: 1px solid var(--border); background: transparent; color: var(--fg); }
  `;

  title = 'Confirm';
  message = 'Are you sure?';
  confirmLabel = 'Delete';

  private _confirm() {
    this.dispatchEvent(new CustomEvent('confirm'));
  }

  private _close() {
    this.dispatchEvent(new CustomEvent('close'));
  }

  render() {
    return html`
      <div @click="${this._close}">
        <div class="modal" @click="${(e: Event) => e.stopPropagation()}">
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
}
