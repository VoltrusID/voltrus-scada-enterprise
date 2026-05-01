use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Message {
    pub payload: Value,
    pub metadata: HashMap<String, String>,
}

impl Message {
    pub fn new(payload: Value) -> Self {
        Self { payload, metadata: HashMap::new() }
    }
}
