import { LitElement } from 'lit';
import type { SensorConfig } from '../types.js';
export declare class EditPanel extends LitElement {
    static styles: import("lit").CSSResult;
    private _tab;
    private _form;
    private _srcConfig;
    sensor?: SensorConfig;
    connectedCallback(): void;
    private _save;
    private _delete;
    private _close;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=edit-panel.d.ts.map