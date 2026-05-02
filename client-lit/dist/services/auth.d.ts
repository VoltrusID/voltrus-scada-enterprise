export interface AuthState {
    authed: boolean;
    role: string;
    loading: boolean;
    error: string;
    mustChangePassword: boolean;
}
declare class AuthManager extends EventTarget {
    private _state;
    get state(): {
        authed: boolean;
        role: string;
        loading: boolean;
        error: string;
        mustChangePassword: boolean;
    };
    private _set;
    check(): Promise<void>;
    login(username: string, password: string): Promise<void>;
    changePassword(newPassword: string): Promise<boolean>;
    logout(): Promise<void>;
}
export declare const authManager: AuthManager;
export {};
//# sourceMappingURL=auth.d.ts.map