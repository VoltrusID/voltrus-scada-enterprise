# v0.28.0 — Advanced Analytics Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Python/Tableau exports for basic analysis with a full SPC toolbox, histograms with capability indices, run charts, tag-based custom calculations, and scheduled analytics email digests — all built into Voltrus.

**Architecture:** Backend analytics engine in `src/analytics/` with sub-modules for SPC, histogram, run chart, and custom metrics. Frontend React components using custom Canvas 2D rendering (matching existing pattern). Scheduled analytics reports extend the existing report scheduler.

**Tech Stack:** Rust (Axum, SQLite, expression.rs evaluator), TypeScript/React (Canvas 2D, TanStack Query)

---

## File Structure

### Backend (Rust)

| File | Purpose |
|------|---------|
| `src/analytics/mod.rs` | Module re-exports, existing aggregate/compare code moved here |
| `src/analytics/spc.rs` | SPC calculations: X-bar/R, CUSUM, EWMA, P-chart, NP-chart |
| `src/analytics/histogram.rs` | Histogram binning, normal curve overlay, Cp/Cpk/Pp/Ppk |
| `src/analytics/runchart.rs` | Run chart with median, runs/trend/cluster detection |
| `src/analytics/custom.rs` | Custom metric CRUD + formula evaluation via expression.rs |
| `src/analytics.rs` | Backward-compat re-export of `analytics::mod` public API |
| `src/api/analytics.rs` | New API endpoints for SPC, histogram, run chart, custom metrics |
| `src/storage.rs` | New `custom_metrics` table, new query methods |
| `src/models.rs` | New model structs |
| `src/report_scheduler.rs` | Handle `analytics_digest` report type |
| `src/report.rs` | Analytics digest HTML generation |
| `src/api/mod.rs` | Register new routes |
| `src/api/openapi.rs` | Register new paths/schemas |

### Frontend (React/TypeScript)

| File | Purpose |
|------|---------|
| `client/src/components/SPCChart.tsx` | Canvas 2D SPC control chart renderer |
| `client/src/components/HistogramChart.tsx` | Canvas 2D histogram with normal overlay |
| `client/src/components/RunChart.tsx` | Canvas 2D run chart with signal markers |
| `client/src/components/CalcBuilder.tsx` | Custom metric formula editor UI |
| `client/src/AnalyticsPanel.tsx` | Refactor into tabbed layout |
| `client/src/queries/analytics.ts` | New query hooks |

### Landing / Docs

| File | Purpose |
|------|---------|
| `voltrus-landing/changelog.html` | v0.28.0 entry |
| `voltrus-landing/docs.html` | Analytics documentation section |
| `voltrus-landing/index.html` | Advanced Analytics feature mention |

---

## Task 1: Refactor analytics.rs into analytics module directory

**Files:**
- Create: `src/analytics/mod.rs`
- Create: `src/analytics/spc.rs` (empty placeholder)
- Create: `src/analytics/histogram.rs` (empty placeholder)
- Create: `src/analytics/runchart.rs` (empty placeholder)
- Create: `src/analytics/custom.rs` (empty placeholder)
- Modify: `src/analytics.rs` → becomes re-export shim

- [ ] **Step 1: Create analytics module directory and move existing code**

Create `src/analytics/mod.rs` by copying the entire content of `src/analytics.rs` into it, then add module declarations:

```rust
// src/analytics/mod.rs — move ALL existing code from src/analytics.rs here

pub mod spc;
pub mod histogram;
pub mod runchart;
pub mod custom;

use anyhow::Result;
use serde::Serialize;
// ... rest of existing analytics.rs code (AggregatedMetric, ComparisonResult, aggregate_metrics, compare_periods, tests)
```

- [ ] **Step 2: Create placeholder sub-modules**

Create `src/analytics/spc.rs`:
```rust
// SPC calculations — to be implemented
```

Create `src/analytics/histogram.rs`:
```rust
// Histogram calculations — to be implemented
```

Create `src/analytics/runchart.rs`:
```rust
// Run chart calculations — to be implemented
```

Create `src/analytics/custom.rs`:
```rust
// Custom metric calculations — to be implemented
```

- [ ] **Step 3: Replace `src/analytics.rs` with re-export shim**

```rust
// src/analytics.rs — backward-compatible re-export
pub mod analytics_module;
pub use analytics_module::*;
```

Wait — Rust doesn't allow both `analytics.rs` and `analytics/` directory. The correct approach: delete `src/analytics.rs` and create `src/analytics/mod.rs` instead. Update the reference in `src/api/analytics.rs` — the import `use crate::analytics;` will still resolve because `mod.rs` in a directory works the same as a file.

Replace `src/analytics.rs` entirely with:
```rust
// This file is replaced by src/analytics/mod.rs
// The module system resolves crate::analytics to analytics/mod.rs
```

Actually, Rust **cannot** have both `src/analytics.rs` and `src/analytics/` directory. So:
1. Move `src/analytics.rs` content to `src/analytics/mod.rs`
2. Delete `src/analytics.rs`
3. All existing `use crate::analytics` imports continue to work unchanged

- [ ] **Step 4: Run tests to verify nothing broke**

Run: `cargo test --lib analytics`
Expected: All existing analytics tests pass

- [ ] **Step 5: Commit**

```bash
git add src/analytics/ src/analytics.rs
git commit -m "refactor(analytics): move analytics.rs into module directory"
```

---

## Task 2: Add new model types and storage schema

**Files:**
- Modify: `src/models.rs` — add CustomMetric, SpcConfig, HistogramConfig, etc.
- Modify: `src/storage.rs` — add custom_metrics table + new query methods

- [ ] **Step 1: Add model types to `src/models.rs`**

Add after the existing `Report` struct (around line 1719):

```rust
// ── Analytics v0.28.0 models ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CustomMetric {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub formula: String,
    pub unit: Option<String>,
    pub tag_refs: String, // JSON array of tag keys
    pub validation_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateCustomMetricRequest {
    pub name: String,
    pub description: String,
    pub formula: String,
    pub unit: Option<String>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateCustomMetricRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub formula: Option<String>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct CustomMetricValue {
    pub id: i64,
    pub name: String,
    pub value: f64,
    pub unit: Option<String>,
    pub computed_at: i64,
    pub tag_values: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct SpcConfig {
    pub chart_type: String, // "xbar_r", "cusum", "ewma", "p", "np"
    pub subgroup_size: Option<u32>,
    pub target: Option<f64>,
    pub lambda: Option<f64>,
    pub k: Option<f64>,
    pub h: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct SpcResult {
    pub chart_type: String,
    pub data_points: Vec<SpcPoint>,
    pub control_limits: ControlLimits,
    pub violations: Vec<usize>,
    pub in_control_percent: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct SpcPoint {
    pub index: usize,
    pub value: f64,
    pub timestamp: i64,
    pub subgroup_values: Option<Vec<f64>>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ControlLimits {
    pub ucl: f64,
    pub cl: f64,
    pub lcl: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct HistogramResult {
    pub bins: Vec<HistogramBin>,
    pub normal_curve: Vec<CurvePoint>,
    pub stats: DistributionStats,
    pub capability: Option<CapabilityIndices>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct HistogramBin {
    pub lower: f64,
    pub upper: f64,
    pub count: u64,
    pub frequency: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DistributionStats {
    pub mean: f64,
    pub stddev: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub count: u64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct CapabilityIndices {
    pub cp: f64,
    pub cpk: f64,
    pub pp: f64,
    pub ppk: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct RunChartResult {
    pub points: Vec<RunChartPoint>,
    pub median: f64,
    pub signals: Vec<Signal>,
    pub runs_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct RunChartPoint {
    pub timestamp: i64,
    pub value: f64,
    pub above_median: bool,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct Signal {
    pub rule: String, // "run", "trend", "cluster"
    pub start_idx: usize,
    pub end_idx: usize,
    pub description: String,
}
```

- [ ] **Step 2: Add `custom_metrics` table to `init_schema` in `src/storage.rs`**

Find the `init_schema` function. Inside the `batch.execute_batch(...)` call, add a new CREATE TABLE statement:

```sql
CREATE TABLE IF NOT EXISTS custom_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    formula TEXT NOT NULL,
    unit TEXT,
    tag_refs TEXT NOT NULL DEFAULT '[]',
    validation_error TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

Add it alongside the other CREATE TABLE statements inside the `execute_batch` call in `init_schema`.

- [ ] **Step 3: Add storage methods for custom metrics**

Add these methods to the `impl Storage` block in `src/storage.rs`:

```rust
    // ── Custom Metrics CRUD ──

    pub async fn list_custom_metrics(&self) -> Result<Vec<crate::models::CustomMetric>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, description, formula, unit, tag_refs, validation_error, created_at, updated_at
                 FROM custom_metrics ORDER BY name"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(crate::models::CustomMetric {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    formula: row.get(3)?,
                    unit: row.get(4)?,
                    tag_refs: row.get(5)?,
                    validation_error: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| anyhow::anyhow!(e))
        }).await.context("Task join failed")?
    }

    pub async fn get_custom_metric(&self, id: i64) -> Result<Option<crate::models::CustomMetric>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, description, formula, unit, tag_refs, validation_error, created_at, updated_at
                 FROM custom_metrics WHERE id = ?1"
            )?;
            let mut rows = stmt.query_map([id], |row| {
                Ok(crate::models::CustomMetric {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    formula: row.get(3)?,
                    unit: row.get(4)?,
                    tag_refs: row.get(5)?,
                    validation_error: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?;
            match rows.next() {
                Some(row) => Ok(Some(row.context("Row parse failed")?)),
                None => Ok(None),
            }
        }).await.context("Task join failed")?
    }

    pub async fn create_custom_metric(
        &self,
        name: &str,
        description: &str,
        formula: &str,
        unit: Option<&str>,
        tag_refs: &str,
        validation_error: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let description = description.to_string();
        let formula = formula.to_string();
        let unit = unit.map(String::from);
        let tag_refs = tag_refs.to_string();
        let validation_error = validation_error.map(String::from);
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO custom_metrics (name, description, formula, unit, tag_refs, validation_error, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                rusqlite::params![name, description, formula, unit, tag_refs, validation_error, now],
            ).context("Insert custom_metric failed")?;
            Ok(conn.last_insert_rowid())
        }).await.context("Task join failed")?
    }

    pub async fn update_custom_metric(
        &self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
        formula: Option<&str>,
        unit: Option<&str>,
        tag_refs: Option<&str>,
        validation_error: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let name = name.map(String::from);
        let description = description.map(String::from);
        let formula = formula.map(String::from);
        let unit = unit.map(String::from);
        let tag_refs = tag_refs.map(String::from);
        let validation_error = validation_error.map(String::from);
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // Build dynamic UPDATE
            let mut sets = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if let Some(ref v) = name { sets.push("name = ?"); params.push(Box::new(v.clone())); }
            if let Some(ref v) = description { sets.push("description = ?"); params.push(Box::new(v.clone())); }
            if let Some(ref v) = formula { sets.push("formula = ?"); params.push(Box::new(v.clone())); }
            if let Some(ref v) = unit { sets.push("unit = ?"); params.push(Box::new(v.clone())); }
            if let Some(ref v) = tag_refs { sets.push("tag_refs = ?"); params.push(Box::new(v.clone())); }
            if let Some(ref v) = validation_error { sets.push("validation_error = ?"); params.push(Box::new(v.clone())); }
            sets.push("updated_at = ?");
            params.push(Box::new(now));
            params.push(Box::new(id));
            let sql = format!("UPDATE custom_metrics SET {} WHERE id = ?", sets.join(", "));
            conn.execute(&sql, rusqlite::params_from_iter(params.iter())).context("Update custom_metric failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn delete_custom_metric(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM custom_metrics WHERE id = ?1", [id])
                .context("Delete custom_metric failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    /// Fetch raw telemetry values for analytics (SPC, histogram, run chart).
    /// Returns (timestamp, value) pairs ordered by timestamp.
    pub async fn get_telemetry_series(
        &self,
        device_id: &str,
        metric: &str,
        from: i64,
        to: i64,
        limit: Option<u32>,
    ) -> Result<Vec<(i64, f64)>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let metric = metric.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let sql = match limit {
                Some(_) => "SELECT timestamp, value FROM telemetry WHERE device_id = ?1 AND metric = ?2 AND timestamp >= ?3 AND timestamp <= ?4 ORDER BY timestamp LIMIT ?5",
                None => "SELECT timestamp, value FROM telemetry WHERE device_id = ?1 AND metric = ?2 AND timestamp >= ?3 AND timestamp <= ?4 ORDER BY timestamp",
            };
            let mut stmt = conn.prepare(sql).context("Prepare get_telemetry_series failed")?;
            let rows = match limit {
                Some(lim) => stmt.query_map(rusqlite::params![device_id, metric, from, to, lim], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
                })?,
                None => stmt.query_map(rusqlite::params![device_id, metric, from, to], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
                })?,
            };
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| anyhow::anyhow!(e))
        }).await.context("Task join failed")?
    }
```

Also add the import at the top of storage.rs:
```rust
use crate::models::{
    // ... existing imports ...
    CustomMetric, CreateCustomMetricRequest, UpdateCustomMetricRequest,
};
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check`
Expected: Compiles with no errors (models are defined, storage methods are present)

- [ ] **Step 5: Commit**

```bash
git add src/models.rs src/storage.rs
git commit -m "feat(analytics): add v0.28.0 model types and custom_metrics storage"
```

---

## Task 3: Implement SPC calculations

**Files:**
- Create: `src/analytics/spc.rs`

- [ ] **Step 1: Write SPC calculation tests in `src/analytics/spc.rs`**

```rust
use crate::models::{SpcResult, SpcPoint, ControlLimits};

/// SPC constants table for X-bar and R charts (subset: subgroup sizes 2-10).
/// A2, D3, D4 indexed by subgroup_size - 2.
const A2: [f64; 9] = [1.880, 1.023, 0.729, 0.577, 0.483, 0.419, 0.373, 0.337, 0.308];
const D3: [f64; 9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.076, 0.136, 0.184, 0.223];
const D4: [f64; 9] = [3.267, 2.575, 2.282, 2.115, 2.004, 1.924, 1.864, 1.816, 1.777];

/// Compute X-bar and R chart data from raw values.
/// `values` is a flat list of individual measurements.
/// `subgroup_size` is the number of samples per subgroup (2-10).
/// Returns (xbar_result, r_result).
pub fn xbar_r(
    values: &[f64],
    subgroup_size: usize,
    timestamps: &[i64],
) -> Result<(SpcResult, SpcResult), String> {
    if subgroup_size < 2 || subgroup_size > 10 {
        return Err("Subgroup size must be 2-10".into());
    }
    if values.is_empty() {
        return Err("No data points".into());
    }

    // Divide into subgroups
    let mut xbars: Vec<f64> = Vec::new();
    let mut ranges: Vec<f64> = Vec::new();
    let mut subgroup_timestamps: Vec<i64> = Vec::new();
    let mut subgroup_values_list: Vec<Vec<f64>> = Vec::new();

    let chunks = values.chunks(subgroup_size);
    let ts_chunks = timestamps.chunks(subgroup_size);

    for (chunk, ts_chunk) in chunks.zip(ts_chunks) {
        if chunk.len() < subgroup_size {
            break; // skip incomplete subgroup
        }
        let mean = chunk.iter().sum::<f64>() / chunk.len() as f64;
        let range = chunk.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - chunk.iter().cloned().fold(f64::INFINITY, f64::min);
        xbars.push(mean);
        ranges.push(range);
        subgroup_timestamps.push(ts_chunk[0]);
        subgroup_values_list.push(chunk.to_vec());
    }

    if xbars.is_empty() {
        return Err("Not enough data for any complete subgroup".into());
    }

    // X-bar chart
    let xbar_avg = xbars.iter().sum::<f64>() / xbars.len() as f64;
    let r_avg = ranges.iter().sum::<f64>() / ranges.len() as f64;
    let idx = subgroup_size - 2;
    let a2 = A2[idx];
    let d3 = D3[idx];
    let d4 = D4[idx];

    let xbar_limits = ControlLimits {
        ucl: xbar_avg + a2 * r_avg,
        cl: xbar_avg,
        lcl: xbar_avg - a2 * r_avg,
    };

    let xbar_points: Vec<SpcPoint> = xbars.iter().enumerate().map(|(i, &v)| SpcPoint {
        index: i,
        value: v,
        timestamp: subgroup_timestamps.get(i).copied().unwrap_or(0),
        subgroup_values: Some(subgroup_values_list.get(i).cloned().unwrap_or_default()),
    }).collect();

    let xbar_violations: Vec<usize> = xbar_points.iter().enumerate()
        .filter(|(_, p)| p.value > xbar_limits.ucl || p.value < xbar_limits.lcl)
        .map(|(i, _)| i)
        .collect();

    let xbar_in_control = if !xbar_points.is_empty() {
        (1.0 - xbar_violations.len() as f64 / xbar_points.len() as f64) * 100.0
    } else { 100.0 };

    // R chart
    let r_limits = ControlLimits {
        ucl: d4 * r_avg,
        cl: r_avg,
        lcl: d3 * r_avg,
    };

    let r_points: Vec<SpcPoint> = ranges.iter().enumerate().map(|(i, &v)| SpcPoint {
        index: i,
        value: v,
        timestamp: subgroup_timestamps.get(i).copied().unwrap_or(0),
        subgroup_values: None,
    }).collect();

    let r_violations: Vec<usize> = r_points.iter().enumerate()
        .filter(|(_, p)| p.value > r_limits.ucl || p.value < r_limits.lcl)
        .map(|(i, _)| i)
        .collect();

    let r_in_control = if !r_points.is_empty() {
        (1.0 - r_violations.len() as f64 / r_points.len() as f64) * 100.0
    } else { 100.0 };

    Ok((
        SpcResult {
            chart_type: "xbar".into(),
            data_points: xbar_points,
            control_limits: xbar_limits,
            violations: xbar_violations,
            in_control_percent: xbar_in_control,
        },
        SpcResult {
            chart_type: "r".into(),
            data_points: r_points,
            control_limits: r_limits,
            violations: r_violations,
            in_control_percent: r_in_control,
        },
    ))
}

/// Compute CUSUM chart data.
pub fn cusum(
    values: &[f64],
    target: f64,
    k: f64,
    h: f64,
    timestamps: &[i64],
) -> Result<SpcResult, String> {
    if values.is_empty() {
        return Err("No data points".into());
    }
    if k <= 0.0 || h <= 0.0 {
        return Err("k and h must be positive".into());
    }

    let mut cusum_pos = 0.0_f64;
    let mut cusum_neg = 0.0_f64;
    let mut points = Vec::with_capacity(values.len());
    let mut violations = Vec::new();

    for (i, &v) in values.iter().enumerate() {
        cusum_pos = (cusum_pos + (v - target) - k).max(0.0);
        cusum_neg = (cusum_neg + (v - target) + k).min(0.0);
        let val = cusum_pos.max(cusum_neg.abs());
        points.push(SpcPoint {
            index: i,
            value: val,
            timestamp: timestamps.get(i).copied().unwrap_or(0),
            subgroup_values: None,
        });
        if cusum_pos > h || cusum_neg.abs() > h {
            violations.push(i);
        }
    }

    let limits = ControlLimits { ucl: h, cl: 0.0, lcl: -h };
    let in_control = (1.0 - violations.len() as f64 / points.len() as f64) * 100.0;

    Ok(SpcResult {
        chart_type: "cusum".into(),
        data_points: points,
        control_limits: limits,
        violations,
        in_control_percent: in_control,
    })
}

/// Compute EWMA chart data.
pub fn ewma(
    values: &[f64],
    target: f64,
    lambda: f64,
    timestamps: &[i64],
) -> Result<SpcResult, String> {
    if values.is_empty() {
        return Err("No data points".into());
    }
    if lambda <= 0.0 || lambda > 1.0 {
        return Err("Lambda must be in (0, 1]".into());
    }

    let sigma = {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    };

    let mut z = target;
    let mut points = Vec::with_capacity(values.len());
    let mut violations = Vec::new();

    // EWMA variance grows: sigma_z = sigma * sqrt(lambda/(2-lambda) * (1 - (1-lambda)^(2*i)))
    let l = 3.0; // 3-sigma limits

    for (i, &v) in values.iter().enumerate() {
        z = lambda * v + (1.0 - lambda) * z;
        let sigma_z = sigma * (lambda / (2.0 - lambda) * (1.0 - (1.0 - lambda).powi(2 * (i + 1) as i32))).sqrt();
        let ucl = target + l * sigma_z;
        let lcl = target - l * sigma_z;

        points.push(SpcPoint {
            index: i,
            value: z,
            timestamp: timestamps.get(i).copied().unwrap_or(0),
            subgroup_values: None,
        });
        if z > ucl || z < lcl {
            violations.push(i);
        }
    }

    // Final limits for display
    let final_sigma_z = sigma * (lambda / (2.0 - lambda)).sqrt();
    let limits = ControlLimits {
        ucl: target + l * final_sigma_z,
        cl: target,
        lcl: target - l * final_sigma_z,
    };

    let in_control = (1.0 - violations.len() as f64 / points.len() as f64) * 100.0;

    Ok(SpcResult {
        chart_type: "ewma".into(),
        data_points: points,
        control_limits: limits,
        violations,
        in_control_percent: in_control,
    })
}

/// Compute P-chart (proportion defective).
pub fn p_chart(
    defectives: &[u32],
    sample_sizes: &[u32],
    timestamps: &[i64],
) -> Result<SpcResult, String> {
    if defectives.is_empty() {
        return Err("No data points".into());
    }
    if defectives.len() != sample_sizes.len() {
        return Err("defectives and sample_sizes must have same length".into());
    }

    let total_defective: u64 = defectives.iter().map(|&d| d as u64).sum();
    let total_inspected: u64 = sample_sizes.iter().map(|&n| n as u64).sum();
    let p_bar = total_defective as f64 / total_inspected as f64;

    let mut points = Vec::with_capacity(defectives.len());
    let mut violations = Vec::new();
    let mut ucl_sum = 0.0_f64;
    let mut lcl_sum = 0.0_f64;

    for (i, (&d, &n)) in defectives.iter().zip(sample_sizes.iter()).enumerate() {
        let p = d as f64 / n as f64;
        let se = (p_bar * (1.0 - p_bar) / n as f64).sqrt();
        let ucl = p_bar + 3.0 * se;
        let lcl = (p_bar - 3.0 * se).max(0.0);
        ucl_sum += ucl;
        lcl_sum += lcl;

        points.push(SpcPoint {
            index: i,
            value: p,
            timestamp: timestamps.get(i).copied().unwrap_or(0),
            subgroup_values: None,
        });
        if p > ucl || p < lcl {
            violations.push(i);
        }
    }

    let n = points.len() as f64;
    let limits = ControlLimits {
        ucl: ucl_sum / n,
        cl: p_bar,
        lcl: lcl_sum / n,
    };

    let in_control = (1.0 - violations.len() as f64 / points.len() as f64) * 100.0;

    Ok(SpcResult {
        chart_type: "p".into(),
        data_points: points,
        control_limits: limits,
        violations,
        in_control_percent: in_control,
    })
}

/// Compute NP-chart (count defective).
pub fn np_chart(
    defectives: &[u32],
    sample_size: u32,
    timestamps: &[i64],
) -> Result<SpcResult, String> {
    if defectives.is_empty() {
        return Err("No data points".into());
    }
    if sample_size == 0 {
        return Err("Sample size must be positive".into());
    }

    let p_bar = defectives.iter().map(|&d| d as f64).sum::<f64>()
        / (defectives.len() as f64 * sample_size as f64);
    let np_bar = sample_size as f64 * p_bar;

    let ucl = np_bar + 3.0 * (np_bar * (1.0 - p_bar)).sqrt();
    let lcl = (np_bar - 3.0 * (np_bar * (1.0 - p_bar)).sqrt()).max(0.0);

    let mut points = Vec::with_capacity(defectives.len());
    let mut violations = Vec::new();

    for (i, &d) in defectives.iter().enumerate() {
        points.push(SpcPoint {
            index: i,
            value: d as f64,
            timestamp: timestamps.get(i).copied().unwrap_or(0),
            subgroup_values: None,
        });
        if d as f64 > ucl || d as f64 < lcl {
            violations.push(i);
        }
    }

    let in_control = (1.0 - violations.len() as f64 / points.len() as f64) * 100.0;

    Ok(SpcResult {
        chart_type: "np".into(),
        data_points: points,
        control_limits: ControlLimits { ucl, cl: np_bar, lcl },
        violations,
        in_control_percent: in_control,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_timestamps(n: usize) -> Vec<i64> {
        (0..n).map(|i| 1000 + i as i64 * 60).collect()
    }

    #[test]
    fn test_xbar_r_basic() {
        // 20 values, subgroup_size=4 → 5 subgroups
        let values: Vec<f64> = vec![10.0, 12.0, 11.0, 13.0, 14.0, 10.0, 12.0, 11.0, 13.0, 14.0, 10.0, 12.0, 11.0, 13.0, 10.0, 12.0, 11.0, 13.0, 14.0, 10.0];
        let timestamps = make_timestamps(20);
        let (xbar, r) = xbar_r(&values, 4, &timestamps).unwrap();
        assert_eq!(xbar.data_points.len(), 5);
        assert_eq!(r.data_points.len(), 5);
        assert_eq!(xbar.chart_type, "xbar");
        assert_eq!(r.chart_type, "r");
        // X-bar of first subgroup: (10+12+11+13)/4 = 11.5
        assert!((xbar.data_points[0].value - 11.5).abs() < 0.001);
        // R of first subgroup: 13-10 = 3
        assert!((r.data_points[0].value - 3.0).abs() < 0.001);
        // All in control for this stable data
        assert!(xbar.in_control_percent > 0.0);
    }

    #[test]
    fn test_xbar_r_invalid_subgroup() {
        let values = vec![1.0, 2.0];
        let ts = make_timestamps(2);
        assert!(xbar_r(&values, 0, &ts).is_err());
        assert!(xbar_r(&values, 11, &ts).is_err());
    }

    #[test]
    fn test_cusum_basic() {
        // Values centered around target=10, should stay in control
        let values: Vec<f64> = vec![10.1, 9.9, 10.0, 10.2, 9.8, 10.1, 9.9, 10.0];
        let ts = make_timestamps(8);
        let result = cusum(&values, 10.0, 0.5, 5.0, &ts).unwrap();
        assert_eq!(result.data_points.len(), 8);
        assert_eq!(result.chart_type, "cusum");
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_ewma_basic() {
        let values: Vec<f64> = vec![10.0, 10.1, 9.9, 10.0, 10.2, 9.8, 10.1, 10.0];
        let ts = make_timestamps(8);
        let result = ewma(&values, 10.0, 0.2, &ts).unwrap();
        assert_eq!(result.data_points.len(), 8);
        assert_eq!(result.chart_type, "ewma");
    }

    #[test]
    fn test_p_chart_basic() {
        let defectives = vec![2, 3, 1, 4, 2, 3, 1, 2];
        let sample_sizes = vec![50; 8];
        let ts = make_timestamps(8);
        let result = p_chart(&defectives, &sample_sizes, &ts).unwrap();
        assert_eq!(result.data_points.len(), 8);
        assert_eq!(result.chart_type, "p");
        assert!(result.control_limits.cl > 0.0);
    }

    #[test]
    fn test_np_chart_basic() {
        let defectives = vec![2, 3, 1, 4, 2, 3, 1, 2];
        let ts = make_timestamps(8);
        let result = np_chart(&defectives, 50, &ts).unwrap();
        assert_eq!(result.data_points.len(), 8);
        assert_eq!(result.chart_type, "np");
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib analytics::spc`
Expected: All 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/analytics/spc.rs
git commit -m "feat(analytics): implement SPC calculations (X-bar/R, CUSUM, EWMA, P, NP)"
```

---

## Task 4: Implement histogram calculations

**Files:**
- Create: `src/analytics/histogram.rs`

- [ ] **Step 1: Write histogram calculation code and tests in `src/analytics/histogram.rs`**

```rust
use crate::models::{HistogramResult, HistogramBin, CurvePoint, DistributionStats, CapabilityIndices};

/// Compute histogram bins from raw values.
pub fn compute_histogram(
    values: &[f64],
    num_bins: u32,
    lsl: Option<f64>,
    usl: Option<f64>,
) -> Result<HistogramResult, String> {
    if values.is_empty() {
        return Err("No data points".into());
    }
    if num_bins < 2 {
        return Err("Need at least 2 bins".into());
    }

    let stats = compute_distribution_stats(values);

    // Compute bin edges
    let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max_val - min_val;
    let bin_width = if range > 0.0 { range / num_bins as f64 } else { 1.0 };

    let mut bins: Vec<HistogramBin> = (0..num_bins)
        .map(|i| HistogramBin {
            lower: min_val + i as f64 * bin_width,
            upper: min_val + (i + 1) as f64 * bin_width,
            count: 0,
            frequency: 0.0,
        })
        .collect();

    // Assign values to bins
    for &v in values {
        let idx = if range > 0.0 {
            ((v - min_val) / bin_width).floor() as usize
        } else {
            0
        };
        let idx = idx.min(num_bins as usize - 1);
        bins[idx].count += 1;
    }

    // Normalize frequencies
    let total = values.len() as f64;
    for bin in &mut bins {
        bin.frequency = bin.count as f64 / total;
    }

    // Normal curve overlay
    let normal_curve = compute_normal_curve(&stats, min_val, max_val, 100);

    // Capability indices (only if both spec limits provided)
    let capability = match (lsl, usl) {
        (Some(lsl_val), Some(usl_val)) => {
            let sigma_within = stats.stddev;
            let cp = (usl_val - lsl_val) / (6.0 * sigma_within);
            let cpk = ((usl_val - stats.mean).min(stats.mean - lsl_val)) / (3.0 * sigma_within);
            // Pp/Ppk use overall stddev (same here since we don't separate within/overall)
            let pp = cp;
            let ppk = cpk;
            Some(CapabilityIndices { cp, cpk, pp, ppk })
        }
        _ => None,
    };

    Ok(HistogramResult {
        bins,
        normal_curve,
        stats,
        capability,
    })
}

fn compute_distribution_stats(values: &[f64]) -> DistributionStats {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;

    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();

    let m3 = values.iter().map(|v| ((v - mean) / stddev).powi(3)).sum::<f64>() / n;
    let skewness = if stddev > 0.0 { m3 } else { 0.0 };

    let m4 = values.iter().map(|v| ((v - mean) / stddev).powi(4)).sum::<f64>() / n;
    let kurtosis = if stddev > 0.0 { m4 - 3.0 } else { 0.0 }; // Excess kurtosis

    DistributionStats {
        mean,
        stddev,
        skewness,
        kurtosis,
        count: values.len() as u64,
    }
}

fn compute_normal_curve(stats: &DistributionStats, min: f64, max: f64, points: usize) -> Vec<CurvePoint> {
    let mut result = Vec::with_capacity(points);
    let range = max - min;
    if range <= 0.0 || stats.stddev <= 0.0 {
        return result;
    }

    for i in 0..=points {
        let x = min + (i as f64 / points as f64) * range;
        let z = (x - stats.mean) / stats.stddev;
        let y = (-0.5 * z * z).exp() / (stats.stddev * (2.0 * std::f64::consts::PI).sqrt());
        result.push(CurvePoint { x, y });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_basic() {
        // 100 values from normal-ish distribution
        let values: Vec<f64> = (0..100).map(|i| {
            let x = (i as f64 - 50.0) / 10.0;
            10.0 + x + 0.1 * (x * 3.0).sin()
        }).collect();
        let result = compute_histogram(&values, 10, None, None).unwrap();
        assert_eq!(result.bins.len(), 10);
        assert!(result.stats.count == 100);
        assert!(result.capability.is_none());
        // All values should be in some bin
        let total: u64 = result.bins.iter().map(|b| b.count).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_histogram_with_spec_limits() {
        let values = vec![10.0, 11.0, 12.0, 10.5, 11.5, 10.2, 11.8, 10.8, 11.2, 10.9];
        let result = compute_histogram(&values, 5, Some(9.0), Some(13.0)).unwrap();
        assert!(result.capability.is_some());
        let cap = result.capability.unwrap();
        assert!(cap.cp > 0.0);
        assert!(cap.cpk > 0.0);
    }

    #[test]
    fn test_histogram_empty() {
        let result = compute_histogram(&[], 10, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_distribution_stats() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let stats = compute_distribution_stats(&values);
        assert!((stats.mean - 5.0).abs() < 0.001);
        assert!(stats.stddev > 0.0);
        assert!(stats.count == 8);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib analytics::histogram`
Expected: All 4 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/analytics/histogram.rs
git commit -m "feat(analytics): implement histogram with capability indices"
```

---

## Task 5: Implement run chart calculations

**Files:**
- Create: `src/analytics/runchart.rs`

- [ ] **Step 1: Write run chart code and tests in `src/analytics/runchart.rs`**

```rust
use crate::models::{RunChartResult, RunChartPoint, Signal};

/// Compute run chart data with signal detection.
pub fn compute_run_chart(
    values: &[f64],
    timestamps: &[i64],
) -> Result<RunChartResult, String> {
    if values.is_empty() {
        return Err("No data points".into());
    }
    if values.len() != timestamps.len() {
        return Err("values and timestamps must have same length".into());
    }

    // Compute median
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };

    // Build points
    let points: Vec<RunChartPoint> = values.iter().zip(timestamps.iter())
        .map(|(&v, &ts)| RunChartPoint {
            timestamp: ts,
            value: v,
            above_median: v > median,
        })
        .collect();

    // Detect signals
    let mut signals = Vec::new();

    // Rule 1: Runs — 8+ consecutive points on one side of median
    let mut run_start = 0;
    let mut run_above = points.get(0).map(|p| p.above_median).unwrap_or(false);
    for (i, p) in points.iter().enumerate() {
        if p.above_median != run_above || i == points.len() - 1 {
            let end = if p.above_median != run_above { i - 1 } else { i };
            let run_len = end - run_start + 1;
            if run_len >= 8 {
                signals.push(Signal {
                    rule: "run".into(),
                    start_idx: run_start,
                    end_idx: end,
                    description: format!(
                        "{} consecutive points {} median",
                        run_len,
                        if run_above { "above" } else { "below" }
                    ),
                });
            }
            run_start = i;
            run_above = p.above_median;
        }
    }

    // Rule 2: Trends — 6+ consecutive increasing or decreasing
    let mut trend_start = 0;
    let mut trend_dir = 0; // 1 = increasing, -1 = decreasing
    for i in 1..points.len() {
        let cmp = values[i].partial_cmp(&values[i - 1]).unwrap_or(std::cmp::Ordering::Equal);
        let dir = match cmp {
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
        };
        if dir == 0 || dir != trend_dir {
            let trend_len = i - trend_start;
            if trend_len >= 6 && trend_dir != 0 {
                signals.push(Signal {
                    rule: "trend".into(),
                    start_idx: trend_start,
                    end_idx: i - 1,
                    description: format!(
                        "{} consecutive {} points",
                        trend_len,
                        if trend_dir > 0 { "increasing" } else { "decreasing" }
                    ),
                });
            }
            if dir != 0 {
                trend_start = i - 1;
                trend_dir = dir;
            } else {
                trend_start = i;
                trend_dir = 0;
            }
        }
    }
    // Check final trend
    let final_trend_len = points.len() - trend_start;
    if final_trend_len >= 6 && trend_dir != 0 {
        signals.push(Signal {
            rule: "trend".into(),
            start_idx: trend_start,
            end_idx: points.len() - 1,
            description: format!(
                "{} consecutive {} points",
                final_trend_len,
                if trend_dir > 0 { "increasing" } else { "decreasing" }
            ),
        });
    }

    // Count runs (groups of consecutive above/below median)
    let runs_count = count_runs(&points);

    Ok(RunChartResult {
        points,
        median,
        signals,
        runs_count,
    })
}

fn count_runs(points: &[RunChartPoint]) -> u32 {
    if points.is_empty() {
        return 0;
    }
    let mut count = 1;
    for i in 1..points.len() {
        if points[i].above_median != points[i - 1].above_median {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(n: usize) -> Vec<i64> {
        (0..n).map(|i| 1000 + i as i64 * 60).collect()
    }

    #[test]
    fn test_run_chart_basic() {
        let values = vec![10.0, 11.0, 12.0, 10.5, 9.5, 10.0, 11.0, 9.0, 10.5, 10.0];
        let timestamps = ts(10);
        let result = compute_run_chart(&values, &timestamps).unwrap();
        assert_eq!(result.points.len(), 10);
        assert!(result.median > 0.0);
        assert!(result.runs_count >= 1);
    }

    #[test]
    fn test_run_chart_detects_run() {
        // 10 points all above median (0-9), median should be 5, all above
        let values: Vec<f64> = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
        let timestamps = ts(10);
        let result = compute_run_chart(&values, &timestamps).unwrap();
        let run_signals: Vec<_> = result.signals.iter().filter(|s| s.rule == "run").collect();
        assert!(!run_signals.is_empty(), "Should detect a run signal");
    }

    #[test]
    fn test_run_chart_detects_trend() {
        let values: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let timestamps = ts(10);
        let result = compute_run_chart(&values, &timestamps).unwrap();
        let trend_signals: Vec<_> = result.signals.iter().filter(|s| s.rule == "trend").collect();
        assert!(!trend_signals.is_empty(), "Should detect a trend signal");
    }

    #[test]
    fn test_run_chart_empty() {
        assert!(compute_run_chart(&[], &[]).is_err());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib analytics::runchart`
Expected: All 4 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/analytics/runchart.rs
git commit -m "feat(analytics): implement run chart with signal detection"
```

---

## Task 6: Implement custom metric evaluation

**Files:**
- Create: `src/analytics/custom.rs`

- [ ] **Step 1: Write custom metric code and tests in `src/analytics/custom.rs`**

```rust
use std::collections::HashMap;
use anyhow::Result;

use crate::cache::DeviceCache;
use crate::expression;
use crate::models::{CustomMetric, CustomMetricValue};
use crate::storage::Storage;
use crate::timeutil;

/// Parse a formula and extract tag references (identifiers that aren't numbers).
pub fn extract_tag_refs(formula: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = formula.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' => {
                if !current.is_empty() {
                    refs.push(current.clone());
                    current.clear();
                }
                i += 1;
            }
            '+' | '-' | '*' | '/' | '(' | ')' | ',' | '.' => {
                if !current.is_empty() {
                    refs.push(current.clone());
                    current.clear();
                }
                // Check for >=, <=, ==, &&, ||
                if i + 1 < chars.len() && (chars[i + 1] == '=' || chars[i] == '&' || chars[i] == '|') {
                    if chars[i + 1] == '=' || chars[i + 1] == '&' || chars[i + 1] == '|' {
                        i += 2;
                        continue;
                    }
                }
                i += 1;
            }
            '>' | '<' | '=' | '&' | '|' => {
                if !current.is_empty() {
                    refs.push(current.clone());
                    current.clear();
                }
                if i + 1 < chars.len() && chars[i + 1] == '=' { i += 2; } else { i += 1; }
            }
            c if c.is_ascii_digit() => {
                // Part of a number — don't treat as tag ref
                if !current.is_empty() && !current.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    current.clear();
                }
                current.push(c);
                i += 1;
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                current.push(c);
                i += 1;
            }
            _ => { i += 1; }
        }
    }
    if !current.is_empty() {
        refs.push(current);
    }

    // Filter out things that look like numbers
    refs.into_iter()
        .filter(|r| !r.parse::<f64>().is_ok())
        .filter(|r| !r.is_empty())
        .collect()
}

/// Validate a formula by attempting to parse it.
pub fn validate_formula(formula: &str) -> Result<(), String> {
    let test_values: HashMap<String, f64> = extract_tag_refs(formula)
        .into_iter()
        .map(|r| (r, 1.0))
        .collect();

    expression::evaluate(formula, &test_values)?;
    Ok(())
}

/// Build a values map from the device cache for the given tag references.
pub async fn resolve_tag_values(cache: &DeviceCache, tag_refs: &[String]) -> HashMap<String, f64> {
    let all = cache.get_all().await;
    let mut values = HashMap::new();
    for tag in tag_refs {
        for snapshot in &all {
            if let Some(&v) = snapshot.metrics.get(tag) {
                values.insert(tag.clone(), v);
                break;
            }
        }
    }
    values
}

/// Evaluate a custom metric formula against live cache data.
pub async fn evaluate_metric(
    cache: &DeviceCache,
    metric: &CustomMetric,
) -> Result<CustomMetricValue> {
    let tag_refs: Vec<String> = serde_json::from_str(&metric.tag_refs)
        .unwrap_or_default();

    let tag_values = resolve_tag_values(cache, &tag_refs).await;

    let value = expression::evaluate(&metric.formula, &tag_values)
        .map_err(|e| anyhow::anyhow!("Formula evaluation failed: {}", e))?;

    Ok(CustomMetricValue {
        id: metric.id,
        name: metric.name.clone(),
        value,
        unit: metric.unit.clone(),
        computed_at: timeutil::now_secs(),
        tag_values,
    })
}

/// Evaluate all custom metrics.
pub async fn evaluate_all_metrics(
    storage: &Storage,
    cache: &DeviceCache,
) -> Result<Vec<CustomMetricValue>> {
    let metrics = storage.list_custom_metrics().await?;
    let mut results = Vec::with_capacity(metrics.len());

    for metric in &metrics {
        if metric.validation_error.is_some() {
            continue; // Skip invalid formulas
        }
        match evaluate_metric(cache, metric).await {
            Ok(val) => results.push(val),
            Err(e) => tracing::warn!("Failed to evaluate metric '{}': {}", metric.name, e),
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tag_refs() {
        let refs = extract_tag_refs("PUMP_FLOW / PUMP_RATED * 100");
        assert_eq!(refs, vec!["PUMP_FLOW", "PUMP_RATED"]);
    }

    #[test]
    fn test_extract_tag_refs_with_numbers() {
        let refs = extract_tag_refs("temp * 1.8 + 32");
        // Should extract "temp" but not "1.8" or "32"
        assert!(refs.contains(&"temp".to_string()));
        assert!(!refs.contains(&"1.8".to_string()));
        assert!(!refs.contains(&"32".to_string()));
    }

    #[test]
    fn test_validate_formula_valid() {
        assert!(validate_formula("A + B * C").is_ok());
    }

    #[test]
    fn test_validate_formula_division_by_zero() {
        // This is valid syntactically, division by zero happens at runtime
        assert!(validate_formula("A / 0").is_ok());
    }

    #[test]
    fn test_evaluate_simple() {
        let mut values = HashMap::new();
        values.insert("A".into(), 10.0);
        values.insert("B".into(), 2.0);
        let result = expression::evaluate("A / B * 100", &values).unwrap();
        assert!((result - 500.0).abs() < 0.001);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib analytics::custom`
Expected: All 5 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/analytics/custom.rs
git commit -m "feat(analytics): implement custom metric formula evaluation"
```

---

## Task 7: Add API endpoints and register routes

**Files:**
- Modify: `src/api/analytics.rs` — add SPC, histogram, run chart, custom metric endpoints
- Modify: `src/api/mod.rs` — register new routes
- Modify: `src/api/openapi.rs` — register new paths and schemas

- [ ] **Step 1: Add new API endpoints to `src/api/analytics.rs`**

Append the following to the file (after the existing `get_compare` handler):

```rust
use std::collections::HashMap;
use serde::Deserialize;
use crate::analytics::{spc, histogram, runchart, custom};
use crate::cache::DeviceCache;

#[derive(Debug, Deserialize)]
pub struct SpcQuery {
    pub device_id: String,
    pub metric: String,
    pub from: i64,
    pub to: i64,
    pub chart_type: String, // "xbar_r", "cusum", "ewma", "p", "np"
    pub subgroup_size: Option<u32>,
    pub target: Option<f64>,
    pub lambda: Option<f64>,
    pub k: Option<f64>,
    pub h: Option<f64>,
}

/// GET /api/v1/analytics/spc
#[utoipa::path(
    get,
    path = "/api/v1/analytics/spc",
    params(
        ("device_id" = String, Query, description = "Device ID"),
        ("metric" = String, Query, description = "Metric name"),
        ("from" = i64, Query, description = "Start timestamp"),
        ("to" = i64, Query, description = "End timestamp"),
        ("chart_type" = String, Query, description = "Chart type: xbar_r, cusum, ewma, p, np"),
        ("subgroup_size" = Option<u32>, Query, description = "Subgroup size (2-10, for xbar_r)"),
        ("target" = Option<f64>, Query, description = "Target value (for cusum, ewma)"),
        ("lambda" = Option<f64>, Query, description = "Smoothing factor 0.05-0.3 (for ewma)"),
        ("k" = Option<f64>, Query, description = "Reference value (for cusum)"),
        ("h" = Option<f64>, Query, description = "Decision interval (for cusum)"),
    ),
    responses(
        (status = 200, description = "SPC chart data"),
        (status = 400, description = "Invalid parameters"),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_spc(
    State(state): State<AppState>,
    Query(q): Query<SpcQuery>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!("SPC requires Professional tier")));
    }

    let data = state.storage.get_telemetry_series(
        &q.device_id, &q.metric, q.from, q.to, Some(10000),
    ).await?;

    let timestamps: Vec<i64> = data.iter().map(|(ts, _)| *ts).collect();
    let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();

    if values.is_empty() {
        return Err(AppError(anyhow::anyhow!("No data found for the specified range")));
    }

    let result = match q.chart_type.as_str() {
        "xbar_r" => {
            let sg = q.subgroup_size.unwrap_or(5).min(10).max(2) as usize;
            let (xbar, _r) = spc::xbar_r(&values, sg, &timestamps)
                .map_err(|e| AppError(anyhow::anyhow!(e)))?;
            serde_json::to_value(xbar)
        }
        "cusum" => {
            let target = q.target.unwrap_or_else(|| {
                values.iter().sum::<f64>() / values.len() as f64
            });
            let k = q.k.unwrap_or(0.5);
            let h = q.h.unwrap_or(5.0);
            let result = spc::cusum(&values, target, k, h, &timestamps)
                .map_err(|e| AppError(anyhow::anyhow!(e)))?;
            serde_json::to_value(result)
        }
        "ewma" => {
            let target = q.target.unwrap_or_else(|| {
                values.iter().sum::<f64>() / values.len() as f64
            });
            let lambda = q.lambda.unwrap_or(0.2).max(0.05).min(1.0);
            let result = spc::ewma(&values, target, lambda, &timestamps)
                .map_err(|e| AppError(anyhow::anyhow!(e)))?;
            serde_json::to_value(result)
        }
        _ => return Err(AppError(anyhow::anyhow!("Unsupported chart_type: {}. Use xbar_r, cusum, or ewma.", q.chart_type))),
    }.map_err(|e| AppError(anyhow::anyhow!("Serialization error: {}", e)))?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct HistogramQuery {
    pub device_id: String,
    pub metric: String,
    pub from: i64,
    pub to: i64,
    pub bins: Option<u32>,
    pub lsl: Option<f64>,
    pub usl: Option<f64>,
}

/// GET /api/v1/analytics/histogram
#[utoipa::path(
    get,
    path = "/api/v1/analytics/histogram",
    params(
        ("device_id" = String, Query),
        ("metric" = String, Query),
        ("from" = i64, Query),
        ("to" = i64, Query),
        ("bins" = Option<u32>, Query, description = "Number of bins (5-100, default 20)"),
        ("lsl" = Option<f64>, Query, description = "Lower specification limit"),
        ("usl" = Option<f64>, Query, description = "Upper specification limit"),
    ),
    responses((status = 200, description = "Histogram data"), (status = 403, description = "License tier too low")),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_histogram(
    State(state): State<AppState>,
    Query(q): Query<HistogramQuery>,
) -> Result<Json<crate::models::HistogramResult>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!("Analytics requires Starter tier or above"));
    }

    let data = state.storage.get_telemetry_series(
        &q.device_id, &q.metric, q.from, q.to, None,
    ).await?;

    let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();

    let bins = q.bins.unwrap_or(20).max(5).min(100);
    let result = histogram::compute_histogram(&values, bins, q.lsl, q.usl)
        .map_err(|e| AppError(anyhow::anyhow!(e)))?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct RunChartQuery {
    pub device_id: String,
    pub metric: String,
    pub from: i64,
    pub to: i64,
}

/// GET /api/v1/analytics/run-chart
#[utoipa::path(
    get,
    path = "/api/v1/analytics/run-chart",
    params(
        ("device_id" = String, Query), ("metric" = String, Query),
        ("from" = i64, Query), ("to" = i64, Query),
    ),
    responses((status = 200, description = "Run chart data"), (status = 403, description = "License tier too low")),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_run_chart(
    State(state): State<AppState>,
    Query(q): Query<RunChartQuery>,
) -> Result<Json<crate::models::RunChartResult>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!("Analytics requires Starter tier or above"));
    }

    let data = state.storage.get_telemetry_series(
        &q.device_id, &q.metric, q.from, q.to, None,
    ).await?;

    let timestamps: Vec<i64> = data.iter().map(|(ts, _)| *ts).collect();
    let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();

    let result = runchart::compute_run_chart(&values, &timestamps)
        .map_err(|e| AppError(anyhow::anyhow!(e)))?;

    Ok(Json(result))
}

// ── Custom Metrics CRUD ──

/// GET /api/v1/analytics/custom-metrics
#[utoipa::path(
    get, path = "/api/v1/analytics/custom-metrics",
    responses((status = 200, body = Vec<crate::models::CustomMetric>)),
    tag = "Analytics", security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_custom_metrics(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::CustomMetric>>, AppError> {
    let metrics = state.storage.list_custom_metrics().await?;
    Ok(Json(metrics))
}

/// POST /api/v1/analytics/custom-metrics
#[utoipa::path(
    post, path = "/api/v1/analytics/custom-metrics",
    request_body = CreateCustomMetricRequest,
    responses((status = 200, description = "Created"), (status = 400, description = "Invalid formula")),
    tag = "Analytics", security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn create_custom_metric(
    State(state): State<AppState>,
    Json(input): Json<crate::models::CreateCustomMetricRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!("Custom metrics require Professional tier")));
    }

    let tag_refs = custom::extract_tag_refs(&input.formula);
    let tag_refs_json = serde_json::to_string(&tag_refs).unwrap_or("[]".into());

    let validation_error = match custom::validate_formula(&input.formula) {
        Ok(()) => None,
        Err(e) => Some(e),
    };

    let id = state.storage.create_custom_metric(
        &input.name, &input.description, &input.formula,
        input.unit.as_deref(), &tag_refs_json, validation_error.as_deref(),
    ).await?;

    Ok(Json(serde_json::json!({ "id": id })))
}

/// PUT /api/v1/analytics/custom-metrics/{id}
#[utoipa::path(
    put, path = "/api/v1/analytics/custom-metrics/{id}",
    request_body = UpdateCustomMetricRequest,
    responses((status = 200, description = "Updated")),
    tag = "Analytics", security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_custom_metric(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<crate::models::UpdateCustomMetricRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tag_refs = input.formula.as_ref().map(|f| {
        let refs = custom::extract_tag_refs(f);
        serde_json::to_string(&refs).unwrap_or("[]".into())
    });

    let validation_error = match &input.formula {
        Some(f) => match custom::validate_formula(f) {
            Ok(()) => None,
            Err(e) => Some(e),
        },
        None => None,
    };

    state.storage.update_custom_metric(
        id, input.name.as_deref(), input.description.as_deref(),
        input.formula.as_deref(), input.unit.as_deref(),
        tag_refs.as_deref(), validation_error.as_deref(),
    ).await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

/// DELETE /api/v1/analytics/custom-metrics/{id}
#[utoipa::path(
    delete, path = "/api/v1/analytics/custom-metrics/{id}",
    responses((status = 200, description = "Deleted")),
    tag = "Analytics", security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn delete_custom_metric(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.storage.delete_custom_metric(id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// GET /api/v1/analytics/custom-metrics/evaluate-all
#[utoipa::path(
    get, path = "/api/v1/analytics/custom-metrics/evaluate-all",
    responses((status = 200, body = Vec<crate::models::CustomMetricValue>)),
    tag = "Analytics", security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn evaluate_all_custom_metrics(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::CustomMetricValue>>, AppError> {
    let results = custom::evaluate_all_metrics(&state.storage, &state.cache).await?;
    Ok(Json(results))
}
```

Also add the missing imports at the top of the file. The existing imports already have `axum::extract::Query`, `State`, `Json`, `Deserialize`. Add:
```rust
use axum::extract::Path;
```

- [ ] **Step 2: Register new routes in `src/api/mod.rs`**

Find the `protected` router section (around line 260) where the existing analytics routes are. Add after the existing analytics routes:

```rust
        .route("/api/v1/analytics/spc", get(analytics::get_spc))
        .route("/api/v1/analytics/histogram", get(analytics::get_histogram))
        .route("/api/v1/analytics/run-chart", get(analytics::get_run_chart))
        .route("/api/v1/analytics/custom-metrics", get(analytics::list_custom_metrics))
        .route("/api/v1/analytics/custom-metrics/evaluate-all", get(analytics::evaluate_all_custom_metrics))
```

Find the `admin_protected` router section and add:

```rust
        .route("/api/v1/analytics/custom-metrics", post(analytics::create_custom_metric))
        .route("/api/v1/analytics/custom-metrics/{id}", put(analytics::update_custom_metric).delete(analytics::delete_custom_metric))
```

- [ ] **Step 3: Register new paths and schemas in `src/api/openapi.rs`**

Find the `#[openapi(paths(...))]` attribute and add the new handlers:

```rust
crate::api::analytics::get_spc,
crate::api::analytics::get_histogram,
crate::api::analytics::get_run_chart,
crate::api::analytics::list_custom_metrics,
crate::api::analytics::create_custom_metric,
crate::api::analytics::update_custom_metric,
crate::api::analytics::delete_custom_metric,
crate::api::analytics::evaluate_all_custom_metrics,
```

Find the `components(schemas(...)))` attribute and add the new types:

```rust
crate::models::CustomMetric,
crate::models::CustomMetricValue,
crate::models::CreateCustomMetricRequest,
crate::models::UpdateCustomMetricRequest,
crate::models::SpcResult,
crate::models::SpcPoint,
crate::models::ControlLimits,
crate::models::HistogramResult,
crate::models::HistogramBin,
crate::models::CurvePoint,
crate::models::DistributionStats,
crate::models::CapabilityIndices,
crate::models::RunChartResult,
crate::models::RunChartPoint,
crate::models::Signal,
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check`
Expected: Compiles with no errors

- [ ] **Step 5: Run all tests**

Run: `cargo test --lib`
Expected: All tests pass (existing + new)

- [ ] **Step 6: Commit**

```bash
git add src/api/analytics.rs src/api/mod.rs src/api/openapi.rs
git commit -m "feat(analytics): add SPC, histogram, run chart, custom metrics API endpoints"
```

---

## Task 8: Extend report scheduler for analytics digest

**Files:**
- Modify: `src/report_scheduler.rs` — handle analytics_digest type
- Modify: `src/report.rs` — add analytics digest HTML generation

- [ ] **Step 1: Read current report.rs to understand the HTML generation pattern**

Read `src/report.rs` to understand how HTML reports are generated. The pattern will be followed for the analytics digest.

- [ ] **Step 2: Add analytics digest generation function to `src/report.rs`**

Add a function `generate_analytics_digest_html` that takes the storage, cache, and a JSON config string. It generates an HTML email body with:

- Header with report name and date range
- Summary cards for each tracked metric
- SPC violation alerts if any
- Custom metric current values
- Period comparison summary

The function queries storage for the relevant data and renders inline HTML. No external template engine.

- [ ] **Step 3: Modify `src/report_scheduler.rs` to dispatch analytics digest**

In the `process_reports` function, after loading a report, check if `report.report_type == "analytics_digest"` (add `report_type` field to the `Report` model if not present). If so, call the analytics digest generation function instead of the normal report generation.

- [ ] **Step 4: Add `report_type` field to the Report model in `src/models.rs`**

Add `pub report_type: Option<String>` to the `Report` struct. Update the storage queries to handle the new column with a default of 'pdf'.

- [ ] **Step 5: Run cargo check and tests**

Run: `cargo check && cargo test --lib`
Expected: Compiles, all tests pass

- [ ] **Step 6: Commit**

```bash
git add src/report.rs src/report_scheduler.rs src/models.rs src/storage.rs
git commit -m "feat(analytics): add analytics digest report type to scheduler"
```

---

## Task 9: Frontend — SPC chart component

**Files:**
- Create: `client/src/components/SPCChart.tsx`

- [ ] **Step 1: Create `SPCChart.tsx`**

Canvas 2D component that:
- Takes `SpcResult` data as prop
- Renders a line chart with data points
- Draws UCL/CL/LCL as horizontal dashed lines
- Highlights violation points in red
- Shows chart type and in-control percentage as header

Follows the existing Canvas 2D pattern from `Chart.tsx` — uses `useRef<HTMLCanvasElement>`, `useEffect` for drawing, and `useCanvasSize` hook.

- [ ] **Step 2: Commit**

```bash
git add client/src/components/SPCChart.tsx
git commit -m "feat(analytics): add SPC chart Canvas component"
```

---

## Task 10: Frontend — Histogram chart component

**Files:**
- Create: `client/src/components/HistogramChart.tsx`

- [ ] **Step 1: Create `HistogramChart.tsx`**

Canvas 2D component that:
- Takes `HistogramResult` data as prop
- Renders vertical bars for each bin
- Overlays normal distribution curve as a smooth line
- Draws LSL/USL as vertical dashed lines if present
- Shows capability indices (Cp, Cpk, Pp, Ppk) as stat cards below

- [ ] **Step 2: Commit**

```bash
git add client/src/components/HistogramChart.tsx
git commit -m "feat(analytics): add histogram Canvas component"
```

---

## Task 11: Frontend — Run chart and Calc builder components

**Files:**
- Create: `client/src/components/RunChart.tsx`
- Create: `client/src/components/CalcBuilder.tsx`

- [ ] **Step 1: Create `RunChart.tsx`**

Canvas 2D component with:
- Line chart of individual values
- Median as horizontal dashed line
- Signal regions highlighted with colored background

- [ ] **Step 2: Create `CalcBuilder.tsx`**

React component with:
- Text input for formula
- Tag autocomplete (fetched from data sources API)
- Live preview showing current evaluation result
- Name, description, unit fields
- List of existing custom metrics with edit/delete

- [ ] **Step 3: Commit**

```bash
git add client/src/components/RunChart.tsx client/src/components/CalcBuilder.tsx
git commit -m "feat(analytics): add run chart and calc builder components"
```

---

## Task 12: Frontend — Refactor AnalyticsPanel + query hooks

**Files:**
- Modify: `client/src/AnalyticsPanel.tsx` — tabbed layout
- Modify: `client/src/queries/analytics.ts` — new hooks

- [ ] **Step 1: Add new query hooks to `client/src/queries/analytics.ts`**

Add interfaces and hooks for SPC, histogram, run chart, custom metrics, matching the backend API responses.

- [ ] **Step 2: Refactor `AnalyticsPanel.tsx` into tabbed layout**

Replace the current single-view panel with a tabbed interface:
- Tabs: Overview | SPC | Histogram | Run Chart | Compare | Custom Metrics
- Each tab renders its respective component
- Shared device/metric/range selectors persist across tabs

- [ ] **Step 3: Build and verify**

Run: `cd client && npm run build`
Expected: Builds successfully

- [ ] **Step 4: Commit**

```bash
git add client/src/AnalyticsPanel.tsx client/src/queries/analytics.ts
git commit -m "feat(analytics): refactor analytics panel with tabbed layout"
```

---

## Task 13: Landing page, docs, changelog, version bump

**Files:**
- Modify: `voltrus-landing/changelog.html`
- Modify: `voltrus-landing/docs.html`
- Modify: `voltrus-landing/index.html`
- Modify: `Cargo.toml` — version 0.28.0

- [ ] **Step 1: Add v0.28.0 changelog entry**

Add to `voltrus-landing/changelog.html` at the top:
```html
<div class="changelog-entry">
    <span class="badge badge-current">Current</span>
    <h3>v0.28.0 — Advanced Analytics Engine</h3>
    <ul>
        <li>Full SPC toolbox: X-bar & R, CUSUM, EWMA, P-chart, NP-chart</li>
        <li>Histogram with normal curve overlay and process capability indices (Cp, Cpk, Pp, Ppk)</li>
        <li>Run charts with Western Electric signal detection</li>
        <li>Custom calculation builder — define metrics from tag formulas</li>
        <li>Scheduled analytics email digests</li>
    </ul>
</div>
```

- [ ] **Step 2: Update docs.html with analytics section**

Add an API Reference / Analytics section documenting the new endpoints.

- [ ] **Step 3: Update index.html feature list**

Add "Advanced Analytics" to the features and update `softwareVersion` to `0.28.0`.

- [ ] **Step 4: Bump version in Cargo.toml**

Change `version = "0.27.0"` to `version = "0.28.0"`.

- [ ] **Step 5: Commit**

```bash
git add voltrus-landing/ Cargo.toml
git commit -m "feat: v0.28.0 — Advanced Analytics Engine"
```

- [ ] **Step 6: Tag**

```bash
git tag v0.28.0
```
