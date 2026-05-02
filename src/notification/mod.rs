pub mod email;
pub mod push;
pub mod sms;
pub mod webhook;

use crate::models::Alarm;
use crate::storage::Storage;
use email::{AlarmEmailData, EmailNotifier};
use push::{PushNotifier, build_alarm_payload};
use webhook::{WebhookPayload, send_webhook_alarm, format_triggered_at};

/// Dispatch alarm notifications to all configured channels for the given source_key.
///
/// Loads notification settings from the database and sends via email, webhook, and/or push.
/// This is spawned as an async task so it never blocks the alarm creation flow.
pub async fn dispatch_alarm_notifications(
    storage: Storage,
    alarm: &Alarm,
    email_notifier: Option<EmailNotifier>,
    push_notifier: Option<PushNotifier>,
) {
    let settings = match storage.get_notification_settings_by_source(&alarm.source_key).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to load notification settings for {}: {}", alarm.source_key, e);
            return;
        }
    };

    if settings.is_empty() {
        tracing::debug!("No notification settings configured for source {}", alarm.source_key);
        return;
    }

    // Resolve source_name and device_id from data_sources
    let sources = storage.list_data_sources(None).await.unwrap_or_default();
    let source_info = sources.iter().find(|s| s.key == alarm.source_key);

    let source_name = source_info
        .map(|s| s.name.clone())
        .unwrap_or_else(|| alarm.source_key.clone());

    let device_id = source_info
        .map(|s| s.device_id.clone())
        .unwrap_or_else(|| alarm.source_key.split(':').next().unwrap_or("unknown").to_string());

    let triggered_at_str = format_triggered_at(alarm.triggered_at);

    for setting in &settings {
        if !setting.enabled {
            continue;
        }

        match setting.channel_type.as_str() {
            "email" => {
                if let Some(ref notifier) = email_notifier {
                    // config is a JSON array of recipient email addresses
                    let recipients: Vec<String> =
                        serde_json::from_str(&setting.config).unwrap_or_default();
                    for to in &recipients {
                        let data = AlarmEmailData {
                            source_key: alarm.source_key.clone(),
                            source_name: source_name.clone(),
                            severity: alarm.severity.clone(),
                            value: alarm.value,
                            threshold: alarm.threshold,
                            message: alarm.message.clone(),
                            unit: source_info
                                .map(|s| s.unit.clone())
                                .unwrap_or_default(),
                            triggered_at: triggered_at_str.clone(),
                        };
                        if let Err(e) = notifier.send_alarm_email(to, &data).await {
                            tracing::error!(
                                "Failed to send alarm email to {}: {}",
                                to,
                                e
                            );
                        }
                    }
                } else {
                    tracing::warn!(
                        "Email notification requested but SMTP is not configured"
                    );
                }
            }
            "webhook" => {
                // config is a JSON array of webhook URLs
                let urls: Vec<String> =
                    serde_json::from_str(&setting.config).unwrap_or_default();
                for url in &urls {
                    let payload = WebhookPayload {
                        severity: alarm.severity.clone(),
                        source_key: alarm.source_key.clone(),
                        source_name: source_name.clone(),
                        value: alarm.value,
                        threshold: alarm.threshold,
                        message: alarm.message.clone(),
                        triggered_at: triggered_at_str.clone(),
                        device_info: device_id.clone(),
                    };
                    if let Err(e) = send_webhook_alarm(url, &payload).await {
                        tracing::error!(
                            "Failed to send webhook to {}: {}",
                            url,
                            e
                        );
                    }
                }
            }
            "push" => {
                if let Some(ref notifier) = push_notifier {
                    // config is a JSON array of usernames to send push notifications to
                    let usernames: Vec<String> =
                        serde_json::from_str(&setting.config).unwrap_or_default();
                    let payload = build_alarm_payload(
                        &source_name,
                        &alarm.severity,
                        &alarm.message,
                        &alarm.source_key,
                    );
                    for username in &usernames {
                        match storage.get_push_subscriptions_by_user(username).await {
                            Ok(subscriptions) => {
                                for sub in &subscriptions {
                                    match notifier.send_push_alarm(
                                        &sub.endpoint,
                                        &sub.p256dh,
                                        &sub.auth_key,
                                        &payload,
                                    ).await {
                                        push::PushSendResult::Sent => {}
                                        push::PushSendResult::SubscriptionGone => {
                                            tracing::warn!(
                                                "Push subscription {} for user {} is stale, removing",
                                                sub.id, username
                                            );
                                            if let Err(e) = storage.delete_push_subscription(sub.id).await {
                                                tracing::error!(
                                                    "Failed to delete stale push subscription {}: {}",
                                                    sub.id, e
                                                );
                                            }
                                        }
                                        push::PushSendResult::ServerError => {
                                            tracing::warn!(
                                                "Push service server error for subscription {}", sub.id
                                            );
                                        }
                                        push::PushSendResult::Other(err) => {
                                            tracing::error!(
                                                "Push notification error for subscription {}: {}",
                                                sub.id, err
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to load push subscriptions for {}: {}", username, e
                                );
                            }
                        }
                    }
                } else {
                    tracing::warn!(
                        "Push notification requested but VAPID is not configured"
                    );
                }
            }
            "sms" => {
                if let Some(ref notifier) = email_notifier {
                    // config is a JSON array of objects: [{"phone": "5551234567", "carrier": "att"}]
                    let targets: Vec<serde_json::Value> =
                        serde_json::from_str(&setting.config).unwrap_or_default();
                    for target in &targets {
                        let phone = target["phone"].as_str().unwrap_or("");
                        let carrier = target["carrier"].as_str().unwrap_or("");
                        if !phone.is_empty() && !carrier.is_empty() {
                            if let Err(e) = sms::send_sms_alarm(
                                &notifier.smtp_config(),
                                phone,
                                carrier,
                                &alarm.message,
                            )
                            .await
                            {
                                tracing::error!(
                                    "Failed to send SMS to {}@{}: {}",
                                    phone,
                                    carrier,
                                    e
                                );
                            }
                        }
                    }
                } else {
                    tracing::warn!(
                        "SMS notification requested but SMTP is not configured"
                    );
                }
            }
            other => {
                tracing::warn!("Unknown notification channel type: {}", other);
            }
        }
    }
}
