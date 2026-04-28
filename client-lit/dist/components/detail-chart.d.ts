import { LitElement } from 'lit';
import type { DataPoint } from '../types.js';
export interface OverlaySeries {
    data: DataPoint[];
    color: string;
    name: string;
    unit: string;
}
export declare class DetailChart extends LitElement {
    static styles: import("lit").CSSResult;
    data: DataPoint[];
    color: string;
    warn: number;
    crit: number;
    unit: string;
    overlays: OverlaySeries[];
    pointWidth: number;
    canvas: HTMLCanvasElement;
    scrollEl: HTMLDivElement;
    wrapperEl: HTMLDivElement;
    private _area;
    private _hover;
    updated(changed: Map<string, any>): void;
    private _draw;
    private _onMove;
    private _onLeave;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=detail-chart.d.ts.map