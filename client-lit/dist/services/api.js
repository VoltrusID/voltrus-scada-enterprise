/**
 * Backend base URL — derived from current origin so it works
 * regardless of server port or hostname.
 */
const API_BASE = window.location.origin;
/**
 * Minimal SSE data manager using native EventSource and CustomEvent.
 */
class DataManager extends EventTarget {
    constructor() {
        super(...arguments);
        this._es = null;
        this._url = `${API_BASE}/api/stream`;
    }
    connect() {
        if (this._es)
            return;
        this._es = new EventSource(this._url, { withCredentials: true });
        this._es.onmessage = (ev) => {
            try {
                const payload = JSON.parse(ev.data);
                const sensors = Array.isArray(payload) ? payload : payload.sensors ?? [];
                this.dispatchEvent(new CustomEvent('data', { detail: sensors }));
            }
            catch {
                // ignore malformed SSE payloads
            }
        };
        this._es.onerror = () => {
            // auto-reconnect is handled by EventSource
        };
    }
    disconnect() {
        this._es?.close();
        this._es = null;
    }
}
export const dataManager = new DataManager();
/**
 * Fetch wrapper for REST API calls.
 */
export async function api(path, init) {
    const res = await fetch(`${API_BASE}/api${path}`, {
        credentials: 'include',
        headers: { 'Content-Type': 'application/json', ...(init?.headers ?? {}) },
        ...init,
    });
    if (!res.ok)
        throw new Error(`API ${path} failed: ${res.status}`);
    return res.json();
}
export async function fetchOk(url, init) {
    const res = await fetch(`${API_BASE}${url}`, { credentials: 'include', ...init });
    if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || `Request failed: ${res.status}`);
    }
    return res;
}
//# sourceMappingURL=api.js.map