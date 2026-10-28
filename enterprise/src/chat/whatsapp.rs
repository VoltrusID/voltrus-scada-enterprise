//! WhatsApp Business Cloud API (Meta Graph `/{phone_number_id}/messages`).
//!
//! A business-initiated message can only be a pre-approved template, so the
//! alarm is delivered as template parameters, in the order the rule maps them.

use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};
use voltrus::edition::ChatAlarm;
use voltrus::notification::chat::WhatsAppConfig;

use super::{Attempt, ChatChannels, format_number, network_error, truncate_chars, with_retries};

/// Graph API version the requests are made against.
const GRAPH_VERSION: &str = "v23.0";
/// Longest text a template parameter may hold.
const MAX_PARAMETER: usize = 1024;

/// One alarm field as a template parameter. Meta rejects empty parameters,
/// newlines, tabs and runs of more than four spaces, so whitespace is
/// collapsed to single spaces and an empty value becomes "-".
pub(super) fn parameter_text(field: &str, alarm: &ChatAlarm) -> String {
    let raw = match field {
        "severity" => alarm.severity.to_uppercase(),
        "source" => alarm.source_name.clone(),
        "value" if alarm.unit.is_empty() => format_number(alarm.value),
        "value" => format!("{} {}", format_number(alarm.value), alarm.unit),
        "message" => alarm.message.clone(),
        "time" => alarm.triggered_at.clone(),
        other => other.to_string(),
    };
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        "-".to_string()
    } else {
        truncate_chars(&collapsed, MAX_PARAMETER)
    }
}

/// The `messages` request body for one recipient.
pub(super) fn request_body(config: &WhatsAppConfig, to: &str, alarm: &ChatAlarm) -> Value {
    let mut template = json!({
        "name": config.template_name,
        "language": { "code": config.language },
    });
    if !config.parameters.is_empty() {
        let parameters: Vec<Value> = config
            .parameters
            .iter()
            .map(|f| json!({ "type": "text", "text": parameter_text(f, alarm) }))
            .collect();
        template["components"] = json!([{ "type": "body", "parameters": parameters }]);
    }
    json!({
        "messaging_product": "whatsapp",
        "to": to.trim_start_matches('+'),
        "type": "template",
        "template": template,
    })
}

#[derive(Deserialize)]
struct GraphError {
    #[serde(default)]
    message: String,
    #[serde(default)]
    code: i64,
    #[serde(default)]
    error_data: Option<ErrorData>,
}

#[derive(Deserialize)]
struct ErrorData {
    #[serde(default)]
    details: String,
}

#[derive(Deserialize)]
struct ErrorEnvelope {
    error: GraphError,
}

#[derive(Deserialize)]
struct SendOk {
    #[serde(default)]
    messages: Vec<Value>,
}

/// What an operator can do about a Graph error code.
fn hint(code: i64) -> Option<&'static str> {
    Some(match code {
        190 => "the access token is invalid or expired; create a new system user token",
        10 | 200 | 299 => "the token lacks the whatsapp_business_messaging permission",
        100 => "a request field was rejected; check the phone number id",
        131030 => "the recipient is not in the allowed list of this test number",
        131026 => "the message could not be delivered; the number may not be on WhatsApp",
        131042 => "the business payment method is not set up",
        131047 | 131049 => "WhatsApp refused to deliver this message",
        132000 => "the number of template parameters does not match the template",
        132001 => "the template does not exist in this language or is not approved",
        132005 | 132007 | 132012 | 132015 | 132016 => {
            "the template text was rejected; check the parameter mapping"
        }
        _ => return None,
    })
}

/// Errors worth another attempt: throttling and Meta-side hiccups.
fn transient(code: i64) -> bool {
    matches!(
        code,
        1 | 2 | 4 | 17 | 32 | 613 | 80007 | 130429 | 131016 | 131056 | 133004
    )
}

fn describe_error(status: u16, body: Option<&ErrorEnvelope>) -> String {
    match body {
        Some(b) => {
            let e = &b.error;
            let mut out = format!("WhatsApp error {} (HTTP {status}): {}", e.code, e.message);
            if let Some(d) = e.error_data.as_ref().filter(|d| !d.details.is_empty()) {
                out.push_str(&format!(" ({})", d.details));
            }
            if let Some(h) = hint(e.code) {
                out.push_str(&format!(". Hint: {h}"));
            }
            out
        }
        None => format!("WhatsApp error: HTTP {status}"),
    }
}

/// Send the alarm template to one recipient.
pub(super) async fn send(
    channels: &ChatChannels,
    config: &WhatsAppConfig,
    to: &str,
    alarm: &ChatAlarm,
) -> Result<(), String> {
    let url = format!(
        "{}/{}/{}/messages",
        channels.graph_base, GRAPH_VERSION, config.phone_number_id
    );
    let body = request_body(config, to, alarm);
    with_retries(&channels.policy, || async {
        let resp = match channels
            .client
            .post(&url)
            .bearer_auth(&config.access_token)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return Attempt::Retry {
                    after: None,
                    error: format!("WhatsApp request failed: {}", network_error(e)),
                };
            }
        };
        let status = resp.status();
        let retry_after = resp
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs);
        if status.is_success() {
            let ok: Option<SendOk> = resp.json().await.ok();
            return match ok {
                Some(o) if !o.messages.is_empty() => Attempt::Sent,
                _ => Attempt::Fatal(
                    "WhatsApp accepted the request but returned no message id".into(),
                ),
            };
        }
        let envelope: Option<ErrorEnvelope> = resp.json().await.ok();
        let error = describe_error(status.as_u16(), envelope.as_ref());
        let code = envelope.as_ref().map(|e| e.error.code).unwrap_or(0);
        if status.as_u16() == 429 || status.is_server_error() || transient(code) {
            Attempt::Retry {
                after: retry_after,
                error,
            }
        } else {
            Attempt::Fatal(error)
        }
    })
    .await
}
