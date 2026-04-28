import { LitElement } from 'lit';
export declare class ChartSparkline extends LitElement {
    static styles: import("lit").CSSResult;
    data: number[];
    color: string;
    warn: number;
    crit: number;
    canvas: HTMLCanvasElement;
    updated(changed: Map<string, any>): void;
    private _draw;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=chart-sparkline.d.ts.map