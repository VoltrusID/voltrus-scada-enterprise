use std::collections::HashMap;

use futures_util::StreamExt;
use tokio_tungstenite::tungstenite;
use voltrus::cache::DeviceCache;
use voltrus::models::HubSite;
use voltrus::storage::Storage;

struct Backoff {
    attempt: u32,
    max_delay_ms: u64,
}

impl Backoff {
    fn new() -> Self {
        Self {
            attempt: 0,
            max_delay_ms: 60_000,
        }
    }

    fn reset(&mut self) {
        self.attempt = 0;
    }

    fn next(&mut self) -> u64 {
        let attempt = self.attempt.min(30);
        let delay = 1000 * 2u64.pow(attempt);
        self.attempt = self.attempt.saturating_add(1);
        delay.min(self.max_delay_ms)
    }
}

/// What the hub knows about one site's link right now (not persisted).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SiteLink {
    /// The WebSocket to the site is open and authenticated.
    pub connected: bool,
    /// Unix seconds of the last tag update received.
    pub last_message_at: Option<i64>,
    /// Distinct tags received from the site since the hub started.
    pub tags: usize,
    /// Why the last attempt failed, until the next successful connect.
    pub last_error: Option<String>,
}

pub type SiteLinks = std::sync::Arc<dashmap::DashMap<i64, SiteLink>>;

/// Connect to a remote Voltrus instance via WebSocket and merge data into the local cache.
/// Every tag of the site goes to the hub device `{site_name}:default`, under its
/// original tag key.
pub async fn connect_site(site: HubSite, cache: DeviceCache, storage: Storage, links: SiteLinks) {
    let site_id = site.id;
    let site_name = site.name.clone();
    let mut backoff = Backoff::new();

    loop {
        let result = connect_and_stream(&site, &cache, &storage, &links).await;
        {
            let mut link = links.entry(site_id).or_default();
            link.connected = false;
            if let Err(e) = &result {
                link.last_error = Some(e.to_string());
            }
        }
        // The last values stay readable but are marked bad, so the hub never
        // shows a lost site's values as current.
        cache
            .set_status(
                &site_device_id(&site_name),
                voltrus::models::DeviceStatus::Offline,
            )
            .await;
        match result {
            Ok(()) => backoff.reset(),
            Err(e) => {
                tracing::warn!("Hub site '{}' (id={}) error: {}", site_name, site_id, e);
                let delay = backoff.next();
                tracing::info!("Hub site '{}' retrying in {}s", site_name, delay / 1000);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
        }
    }
}

/// The hub device that holds every tag of a site.
pub fn site_device_id(site_name: &str) -> String {
    format!("{site_name}:default")
}

/// The WebSocket handshake request for a site: its `/api/v1/ws` with the API key.
pub fn site_request(site: &HubSite) -> anyhow::Result<tungstenite::handshake::client::Request> {
    let ws_url = format!(
        "{}/api/v1/ws",
        site.url
            .trim_end_matches('/')
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1)
    );
    // `into_client_request` adds the WebSocket handshake headers (key, version,
    // upgrade); a bare http::Request has none and every site refuses it.
    use tungstenite::client::IntoClientRequest;
    let mut request = ws_url.as_str().into_client_request()?;
    request.headers_mut().insert(
        "X-API-Key",
        tungstenite::http::HeaderValue::from_str(&site.api_key)
            .map_err(|_| anyhow::anyhow!("the API key holds characters a header cannot carry"))?,
    );
    Ok(request)
}

async fn connect_and_stream(
    site: &HubSite,
    cache: &DeviceCache,
    storage: &Storage,
    links: &SiteLinks,
) -> anyhow::Result<()> {
    let request = site_request(site)?;
    let ws_url = request.uri().to_string();
    let mut last_seen_update = std::time::Instant::now();

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(request).await?;

    tracing::info!("Hub site '{}' connected to {}", site.name, ws_url);
    let mut seen_tags = std::collections::HashSet::new();

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(tungstenite::Message::Binary(data)) => {
                let parsed: serde_json::Value = match rmp_serde::from_slice(&data) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if msg_type == "AuthError" {
                    let why = parsed
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("rejected");
                    anyhow::bail!("site refused the API key: {why}");
                }
                if msg_type == "AuthOk" {
                    links.entry(site.id).or_default().connected = true;
                    if let Some(mut link) = links.get_mut(&site.id) {
                        link.last_error = None;
                    }
                    if let Err(e) = storage.update_hub_site_last_seen(site.id).await {
                        tracing::warn!("hub site '{}': last_seen: {e}", site.name);
                    }
                    last_seen_update = std::time::Instant::now();
                }

                match msg_type {
                    "TagUpdateBatch" => {
                        if let Some(updates) = parsed.get("updates").and_then(|u| u.as_array()) {
                            let mut device_metrics: HashMap<String, HashMap<String, f64>> =
                                HashMap::new();

                            for entry in updates {
                                let key =
                                    entry.get("tag_id").and_then(|v| v.as_str()).unwrap_or("");
                                let value =
                                    entry.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                if !key.is_empty() {
                                    device_metrics
                                        .entry(String::new()) // all go to default device
                                        .or_default()
                                        .insert(key.to_string(), value);
                                }
                            }

                            for (_device_id, metrics) in device_metrics {
                                seen_tags.extend(metrics.keys().cloned());
                                let prefixed_id = site_device_id(&site.name);
                                cache.update(&prefixed_id, metrics, false).await;
                            }
                            note_message(links, site.id, seen_tags.len());
                        }
                    }
                    "TagUpdate" => {
                        let key = parsed.get("tag_id").and_then(|v| v.as_str()).unwrap_or("");
                        let value = parsed.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        if !key.is_empty() {
                            let prefixed_id = site_device_id(&site.name);
                            seen_tags.insert(key.to_string());
                            let metrics = HashMap::from([(key.to_string(), value)]);
                            cache.update(&prefixed_id, metrics, false).await;
                            note_message(links, site.id, seen_tags.len());
                        }
                    }
                    _ => {}
                }
            }
            Ok(tungstenite::Message::Close(_)) => {
                anyhow::bail!("WebSocket closed for site '{}'", site.name);
            }
            Ok(_) => {}
            Err(e) => {
                anyhow::bail!("WebSocket error for site '{}': {}", site.name, e);
            }
        }

        // Update last_seen every 15 seconds while data flows
        if last_seen_update.elapsed() >= std::time::Duration::from_secs(15) {
            if let Err(e) = storage.update_hub_site_last_seen(site.id).await {
                tracing::warn!(
                    "Failed to update last_seen for hub site '{}': {}",
                    site.name,
                    e
                );
            }
            last_seen_update = std::time::Instant::now();
        }
    }

    anyhow::bail!("WebSocket stream ended for site '{}'", site.name);
}

fn note_message(links: &SiteLinks, site_id: i64, tags: usize) {
    let mut link = links.entry(site_id).or_default();
    // A site that sends data is connected even if its AuthOk was missed.
    link.connected = true;
    link.last_message_at = Some(voltrus::timeutil::now_secs());
    link.tags = tags;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_request_is_a_websocket_handshake_with_the_key() {
        let site = HubSite {
            id: 1,
            name: "A".into(),
            url: "https://plant-a.example.com/".into(),
            api_key: "k123".into(),
            enabled: true,
            last_seen: None,
            created_at: 0,
        };
        let req = site_request(&site).unwrap();
        assert_eq!(req.uri().to_string(), "wss://plant-a.example.com/api/v1/ws");
        for h in [
            "sec-websocket-key",
            "sec-websocket-version",
            "upgrade",
            "connection",
            "host",
        ] {
            assert!(req.headers().contains_key(h), "missing {h}");
        }
        assert_eq!(req.headers()["x-api-key"], "k123");
        let bad = HubSite {
            api_key: "bad\nkey".into(),
            ..site
        };
        assert!(site_request(&bad).is_err());
    }

    #[test]
    fn test_backoff_increases() {
        let mut b = Backoff::new();
        let d1 = b.next();
        let d2 = b.next();
        assert!(d2 > d1);
    }

    #[test]
    fn test_backoff_resets() {
        let mut b = Backoff::new();
        let d1 = b.next();
        let d2 = b.next();
        assert!(d2 > d1);
        b.reset();
        let d3 = b.next();
        assert_eq!(d1, d3);
    }

    #[test]
    fn test_backoff_max_delay() {
        let mut b = Backoff::new();
        for _ in 0..50 {
            b.next();
        }
        let delay = b.next();
        assert!(delay <= b.max_delay_ms);
    }
}
