# v0.28.0 — Advanced Analytics Engine

**Date:** 2026-05-03
**Status:** Approved
**Timeline:** 6-8 weeks
**Tier:** Starter ($249, basic analytics) / Professional ($499, full SPC + custom metrics)
**Depends on:** v0.27.0 (SDKs & OpenAPI) for merge coordination

---

## Goal

Replace Python/Tableau exports for basic analysis. Full SPC toolbox, histograms with capability indices, run charts, tag-based custom calculations, and scheduled analytics email digests — all built into Voltrus.

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| SPC chart types | X-bar & R, CUSUM, EWMA, P-chart, NP-chart | Full toolbox per user requirement |
| Histogram | Frequency bins + normal curve overlay + Cp/Cpk/Pp/Ppk | Standard capability analysis |
| Run charts | Median line + runs/clustering detection | Simple but essential trend analysis |
| Custom calculations | Tag-based arithmetic formulas | Reuse existing `expression.rs`, covers 80% of use cases |
| Scheduled reports | Extend existing `report_scheduler.rs` | New `analytics_digest` report type, no duplicated infrastructure |
| Frontend charts | Custom Canvas 2D rendering | Matches existing pattern, no new dependencies |
| Comparison views | Extend existing compare endpoint | Add shift-vs-shift, week-vs-week presets |
| Module structure | `src/analytics/` directory with sub-modules | Clean separation of SPC, histogram, run chart, custom metrics |

---

## Section 1: SPC Engine

### Supported Chart Types

| Chart | Use Case | Data Required |
|-------|----------|---------------|
| **X-bar & R** | Variable data (measurements). Monitors process mean and range over subgroups. | Subgroup size (2-25), sample values |
| **CUSUM** | Detect small sustained shifts. Cumulative sum of deviations from target. | Target value, reference value (k), decision interval (h) |
| **EWMA** | Exponentially weighted moving average. Detects small shifts faster than X-bar. | Lambda (smoothing factor, 0.05-0.3), target |
| **P-chart** | Attribute data (proportion defective). For pass/fail type measurements. | Sample size, number defective |
| **NP-chart** | Attribute data (count defective). Same as P-chart but in count units. | Sample size, number defective |

### Control Limit Calculations

All control limits are computed server-side and returned with chart data.

**X-bar chart:**
- UCL = X-double-bar + A2 * R-bar
- LCL = X-double-bar - A2 * R-bar
- CL = X-double-bar

**R chart:**
- UCL = D4 * R-bar
- LCL = D3 * R-bar
- CL = R-bar

Constants A2, D3, D4 depend on subgroup size (standard SPC lookup table).

**CUSUM:**
- S_i+ = max(0, S_i + (x_i - target) - k)
- S_i- = min(0, S_i + (x_i - target) + k)
- Out of control when |S| > h

**EWMA:**
- Z_i = lambda * x_i + (1 - lambda) * Z_(i-1)
- UCL = target + L * sigma * sqrt(lambda/(2-lambda))
- LCL = target - L * sigma * sqrt(lambda/(2-lambda))

**P-chart:**
- p-bar = total defective / total inspected
- UCL = p-bar + 3 * sqrt(p-bar * (1-p-bar) / n)
- LCL = max(0, p-bar - 3 * sqrt(p-bar * (1-p-bar) / n))

**NP-chart:**
- np-bar = n * p-bar
- UCL = np-bar + 3 * sqrt(np-bar * (1 - p-bar))
- LCL = max(0, np-bar - 3 * sqrt(np-bar * (1 - p-bar)))

### New Backend Module: `src/analytics/spc.rs`

```rust
pub struct SpcConfig {
    pub chart_type: SpcChartType,  // XbarR, Cusum, Ewma, P, Np
    pub subgroup_size: u32,        // 2-25 for X-bar/R, sample size for P/NP
    pub target: Option<f64>,       // For CUSUM/EWMA
    pub lambda: Option<f64>,       // For EWMA (0.05-0.3)
    pub k: Option<f64>,            // For CUSUM reference value
    pub h: Option<f64>,            // For CUSUM decision interval
}

pub struct SpcResult {
    pub chart_type: SpcChartType,
    pub data_points: Vec<SpcPoint>,
    pub control_limits: ControlLimits,
    pub violations: Vec<usize>,    // Indices of out-of-control points
    pub summary: SpcSummary,       // In-control %, violation count, etc.
}
```

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/analytics/spc` | GET | Compute SPC chart data. Params: `device_id`, `metric`, `from`, `to`, `chart_type`, `subgroup_size`, `target`, `lambda`, `k`, `h` |
| `/api/v1/analytics/spc/summary` | GET | Summary of SPC violations for a metric over a period. Returns in-control %, violation count, trend direction. |

**Tier:** Professional ($499)

---

## Section 2: Histogram Engine

### Features

- Configurable number of bins (5-100, default 20)
- Frequency count per bin
- Normal distribution curve overlay (computed from sample mean/stddev)
- Process capability indices: Cp, Cpk, Pp, Ppk
- Specification limits (LSL, USL) — optional, from user input

### Capability Indices Formulas

- **Cp** = (USL - LSL) / (6 * sigma)
- **Cpk** = min((USL - mean) / (3 * sigma), (mean - LSL) / (3 * sigma))
- **Pp** = (USL - LSL) / (6 * s) where s = sample stddev (overall)
- **Ppk** = min((USL - mean) / (3 * s), (mean - LSL) / (3 * s))

When LSL/USL not provided, Cp/Cpk/Pp/Ppk are omitted.

### New Backend Module: `src/analytics/histogram.rs`

```rust
pub struct HistogramConfig {
    pub bins: u32,               // 5-100, default 20
    pub lsl: Option<f64>,        // Lower spec limit
    pub usl: Option<f64>,        // Upper spec limit
}

pub struct HistogramResult {
    pub bins: Vec<Bin>,
    pub normal_curve: Vec<Point>, // Normal PDF overlay
    pub stats: DistributionStats, // mean, stddev, skewness, kurtosis
    pub capability: Option<CapabilityIndices>,
}

pub struct Bin {
    pub lower: f64,
    pub upper: f64,
    pub count: u64,
    pub frequency: f64,           // Normalized
}

pub struct CapabilityIndices {
    pub cp: f64,
    pub cpk: f64,
    pub pp: f64,
    pub ppk: f64,
}
```

### API Endpoint

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/analytics/histogram` | GET | Histogram data. Params: `device_id`, `metric`, `from`, `to`, `bins`, `lsl`, `usl` |

**Tier:** Starter ($249, basic histogram) / Professional ($499, +capability indices)

---

## Section 3: Run Chart Engine

### Features

- Plot individual values in time order
- Median line (horizontal reference)
- Runs detection: count consecutive points on same side of median
- Clustering detection: groups of 6+ points trending same direction
- Trend detection: 6+ consecutive increasing or decreasing points

### Rules (Western Electric simplified)

| Rule | Description | Signal |
|------|-------------|--------|
| **Runs** | 8+ consecutive points on one side of median | Shift |
| **Trends** | 6+ consecutive points all increasing or decreasing | Drift |
| **Clustering** | 6+ consecutive points in same direction | Trending |

### New Backend Module: `src/analytics/runchart.rs`

```rust
pub struct RunChartResult {
    pub points: Vec<RunChartPoint>,
    pub median: f64,
    pub signals: Vec<Signal>,    // Detected rule violations
    pub runs_count: u32,         // Number of runs (groups above/below median)
    pub expected_runs: f64,      // Expected number of runs for random data
}

pub struct RunChartPoint {
    pub timestamp: i64,
    pub value: f64,
    pub above_median: bool,
}

pub struct Signal {
    pub rule: SignalRule,        // Run, Trend, Cluster
    pub start_idx: usize,
    pub end_idx: usize,
    pub description: String,
}
```

### API Endpoint

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/analytics/run-chart` | GET | Run chart data with signal detection. Params: `device_id`, `metric`, `from`, `to` |

**Tier:** Starter ($249)

---

## Section 4: Custom Calculations

### Data Model

New table `custom_metrics`:

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto ID |
| name | TEXT UNIQUE | Metric name (e.g., "efficiency") |
| description | TEXT | Human-readable description |
| formula | TEXT | Expression (e.g., "tag_A / tag_B * 100") |
| unit | TEXT | Optional unit string (e.g., "%", "kW") |
| tag_refs | TEXT | JSON array of tag IDs referenced in formula |
| created_at | INTEGER | Timestamp |
| updated_at | INTEGER | Timestamp |

### Formula Evaluation

Reuses existing `src/expression.rs` tokenizer and evaluator. Tag references in formulas use tag keys as variable names:

- Formula: `PUMP_FLOW / PUMP_RATED_FLOW * 100`
- Evaluation: `expression::evaluate()` with a HashMap of `{ "PUMP_FLOW": 45.2, "PUMP_RATED_FLOW": 50.0 }`
- Result: `90.4`

Variables resolve from the live device cache (`DeviceCache`). The formula references tag keys (strings), and the cache provides current values.

### Validation

On save: parse the formula, extract tag references, verify all referenced tags exist in cache. Store validation errors in a `validation_error` column (NULL if valid).

### New Backend Module: `src/analytics/custom.rs`

```rust
pub struct CustomMetric {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub formula: String,
    pub unit: Option<String>,
    pub tag_refs: Vec<String>,
    pub validation_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct CustomMetricValue {
    pub metric: CustomMetric,
    pub value: f64,
    pub computed_at: i64,
    pub tag_values: HashMap<String, f64>,  // Snapshot of tag values used
}
```

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/analytics/custom-metrics` | GET | List all custom metrics |
| `/api/v1/analytics/custom-metrics` | POST | Create custom metric (formula, name, unit) |
| `/api/v1/analytics/custom-metrics/{id}` | PUT | Update formula/description |
| `/api/v1/analytics/custom-metrics/{id}` | DELETE | Delete custom metric |
| `/api/v1/analytics/custom-metrics/{id}/evaluate` | GET | Evaluate metric now (returns current value) |
| `/api/v1/analytics/custom-metrics/evaluate-all` | GET | Evaluate all metrics (returns all current values) |

**Tier:** Professional ($499)

---

## Section 5: Scheduled Analytics Reports

### Extension of Existing Report Scheduler

The existing `report_scheduler.rs` checks for enabled reports hourly. v0.28.0 adds a new report type `analytics_digest`.

### New Fields on `reports` Table

The existing `reports` table gets new columns:
- `report_type` TEXT DEFAULT 'pdf' — now supports 'pdf' | 'analytics_digest'
- `analytics_config` TEXT — JSON config for analytics digest

### Analytics Digest Config

```json
{
  "metrics": ["voltage", "temperature", "efficiency"],
  "custom_metrics": [1, 3],
  "spc_charts": [
    { "metric": "voltage", "chart_type": "xbar_r", "subgroup_size": 5 }
  ],
  "histograms": [
    { "metric": "temperature", "bins": 20, "lsl": 18, "usl": 25 }
  ],
  "include_comparison": true,
  "comparison_period": "previous_shift"
}
```

### Generated Report

The scheduler generates an HTML email containing:
1. **Header:** Report name, date range, generated timestamp
2. **Summary cards:** Key metric values (current, min, max, avg for period)
3. **SPC alerts:** Any out-of-control points detected
4. **Custom metrics:** Current values of all tracked custom metrics
5. **Comparison:** This period vs previous period (change %)
6. **Footer:** Voltrus branding

The HTML is sent via the existing `EmailNotifier`. No PDF conversion needed (email is HTML).

### API Endpoints

Uses existing report endpoints with the new `report_type`:
- `POST /api/v1/reports` — body includes `report_type: "analytics_digest"` and `analytics_config`
- `GET /api/v1/reports` — lists all reports including analytics digests
- No new endpoints needed — extend existing CRUD.

**Tier:** Professional ($499)

---

## Section 6: Frontend — Analytics Panel Overhaul

### Tabbed Layout

The existing `AnalyticsPanel.tsx` gets refactored into a tabbed interface:

| Tab | Component | Content |
|-----|-----------|---------|
| **Overview** | Existing aggregate stats + sparkline | Current: stat cards, range selector, CSV export |
| **SPC** | `SPCChart.tsx` | Chart type selector, control chart Canvas, violation markers |
| **Histogram** | `HistogramChart.tsx` | Bin count slider, spec limit inputs, normal curve overlay, Cp/Cpk display |
| **Run Chart** | `RunChart.tsx` | Time-ordered points, median line, signal highlights |
| **Compare** | Existing comparison view | Enhanced with shift/week presets |
| **Custom Metrics** | `CalcBuilder.tsx` | Formula editor with tag autocomplete, live evaluation preview |

### SPC Chart Component (`SPCChart.tsx`)

Canvas 2D rendering matching existing chart style:
- Plot points as small circles
- Control limits as dashed horizontal lines (UCL red, CL green, LCL red)
- Violation points highlighted in red with larger radius
- Tooltip on hover showing point value, timestamp, within/out-of-control
- Chart type selector: X-bar & R, CUSUM, EWMA, P, NP
- Configurable parameters per chart type (subgroup size, target, lambda, k, h)

### Histogram Component (`HistogramChart.tsx`)

Canvas 2D bar chart:
- Vertical bars for each bin
- Normal curve overlay as smooth line
- Spec limits (LSL/USL) as vertical dashed lines if provided
- Capability indices (Cp, Cpk, Pp, Ppk) displayed as stat cards below chart
- Bin count slider (5-100)
- Optional LSL/USL input fields

### Run Chart Component (`RunChart.tsx`)

Canvas 2D line chart:
- Individual data points connected by line
- Median as horizontal dashed line
- Signal regions highlighted with colored background bands
- Signal list below chart with descriptions

### Calculation Builder (`CalcBuilder.tsx`)

- Text input for formula (e.g., `PUMP_FLOW / PUMP_RATED_FLOW * 100`)
- Tag autocomplete dropdown: shows available tag keys from device cache
- Live preview: evaluates formula with current tag values, shows result
- Name, description, unit fields
- List of existing custom metrics with current values
- Edit/delete buttons per metric

### New Frontend Query Hooks

Added to `client/src/queries/analytics.ts`:

```typescript
export function useSpcChart(...)
export function useHistogram(...)
export function useRunChart(...)
export function useCustomMetrics()
export function useCreateCustomMetric(...)
export function useUpdateCustomMetric(...)
export function useDeleteCustomMetric(...)
export function useEvaluateCustomMetric(...)
export function useEvaluateAllCustomMetrics()
```

---

## Section 7: New Files & Module Structure

### Backend (Rust)

| File | Purpose |
|------|---------|
| Create `src/analytics/mod.rs` | Module re-exports for analytics sub-modules |
| Create `src/analytics/spc.rs` | SPC calculations (X-bar/R, CUSUM, EWMA, P, NP) |
| Create `src/analytics/histogram.rs` | Histogram binning, normal curve, capability indices |
| Create `src/analytics/runchart.rs` | Run chart with signal detection |
| Create `src/analytics/custom.rs` | Custom metric CRUD + formula evaluation |
| Modify `src/analytics.rs` | Move existing code into `src/analytics/mod.rs`, keep `src/analytics.rs` as re-export for backward compat |
| Modify `src/api/analytics.rs` | Add SPC, histogram, run chart, custom metric endpoints |
| Modify `src/api/mod.rs` | Register new routes |
| Modify `src/storage.rs` | New `custom_metrics` table, new analytics query methods |
| Modify `src/report_scheduler.rs` | Handle `analytics_digest` report type |
| Modify `src/models.rs` | Add CustomMetric model, AnalyticsDigestConfig |
| Modify `Cargo.toml` | Bump version to 0.28.0 |

### Frontend (React/TypeScript)

| File | Purpose |
|------|---------|
| Create `client/src/components/SPCChart.tsx` | Canvas 2D SPC control chart renderer |
| Create `client/src/components/HistogramChart.tsx` | Canvas 2D histogram with normal overlay |
| Create `client/src/components/RunChart.tsx` | Canvas 2D run chart with signals |
| Create `client/src/components/CalcBuilder.tsx` | Custom metric formula editor |
| Modify `client/src/AnalyticsPanel.tsx` | Refactor into tabbed layout |
| Modify `client/src/queries/analytics.ts` | Add new query hooks |

### Landing / Docs

| File | Purpose |
|------|---------|
| Modify `voltrus-landing/changelog.html` | v0.28.0 entry |
| Modify `voltrus-landing/docs.html` | Analytics documentation section |
| Modify `voltrus-landing/index.html` | Advanced Analytics feature mention |

---

## Section 8: v0.27.0 Merge Coordination

v0.27.0 (SDKs & OpenAPI) adds utoipa annotations to all handlers including the analytics handlers. v0.28.0 modifies `src/api/analytics.rs` (new endpoints) and `src/api/mod.rs` (new routes). Both modify `Cargo.toml`.

**Merge plan:**
1. v0.27.0 is already merged (complete)
2. v0.28.0 builds on top of v0.27.0
3. New endpoints in v0.28.0 need `#[utoipa::path]` annotations (same pattern as v0.27.0)
4. New model types need `ToSchema` derives
5. `src/api/openapi.rs` needs new paths and schemas registered

**Files modified by both:**
- `src/api/analytics.rs` — v0.27.0 added annotations, v0.28.0 adds new endpoints (additive)
- `src/api/mod.rs` — v0.27.0 added Swagger UI, v0.28.0 adds routes (additive)
- `Cargo.toml` — both bump version (v0.28.0 wins)

---

## Testing Strategy

- **Unit tests:** SPC calculations (control limits, violation detection) with known datasets. Histogram binning. Run chart signal detection. Custom metric formula evaluation.
- **Integration tests:** API endpoints return valid JSON. SPC endpoints validate parameters. Custom metrics CRUD works end-to-end. Analytics digest generation produces valid HTML.
- **Frontend tests:** Chart rendering with mock data (Canvas snapshot comparison). Formula editor validation.
- **Performance:** SPC calculation on 10K data points < 100ms. Histogram on 100K points < 200ms.

---

## Implementation Order

1. Create `src/analytics/` directory with sub-modules (spc, histogram, runchart, custom)
2. Implement SPC calculations with unit tests
3. Implement histogram calculations with unit tests
4. Implement run chart calculations with unit tests
5. Implement custom metric CRUD + evaluation with unit tests
6. Add new tables to storage.rs
7. Add new API endpoints with utoipa annotations
8. Register routes and OpenAPI schemas
9. Extend report scheduler for analytics_digest type
10. Build frontend SPC chart component
11. Build frontend histogram component
12. Build frontend run chart component
13. Build frontend calculation builder
14. Refactor AnalyticsPanel into tabbed layout
15. Update landing page, docs, changelog
16. Bump version to 0.28.0
