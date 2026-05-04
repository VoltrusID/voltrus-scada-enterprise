use std::collections::HashMap;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cache::DeviceCache;
use crate::models::{DeviceStatus, WriteResult};

use super::types::*;
use super::client::S7Client;

// ── Poller ──

/// Runtime config for an S7 device, used by the poller.
#[derive(Debug, Clone)]
pub struct S7DeviceConfig {
    pub id: String,
    pub url: String,
    pub rack: u8,
    pub slot: u8,
    pub interval_ms: u64,
    pub tags: Vec<S7Tag>,
}

/// Poll an S7 device in a loop. Intended to be spawned as a tokio task.
pub async fn poll_s7_device(config: S7DeviceConfig, cache: DeviceCache) {
    let device_id = config.id.clone();
    let mut backoff_secs: u64 = 1;

    loop {
        let mut client = S7Client::new(&config.url, config.rack, config.slot);

        match client.connect().await {
            Ok(()) => {
                tracing::info!("S7 device {} connected", device_id);
                backoff_secs = 1;

                loop {
                    let tags_to_read: Vec<S7Tag> = config.tags.clone();
                    match client.read_tags(&tags_to_read).await {
                        Ok(values) => {
                            let mut metrics = HashMap::new();
                            for (i, value) in values.iter().enumerate() {
                                let tag_name = if i < config.tags.len() {
                                    format!("{}_{}", config.tags[i].area.to_string().to_lowercase(),
                                            config.tags[i].offset)
                                } else {
                                    format!("tag_{}", i)
                                };
                                metrics.insert(tag_name, value.to_f64());
                            }
                            cache.update(&device_id, metrics, false).await;
                        }
                        Err(e) => {
                            tracing::warn!("S7 device {} read error: {}", device_id, e);
                            cache.set_status(&device_id, DeviceStatus::Offline).await;
                            break; // Reconnect
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(config.interval_ms)).await;
                }
            }
            Err(e) => {
                tracing::warn!(
                    "S7 device {} connection failed: {}, retrying in {}s",
                    device_id, e, backoff_secs
                );
                cache.set_status(&device_id, DeviceStatus::Offline).await;
            }
        }

        let _ = client.disconnect().await;
        tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(60);
    }
}

// ── One-shot write function ──

/// Write a single f64 value to an S7 PLC tag.
///
/// Connects to the PLC at `ip:port`, resolves the area string to an S7 area,
/// writes the value as a REAL (4-byte float), and returns a `WriteResult`.
///
/// The `area` parameter accepts strings like "DB", "Input", "Output", "Marker",
/// "Timer", or "Counter".
pub async fn write_tag(
    ip: &str,
    port: u16,
    area: &str,
    db_number: u32,
    offset: u16,
    value: f64,
) -> WriteResult {
    let start = std::time::Instant::now();
    let url = format!("{}:{}", ip, port);

    // Resolve area string
    let s7_area = match parse_area_string(area) {
        Some(a) => a,
        None => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Unknown S7 area: '{}'", area)),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
    };

    // Build an S7Tag for a REAL (f32) write at the given offset
    let tag = S7Tag {
        area: s7_area,
        db_number: db_number as u16,
        offset,
        bit_offset: 0,
        data_type: S7DataType::Real,
        length: 1,
    };

    let s7_value = S7Value::Real(value as f32);

    // Connect with timeout
    let mut client = S7Client::new(&url, 0, 1);
    let connect_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.connect(),
    ).await;

    match connect_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("S7 connection failed: {e}")),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(_) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some("S7 connect timeout (5s)".into()),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    // Write with timeout
    let write_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.write_tag(&tag, &s7_value),
    ).await;

    let _ = client.disconnect().await;

    match write_result {
        Ok(Ok(())) => WriteResult {
            success: true,
            device_response: Some(format!(
                "Wrote {} {} offset {} = {} (REAL)",
                area, db_number, offset, value
            )),
            error: None,
            latency_ms: start.elapsed().as_millis() as u64,
        },
        Ok(Err(e)) => WriteResult {
            success: false,
            device_response: None,
            error: Some(format!("S7 write failed: {e}")),
            latency_ms: start.elapsed().as_millis() as u64,
        },
        Err(_) => WriteResult {
            success: false,
            device_response: None,
            error: Some("S7 write timeout (5s)".into()),
            latency_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Parse an area string (case-insensitive) to an `S7Area`.
fn parse_area_string(s: &str) -> Option<S7Area> {
    match s.to_lowercase().as_str() {
        "db" | "data" => Some(S7Area::DB),
        "input" | "i" => Some(S7Area::Input),
        "output" | "q" | "o" => Some(S7Area::Output),
        "marker" | "m" => Some(S7Area::Marker),
        "timer" | "t" => Some(S7Area::Timer),
        "counter" | "c" => Some(S7Area::Counter),
        _ => None,
    }
}
