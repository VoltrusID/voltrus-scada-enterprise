export interface SensorConfig {
    id: number;
    key: string;
    name: string;
    unit: string;
    color: string;
    min_val: number;
    max_val: number;
    warn_val: number;
    crit_val: number;
    base_val: number;
    sort_order: number;
    decimals: number;
    source_type: string;
    source_config: string;
    device_id: string;
    equipment_id: string | null;
    chart_type: string;
}
export interface SensorData {
    id: number;
    key: string;
    value: number;
    warn: number;
    crit: number;
    source_type: string;
}
export interface DataPoint {
    value: number;
    ts: number;
}
export interface Screen {
    id: string;
    name: string;
    type: string;
    sort_order: number;
}
export interface LayoutItem {
    i: string;
    x: number;
    y: number;
    w: number;
    h: number;
}
export type ScreenLayouts = {
    lg: LayoutItem[];
    md: LayoutItem[];
    sm: LayoutItem[];
};
export interface CanvasLayoutItem {
    data_source_id: number;
    x: number;
    y: number;
    width: number;
    height: number;
}
export interface BrandingSettings {
    title: string;
    subtitle: string;
    logo: string;
}
export declare const LIVE_PTS = 180;
export declare const SOURCE_CATEGORY: Record<string, {
    label: string;
    tag: string;
    color: string;
}>;
export declare const CHART_TYPES: readonly ["sparkline", "bignumber", "gauge", "image", "text"];
export declare const SIMULATE_TYPES: readonly ["random_walk", "sine", "step", "sawtooth", "constant", "noise"];
export declare function status(val: number, warn: number, crit: number): "critical" | "warning" | "normal";
export declare function stColor(s: string): "var(--ok)" | "var(--muted)" | "var(--crit)" | "var(--warn)";
export declare function safeMinMax(arr: number[]): [number, number];
export declare function parseConfig(cfg: string): Record<string, any>;
export declare function fmtTs(ts: number): string;
export declare function toHexColor(color: string): string;
export interface Alarm {
    id: number;
    source_key: string;
    source_name: string;
    severity: 'warning' | 'critical';
    value: number;
    threshold: number;
    message: string;
    triggered_at: number;
    resolved_at: number | null;
    acknowledged_at: number | null;
    acknowledged_by: string | null;
    cooldown_until: number | null;
}
//# sourceMappingURL=types.d.ts.map