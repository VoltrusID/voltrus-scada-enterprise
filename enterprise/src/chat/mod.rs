//! Telegram and WhatsApp alarm notifications.
//!
//! The core owns the stored config shape and its validation
//! (`voltrus::notification::chat`); this module delivers. [`ChatChannels`] is
//! registered as the core's `ChatSender` at startup.

mod telegram;
mod whatsapp;

use std::time::Duration;

use voltrus::edition::{BoxFuture, ChatAlarm, ChatDelivery, ChatSender};
use voltrus::notification::chat;

const TELEGRAM_API: &str = "https://api.telegram.org";
const GRAPH_API: &str = "https://graph.facebook.com";

/// How hard a send is retried.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Retries after the first attempt.
    pub max_retries: u32,
    /// First backoff; doubles per retry (500, 1000, 2000 ms).
    pub base_delay: Duration,
    /// Longest wait a provider's rate limit (`retry_after`) may ask for. Longer
    /// waits fail the send instead of holding the alarm task.
    pub max_retry_after: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_retry_after: Duration::from_secs(30),
        }
    }
}

/// Result of one HTTP attempt.
enum Attempt {
    Sent,
    /// Try again: after the provider's `retry_after` if it gave one, else the backoff.
    Retry {
        after: Option<Duration>,
        error: String,
    },
    /// Retrying cannot help (bad token, unknown chat, unapproved template...).
    Fatal(String),
}

/// Run `op` until it sends, is fatal, or the retries are spent.
async fn with_retries<F, Fut>(policy: &RetryPolicy, mut op: F) -> Result<(), String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Attempt>,
{
    let mut attempt = 0u32;
    loop {
        match op().await {
            Attempt::Sent => return Ok(()),
            Attempt::Fatal(e) => return Err(e),
            Attempt::Retry { after, error } => {
                if attempt >= policy.max_retries {
                    return Err(format!("{error} (gave up after {} attempts)", attempt + 1));
                }
                let wait = match after {
                    Some(a) if a > policy.max_retry_after => {
                        return Err(format!(
                            "{error} (rate limited for {}s, not waiting)",
                            a.as_secs()
                        ));
                    }
                    Some(a) => a,
                    None => policy.base_delay * 2u32.pow(attempt),
                };
                tracing::warn!("Chat notification failed: {error}, retrying in {wait:?}");
                tokio::time::sleep(wait).await;
                attempt += 1;
            }
        }
    }
}

/// A network failure as text, without the request URL (Telegram puts the bot
/// token in it).
fn network_error(e: reqwest::Error) -> String {
    e.without_url().to_string()
}

/// The chat channels of the enterprise edition.
pub struct ChatChannels {
    client: reqwest::Client,
    telegram_base: String,
    graph_base: String,
    policy: RetryPolicy,
}

impl Default for ChatChannels {
    fn default() -> Self {
        Self::new(TELEGRAM_API, GRAPH_API, RetryPolicy::default())
    }
}

impl ChatChannels {
    /// Channels talking to the given API roots (tests point them at a local server).
    pub fn new(telegram_base: &str, graph_base: &str, policy: RetryPolicy) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            client,
            telegram_base: telegram_base.trim_end_matches('/').to_string(),
            graph_base: graph_base.trim_end_matches('/').to_string(),
            policy,
        }
    }

    async fn deliver(
        &self,
        channel: &str,
        config: &str,
        alarm: &ChatAlarm,
    ) -> anyhow::Result<Vec<ChatDelivery>> {
        chat::validate(channel, config).map_err(anyhow::Error::msg)?;
        let mut deliveries = Vec::new();
        match channel {
            "telegram" => {
                let cfg = chat::parse_telegram(config).map_err(anyhow::Error::msg)?;
                let text = telegram::format_text(alarm);
                for chat_id in &cfg.chat_ids {
                    let result = telegram::send(self, &cfg.bot_token, chat_id, &text).await;
                    deliveries.push(ChatDelivery {
                        target: chat_id.clone(),
                        error: result.err(),
                    });
                }
            }
            "whatsapp" => {
                let cfg = chat::parse_whatsapp(config).map_err(anyhow::Error::msg)?;
                for to in &cfg.recipients {
                    let result = whatsapp::send(self, &cfg, to, alarm).await;
                    deliveries.push(ChatDelivery {
                        target: to.clone(),
                        error: result.err(),
                    });
                }
            }
            other => anyhow::bail!("'{other}' is not a chat channel"),
        }
        Ok(deliveries)
    }
}

impl ChatSender for ChatChannels {
    fn send<'a>(
        &'a self,
        channel: &'a str,
        config: &'a str,
        alarm: &'a ChatAlarm,
    ) -> BoxFuture<'a, anyhow::Result<Vec<ChatDelivery>>> {
        Box::pin(self.deliver(channel, config, alarm))
    }
}

/// `value` with at most three decimals and no trailing zeros.
pub(crate) fn format_number(value: f64) -> String {
    let s = format!("{value:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-" {
        "0".into()
    } else {
        s.to_string()
    }
}

/// `text` cut to at most `max` characters, with an ellipsis when cut.
pub(crate) fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests;
