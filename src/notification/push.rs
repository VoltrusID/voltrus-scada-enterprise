use anyhow::Result;
use serde::Serialize;
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder,
    WebPushClient, WebPushMessageBuilder, WebPushError,
};

use crate::models::PushConfig;

/// JSON payload sent to the browser push service, then displayed by the
/// service worker as a notification.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct AlarmPayload {
    pub title: String,
    pub body: String,
    pub severity: String,
    pub source_key: String,
}

/// Result of sending a push notification.
#[derive(Debug)]
pub enum PushSendResult {
    /// Notification was accepted by the push service.
    Sent,
    /// The subscription endpoint is no longer valid (410 Gone or equivalent).
    /// The caller should remove the subscription from the database.
    SubscriptionGone,
    /// A transient server error occurred; the caller may retry later.
    ServerError,
    /// Any other error (network, crypto, etc.).
    Other(String),
}

/// Web Push notifier backed by the `web-push` crate.
///
/// Holds VAPID credentials and an HTTP client for sending push notifications
/// to browser push subscriptions.
#[derive(Clone)]
pub struct PushNotifier {
    vapid_private_key: String,
    vapid_public_key: String,
    vapid_subject: String,
    client: IsahcWebPushClient,
}

impl PushNotifier {
    /// Create a new `PushNotifier` from the parsed `PushConfig`.
    ///
    /// Validates that the VAPID private key can be parsed by the `web-push` crate.
    pub fn new(config: &PushConfig) -> Result<Self> {
        // Validate the private key is usable by building a partial VAPID signature.
        let _partial = VapidSignatureBuilder::from_base64_no_sub(&config.vapid_private_key)
            .map_err(|e| anyhow::anyhow!("Invalid VAPID private key: {}", e))?;

        let client = IsahcWebPushClient::new()
            .map_err(|e| anyhow::anyhow!("Failed to create web push client: {}", e))?;

        Ok(Self {
            vapid_private_key: config.vapid_private_key.clone(),
            vapid_public_key: config.vapid_public_key.clone(),
            vapid_subject: config.vapid_subject.clone(),
            client,
        })
    }

    /// Returns the public VAPID key (for serving to the frontend).
    pub fn public_key(&self) -> &str {
        &self.vapid_public_key
    }

    /// Send an alarm notification to a push subscription.
    ///
    /// * `endpoint` — push subscription endpoint URL
    /// * `p256dh` — client's P-256 ECDH public key (base64url)
    /// * `auth_key` — client's auth secret (base64url)
    /// * `alarm` — payload containing title, body, severity, source_key
    pub async fn send_push_alarm(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth_key: &str,
        alarm: &AlarmPayload,
    ) -> PushSendResult {
        let subscription = SubscriptionInfo::new(endpoint, p256dh, auth_key);

        // Build VAPID signature
        let mut sig_builder = match VapidSignatureBuilder::from_base64(
            &self.vapid_private_key,
            &subscription,
        ) {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("VAPID signature build failed: {}", e);
                return PushSendResult::Other(format!("VAPID signature error: {}", e));
            }
        };

        // Set the VAPID subject claim (required by push services)
        sig_builder.add_claim("sub", self.vapid_subject.as_str());

        let signature = match sig_builder.build() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("VAPID signature build failed: {}", e);
                return PushSendResult::Other(format!("VAPID signature error: {}", e));
            }
        };

        // Serialize payload
        let payload_bytes = match serde_json::to_vec(alarm) {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to serialize push payload: {}", e);
                return PushSendResult::Other(format!("Serialization error: {}", e));
            }
        };

        // Build message
        let mut builder = WebPushMessageBuilder::new(&subscription);
        builder.set_payload(ContentEncoding::Aes128Gcm, &payload_bytes);
        builder.set_vapid_signature(signature);
        builder.set_ttl(86400); // 24 hours

        // Set urgency based on severity
        let urgency = match alarm.severity.to_lowercase().as_str() {
            "critical" => web_push::Urgency::High,
            _ => web_push::Urgency::Normal,
        };
        builder.set_urgency(urgency);

        let message = match builder.build() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to build push message: {}", e);
                return PushSendResult::Other(format!("Message build error: {}", e));
            }
        };

        // Send
        match self.client.send(message).await {
            Ok(_) => {
                tracing::info!(
                    "Push notification sent for {} [{}]",
                    alarm.source_key,
                    alarm.severity
                );
                PushSendResult::Sent
            }
            Err(WebPushError::EndpointNotValid(_)) | Err(WebPushError::EndpointNotFound(_)) => {
                tracing::warn!(
                    "Push subscription endpoint no longer valid: {}",
                    endpoint
                );
                PushSendResult::SubscriptionGone
            }
            Err(WebPushError::ServerError { .. }) => {
                tracing::warn!(
                    "Push service server error for endpoint: {}",
                    endpoint
                );
                PushSendResult::ServerError
            }
            Err(WebPushError::Unauthorized(_)) => {
                tracing::error!(
                    "Push notification unauthorized — check VAPID keys and subject"
                );
                PushSendResult::Other("Unauthorized: check VAPID configuration".to_string())
            }
            Err(e) => {
                tracing::error!("Push notification error: {}", e);
                PushSendResult::Other(e.to_string())
            }
        }
    }
}

/// Build an `AlarmPayload` from alarm fields.
pub fn build_alarm_payload(
    source_name: &str,
    severity: &str,
    message: &str,
    source_key: &str,
) -> AlarmPayload {
    AlarmPayload {
        title: format!("[{}] {}", severity.to_uppercase(), source_name),
        body: message.to_string(),
        severity: severity.to_string(),
        source_key: source_key.to_string(),
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_alarm_payload_critical() {
        let payload = build_alarm_payload(
            "PLTS 1 Voltage",
            "critical",
            "PLTS 1 Voltage reached 250.5V (critical: 240.0V)",
            "plts_1:voltage",
        );

        assert_eq!(payload.title, "[CRITICAL] PLTS 1 Voltage");
        assert_eq!(payload.body, "PLTS 1 Voltage reached 250.5V (critical: 240.0V)");
        assert_eq!(payload.severity, "critical");
        assert_eq!(payload.source_key, "plts_1:voltage");
    }

    #[test]
    fn test_build_alarm_payload_warning() {
        let payload = build_alarm_payload(
            "Generator Temp",
            "warning",
            "Generator Temp reached 85.0C (warning: 80.0C)",
            "gen_1:temp",
        );

        assert_eq!(payload.title, "[WARNING] Generator Temp");
        assert_eq!(payload.severity, "warning");
    }

    #[test]
    fn test_alarm_payload_serialization() {
        let payload = build_alarm_payload(
            "PLTS 1 Voltage",
            "critical",
            "Voltage high",
            "plts_1:voltage",
        );

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"title\":\"[CRITICAL] PLTS 1 Voltage\""));
        assert!(json.contains("\"severity\":\"critical\""));
        assert!(json.contains("\"source_key\":\"plts_1:voltage\""));
        assert!(json.contains("\"body\":\"Voltage high\""));

        // Verify it can round-trip
        let back: AlarmPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, payload.title);
        assert_eq!(back.severity, payload.severity);
    }

    #[test]
    fn test_alarm_payload_json_structure() {
        let payload = AlarmPayload {
            title: "Test".to_string(),
            body: "Body".to_string(),
            severity: "critical".to_string(),
            source_key: "key".to_string(),
        };

        let json: serde_json::Value = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["title"], "Test");
        assert_eq!(json["body"], "Body");
        assert_eq!(json["severity"], "critical");
        assert_eq!(json["source_key"], "key");
        // Verify no extra fields
        assert_eq!(json.as_object().unwrap().len(), 4);
    }

    #[test]
    fn test_push_config_defaults() {
        let yaml = r#"
vapid_private_key: "test-private"
vapid_public_key: "test-public"
"#;
        let config: PushConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vapid_private_key, "test-private");
        assert_eq!(config.vapid_public_key, "test-public");
        assert_eq!(config.vapid_subject, "mailto:admin@example.com");
    }

    #[test]
    fn test_push_config_custom_subject() {
        let yaml = r#"
vapid_private_key: "test-private"
vapid_public_key: "test-public"
vapid_subject: "mailto:admin@voltrus.id"
"#;
        let config: PushConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vapid_subject, "mailto:admin@voltrus.id");
    }
}
