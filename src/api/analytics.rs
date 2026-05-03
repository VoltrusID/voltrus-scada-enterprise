use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::api::{AppError, AppState};
use crate::analytics;
use crate::models;

#[derive(Debug, Deserialize)]
pub struct AggregateQuery {
    pub device_id: String,
    pub metric: String,
    pub from: i64,
    pub to: i64,
    #[serde(default = "default_interval")]
    pub interval: i64,
}

fn default_interval() -> i64 { 3600 }

/// GET /api/v1/analytics/aggregate -- aggregate metrics over a time range
#[utoipa::path(
    get,
    path = "/api/v1/analytics/aggregate",
    params(
        ("device_id" = String, Query, description = "Device ID"),
        ("metric" = String, Query, description = "Metric name"),
        ("from" = i64, Query, description = "Start timestamp"),
        ("to" = i64, Query, description = "End timestamp"),
        ("interval" = Option<i64>, Query, description = "Bucket interval in seconds (default 3600)"),
    ),
    responses(
        (status = 200, body = Vec<(i64, crate::analytics::AggregatedMetric)>),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_aggregate(
    State(state): State<AppState>,
    Query(q): Query<AggregateQuery>,
) -> Result<Json<Vec<(i64, analytics::AggregatedMetric)>>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Analytics requires Starter tier or above"
        )));
    }

    let results = analytics::aggregate_metrics(
        &state.storage,
        &q.device_id,
        &q.metric,
        q.from,
        q.to,
        q.interval,
    )
    .await?;

    Ok(Json(results))
}

#[derive(Debug, Deserialize)]
pub struct CompareQuery {
    pub device_id: String,
    pub metric: String,
    pub period1_from: i64,
    pub period1_to: i64,
    pub period2_from: i64,
    pub period2_to: i64,
}

/// GET /api/v1/analytics/compare -- compare two time periods
#[utoipa::path(
    get,
    path = "/api/v1/analytics/compare",
    params(
        ("device_id" = String, Query, description = "Device ID"),
        ("metric" = String, Query, description = "Metric name"),
        ("period1_from" = i64, Query, description = "Period 1 start timestamp"),
        ("period1_to" = i64, Query, description = "Period 1 end timestamp"),
        ("period2_from" = i64, Query, description = "Period 2 start timestamp"),
        ("period2_to" = i64, Query, description = "Period 2 end timestamp"),
    ),
    responses(
        (status = 200, body = crate::analytics::ComparisonResult),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_compare(
    State(state): State<AppState>,
    Query(q): Query<CompareQuery>,
) -> Result<Json<analytics::ComparisonResult>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Analytics requires Starter tier or above"
        )));
    }

    let result = analytics::compare_periods(
        &state.storage,
        &q.device_id,
        &q.metric,
        (q.period1_from, q.period1_to),
        (q.period2_from, q.period2_to),
    )
    .await?;

    Ok(Json(result))
}

// ── SPC endpoint ──

#[derive(Debug, Deserialize)]
pub struct SpcQuery {
    pub device_id: String,
    pub metric: String,
    pub from: i64,
    pub to: i64,
    pub chart_type: String,
    pub subgroup_size: Option<u32>,
    pub target: Option<f64>,
    pub lambda: Option<f64>,
    pub k: Option<f64>,
    pub h: Option<f64>,
}

/// GET /api/v1/analytics/spc -- Statistical Process Control charts
#[utoipa::path(
    get,
    path = "/api/v1/analytics/spc",
    params(
        ("device_id" = String, Query, description = "Device ID"),
        ("metric" = String, Query, description = "Metric name"),
        ("from" = i64, Query, description = "Start timestamp"),
        ("to" = i64, Query, description = "End timestamp"),
        ("chart_type" = String, Query, description = "Chart type: xbar_r, cusum, ewma"),
        ("subgroup_size" = Option<u32>, Query, description = "Subgroup size for xbar_r (2-10, default 5)"),
        ("target" = Option<f64>, Query, description = "Target value for cusum/ewma"),
        ("lambda" = Option<f64>, Query, description = "Smoothing parameter for ewma (0 < lambda <= 1)"),
        ("k" = Option<f64>, Query, description = "Slack reference for cusum"),
        ("h" = Option<f64>, Query, description = "Decision interval for cusum"),
    ),
    responses(
        (status = 200, description = "SPC chart result"),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_spc(
    State(state): State<AppState>,
    Query(q): Query<SpcQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "SPC requires Professional tier"
        )));
    }

    let data = state
        .storage
        .get_telemetry_series(&q.device_id, &q.metric, q.from, q.to, 10000)
        .await?;

    let timestamps: Vec<i64> = data.iter().map(|d| d.timestamp).collect();
    let values: Vec<f64> = data.iter().map(|d| d.value).collect();

    let result = match q.chart_type.as_str() {
        "xbar_r" => {
            let subgroup_size = q.subgroup_size.unwrap_or(5) as usize;
            let (xbar, r) = analytics::spc::xbar_r(&values, subgroup_size, &timestamps)
                .map_err(|e| anyhow::anyhow!(e))?;
            serde_json::json!({
                "xbar": xbar,
                "r": r,
            })
        }
        "cusum" => {
            let target = q.target.unwrap_or(0.0);
            let k = q.k.unwrap_or(0.5);
            let h = q.h.unwrap_or(5.0);
            let result =
                analytics::spc::cusum(&values, target, k, h, &timestamps)
                    .map_err(|e| anyhow::anyhow!(e))?;
            serde_json::json!(result)
        }
        "ewma" => {
            let target = q.target.unwrap_or(0.0);
            let lambda = q.lambda.unwrap_or(0.2);
            let result =
                analytics::spc::ewma(&values, target, lambda, &timestamps)
                    .map_err(|e| anyhow::anyhow!(e))?;
            serde_json::json!(result)
        }
        other => {
            return Err(AppError(anyhow::anyhow!(
                "Unknown chart_type '{}'. Supported: xbar_r, cusum, ewma",
                other
            )));
        }
    };

    Ok(Json(result))
}

// ── Histogram endpoint ──

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

/// GET /api/v1/analytics/histogram -- Histogram with normal curve and capability indices
#[utoipa::path(
    get,
    path = "/api/v1/analytics/histogram",
    params(
        ("device_id" = String, Query, description = "Device ID"),
        ("metric" = String, Query, description = "Metric name"),
        ("from" = i64, Query, description = "Start timestamp"),
        ("to" = i64, Query, description = "End timestamp"),
        ("bins" = Option<u32>, Query, description = "Number of bins (default 20)"),
        ("lsl" = Option<f64>, Query, description = "Lower Specification Limit"),
        ("usl" = Option<f64>, Query, description = "Upper Specification Limit"),
    ),
    responses(
        (status = 200, body = models::HistogramResult),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_histogram(
    State(state): State<AppState>,
    Query(q): Query<HistogramQuery>,
) -> Result<Json<models::HistogramResult>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Histogram requires Starter tier or above"
        )));
    }

    let data = state
        .storage
        .get_telemetry_series(&q.device_id, &q.metric, q.from, q.to, -1)
        .await?;

    let values: Vec<f64> = data.iter().map(|d| d.value).collect();
    let bins = q.bins.unwrap_or(20);

    let result = analytics::histogram::compute_histogram(&values, bins, q.lsl, q.usl)
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(Json(result))
}

// ── Run chart endpoint ──

#[derive(Debug, Deserialize)]
pub struct RunChartQuery {
    pub device_id: String,
    pub metric: String,
    pub from: i64,
    pub to: i64,
}

/// GET /api/v1/analytics/run-chart -- Run chart with signal detection
#[utoipa::path(
    get,
    path = "/api/v1/analytics/run-chart",
    params(
        ("device_id" = String, Query, description = "Device ID"),
        ("metric" = String, Query, description = "Metric name"),
        ("from" = i64, Query, description = "Start timestamp"),
        ("to" = i64, Query, description = "End timestamp"),
    ),
    responses(
        (status = 200, body = models::RunChartResult),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_run_chart(
    State(state): State<AppState>,
    Query(q): Query<RunChartQuery>,
) -> Result<Json<models::RunChartResult>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Run chart requires Starter tier or above"
        )));
    }

    let data = state
        .storage
        .get_telemetry_series(&q.device_id, &q.metric, q.from, q.to, -1)
        .await?;

    let timestamps: Vec<i64> = data.iter().map(|d| d.timestamp).collect();
    let values: Vec<f64> = data.iter().map(|d| d.value).collect();

    let result = analytics::runchart::compute_run_chart(&values, &timestamps)
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(Json(result))
}

// ── Custom metrics CRUD ──

/// GET /api/v1/analytics/custom-metrics -- List all custom metrics
#[utoipa::path(
    get,
    path = "/api/v1/analytics/custom-metrics",
    responses(
        (status = 200, body = Vec<models::CustomMetric>),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_custom_metrics(
    State(state): State<AppState>,
) -> Result<Json<Vec<models::CustomMetric>>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Custom metrics requires Starter tier or above"
        )));
    }

    let metrics = state.storage.list_custom_metrics().await?;
    Ok(Json(metrics))
}

/// POST /api/v1/analytics/custom-metrics -- Create a new custom metric
#[utoipa::path(
    post,
    path = "/api/v1/analytics/custom-metrics",
    request_body = models::CreateCustomMetricRequest,
    responses(
        (status = 200, body = models::CustomMetric),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn create_custom_metric(
    State(state): State<AppState>,
    Json(req): Json<models::CreateCustomMetricRequest>,
) -> Result<Json<models::CustomMetric>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Custom metrics requires Professional tier"
        )));
    }

    // Extract tag references from formula
    let tag_refs = analytics::custom::extract_tag_refs(&req.formula);

    // Validate formula
    let validation_error = match analytics::custom::validate_formula(&req.formula) {
        Ok(()) => None,
        Err(e) => Some(e),
    };

    let tag_refs_json = serde_json::to_string(&tag_refs)
        .map_err(|e| anyhow::anyhow!("Failed to serialize tag_refs: {}", e))?;

    let id = state
        .storage
        .create_custom_metric(
            &req.name,
            &req.description,
            &req.formula,
            req.unit.as_deref(),
            &tag_refs_json,
            validation_error.as_deref(),
        )
        .await?;

    // Return the created metric
    let metrics = state.storage.list_custom_metrics().await?;
    let created = metrics
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created custom metric"))?;

    Ok(Json(created))
}

/// PUT /api/v1/analytics/custom-metrics/{id} -- Update a custom metric
#[utoipa::path(
    put,
    path = "/api/v1/analytics/custom-metrics/{id}",
    request_body = models::UpdateCustomMetricRequest,
    responses(
        (status = 200, body = models::CustomMetric),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_custom_metric(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<models::UpdateCustomMetricRequest>,
) -> Result<Json<models::CustomMetric>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Custom metrics requires Professional tier"
        )));
    }

    // If formula is being updated, re-extract tag refs and re-validate
    let (tag_refs_str, validation_error) = if let Some(ref formula) = req.formula {
        let tag_refs = analytics::custom::extract_tag_refs(formula);
        let tag_refs_json = serde_json::to_string(&tag_refs)
            .map_err(|e| anyhow::anyhow!("Failed to serialize tag_refs: {}", e))?;
        let val_err = match analytics::custom::validate_formula(formula) {
            Ok(()) => None,
            Err(e) => Some(e),
        };
        (Some(tag_refs_json), val_err)
    } else {
        (None, None)
    };

    state
        .storage
        .update_custom_metric(
            id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.formula.as_deref(),
            req.unit.as_deref(),
            tag_refs_str.as_deref(),
            validation_error.as_deref(),
        )
        .await?;

    // Return the updated metric
    let metrics = state.storage.list_custom_metrics().await?;
    let updated = metrics
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| anyhow::anyhow!("Custom metric not found"))?;

    Ok(Json(updated))
}

/// DELETE /api/v1/analytics/custom-metrics/{id} -- Delete a custom metric
#[utoipa::path(
    delete,
    path = "/api/v1/analytics/custom-metrics/{id}",
    responses(
        (status = 200, description = "Metric deleted"),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn delete_custom_metric(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Custom metrics requires Professional tier"
        )));
    }

    state.storage.delete_custom_metric(id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

/// GET /api/v1/analytics/custom-metrics/evaluate-all -- Evaluate all custom metrics
#[utoipa::path(
    get,
    path = "/api/v1/analytics/custom-metrics/evaluate-all",
    responses(
        (status = 200, body = Vec<models::CustomMetricValue>),
        (status = 403, description = "License tier too low")
    ),
    tag = "Analytics",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn evaluate_all_custom_metrics(
    State(state): State<AppState>,
) -> Result<Json<Vec<models::CustomMetricValue>>, AppError> {
    if !state.license_tier.analytics() {
        return Err(AppError(anyhow::anyhow!(
            "Custom metrics requires Starter tier or above"
        )));
    }

    let values = analytics::custom::evaluate_all_metrics(&state.storage, &state.cache).await?;
    Ok(Json(values))
}
