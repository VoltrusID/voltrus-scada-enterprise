use axum::{extract::{Extension, Path, State}, Json, http::StatusCode};
use serde_json::json;
use crate::api::{AppState, AppError, AuthUser};
use crate::models::Flow;

pub async fn list_flows(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let flows = state.storage.list_flows().await?;
    Ok(Json(json!({ "flows": flows })))
}

pub async fn get_flow(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Flow>, AppError> {
    let flow = state.storage.get_flow(&id).await?
        .ok_or_else(|| anyhow::anyhow!("Flow not found"))?;
    Ok(Json(flow))
}

pub async fn create_flow(State(state): State<AppState>, Extension(auth): Extension<AuthUser>, Json(req): Json<Flow>) -> Result<Json<serde_json::Value>, AppError> {
    let id = req.id.clone();
    state.storage.create_flow(&req.id, &req.name, req.enabled, &req.trigger_config, &req.nodes).await?;
    if let Some(engine) = &state.flow_engine {
        let runtime_flow = db_flow_to_runtime_flow(req)?;
        engine.add_flow(runtime_flow).await;
    }
    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_flow".to_string(),
        username: auth.username.clone(),
        resource: format!("flow:{}", id),
        details: None,
    });
    Ok(Json(json!({ "id": id })))
}

pub async fn update_flow(State(state): State<AppState>, Extension(auth): Extension<AuthUser>, Path(id): Path<String>, Json(req): Json<Flow>) -> Result<StatusCode, AppError> {
    state.storage.update_flow(&id, &req.name, req.enabled, &req.trigger_config, &req.nodes).await?;
    if let Some(engine) = &state.flow_engine {
        let runtime_flow = db_flow_to_runtime_flow(req)?;
        engine.update_flow(runtime_flow).await;
    }
    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_flow".to_string(),
        username: auth.username.clone(),
        resource: format!("flow:{}", id),
        details: None,
    });
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_flow(State(state): State<AppState>, Extension(auth): Extension<AuthUser>, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    state.storage.delete_flow(&id).await?;
    if let Some(engine) = &state.flow_engine {
        engine.remove_flow(&id).await;
    }
    state.audit_logger.log(crate::security::AuditEvent {
        action: "delete_flow".to_string(),
        username: auth.username.clone(),
        resource: format!("flow:{}", id),
        details: None,
    });
    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_flow(State(state): State<AppState>, Extension(auth): Extension<AuthUser>, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    if let Some(engine) = &state.flow_engine {
        if let Some(mut flow) = engine.get_flow(&id).await {
            flow.enabled = true;
            engine.update_flow(flow).await;
        }
    }
    state.audit_logger.log(crate::security::AuditEvent {
        action: "start_flow".to_string(),
        username: auth.username.clone(),
        resource: format!("flow:{}", id),
        details: None,
    });
    Ok(StatusCode::NO_CONTENT)
}

pub async fn stop_flow(State(state): State<AppState>, Extension(_auth): Extension<AuthUser>, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    if let Some(engine) = &state.flow_engine {
        if let Some(mut flow) = engine.get_flow(&id).await {
            flow.enabled = false;
            engine.update_flow(flow).await;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_flow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let flow = state.storage.get_flow(&id).await?
        .ok_or_else(|| anyhow::anyhow!("Flow not found"))?;

    let nodes: Vec<crate::flow_engine::node::NodeConfig> = serde_json::from_str(&flow.nodes)
        .map_err(|e| anyhow::anyhow!("Invalid nodes: {}", e))?;

    let payload = req.get("payload").cloned().unwrap_or(serde_json::Value::Null);
    let mut msg = crate::flow_engine::message::Message::new(payload);
    let state_arc = crate::flow_engine::state::FlowState::new();
    let cache = state.cache.clone();

    let mut results = Vec::new();
    let mut output = msg.payload.clone();
    for node in &nodes {
        let ctx = crate::flow_engine::node::NodeContext {
            flow_id: flow.id.clone(),
            node_id: node_id(node),
            state: state_arc.clone(),
            storage: state.storage.clone(),
            cache: cache.clone(),
        };
        match node.process(msg, &ctx) {
            Ok(crate::flow_engine::node::NodeResult::Continue(new_msg)) => {
                output = new_msg.payload.clone();
                results.push(json!({
                    "node_id": node_id(node),
                    "result": "continue",
                    "output": output,
                }));
                msg = new_msg;
            }
            Ok(crate::flow_engine::node::NodeResult::FilterReject) => {
                results.push(json!({
                    "node_id": node_id(node),
                    "result": "filter_reject",
                }));
                break;
            }
            Err(e) => {
                results.push(json!({
                    "node_id": node_id(node),
                    "error": e,
                }));
                break;
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "output": output,
        "node_results": results,
    })))
}

pub async fn list_flow_runs(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    let runs = state.storage.list_flow_runs(&id, 20).await?;
    Ok(Json(json!({ "runs": runs })))
}

fn db_flow_to_runtime_flow(db_flow: Flow) -> Result<crate::flow_engine::engine::Flow, AppError> {
    let trigger: crate::flow_engine::trigger::TriggerConfig = serde_json::from_str(&db_flow.trigger_config)
        .map_err(|e| anyhow::anyhow!("Invalid trigger_config: {}", e))?;
    let nodes: Vec<crate::flow_engine::node::NodeConfig> = serde_json::from_str(&db_flow.nodes)
        .map_err(|e| anyhow::anyhow!("Invalid nodes: {}", e))?;
    Ok(crate::flow_engine::engine::Flow {
        id: db_flow.id,
        name: db_flow.name,
        enabled: db_flow.enabled,
        trigger,
        nodes,
    })
}

fn node_id(node: &crate::flow_engine::node::NodeConfig) -> String {
    use crate::flow_engine::node::NodeConfig;
    match node {
        NodeConfig::Filter { id, .. } => id.clone(),
        NodeConfig::Transform { id, .. } => id.clone(),
        NodeConfig::Threshold { id, .. } => id.clone(),
        NodeConfig::Lookup { id, .. } => id.clone(),
        NodeConfig::Expression { id, .. } => id.clone(),
        NodeConfig::DataSourceOut { id, .. } => id.clone(),
        NodeConfig::HttpOut { id, .. } => id.clone(),
        NodeConfig::AlarmOut { id, .. } => id.clone(),
        NodeConfig::WriteOut { id, .. } => id.clone(),
        NodeConfig::Script { id, .. } => id.clone(),
    }
}
