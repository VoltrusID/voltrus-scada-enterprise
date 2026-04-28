import { LitElement } from 'lit';
export declare class AlarmPanel extends LitElement {
    static styles: import("lit").CSSResult;
    private _filter;
    private _alarms;
    private _loading;
    connectedCallback(): void;
    disconnectedCallback(): void;
    private _cleanup?;
    private _loadAlarms;
    private _ack;
    private _ackAll;
    private _setFilter;
    private _close;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=alarm-panel.d.ts.map