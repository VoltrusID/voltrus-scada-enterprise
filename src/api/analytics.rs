use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::api::{AppError, AppState};
use crate::analytics;

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
