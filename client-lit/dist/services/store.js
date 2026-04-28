/**
 * Tiny observable store for shared UI state.
 * No external deps. EventTarget-based reactivity.
 */
class Store extends EventTarget {
    constructor() {
        super(...arguments);
        this._branding = { title: 'Voltrus Lite', subtitle: 'Industrial monitoring', logo: '' };
        this._shortcuts = {};
        this._navShortcuts = {};
    }
    get branding() { return { ...this._branding }; }
    get shortcuts() { return { ...this._shortcuts }; }
    get navShortcuts() { return { ...this._navShortcuts }; }
    setBranding(v) { this._branding = { ...v }; this.dispatchEvent(new CustomEvent('branding', { detail: this._branding })); }
    setShortcuts(v) { this._shortcuts = { ...v }; this.dispatchEvent(new CustomEvent('shortcuts', { detail: this._shortcuts })); }
    setNavShortcuts(v) { this._navShortcuts = { ...v }; this.dispatchEvent(new CustomEvent('navShortcuts', { detail: this._navShortcuts })); }
}
export const store = new Store();
//# sourceMappingURL=store.js.map