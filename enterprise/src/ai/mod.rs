//! Voltrus AI: hosted completions for the `voltrus_ai` copilot provider.

use serde::Deserialize;
use voltrus::ai_copilot::error::AiError;
use voltrus::ai_copilot::provider::{AiRequest, AiResponse, TokenUsage};
use voltrus::edition::{BoxFuture, HostedAi};

const ENDPOINT: &str = "https://ai.voltrus.id/v1/completions";

/// Hosted AI service, authenticated with the key the operator entered in the
/// copilot settings.
pub struct VoltrusAi;

#[derive(Deserialize)]
struct VoltrusAiResponse {
    content: String,
    model: String,
    usage: VoltrusAiUsage,
}

#[derive(Deserialize)]
struct VoltrusAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl HostedAi for VoltrusAi {
    fn complete<'a>(
        &'a self,
        api_key: &'a str,
        model: &'a str,
        request: AiRequest,
    ) -> BoxFuture<'a, Result<AiResponse, AiError>> {
        Box::pin(async move {
            let client = reqwest::Client::new();
            let body = serde_json::json!({
                "model": model,
                "system_prompt": request.system_prompt,
                "user_prompt": request.user_prompt,
                "max_tokens": request.max_tokens.unwrap_or(2048),
            });

            let resp = client
                .post(ENDPOINT)
                .header("Authorization", format!("Bearer {api_key}"))
                .header("Content-Type", "application/json")
                .timeout(std::time::Duration::from_secs(60))
                .json(&body)
                .send()
                .await?;

            let status = resp.status();
            if status.is_client_error() || status.is_server_error() {
                let text = resp.text().await.unwrap_or_default();
                return Err(AiError::ProviderError(format!(
                    "Voltrus AI error ({status}): {text}"
                )));
            }

            let data: VoltrusAiResponse = resp.json().await.map_err(|e| {
                AiError::MalformedResponse(format!("Failed to parse Voltrus AI response: {e}"))
            })?;

            Ok(AiResponse {
                content: data.content,
                model: data.model,
                usage: TokenUsage {
                    prompt_tokens: data.usage.prompt_tokens,
                    completion_tokens: data.usage.completion_tokens,
                    total_tokens: data.usage.total_tokens,
                },
                provider: "voltrus_ai".into(),
            })
        })
    }
}
