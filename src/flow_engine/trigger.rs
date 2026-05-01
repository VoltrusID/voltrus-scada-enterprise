use serde::{Deserialize, Serialize};
use super::message::Message;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum TriggerConfig {
    #[serde(rename = "modbus")]
    Modbus {
        device_id: String,
        metric: Option<String>,
        #[serde(default)]
        debounce_ms: Option<u64>,
        #[serde(default)]
        on_change_only: Option<bool>,
    },
    #[serde(rename = "timer")]
    Timer {
        interval_ms: u64,
    },
    #[serde(rename = "mqtt")]
    Mqtt {
        topic: String,
        #[serde(default)]
        debounce_ms: Option<u64>,
    },
}

impl TriggerConfig {
    pub fn matches(&self, msg: &Message) -> bool {
        match self {
            TriggerConfig::Modbus { device_id, metric, .. } => {
                if msg.metadata.get("source") != Some(&"modbus".to_string()) {
                    return false;
                }
                if device_id != "*" {
                    let payload_device_id = msg.payload.get("device_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if payload_device_id != device_id {
                        return false;
                    }
                }
                if let Some(metric_name) = metric {
                    if let Some(metrics) = msg.payload.get("metrics") {
                        if metrics.get(metric_name).is_none() {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            TriggerConfig::Timer { .. } => false,
            TriggerConfig::Mqtt { .. } => false,
        }
    }
}
