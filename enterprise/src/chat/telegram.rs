//! Telegram Bot API `sendMessage`.

use std::time::Duration;

use serde::Deserialize;
use serde_json::json;
use voltrus::edition::ChatAlarm;

use super::{Attempt, ChatChannels, format_number, network_error, truncate_chars, with_retries};

/// Telegram counts message length after parsing entities, so the caps below
/// apply to the plain text: 1500 + 2 x 200 + the fixed lines stay under its
/// 4096 character limit.
const MAX_ALARM_MESSAGE: usize = 1500;
const MAX_LABEL: usize = 200;

/// Escape text for `parse_mode: "HTML"`: only `&`, `<` and `>` are special.
pub(super) fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

/// The HTML message for an alarm. Every alarm-derived string is escaped.
pub(super) fn format_text(alarm: &ChatAlarm) -> String {
    let value = if alarm.unit.is_empty() {
        format_number(alarm.value)
    } else {
        format!("{} {}", format_number(alarm.value), alarm.unit)
    };
    format!(
        "<b>[{severity}] {name}</b>\n{message}\nValue: <b>{value}</b> (threshold {threshold})\nSource: <code>{key}</code>\nTime: {time}",
        severity = escape_html(&alarm.severity.to_uppercase()),
        name = escape_html(&truncate_chars(&alarm.source_name, MAX_LABEL)),
        message = escape_html(&truncate_chars(&alarm.message, MAX_ALARM_MESSAGE)),
        value = escape_html(&value),
        threshold = escape_html(&format_number(alarm.threshold)),
        key = escape_html(&truncate_chars(&alarm.source_key, MAX_LABEL)),
        time = escape_html(&alarm.triggered_at),
    )
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    description: String,
    #[serde(default)]
    parameters: Option<ErrorParameters>,
}

#[derive(Deserialize)]
struct ErrorParameters {
    retry_after: Option<u64>,
    migrate_to_chat_id: Option<i64>,
}

/// Send `text` to one chat.
pub(super) async fn send(
    channels: &ChatChannels,
    bot_token: &str,
    chat_id: &str,
    text: &str,
) -> Result<(), String> {
    let url = format!("{}/bot{}/sendMessage", channels.telegram_base, bot_token);
    let body = json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML",
        "disable_web_page_preview": true,
    });
    with_retries(&channels.policy, || async {
        let resp = match channels.client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                return Attempt::Retry {
                    after: None,
                    error: format!("Telegram request failed: {}", network_error(e)),
                };
            }
        };
        let status = resp.status();
        if status.is_success() {
            return Attempt::Sent;
        }
        let parsed: Option<ErrorBody> = resp.json().await.ok();
        let description = parsed
            .as_ref()
            .map(|b| b.description.clone())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| status.to_string());
        let error = format!("Telegram error {}: {}", status.as_u16(), description);
        if status.as_u16() == 429 {
            let after = parsed
                .as_ref()
                .and_then(|b| b.parameters.as_ref())
                .and_then(|p| p.retry_after)
                .map(Duration::from_secs);
            return Attempt::Retry { after, error };
        }
        if status.is_server_error() {
            return Attempt::Retry { after: None, error };
        }
        if let Some(new_id) = parsed
            .as_ref()
            .and_then(|b| b.parameters.as_ref())
            .and_then(|p| p.migrate_to_chat_id)
        {
            return Attempt::Fatal(format!(
                "{error}. The group was upgraded to a supergroup: use chat id {new_id}"
            ));
        }
        Attempt::Fatal(match status.as_u16() {
            401 => format!("{error}. The bot token is invalid or was revoked"),
            403 => format!("{error}. The bot cannot message this chat: add it to the group or have the user press Start"),
            400 if description.contains("chat not found") => {
                format!("{error}. Check the chat id, and that the bot is a member of the chat")
            }
            _ => error,
        })
    })
    .await
}
