import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { alarmAudio } from "../services/alarm-audio.js";
import { api, fetchOk } from "../services/api.js";
import { store } from "../services/store.js";
import type { SensorConfig, BrandingSettings } from "../types.js";

@customElement("vlt-dashboard-settings")
export class DashboardSettings extends LitElement {
  static styles = css`
    :host {
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.6);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 100;
    }
    .modal {
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 0;
      min-width: 480px;
      max-width: 600px;
      max-height: 80vh;
      display: flex;
      flex-direction: column;
    }
    .header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 16px 20px;
      border-bottom: 1px solid var(--border);
    }
    .title {
      font-size: 15px;
      font-weight: 600;
      color: var(--fg);
    }
    .tabs {
      display: flex;
      gap: 0;
      border-bottom: 1px solid var(--border);
    }
    .tabs button {
      flex: 1;
      padding: 10px;
      font-size: 12px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      background: transparent;
      border: none;
      border-bottom: 2px solid transparent;
      color: var(--muted);
      cursor: pointer;
      font-family: inherit;
    }
    .tabs button.active {
      border-bottom-color: var(--cyan);
      color: var(--cyan);
    }
    .body {
      padding: 20px;
      overflow-y: auto;
      flex: 1;
    }
    .row {
      display: flex;
      flex-direction: column;
      gap: 8px;
      margin-bottom: 14px;
    }
    .label {
      font-size: 11px;
      color: var(--muted);
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }
    input[type="text"],
    input[type="url"] {
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
    input[type="range"] {
      width: 100%;
    }
    .value {
      font-size: 11px;
      font-family: "JetBrains Mono", monospace;
    }
    .agg-btns {
      display: flex;
      gap: 4px;
      margin-top: 4px;
    }
    .agg-btns button {
      flex: 1;
      padding: 8px 6px;
      font-size: 11px;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: transparent;
      color: var(--fg);
      cursor: pointer;
      font-family: inherit;
    }
    .agg-btns button.active {
      border-color: var(--warn);
      color: var(--warn);
      background: rgba(234, 179, 8, 0.08);
      font-weight: 600;
    }
    .desc {
      font-size: 11px;
      color: var(--muted);
      margin-top: 6px;
      line-height: 1.4;
    }
    .mute-btn {
      padding: 6px 14px;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: transparent;
      color: var(--fg);
      cursor: pointer;
      font-family: inherit;
      font-size: 13px;
      display: flex;
      align-items: center;
      gap: 6px;
      width: fit-content;
    }
    .mute-btn.muted {
      border-color: var(--warn);
      background: rgba(234, 179, 8, 0.1);
      color: var(--warn);
    }
    .close-btn {
      margin: 0 20px 20px;
      padding: 8px 16px;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: transparent;
      color: var(--fg);
      cursor: pointer;
      font-family: inherit;
      font-size: 13px;
    }
    .shortcut-row {
      display: grid;
      grid-template-columns: 1fr 80px;
      gap: 8px;
      align-items: center;
      margin-bottom: 8px;
    }
    .shortcut-row input {
      background: var(--bg);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 5px 8px;
      color: var(--fg);
      font-size: 13px;
      font-family: inherit;
      outline: none;
      text-align: center;
      text-transform: uppercase;
    }
    .shortcut-row span {
      font-size: 13px;
      color: var(--fg);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .save-btn {
      margin-top: 12px;
      padding: 8px 16px;
      border-radius: 6px;
      border: 1px solid var(--cyan);
      background: transparent;
      color: var(--cyan);
      cursor: pointer;
      font-family: inherit;
      font-size: 13px;
      font-weight: 600;
    }
    .section-title {
      font-size: 12px;
      font-weight: 600;
      color: var(--muted);
      margin: 16px 0 8px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
  `;

  @state() private _tab: "general" | "branding" | "shortcuts" | "apikeys" =
    "general";
  @state() private _gap = 12;
  @state() private _refresh = 1000;
  @state() private _agg: "sampling" | "avg" | "minmax" = "sampling";
  @state() private _muted = false;
  @state() private _branding: BrandingSettings = {
    title: "Voltrus",
    subtitle: "Industrial monitoring",
    logo: "",
  };
  @state() private _sensors: SensorConfig[] = [];
  @state() private _shortcuts: Record<number, string> = {};
  @state() private _navShortcuts: Record<string, string> = {};
  @state() private _saving = false;
  @state() private _apiKeys: Array<{
    id: number;
    name: string;
    key_prefix: string;
    created_at: number;
    expires_at: number | null;
  }> = [];
  @state() private _newKeyName = "";
  @state() private _newKeyExpiry = 365;
  @state() private _generatedKey = "";
  @state() private _apiKeyError = "";

  private _navActions = [
    { key: "grid", label: "Grid View" },
    { key: "canvas", label: "Canvas View" },
    { key: "scada", label: "2D SCADA" },
    { key: "twin", label: "3D Twin" },
    { key: "system", label: "System" },
    { key: "alarms", label: "Alarms" },
    { key: "settings", label: "Settings" },
    { key: "data", label: "Data Manager" },
  ];

  connectedCallback() {
    super.connectedCallback();
    this._muted = alarmAudio.isMuted;
    this._load();
  }

  private async _load() {
    try {
      this._branding = await api<BrandingSettings>("/settings/branding");
      this._sensors = await api<SensorConfig[]>("/data-sources");
      this._shortcuts = await api<Record<number, string>>("/shortcuts");
      this._navShortcuts = await api<Record<string, string>>("/nav-shortcuts");
    } catch {
      /* ignore */
    }
    await this._loadApiKeys();
  }

  private async _loadApiKeys() {
    try {
      const res = await api<{
        keys: Array<{
          id: number;
          name: string;
          key_prefix: string;
          created_at: number;
          expires_at: number | null;
        }>;
      }>("/api-keys");
      this._apiKeys = res?.keys || [];
    } catch {
      this._apiKeys = [];
    }
  }

  private async _createApiKey() {
    this._apiKeyError = "";
    this._generatedKey = "";
    if (!this._newKeyName.trim()) {
      this._apiKeyError = "Name is required";
      return;
    }
    try {
      const body: any = {
        name: this._newKeyName.trim(),
        permissions: '["read"]',
      };
      if (this._newKeyExpiry > 0) {
        body.expires_at =
          Math.floor(Date.now() / 1000) + this._newKeyExpiry * 86400;
      }
      const res = await api<{ key: string; ok: boolean }>("/api-keys", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      this._generatedKey = res.key || "";
      this._newKeyName = "";
      await this._loadApiKeys();
    } catch (e: any) {
      this._apiKeyError = e?.message || "Failed to create key";
    }
  }

  private async _revokeApiKey(id: number) {
    if (!confirm("Revoke this API key? It will stop working immediately."))
      return;
    try {
      await fetchOk(`/api/v1/api-keys/${id}`, { method: "DELETE" });
      await this._loadApiKeys();
    } catch {
      /* ignore */
    }
  }

  private _setAgg(a: "sampling" | "avg" | "minmax") {
    this._agg = a;
  }

  private _toggleMute() {
    this._muted = alarmAudio.toggleMute();
  }

  private async _saveBranding() {
    try {
      await fetchOk("/api/v1/settings/branding", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(this._branding),
      });
      store.setBranding(this._branding);
    } catch {
      /* ignore */
    }
  }

  private async _saveShortcuts() {
    this._saving = true;
    try {
      await fetchOk("/api/v1/shortcuts", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(this._shortcuts),
      });
      await fetchOk("/api/v1/nav-shortcuts", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(this._navShortcuts),
      });
      store.setShortcuts(this._shortcuts);
      store.setNavShortcuts(this._navShortcuts);
    } catch {
      /* ignore */
    }
    this._saving = false;
  }

  private _updateShortcut(id: number, val: string) {
    const v = val.trim().toLowerCase();
    if (!v) {
      const copy = { ...this._shortcuts };
      delete copy[id];
      this._shortcuts = copy;
    } else {
      this._shortcuts = { ...this._shortcuts, [id]: v };
    }
  }

  private _updateNavShortcut(action: string, val: string) {
    const v = val.trim().toLowerCase();
    if (!v) {
      const copy = { ...this._navShortcuts };
      delete copy[action];
      this._navShortcuts = copy;
    } else {
      this._navShortcuts = { ...this._navShortcuts, [action]: v };
    }
  }

  private _close() {
    this.dispatchEvent(new CustomEvent("close"));
  }

  render() {
    const aggOptions = [
      {
        value: "sampling" as const,
        label: "Sampling",
        desc: "Pick evenly-spaced points. Fastest, but skips data between intervals.",
      },
      {
        value: "avg" as const,
        label: "Average",
        desc: "Average all values within each interval bucket. Smooth curve, hides peaks.",
      },
      {
        value: "minmax" as const,
        label: "Min/Max/Avg",
        desc: "Shows average line with min/max band. Best overview — no data is lost.",
      },
    ];

    return html`
      <div @click="${this._close}">
        <div class="modal" @click="${(e: Event) => e.stopPropagation()}">
          <div class="header">
            <div class="title">Dashboard Settings</div>
            <button
              style="background:transparent;border:none;color:var(--muted);cursor:pointer;font-size:18px;line-height:1"
              @click="${this._close}"
            >
              ×
            </button>
          </div>
          <div class="tabs">
            <button
              class="${this._tab === "general" ? "active" : ""}"
              @click="${() => (this._tab = "general")}"
            >
              General
            </button>
            <button
              class="${this._tab === "branding" ? "active" : ""}"
              @click="${() => (this._tab = "branding")}"
            >
              Branding
            </button>
            <button
              class="${this._tab === "shortcuts" ? "active" : ""}"
              @click="${() => (this._tab = "shortcuts")}"
            >
              Shortcuts
            </button>
            <button
              class="${this._tab === "apikeys" ? "active" : ""}"
              @click="${() => {
                this._tab = "apikeys";
                this._loadApiKeys();
              }}"
            >
              API Keys
            </button>
          </div>
          <div class="body">
            ${this._tab === "general"
              ? html`
                  <div class="row">
                    <span class="label">Grid Gap (px)</span>
                    <input
                      type="range"
                      min="4"
                      max="24"
                      .value="${this._gap}"
                      @input="${(e: InputEvent) =>
                        (this._gap = parseInt(
                          (e.target as HTMLInputElement).value,
                        ))}"
                    />
                    <span class="value">${this._gap}px</span>
                  </div>
                  <div class="row">
                    <span class="label">History Auto Mode</span>
                    <div class="agg-btns">
                      ${aggOptions.map(
                        (opt) => html`
                          <button
                            class="${this._agg === opt.value ? "active" : ""}"
                            @click="${() => this._setAgg(opt.value)}"
                          >
                            ${opt.label}
                          </button>
                        `,
                      )}
                    </div>
                    <div class="desc">
                      ${aggOptions.find((o) => o.value === this._agg)?.desc}
                    </div>
                  </div>
                  <div class="row">
                    <span class="label">Mute Alarms</span>
                    <button
                      class="mute-btn ${this._muted ? "muted" : ""}"
                      @click="${this._toggleMute}"
                    >
                      ${this._muted ? "Muted" : "Active"}
                    </button>
                  </div>
                `
              : ""}
            ${this._tab === "branding"
              ? html`
                  <div class="row">
                    <span class="label">Title</span>
                    <input
                      type="text"
                      .value="${this._branding.title}"
                      @input="${(e: InputEvent) =>
                        (this._branding = {
                          ...this._branding,
                          title: (e.target as HTMLInputElement).value,
                        })}"
                    />
                  </div>
                  <div class="row">
                    <span class="label">Subtitle</span>
                    <input
                      type="text"
                      .value="${this._branding.subtitle}"
                      @input="${(e: InputEvent) =>
                        (this._branding = {
                          ...this._branding,
                          subtitle: (e.target as HTMLInputElement).value,
                        })}"
                    />
                  </div>
                  <div class="row">
                    <span class="label">Logo URL</span>
                    <input
                      type="url"
                      .value="${this._branding.logo}"
                      @input="${(e: InputEvent) =>
                        (this._branding = {
                          ...this._branding,
                          logo: (e.target as HTMLInputElement).value,
                        })}"
                      placeholder="https://..."
                    />
                  </div>
                  <button class="save-btn" @click="${this._saveBranding}">
                    Save Branding
                  </button>
                `
              : ""}
            ${this._tab === "shortcuts"
              ? html`
                  <div class="section-title">Navigation Shortcuts</div>
                  ${this._navActions.map(
                    (a) => html`
                      <div class="shortcut-row">
                        <span title="${a.label}">${a.label}</span>
                        <input
                          type="text"
                          maxlength="1"
                          .value="${this._navShortcuts[a.key] || ""}"
                          @input="${(e: InputEvent) =>
                            this._updateNavShortcut(
                              a.key,
                              (e.target as HTMLInputElement).value,
                            )}"
                        />
                      </div>
                    `,
                  )}
                  <div class="section-title">Sensor Shortcuts</div>
                  ${this._sensors.slice(0, 20).map(
                    (s) => html`
                      <div class="shortcut-row">
                        <span title="${s.name}">${s.name}</span>
                        <input
                          type="text"
                          maxlength="1"
                          .value="${this._shortcuts[s.id] || ""}"
                          @input="${(e: InputEvent) =>
                            this._updateShortcut(
                              s.id,
                              (e.target as HTMLInputElement).value,
                            )}"
                        />
                      </div>
                    `,
                  )}
                  ${this._sensors.length > 20
                    ? html`<div style="font-size:11px;color:var(--muted)">
                        Showing first 20 sensors.
                      </div>`
                    : ""}
                  <button
                    class="save-btn"
                    @click="${this._saveShortcuts}"
                    .disabled="${this._saving}"
                  >
                    ${this._saving ? "Saving..." : "Save Shortcuts"}
                  </button>
                `
              : ""}
            ${this._tab === "apikeys"
              ? html`
                  <div class="section-title">Create New API Key</div>
                  <div class="row">
                    <span class="label">Key Name</span>
                    <input
                      type="text"
                      .value="${this._newKeyName}"
                      @input="${(e: InputEvent) =>
                        (this._newKeyName = (
                          e.target as HTMLInputElement
                        ).value)}"
                      placeholder="e.g. Grafana Integration"
                    />
                  </div>
                  <div class="row">
                    <span class="label">Expires In (days)</span>
                    <input
                      type="text"
                      .value="${String(this._newKeyExpiry)}"
                      @input="${(e: InputEvent) =>
                        (this._newKeyExpiry =
                          parseInt((e.target as HTMLInputElement).value) ||
                          365)}"
                    />
                  </div>
                  ${this._apiKeyError
                    ? html`<div
                        style="font-size:11px;color:var(--red);margin-bottom:8px"
                      >
                        ${this._apiKeyError}
                      </div>`
                    : ""}
                  ${this._generatedKey
                    ? html`
                        <div
                          style="background:var(--bg);border:1px solid var(--warn);border-radius:6px;padding:10px;margin-bottom:12px"
                        >
                          <div
                            style="font-size:11px;color:var(--warn);margin-bottom:4px;font-weight:600"
                          >
                            ⚠️ COPY THIS KEY NOW — IT WILL NOT BE SHOWN AGAIN
                          </div>
                          <div
                            style="font-size:12px;font-family:'JetBrains Mono',monospace;word-break:break-all;color:var(--fg)"
                          >
                            ${this._generatedKey}
                          </div>
                        </div>
                      `
                    : ""}
                  <button class="save-btn" @click="${this._createApiKey}">
                    Generate API Key
                  </button>
                  ${this._apiKeys.length > 0
                    ? html`
                        <div class="section-title">
                          Active API Keys (${this._apiKeys.length})
                        </div>
                        ${this._apiKeys.map(
                          (k) => html`
                            <div
                              style="display:flex;align-items:center;justify-content:space-between;padding:8px 0;border-bottom:1px solid var(--border)"
                            >
                              <div>
                                <div style="font-size:13px;color:var(--fg)">
                                  ${k.name}
                                </div>
                                <div style="font-size:10px;color:var(--muted)">
                                  ${k.key_prefix}... · Created
                                  ${new Date(
                                    k.created_at * 1000,
                                  ).toLocaleDateString()}${k.expires_at
                                    ? " · Expires " +
                                      new Date(
                                        k.expires_at * 1000,
                                      ).toLocaleDateString()
                                    : " · No expiry"}
                                </div>
                              </div>
                              <button
                                @click="${() => this._revokeApiKey(k.id)}"
                                style="background:transparent;border:1px solid var(--red);color:var(--red);border-radius:4px;padding:4px 8px;cursor:pointer;font-size:11px;font-family:inherit"
                              >
                                Revoke
                              </button>
                            </div>
                          `,
                        )}
                      `
                    : html`<div
                        style="font-size:11px;color:var(--muted);margin-top:12px"
                      >
                        No API keys created yet.
                      </div>`}
                `
              : ""}
          </div>
          <button class="close-btn" @click="${this._close}">Close</button>
        </div>
      </div>
    `;
  }
}
