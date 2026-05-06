/**
 * Tiny observable store for shared UI state.
 * No external deps. EventTarget-based reactivity.
 */
class Store extends EventTarget {
  private _branding = {
    title: "Voltrus",
    subtitle: "Industrial monitoring",
    logo: "",
  };
  private _shortcuts: Record<number, string> = {};
  private _navShortcuts: Record<string, string> = {};

  get branding() {
    return { ...this._branding };
  }
  get shortcuts() {
    return { ...this._shortcuts };
  }
  get navShortcuts() {
    return { ...this._navShortcuts };
  }

  setBranding(v: typeof this._branding) {
    this._branding = { ...v };
    this.dispatchEvent(new CustomEvent("branding", { detail: this._branding }));
  }
  setShortcuts(v: Record<number, string>) {
    this._shortcuts = { ...v };
    this.dispatchEvent(
      new CustomEvent("shortcuts", { detail: this._shortcuts }),
    );
  }
  setNavShortcuts(v: Record<string, string>) {
    this._navShortcuts = { ...v };
    this.dispatchEvent(
      new CustomEvent("navShortcuts", { detail: this._navShortcuts }),
    );
  }
}

export const store = new Store();
