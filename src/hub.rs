use std::collections::HashMap;

use crate::cache::DeviceCache;
use crate::models::HubSite;
use crate::storage::Storage;

struct Backoff {
    attempt: u32,
    max_delay_ms: u64,
}

impl Backoff {
    fn new() -> Self {
        Self { attempt: 0, max_delay_ms: 60_000 }
    }

    fn reset(&mut self) { self.attempt = 0; }

    fn next(&mut self) -> u64 {
        let attempt = self.attempt.min(30);
        let delay = 1000 * 2u64.pow(attempt);
        self.attempt = self.attempt.saturating_add(1);
        delay.min(self.max_delay_ms)
    }
}

/// Connect to a remote Voltrus instance SSE stream and merge data into the local cache.
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
    let stream_url = format!("{}/api/v1/stream", site.url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut last_seen_update = std::time::Instant::now();

    let response = client
        .get(&stream_url)
        .header("X-API-Key", &site.api_key)
        .header("Accept", "text/event-stream")
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("SSE connection returned HTTP {}", response.status());
    }

    tracing::info!("Hub site '{}' connected to {}", site.name, stream_url);

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    use tokio_stream::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // SSE events are separated by double newlines
        while let Some(pos) = buffer.find("\n\n") {
            let event_text = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            // Parse data lines from the event
            for line in event_text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    process_sse_data(data, &site.name, cache).await;
                }
            }

            // Update last_seen every 60 seconds
            if last_seen_update.elapsed() >= std::time::Duration::from_secs(60) {
                if let Err(e) = storage.update_hub_site_last_seen(site.id).await {
                    tracing::warn!("Failed to update last_seen for hub site '{}': {}", site.name, e);
                }
                last_seen_update = std::time::Instant::now();
            }
        }
    }

    // Stream ended
    anyhow::bail!("SSE stream ended for site '{}'", site.name);
}

/// Parse an SSE data payload and merge device metrics into the local cache.
/// Expected format: `{"sensors": [{"id": N, "key": "...", "value": V, ...}, ...]}` or
/// raw device format: `{"devices":{"device_id":{"metrics":{"key":value}}}}`
async fn process_sse_data(data: &str, site_name: &str, cache: &DeviceCache) {
    // Try parsing as the SSE composer output format first: {"sensors": [...]}
    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(sensors) = payload.get("sensors").and_then(|v| v.as_array()) {
            // Group sensor values by device_id (extracted from source_config)
            let mut device_metrics: HashMap<String, HashMap<String, f64>> = HashMap::new();

            for sensor in sensors {
                let key = sensor.get("key").and_then(|v| v.as_str()).unwrap_or("");
                let value = sensor.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let source_type = sensor.get("source_type").and_then(|v| v.as_str()).unwrap_or("");

                // Skip expression-based data sources (computed locally)
                if source_type == "expression" {
                    continue;
                }

                // Extract device_id from source_config or key prefix
                let device_id = extract_device_id(sensor, key);
                if !device_id.is_empty() {
                    device_metrics
                        .entry(device_id)
                        .or_default()
                        .insert(key.to_string(), value);
                }
            }

            for (device_id, metrics) in device_metrics {
                let prefixed_id = format!("{}:{}", site_name, device_id);
                cache.update(&prefixed_id, metrics, false).await;
            }
            return;
        }

        // Try raw devices format: {"devices": {"device_id": {"metrics": {"key": value}}}}
        if let Some(devices) = payload.get("devices").and_then(|v| v.as_object()) {
            for (device_id, device_data) in devices {
                if let Some(metrics) = device_data.get("metrics").and_then(|v| v.as_object()) {
                    let mut metrics_map = HashMap::new();
                    for (k, v) in metrics {
                        if let Some(val) = v.as_f64() {
                            metrics_map.insert(k.clone(), val);
                        }
                    }
                    if !metrics_map.is_empty() {
                        let prefixed_id = format!("{}:{}", site_name, device_id);
                        cache.update(&prefixed_id, metrics_map, false).await;
                    }
                }
            }
        }
    }
}

/// Extract device_id from a sensor object in the SSE payload.
fn extract_device_id(sensor: &serde_json::Value, key: &str) -> String {
    // Try source_config.device_id first (JSON object)
    if let Some(source_config) = sensor.get("source_config") {
        if let Some(device_id) = source_config.get("device_id").and_then(|v| v.as_str()) {
            if !device_id.is_empty() && device_id != "default" {
                return device_id.to_string();
            }
        }
    }

    // Fall back to key prefix (format: "device_id:metric")
    if let Some(colon_pos) = key.find(':') {
        let prefix = &key[..colon_pos];
        // Only return if it looks like a device ID (not a known non-device prefix)
        if !prefix.is_empty() && prefix != "default" {
            return prefix.to_string();
        }
    }

    String::new()
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

    #[test]
    fn test_extract_device_id_from_source_config() {
        let sensor = serde_json::json!({
            "key": "temp",
            "source_type": "modbus",
            "source_config": "{\"device_id\": \"plts_1\", \"register\": \"temp\"}"
        });
        // source_config is a string here, not an object
        let device_id = extract_device_id(&sensor, "temp");
        // String source_config won't parse as object, falls back to key prefix
        assert_eq!(device_id, "");
    }

    #[test]
    fn test_extract_device_id_from_key_prefix() {
        let sensor = serde_json::json!({
            "key": "plts_1:voltage",
            "source_type": "modbus",
        });
        let device_id = extract_device_id(&sensor, "plts_1:voltage");
        assert_eq!(device_id, "plts_1");
    }

    #[test]
    fn test_extract_device_id_empty_for_default() {
        let sensor = serde_json::json!({
            "key": "default:something",
            "source_type": "random",
        });
        let device_id = extract_device_id(&sensor, "default:something");
        assert_eq!(device_id, "");
    }
}
