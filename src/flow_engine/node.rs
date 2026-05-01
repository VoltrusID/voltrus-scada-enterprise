use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cache::DeviceCache;
use crate::storage::Storage;
use super::message::Message;
use super::state::FlowState;

#[derive(Debug, Clone)]
pub enum NodeResult {
    Continue(Message),
    FilterReject,
}

#[derive(Clone)]
pub struct NodeContext {
    pub flow_id: String,
    pub node_id: String,
    pub state: FlowState,
    pub storage: Storage,
    pub cache: DeviceCache,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThresholdRule {
    pub gt: Option<f64>,
    pub gte: Option<f64>,
    pub lt: Option<f64>,
    pub lte: Option<f64>,
    pub eq: Option<f64>,
    #[serde(default)]
    pub set: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum NodeConfig {
    #[serde(rename = "filter")]
    Filter { id: String, condition: String },
    #[serde(rename = "transform")]
    Transform { id: String, #[serde(default)] map: HashMap<String, String> },
    #[serde(rename = "threshold")]
    Threshold { id: String, field: String, rules: Vec<ThresholdRule> },
    #[serde(rename = "lookup")]
    Lookup { id: String, field: String, output_field: String, #[serde(default)] table: HashMap<String, Value>, default: Value },
    #[serde(rename = "expression")]
    Expression { id: String, output_field: String, expr: String },
    #[serde(rename = "data_source_out")]
    DataSourceOut { id: String, value_field: String, source_key: String, name: String, unit: String, decimals: i64 },
    #[serde(rename = "http_out")]
    HttpOut { id: String, url: String, #[serde(default)] headers: HashMap<String, String>, timeout_ms: u64, retry_count: u64 },
    #[serde(rename = "alarm_out")]
    AlarmOut { id: String, severity: String, source_key: String, message: String },
}

impl NodeConfig {
    pub fn process(&self, mut msg: Message, ctx: &NodeContext) -> Result<NodeResult, String> {
        match self {
            NodeConfig::Filter { condition, .. } => {
                let result = crate::expression::evaluate_json(condition, &msg.payload)?;
                let truthy = is_truthy(&result);
                if truthy {
                    Ok(NodeResult::Continue(msg))
                } else {
                    Ok(NodeResult::FilterReject)
                }
            }
            NodeConfig::Transform { map, .. } => {
                let mut new_payload = serde_json::json!({});
                for (path, expr) in map {
                    let value = crate::expression::evaluate_json(expr, &msg.payload)?;
                    set_json_path(&mut new_payload, path, value);
                }
                msg.payload = new_payload;
                Ok(NodeResult::Continue(msg))
            }
            NodeConfig::Threshold { field, rules, .. } => {
                let field_value = get_json_path(&msg.payload, field)?;
                let field_num = to_f64(&field_value)?;
                for rule in rules {
                    if rule_matches(field_num, rule) {
                        for (path, value) in &rule.set {
                            set_json_path(&mut msg.payload, path, value.clone());
                        }
                        return Ok(NodeResult::Continue(msg));
                    }
                }
                Ok(NodeResult::Continue(msg))
            }
            NodeConfig::Lookup { field, output_field, table, default, .. } => {
                let field_value = get_json_path(&msg.payload, field)?;
                let lookup_key = match &field_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    _ => return Err("Unsupported lookup key type".to_string()),
                };
                let result = table.get(&lookup_key).cloned().unwrap_or_else(|| default.clone());
                set_json_path(&mut msg.payload, output_field, result);
                Ok(NodeResult::Continue(msg))
            }
            NodeConfig::Expression { output_field, expr, .. } => {
                let result = crate::expression::evaluate_json(expr, &msg.payload)?;
                set_json_path(&mut msg.payload, output_field, result);
                Ok(NodeResult::Continue(msg))
            }
            NodeConfig::DataSourceOut { value_field, source_key, .. } => {
                let value = get_json_path(&msg.payload, value_field)?;
                let numeric_value = to_f64(&value)?;
                let device_id = format!("flow_{}", ctx.flow_id);
                let mut metrics = HashMap::new();
                metrics.insert(source_key.clone(), numeric_value);
                let cache = ctx.cache.clone();
                tokio::runtime::Handle::current().spawn(async move {
                    cache.update(&device_id, metrics).await;
                });
                Ok(NodeResult::Continue(msg))
            }
            NodeConfig::HttpOut { url, headers, timeout_ms, retry_count, .. } => {
                let payload = msg.payload.clone();
                let url = url.clone();
                let headers = headers.clone();
                let timeout_ms = *timeout_ms;
                let retry_count = *retry_count;
                tokio::runtime::Handle::current().spawn(async move {
                    let client = reqwest::Client::new();
                    for attempt in 0..=retry_count {
                        let mut request = client.post(&url).json(&payload);
                        for (k, v) in &headers {
                            request = request.header(k, v);
                        }
                        request = request.timeout(std::time::Duration::from_millis(timeout_ms));
                        match request.send().await {
                            Ok(resp) if resp.status().is_success() => break,
                            _ => {
                                if attempt < retry_count {
                                    tokio::time::sleep(std::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                                }
                            }
                        }
                    }
                });
                Ok(NodeResult::Continue(msg))
            }
            NodeConfig::AlarmOut { severity, source_key, message, .. } => {
                let storage = ctx.storage.clone();
                let source_key = source_key.clone();
                let severity = severity.clone();
                let message = message.clone();
                tokio::runtime::Handle::current().spawn(async move {
                    let _ = storage.create_alarm(&source_key, &severity, 0.0, 0.0, &message).await;
                });
                Ok(NodeResult::Continue(msg))
            }
        }
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|v| v != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Null => false,
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn get_json_path(value: &Value, path: &str) -> Result<Value, String> {
    let mut current = value;
    for segment in path.split('.') {
        match current {
            Value::Object(map) => {
                current = map.get(segment)
                    .ok_or_else(|| format!("Path segment '{}' not found", segment))?;
            }
            _ => return Err(format!("Cannot resolve path segment '{}' in non-object", segment)),
        }
    }
    Ok(current.clone())
}

fn set_json_path(value: &mut Value, path: &str, new_value: Value) {
    let segments: Vec<&str> = path.split('.').collect();
    if segments.is_empty() {
        *value = new_value;
        return;
    }
    let mut current = value;
    for i in 0..segments.len() - 1 {
        let segment = segments[i];
        if !current.is_object() {
            *current = serde_json::json!({});
        }
        let map = current.as_object_mut().unwrap();
        if !map.contains_key(segment) {
            map.insert(segment.to_string(), serde_json::json!({}));
        }
        current = map.get_mut(segment).unwrap();
    }
    let last = segments.last().unwrap();
    if !current.is_object() {
        *current = serde_json::json!({});
    }
    current.as_object_mut().unwrap().insert(last.to_string(), new_value);
}

fn to_f64(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) => n.as_f64().ok_or_else(|| "Invalid number".to_string()),
        Value::String(s) => s.parse().map_err(|_| format!("Cannot convert string '{}' to number", s)),
        _ => Err(format!("Cannot convert {:?} to number", value)),
    }
}

fn rule_matches(value: f64, rule: &ThresholdRule) -> bool {
    if let Some(gt) = rule.gt {
        if value <= gt {
            return false;
        }
    }
    if let Some(gte) = rule.gte {
        if value < gte {
            return false;
        }
    }
    if let Some(lt) = rule.lt {
        if value >= lt {
            return false;
        }
    }
    if let Some(lte) = rule.lte {
        if value > lte {
            return false;
        }
    }
    if let Some(eq) = rule.eq {
        if (value - eq).abs() > f64::EPSILON {
            return false;
        }
    }
    true
}
