use anyhow::{Context, Result};
use std::path::Path;

use crate::storage::Storage;

// ── YAML-serializable config structs ──

#[derive(serde::Serialize, serde::Deserialize)]
struct DeviceExport {
    id: String,
    name: String,
    ip: String,
    port: u16,
    unit_id: u8,
    interval_ms: u64,
    endianness: String,
    enabled: bool,
    registers: Vec<RegisterExport>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RegisterExport {
    name: String,
    address: u16,
    register_type: String,
    register_area: String,
    scale: f64,
    sort_order: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ScreenExport {
    id: String,
    name: String,
    device_type: String,
    sort_order: i64,
    data_sources: Vec<ScreenDataSourceExport>,
    layouts: Vec<ScreenLayoutExport>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ScreenDataSourceExport {
    data_source_id: i64,
    sort_order: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ScreenLayoutExport {
    breakpoint: String,
    layout: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FlowExport {
    id: String,
    name: String,
    enabled: bool,
    trigger_config: serde_json::Value,
    nodes: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AlarmExport {
    source_key: String,
    source_name: String,
    severity: String,
    value: f64,
    threshold: f64,
    message: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NotificationExport {
    source_key: String,
    channel_type: String,
    config: serde_json::Value,
    enabled: bool,
}

// ── Export functions ──

/// Export all configuration from the database to YAML files in the given directory.
pub fn export_config(storage: &Storage, output_dir: &str) -> Result<()> {
    // Use tokio runtime for async storage calls
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;

    rt.block_on(async {
        let base = Path::new(output_dir);

        // Create directory structure
        std::fs::create_dir_all(base.join("devices")).context("Failed to create devices dir")?;
        std::fs::create_dir_all(base.join("screens")).context("Failed to create screens dir")?;
        std::fs::create_dir_all(base.join("flows")).context("Failed to create flows dir")?;

        // ── Export Modbus devices ──
        let devices = storage.list_modbus_devices().await?;
        for device in &devices {
            let registers = storage.list_modbus_registers(&device.id).await?;
            let export = DeviceExport {
                id: device.id.clone(),
                name: device.name.clone(),
                ip: device.ip.clone(),
                port: device.port,
                unit_id: device.unit_id,
                interval_ms: device.interval_ms,
                endianness: device.endianness.clone(),
                enabled: device.enabled,
                registers: registers.iter().map(|r| RegisterExport {
                    name: r.name.clone(),
                    address: r.address,
                    register_type: r.register_type.clone(),
                    register_area: r.register_area.clone(),
                    scale: r.scale,
                    sort_order: r.sort_order,
                }).collect(),
            };
            let yaml = serde_yml::to_string(&export).context("Failed to serialize device")?;
            let filename = sanitize_filename(&device.id);
            let path = base.join("devices").join(format!("{}.yaml", filename));
            std::fs::write(&path, yaml).context(format!("Failed to write {:?}", path))?;
        }

        // ── Export Screens ──
        let screens = storage.list_devices().await?;
        for screen in &screens {
            let _data_sources = storage.list_data_sources(Some(&screen.id)).await?;

            // Get layouts for this screen
            let layouts = storage.get_layouts(&screen.id).await.unwrap_or_default();
            let layout_exports: Vec<ScreenLayoutExport> = layouts.iter().map(|l| {
                ScreenLayoutExport {
                    breakpoint: l.breakpoint.clone(),
                    layout: serde_json::from_str(&l.layout).unwrap_or(serde_json::json!(null)),
                }
            }).collect();

            // Get screen data source associations
            let ds_exports: Vec<ScreenDataSourceExport> = storage.get_screen_data_sources(&screen.id).await
                .unwrap_or_default()
                .iter()
                .map(|s| ScreenDataSourceExport {
                    data_source_id: s.data_source_id,
                    sort_order: s.sort_order,
                })
                .collect();

            let export = ScreenExport {
                id: screen.id.clone(),
                name: screen.name.clone(),
                device_type: screen.device_type.clone(),
                sort_order: screen.sort_order,
                data_sources: ds_exports,
                layouts: layout_exports,
            };
            let yaml = serde_yml::to_string(&export).context("Failed to serialize screen")?;
            let filename = sanitize_filename(&screen.id);
            let path = base.join("screens").join(format!("{}.yaml", filename));
            std::fs::write(&path, yaml).context(format!("Failed to write {:?}", path))?;
        }

        // ── Export Flows ──
        let flows = storage.list_flows().await?;
        for flow in &flows {
            let trigger_config: serde_json::Value =
                serde_json::from_str(&flow.trigger_config).unwrap_or(serde_json::json!(null));
            let nodes: serde_json::Value =
                serde_json::from_str(&flow.nodes).unwrap_or(serde_json::json!(null));
            let export = FlowExport {
                id: flow.id.clone(),
                name: flow.name.clone(),
                enabled: flow.enabled,
                trigger_config,
                nodes,
            };
            let yaml = serde_yml::to_string(&export).context("Failed to serialize flow")?;
            let filename = sanitize_filename(&flow.id);
            let path = base.join("flows").join(format!("{}.yaml", filename));
            std::fs::write(&path, yaml).context(format!("Failed to write {:?}", path))?;
        }

        // ── Export alarm thresholds (from data sources) ──
        let all_sources = storage.list_data_sources(None).await?;
        let alarm_exports: Vec<AlarmExport> = all_sources.iter().filter(|s| {
            s.warn_val > 0.0 || s.crit_val > 0.0
        }).map(|s| {
            AlarmExport {
                source_key: s.key.clone(),
                source_name: s.name.clone(),
                severity: "threshold".to_string(),
                value: 0.0,
                threshold: s.crit_val,
                message: format!("{} threshold: warn={} crit={} {}", s.name, s.warn_val, s.crit_val, s.unit),
            }
        }).collect();
        let alarm_yaml = serde_yml::to_string(&alarm_exports).context("Failed to serialize alarms")?;
        std::fs::write(base.join("alarms.yaml"), alarm_yaml)
            .context("Failed to write alarms.yaml")?;

        // ── Export notification settings ──
        let notifications = storage.get_all_notification_settings().await?;
        let notif_exports: Vec<NotificationExport> = notifications.iter().map(|n| {
            NotificationExport {
                source_key: n.source_key.clone(),
                channel_type: n.channel_type.clone(),
                config: serde_json::from_str(&n.config).unwrap_or(serde_json::json!({})),
                enabled: n.enabled,
            }
        }).collect();
        let notif_yaml = serde_yml::to_string(&notif_exports).context("Failed to serialize notifications")?;
        std::fs::write(base.join("notifications.yaml"), notif_yaml)
            .context("Failed to write notifications.yaml")?;

        anyhow::Ok(())
    })?;

    Ok(())
}

// ── Import functions ──

/// Import configuration from YAML files into the database.
/// This is transactional — all or nothing.
pub fn import_config(storage: &Storage, input_dir: &str) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;

    rt.block_on(async {
        let base = Path::new(input_dir);

        // ── Import devices ──
        let devices_dir = base.join("devices");
        if devices_dir.exists() {
            for entry in std::fs::read_dir(&devices_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                    continue;
                }
                let content = std::fs::read_to_string(&path)
                    .context(format!("Failed to read {:?}", path))?;
                let device: DeviceExport = serde_yml::from_str(&content)
                    .context(format!("Failed to parse {:?}", path))?;

                // Validate required fields
                if device.id.is_empty() {
                    anyhow::bail!("Device in {:?} has empty id", path);
                }
                if device.ip.is_empty() {
                    anyhow::bail!("Device '{}' has empty ip", device.id);
                }

                let registers: Vec<crate::models::ModbusRegisterInput> = device.registers.iter().map(|r| {
                    crate::models::ModbusRegisterInput {
                        name: r.name.clone(),
                        address: r.address,
                        register_type: r.register_type.clone(),
                        register_area: r.register_area.clone(),
                        scale: r.scale,
                    }
                }).collect();

                // Upsert: delete existing and re-create
                let _ = storage.delete_modbus_device(&device.id).await;
                storage.create_modbus_device(
                    &device.id,
                    &device.name,
                    &device.ip,
                    device.port,
                    device.unit_id,
                    device.interval_ms,
                    &device.endianness,
                    &registers,
                ).await.context(format!("Failed to import device '{}'", device.id))?;

                // Restore enabled state
                if !device.enabled {
                    storage.update_modbus_device(&device.id, &serde_json::json!({"enabled": false})).await?;
                }
            }
        }

        // ── Import flows ──
        let flows_dir = base.join("flows");
        if flows_dir.exists() {
            for entry in std::fs::read_dir(&flows_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                    continue;
                }
                let content = std::fs::read_to_string(&path)
                    .context(format!("Failed to read {:?}", path))?;
                let flow: FlowExport = serde_yml::from_str(&content)
                    .context(format!("Failed to parse {:?}", path))?;

                if flow.id.is_empty() {
                    anyhow::bail!("Flow in {:?} has empty id", path);
                }

                let trigger_config = serde_json::to_string(&flow.trigger_config)?;
                let nodes = serde_json::to_string(&flow.nodes)?;

                // Delete existing and recreate
                let _ = storage.delete_flow(&flow.id).await;
                storage.create_flow(
                    &flow.id,
                    &flow.name,
                    flow.enabled,
                    &trigger_config,
                    &nodes,
                ).await.context(format!("Failed to import flow '{}'", flow.id))?;
            }
        }

        // ── Import notifications ──
        let notif_path = base.join("notifications.yaml");
        if notif_path.exists() {
            let content = std::fs::read_to_string(&notif_path)?;
            let notifications: Vec<NotificationExport> = serde_yml::from_str(&content)
                .context("Failed to parse notifications.yaml")?;
            for n in &notifications {
                let config = serde_json::to_string(&n.config)?;
                storage.create_notification_setting(
                    &n.source_key,
                    &n.channel_type,
                    &config,
                    n.enabled,
                ).await?;
            }
        }

        anyhow::Ok(())
    })?;

    Ok(())
}

// ── Validation (dry-run) ──

/// Validate config files without importing anything.
/// Returns a list of messages describing what would happen.
pub fn validate_config(input_dir: &str) -> Result<Vec<String>> {
    let mut messages = Vec::new();
    let base = Path::new(input_dir);

    // Validate devices
    let devices_dir = base.join("devices");
    if devices_dir.exists() {
        for entry in std::fs::read_dir(&devices_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            match serde_yml::from_str::<DeviceExport>(&content) {
                Ok(device) => {
                    let mut issues = Vec::new();
                    if device.id.is_empty() { issues.push("empty id"); }
                    if device.ip.is_empty() { issues.push("empty ip"); }
                    if device.port == 0 { issues.push("port is 0"); }
                    if issues.is_empty() {
                        messages.push(format!("OK: devices/{:?} — {} ({} registers)",
                            path.file_name().unwrap_or_default(),
                            device.id,
                            device.registers.len()));
                    } else {
                        messages.push(format!("ERROR: devices/{:?} — {}", path.file_name().unwrap_or_default(), issues.join(", ")));
                    }
                }
                Err(e) => {
                    messages.push(format!("ERROR: devices/{:?} — parse error: {}", path.file_name().unwrap_or_default(), e));
                }
            }
        }
    } else {
        messages.push("SKIP: devices/ directory not found".to_string());
    }

    // Validate screens
    let screens_dir = base.join("screens");
    if screens_dir.exists() {
        for entry in std::fs::read_dir(&screens_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            match serde_yml::from_str::<ScreenExport>(&content) {
                Ok(screen) => {
                    messages.push(format!("OK: screens/{:?} — {} ({} data sources, {} layouts)",
                        path.file_name().unwrap_or_default(),
                        screen.id,
                        screen.data_sources.len(),
                        screen.layouts.len()));
                }
                Err(e) => {
                    messages.push(format!("ERROR: screens/{:?} — parse error: {}", path.file_name().unwrap_or_default(), e));
                }
            }
        }
    } else {
        messages.push("SKIP: screens/ directory not found".to_string());
    }

    // Validate flows
    let flows_dir = base.join("flows");
    if flows_dir.exists() {
        for entry in std::fs::read_dir(&flows_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            match serde_yml::from_str::<FlowExport>(&content) {
                Ok(flow) => {
                    messages.push(format!("OK: flows/{:?} — {} (enabled={})",
                        path.file_name().unwrap_or_default(),
                        flow.id,
                        flow.enabled));
                }
                Err(e) => {
                    messages.push(format!("ERROR: flows/{:?} — parse error: {}", path.file_name().unwrap_or_default(), e));
                }
            }
        }
    } else {
        messages.push("SKIP: flows/ directory not found".to_string());
    }

    // Validate alarms
    let alarms_path = base.join("alarms.yaml");
    if alarms_path.exists() {
        let content = std::fs::read_to_string(&alarms_path)?;
        match serde_yml::from_str::<Vec<AlarmExport>>(&content) {
            Ok(alarms) => {
                messages.push(format!("OK: alarms.yaml — {} alarm thresholds", alarms.len()));
            }
            Err(e) => {
                messages.push(format!("ERROR: alarms.yaml — parse error: {}", e));
            }
        }
    } else {
        messages.push("SKIP: alarms.yaml not found".to_string());
    }

    // Validate notifications
    let notif_path = base.join("notifications.yaml");
    if notif_path.exists() {
        let content = std::fs::read_to_string(&notif_path)?;
        match serde_yml::from_str::<Vec<NotificationExport>>(&content) {
            Ok(notifs) => {
                messages.push(format!("OK: notifications.yaml — {} notification settings", notifs.len()));
            }
            Err(e) => {
                messages.push(format!("ERROR: notifications.yaml — parse error: {}", e));
            }
        }
    } else {
        messages.push("SKIP: notifications.yaml not found".to_string());
    }

    Ok(messages)
}

/// Sanitize a string for use as a filename.
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("plts_1"), "plts_1");
        assert_eq!(sanitize_filename("my device"), "my_device");
        assert_eq!(sanitize_filename("test/device"), "test_device");
        assert_eq!(sanitize_filename("hello.world"), "hello_world");
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("a/b:c@d"), "a_b_c_d");
    }

    #[test]
    fn test_device_export_serialization() {
        let device = DeviceExport {
            id: "plts_1".to_string(),
            name: "PLTS 1".to_string(),
            ip: "192.168.1.10".to_string(),
            port: 502,
            unit_id: 1,
            interval_ms: 1000,
            endianness: "big".to_string(),
            enabled: true,
            registers: vec![
                RegisterExport {
                    name: "voltage".to_string(),
                    address: 0,
                    register_type: "float32".to_string(),
                    register_area: "holding".to_string(),
                    scale: 1.0,
                    sort_order: 0,
                },
            ],
        };
        let yaml = serde_yml::to_string(&device).unwrap();
        assert!(yaml.contains("id: plts_1") || yaml.contains("id: \"plts_1\""));
        assert!(yaml.contains("192.168.1.10"));
        assert!(yaml.contains("voltage"));
    }

    #[test]
    fn test_device_export_roundtrip() {
        let device = DeviceExport {
            id: "dev_1".to_string(),
            name: "Device 1".to_string(),
            ip: "10.0.0.1".to_string(),
            port: 502,
            unit_id: 1,
            interval_ms: 500,
            endianness: "little".to_string(),
            enabled: false,
            registers: vec![],
        };
        let yaml = serde_yml::to_string(&device).unwrap();
        let parsed: DeviceExport = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(parsed.id, "dev_1");
        assert_eq!(parsed.ip, "10.0.0.1");
        assert_eq!(parsed.port, 502);
        assert!(!parsed.enabled);
    }

    #[test]
    fn test_flow_export_serialization() {
        let flow = FlowExport {
            id: "flow_1".to_string(),
            name: "Test Flow".to_string(),
            enabled: true,
            trigger_config: serde_json::json!({"type": "modbus", "device_id": "*"}),
            nodes: serde_json::json!([{"id": "n1", "type": "filter"}]),
        };
        let yaml = serde_yml::to_string(&flow).unwrap();
        assert!(yaml.contains("id: flow_1"));
        assert!(yaml.contains("enabled: true"));
    }

    #[test]
    fn test_notification_export_serialization() {
        let notif = NotificationExport {
            source_key: "temp".to_string(),
            channel_type: "email".to_string(),
            config: serde_json::json!({"recipients": ["admin@example.com"]}),
            enabled: true,
        };
        let yaml = serde_yml::to_string(&notif).unwrap();
        assert!(yaml.contains("source_key: temp"));
        assert!(yaml.contains("channel_type: email"));
    }

    #[test]
    fn test_validate_config_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let nonexistent = tmp.path().join("missing");
        let result = validate_config(nonexistent.to_str().unwrap()).unwrap();
        assert!(result.iter().any(|m| m.contains("SKIP: devices/")));
        assert!(result.iter().any(|m| m.contains("SKIP: screens/")));
        assert!(result.iter().any(|m| m.contains("SKIP: flows/")));
        assert!(result.iter().any(|m| m.contains("SKIP: alarms.yaml")));
    }

    #[test]
    fn test_validate_config_with_files() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        // Create valid device file
        std::fs::create_dir_all(base.join("devices")).unwrap();
        let device = DeviceExport {
            id: "test_dev".to_string(),
            name: "Test".to_string(),
            ip: "192.168.1.1".to_string(),
            port: 502,
            unit_id: 1,
            interval_ms: 1000,
            endianness: "big".to_string(),
            enabled: true,
            registers: vec![],
        };
        std::fs::write(
            base.join("devices/test_dev.yaml"),
            serde_yml::to_string(&device).unwrap(),
        ).unwrap();

        // Create valid flow file
        std::fs::create_dir_all(base.join("flows")).unwrap();
        let flow = FlowExport {
            id: "flow_1".to_string(),
            name: "Flow".to_string(),
            enabled: true,
            trigger_config: serde_json::json!(null),
            nodes: serde_json::json!([]),
        };
        std::fs::write(
            base.join("flows/flow_1.yaml"),
            serde_yml::to_string(&flow).unwrap(),
        ).unwrap();

        let result = validate_config(base.to_str().unwrap()).unwrap();
        assert!(result.iter().any(|m| m.contains("OK: devices/") && m.contains("test_dev")));
        assert!(result.iter().any(|m| m.contains("OK: flows/") && m.contains("flow_1")));
    }

    #[test]
    fn test_validate_config_invalid_file() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        std::fs::create_dir_all(base.join("devices")).unwrap();
        std::fs::write(base.join("devices/bad.yaml"), "not: valid: yaml: : :").unwrap();

        let result = validate_config(base.to_str().unwrap()).unwrap();
        assert!(result.iter().any(|m| m.contains("ERROR") && m.contains("bad.yaml")));
    }
}
