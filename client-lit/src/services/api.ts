import type { SensorConfig, SensorData } from '../types.js';

/**
 * Backend base URL — derived from current origin so it works
 * regardless of server port or hostname.
 */
const API_BASE = window.location.origin;

/**
 * Minimal SSE data manager using native EventSource and CustomEvent.
 */
class DataManager extends EventTarget {
  private _es: EventSource | null = null;
  private _url = `${API_BASE}/api/v1/stream`;

  connect() {
    if (this._es) return;
    this._es = new EventSource(this._url, { withCredentials: true });
    this._es.onmessage = (ev) => {
      try {
        const payload = JSON.parse(ev.data);
        const sensors: SensorData[] = Array.isArray(payload) ? payload : payload.sensors ?? [];
        this.dispatchEvent(new CustomEvent('data', { detail: sensors }));
      } catch {
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

// Global in-flight refresh promise
let refreshing: Promise<boolean> | null = null

async function doRefresh(): Promise<boolean> {
  if (refreshing) {
    return refreshing
  }
  refreshing = (async () => {
    try {
      const r = await fetch(`${API_BASE}/api/v1/auth/refresh`, {
        method: 'POST',
        credentials: 'include',
      })
      return r.ok
    } catch {
      return false
    } finally {
      refreshing = null
    }
  })()
  return refreshing
}

async function fetchWithRefresh(
  url: string,
  init?: RequestInit,
): Promise<Response> {
  const r = await fetch(url, { credentials: 'include', ...init })
  if (r.status === 401) {
    const ok = await doRefresh()
    if (ok) {
      return fetch(url, { credentials: 'include', ...init })
    }
    window.location.href = '/login'
    throw new Error('Session expired')
  }
  return r
}

/**
 * Fetch wrapper for REST API calls.
 */
export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetchWithRefresh(`${API_BASE}/api/v1${path}`, {
    headers: { 'Content-Type': 'application/json', ...((init?.headers as Record<string, string>) ?? {}) },
    ...init,
  });
  if (!res.ok) throw new Error(`API ${path} failed: ${res.status}`);
  return res.json() as Promise<T>;
}

export async function fetchOk(url: string, init?: RequestInit): Promise<Response> {
  const res = await fetchWithRefresh(`${API_BASE}${url}`, { credentials: 'include', ...init });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.error || `Request failed: ${res.status}`);
  }
  return res;
}
