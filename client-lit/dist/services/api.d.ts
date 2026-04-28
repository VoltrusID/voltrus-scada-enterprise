/**
 * Minimal SSE data manager using native EventSource and CustomEvent.
 */
declare class DataManager extends EventTarget {
    private _es;
    private _url;
    connect(): void;
    disconnect(): void;
}
export declare const dataManager: DataManager;
/**
 * Fetch wrapper for REST API calls.
 */
export declare function api<T>(path: string, init?: RequestInit): Promise<T>;
export declare function fetchOk(url: string, init?: RequestInit): Promise<Response>;
export {};
//# sourceMappingURL=api.d.ts.map