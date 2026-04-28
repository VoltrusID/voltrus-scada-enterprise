import { LitElement } from 'lit';
export declare class PromptDialog extends LitElement {
    static styles: import("lit").CSSResult;
    private _value;
    title: string;
    label: string;
    placeholder: string;
    initialValue: string;
    okLabel: string;
    connectedCallback(): void;
    private _submit;
    private _close;
    render(): import("lit-html").TemplateResult<1>;
}
export declare class ConfirmDialog extends LitElement {
    static styles: import("lit").CSSResult;
    title: string;
    message: string;
    confirmLabel: string;
    private _confirm;
    private _close;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=screen-dialog.d.ts.map