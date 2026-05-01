import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';

@customElement('vlt-license-activation')
export class LicenseActivationScreen extends LitElement {
  static styles = css`
    :host { display: flex; align-items: center; justify-content: center; min-height: 100vh; background: var(--bg); font-family: Inter, sans-serif; }
    .card {
      background: var(--card); border: 1px solid var(--border); border-radius: 12px;
      padding: 40px; max-width: 560px; width: 90%; box-shadow: 0 4px 24px rgba(0,0,0,0.08);
    }
    h1 { margin: 0 0 8px; font-size: 22px; color: var(--fg); }
    p { margin: 0 0 24px; font-size: 14px; color: var(--muted); line-height: 1.5; }
    ol { margin: 0 0 24px; padding-left: 20px; color: var(--fg); font-size: 14px; line-height: 1.7; }
    li { margin-bottom: 8px; }
    code {
      background: var(--bg); border: 1px solid var(--border); border-radius: 4px;
      padding: 2px 6px; font-family: 'JetBrains Mono', monospace; font-size: 12px; color: var(--fg);
    }
    .cmd {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px;
      padding: 10px 12px; font-family: 'JetBrains Mono', monospace; font-size: 12px;
      color: var(--fg); margin: 6px 0; display: block; word-break: break-all;
    }
    button {
      background: var(--accent); color: #fff; border: none; border-radius: 8px;
      padding: 10px 20px; font-size: 14px; font-weight: 600; cursor: pointer;
    }
    button:hover { opacity: 0.9; }
    .error { color: var(--crit); font-size: 13px; margin-top: 12px; }
  `;

  @state() private _status: any = null;
  @state() private _loading = true;
  @state() private _error = '';

  connectedCallback() {
    super.connectedCallback();
    this._checkLicense();
  }

  private async _checkLicense() {
    try {
      const res = await fetch('/api/license/status');
      const data = await res.json();
      this._status = data;
      this._loading = false;
      if (data.status === 'Valid') {
        this.dispatchEvent(new CustomEvent('activated', { bubbles: true, composed: true }));
      }
    } catch (e) {
      this._error = 'Failed to check license status. Please try again.';
      this._loading = false;
    }
  }

  render() {
    if (this._loading) {
      return html`<div class="loading">Checking license...</div>`;
    }

    if (this._status?.status === 'Valid') {
      return html``;
    }

    return html`
      <div class="card">
        <h1>Welcome to Voltrus Lite</h1>
        <p>Your license is not activated. Follow the steps below to activate your installation.</p>

        <ol>
          <li>Run this command on your server terminal:
            <code class="cmd">./voltrus-lite --generate-request YOUR_LICENSE_KEY</code>
          </li>
          <li>Copy the generated <code>license-request.bin</code> to a PC with internet access.</li>
          <li>Upload it at <a href="https://activate.voltrus.id" target="_blank">activate.voltrus.id</a>.</li>
          <li>Copy the returned <code>license-activation.bin</code> back to your server.</li>
          <li>Run this command to activate:
            <code class="cmd">./voltrus-lite --import-activation license-activation.bin</code>
          </li>
          <li>Refresh this page or click "Check Again" below.</li>
        </ol>

        <button @click="${this._checkLicense}">Check Again</button>
        ${this._error ? html`<div class="error">${this._error}</div>` : ''}
        ${this._status?.message ? html`<div class="error">${this._status.message}</div>` : ''}
      </div>
    `;
  }
}
