/**
 * Auth manager using EventTarget for reactive state.
 * No external dependencies. Cookies handled automatically by fetch.
 */
export interface AuthState {
    authed: boolean;
    role: string;
    loading: boolean;
    error: string;
}
declare class AuthManager extends EventTarget {
    private _state;
    get state(): {
        authed: boolean;
        role: string;
        loading: boolean;
        error: string;
    };
    private _set;
    check(): Promise<void>;
    login(username: string, password: string): Promise<void>;
    logout(): Promise<void>;
}
export declare const authManager: AuthManager;
export {};
//# sourceMappingURL=auth.d.ts.map