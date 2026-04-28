/**
 * Tiny observable store for shared UI state.
 * No external deps. EventTarget-based reactivity.
 */
declare class Store extends EventTarget {
    private _branding;
    private _shortcuts;
    private _navShortcuts;
    get branding(): {
        title: string;
        subtitle: string;
        logo: string;
    };
    get shortcuts(): {
        [x: number]: string;
    };
    get navShortcuts(): {
        [x: string]: string;
    };
    setBranding(v: typeof this._branding): void;
    setShortcuts(v: Record<number, string>): void;
    setNavShortcuts(v: Record<string, string>): void;
}
export declare const store: Store;
export {};
//# sourceMappingURL=store.d.ts.map