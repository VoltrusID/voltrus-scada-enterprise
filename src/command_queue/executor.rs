use super::*;
#[cfg(feature = "dnp3")]
use crate::dnp3::master::Dnp3MasterSession;
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
                };
            }
        };

        match cmd.protocol.as_str() {
            #[cfg(feature = "gateway-modbus")]
            "modbus" => self.execute_modbus_write_gw(cmd, value).await,
            #[cfg(all(feature = "modbus", not(feature = "gateway-modbus")))]
            "modbus" => self.execute_modbus_write(cmd, value).await,
            #[cfg(feature = "gateway-s7")]
            "s7" => self.execute_s7_write_gw(cmd, value).await,
            #[cfg(all(feature = "s7", not(feature = "gateway-s7")))]
            "s7" => self.execute_s7_write(cmd, value).await,
            #[cfg(feature = "gateway-opcua")]
            "opcua" => self.execute_opcua_write_gw(cmd, value).await,
            #[cfg(all(feature = "opcua", not(feature = "gateway-opcua")))]
            "opcua" => self.execute_opcua_write(cmd, value).await,
            #[cfg(feature = "gateway-enip")]
            "ethernet_ip" => self.execute_enip_write_gw(cmd, value).await,
            #[cfg(all(feature = "ethernet-ip", not(feature = "gateway-enip")))]
            "ethernet_ip" => self.execute_enip_write(cmd, value).await,
            // MQTT is fire-and-forget publish, not request/response. Publish to
            // the topic in source_config via the embedded broker (see
            // execute_mqtt_write); without the broker feature it fails explicitly.
            "mqtt" => self.execute_mqtt_write(cmd, value).await,
            #[cfg(feature = "gateway-dnp3")]
            "dnp3" => self.execute_dnp3_write_gw(cmd, value).await,
            #[cfg(all(feature = "dnp3", not(feature = "gateway-dnp3")))]
            "dnp3" => self.execute_dnp3_write(cmd, value).await,
            #[cfg(feature = "gateway-bacnet")]
            "bacnet" => self.execute_bacnet_write_gw(cmd, value).await,
            #[cfg(all(feature = "bacnet", not(feature = "gateway-bacnet")))]
            "bacnet" => self.execute_bacnet_write(cmd, value).await,
            _ => WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Unsupported protocol: '{}'", cmd.protocol)),
                latency_ms: 0,
            },
        }
    }
}

#[cfg(feature = "modbus")]
impl CommandQueue {
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
                };
            }
        };

        // Parse register address from source_config JSON
        let address = match Self::parse_modbus_address(&ds.source_config) {
            Some(addr) => addr,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(
                        "Could not determine register address from source_config".to_string(),
                    ),
                    latency_ms: 0,
                };
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
                };
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
            readback: None,
            network: None,
        };

        let register = crate::models::RegisterConfig {
            name: "command".into(),
            address,
            reg_type: crate::models::RegisterType::Uint16,
            scale: 1.0,
            register_area: "holding".into(),
            zero_based: false,
        };
        crate::modbus::write_tag(&self.modbus_pool, &config, &register, value).await
    }
}

#[cfg(feature = "gateway-modbus")]
impl CommandQueue {
    pub(super) async fn execute_modbus_write_gw(
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
                };
            }
        };

        let address = match Self::parse_modbus_address(&ds.source_config) {
            Some(addr) => addr,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(
                        "Could not determine register address from source_config".to_string(),
                    ),
                    latency_ms: 0,
                };
            }
        };

        let device = match self.storage.get_modbus_device(&ds.device_id).await {
            Ok(Some(d)) => d,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("Modbus device '{}' not found", ds.device_id)),
                    latency_ms: 0,
                };
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
            readback: None,
            network: None,
        };

        let register = crate::models::RegisterConfig {
            name: "command".into(),
            address,
            reg_type: crate::models::RegisterType::Uint16,
            scale: 1.0,
            register_area: "holding".into(),
            zero_based: false,
        };

        crate::gateway_client::write_modbus_register(&config, &register, value).await
    }
}

#[cfg(feature = "gateway-opcua")]
impl CommandQueue {
    pub(super) async fn execute_opcua_write_gw(
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
                };
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
                };
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
                };
            }
        };

        let config = crate::models::OpcUaDeviceConfig {
            id: device.name.clone(),
            url: device.url.clone(),
            security_policy: device.security_policy.clone(),
            security_mode: device.security_mode.clone(),
            username: device.username.clone(),
            password: device.password.clone(),
            interval_ms: device.interval_ms,
            nodes: vec![],
        };

        crate::gateway_client::write_opcua_node(&config, &node_id, value).await
    }
}

#[cfg(feature = "gateway-enip")]
impl CommandQueue {
    pub(super) async fn execute_enip_write_gw(
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
                };
            }
        };

        let tag_name = match Self::parse_enip_tag_name(&ds.source_config) {
            Some(name) => name,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(
                        "Could not parse EtherNet/IP tag name from source_config".to_string(),
                    ),
                    latency_ms: 0,
                };
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
                };
            }
        };

        let config = crate::models::EnipDeviceConfig {
            id: device.name.clone(),
            url: device.url.clone(),
            slot: device.slot,
            interval_ms: device.interval_ms,
            assembly_instance: device.assembly_instance,
            tags: vec![],
        };

        crate::gateway_client::write_enip_tag(&config, &tag_name, value).await
    }
}

#[cfg(feature = "gateway-s7")]
impl CommandQueue {
    pub(super) async fn execute_s7_write_gw(
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
                };
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
                };
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
                };
            }
        };

        let config = crate::s7::S7DeviceConfig {
            id: device.name.clone(),
            url: device.url.clone(),
            rack: device.rack,
            slot: device.slot,
            interval_ms: device.interval_ms,
            tags: vec![],
        };

        crate::gateway_client::write_s7_tag(&config, &area, db_number, offset, value).await
    }
}

#[cfg(feature = "s7")]
impl CommandQueue {
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
                };
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
                };
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
                };
            }
        };

        // Parse ip:port from device URL
        let (ip, port) = Self::parse_ip_port(&device.url, 102);

        crate::s7::write_tag(&ip, port, &area, db_number, offset, value, None).await
    }
}

#[cfg(feature = "opcua")]
impl CommandQueue {
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
                };
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
                };
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
                };
            }
        };

        crate::opcua::write_tag(&device.url, &node_id, value).await
    }
}

#[cfg(feature = "ethernet-ip")]
impl CommandQueue {
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
                };
            }
        };

        let tag_name = match Self::parse_enip_tag_name(&ds.source_config) {
            Some(name) => name,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(
                        "Could not parse EtherNet/IP tag name from source_config".to_string(),
                    ),
                    latency_ms: 0,
                };
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
                };
            }
        };

        let (ip, port) = Self::parse_ip_port(&device.url, 44818);

        crate::ethernet_ip::write_tag(&ip, port, &tag_name, value).await
    }
}

#[cfg(feature = "gateway-dnp3")]
impl CommandQueue {
    pub(super) async fn execute_dnp3_write_gw(
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
                };
            }
        };

        let point_index = match Self::parse_dnp3_point_index(&ds.source_config) {
            Some(idx) => idx,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Could not parse DNP3 point index from source_config".to_string()),
                    latency_ms: 0,
                };
            }
        };

        let device = match self.storage.get_dnp3_device(cmd.device_id).await {
            Ok(Some(d)) => d,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("DNP3 device {} not found", cmd.device_id)),
                    latency_ms: 0,
                };
            }
        };

        let config = crate::models::Dnp3DeviceConfig {
            id: device.name.clone(),
            ip: device.ip.clone(),
            port: device.port,
            master_address: device.master_address,
            outstation_address: device.outstation_address,
            event_scan_interval_ms: device.event_scan_interval_ms,
            points: vec![],
        };

        crate::gateway_client::write_dnp3_point(&config, point_index, value).await
    }
}

#[cfg(feature = "dnp3")]
impl CommandQueue {
    pub(super) async fn execute_dnp3_write(
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
                };
            }
        };

        let point_index = match Self::parse_dnp3_point_index(&ds.source_config) {
            Some(idx) => idx,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Could not parse DNP3 point index from source_config".to_string()),
                    latency_ms: 0,
                };
            }
        };

        let device = match self.storage.get_dnp3_device(cmd.device_id).await {
            Ok(Some(d)) => d,
            _ => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("DNP3 device {} not found", cmd.device_id)),
                    latency_ms: 0,
                };
            }
        };

        let on = value != 0.0;
        match Dnp3MasterSession::connect(
            &device.ip,
            device.port,
            device.master_address,
            device.outstation_address,
        )
        .await
        {
            Ok(mut session) => session
                .direct_operate_binary(point_index, on)
                .await
                .unwrap_or_else(|e| WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("DNP3 write failed: {e}")),
                    latency_ms: 0,
                }),
            Err(e) => WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("DNP3 connect failed: {e}")),
                latency_ms: 0,
            },
        }
    }
}

#[cfg(feature = "gateway-bacnet")]
impl CommandQueue {
    pub(super) async fn execute_bacnet_write_gw(
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
                };
            }
        };

        let (object_type, object_instance, property) =
            match Self::parse_bacnet_object_info(&ds.source_config) {
                Some(info) => info,
                None => {
                    return WriteResult {
                        success: false,
                        device_response: None,
                        error: Some(
                            "Could not parse BACnet object info from source_config".to_string(),
                        ),
                        latency_ms: 0,
                    };
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
                };
            }
        };

        let config = crate::models::BacnetDeviceConfig {
            id: device.name.clone(),
            ip: device.ip.clone(),
            port: device.port,
            device_instance: device.device_instance,
            interval_ms: device.interval_ms,
            cov_enabled: device.cov_enabled,
            cov_lifetime: device.cov_lifetime,
            objects: vec![],
        };

        crate::gateway_client::write_bacnet_point(
            &config,
            &object_type,
            object_instance,
            &property,
            value,
        )
        .await
    }
}

#[cfg(feature = "bacnet")]
impl CommandQueue {
    pub(super) async fn execute_bacnet_write(
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
                };
            }
        };

        let (object_type, object_instance, property) =
            match Self::parse_bacnet_object_info(&ds.source_config) {
                Some(info) => info,
                None => {
                    return WriteResult {
                        success: false,
                        device_response: None,
                        error: Some(
                            "Could not parse BACnet object info from source_config".to_string(),
                        ),
                        latency_ms: 0,
                    };
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
                };
            }
        };

        crate::bacnet::write_property(
            &device.ip,
            device.port,
            &object_type,
            object_instance,
            &property,
            value,
        )
        .await
    }
}

#[cfg(feature = "mqtt-broker")]
impl CommandQueue {
    /// Publish the command value to the MQTT topic in the data source's
    /// `source_config` (`{"topic": "..."}`). MQTT has no device ACK, so success
    /// means "the broker accepted the publish", not "the device acted on it" —
    /// the closest faithful analogue of the request/response protocols' ACK.
    pub(super) async fn execute_mqtt_write(
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
                };
            }
        };

        let topic = match Self::parse_mqtt_topic(&ds.source_config) {
            Some(t) => t,
            None => {
                return WriteResult {
                    success: false,
                    device_response: None,
                    error: Some("Could not parse MQTT topic from source_config".to_string()),
                    latency_ms: 0,
                };
            }
        };

        let payload = Self::mqtt_payload(&ds.source_config, value);
        let started = std::time::Instant::now();
        let broker = self.mqtt_broker.read().await;
        match broker.as_ref() {
            Some(b) => match b.publish_sync(&topic, payload.as_bytes()) {
                Ok(_) => WriteResult {
                    success: true,
                    device_response: Some(format!("published {} bytes to {topic}", payload.len())),
                    error: None,
                    latency_ms: started.elapsed().as_millis() as u64,
                },
                Err(e) => WriteResult {
                    success: false,
                    device_response: None,
                    error: Some(format!("MQTT publish failed: {e}")),
                    latency_ms: started.elapsed().as_millis() as u64,
                },
            },
            None => WriteResult {
                success: false,
                device_response: None,
                error: Some("MQTT broker is not running; cannot publish command".to_string()),
                latency_ms: 0,
            },
        }
    }
}

#[cfg(not(feature = "mqtt-broker"))]
impl CommandQueue {
    pub(super) async fn execute_mqtt_write(
        &self,
        _cmd: &crate::models::Command,
        _value: f64,
    ) -> WriteResult {
        WriteResult {
            success: false,
            device_response: None,
            error: Some("MQTT broker feature is not enabled in this build".to_string()),
            latency_ms: 0,
        }
    }
}
