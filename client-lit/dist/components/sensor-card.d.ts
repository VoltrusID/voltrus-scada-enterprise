import { LitElement } from 'lit';
import { SensorConfig, SensorData } from '../types.js';
import './chart-sparkline.js';
export interface CardSensor {
    config: SensorConfig;
    data?: SensorData;
    chartData: number[];
}
export declare class SensorCard extends LitElement {
    static styles: import("lit").CSSResult;
    sensor: CardSensor;
    selected: boolean;
    private _click;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=sensor-card.d.ts.map