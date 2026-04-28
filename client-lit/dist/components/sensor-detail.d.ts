import { LitElement } from 'lit';
import type { SensorConfig, SensorData } from '../types.js';
import './detail-chart.js';
export declare class SensorDetail extends LitElement {
    static styles: import("lit").CSSResult;
    sensor: SensorConfig;
    currentValue: SensorData | undefined;
    allSensors: SensorConfig[];
    private _range;
    private _historyData;
    private _loading;
    private _overlayIds;
    private _overlayData;
    private _liveValue;
    private _liveHistory;
    connectedCallback(): void;
    disconnectedCallback(): void;
    private _onKey;
    private _onLiveData;
    private _loadHistory;
    private _loadOverlayHistories;
    private _back;
    private _setRange;
    private _toggleOverlay;
    private _removeOverlay;
    private _overlays;
    private _chartData;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=sensor-detail.d.ts.map