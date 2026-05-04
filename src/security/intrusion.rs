use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SUSPICIOUS_401_THRESHOLD: u32 = 10;
const SUSPICIOUS_WINDOW_SECS: u64 = 60;

/// Tracks suspicious activity (e.g., repeated 401s) per IP for intrusion detection.
#[derive(Clone)]
pub struct IntrusionDetector {
    suspicious_401s: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl IntrusionDetector {
    pub fn new() -> Self {
        Self {
            suspicious_401s: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record_401(&self, ip: &str) {
        let mut map = self.suspicious_401s.lock().unwrap();
        let now = Instant::now();
        let key = ip.to_string();
        let entries = map.entry(key).or_default();
        entries.retain(|t| now.duration_since(*t) < Duration::from_secs(SUSPICIOUS_WINDOW_SECS));
        entries.push(now);

        if entries.len() >= SUSPICIOUS_401_THRESHOLD as usize {
            tracing::warn!("SECURITY ALERT: {} failed auth attempts from IP {} in {}s", entries.len(), ip, SUSPICIOUS_WINDOW_SECS);
        }
    }

    /// Record a 401 and persist the event to the security log database.
    pub async fn record_401_with_storage(&self, ip: &str, storage: &crate::storage::Storage) {
        self.record_401(ip);
        let _ = storage.insert_security_event(&crate::models::CreateSecurityEvent {
            event_type: "auth_401".into(),
            severity: "warning".into(),
            source_ip: Some(ip.to_string()),
            user_agent: None,
            username: None,
            details: Some("Unauthorized request".into()),
        }).await;
    }
}

impl Default for IntrusionDetector {
    fn default() -> Self {
        Self::new()
    }
}
