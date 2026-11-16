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

use voltrus::api::AppState;
use voltrus::cache::DeviceCache;
use voltrus::models::{CreateHubSiteRequest, HubSite, UpdateHubSiteRequest};
use voltrus::storage::Storage;

/// Tracks running hub site connections.
/// Key is the site ID, value is the JoinHandle for the spawned task.
#[derive(Clone, Default)]
pub struct HubManager {
    handles: Arc<RwLock<HashMap<i64, tokio::task::JoinHandle<()>>>>,
    links: crate::hub::connector::SiteLinks,
}

impl HubManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// The live state of a site's link (connected, last data, last error).
    pub fn link(&self, site_id: i64) -> crate::hub::connector::SiteLink {
        self.links
            .get(&site_id)
            .map(|l| l.clone())
            .unwrap_or_default()
    }

    pub async fn start_site(
        &self,
        site: voltrus::models::HubSite,
        cache: DeviceCache,
        storage: Storage,
    ) {
        let site_id = site.id;
        // Stop existing connection if any
        self.stop_site(site_id).await;

        let links = self.links.clone();
        let handle = tokio::spawn(async move {
            crate::hub::connector::connect_site(site, cache, storage, links).await;
        });
        self.handles.write().await.insert(site_id, handle);
    }

    pub async fn stop_site(&self, site_id: i64) {
        if let Some(handle) = self.handles.write().await.remove(&site_id) {
            handle.abort();
        }
        if let Some(mut link) = self.links.get_mut(&site_id) {
            link.connected = false;
        }
        if let Some(rt) = crate::runtime::try_rt()
            && let Ok(Some(site)) = rt
                .storage
                .list_hub_sites()
                .await
                .map(|sites| sites.into_iter().find(|s| s.id == site_id))
        {
            rt.cache
                .set_status(
                    &crate::hub::connector::site_device_id(&site.name),
                    voltrus::models::DeviceStatus::Offline,
                )
                .await;
        }
    }

    pub async fn is_running(&self, site_id: i64) -> bool {
        self.handles.read().await.contains_key(&site_id)
    }

    /// Stop every site link (the node stopped being the active one of a pair).
    pub async fn stop_all(&self) {
        let mut handles = self.handles.write().await;
        for (_, handle) in handles.drain() {
            handle.abort();
        }
        for mut link in self.links.iter_mut() {
            link.connected = false;
        }
    }
}

fn multi_site_required(state: &AppState) -> Option<(HttpStatus, Json<serde_json::Value>)> {
    if !state.tier().multi_site() {
        return Some((
            HttpStatus::FORBIDDEN,
            Json(json!({
                "error": "Multi-site hub requires an Integrator or Enterprise licence. See https://voltrus.id/voltrus-scada"
            })),
        ));
    }
    None
}

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
                url: url.to_string(),
                api_key: body.api_key,
                enabled,
                last_seen: None,
                created_at: voltrus::timeutil::now_secs(),
            };
            // An enabled site connects at once, as it would after a restart.
            if enabled && voltrus::node_role::is_active() {
                crate::rt()
                    .hub
                    .start_site(site.clone(), state.cache.clone(), state.storage.clone())
                    .await;
            }
            (HttpStatus::CREATED, Json(site)).into_response()
        }
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{}", e)})),
        )
            .into_response(),
    }
}

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
        Ok(()) => {
            // A running link picks up the new URL or key; a disabled site stops.
            let hub = &crate::rt().hub;
            if hub.is_running(id).await {
                hub.stop_site(id).await;
                let site = state
                    .storage
                    .list_hub_sites()
                    .await
                    .ok()
                    .and_then(|sites| sites.into_iter().find(|s| s.id == id));
                if let Some(site) = site.filter(|s| s.enabled) {
                    hub.start_site(site, state.cache.clone(), state.storage.clone())
                        .await;
                }
            }
            HttpStatus::NO_CONTENT.into_response()
        }
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{}", e)})),
        )
            .into_response(),
    }
}

pub async fn delete_site(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }

    // Stop the connection first
    {
        let hub_manager = &crate::rt().hub;
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

    {
        let hub_manager = &crate::rt().hub;
        hub_manager
            .start_site(site, state.cache.clone(), state.storage.clone())
            .await;
    }

    Json(json!({"ok": true})).into_response()
}

pub async fn stop_site_connection(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }

    {
        let hub_manager = &crate::rt().hub;
        hub_manager.stop_site(id).await;
    }

    Json(json!({"ok": true})).into_response()
}

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
        let hub = &crate::rt().hub;
        let running = hub.is_running(site.id).await;
        let link = hub.link(site.id);
        status_entries.push(json!({
            "id": site.id,
            "name": site.name,
            "url": site.url,
            "enabled": site.enabled,
            "running": running,
            "connected": running && link.connected,
            "last_message_at": link.last_message_at,
            "tags": link.tags,
            "last_error": link.last_error,
            "device_id": format!("{}:default", site.name),
            "last_seen": site.last_seen,
        }));
    }

    Json(json!({
        "sites": status_entries,
        "total": sites.len(),
    }))
    .into_response()
}

/// A remote tag as `GET /api/v1/data-sources` describes it.
#[derive(Debug, serde::Deserialize)]
pub struct RemoteTag {
    pub key: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub min_val: f64,
    #[serde(default = "hundred")]
    pub max_val: f64,
    #[serde(default)]
    pub warn_val: f64,
    #[serde(default)]
    pub crit_val: f64,
    #[serde(default)]
    pub base_val: f64,
    #[serde(default = "one")]
    pub decimals: i64,
    #[serde(default)]
    pub chart_type: Option<String>,
}

fn hundred() -> f64 {
    100.0
}
fn one() -> i64 {
    1
}

/// The hub tag key of a remote tag: `<site>:default.<remote key>`.
pub fn hub_tag_key(site_name: &str, remote_key: &str) -> String {
    format!("{site_name}:default.{remote_key}")
}

/// POST /api/v1/hub/sites/{id}/import-tags
///
/// Reads the site's tag list with its API key and creates one hub tag per
/// remote tag (same name, unit, range and alarm limits), reading device
/// `<site>:default`. Tags that exist already are left alone, so it can run
/// again after the site gains tags.
pub async fn import_site_tags(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Some((status, err)) = multi_site_required(&state) {
        return (status, err).into_response();
    }
    let site = match state.storage.list_hub_sites().await {
        Ok(sites) => sites.into_iter().find(|s| s.id == id),
        Err(e) => {
            return (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    let Some(site) = site else {
        return (
            HttpStatus::NOT_FOUND,
            Json(json!({"error": "hub site not found"})),
        )
            .into_response();
    };
    let url = format!("{}/api/v1/data-sources", site.url.trim_end_matches('/'));
    let remote: Vec<RemoteTag> = match async {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()?
            .get(&url)
            .header("X-API-Key", &site.api_key)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    .await
    {
        Ok(tags) => tags,
        Err(e) => {
            return (
                HttpStatus::BAD_GATEWAY,
                Json(json!({"error": format!("could not read the tags of '{}': {e}", site.name)})),
            )
                .into_response();
        }
    };

    let existing: std::collections::HashSet<String> =
        match state.storage.list_data_sources(None).await {
            Ok(ds) => ds.into_iter().map(|d| d.key).collect(),
            Err(e) => {
                return (
                    HttpStatus::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        };
    let device_id = format!("{}:default", site.name);
    let mut sort = state
        .storage
        .max_data_source_sort_order()
        .await
        .unwrap_or(0);
    let (mut created, mut skipped) = (0usize, 0usize);
    for tag in &remote {
        let key = hub_tag_key(&site.name, &tag.key);
        if existing.contains(&key) {
            skipped += 1;
            continue;
        }
        sort += 1;
        let name = format!(
            "{} {}",
            site.name,
            if tag.name.is_empty() {
                &tag.key
            } else {
                &tag.name
            }
        );
        if let Err(e) = state
            .storage
            .create_data_source(
                &key,
                &name,
                &tag.unit,
                tag.color.as_deref().unwrap_or("#94a3b8"),
                tag.min_val,
                tag.max_val,
                tag.warn_val,
                tag.crit_val,
                tag.base_val,
                sort,
                tag.decimals,
                "modbus",
                "{}",
                &device_id,
                None,
                tag.chart_type.as_deref().unwrap_or("sparkline"),
            )
            .await
        {
            return (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("creating {key}: {e}"), "created": created})),
            )
                .into_response();
        }
        created += 1;
    }
    Json(json!({
        "ok": true,
        "device_id": device_id,
        "remote_tags": remote.len(),
        "created": created,
        "skipped": skipped,
    }))
    .into_response()
}

#[cfg(test)]
mod import_tests {
    use super::*;

    #[test]
    fn remote_tags_parse_with_defaults_and_map_to_hub_keys() {
        let tags: Vec<RemoteTag> = serde_json::from_str(
            r#"[{"key":"voltage_l1n","name":"Voltage","unit":"V","warn_val":250,"crit_val":260,"extra":1},{"key":"flow"}]"#,
        )
        .unwrap();
        assert_eq!(tags[0].unit, "V");
        assert_eq!(tags[0].crit_val, 260.0);
        assert_eq!(tags[1].max_val, 100.0);
        assert_eq!(tags[1].decimals, 1);
        assert_eq!(
            hub_tag_key("Plant A", &tags[0].key),
            "Plant A:default.voltage_l1n"
        );
    }
}
