use super::*;
use crate::models::WriteResult;

impl CommandQueue {
    pub(super) async fn execute_write(&self, cmd: &crate::models::Command) -> WriteResult {
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

    pub(super) async fn execute_modbus_write(
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

    pub(super) async fn execute_s7_write(
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

    pub(super) async fn execute_opcua_write(
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

    pub(super) async fn execute_enip_write(
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
}
