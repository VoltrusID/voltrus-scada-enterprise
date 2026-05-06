import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { authManager } from "../services/auth.js";

@customElement("vlt-login-screen")
export class LoginScreen extends LitElement {
  static styles = css`
    :host {
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
      background: var(--bg);
    }
    form {
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 32px;
      width: 340px;
      display: flex;
      flex-direction: column;
      gap: 12px;
    }
    .header {
      display: flex;
      align-items: center;
      gap: 10;
      margin-bottom: 12px;
    }
    .header img {
      width: 36px;
      height: 36px;
      border-radius: 8px;
      object-fit: contain;
    }
    .header span {
      font-size: 18px;
      font-weight: 700;
      color: var(--fg);
    }
    label {
      display: flex;
      flex-direction: column;
      gap: 4px;
    }
    .label-text {
      font-size: 11px;
      color: var(--muted);
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }
    input {
      background: var(--bg);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 6px 8px;
      color: var(--fg);
      font-size: 13px;
      font-family: inherit;
      outline: none;
      width: 100%;
      box-sizing: border-box;
    }
    input:focus {
      border-color: var(--border-light);
    }
    .error {
      color: var(--crit);
      font-size: 12px;
    }
    button[type="submit"] {
      justify-content: center;
      width: 100%;
      margin-top: 4px;
      background: var(--cyan);
      color: #000;
      border-color: var(--cyan);
      font-weight: 600;
      cursor: pointer;
      font-family: inherit;
      font-size: 13px;
      padding: 7px 14px;
      border-radius: 6px;
      border: 1px solid var(--cyan);
    }
  `;

  @state() private _user = "";
  @state() private _pass = "";
  @state() private _error = "";

  connectedCallback() {
    super.connectedCallback();
    authManager.addEventListener("change", this._onAuthChange as EventListener);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    authManager.removeEventListener(
      "change",
      this._onAuthChange as EventListener,
    );
  }

  private _onAuthChange = (e: CustomEvent) => {
    this._error = e.detail.error || "";
  };

  private _submit(e: Event) {
    e.preventDefault();
    authManager.login(this._user, this._pass);
  }

  render() {
    return html`
      <form @submit="${this._submit}">
        <div class="header">
          <img src="favicon-32.png" alt="Voltrus" />
          <span>Voltrus</span>
        </div>
        <label>
          <span class="label-text">Username</span>
          <input
            .value="${this._user}"
            @input="${(e: InputEvent) =>
              (this._user = (e.target as HTMLInputElement).value)}"
            autocomplete="username"
            autofocus
          />
        </label>
        <label>
          <span class="label-text">Password</span>
          <input
            type="password"
            .value="${this._pass}"
            @input="${(e: InputEvent) =>
              (this._pass = (e.target as HTMLInputElement).value)}"
            autocomplete="current-password"
          />
        </label>
        ${this._error ? html`<div class="error">${this._error}</div>` : ""}
        <button type="submit">Sign In</button>
      </form>
    `;
  }
}
