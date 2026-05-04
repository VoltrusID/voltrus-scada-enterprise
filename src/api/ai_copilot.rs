use axum::extract::{Extension, Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use super::{AppError, AppState, AuthUser};
use crate::ai_copilot::{self, AiProviderType, AiRequest, AiSettings, resolve_provider};
use crate::models;
use crate::security::AuditEvent;

// ── Settings ──

#[utoipa::path(
    get,
    path = "/api/v1/ai/settings",
    responses((status = 200, description = "AI copilot settings")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_ai_settings(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let provider = state.storage.get_ai_setting("provider").await?
        .unwrap_or_else(|| "byok_openai".into());
    let api_key = state.storage.get_ai_setting("api_key").await?;
    let default_model = state.storage.get_ai_setting("default_model").await?;

    let api_key_preview = match &api_key {
        Some(key) if key.len() > 8 => {
            let last4 = &key[key.len()-4..];
            format!("****{}", last4)
        }
        Some(_) => "****".into(),
        None => "".into(),
    };

    Ok(Json(models::AiSettingsResponse {
        provider,
        api_key_set: api_key.is_some(),
        api_key_preview,
        default_model,
    }))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateAiSettingsBody {
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
}

#[utoipa::path(
    put,
    path = "/api/v1/ai/settings",
    responses((status = 200, description = "Settings updated")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_ai_settings(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<UpdateAiSettingsBody>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    if let Some(provider) = &body.provider {
        if AiProviderType::from_str(provider).is_none() {
            return Err(AppError(anyhow::anyhow!("Invalid provider. Use: byok_openai, byok_anthropic, voltrus_ai")));
        }
        state.storage.set_ai_setting("provider", provider).await?;
    }

    if let Some(api_key) = &body.api_key {
        if !api_key.is_empty() {
            state.storage.set_ai_setting("api_key", api_key).await?;
        }
    }

    if let Some(model) = &body.default_model {
        state.storage.set_ai_setting("default_model", model).await?;
    }

    state.audit_logger.log(AuditEvent {
        action: "ai_settings_updated".into(),
        username: auth.username.clone(),
        resource: "ai_copilot".into(),
        details: Some(json!({"provider": body.provider}).to_string()),
    });

    Ok(Json(json!({"ok": true})))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/settings/test",
    responses((status = 200, description = "Test result")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn test_ai_settings(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let settings = load_ai_settings(&state).await?;
    let provider = resolve_provider(&settings);

    let request = AiRequest {
        system_prompt: "You are a test assistant. Reply with exactly: OK".into(),
        user_prompt: "Test connection".into(),
        max_tokens: Some(10),
        temperature: Some(0.0),
    };

    match provider.complete(request).await {
        Ok(resp) => Ok(Json(json!({
            "ok": true,
            "model": resp.model,
            "provider": resp.provider,
        }))),
        Err(e) => Ok(Json(json!({
            "ok": false,
            "error": e.to_string(),
        }))),
    }
}

// ── Proposals ──

#[derive(Deserialize)]
pub struct ListProposalsQuery {
    pub proposal_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/ai/proposals",
    responses((status = 200, description = "List AI proposals")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_proposals(
    State(state): State<AppState>,
    Query(query): Query<ListProposalsQuery>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let proposals = state.storage.list_ai_proposals(
        query.proposal_type.as_deref(),
        query.status.as_deref(),
        query.limit.unwrap_or(50),
    ).await?;

    Ok(Json(proposals))
}

#[utoipa::path(
    get,
    path = "/api/v1/ai/proposals/{id}",
    responses((status = 200, description = "Get AI proposal")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_proposal(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let proposal = state.storage.get_ai_proposal(id).await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Proposal not found")))?;

    Ok(Json(proposal))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ReviewProposalBody {
    pub applied_config: Option<serde_json::Value>,
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/proposals/{id}/approve",
    responses((status = 200, description = "Proposal approved")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn approve_proposal(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(body): Json<ReviewProposalBody>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let applied_config = body.applied_config.map(|v| v.to_string());
    state.storage.update_ai_proposal_status(
        id,
        "approved",
        Some(&auth.username),
        applied_config.as_deref(),
    ).await?;

    Ok(Json(json!({"ok": true})))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/proposals/{id}/reject",
    responses((status = 200, description = "Proposal rejected")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn reject_proposal(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    state.storage.update_ai_proposal_status(id, "rejected", Some(&auth.username), None).await?;

    Ok(Json(json!({"ok": true})))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/proposals/{id}/apply",
    responses((status = 200, description = "Proposal applied to production")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn apply_proposal(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let proposal = state.storage.get_ai_proposal(id).await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Proposal not found")))?;

    if proposal.status != "approved" {
        return Err(AppError(anyhow::anyhow!("Proposal must be approved before applying")));
    }

    let config_to_apply = proposal.applied_config
        .as_ref()
        .unwrap_or(&proposal.proposed_config);

    // Apply config based on proposal type
    match proposal.proposal_type.as_str() {
        "register_map" => apply_register_map(&state, config_to_apply, &proposal.target_entity_id).await?,
        "alarm_thresholds" => apply_alarm_thresholds(&state, config_to_apply).await?,
        "tag_rename" => apply_tag_rename(&state, config_to_apply).await?,
        "dashboard_layout" => apply_dashboard_layout(&state, config_to_apply, &proposal.target_entity_id).await?,
        "flow_rule" => apply_flow_rule(&state, config_to_apply).await?,
        "equipment_template" => apply_equipment_template(&state, config_to_apply, &proposal.target_entity_id).await?,
        _ => return Err(AppError(anyhow::anyhow!("Unknown proposal type: {}", proposal.proposal_type))),
    }

    state.storage.mark_ai_proposal_applied(id, Some(&config_to_apply.to_string())).await?;

    state.audit_logger.log(AuditEvent {
        action: "ai_proposal_applied".into(),
        username: auth.username.clone(),
        resource: "ai_copilot".into(),
        details: Some(json!({
            "proposal_id": id,
            "proposal_type": proposal.proposal_type,
            "source": "ai-assist",
            "title": proposal.title,
        }).to_string()),
    });

    Ok(Json(json!({"ok": true})))
}

// ── Generation endpoints ──

#[utoipa::path(
    post,
    path = "/api/v1/ai/generate/register-map",
    responses((status = 200, description = "Register map generated")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn generate_register_map(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(body): Json<models::RegisterMapRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let settings = load_ai_settings(&state).await?;
    let provider = resolve_provider(&settings);

    let user_prompt = ai_copilot::prompts::register_map_user_prompt(
        &ai_copilot::prompts::scrub_prompt(&body.file_content),
        &body.device_name,
    );

    let response = provider.complete(AiRequest {
        system_prompt: ai_copilot::prompts::register_map_system_prompt().into(),
        user_prompt,
        max_tokens: Some(4096),
        temperature: Some(0.2),
    }).await.map_err(|e| AppError(e.into()))?;

    let json_str = ai_copilot::parser::extract_json(&response.content)?;
    let registers = ai_copilot::parser::parse_register_map(&json_str)?;

    let proposal_id = state.storage.create_ai_proposal(
        "register_map",
        &format!("Register map for {}", body.device_name),
        &serde_json::json!({"device_name": body.device_name, "file_type": body.file_type}).to_string(),
        &serde_json::to_string(&registers)?,
        body.device_id.as_deref(),
        &response.model,
        response.usage.total_tokens as i64,
    ).await?;

    let proposal = state.storage.get_ai_proposal(proposal_id).await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Failed to create proposal")))?;

    Ok(Json(proposal))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/generate/dashboard-layout",
    responses((status = 200, description = "Dashboard layout generated")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn generate_dashboard_layout(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(body): Json<models::DashboardLayoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let settings = load_ai_settings(&state).await?;
    let provider = resolve_provider(&settings);

    // Fetch available data sources for context
    let data_sources = state.storage.list_data_sources(None).await?;
    let tags_str = data_sources.iter()
        .map(|ds| format!("{}: {} [{}]", ds.key, ds.name, ds.unit))
        .collect::<Vec<_>>()
        .join("\n");

    let user_prompt = ai_copilot::prompts::dashboard_layout_user_prompt(
        &ai_copilot::prompts::scrub_prompt(&body.description),
        &ai_copilot::prompts::scrub_prompt(&tags_str),
    );

    let response = provider.complete(AiRequest {
        system_prompt: ai_copilot::prompts::dashboard_layout_system_prompt().into(),
        user_prompt,
        max_tokens: Some(4096),
        temperature: Some(0.3),
    }).await.map_err(|e| AppError(e.into()))?;

    let json_str = ai_copilot::parser::extract_json(&response.content)?;
    let layout = ai_copilot::parser::parse_dashboard_layout(&json_str)?;

    let proposal_id = state.storage.create_ai_proposal(
        "dashboard_layout",
        &format!("Dashboard layout: {}", body.description.chars().take(60).collect::<String>()),
        &serde_json::json!({"description": body.description}).to_string(),
        &serde_json::to_string(&layout)?,
        body.screen_id.as_deref(),
        &response.model,
        response.usage.total_tokens as i64,
    ).await?;

    let proposal = state.storage.get_ai_proposal(proposal_id).await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Failed to create proposal")))?;

    Ok(Json(proposal))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/generate/alarm-thresholds",
    responses((status = 200, description = "Alarm thresholds generated")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn generate_alarm_thresholds(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(body): Json<models::AlarmThresholdRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let settings = load_ai_settings(&state).await?;
    let provider = resolve_provider(&settings);

    let tags_str = body.tags.iter()
        .map(|t| format!("{}: {} [{}] range {}-{}", t.source_key, t.name, t.unit.as_deref().unwrap_or(""), t.min_val.map(|v| v.to_string()).unwrap_or_default(), t.max_val.map(|v| v.to_string()).unwrap_or_default()))
        .collect::<Vec<_>>()
        .join("\n");

    let user_prompt = ai_copilot::prompts::alarm_threshold_user_prompt(
        &body.device_type,
        &tags_str,
    );

    let response = provider.complete(AiRequest {
        system_prompt: ai_copilot::prompts::alarm_threshold_system_prompt().into(),
        user_prompt,
        max_tokens: Some(2048),
        temperature: Some(0.2),
    }).await.map_err(|e| AppError(e.into()))?;

    let json_str = ai_copilot::parser::extract_json(&response.content)?;
    let thresholds = ai_copilot::parser::parse_alarm_thresholds(&json_str)?;

    let proposal_id = state.storage.create_ai_proposal(
        "alarm_thresholds",
        &format!("Alarm thresholds for {}", body.device_type),
        &serde_json::json!({"device_type": body.device_type, "tag_count": body.tags.len()}).to_string(),
        &serde_json::to_string(&thresholds)?,
        None,
        &response.model,
        response.usage.total_tokens as i64,
    ).await?;

    let proposal = state.storage.get_ai_proposal(proposal_id).await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Failed to create proposal")))?;

    Ok(Json(proposal))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/generate/tag-rename",
    responses((status = 200, description = "Tag rename mapping generated")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn generate_tag_rename(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(body): Json<models::TagRenameRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let settings = load_ai_settings(&state).await?;
    let provider = resolve_provider(&settings);

    let tags_str = body.tags.iter()
        .map(|t| t.current.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let user_prompt = ai_copilot::prompts::tag_rename_user_prompt(
        &tags_str,
        &body.convention,
        body.custom_pattern.as_deref(),
    );

    let response = provider.complete(AiRequest {
        system_prompt: ai_copilot::prompts::tag_rename_system_prompt().into(),
        user_prompt,
        max_tokens: Some(4096),
        temperature: Some(0.1),
    }).await.map_err(|e| AppError(e.into()))?;

    let json_str = ai_copilot::parser::extract_json(&response.content)?;
    let renames = ai_copilot::parser::parse_tag_rename(&json_str)?;

    let proposal_id = state.storage.create_ai_proposal(
        "tag_rename",
        &format!("Tag rename to {} convention", body.convention),
        &serde_json::json!({"convention": body.convention, "tag_count": body.tags.len()}).to_string(),
        &serde_json::to_string(&renames)?,
        None,
        &response.model,
        response.usage.total_tokens as i64,
    ).await?;

    let proposal = state.storage.get_ai_proposal(proposal_id).await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Failed to create proposal")))?;

    Ok(Json(proposal))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/generate/flow-rule",
    responses((status = 200, description = "Flow rule generated")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn generate_flow_rule(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(body): Json<models::FlowRuleRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let settings = load_ai_settings(&state).await?;
    let provider = resolve_provider(&settings);

    let tags_str = body.available_tags.join("\n");

    let user_prompt = ai_copilot::prompts::flow_rule_user_prompt(
        &body.description,
        &tags_str,
    );

    let response = provider.complete(AiRequest {
        system_prompt: ai_copilot::prompts::flow_rule_system_prompt().into(),
        user_prompt,
        max_tokens: Some(2048),
        temperature: Some(0.2),
    }).await.map_err(|e| AppError(e.into()))?;

    let json_str = ai_copilot::parser::extract_json(&response.content)?;
    let flow = ai_copilot::parser::parse_flow_rule(&json_str)?;

    let proposal_id = state.storage.create_ai_proposal(
        "flow_rule",
        &format!("Flow rule: {}", body.description.chars().take(60).collect::<String>()),
        &serde_json::json!({"description": body.description}).to_string(),
        &serde_json::to_string(&flow)?,
        None,
        &response.model,
        response.usage.total_tokens as i64,
    ).await?;

    let proposal = state.storage.get_ai_proposal(proposal_id).await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Failed to create proposal")))?;

    Ok(Json(proposal))
}

// ── Templates ──

#[utoipa::path(
    get,
    path = "/api/v1/ai/templates",
    responses((status = 200, description = "List equipment templates")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_templates(
    State(state): State<AppState>,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    let templates = state.storage.list_equipment_templates(query.industry.as_deref()).await?;
    Ok(Json(templates))
}

#[derive(Deserialize)]
pub struct ListTemplatesQuery {
    pub industry: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/generate/from-template",
    responses((status = 200, description = "Template instantiated")),
    tag = "AI Copilot",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn generate_from_template(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(body): Json<models::TemplateInstantiateRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.ai_copilot() {
        return Err(AppError(anyhow::anyhow!("AI Copilot requires Starter tier or above")));
    }

    // Look up template by ID via list + filter (no dedicated get method)
    let templates = state.storage.list_equipment_templates(None).await?;
    let template = templates.into_iter()
        .find(|t| t.id == body.template_id)
        .ok_or_else(|| AppError(anyhow::anyhow!("Template not found")))?;

    let prefix = body.prefix.as_deref().unwrap_or(&template.equipment_type);

    let config = json!({
        "template_id": template.id,
        "template_name": template.name,
        "prefix": prefix,
        "device_id": body.device_id,
        "tags": template.tag_template,
        "alarms": template.alarm_template,
        "dashboard": template.dashboard_template,
        "flows": template.flow_template,
    });

    let proposal_id = state.storage.create_ai_proposal(
        "equipment_template",
        &format!("Equipment template: {}", template.name),
        &json!({"template_id": template.id, "prefix": prefix}).to_string(),
        &config.to_string(),
        body.device_id.as_deref(),
        "template",
        0,
    ).await?;

    let proposal = state.storage.get_ai_proposal(proposal_id).await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Failed to create proposal")))?;

    Ok(Json(proposal))
}

// ── Helpers ──

async fn load_ai_settings(state: &AppState) -> Result<AiSettings, AppError> {
    let provider_str = state.storage.get_ai_setting("provider").await?
        .ok_or_else(|| AppError(anyhow::anyhow!("AI provider not configured. Please set up AI Copilot in settings.")))?;
    let provider = AiProviderType::from_str(&provider_str)
        .ok_or_else(|| AppError(anyhow::anyhow!("Invalid AI provider: {}", provider_str)))?;
    let api_key = state.storage.get_ai_setting("api_key").await?
        .ok_or_else(|| AppError(anyhow::anyhow!("API key not configured")))?;
    let default_model = state.storage.get_ai_setting("default_model").await?;

    Ok(AiSettings {
        provider,
        api_key,
        default_model,
    })
}

async fn apply_register_map(state: &AppState, config: &serde_json::Value, device_id: &Option<String>) -> Result<(), AppError> {
    let device_id_str = device_id.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("No device ID specified for register map")))?
        .as_str();

    let registers = config.as_array()
        .ok_or_else(|| AppError(anyhow::anyhow!("Invalid register map format")))?;

    for reg in registers {
        let name = reg["name"].as_str().unwrap_or("unnamed");
        let address = reg["address"].as_i64().unwrap_or(0) as u16;
        let register_type = reg["register_type"].as_str().unwrap_or("float32");
        let scale = reg["scale"].as_f64().unwrap_or(1.0);

        state.storage.create_data_source(
            &format!("{}:{}", device_id_str, name),
            name,
            reg["unit"].as_str().unwrap_or(""),
            "#f97316",
            0.0,   // min_val
            0.0,   // max_val
            0.0,   // warn_val
            0.0,   // crit_val
            0.0,   // base_val
            0,     // sort_order
            1,     // decimals
            "modbus",
            &serde_json::json!({
                "device_id": device_id_str,
                "address": address,
                "register_type": register_type,
                "scale": scale,
            }).to_string(),
            device_id_str,
            None,  // equipment_id
            "gauge", // chart_type
        ).await.map_err(|e| AppError(e))?;
    }

    Ok(())
}

async fn apply_alarm_thresholds(state: &AppState, config: &serde_json::Value) -> Result<(), AppError> {
    let thresholds = config.as_array()
        .ok_or_else(|| AppError(anyhow::anyhow!("Invalid alarm thresholds format")))?;

    let data_sources = state.storage.list_data_sources(None).await?;

    for threshold in thresholds {
        let source_key = threshold["source_key"].as_str()
            .ok_or_else(|| AppError(anyhow::anyhow!("Missing source_key in threshold")))?;

        if let Some(ds) = data_sources.iter().find(|ds| ds.key.contains(source_key) || source_key.contains(&ds.key)) {
            let warn_val = threshold["warn_val"].as_f64().unwrap_or(ds.warn_val);
            let crit_val = threshold["crit_val"].as_f64().unwrap_or(ds.crit_val);

            state.storage.update_data_source(
                ds.id,
                &json!({"warn_val": warn_val, "crit_val": crit_val}),
            ).await.map_err(|e| AppError(e))?;
        }
    }

    Ok(())
}

async fn apply_tag_rename(state: &AppState, config: &serde_json::Value) -> Result<(), AppError> {
    let renames = config.as_array()
        .ok_or_else(|| AppError(anyhow::anyhow!("Invalid tag rename format")))?;

    let data_sources = state.storage.list_data_sources(None).await?;

    for entry in renames {
        let current = entry["current"].as_str().unwrap_or("");
        let proposed = entry["proposed"].as_str().unwrap_or("");
        if current.is_empty() || proposed.is_empty() { continue; }

        if let Some(ds) = data_sources.iter().find(|ds| ds.name == current || ds.key == current) {
            state.storage.update_data_source(
                ds.id,
                &json!({"name": proposed}),
            ).await.map_err(|e| AppError(e))?;
        }
    }

    Ok(())
}

async fn apply_dashboard_layout(_state: &AppState, _config: &serde_json::Value, _screen_id: &Option<String>) -> Result<(), AppError> {
    // Placeholder — full implementation would update screen_layouts table
    Ok(())
}

async fn apply_flow_rule(state: &AppState, config: &serde_json::Value) -> Result<(), AppError> {
    let name = config["name"].as_str().unwrap_or("AI-generated flow");
    let trigger_config = config["trigger_config"].to_string();
    let nodes = config["nodes"].to_string();

    let id = format!("ai_flow_{}", chrono_like_id());

    state.storage.create_flow(
        &id,
        name,
        false, // Disabled by default — engineer must enable
        &trigger_config,
        &nodes,
    ).await.map_err(|e| AppError(e))?;

    Ok(())
}

async fn apply_equipment_template(state: &AppState, config: &serde_json::Value, device_id: &Option<String>) -> Result<(), AppError> {
    if let Some(tags) = config["tags"].as_array() {
        let prefix = config["prefix"].as_str().unwrap_or("equip");
        let dev_id = device_id.as_deref().unwrap_or(prefix);

        for tag in tags {
            let name = tag["name_pattern"].as_str().unwrap_or("TAG")
                .replace("{n}", "1");
            state.storage.create_data_source(
                &format!("{}:{}", prefix, name),
                &name,
                tag["unit"].as_str().unwrap_or(""),
                "#f97316",
                0.0, 0.0, 0.0, 0.0, 0.0, // min/max/warn/crit/base
                0,   // sort_order
                1,   // decimals
                tag["register_area"].as_str().unwrap_or("holding"),
                &serde_json::json!({"default_type": tag["default_type"].as_str().unwrap_or("float32")}).to_string(),
                dev_id,
                None,   // equipment_id
                "gauge", // chart_type
            ).await.map_err(|e| AppError(e))?;
        }
    }
    Ok(())
}

fn chrono_like_id() -> i64 {
    (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1000) as i64
}
