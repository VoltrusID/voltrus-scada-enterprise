use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode as HttpStatus,
    response::IntoResponse,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::AppState;
use crate::cache::DeviceCache;
use crate::models::{CreateHubSiteRequest, HubSite, UpdateHubSiteRequest};
use crate::storage::Storage;

/// Tracks running hub site connections.
/// Key is the site ID, value is the JoinHandle for the spawned task.
#[derive(Clone, Default)]
pub struct HubManager {
    handles: Arc<RwLock<HashMap<i64, tokio::task::JoinHandle<()>>>>,
}

impl HubManager {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_site(
        &self,
        site: crate::models::HubSite,
        cache: DeviceCache,
        storage: Storage,
    ) {
        let site_id = site.id;
        // Stop existing connection if any
        self.stop_site(site_id).await;

        let handle = tokio::spawn(async move {
            crate::hub::connect_site(site, cache, storage).await;
        });
        self.handles.write().await.insert(site_id, handle);
    }

    pub async fn stop_site(&self, site_id: i64) {
        if let Some(handle) = self.handles.write().await.remove(&site_id) {
            handle.abort();
        }
    }

    pub async fn is_running(&self, site_id: i64) -> bool {
        self.handles.read().await.contains_key(&site_id)
    }

    #[allow(dead_code)]
    // Utility for graceful shutdown; not yet wired to the shutdown handler.
    pub async fn stop_all(&self) {
        let mut handles = self.handles.write().await;
        for (_, handle) in handles.drain() {
            handle.abort();
        }
    }
}

fn multi_site_required(state: &AppState) -> Option<(HttpStatus, Json<serde_json::Value>)> {
    if !state.license_tier.multi_site() {
        return Some((
            HttpStatus::FORBIDDEN,
            Json(json!({
                "error": "Multi-site hub requires Professional tier or higher. Upgrade at https://voltrus.id"
            })),
        ));
    }
    None
}

#[utoipa::path(
    get,
    path = "/api/v1/hub/sites",
    responses(
        (status = 200, body = Vec<crate::models::HubSite>),
        (status = 403),
    ),
    tag = "Hub",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_sites(State(state): State<AppState>) -> impl IntoResponse {
    if let Some((status, body)) = multi_site_required(&state) {
        return (status, body).into_response();
    }

    match state.storage.list_hub_sites().await {
        Ok(sites) => Json(sites).into_response(),
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/hub/sites",
    request_body = CreateHubSiteRequest,
    responses(
        (status = 201, body = crate::models::HubSite),
        (status = 400),
        (status = 403),
    ),
    tag = "Hub",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn create_site(
    State(state): State<AppState>,
    Json(body): Json<CreateHubSiteRequest>,
) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }

    // Validate
    if body.name.trim().is_empty() {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "name must not be empty"})),
        )
            .into_response();
    }

    let url = body.url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "url must start with http:// or https://"})),
        )
            .into_response();
    }

    let enabled = body.enabled.unwrap_or(true);

    match state
        .storage
        .create_hub_site(&body.name, url, &body.api_key, enabled)
        .await
    {
        Ok(id) => {
            let site = HubSite {
                id,
                name: body.name,
                url: body.url,
                api_key: body.api_key,
                enabled,
                last_seen: None,
                created_at: crate::timeutil::now_secs(),
            };
            (HttpStatus::CREATED, Json(site)).into_response()
        }
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/hub/sites/{id}",
    params(
        ("id" = i64, Path, description = "Hub site ID")
    ),
    request_body = UpdateHubSiteRequest,
    responses(
        (status = 204),
        (status = 400),
        (status = 403),
    ),
    tag = "Hub",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_site(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateHubSiteRequest>,
) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }

    // Validate URL if provided
    if let Some(ref url) = body.url {
        let url = url.trim();
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return (
                HttpStatus::BAD_REQUEST,
                Json(json!({"error": "url must start with http:// or https://"})),
            )
                .into_response();
        }
    }

    // Validate name if provided
    if let Some(ref name) = body.name
        && name.trim().is_empty()
    {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "name must not be empty"})),
        )
            .into_response();
    }

    match state
        .storage
        .update_hub_site(
            id,
            body.name.as_deref(),
            body.url.as_deref().map(|u| u.trim()),
            body.api_key.as_deref(),
            body.enabled,
        )
        .await
    {
        Ok(()) => HttpStatus::NO_CONTENT.into_response(),
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/hub/sites/{id}",
    params(
        ("id" = i64, Path, description = "Hub site ID")
    ),
    responses(
        (status = 204),
        (status = 403),
    ),
    tag = "Hub",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn delete_site(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }

    // Stop the connection first
    if let Some(hub_manager) = &state.hub_manager {
        hub_manager.stop_site(id).await;
    }

    match state.storage.delete_hub_site(id).await {
        Ok(()) => HttpStatus::NO_CONTENT.into_response(),
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/hub/sites/{id}/start",
    params(
        ("id" = i64, Path, description = "Hub site ID")
    ),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 400),
        (status = 403),
        (status = 404),
    ),
    tag = "Hub",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn start_site_connection(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }

    let sites = match state.storage.list_hub_sites().await {
        Ok(s) => s,
        Err(e) => {
            return (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{}", e)})),
            )
                .into_response();
        }
    };

    let site = match sites.into_iter().find(|s| s.id == id) {
        Some(s) => s,
        None => {
            return (
                HttpStatus::NOT_FOUND,
                Json(json!({"error": "hub site not found"})),
            )
                .into_response();
        }
    };

    if !site.enabled {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "hub site is disabled"})),
        )
            .into_response();
    }

    if let Some(hub_manager) = &state.hub_manager {
        hub_manager
            .start_site(site, state.cache.clone(), state.storage.clone())
            .await;
    }

    Json(json!({"ok": true})).into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/hub/sites/{id}/stop",
    params(
        ("id" = i64, Path, description = "Hub site ID")
    ),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
    ),
    tag = "Hub",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn stop_site_connection(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }

    if let Some(hub_manager) = &state.hub_manager {
        hub_manager.stop_site(id).await;
    }

    Json(json!({"ok": true})).into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/hub/status",
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
    ),
    tag = "Hub",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_hub_status(State(state): State<AppState>) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }

    let sites = match state.storage.list_hub_sites().await {
        Ok(s) => s,
        Err(e) => {
            return (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{}", e)})),
            )
                .into_response();
        }
    };

    let mut status_entries = Vec::new();
    for site in &sites {
        let running = if let Some(hm) = &state.hub_manager {
            hm.is_running(site.id).await
        } else {
            false
        };
        status_entries.push(json!({
            "id": site.id,
            "name": site.name,
            "url": site.url,
            "enabled": site.enabled,
            "running": running,
            "last_seen": site.last_seen,
        }));
    }

    Json(json!({
        "sites": status_entries,
        "total": sites.len(),
    }))
    .into_response()
}
