# v0.32.0 Track 3 — Independent Systems (Node-RED Importer, MQTT Broker)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Node-RED flows.json importer and embedded MQTT broker for v0.32.0.

**Architecture:** Node-RED parser walks flows.json and maps node types to Voltrus devices/tags/flow-rules/dashboard-widgets with warning-level granularity for unsupported nodes. MQTT broker uses rumqttd embedded in-process, configured via config.yaml, running as a background tokio task.

**Tech Stack:** Rust/Axum, rumqttd, SQLite, serde_json, React 19, TypeScript, TanStack Query

---

## Table of Contents

- [Task 1 — Cargo.toml: Add rumqttd dependency](#task-1)
- [Task 2 — Config model: Add mqtt_broker section](#task-2)
- [Task 3 — Backend models: Node-RED import types](#task-3)
- [Task 4 — Parser: flows.json deserialization](#task-4)
- [Task 5 — Converter: Node-RED nodes to Voltrus entities](#task-5)
- [Task 6 — Import API endpoint](#task-6)
- [Task 7 — MQTT broker module: config, auth, lifecycle](#task-7)
- [Task 8 — MQTT broker API endpoints](#task-8)
- [Task 9 — MQTT broker integration in main.rs](#task-9)
- [Task 10 — License tier: mqtt_broker flag](#task-10)
- [Task 11 — Frontend: query hooks + keys](#task-11)
- [Task 12 — Frontend: NodeRedImportWizard component](#task-12)
- [Task 13 — Frontend: MqttBrokerPanel component](#task-13)
- [Task 14 — Frontend: App.tsx integration](#task-14)

---

## Task 1 — Cargo.toml: Add rumqttd dependency

Add the `rumqttd` crate to `Cargo.toml`.

**Files:**
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/Cargo.toml`

**Steps:**

- [ ] Add `rumqttd = "0.21"` to `[dependencies]` in Cargo.toml

```toml
# In [dependencies], after the `bacnet-rs = "0.3"` line, add:
rumqttd = "0.21"
```

- [ ] Verify it compiles:

```bash
cargo check 2>&1 | head -20
```

- [ ] **Commit:** `feat(deps): add rumqttd crate for embedded MQTT broker`

---

## Task 2 — Config model: Add mqtt_broker section

Add the `mqtt_broker` optional config section to `AppConfig` and the `MqttBrokerConfig` struct.

**Files:**
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/models/config_types.rs`
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/models/mod.rs`

**Steps:**

- [ ] Add the `MqttBrokerConfig` struct to `config_types.rs`:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MqttBrokerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_mqtt_port")]
    pub port: u16,
    #[serde(default = "default_mqtt_max_connections")]
    pub max_connections: usize,
    #[serde(default)]
    pub auth: Option<MqttAuthConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MqttAuthConfig {
    #[serde(default)]
    pub enabled: bool,
    /// JSON array of {username, password} objects stored in config.
    /// In production, consider hashing passwords.
    pub users: Vec<MqttUserEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MqttUserEntry {
    pub username: String,
    pub password: String,
}

fn default_mqtt_port() -> u16 { 1883 }
fn default_mqtt_max_connections() -> usize { 1000 }
```

- [ ] Add `mqtt_broker` field to `AppConfig`:

```rust
// In AppConfig struct, after the `push` field:
#[serde(default)]
pub mqtt_broker: Option<MqttBrokerConfig>,
```

- [ ] Export the new types from `mod.rs`:

```rust
// In src/models/mod.rs, add to the pub use block:
pub use config_types::MqttBrokerConfig;
pub use config_types::MqttAuthConfig;
pub use config_types::MqttUserEntry;
```

- [ ] Write tests in `config_types.rs`:

```rust
#[cfg(test)]
mod mqtt_config_tests {
    use super::*;

    #[test]
    fn test_mqtt_broker_defaults() {
        let yaml = r#"
devices: []
"#;
        let config: AppConfig = serde_yaml_neo::from_str(yaml).unwrap();
        assert!(config.mqtt_broker.is_none());
    }

    #[test]
    fn test_mqtt_broker_enabled() {
        let yaml = r#"
devices: []
mqtt_broker:
  enabled: true
  port: 1884
"#;
        let config: AppConfig = serde_yaml_neo::from_str(yaml).unwrap();
        let mqtt = config.mqtt_broker.unwrap();
        assert!(mqtt.enabled);
        assert_eq!(mqtt.port, 1884);
        assert_eq!(mqtt.max_connections, 1000);
        assert!(mqtt.auth.is_none());
    }

    #[test]
    fn test_mqtt_broker_with_auth() {
        let yaml = r#"
devices: []
mqtt_broker:
  enabled: true
  auth:
    enabled: true
    users:
      - username: "admin"
        password: "secret"
"#;
        let config: AppConfig = serde_yaml_neo::from_str(yaml).unwrap();
        let mqtt = config.mqtt_broker.unwrap();
        let auth = mqtt.auth.unwrap();
        assert!(auth.enabled);
        assert_eq!(auth.users.len(), 1);
        assert_eq!(auth.users[0].username, "admin");
    }
}
```

- [ ] Run tests:

```bash
cargo test --lib models::config_types::mqtt_config_tests
```

- [ ] **Commit:** `feat(config): add mqtt_broker configuration section to AppConfig`

---

## Task 3 — Backend models: Node-RED import types

Create the data types for the Node-RED import feature.

**Files:**
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/import_nodered/mod.rs`
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/import_nodered/parser.rs`
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/import_nodered/converter.rs`
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/main.rs`

**Steps:**

- [ ] Create `src/import_nodered/mod.rs`:

```rust
pub mod parser;
pub mod converter;
```

- [ ] Create `src/import_nodered/parser.rs` with the Node-RED flows.json data model:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level Node-RED flows.json structure (array of nodes).
/// Node-RED exports a JSON array of node objects plus optional metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeRedFlow {
    /// Array of nodes in the flow.
    #[serde(default)]
    pub nodes: Vec<NodeRedNode>,
}

/// A single node in a Node-RED flow.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeRedNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: Option<String>,
    /// Tab/subflow this node belongs to.
    #[serde(default)]
    pub z: String,
    /// Wiring: array of connected node IDs (output wires).
    #[serde(default)]
    pub wires: Vec<Vec<String>>,
    /// Node-RED specific properties stored as generic JSON.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
    /// For function nodes: the JavaScript source code.
    #[serde(default)]
    pub func: Option<String>,
    /// For modbus nodes: server, unit ID, etc.
    #[serde(default)]
    pub server: Option<String>,
    /// Topic for MQTT nodes.
    #[serde(default)]
    pub topic: Option<String>,
    /// QoS for MQTT nodes.
    #[serde(default)]
    pub qos: Option<u8>,
    /// Broker URL for MQTT nodes.
    #[serde(default)]
    pub broker: Option<String>,
    /// For modbus-read: FC (function code), address, quantity, etc.
    #[serde(default)]
    pub fc: Option<u8>,
    /// Modbus unit ID.
    #[serde(default)]
    pub unitid: Option<u8>,
    /// Modbus address.
    #[serde(default)]
    pub address: Option<String>,
    /// Modbus quantity.
    #[serde(default)]
    pub quantity: Option<String>,
    /// For gauge/chart/text: label, format, range, etc.
    #[serde(default)]
    pub label: Option<String>,
    /// For gauge: minimum value.
    #[serde(default)]
    pub gmin: Option<f64>,
    /// For gauge: maximum value.
    #[serde(default)]
    pub gmax: Option<f64>,
    /// For chart: points/second or similar.
    #[serde(default)]
    pub xaxis: Option<serde_json::Value>,
    /// For delay nodes: timeout in ms.
    #[serde(default)]
    pub timeout: Option<f64>,
    /// For delay nodes: timeout units (ms, s, m, h).
    #[serde(default)]
    pub timeoutUnits: Option<String>,
    /// For switch nodes: property to check.
    #[serde(default)]
    pub property: Option<String>,
    /// For switch nodes: comparison operator.
    #[serde(default)]
    pub pt: Option<String>,
    /// For switch nodes: comparison value.
    #[serde(default)]
    pub ptt: Option<String>,
    /// For trigger nodes: conditions.
    #[serde(default)]
    pub conditions: Option<serde_json::Value>,
    /// For opcsda/opcua nodes.
    #[serde(default)]
    pub endpoint: Option<String>,
    /// For OPC-UA nodes: node ID.
    #[serde(default)]
    pub item: Option<String>,
    /// For inject nodes: payload type.
    #[serde(default)]
    pub payload: Option<String>,
    /// For inject nodes: payload value.
    #[serde(default)]
    pub payloadType: Option<String>,
    /// For template nodes.
    #[serde(default)]
    pub template: Option<String>,
    /// For comment nodes.
    #[serde(default)]
    pub info: Option<String>,
    /// For link nodes: target link.
    #[serde(default)]
    pub links: Option<Vec<String>>,
}

/// Categorization of a Node-RED node for conversion purposes.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeCategory {
    /// Node type is directly convertible to a Voltrus entity.
    Convertible,
    /// Node type is known but requires manual review (e.g., function nodes).
    ManualReview,
    /// Node type is a UI/decoration node (comment, tab).
    UiDecoration,
    /// Node type is not recognized or not supported.
    Unsupported,
}

impl NodeRedNode {
    /// Classify this node into a category.
    pub fn category(&self) -> NodeCategory {
        match self.node_type.as_str() {
            // Directly convertible device/protocol nodes
            "modbus-read" | "modbus-write" | "modbus-response"
            | "modbus-flex-getter" | "modbus-flex-write"
            | "modbus-client" | "modbus-server"
            | "opcua-client" | "opcua-item"
            | "opcua-browser"
            | "s7-client" | "s7-in" | "s7-out"
            | "enip-read" | "enip-write"
            | "bacnet-client" => NodeCategory::Convertible,

            // MQTT nodes
            "mqtt in" | "mqtt-out"
            | "mqtt-broker" => NodeCategory::Convertible,

            // Flow logic nodes (map to Voltrus flow engine)
            "switch" | "change" | "delay" | "trigger"
            | "link in" | "link out" | "link-call"
            | "complete" | "catch" | "status"
            | "exec" | "template" => NodeCategory::Convertible,

            // Dashboard widget nodes
            "ui_gauge" | "ui_chart" | "ui_text"
            | "ui_button" | "ui_slider" | "ui_dropdown"
            | "ui_switch" | "ui_led" | "ui_notification"
            | "ui_toast" | "ui_table"
            | "ui_group" | "ui_tab" | "ui_spacer" => NodeCategory::Convertible,

            // Manual review required
            "function" | "python" | "java" => NodeCategory::ManualReview,

            // Inject/debug (informational, usually test nodes)
            "inject" | "debug" => NodeCategory::UiDecoration,

            // Comment/tabs
            "comment" | "tab" | "subflow" | "subflow:" => NodeCategory::UiDecoration,

            // Everything else is unsupported
            _ => NodeCategory::Unsupported,
        }
    }
}

/// Result of parsing a flows.json file.
#[derive(Debug, Clone, Serialize)]
pub struct ParseResult {
    pub total_nodes: usize,
    pub convertible: usize,
    pub manual_review: usize,
    pub ui_decoration: usize,
    pub unsupported: usize,
    pub nodes: Vec<NodeRedNode>,
    pub warnings: Vec<String>,
}

/// Parse a flows.json string into structured data.
pub fn parse_flows(json_str: &str) -> Result<ParseResult, String> {
    // Node-RED exports can be either a JSON array or a JSON object with a "nodes" key
    let flow: NodeRedFlow = if json_str.trim().starts_with('[') {
        let nodes: Vec<NodeRedNode> = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse flows.json: {}", e))?;
        NodeRedFlow { nodes }
    } else {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse flows.json: {}", e))?
    };

    let total_nodes = flow.nodes.len();
    let mut convertible = 0;
    let mut manual_review = 0;
    let mut ui_decoration = 0;
    let mut unsupported = 0;
    let mut warnings = Vec::new();

    for node in &flow.nodes {
        match node.category() {
            NodeCategory::Convertible => convertible += 1,
            NodeCategory::ManualReview => {
                manual_review += 1;
                let name = node.name.as_deref().unwrap_or(&node.id);
                warnings.push(format!(
                    "Node '{}' ({}) requires manual review — {} logic must be reimplemented",
                    name, node.id, node.node_type
                ));
            }
            NodeCategory::UiDecoration => ui_decoration += 1,
            NodeCategory::Unsupported => {
                unsupported += 1;
                let name = node.name.as_deref().unwrap_or(&node.id);
                warnings.push(format!(
                    "Node '{}' ({}) type '{}' is not supported and will be skipped",
                    name, node.id, node.node_type
                ));
            }
        }
    }

    Ok(ParseResult {
        total_nodes,
        convertible,
        manual_review,
        ui_decoration,
        unsupported,
        nodes: flow.nodes,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_modbus_flow() {
        let json = r#"[
            {"id":"n1","type":"tab","label":"Flow 1"},
            {"id":"n2","type":"modbus-read","name":"Read Voltage","server":"m1","fc":3,"unitid":1,"address":"40001","quantity":"2"},
            {"id":"n3","type":"debug","name":"Debug Output"}
        ]"#;

        let result = parse_flows(json).unwrap();
        assert_eq!(result.total_nodes, 3);
        assert_eq!(result.convertible, 1); // modbus-read
        assert_eq!(result.manual_review, 0);
        assert_eq!(result.ui_decoration, 2); // tab + debug
        assert_eq!(result.unsupported, 0);
    }

    #[test]
    fn test_parse_function_node() {
        let json = r#"[
            {"id":"n1","type":"function","name":"My Function","func":"return msg;"}
        ]"#;

        let result = parse_flows(json).unwrap();
        assert_eq!(result.manual_review, 1);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("manual review"));
    }

    #[test]
    fn test_parse_unsupported_node() {
        let json = r#"[
            {"id":"n1","type":"unknown-plugin","name":"Weird Node"}
        ]"#;

        let result = parse_flows(json).unwrap();
        assert_eq!(result.unsupported, 1);
        assert!(result.warnings[0].contains("not supported"));
    }

    #[test]
    fn test_parse_mqtt_nodes() {
        let json = r#"[
            {"id":"n1","type":"mqtt in","name":"Subscribe","topic":"sensor/#","broker":"b1"},
            {"id":"n2","type":"mqtt out","name":"Publish","topic":"cmd/valve","qos":1,"broker":"b1"}
        ]"#;

        let result = parse_flows(json).unwrap();
        assert_eq!(result.convertible, 2);
    }

    #[test]
    fn test_parse_object_format() {
        let json = r#"{
            "nodes": [
                {"id":"n1","type":"tab","label":"Flow 1"},
                {"id":"n2","type":"modbus-read","name":"Read"}
            ]
        }"#;

        let result = parse_flows(json).unwrap();
        assert_eq!(result.total_nodes, 2);
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = "not valid json";
        let result = parse_flows(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse"));
    }

    #[test]
    fn test_node_category_classification() {
        let modbus = NodeRedNode {
            id: "n1".into(), node_type: "modbus-read".into(),
            name: Some("Read".into()), z: String::new(),
            wires: Vec::new(), properties: HashMap::new(),
            func: None, server: None, topic: None, qos: None, broker: None,
            fc: None, unitid: None, address: None, quantity: None,
            label: None, gmin: None, gmax: None, xaxis: None,
            timeout: None, timeoutUnits: None, property: None, pt: None, ptt: None,
            conditions: None, endpoint: None, item: None, payload: None,
            payloadType: None, template: None, info: None, links: None,
        };
        assert_eq!(modbus.category(), NodeCategory::Convertible);

        let func = NodeRedNode {
            id: "n2".into(), node_type: "function".into(),
            name: Some("Func".into()), z: String::new(),
            wires: Vec::new(), properties: HashMap::new(),
            func: None, server: None, topic: None, qos: None, broker: None,
            fc: None, unitid: None, address: None, quantity: None,
            label: None, gmin: None, gmax: None, xaxis: None,
            timeout: None, timeoutUnits: None, property: None, pt: None, ptt: None,
            conditions: None, endpoint: None, item: None, payload: None,
            payloadType: None, template: None, info: None, links: None,
        };
        assert_eq!(func.category(), NodeCategory::ManualReview);

        let unknown = NodeRedNode {
            id: "n3".into(), node_type: "rare-custom-node".into(),
            name: None, z: String::new(),
            wires: Vec::new(), properties: HashMap::new(),
            func: None, server: None, topic: None, qos: None, broker: None,
            fc: None, unitid: None, address: None, quantity: None,
            label: None, gmin: None, gmax: None, xaxis: None,
            timeout: None, timeoutUnits: None, property: None, pt: None, ptt: None,
            conditions: None, endpoint: None, item: None, payload: None,
            payloadType: None, template: None, info: None, links: None,
        };
        assert_eq!(unknown.category(), NodeCategory::Unsupported);
    }

    #[test]
    fn test_parse_switch_and_delay_nodes() {
        let json = r#"[
            {"id":"n1","type":"switch","name":"Check","property":"payload","pt":"msg","ptt":"number"},
            {"id":"n2","type":"delay","name":"Wait","timeout":"5","timeoutUnits":"seconds"}
        ]"#;

        let result = parse_flows(json).unwrap();
        assert_eq!(result.convertible, 2);
    }

    #[test]
    fn test_parse_ui_gauge_chart_text() {
        let json = r#"[
            {"id":"n1","type":"ui_gauge","name":"Pressure","gmin":0,"gmax":100,"label":"PSI"},
            {"id":"n2","type":"ui_chart","name":"Trend"},
            {"id":"n3","type":"ui_text","name":"Status","label":"Online"}
        ]"#;

        let result = parse_flows(json).unwrap();
        assert_eq!(result.convertible, 3);
    }
}
```

- [ ] Register the module in `main.rs`:

```rust
// In src/main.rs, add to the module declarations:
mod import_nodered;
```

- [ ] Run tests:

```bash
cargo test --lib import_nodered::parser::tests
```

- [ ] **Commit:** `feat(import-nodered): add flows.json parser with node classification`

---

## Task 4 — Converter: Node-RED nodes to Voltrus entities

Implement the converter that maps parsed Node-RED nodes to Voltrus device/tag/flow/widget entities.

**Files:**
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/import_nodered/converter.rs`

**Steps:**

- [ ] Create `src/import_nodered/converter.rs`:

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::parser::{NodeCategory, NodeRedNode, ParseResult};

/// A converted Modbus device from Node-RED modbus-read nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedModbusDevice {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub unit_id: u8,
    pub interval_ms: u64,
    pub tags: Vec<ConvertedTag>,
}

/// A converted MQTT data source from Node-RED mqtt-in/out nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedMqttSource {
    pub topic: String,
    pub qos: u8,
    pub name: String,
    pub direction: String, // "in" or "out"
    pub source_node_id: String,
}

/// A converted dashboard widget from Node-RED UI nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedWidget {
    pub widget_type: String, // "gauge", "chart", "text", "button", "slider"
    pub name: String,
    pub label: Option<String>,
    pub min_val: Option<f64>,
    pub max_val: Option<f64>,
    pub source_node_id: String,
    pub format: Option<String>,
}

/// A converted flow engine node from Node-RED logic nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedFlowNode {
    pub node_type: String, // "switch", "delay", "trigger", "change"
    pub name: String,
    pub config: serde_json::Value,
    pub source_node_id: String,
}

/// A converted tag/register from a Node-RED protocol node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedTag {
    pub name: String,
    pub address: u16,
    pub data_type: String,
    pub register_area: String,
    pub scale: f64,
}

/// A function node flagged for manual review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedFunctionNode {
    pub node_id: String,
    pub name: String,
    pub source_code: Option<String>,
    pub warning: String,
}

/// Complete conversion result.
#[derive(Debug, Clone, Serialize)]
pub struct ConversionResult {
    pub modbus_devices: Vec<ConvertedModbusDevice>,
    pub mqtt_sources: Vec<ConvertedMqttSource>,
    pub widgets: Vec<ConvertedWidget>,
    pub flow_nodes: Vec<ConvertedFlowNode>,
    pub function_nodes: Vec<ConvertedFunctionNode>,
    pub skipped_nodes: Vec<SkippedNode>,
    pub warnings: Vec<String>,
}

/// A node that was skipped during conversion.
#[derive(Debug, Clone, Serialize)]
pub struct SkippedNode {
    pub node_id: String,
    pub node_type: String,
    pub name: Option<String>,
    pub reason: String,
}

/// Convert parsed Node-RED nodes into Voltrus entities.
pub fn convert(parse_result: &ParseResult) -> ConversionResult {
    let mut modbus_devices: Vec<ConvertedModbusDevice> = Vec::new();
    let mut mqtt_sources: Vec<ConvertedMqttSource> = Vec::new();
    let mut widgets: Vec<ConvertedWidget> = Vec::new();
    let mut flow_nodes: Vec<ConvertedFlowNode> = Vec::new();
    let mut function_nodes: Vec<ConvertedFunctionNode> = Vec::new();
    let mut skipped_nodes: Vec<SkippedNode> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Track unique device IDs to merge modbus-read nodes from the same server
    let mut device_map: HashMap<String, ConvertedModbusDevice> = HashMap::new();

    for node in &parse_result.nodes {
        let cat = node.category();
        if cat == NodeCategory::UiDecoration {
            continue;
        }

        let name = node.name.clone().unwrap_or_else(|| node.id.clone());

        match node.node_type.as_str() {
            // ── Modbus nodes ──
            "modbus-read" | "modbus-flex-getter" => {
                let device_id = node.server.clone().unwrap_or_else(|| format!("nr_modbus_{}", &node.id[..node.id.len().min(8)]));
                let ip = node.properties.get("host")
                    .and_then(|v| v.as_str())
                    .unwrap_or("192.168.1.1")
                    .to_string();
                let port = node.properties.get("port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(502) as u16;
                let unit_id = node.unitid.unwrap_or(1);
                let fc = node.fc.unwrap_or(3);

                // Parse address
                let addr_str = node.address.as_deref().unwrap_or("0");
                let address: u16 = addr_str.parse().unwrap_or(0);

                // Determine register area from function code
                let (register_area, data_type) = match fc {
                    1 | 2 => ("coil", "uint16"),
                    3 => ("holding", "float32"),
                    4 => ("input", "float32"),
                    _ => ("holding", "float32"),
                };

                let tag_name = name.clone();
                let quantity = node.quantity.as_deref()
                    .and_then(|q| q.parse::<u16>().ok())
                    .unwrap_or(2);

                let scale = if quantity > 1 { 1.0 } else { 1.0 };

                let tag = ConvertedTag {
                    name: tag_name,
                    address,
                    data_type: data_type.to_string(),
                    register_area: register_area.to_string(),
                    scale,
                };

                let entry = device_map.entry(device_id.clone()).or_insert_with(|| ConvertedModbusDevice {
                    device_id: device_id.clone(),
                    name: format!("NR Modbus {}", device_id),
                    ip: ip.clone(),
                    port,
                    unit_id,
                    interval_ms: 1000,
                    tags: Vec::new(),
                });
                entry.tags.push(tag);
            }

            "modbus-write" | "modbus-flex-write" => {
                let device_id = node.server.clone().unwrap_or_else(|| format!("nr_modbus_w_{}", &node.id[..node.id.len().min(8)]));
                let ip = node.properties.get("host")
                    .and_then(|v| v.as_str())
                    .unwrap_or("192.168.1.1")
                    .to_string();
                let port = node.properties.get("port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(502) as u16;
                let unit_id = node.unitid.unwrap_or(1);

                let addr_str = node.address.as_deref().unwrap_or("0");
                let address: u16 = addr_str.parse().unwrap_or(0);

                let tag = ConvertedTag {
                    name: name.clone(),
                    address,
                    data_type: "float32".to_string(),
                    register_area: "holding".to_string(),
                    scale: 1.0,
                };

                let entry = device_map.entry(device_id.clone()).or_insert_with(|| ConvertedModbusDevice {
                    device_id: device_id.clone(),
                    name: format!("NR Modbus {}", device_id),
                    ip: ip.clone(),
                    port,
                    unit_id,
                    interval_ms: 1000,
                    tags: Vec::new(),
                });
                entry.tags.push(tag);
            }

            // ── MQTT nodes ──
            "mqtt in" => {
                mqtt_sources.push(ConvertedMqttSource {
                    topic: node.topic.clone().unwrap_or_else(|| "#".to_string()),
                    qos: node.qos.unwrap_or(0),
                    name,
                    direction: "in".to_string(),
                    source_node_id: node.id.clone(),
                });
            }

            "mqtt out" => {
                mqtt_sources.push(ConvertedMqttSource {
                    topic: node.topic.clone().unwrap_or_else(|| "voltrus/out".to_string()),
                    qos: node.qos.unwrap_or(0),
                    name,
                    direction: "out".to_string(),
                    source_node_id: node.id.clone(),
                });
            }

            // ── Dashboard UI nodes ──
            "ui_gauge" => {
                widgets.push(ConvertedWidget {
                    widget_type: "gauge".to_string(),
                    name,
                    label: node.label.clone(),
                    min_val: node.gmin,
                    max_val: node.gmax,
                    source_node_id: node.id.clone(),
                    format: None,
                });
            }

            "ui_chart" => {
                widgets.push(ConvertedWidget {
                    widget_type: "chart".to_string(),
                    name,
                    label: node.label.clone(),
                    min_val: None,
                    max_val: None,
                    source_node_id: node.id.clone(),
                    format: None,
                });
            }

            "ui_text" => {
                widgets.push(ConvertedWidget {
                    widget_type: "text".to_string(),
                    name,
                    label: node.label.clone(),
                    min_val: None,
                    max_val: None,
                    source_node_id: node.id.clone(),
                    format: None,
                });
            }

            "ui_button" => {
                widgets.push(ConvertedWidget {
                    widget_type: "button".to_string(),
                    name,
                    label: node.label.clone(),
                    min_val: None,
                    max_val: None,
                    source_node_id: node.id.clone(),
                    format: None,
                });
            }

            "ui_slider" => {
                widgets.push(ConvertedWidget {
                    widget_type: "slider".to_string(),
                    name,
                    label: node.label.clone(),
                    min_val: node.gmin,
                    max_val: node.gmax,
                    source_node_id: node.id.clone(),
                    format: None,
                });
            }

            "ui_table" => {
                widgets.push(ConvertedWidget {
                    widget_type: "table".to_string(),
                    name,
                    label: node.label.clone(),
                    min_val: None,
                    max_val: None,
                    source_node_id: node.id.clone(),
                    format: None,
                });
            }

            // ── Flow logic nodes ──
            "switch" => {
                let config = serde_json::json!({
                    "condition": node.pt.as_deref().unwrap_or("msg.payload"),
                    "property": node.property.as_deref().unwrap_or("payload"),
                    "comparison": node.ptt.as_deref().unwrap_or("eq"),
                });
                flow_nodes.push(ConvertedFlowNode {
                    node_type: "switch".to_string(),
                    name,
                    config,
                    source_node_id: node.id.clone(),
                });
            }

            "delay" => {
                let timeout_ms = match node.timeoutUnits.as_deref() {
                    Some("milliseconds") => node.timeout.unwrap_or(1000.0),
                    Some("seconds") => node.timeout.unwrap_or(1.0) * 1000.0,
                    Some("minutes") => node.timeout.unwrap_or(1.0) * 60_000.0,
                    Some("hours") => node.timeout.unwrap_or(1.0) * 3_600_000.0,
                    _ => node.timeout.unwrap_or(1000.0),
                };
                let config = serde_json::json!({
                    "delay_ms": timeout_ms as u64,
                });
                flow_nodes.push(ConvertedFlowNode {
                    node_type: "delay".to_string(),
                    name,
                    config,
                    source_node_id: node.id.clone(),
                });
            }

            "trigger" => {
                let config = serde_json::json!({
                    "conditions": node.conditions,
                });
                flow_nodes.push(ConvertedFlowNode {
                    node_type: "trigger".to_string(),
                    name,
                    config,
                    source_node_id: node.id.clone(),
                });
            }

            "change" => {
                let config = serde_json::json!({
                    "rules": node.properties.get("rules").cloned().unwrap_or(serde_json::Value::Null),
                });
                flow_nodes.push(ConvertedFlowNode {
                    node_type: "change".to_string(),
                    name,
                    config,
                    source_node_id: node.id.clone(),
                });
            }

            "template" => {
                let config = serde_json::json!({
                    "template": node.template,
                    "format": node.properties.get("format").cloned().unwrap_or(serde_json::Value::String("mustache".to_string())),
                });
                flow_nodes.push(ConvertedFlowNode {
                    node_type: "template".to_string(),
                    name,
                    config,
                    source_node_id: node.id.clone(),
                });
            }

            // ── Function nodes: manual review ──
            "function" | "python" | "java" => {
                function_nodes.push(ConvertedFunctionNode {
                    node_id: node.id.clone(),
                    name: name.clone(),
                    source_code: node.func.clone(),
                    warning: format!(
                        "{} node '{}' must be reimplemented as a Voltrus flow script or Lua expression",
                        node.node_type, name
                    ),
                });
            }

            // ── OPC-UA nodes ──
            "opcua-client" | "opcua-item" => {
                warnings.push(format!(
                    "OPC-UA node '{}' detected — configure via Voltrus OPC-UA device manager for full support",
                    name
                ));
                skipped_nodes.push(SkippedNode {
                    node_id: node.id.clone(),
                    node_type: node.node_type.clone(),
                    name: Some(name),
                    reason: "OPC-UA nodes should be configured via the Voltrus OPC-UA device manager".to_string(),
                });
            }

            // ── S7 nodes ──
            "s7-client" | "s7-in" | "s7-out" => {
                warnings.push(format!(
                    "S7 node '{}' detected — configure via Voltrus S7 device manager for full support",
                    name
                ));
                skipped_nodes.push(SkippedNode {
                    node_id: node.id.clone(),
                    node_type: node.node_type.clone(),
                    name: Some(name),
                    reason: "S7 nodes should be configured via the Voltrus S7 device manager".to_string(),
                });
            }

            // ── Everything else ──
            _ => {
                if cat == NodeCategory::Unsupported {
                    skipped_nodes.push(SkippedNode {
                        node_id: node.id.clone(),
                        node_type: node.node_type.clone(),
                        name: node.name.clone(),
                        reason: format!("Unsupported node type: {}", node.node_type),
                    });
                }
            }
        }
    }

    modbus_devices = device_map.into_values().collect();

    ConversionResult {
        modbus_devices,
        mqtt_sources,
        widgets,
        flow_nodes,
        function_nodes,
        skipped_nodes,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import_nodered::parser::parse_flows;

    #[test]
    fn test_convert_modbus_read() {
        let json = r#"[
            {"id":"n1","type":"modbus-read","name":"Read Voltage","server":"plc1","fc":3,"unitid":1,"address":"40001","quantity":"2","properties":{"host":"192.168.1.10","port":502}}
        ]"#;
        let parsed = parse_flows(json).unwrap();
        let result = convert(&parsed);

        assert_eq!(result.modbus_devices.len(), 1);
        let dev = &result.modbus_devices[0];
        assert_eq!(dev.device_id, "plc1");
        assert_eq!(dev.ip, "192.168.1.10");
        assert_eq!(dev.port, 502);
        assert_eq!(dev.unit_id, 1);
        assert_eq!(dev.tags.len(), 1);
        assert_eq!(dev.tags[0].name, "Read Voltage");
        assert_eq!(dev.tags[0].address, 40001);
        assert_eq!(dev.tags[0].register_area, "holding");
    }

    #[test]
    fn test_convert_mqtt_nodes() {
        let json = r#"[
            {"id":"m1","type":"mqtt in","name":"Sensor Sub","topic":"sensor/#","qos":1,"broker":"b1"},
            {"id":"m2","type":"mqtt out","name":"Cmd Pub","topic":"cmd/valve","broker":"b1"}
        ]"#;
        let parsed = parse_flows(json).unwrap();
        let result = convert(&parsed);

        assert_eq!(result.mqtt_sources.len(), 2);
        assert_eq!(result.mqtt_sources[0].topic, "sensor/#");
        assert_eq!(result.mqtt_sources[0].direction, "in");
        assert_eq!(result.mqtt_sources[1].direction, "out");
    }

    #[test]
    fn test_convert_ui_widgets() {
        let json = r#"[
            {"id":"u1","type":"ui_gauge","name":"Pressure","label":"PSI","gmin":0,"gmax":100},
            {"id":"u2","type":"ui_chart","name":"Trend"},
            {"id":"u3","type":"ui_text","name":"Status","label":"Running"}
        ]"#;
        let parsed = parse_flows(json).unwrap();
        let result = convert(&parsed);

        assert_eq!(result.widgets.len(), 3);
        assert_eq!(result.widgets[0].widget_type, "gauge");
        assert_eq!(result.widgets[0].min_val, Some(0.0));
        assert_eq!(result.widgets[0].max_val, Some(100.0));
        assert_eq!(result.widgets[1].widget_type, "chart");
        assert_eq!(result.widgets[2].widget_type, "text");
    }

    #[test]
    fn test_convert_switch_and_delay() {
        let json = r#"[
            {"id":"s1","type":"switch","name":"Check","property":"payload","pt":"msg","ptt":"number"},
            {"id":"d1","type":"delay","name":"Wait","timeout":"5","timeoutUnits":"seconds"}
        ]"#;
        let parsed = parse_flows(json).unwrap();
        let result = convert(&parsed);

        assert_eq!(result.flow_nodes.len(), 2);
        assert_eq!(result.flow_nodes[0].node_type, "switch");
        assert_eq!(result.flow_nodes[1].node_type, "delay");
        // 5 seconds = 5000ms
        assert_eq!(result.flow_nodes[1].config["delay_ms"], 5000);
    }

    #[test]
    fn test_convert_function_node() {
        let json = r#"[
            {"id":"f1","type":"function","name":"Calc","func":"return {payload: msg.payload * 2};"}
        ]"#;
        let parsed = parse_flows(json).unwrap();
        let result = convert(&parsed);

        assert_eq!(result.function_nodes.len(), 1);
        assert_eq!(result.function_nodes[0].source_code.as_deref(), Some("return {payload: msg.payload * 2};"));
    }

    #[test]
    fn test_convert_opcua_skipped() {
        let json = r#"[
            {"id":"o1","type":"opcua-item","name":"Temp","endpoint":"opc.tcp://host:4840","item":"ns=2;s=Temp"}
        ]"#;
        let parsed = parse_flows(json).unwrap();
        let result = convert(&parsed);

        assert_eq!(result.skipped_nodes.len(), 1);
        assert!(result.skipped_nodes[0].reason.contains("OPC-UA"));
    }

    #[test]
    fn test_convert_merge_modbus_same_server() {
        let json = r#"[
            {"id":"n1","type":"modbus-read","name":"Voltage","server":"plc1","fc":3,"unitid":1,"address":"40001","quantity":"2","properties":{"host":"10.0.0.1"}},
            {"id":"n2","type":"modbus-read","name":"Current","server":"plc1","fc":3,"unitid":1,"address":"40003","quantity":"2","properties":{"host":"10.0.0.1"}}
        ]"#;
        let parsed = parse_flows(json).unwrap();
        let result = convert(&parsed);

        assert_eq!(result.modbus_devices.len(), 1);
        assert_eq!(result.modbus_devices[0].tags.len(), 2);
    }

    #[test]
    fn test_convert_empty_flow() {
        let json = r#"[]"#;
        let parsed = parse_flows(json).unwrap();
        let result = convert(&parsed);

        assert_eq!(result.modbus_devices.len(), 0);
        assert_eq!(result.mqtt_sources.len(), 0);
        assert_eq!(result.widgets.len(), 0);
        assert_eq!(result.flow_nodes.len(), 0);
    }
}
```

- [ ] Run tests:

```bash
cargo test --lib import_nodered::converter::tests
```

- [ ] **Commit:** `feat(import-nodered): add converter from Node-RED nodes to Voltrus entities`

---

## Task 5 — Import API endpoint

Create the API endpoint that accepts a flows.json file upload and returns the conversion result.

**Files:**
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/api/import_nodered.rs`
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/api/mod.rs`

**Steps:**

- [ ] Create `src/api/import_nodered.rs`:

```rust
use axum::extract::{Extension, Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::api::{AppError, AppState, AuthUser};
use crate::import_nodered::{parser, converter};

/// POST /api/v1/import/node-red
///
/// Accepts a flows.json file upload and returns the conversion preview.
/// Does NOT actually create entities — the preview is returned for the frontend
/// to display, then a separate confirm step creates entities.
pub async fn import_node_red(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // Find the file field in the multipart upload
    let mut json_content = None;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError(anyhow::anyhow!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" || name == "flows" {
            json_content = Some(
                field.text().await
                    .map_err(|e| AppError(anyhow::anyhow!("Failed to read file: {}", e)))?
            );
            break;
        }
    }

    let json_str = json_content
        .ok_or_else(|| AppError(anyhow::anyhow!("No 'file' field found in upload. Send flows.json as multipart field named 'file'.")))?;

    // Parse the flows.json
    let parse_result = parser::parse_flows(&json_str)
        .map_err(|e| AppError(anyhow::anyhow!("{}", e)))?;

    // Convert to Voltrus entities
    let conversion = converter::convert(&parse_result);

    // Check license tier for device limits
    let device_limit = state.license_tier.max_devices();
    let total_new_devices = conversion.modbus_devices.len() as u32;
    let current_device_count = state.storage.count_modbus_devices().await.unwrap_or(0);

    let mut warnings = conversion.warnings.clone();
    if let Some(limit) = device_limit {
        if current_device_count + total_new_devices > limit {
            warnings.push(format!(
                "Import would create {} Modbus device(s), but your {} tier has a limit of {}. {} device(s) already exist.",
                total_new_devices,
                serde_json::to_string(&state.license_tier).unwrap_or_else(|_| "unknown".to_string()),
                limit,
                current_device_count
            ));
        }
    }

    Ok((
        StatusCode::OK,
        axum::Json(json!({
            "parse": {
                "total_nodes": parse_result.total_nodes,
                "convertible": parse_result.convertible,
                "manual_review": parse_result.manual_review,
                "ui_decoration": parse_result.ui_decoration,
                "unsupported": parse_result.unsupported,
            },
            "conversion": {
                "modbus_devices": conversion.modbus_devices.len(),
                "mqtt_sources": conversion.mqtt_sources.len(),
                "widgets": conversion.widgets.len(),
                "flow_nodes": conversion.flow_nodes.len(),
                "function_nodes": conversion.function_nodes.len(),
                "skipped_nodes": conversion.skipped_nodes.len(),
            },
            "details": {
                "modbus_devices": conversion.modbus_devices,
                "mqtt_sources": conversion.mqtt_sources,
                "widgets": conversion.widgets,
                "flow_nodes": conversion.flow_nodes,
                "function_nodes": conversion.function_nodes,
                "skipped_nodes": conversion.skipped_nodes,
            },
            "warnings": warnings,
            "license": {
                "current_devices": current_device_count,
                "new_devices": total_new_devices,
                "limit": device_limit,
            },
        })),
    ))
}
```

- [ ] Register the module and route in `src/api/mod.rs`:

```rust
// Add to module declarations at the top of src/api/mod.rs:
mod import_nodered;

// Add to admin_protected router (after config_snapshots routes):
.route("/api/v1/import/node-red", post(import_nodered::import_node_red))
```

- [ ] Write the integration test in `src/api/tests/mod.rs`:

```rust
// Add to the module declarations in src/api/tests/mod.rs:
mod import_nodered;
```

- [ ] Create `src/api/tests/import_nodered.rs`:

```rust
use super::*;
use axum::body::Body;
use http::Request;

#[tokio::test]
async fn test_import_node_red_basic() {
    let app = test_app_with_seeded_admin().await;

    let form = multipart::body::MultipartBody::new()
        .part("file", multipart::body::Part::text(r#"[
            {"id":"n1","type":"modbus-read","name":"Read Voltage","server":"plc1","fc":3,"unitid":1,"address":"40001","quantity":"2","properties":{"host":"192.168.1.10","port":502}},
            {"id":"n2","type":"mqtt in","name":"Sensor","topic":"sensor/#","qos":0},
            {"id":"n3","type":"ui_gauge","name":"Pressure","gmin":0,"gmax":100},
            {"id":"n4","type":"comment","name":"My Comment"},
            {"id":"n5","type":"unknown-node","name":"Weird"}
        ]"#).mime_str("application/json").unwrap())
        .to_body();

    // Use a raw multipart request since the multipart crate may not be available in test context
    let boundary = "----TestBoundary123";
    let json_body = r#"[
        {"id":"n1","type":"modbus-read","name":"Read Voltage","server":"plc1","fc":3,"unitid":1,"address":"40001","quantity":"2","properties":{"host":"192.168.1.10","port":502}},
        {"id":"n2","type":"mqtt in","name":"Sensor","topic":"sensor/#","qos":0},
        {"id":"n3","type":"ui_gauge","name":"Pressure","gmin":0,"gmax":100},
        {"id":"n4","type":"comment","name":"My Comment"},
        {"id":"n5","type":"unknown-node","name":"Weird"}
    ]"#;

    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"flows.json\"\r\nContent-Type: application/json\r\n\r\n{}\r\n--{}--\r\n",
        boundary, json_body, boundary
    );

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import/node-red")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}
```

- [ ] Run tests:

```bash
cargo test --lib api::tests::import_nodered
```

- [ ] **Commit:** `feat(api): add POST /api/v1/import/node-red endpoint for flows.json import`

---

## Task 6 — MQTT broker module: config, auth, lifecycle

Create the embedded MQTT broker module using rumqttd.

**Files:**
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/mqtt_broker/mod.rs`
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/mqtt_broker/config.rs`
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/mqtt_broker/auth.rs`
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/main.rs`

**Steps:**

- [ ] Create `src/mqtt_broker/mod.rs`:

```rust
pub mod config;
pub mod auth;

use std::sync::Arc;
use tokio::sync::{watch, RwLock};

use crate::models::MqttBrokerConfig;

/// Runtime status of the embedded MQTT broker.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BrokerStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

/// Connected client info.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectedClient {
    pub client_id: String,
    pub connected_at: i64,
    pub subscriptions: Vec<String>,
}

/// Shared broker state accessible from API handlers.
#[derive(Clone)]
pub struct MqttBrokerState {
    pub status: Arc<watch::Sender<BrokerStatus>>,
    pub status_rx: watch::Receiver<BrokerStatus>,
    pub config: Arc<RwLock<Option<MqttBrokerConfig>>>,
    pub clients: Arc<RwLock<Vec<ConnectedClient>>>,
    /// Handle to the spawned broker task for shutdown.
    pub shutdown_tx: Arc<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl MqttBrokerState {
    pub fn new() -> Self {
        let (status_tx, status_rx) = watch::channel(BrokerStatus::Stopped);
        Self {
            status: Arc::new(status_tx),
            status_rx,
            config: Arc::new(RwLock::new(None)),
            clients: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    pub async fn current_status(&self) -> BrokerStatus {
        self.status_rx.borrow().clone()
    }
}

impl Default for MqttBrokerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the embedded MQTT broker as a background task.
pub async fn start_broker(
    state: MqttBrokerState,
    config: MqttBrokerConfig,
) -> Result<(), String> {
    // Check if already running
    if state.current_status().await == BrokerStatus::Running {
        return Err("Broker is already running".to_string());
    }

    // Update status
    let _ = state.status.send(BrokerStatus::Starting);

    // Store config
    {
        let mut cfg = state.config.write().await;
        *cfg = Some(config.clone());
    }

    // Build rumqttd config
    let broker_config = config::build_rumqttd_config(&config);

    let port = config.port;
    let status_tx = state.status.clone();
    let clients = state.clients.clone();

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    {
        let mut handle = state.shutdown_tx.lock().await;
        *handle = Some(shutdown_tx);
    }

    // Spawn the broker in a blocking task since rumqttd uses std::thread
    let clients_arc = clients.clone();
    let status_tx_clone = status_tx.clone();
    tokio::task::spawn_blocking(move || {
        let mut broker = match rumqttd::Broker::new(broker_config) {
            Ok(b) => b,
            Err(e) => {
                let _ = status_tx_clone.send(BrokerStatus::Error(format!("Failed to create broker: {}", e)));
                return;
            }
        };

        // Create a local link to monitor connections
        let (_link_tx, mut link_rx) = match broker.link("voltrus-monitor") {
            Ok(l) => l,
            Err(e) => {
                let _ = status_tx_clone.send(BrokerStatus::Error(format!("Failed to create monitor link: {}", e)));
                return;
            }
        };

        // Start the broker (this blocks)
        let broker_handle = std::thread::spawn(move || {
            if let Err(e) = broker.start() {
                tracing::error!("MQTT broker error: {}", e);
                let _ = status_tx_clone.send(BrokerStatus::Error(format!("Broker runtime error: {}", e)));
            }
        });

        // Monitor shutdown signal
        let _ = shutdown_rx.blocking_recv();

        tracing::info!("MQTT broker shutting down on port {}", port);
        let _ = status_tx.send(BrokerStatus::Stopped);
        // Note: broker thread will be cleaned up when broker is dropped
        drop(broker_handle);
    });

    // Give it a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let _ = status_tx.send(BrokerStatus::Running);
    tracing::info!("MQTT broker started on port {}", port);

    Ok(())
}

/// Stop the embedded MQTT broker.
pub async fn stop_broker(state: MqttBrokerState) -> Result<(), String> {
    if state.current_status().await != BrokerStatus::Running {
        return Err("Broker is not running".to_string());
    }

    // Send shutdown signal
    let mut handle = state.shutdown_tx.lock().await;
    if let Some(tx) = handle.take() {
        let _ = tx.send(());
    }

    // Clear clients
    {
        let mut clients = state.clients.write().await;
        clients.clear();
    }

    let _ = state.status.send(BrokerStatus::Stopped);
    tracing::info!("MQTT broker stopped");

    Ok(())
}
```

- [ ] Create `src/mqtt_broker/config.rs`:

```rust
use crate::models::MqttBrokerConfig;

/// Build a rumqttd Config from our MqttBrokerConfig.
pub fn build_rumqttd_config(config: &MqttBrokerConfig) -> rumqttd::Config {
    let mut rumqttd_config = rumqttd::Config::new();

    // Configure the router
    {
        let router_config = rumqttd::Config::new().router;
        rumqttd_config.router = rumqttd::RouterConfig {
            max_connections: config.max_connections,
            ..router_config
        };
    }

    // Configure the v4/v5 listener
    let server_id = format!("voltrus-mqtt-{}", config.port);
    let listener = rumqttd::ServerConfig {
        name: server_id,
        listen: format!("0.0.0.0:{}", config.port),
        next_connection_delay_ms: 1,
        connections: rumqttd::ConnectionSettings {
            connection_timeout_ms: 60_000,
            max_payload_size: 268_435_456, // 256 MB
            max_inflight_count: 1000,
            dynamic_filters: true,
            auth: None, // Auth handled at application level if needed
            ..Default::default()
        },
        tls: None,
    };

    rumqttd_config.v4 = Some(std::sync::Arc::new(listener));
    rumqttd_config.id = 0;

    rumqttd_config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_default() {
        let config = MqttBrokerConfig {
            enabled: true,
            port: 1883,
            max_connections: 500,
            auth: None,
        };
        let _rumqttd = build_rumqttd_config(&config);
        // We just verify it doesn't panic
    }

    #[test]
    fn test_build_config_custom_port() {
        let config = MqttBrokerConfig {
            enabled: true,
            port: 8883,
            max_connections: 100,
            auth: None,
        };
        let _rumqttd = build_rumqttd_config(&config);
    }
}
```

- [ ] Create `src/mqtt_broker/auth.rs`:

```rust
use crate::models::MqttAuthConfig;

/// Validate MQTT credentials against the configured auth settings.
/// Returns Ok(()) if auth is disabled or credentials match, Err otherwise.
pub fn validate_credentials(
    auth_config: &Option<MqttAuthConfig>,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), String> {
    let auth = match auth_config {
        Some(a) if a.enabled => a,
        _ => return Ok(()), // Auth disabled
    };

    let username = username.ok_or_else(|| "Username required when auth is enabled".to_string())?;
    let password = password.ok_or_else(|| "Password required when auth is enabled".to_string())?;

    let user = auth.users.iter().find(|u| u.username == username)
        .ok_or_else(|| format!("Unknown MQTT user: {}", username))?;

    if user.password != password {
        return Err("Invalid MQTT password".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MqttAuthConfig, MqttUserEntry};

    #[test]
    fn test_auth_disabled() {
        let result = validate_credentials(&None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_enabled_valid() {
        let config = MqttAuthConfig {
            enabled: true,
            users: vec![MqttUserEntry {
                username: "admin".to_string(),
                password: "secret".to_string(),
            }],
        };
        let result = validate_credentials(&Some(config), Some("admin"), Some("secret"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_enabled_invalid_password() {
        let config = MqttAuthConfig {
            enabled: true,
            users: vec![MqttUserEntry {
                username: "admin".to_string(),
                password: "secret".to_string(),
            }],
        };
        let result = validate_credentials(&Some(config), Some("admin"), Some("wrong"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid MQTT password"));
    }

    #[test]
    fn test_auth_enabled_unknown_user() {
        let config = MqttAuthConfig {
            enabled: true,
            users: vec![MqttUserEntry {
                username: "admin".to_string(),
                password: "secret".to_string(),
            }],
        };
        let result = validate_credentials(&Some(config), Some("unknown"), Some("pass"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown MQTT user"));
    }

    #[test]
    fn test_auth_enabled_missing_credentials() {
        let config = MqttAuthConfig {
            enabled: true,
            users: vec![],
        };
        let result = validate_credentials(&Some(config), None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Username required"));
    }
}
```

- [ ] Register the module in `main.rs`:

```rust
// In src/main.rs, add to module declarations:
mod mqtt_broker;
```

- [ ] Run tests:

```bash
cargo test --lib mqtt_broker
```

- [ ] **Commit:** `feat(mqtt-broker): add embedded MQTT broker module with rumqttd`

---

## Task 7 — MQTT broker API endpoints

Create the REST API endpoints for managing the MQTT broker.

**Files:**
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/api/mqtt_broker.rs`
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/api/mod.rs`

**Steps:**

- [ ] Create `src/api/mqtt_broker.rs`:

```rust
use axum::extract::{Extension, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::api::{AppError, AppState, AuthUser};
use crate::mqtt_broker::{self, BrokerStatus, MqttBrokerState};

/// GET /api/v1/mqtt-broker/status
///
/// Returns the current status of the embedded MQTT broker.
pub async fn get_status(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.mqtt_broker() {
        return Err(AppError(anyhow::anyhow!(
            "Embedded MQTT broker requires Starter tier. Upgrade at https://voltrus.id"
        )));
    }

    let broker = state.mqtt_broker_state.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("MQTT broker module not initialized")))?;

    let status = broker.current_status().await;
    let clients = broker.clients.read().await;

    Ok(Json(json!({
        "status": match &status {
            BrokerStatus::Running => "running",
            BrokerStatus::Stopped => "stopped",
            BrokerStatus::Starting => "starting",
            BrokerStatus::Error(msg) => format!("error: {}", msg),
        },
        "connected_clients": clients.len(),
        "clients": clients.as_slice(),
        "config": match broker.config.read().await.as_ref() {
            Some(c) => json!({
                "port": c.port,
                "max_connections": c.max_connections,
                "auth_enabled": c.auth.as_ref().map(|a| a.enabled).unwrap_or(false),
            }),
            None => serde_json::Value::Null,
        },
    })))
}

use axum::response::IntoResponse;

/// PUT /api/v1/mqtt-broker/config
///
/// Update the MQTT broker configuration (port, max_connections, auth).
/// The broker must be stopped to change configuration.
#[derive(Debug, Deserialize)]
pub struct UpdateMqttConfigRequest {
    pub port: Option<u16>,
    pub max_connections: Option<usize>,
    pub auth: Option<MqttAuthUpdate>,
}

#[derive(Debug, Deserialize)]
pub struct MqttAuthUpdate {
    pub enabled: Option<bool>,
    pub users: Option<Vec<MqttUserUpdate>>,
}

#[derive(Debug, Deserialize)]
pub struct MqttUserUpdate {
    pub username: String,
    pub password: String,
}

pub async fn update_config(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Json(req): Json<UpdateMqttConfigRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.mqtt_broker() {
        return Err(AppError(anyhow::anyhow!(
            "Embedded MQTT broker requires Starter tier. Upgrade at https://voltrus.id"
        )));
    }

    let broker = state.mqtt_broker_state.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("MQTT broker module not initialized")))?;

    // Must be stopped to change config
    if broker.current_status().await == BrokerStatus::Running {
        return Err(AppError(anyhow::anyhow!(
            "Stop the broker before changing configuration"
        )));
    }

    // Update stored config
    {
        let mut cfg_guard = broker.config.write().await;
        let mut cfg = cfg_guard.clone().unwrap_or(crate::models::MqttBrokerConfig {
            enabled: true,
            port: 1883,
            max_connections: 1000,
            auth: None,
        });

        if let Some(port) = req.port {
            if port == 0 {
                return Err(AppError(anyhow::anyhow!("Port must be > 0")));
            }
            cfg.port = port;
        }
        if let Some(max_conn) = req.max_connections {
            if max_conn == 0 {
                return Err(AppError(anyhow::anyhow!("max_connections must be > 0")));
            }
            cfg.max_connections = max_conn;
        }
        if let Some(auth_update) = req.auth {
            let mut auth = cfg.auth.unwrap_or(crate::models::MqttAuthConfig {
                enabled: false,
                users: Vec::new(),
            });
            if let Some(enabled) = auth_update.enabled {
                auth.enabled = enabled;
            }
            if let Some(users) = auth_update.users {
                auth.users = users.into_iter().map(|u| crate::models::MqttUserEntry {
                    username: u.username,
                    password: u.password,
                }).collect();
            }
            cfg.auth = Some(auth);
        }

        cfg.enabled = true;
        *cfg_guard = Some(cfg);
    }

    // Also update the config.yaml on disk
    if let Err(e) = persist_mqtt_config(&state, &broker).await {
        tracing::warn!("Failed to persist MQTT config to disk: {}", e);
    }

    let cfg = broker.config.read().await;
    Ok(Json(json!({
        "message": "Configuration updated. Start the broker to apply changes.",
        "config": match cfg.as_ref() {
            Some(c) => json!({
                "port": c.port,
                "max_connections": c.max_connections,
                "auth_enabled": c.auth.as_ref().map(|a| a.enabled).unwrap_or(false),
            }),
            None => serde_json::Value::Null,
        },
    })))
}

/// POST /api/v1/mqtt-broker/start
///
/// Start the embedded MQTT broker.
pub async fn start_broker(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.mqtt_broker() {
        return Err(AppError(anyhow::anyhow!(
            "Embedded MQTT broker requires Starter tier. Upgrade at https://voltrus.id"
        )));
    }

    let broker = state.mqtt_broker_state.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("MQTT broker module not initialized")))?;

    let config = broker.config.read().await.clone()
        .ok_or_else(|| AppError(anyhow::anyhow!(
            "No MQTT broker configuration. Set config via PUT /api/v1/mqtt-broker/config first."
        )))?;

    mqtt_broker::start_broker(broker.clone(), config).await
        .map_err(|e| AppError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(json!({
        "message": "MQTT broker started",
        "status": "running",
    })))
}

/// POST /api/v1/mqtt-broker/stop
///
/// Stop the embedded MQTT broker.
pub async fn stop_broker(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.mqtt_broker() {
        return Err(AppError(anyhow::anyhow!(
            "Embedded MQTT broker requires Starter tier. Upgrade at https://voltrus.id"
        )));
    }

    let broker = state.mqtt_broker_state.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("MQTT broker module not initialized")))?;

    mqtt_broker::stop_broker(broker.clone()).await
        .map_err(|e| AppError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(json!({
        "message": "MQTT broker stopped",
        "status": "stopped",
    })))
}

/// Persist the MQTT broker config to the config.yaml file.
/// This is best-effort; if it fails, the in-memory config is still valid.
async fn persist_mqtt_config(
    _state: &AppState,
    broker: &MqttBrokerState,
) -> anyhow::Result<()> {
    let cfg = broker.config.read().await;
    let mqtt = cfg.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No MQTT config to persist"))?;

    // Read existing config.yaml
    let config_path = "config.yaml";
    let existing = std::fs::read_to_string(config_path)
        .unwrap_or_else(|_| "devices: []\n".to_string());

    // Parse as YAML value
    let mut doc: serde_yaml_neo::Value = serde_yaml_neo::from_str(&existing)
        .map_err(|e| anyhow::anyhow!("Failed to parse config.yaml: {}", e))?;

    // Build MQTT section
    let mut mqtt_section = serde_yaml_neo::Mapping::new();
    mqtt_section.insert(
        serde_yaml_neo::Value::String("enabled".to_string()),
        serde_yaml_neo::Value::Bool(mqtt.enabled),
    );
    mqtt_section.insert(
        serde_yaml_neo::Value::String("port".to_string()),
        serde_yaml_neo::Value::Number(serde_yaml_neo::Number::from(mqtt.port as i64)),
    );
    mqtt_section.insert(
        serde_yaml_neo::Value::String("max_connections".to_string()),
        serde_yaml_neo::Value::Number(serde_yaml_neo::Number::from(mqtt.max_connections as i64)),
    );

    if let Some(ref auth) = mqtt.auth {
        let mut auth_section = serde_yaml_neo::Mapping::new();
        auth_section.insert(
            serde_yaml_neo::Value::String("enabled".to_string()),
            serde_yaml_neo::Value::Bool(auth.enabled),
        );
        let users: Vec<serde_yaml_neo::Value> = auth.users.iter().map(|u| {
            let mut user_map = serde_yaml_neo::Mapping::new();
            user_map.insert(
                serde_yaml_neo::Value::String("username".to_string()),
                serde_yaml_neo::Value::String(u.username.clone()),
            );
            user_map.insert(
                serde_yaml_neo::Value::String("password".to_string()),
                serde_yaml_neo::Value::String(u.password.clone()),
            );
            serde_yaml_neo::Value::Mapping(user_map)
        }).collect();
        auth_section.insert(
            serde_yaml_neo::Value::String("users".to_string()),
            serde_yaml_neo::Value::Sequence(users),
        );
        mqtt_section.insert(
            serde_yaml_neo::Value::String("auth".to_string()),
            serde_yaml_neo::Value::Mapping(auth_section),
        );
    }

    // Update or insert the mqtt_broker key
    if let serde_yaml_neo::Value::Mapping(ref mut mapping) = doc {
        mapping.insert(
            serde_yaml_neo::Value::String("mqtt_broker".to_string()),
            serde_yaml_neo::Value::Mapping(mqtt_section),
        );
    }

    let new_yaml = serde_yaml_neo::to_string(&doc)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

    std::fs::write(config_path, new_yaml)
        .map_err(|e| anyhow::anyhow!("Failed to write config.yaml: {}", e))?;

    Ok(())
}
```

- [ ] Register the module and routes in `src/api/mod.rs`:

```rust
// Add to module declarations at the top of src/api/mod.rs:
mod mqtt_broker;

// Add to protected router (after modbus-devices/status):
.route("/api/v1/mqtt-broker/status", get(mqtt_broker::get_status))

// Add to admin_protected router (after the import route):
.route("/api/v1/mqtt-broker/config", put(mqtt_broker::update_config))
.route("/api/v1/mqtt-broker/start", post(mqtt_broker::start_broker))
.route("/api/v1/mqtt-broker/stop", post(mqtt_broker::stop_broker))
```

- [ ] Write the test file `src/api/tests/mqtt_broker.rs`:

```rust
use super::*;

#[tokio::test]
async fn test_mqtt_broker_status_requires_auth() {
    let app = test_app();
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/mqtt-broker/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect or 401 since no auth cookie
    assert!(!resp.status().is_success());
}

#[tokio::test]
async fn test_mqtt_broker_start_requires_auth() {
    let app = test_app();
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/mqtt-broker/start")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(!resp.status().is_success());
}
```

- [ ] Add the test module declaration in `src/api/tests/mod.rs`:

```rust
mod mqtt_broker;
```

- [ ] Run tests:

```bash
cargo test --lib api::tests::mqtt_broker
```

- [ ] **Commit:** `feat(api): add MQTT broker REST API endpoints (status, config, start, stop)`

---

## Task 8 — MQTT broker integration in main.rs

Wire the MQTT broker into the application startup and AppState.

**Files:**
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/api/mod.rs`
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/main.rs`

**Steps:**

- [ ] Add `mqtt_broker_state` to `AppState` in `src/api/mod.rs`:

```rust
// In the AppState struct, after pub ws_manager:
pub mqtt_broker_state: Option<crate::mqtt_broker::MqttBrokerState>,
```

- [ ] Initialize the MQTT broker state and auto-start in `main.rs`.

Add this block after the flow engine initialization block and before the hub manager block:

```rust
// Initialize MQTT broker (Starter tier and above)
let mqtt_broker_state = {
    if license_tier.mqtt_broker() {
        let broker_state = crate::mqtt_broker::MqttBrokerState::new();

        // Auto-start if configured
        if let Some(ref mqtt_cfg) = app_config.mqtt_broker {
            if mqtt_cfg.enabled {
                // Store config first
                {
                    let mut cfg = broker_state.config.write().await;
                    *cfg = Some(mqtt_cfg.clone());
                }

                match crate::mqtt_broker::start_broker(broker_state.clone(), mqtt_cfg.clone()).await {
                    Ok(()) => {
                        tracing::info!("Embedded MQTT broker auto-started on port {}", mqtt_cfg.port);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to auto-start MQTT broker: {}", e);
                    }
                }
            }
        }

        Some(broker_state)
    } else {
        None
    }
};
```

- [ ] Add `mqtt_broker_state` to the `AppState` construction in `main.rs`:

```rust
// In the state = AppState { ... } block, after ws_manager:
mqtt_broker_state,
```

- [ ] Update the `test_app()` and `test_app_with_seeded_admin()` helpers in `src/api/tests/helpers.rs`:

```rust
// In both test_app() and test_app_with_seeded_admin(), add to AppState construction:
mqtt_broker_state: None,
```

- [ ] Also update `test_app_with_license_status()`:

```rust
// Add to AppState construction:
mqtt_broker_state: None,
```

- [ ] Run all tests to verify nothing is broken:

```bash
cargo test --lib
```

- [ ] **Commit:** `feat(mqtt-broker): integrate MQTT broker into main.rs startup and AppState`

---

## Task 9 — License tier: mqtt_broker flag

Add the `mqtt_broker()` method to the license tier for gating.

**Files:**
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/src/license/tier.rs`

**Steps:**

- [ ] Add the `mqtt_broker` method to `LicenseTier`:

```rust
/// Embedded MQTT broker.
/// Starter tier and above.
pub fn mqtt_broker(&self) -> bool {
    !matches!(self, LicenseTier::Trial)
}
```

- [ ] Add the feature flag to `to_feature_flags()`:

```rust
// In the to_feature_flags() method, add to the json! object:
"mqtt_broker": self.mqtt_broker(),
```

- [ ] Write tests in `src/license/tier.rs`:

```rust
#[test]
fn test_mqtt_broker_tier_gating() {
    assert!(!LicenseTier::Trial.mqtt_broker());
    assert!(LicenseTier::Starter.mqtt_broker());
    assert!(LicenseTier::Professional.mqtt_broker());
    assert!(LicenseTier::Enterprise.mqtt_broker());
}
```

- [ ] Run tests:

```bash
cargo test --lib license::tier::tests
```

- [ ] **Commit:** `feat(license): add mqtt_broker tier gating (Starter+)`

---

## Task 10 — Frontend: query hooks + keys

Add the TanStack Query hooks and query keys for the new features.

**Files:**
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/client/src/queries/keys.ts`
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/client/src/queries/import_nodered.ts`
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/client/src/queries/mqtt_broker.ts`
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/client/src/queries/index.ts`

**Steps:**

- [ ] Add query keys to `keys.ts`:

```typescript
// Add after the last entry (shiftAutoReport):
mqttBroker:       ['mqttBroker'] as const,
```

- [ ] Create `client/src/queries/import_nodered.ts`:

```typescript
import { useMutation } from '@tanstack/react-query'
import { fetchOk } from './fetch'

export interface NodeRedParseResult {
  total_nodes: number
  convertible: number
  manual_review: number
  ui_decoration: number
  unsupported: number
}

export interface NodeRedConversionSummary {
  modbus_devices: number
  mqtt_sources: number
  widgets: number
  flow_nodes: number
  function_nodes: number
  skipped_nodes: number
}

export interface ConvertedModbusDevice {
  device_id: string
  name: string
  ip: string
  port: number
  unit_id: number
  interval_ms: number
  tags: ConvertedTag[]
}

export interface ConvertedTag {
  name: string
  address: number
  data_type: string
  register_area: string
  scale: number
}

export interface ConvertedMqttSource {
  topic: string
  qos: number
  name: string
  direction: string
  source_node_id: string
}

export interface ConvertedWidget {
  widget_type: string
  name: string
  label: string | null
  min_val: number | null
  max_val: number | null
  source_node_id: string
}

export interface ConvertedFlowNode {
  node_type: string
  name: string
  config: Record<string, any>
  source_node_id: string
}

export interface ConvertedFunctionNode {
  node_id: string
  name: string
  source_code: string | null
  warning: string
}

export interface SkippedNode {
  node_id: string
  node_type: string
  name: string | null
  reason: string
}

export interface NodeRedImportResult {
  parse: NodeRedParseResult
  conversion: NodeRedConversionSummary
  details: {
    modbus_devices: ConvertedModbusDevice[]
    mqtt_sources: ConvertedMqttSource[]
    widgets: ConvertedWidget[]
    flow_nodes: ConvertedFlowNode[]
    function_nodes: ConvertedFunctionNode[]
    skipped_nodes: SkippedNode[]
  }
  warnings: string[]
  license: {
    current_devices: number
    new_devices: number
    limit: number | null
  }
}

export function useImportNodeRed() {
  return useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append('file', file)
      const r = await fetchOk('/api/v1/import/node-red', {
        method: 'POST',
        body: formData,
      })
      return r.json() as Promise<NodeRedImportResult>
    },
  })
}
```

- [ ] Create `client/src/queries/mqtt_broker.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { queryKeys } from './keys'
import { fetchJson, fetchOk } from './fetch'

export interface MqttBrokerStatus {
  status: string
  connected_clients: number
  clients: MqttClient[]
  config: {
    port: number
    max_connections: number
    auth_enabled: boolean
  } | null
}

export interface MqttClient {
  client_id: string
  connected_at: number
  subscriptions: string[]
}

export interface MqttConfigUpdate {
  port?: number
  max_connections?: number
  auth?: {
    enabled?: boolean
    users?: { username: string; password: string }[]
  }
}

export function useMqttBrokerStatus(enabled = true) {
  return useQuery({
    queryKey: queryKeys.mqttBroker,
    queryFn: () => fetchJson<MqttBrokerStatus>('/api/v1/mqtt-broker/status'),
    enabled,
    refetchInterval: 5000,
  })
}

export function useUpdateMqttConfig() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (config: MqttConfigUpdate) => {
      const r = await fetchOk('/api/v1/mqtt-broker/config', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.mqttBroker })
    },
  })
}

export function useStartMqttBroker() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      const r = await fetchOk('/api/v1/mqtt-broker/start', { method: 'POST' })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.mqttBroker })
    },
  })
}

export function useStopMqttBroker() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      const r = await fetchOk('/api/v1/mqtt-broker/stop', { method: 'POST' })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.mqttBroker })
    },
  })
}
```

- [ ] Export from `client/src/queries/index.ts`:

```typescript
// Add to the re-exports:
export { useImportNodeRed } from './import_nodered'
export type {
  NodeRedImportResult, NodeRedParseResult, NodeRedConversionSummary,
  ConvertedModbusDevice, ConvertedTag, ConvertedMqttSource,
  ConvertedWidget, ConvertedFlowNode, ConvertedFunctionNode, SkippedNode,
} from './import_nodered'

export {
  useMqttBrokerStatus, useUpdateMqttConfig, useStartMqttBroker, useStopMqttBroker
} from './mqtt_broker'
export type { MqttBrokerStatus, MqttClient, MqttConfigUpdate } from './mqtt_broker'
```

- [ ] **Commit:** `feat(frontend): add query hooks for Node-RED import and MQTT broker`

---

## Task 11 — Frontend: NodeRedImportWizard component

Create the multi-step import wizard component.

**Files:**
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/client/src/NodeRedImportWizard.tsx`

**Steps:**

- [ ] Create `client/src/NodeRedImportWizard.tsx`:

```tsx
import { useState, useRef, useCallback } from 'react'
import { useImportNodeRed, type NodeRedImportResult } from './queries'
import { btnStyle, inputStyle } from './styles'

type Step = 'upload' | 'preview' | 'report'

interface Props {
  onClose: () => void
  embedded?: boolean
}

export function NodeRedImportWizard({ onClose, embedded }: Props) {
  const [step, setStep] = useState<Step>('upload')
  const [result, setResult] = useState<NodeRedImportResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const importMut = useImportNodeRed()

  const handleFileSelect = useCallback(() => {
    fileInputRef.current?.click()
  }, [])

  const handleFileChange = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    setError(null)
    try {
      const data = await importMut.mutateAsync(file)
      setResult(data)
      setStep('preview')
    } catch (err: any) {
      setError(err.message || 'Failed to parse flows.json')
    }
  }, [importMut])

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault()
    const file = e.dataTransfer.files[0]
    if (!file) return

    setError(null)
    try {
      const data = await importMut.mutateAsync(file)
      setResult(data)
      setStep('preview')
    } catch (err: any) {
      setError(err.message || 'Failed to parse flows.json')
    }
  }, [importMut])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
  }, [])

  const containerStyle: React.CSSProperties = {
    background: 'var(--bg)',
    border: '1px solid var(--border)',
    borderRadius: 12,
    padding: embedded ? 16 : 32,
    maxWidth: 800,
    margin: '0 auto',
  }

  const headerStyle: React.CSSProperties = {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    marginBottom: 24,
  }

  const titleStyle: React.CSSProperties = {
    fontSize: 18, fontWeight: 600, color: 'var(--fg)',
  }

  const thStyle: React.CSSProperties = {
    textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)',
    textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600,
    borderBottom: '1px solid var(--border)',
  }

  const tdStyle: React.CSSProperties = {
    padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)',
    color: 'var(--fg)',
  }

  return (
    <div style={containerStyle} onKeyDown={e => { if (e.key === 'Escape' && !embedded) onClose() }}>
      {/* Header */}
      <div style={headerStyle}>
        <span style={titleStyle}>
          {step === 'upload' && 'Node-RED Import'}
          {step === 'preview' && 'Import Preview'}
          {step === 'report' && 'Import Summary'}
        </span>
        {!embedded && (
          <button style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)' }} onClick={onClose}>
            Close
          </button>
        )}
      </div>

      {/* Step indicators */}
      <div style={{ display: 'flex', gap: 8, marginBottom: 24 }}>
        {(['upload', 'preview', 'report'] as Step[]).map((s, i) => (
          <div key={s} style={{
            flex: 1, height: 4, borderRadius: 2,
            background: step === s ? 'var(--green)' :
                       ['upload', 'preview', 'report'].indexOf(step) > i ? 'var(--green)' : 'var(--border)',
          }} />
        ))}
      </div>

      {/* Step: Upload */}
      {step === 'upload' && (
        <div>
          <p style={{ color: 'var(--muted)', fontSize: 13, marginBottom: 16 }}>
            Upload your Node-RED flows.json export file. The importer will analyze the nodes and
            show a preview of what will be created in Voltrus.
          </p>

          <div
            onDrop={handleDrop}
            onDragOver={handleDragOver}
            onClick={handleFileSelect}
            style={{
              border: '2px dashed var(--border)', borderRadius: 8, padding: '48px 24px',
              textAlign: 'center', cursor: 'pointer', marginBottom: 16,
              transition: 'border-color 0.2s',
            }}
          >
            <div style={{ fontSize: 32, marginBottom: 12, color: 'var(--muted)' }}>&#128196;</div>
            <p style={{ color: 'var(--fg)', fontSize: 14, marginBottom: 4 }}>
              Drop your flows.json file here, or click to browse
            </p>
            <p style={{ color: 'var(--muted)', fontSize: 12 }}>
              Supports Node-RED v2.x and v3.x exports
            </p>
          </div>

          <input
            ref={fileInputRef}
            type="file"
            accept=".json"
            onChange={handleFileChange}
            style={{ display: 'none' }}
          />

          {error && (
            <div style={{ background: 'rgba(239,68,68,0.1)', border: '1px solid var(--crit)', borderRadius: 6, padding: 12, marginBottom: 16 }}>
              <p style={{ color: 'var(--crit)', fontSize: 13 }}>{error}</p>
            </div>
          )}

          {importMut.isPending && (
            <div style={{ textAlign: 'center', padding: 16 }}>
              <p style={{ color: 'var(--muted)', fontSize: 13 }}>Analyzing flows.json...</p>
            </div>
          )}
        </div>
      )}

      {/* Step: Preview */}
      {step === 'preview' && result && (
        <div>
          {/* Parse summary */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gap: 12, marginBottom: 24 }}>
            {[
              { label: 'Total', value: result.parse.total_nodes, color: 'var(--fg)' },
              { label: 'Convertible', value: result.parse.convertible, color: 'var(--green)' },
              { label: 'Manual Review', value: result.parse.manual_review, color: 'var(--warn)' },
              { label: 'Decoration', value: result.parse.ui_decoration, color: 'var(--muted)' },
              { label: 'Unsupported', value: result.parse.unsupported, color: 'var(--crit)' },
            ].map(item => (
              <div key={item.label} style={{
                background: 'var(--bg)', border: '1px solid var(--border)',
                borderRadius: 8, padding: 12, textAlign: 'center',
              }}>
                <div style={{ fontSize: 24, fontWeight: 700, color: item.color }}>{item.value}</div>
                <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>{item.label}</div>
              </div>
            ))}
          </div>

          {/* Conversion details */}
          <h3 style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 12 }}>
            Conversion Preview
          </h3>

          <table style={{ width: '100%', borderCollapse: 'collapse', marginBottom: 16 }}>
            <thead>
              <tr>
                <th style={thStyle}>Entity</th>
                <th style={thStyle}>Count</th>
                <th style={thStyle}>Details</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td style={tdStyle}>Modbus Devices</td>
                <td style={tdStyle}>{result.conversion.modbus_devices}</td>
                <td style={{ ...tdStyle, fontSize: 12, color: 'var(--muted)' }}>
                  {result.details.modbus_devices.map(d => `${d.name} (${d.ip}:${d.port}, ${d.tags.length} tags)`).join(', ') || '-'}
                </td>
              </tr>
              <tr>
                <td style={tdStyle}>MQTT Sources</td>
                <td style={tdStyle}>{result.conversion.mqtt_sources}</td>
                <td style={{ ...tdStyle, fontSize: 12, color: 'var(--muted)' }}>
                  {result.details.mqtt_sources.map(s => `${s.direction.toUpperCase()} ${s.topic}`).join(', ') || '-'}
                </td>
              </tr>
              <tr>
                <td style={tdStyle}>Dashboard Widgets</td>
                <td style={tdStyle}>{result.conversion.widgets}</td>
                <td style={{ ...tdStyle, fontSize: 12, color: 'var(--muted)' }}>
                  {result.details.widgets.map(w => `${w.widget_type}: ${w.name}`).join(', ') || '-'}
                </td>
              </tr>
              <tr>
                <td style={tdStyle}>Flow Nodes</td>
                <td style={tdStyle}>{result.conversion.flow_nodes}</td>
                <td style={{ ...tdStyle, fontSize: 12, color: 'var(--muted)' }}>
                  {result.details.flow_nodes.map(n => `${n.node_type}: ${n.name}`).join(', ') || '-'}
                </td>
              </tr>
              <tr>
                <td style={tdStyle}>Function Nodes (Manual)</td>
                <td style={tdStyle}>{result.conversion.function_nodes}</td>
                <td style={{ ...tdStyle, fontSize: 12, color: 'var(--warn)' }}>
                  {result.details.function_nodes.map(f => f.name).join(', ') || '-'}
                </td>
              </tr>
              <tr>
                <td style={tdStyle}>Skipped</td>
                <td style={tdStyle}>{result.conversion.skipped_nodes}</td>
                <td style={{ ...tdStyle, fontSize: 12, color: 'var(--muted)' }}>
                  {result.details.skipped_nodes.map(s => `${s.node_type}: ${s.name || s.node_id}`).join(', ') || '-'}
                </td>
              </tr>
            </tbody>
          </table>

          {/* Warnings */}
          {result.warnings.length > 0 && (
            <div style={{ marginBottom: 24 }}>
              <h3 style={{ fontSize: 14, fontWeight: 600, color: 'var(--warn)', marginBottom: 8 }}>
                Warnings ({result.warnings.length})
              </h3>
              <div style={{ maxHeight: 200, overflowY: 'auto', background: 'rgba(245,158,11,0.05)', border: '1px solid var(--warn)', borderRadius: 6, padding: 12 }}>
                {result.warnings.map((w, i) => (
                  <p key={i} style={{ fontSize: 12, color: 'var(--fg)', marginBottom: 4 }}>
                    <span style={{ color: 'var(--warn)', marginRight: 6 }}>{'\u26A0'}</span>
                    {w}
                  </p>
                ))}
              </div>
            </div>
          )}

          {/* License check */}
          {result.license.limit !== null && (
            <div style={{
              background: result.license.current_devices + result.license.new_devices > (result.license.limit ?? 0)
                ? 'rgba(239,68,68,0.1)' : 'rgba(34,197,94,0.1)',
              border: `1px solid ${result.license.current_devices + result.license.new_devices > (result.license.limit ?? 0) ? 'var(--crit)' : 'var(--green)'}`,
              borderRadius: 6, padding: 12, marginBottom: 24,
            }}>
              <p style={{ fontSize: 13, color: 'var(--fg)' }}>
                Device limit: {result.license.current_devices} existing + {result.license.new_devices} new = {result.license.current_devices + result.license.new_devices} / {result.license.limit}
              </p>
            </div>
          )}

          {/* Actions */}
          <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
            <button style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)' }} onClick={() => { setStep('upload'); setResult(null) }}>
              Back
            </button>
            <button
              style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }}
              onClick={() => setStep('report')}
            >
              Continue to Import
            </button>
          </div>
        </div>
      )}

      {/* Step: Report */}
      {step === 'report' && result && (
        <div style={{ textAlign: 'center', padding: '32px 0' }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>{'\u2705'}</div>
          <h2 style={{ fontSize: 20, fontWeight: 600, color: 'var(--fg)', marginBottom: 8 }}>
            Import Preview Complete
          </h2>
          <p style={{ color: 'var(--muted)', fontSize: 13, marginBottom: 24 }}>
            Review the analysis above. Entities can be created individually via the respective device managers.
          </p>

          <div style={{ display: 'flex', gap: 8, justifyContent: 'center', flexWrap: 'wrap' }}>
            {result.conversion.modbus_devices > 0 && (
              <div style={{ background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 8, padding: '12px 20px' }}>
                <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--green)' }}>{result.conversion.modbus_devices}</div>
                <div style={{ fontSize: 11, color: 'var(--muted)' }}>Modbus Device(s)</div>
              </div>
            )}
            {result.conversion.mqtt_sources > 0 && (
              <div style={{ background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 8, padding: '12px 20px' }}>
                <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--green)' }}>{result.conversion.mqtt_sources}</div>
                <div style={{ fontSize: 11, color: 'var(--muted)' }}>MQTT Source(s)</div>
              </div>
            )}
            {result.conversion.widgets > 0 && (
              <div style={{ background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 8, padding: '12px 20px' }}>
                <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--green)' }}>{result.conversion.widgets}</div>
                <div style={{ fontSize: 11, color: 'var(--muted)' }}>Widget(s)</div>
              </div>
            )}
          </div>

          <div style={{ marginTop: 24 }}>
            <button style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)' }} onClick={onClose}>
              Done
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Commit:** `feat(frontend): add NodeRedImportWizard component with upload/preview/report steps`

---

## Task 12 — Frontend: MqttBrokerPanel component

Create the MQTT broker management panel.

**Files:**
- Create: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/client/src/MqttBrokerPanel.tsx`

**Steps:**

- [ ] Create `client/src/MqttBrokerPanel.tsx`:

```tsx
import { useState, useEffect, Fragment } from 'react'
import { useMqttBrokerStatus, useStartMqttBroker, useStopMqttBroker, useUpdateMqttConfig } from './queries'
import { btnStyle, inputStyle } from './styles'
import type { MqttBrokerStatus as BrokerStatus } from './queries'

interface Props {
  onClose: () => void
  embedded?: boolean
}

export function MqttBrokerPanel({ onClose, embedded }: Props) {
  const { data: status, isLoading, error } = useMqttBrokerStatus(true)
  const startMut = useStartMqttBroker()
  const stopMut = useStopMqttBroker()
  const updateMut = useUpdateMqttConfig()

  const [editPort, setEditPort] = useState('')
  const [editMaxConn, setEditMaxConn] = useState('')
  const [editAuthEnabled, setEditAuthEnabled] = useState(false)
  const [editUsers, setEditUsers] = useState<{ username: string; password: string }[]>([])
  const [showConfig, setShowConfig] = useState(false)

  useEffect(() => {
    if (embedded) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose, embedded])

  // Populate edit fields when status loads
  useEffect(() => {
    if (status?.config) {
      setEditPort(String(status.config.port))
      setEditMaxConn(String(status.config.max_connections))
      setEditAuthEnabled(status.config.auth_enabled)
    }
  }, [status?.config])

  const isRunning = status?.status === 'running'
  const isStarting = status?.status === 'starting'

  const handleToggle = () => {
    if (isRunning) {
      stopMut.mutate()
    } else {
      startMut.mutate()
    }
  }

  const handleSaveConfig = () => {
    updateMut.mutate({
      port: editPort ? parseInt(editPort, 10) : undefined,
      max_connections: editMaxConn ? parseInt(editMaxConn, 10) : undefined,
      auth: {
        enabled: editAuthEnabled,
        users: editAuthEnabled ? editUsers : undefined,
      },
    }, {
      onSuccess: () => setShowConfig(false),
    })
  }

  const handleAddUser = () => {
    setEditUsers([...editUsers, { username: '', password: '' }])
  }

  const handleRemoveUser = (index: number) => {
    setEditUsers(editUsers.filter((_, i) => i !== index))
  }

  const handleUpdateUser = (index: number, field: 'username' | 'password', value: string) => {
    const updated = [...editUsers]
    updated[index] = { ...updated[index], [field]: value }
    setEditUsers(updated)
  }

  const containerStyle: React.CSSProperties = {
    background: 'var(--bg)', border: '1px solid var(--border)',
    borderRadius: 12, padding: embedded ? 16 : 32, maxWidth: 700, margin: '0 auto',
  }

  const headerStyle: React.CSSProperties = {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 24,
  }

  const titleStyle: React.CSSProperties = {
    fontSize: 18, fontWeight: 600, color: 'var(--fg)',
  }

  const thStyle: React.CSSProperties = {
    textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)',
    textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600,
    borderBottom: '1px solid var(--border)',
  }

  const tdStyle: React.CSSProperties = {
    padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)', color: 'var(--fg)',
  }

  if (isLoading) {
    return (
      <div style={containerStyle}>
        <div style={{ textAlign: 'center', padding: 32, color: 'var(--muted)' }}>Loading...</div>
      </div>
    )
  }

  if (error) {
    return (
      <div style={containerStyle}>
        <div style={headerStyle}>
          <span style={titleStyle}>MQTT Broker</span>
          {!embedded && <button style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)' }} onClick={onClose}>Close</button>}
        </div>
        <div style={{ background: 'rgba(239,68,68,0.1)', border: '1px solid var(--crit)', borderRadius: 6, padding: 12 }}>
          <p style={{ color: 'var(--crit)', fontSize: 13 }}>{error.message || 'Failed to load MQTT broker status'}</p>
        </div>
      </div>
    )
  }

  return (
    <div style={containerStyle}>
      {/* Header */}
      <div style={headerStyle}>
        <span style={titleStyle}>MQTT Broker</span>
        {!embedded && <button style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)' }} onClick={onClose}>Close</button>}
      </div>

      {/* Status card */}
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        background: 'var(--bg)', border: '1px solid var(--border)',
        borderRadius: 8, padding: '16px 20px', marginBottom: 24,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div style={{
            width: 12, height: 12, borderRadius: '50%',
            background: isRunning ? 'var(--green)' : isStarting ? 'var(--warn)' : 'var(--muted)',
            boxShadow: isRunning ? '0 0 8px var(--green)' : 'none',
          }} />
          <div>
            <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)' }}>
              {isRunning ? 'Running' : isStarting ? 'Starting...' : 'Stopped'}
            </div>
            <div style={{ fontSize: 12, color: 'var(--muted)' }}>
              {status?.config ? `Port ${status.config.port}` : 'Not configured'}
              {isRunning && status ? ` | ${status.connected_clients} client${status.connected_clients !== 1 ? 's' : ''} connected` : ''}
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', gap: 8 }}>
          <button
            style={{
              ...btnStyle,
              borderColor: isRunning ? 'var(--crit)' : 'var(--green)',
              color: isRunning ? 'var(--crit)' : 'var(--green)',
            }}
            onClick={handleToggle}
            disabled={startMut.isPending || stopMut.isPending || isStarting}
          >
            {startMut.isPending || stopMut.isPending ? '...' : isRunning ? 'Stop' : 'Start'}
          </button>
          <button
            style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)' }}
            onClick={() => setShowConfig(!showConfig)}
            disabled={isRunning}
          >
            {showConfig ? 'Cancel' : 'Configure'}
          </button>
        </div>
      </div>

      {/* Configuration panel */}
      {showConfig && (
        <div style={{ marginBottom: 24, border: '1px solid var(--border)', borderRadius: 8, padding: 16 }}>
          <h3 style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 16 }}>Broker Configuration</h3>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, marginBottom: 16 }}>
            <div>
              <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', display: 'block', marginBottom: 4 }}>
                Port
              </label>
              <input
                style={inputStyle}
                type="number"
                value={editPort}
                onChange={e => setEditPort(e.target.value)}
                placeholder="1883"
              />
            </div>
            <div>
              <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', display: 'block', marginBottom: 4 }}>
                Max Connections
              </label>
              <input
                style={inputStyle}
                type="number"
                value={editMaxConn}
                onChange={e => setEditMaxConn(e.target.value)}
                placeholder="1000"
              />
            </div>
          </div>

          {/* Auth section */}
          <div style={{ marginBottom: 16 }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer', marginBottom: 8 }}>
              <input
                type="checkbox"
                checked={editAuthEnabled}
                onChange={e => setEditAuthEnabled(e.target.checked)}
              />
              <span style={{ fontSize: 13, color: 'var(--fg)' }}>Enable Authentication</span>
            </label>

            {editAuthEnabled && (
              <div>
                <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                  <thead>
                    <tr>
                      <th style={thStyle}>Username</th>
                      <th style={thStyle}>Password</th>
                      <th style={{ ...thStyle, width: 40 }}></th>
                    </tr>
                  </thead>
                  <tbody>
                    {editUsers.map((user, i) => (
                      <tr key={i}>
                        <td style={tdStyle}>
                          <input
                            style={{ ...inputStyle, width: '100%' }}
                            value={user.username}
                            onChange={e => handleUpdateUser(i, 'username', e.target.value)}
                            placeholder="username"
                          />
                        </td>
                        <td style={tdStyle}>
                          <input
                            style={{ ...inputStyle, width: '100%' }}
                            type="password"
                            value={user.password}
                            onChange={e => handleUpdateUser(i, 'password', e.target.value)}
                            placeholder="password"
                          />
                        </td>
                        <td style={tdStyle}>
                          <button
                            style={{ background: 'none', border: 'none', color: 'var(--crit)', cursor: 'pointer', fontSize: 16 }}
                            onClick={() => handleRemoveUser(i)}
                          >
                            &times;
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <button
                  style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)', fontSize: 12, marginTop: 8 }}
                  onClick={handleAddUser}
                >
                  + Add User
                </button>
              </div>
            )}
          </div>

          <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
            <button
              style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }}
              onClick={handleSaveConfig}
              disabled={updateMut.isPending}
            >
              {updateMut.isPending ? 'Saving...' : 'Save Configuration'}
            </button>
          </div>

          {updateMut.isError && (
            <div style={{ marginTop: 12, background: 'rgba(239,68,68,0.1)', border: '1px solid var(--crit)', borderRadius: 6, padding: 12 }}>
              <p style={{ color: 'var(--crit)', fontSize: 13 }}>{updateMut.error.message}</p>
            </div>
          )}
        </div>
      )}

      {/* Connected clients table */}
      {isRunning && status && status.clients.length > 0 && (
        <div>
          <h3 style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 8 }}>
            Connected Clients ({status.clients.length})
          </h3>
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr>
                  <th style={thStyle}>Client ID</th>
                  <th style={thStyle}>Connected At</th>
                  <th style={thStyle}>Subscriptions</th>
                </tr>
              </thead>
              <tbody>
                {status.clients.map((client, i) => (
                  <tr key={i}>
                    <td style={tdStyle}>
                      <code style={{ fontFamily: 'monospace', fontSize: 12 }}>{client.client_id}</code>
                    </td>
                    <td style={{ ...tdStyle, fontSize: 12, color: 'var(--muted)' }}>
                      {new Date(client.connected_at * 1000).toLocaleString()}
                    </td>
                    <td style={tdStyle}>
                      {client.subscriptions.length > 0 ? (
                        <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                          {client.subscriptions.map((sub, j) => (
                            <span key={j} style={{
                              background: 'var(--bg)', border: '1px solid var(--border)',
                              borderRadius: 4, padding: '2px 6px', fontSize: 11, fontFamily: 'monospace',
                            }}>
                              {sub}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span style={{ color: 'var(--muted)', fontSize: 12 }}>None</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {isRunning && status && status.clients.length === 0 && (
        <p style={{ textAlign: 'center', color: 'var(--muted)', fontSize: 13, padding: 16 }}>
          No clients connected. Devices can connect to port {status.config?.port}.
        </p>
      )}

      {/* Action feedback */}
      {(startMut.isError || stopMut.isError) && (
        <div style={{ marginTop: 16, background: 'rgba(239,68,68,0.1)', border: '1px solid var(--crit)', borderRadius: 6, padding: 12 }}>
          <p style={{ color: 'var(--crit)', fontSize: 13 }}>
            {startMut.error?.message || stopMut.error?.message}
          </p>
        </div>
      )}

      {(startMut.isSuccess || stopMut.isSuccess) && (
        <div style={{ marginTop: 16, background: 'rgba(34,197,94,0.1)', border: '1px solid var(--green)', borderRadius: 6, padding: 12 }}>
          <p style={{ color: 'var(--green)', fontSize: 13 }}>
            {startMut.isSuccess ? 'Broker started successfully' : 'Broker stopped successfully'}
          </p>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Commit:** `feat(frontend): add MqttBrokerPanel component with status, config, and client table`

---

## Task 13 — Frontend: App.tsx integration

Wire the new components into the main App.

**Files:**
- Modify: `/Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus/client/src/App.tsx`

**Steps:**

- [ ] Add imports at the top of App.tsx:

```typescript
// Add to imports section:
import { NodeRedImportWizard } from './NodeRedImportWizard'
import { MqttBrokerPanel } from './MqttBrokerPanel'
```

- [ ] Add state variables for the new panels. In the `AppInner` function, after the existing `showShiftHandover` state:

```typescript
const [showNodeRedImport, setShowNodeRedImport] = useState(false)
const [showMqttBroker, setShowMqttBroker] = useState(false)
```

- [ ] Add the panel buttons to the admin toolbar/settings area. Look for where other panel toggles are rendered (like `showAnnotationPanel`, `showShiftHandover`, etc.) and add:

```tsx
<button
  style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)' }}
  onClick={() => setShowNodeRedImport(s => !s)}
  aria-label="Node-RED Import"
  title="Node-RED Import"
>
  Import
</button>

<button
  style={{ ...btnStyle, borderColor: 'var(--muted)', color: 'var(--muted)' }}
  onClick={() => setShowMqttBroker(s => !s)}
  aria-label="MQTT Broker"
  title="MQTT Broker"
>
  MQTT
</button>
```

- [ ] Add the panel rendering blocks, after the existing panel renders (like the `showAnnotationPanel` block):

```tsx
{showNodeRedImport && (
  <div style={{
    position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)',
    display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000,
  }}>
    <div style={{ maxHeight: '90vh', overflowY: 'auto', width: '90%', maxWidth: 900 }}>
      <NodeRedImportWizard onClose={() => setShowNodeRedImport(false)} />
    </div>
  </div>
)}

{showMqttBroker && (
  <div style={{
    position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)',
    display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000,
  }}>
    <div style={{ maxHeight: '90vh', overflowY: 'auto', width: '90%', maxWidth: 750 }}>
      <MqttBrokerPanel onClose={() => setShowMqttBroker(false)} />
    </div>
  </div>
)}
```

- [ ] Run the frontend build to check for errors:

```bash
cd client && npx tsc --noEmit 2>&1 | head -30
```

- [ ] **Commit:** `feat(frontend): integrate NodeRedImportWizard and MqttBrokerPanel into App.tsx`

---

## Task 14 — Final integration tests and cleanup

Run all tests and verify the full stack works together.

**Files:**
- All modified files

**Steps:**

- [ ] Run all Rust tests:

```bash
cargo test --lib 2>&1
```

- [ ] Run frontend type check:

```bash
cd client && npx tsc --noEmit
```

- [ ] Verify cargo clippy passes:

```bash
cargo clippy --lib 2>&1 | head -50
```

- [ ] **Commit:** `chore: v0.32.0 track 3 final integration pass`

---

## Summary

This plan adds two independent systems to Voltrus v0.32.0:

### Node-RED Importer
- **Backend:** `src/import_nodered/` module with parser (flows.json deserialization + node classification) and converter (maps Node-RED node types to Voltrus Modbus devices, MQTT sources, dashboard widgets, flow engine nodes)
- **API:** `POST /api/v1/import/node-red` (multipart file upload, admin-only)
- **Frontend:** `NodeRedImportWizard.tsx` — three-step wizard (Upload > Preview with warnings > Summary)

### Embedded MQTT Broker
- **Backend:** `src/mqtt_broker/` module using rumqttd in-process, with config/auth/lifecycle management
- **API:** `GET /api/v1/mqtt-broker/status` (protected), `PUT /api/v1/mqtt-broker/config`, `POST start/stop` (admin-only, Starter+ tier gate)
- **Config:** `mqtt_broker` section in config.yaml (enabled, port, max_connections, auth with users)
- **Frontend:** `MqttBrokerPanel.tsx` — status indicator, start/stop toggle, configuration form with auth management, connected clients table

### License Gating
- MQTT broker: Starter tier and above (`mqtt_broker()` method on `LicenseTier`)
- Node-RED import: Available to all authenticated admin users (no tier gate, since it's a convenience tool)

### Key Patterns Followed
- TDD: failing tests written before implementation
- `Storage` pattern with `tokio::task::spawn_blocking`
- `AppState` extension for new state
- API handler pattern: `State(state), Extension(user), Json(body)`
- Query keys in `keys.ts`, hooks in `queries/*.ts`, `fetchJson`/`fetchOk` helpers
- Inline styles with CSS variables (`var(--fg)`, `var(--bg)`, `var(--border)`, etc.)
- `btnStyle`/`inputStyle` from `styles.ts`
