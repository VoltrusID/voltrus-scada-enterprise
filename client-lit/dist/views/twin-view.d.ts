import { LitElement } from 'lit';
export declare class TwinView extends LitElement {
    static styles: import("lit").CSSResult;
    private _ready;
    private _renderer?;
    private _scene?;
    private _camera?;
    private _frameId?;
    connectedCallback(): void;
    disconnectedCallback(): void;
    firstUpdated(): void;
    private _initThree;
    private _addBox;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=twin-view.d.ts.map