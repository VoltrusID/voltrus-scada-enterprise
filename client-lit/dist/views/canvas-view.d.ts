import { LitElement } from 'lit';
import '../components/sensor-card.js';
export declare class CanvasView extends LitElement {
    static styles: import("lit").CSSResult;
    private _sensors;
    private _layouts;
    private _dataMap;
    private _currentValues;
    private _selectedId;
    private _dragging;
    private _resizing;
    private _loading;
    screenId: string;
    connectedCallback(): void;
    disconnectedCallback(): void;
    updated(changed: Map<string, unknown>): void;
    private _load;
    private _onData;
    private _layoutFor;
    private _onMouseDown;
    private _onResizeDown;
    private _onMouseMove;
    private _onMouseUp;
    private _updateLayout;
    saveLayouts(): Promise<void>;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=canvas-view.d.ts.map