import { LitElement } from 'lit';
import '../components/sensor-card.js';
export declare class GridView extends LitElement {
    static styles: import("lit").CSSResult;
    private _sensors;
    private _dataMap;
    private _currentValues;
    private _screenId;
    private _loading;
    private _layouts;
    private _editMode;
    private _draggingId;
    private _overId;
    private _bp;
    private _containerWidth;
    private _resizeObs?;
    set screenId(id: string);
    get screenId(): string;
    set editMode(v: boolean);
    get editMode(): boolean;
    connectedCallback(): void;
    disconnectedCallback(): void;
    private _loadSensors;
    private _loadLayouts;
    saveLayouts(): Promise<void>;
    private _seedHistory;
    private _onData;
    private _cardSensors;
    private _layoutFor;
    private _gridStyleFor;
    private _onSensorClick;
    private _onDragStart;
    private _onDragOver;
    private _onDragLeave;
    private _onDrop;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=grid-view.d.ts.map