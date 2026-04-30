const API_BASE = window.location.origin;
class AuthManager extends EventTarget {
    constructor() {
        super(...arguments);
        this._state = { authed: false, role: 'user', loading: true, error: '' };
    }
    get state() { return { ...this._state }; }
    _set(partial) {
        this._state = { ...this._state, ...partial };
        this.dispatchEvent(new CustomEvent('change', { detail: this._state }));
    }
    async check() {
        try {
            const res = await fetch(`${API_BASE}/api/auth/me`, { credentials: 'include' });
            const data = await res.json();
            this._set({ authed: data.authenticated === true, role: data.role || 'user', loading: false, error: '' });
        }
        catch {
            this._set({ authed: false, role: 'user', loading: false, error: '' });
        }
    }
    async login(username, password) {
        this._set({ error: '' });
        try {
            const res = await fetch(`${API_BASE}/api/auth/login`, {
                method: 'POST',
                credentials: 'include',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });
            if (res.ok) {
                const data = await res.json();
                this._set({ authed: true, role: data.role || 'user', error: '' });
                fetch(`${API_BASE}/api/seed`, { method: 'POST', credentials: 'include' }).catch(() => { });
            }
            else {
                const data = await res.json().catch(() => ({}));
                this._set({ error: data.error || 'Login failed' });
            }
        }
        catch {
            this._set({ error: 'Network error' });
        }
    }
    async logout() {
        try {
            await fetch(`${API_BASE}/api/auth/logout`, { method: 'POST', credentials: 'include' });
        }
        catch { /* ignore */ }
        this._set({ authed: false, role: 'user', error: '' });
    }
}
export const authManager = new AuthManager();
//# sourceMappingURL=auth.js.map