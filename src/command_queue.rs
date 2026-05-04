use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cache::DeviceCache;
use crate::models::WriteResult;
use crate::storage::Storage;

const COMMAND_EXPIRY_SECS: i64 = 1800; // 30 minutes
const POLL_INTERVAL_MS: u64 = 500;

pub struct CommandQueue {
    storage: Storage,
    cache: DeviceCache,
    in_flight: Arc<RwLock<HashSet<i64>>>,
}

impl CommandQueue {
    pub fn new(storage: Storage, cache: DeviceCache) -> Self {
        Self {
            storage,
            cache,
            in_flight: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Main loop: process the queue every 500 ms.
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(POLL_INTERVAL_MS));
        loop {
            interval.tick().await;
            if let Err(e) = self.process_queue().await {
                tracing::warn!("Command queue processor error: {}", e);
            }
        }
    }

    async fn process_queue(&self) -> anyhow::Result<()> {
        let commands = self.storage.get_queued_commands().await?;

        for cmd in &commands {
            let now = crate::timeutil::now_secs();

            // ── Expiry check ──
            if cmd.status == "queued" && (cmd.created_at + COMMAND_EXPIRY_SECS) < now {
                self.publish_status_event(&cmd.id, "expired", None);
                self.storage
                    .update_command_status(
                        &cmd.id,
                        "expired",
                        Some("Command expired (30 min timeout)"),
                        "system",
                        Some("auto-expired"),
                    )
                    .await?;
                continue;
            }

            // ── Scheduled check ──
            if cmd.policy == "scheduled" {
                if let Some(scheduled_at) = cmd.scheduled_at {
                    if scheduled_at > now {
                        continue; // Not yet time
                    }
                }
            }

            // ── Concurrency check: only one command per device in-flight ──
            {
                let in_flight = self.in_flight.read().await;
                if in_flight.contains(&cmd.device_id) {
                    continue;
                }
            }

            // ── Device online check ──
            let device_id_str = cmd.device_id.to_string();
            if !self.cache.is_device_online(&device_id_str).await {
                continue;
            }

            // Mark as in-flight
            {
                let mut in_flight = self.in_flight.write().await;
                in_flight.insert(cmd.device_id);
            }

            // ── Execute ──
            self.publish_status_event(&cmd.id, "sent", None);
            self.storage
                .update_command_status(&cmd.id, "sent", None, "system", None)
                .await?;

            let result = self.execute_write(cmd).await;

            // ── Handle result ──
            match result.success {
                true => {
                    let response = result
                        .device_response
                        .clone()
                        .unwrap_or_else(|| "OK".to_string());
                    self.publish_ack_event(&cmd.id, &response, result.latency_ms);
                    self.storage
                        .update_command_status(&cmd.id, "acked", None, "system", None)
                        .await?;
                }
                false => {
                    let error = result.error.as_deref().unwrap_or("Unknown error");
                    if cmd.retry_count < cmd.max_retries {
                        self.publish_status_event(&cmd.id, "queued", Some(error));
                        self.storage
                            .update_command_status(
                                &cmd.id,
                                "queued",
                                Some(error),
                                "system",
                                Some("retry"),
                            )
                            .await?;
                    } else {
                        self.publish_status_event(&cmd.id, "failed", Some(error));
                        self.storage
                            .update_command_status(&cmd.id, "failed", Some(error), "system", None)
                            .await?;
                    }
                }
            }

            // Remove from in-flight
            {
                let mut in_flight = self.in_flight.write().await;
                in_flight.remove(&cmd.device_id);
            }
        }

        Ok(())
    }

    async fn execute_write(&self, cmd: &crate::models::Command) -> WriteResult {
        let value: f64 = match cmd.value.parse() {
            Ok(v) => v,
            Err(_) => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("Invalid value: '{}'", cmd.value)),
                    latency_ms: 0,
                }
            }
        };

        match cmd.protocol.as_str() {
            "modbus" => self.execute_modbus_write(cmd, value).await,
            "s7" => self.execute_s7_write(cmd, value).await,
            "opcua" => self.execute_opcua_write(cmd, value).await,
            "ethernet_ip" => self.execute_enip_write(cmd, value).await,
            "bacnet" => self.execute_bacnet_write(cmd, value).await,
            "mqtt" => WriteResult {
                success: true,
                device_response: Some("MQTT publish not implemented".to_string()),
                error: None,
                latency_ms: 0,
            },
            _ => WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Unsupported protocol: '{}'", cmd.protocol)),
                latency_ms: 0,
            },
        }
    }

    async fn execute_modbus_write(
        &self,
        cmd: &crate::models::Command,
        value: f64,
    ) -> WriteResult {
        // Look up the data source to get source_config (register address) and device_id
        let ds = match self.storage.get_data_source_by_id(cmd.data_source_id).await {
            Ok(Some(ds)) => ds,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Data source not found".to_string()),
                    latency_ms: 0,
                }
            }
        };

        // Parse register address from source_config JSON
        let address = match Self::parse_modbus_address(&ds.source_config) {
            Some(addr) => addr,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Could not determine register address from source_config".to_string()),
                    latency_ms: 0,
                }
            }
        };

        // Look up the Modbus device by its string id
        let device = match self.storage.get_modbus_device(&ds.device_id).await {
            Ok(Some(d)) => d,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("Modbus device '{}' not found", ds.device_id)),
                    latency_ms: 0,
                }
            }
        };

        let config = crate::models::DeviceConfig {
            id: device.id.clone(),
            ip: device.ip.clone(),
            port: device.port,
            unit_id: device.unit_id,
            interval_ms: device.interval_ms,
            endianness: match device.endianness.as_str() {
                "little" => crate::models::Endianness::Little,
                "swapped" => crate::models::Endianness::Swapped,
                _ => crate::models::Endianness::Big,
            },
            registers: vec![],
        };

        crate::modbus::write_tag(&config, address, value).await
    }

    async fn execute_s7_write(
        &self,
        cmd: &crate::models::Command,
        value: f64,
    ) -> WriteResult {
        let ds = match self.storage.get_data_source_by_id(cmd.data_source_id).await {
            Ok(Some(ds)) => ds,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Data source not found".to_string()),
                    latency_ms: 0,
                }
            }
        };

        let (area, db_number, offset) = match Self::parse_s7_address(&ds.source_config) {
            Some(v) => v,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Could not parse S7 address from source_config".to_string()),
                    latency_ms: 0,
                }
            }
        };

        let device = match self.storage.get_s7_device(cmd.device_id).await {
            Ok(Some(d)) => d,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("S7 device {} not found", cmd.device_id)),
                    latency_ms: 0,
                }
            }
        };

        // Parse ip:port from device URL
        let (ip, port) = Self::parse_ip_port(&device.url, 102);

        crate::s7::write_tag(&ip, port, &area, db_number, offset, value).await
    }

    async fn execute_opcua_write(
        &self,
        cmd: &crate::models::Command,
        value: f64,
    ) -> WriteResult {
        let ds = match self.storage.get_data_source_by_id(cmd.data_source_id).await {
            Ok(Some(ds)) => ds,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Data source not found".to_string()),
                    latency_ms: 0,
                }
            }
        };

        let node_id = match Self::parse_opcua_node_id(&ds.source_config) {
            Some(id) => id,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Could not parse OPC UA node_id from source_config".to_string()),
                    latency_ms: 0,
                }
            }
        };

        let device = match self.storage.get_opcua_device(cmd.device_id).await {
            Ok(Some(d)) => d,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("OPC UA device {} not found", cmd.device_id)),
                    latency_ms: 0,
                }
            }
        };

        crate::opcua::write_tag(&device.url, &node_id, value).await
    }

    async fn execute_enip_write(
        &self,
        cmd: &crate::models::Command,
        value: f64,
    ) -> WriteResult {
        let ds = match self.storage.get_data_source_by_id(cmd.data_source_id).await {
            Ok(Some(ds)) => ds,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Data source not found".to_string()),
                    latency_ms: 0,
                }
            }
        };

        let tag_name = match Self::parse_enip_tag_name(&ds.source_config) {
            Some(name) => name,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Could not parse EtherNet/IP tag name from source_config".to_string()),
                    latency_ms: 0,
                }
            }
        };

        let device = match self.storage.get_enip_device(cmd.device_id).await {
            Ok(Some(d)) => d,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("EtherNet/IP device {} not found", cmd.device_id)),
                    latency_ms: 0,
                }
            }
        };

        let (ip, port) = Self::parse_ip_port(&device.url, 44818);

        crate::ethernet_ip::write_tag(&ip, port, &tag_name, value).await
    }

    async fn execute_bacnet_write(
        &self,
        cmd: &crate::models::Command,
        value: f64,
    ) -> WriteResult {
        let ds = match self.storage.get_data_source_by_id(cmd.data_source_id).await {
            Ok(Some(ds)) => ds,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Data source not found".to_string()),
                    latency_ms: 0,
                }
            }
        };

        let (object_type, object_instance, property) = match Self::parse_bacnet_address(&ds.source_config) {
            Some(v) => v,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Could not parse BACnet address from source_config".to_string()),
                    latency_ms: 0,
                }
            }
        };

        let device = match self.storage.get_bacnet_device(cmd.device_id).await {
            Ok(Some(d)) => d,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("BACnet device {} not found", cmd.device_id)),
                    latency_ms: 0,
                }
            }
        };

        crate::bacnet::write_property(&device.ip, device.port, &object_type, object_instance, &property, value).await
    }

    fn parse_bacnet_address(source_config: &str) -> Option<(String, u32, String)> {
        let config: serde_json::Value = serde_json::from_str(source_config).ok()?;
        let object_type = config.get("object_type").and_then(|v| v.as_str()).unwrap_or("analogInput").to_string();
        let object_instance = config.get("object_instance").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        let property = config.get("property").and_then(|v| v.as_str()).unwrap_or("presentValue").to_string();
        Some((object_type, object_instance, property))
    }

    fn publish_status_event(&self, command_id: &str, status: &str, error: Option<&str>) {
        let payload = serde_json::json!({
            "type": "command_status",
            "command_id": command_id,
            "status": status,
            "error": error,
        })
        .to_string();
        self.cache.publish_sse_payload(payload);
    }

    fn publish_ack_event(&self, command_id: &str, response: &str, latency_ms: u64) {
        let payload = serde_json::json!({
            "type": "command_ack",
            "command_id": command_id,
            "status": "acked",
            "device_response": response,
            "latency_ms": latency_ms,
        })
        .to_string();
        self.cache.publish_sse_payload(payload);
    }

    // ── Address parsing helpers ──

    fn parse_modbus_address(source_config: &str) -> Option<u16> {
        let config: serde_json::Value = serde_json::from_str(source_config).ok()?;
        // Try "address" field first
        if let Some(addr) = config.get("address").and_then(|v| v.as_u64()) {
            let addr = addr as u16;
            // Convert 5-digit Modbus address to 0-based protocol address
            return Some(if addr >= 40001 {
                addr - 40001
            } else if addr >= 30001 {
                addr - 30001
            } else {
                addr
            });
        }
        // Try "register" field as fallback
        config.get("register").and_then(|v| v.as_u64()).map(|v| {
            let addr = v as u16;
            if addr >= 40001 {
                addr - 40001
            } else if addr >= 30001 {
                addr - 30001
            } else {
                addr
            }
        })
    }

    fn parse_s7_address(source_config: &str) -> Option<(String, u32, u16)> {
        let config: serde_json::Value = serde_json::from_str(source_config).ok()?;
        let area = config
            .get("area")
            .and_then(|v| v.as_str())
            .unwrap_or("DB")
            .to_string();
        let db_number = config
            .get("db_number")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;
        let offset = config
            .get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;
        Some((area, db_number, offset))
    }

    fn parse_opcua_node_id(source_config: &str) -> Option<String> {
        let config: serde_json::Value = serde_json::from_str(source_config).ok()?;
        config
            .get("node_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                config
                    .get("nodeId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
    }

    fn parse_enip_tag_name(source_config: &str) -> Option<String> {
        let config: serde_json::Value = serde_json::from_str(source_config).ok()?;
        config
            .get("tag_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                config
                    .get("tagName")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
    }

    fn parse_ip_port(url: &str, default_port: u16) -> (String, u16) {
        // Strip protocol prefix if present
        let url = url
            .strip_prefix("tcp://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        let parts: Vec<&str> = url.split(':').collect();
        let ip = parts[0].to_string();
        let port = parts
            .get(1)
            .and_then(|p| p.parse().ok())
            .unwrap_or(default_port);
        (ip, port)
    }
}
