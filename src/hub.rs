use std::collections::HashMap;

use crate::cache::DeviceCache;
use crate::models::HubSite;
use crate::storage::Storage;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite;

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

/// Connect to a remote Voltrus instance via WebSocket and merge data into the local cache.
/// Device IDs are prefixed with `{site_name}:` to avoid collisions.
pub async fn connect_site(site: HubSite, cache: DeviceCache, storage: Storage) {
    let site_id = site.id;
    let site_name = site.name.clone();
    let mut backoff = Backoff::new();

    loop {
        match connect_and_stream(&site, &cache, &storage).await {
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

async fn connect_and_stream(
    site: &HubSite,
    cache: &DeviceCache,
    storage: &Storage,
) -> anyhow::Result<()> {
    let ws_url = format!(
        "{}/api/v1/ws",
        site.url
            .trim_end_matches('/')
            .replace("https://", "wss://")
            .replace("http://", "ws://")
    );

    let mut last_seen_update = std::time::Instant::now();

    let request = tungstenite::http::Request::builder()
        .uri(&ws_url)
        .header("X-API-Key", &site.api_key)
        .body(())?;

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(request).await?;

    tracing::info!("Hub site '{}' connected to {}", site.name, ws_url);

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(tungstenite::Message::Binary(data)) => {
                let parsed: serde_json::Value = match rmp_serde::from_slice(&data) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

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
                                let prefixed_id = format!("{}:default", site.name);
                                cache.update(&prefixed_id, metrics, false).await;
                            }
                        }
                    }
                    "TagUpdate" => {
                        let key = parsed.get("tag_id").and_then(|v| v.as_str()).unwrap_or("");
                        let value = parsed.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        if !key.is_empty() {
                            let prefixed_id = format!("{}:default", site.name);
                            let metrics = HashMap::from([(key.to_string(), value)]);
                            cache.update(&prefixed_id, metrics, false).await;
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

        // Update last_seen every 60 seconds
        if last_seen_update.elapsed() >= std::time::Duration::from_secs(60) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
