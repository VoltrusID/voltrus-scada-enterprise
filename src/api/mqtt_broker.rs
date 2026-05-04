use axum::extract::{Extension, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use axum::response::IntoResponse;

use crate::api::{AppError, AppState, AuthUser};
use crate::mqtt_broker::{self, BrokerStatus, MqttBrokerState};

/// GET /api/v1/mqtt-broker/status
pub async fn get_status(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.mqtt_broker() {
        return Err(AppError(anyhow::anyhow!(
            "Embedded MQTT broker requires Starter tier."
        )));
    }

    let broker = state.mqtt_broker_state.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("MQTT broker module not initialized")))?;

    let status = broker.current_status().await;
    let clients = broker.clients.read().await;

    Ok(Json(json!({
        "status": match &status {
            BrokerStatus::Running => "running",
            BrokerStatus::Stopped => "stopped",
            BrokerStatus::Starting => "starting",
            BrokerStatus::Error(msg) => msg,
        },
        "connected_clients": clients.len(),
        "clients": clients.as_slice(),
        "config": match broker.config.read().await.as_ref() {
            Some(c) => json!({
                "port": c.port,
                "max_connections": c.max_connections,
                "auth_enabled": c.auth.mode != "none" && !c.auth.users.is_empty(),
            }),
            None => serde_json::Value::Null,
        },
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateMqttConfigRequest {
    pub port: Option<u16>,
    pub max_connections: Option<u32>,
    pub auth: Option<MqttAuthUpdate>,
}

#[derive(Debug, Deserialize)]
pub struct MqttAuthUpdate {
    pub enabled: Option<bool>,
    pub users: Option<Vec<MqttUserUpdate>>,
}

#[derive(Debug, Deserialize)]
pub struct MqttUserUpdate {
    pub username: String,
    pub password: String,
}

/// PUT /api/v1/mqtt-broker/config
pub async fn update_config(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Json(req): Json<UpdateMqttConfigRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.mqtt_broker() {
        return Err(AppError(anyhow::anyhow!(
            "Embedded MQTT broker requires Starter tier."
        )));
    }

    let broker = state.mqtt_broker_state.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("MQTT broker module not initialized")))?;

    if broker.current_status().await == BrokerStatus::Running {
        return Err(AppError(anyhow::anyhow!(
            "Stop the broker before changing configuration"
        )));
    }

    {
        let mut cfg_guard = broker.config.write().await;
        let mut cfg = cfg_guard.clone().unwrap_or_default();

        if let Some(port) = req.port {
            if port == 0 {
                return Err(AppError(anyhow::anyhow!("Port must be > 0")));
            }
            cfg.port = port;
        }
        if let Some(max_conn) = req.max_connections {
            if max_conn == 0 {
                return Err(AppError(anyhow::anyhow!("max_connections must be > 0")));
            }
            cfg.max_connections = max_conn;
        }
        if let Some(auth_update) = req.auth {
            if let Some(enabled) = auth_update.enabled {
                cfg.auth.mode = if enabled { "password".to_string() } else { "none".to_string() };
            }
            if let Some(users) = auth_update.users {
                cfg.auth.users = users.into_iter().map(|u| crate::models::MqttUser {
                    username: u.username,
                    password: u.password,
                }).collect();
            }
        }

        cfg.enabled = true;
        *cfg_guard = Some(cfg);
    }

    let cfg = broker.config.read().await;
    Ok(Json(json!({
        "message": "Configuration updated. Start the broker to apply changes.",
        "config": match cfg.as_ref() {
            Some(c) => json!({
                "port": c.port,
                "max_connections": c.max_connections,
                "auth_enabled": c.auth.mode != "none" && !c.auth.users.is_empty(),
            }),
            None => serde_json::Value::Null,
        },
    })))
}

/// POST /api/v1/mqtt-broker/start
pub async fn start_broker(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.mqtt_broker() {
        return Err(AppError(anyhow::anyhow!(
            "Embedded MQTT broker requires Starter tier."
        )));
    }

    let broker = state.mqtt_broker_state.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("MQTT broker module not initialized")))?;

    let config = broker.config.read().await.clone()
        .ok_or_else(|| AppError(anyhow::anyhow!(
            "No MQTT broker configuration. Set config via PUT /api/v1/mqtt-broker/config first."
        )))?;

    mqtt_broker::start_broker(broker.clone(), config).await
        .map_err(|e| AppError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(json!({
        "message": "MQTT broker started",
        "status": "running",
    })))
}

/// POST /api/v1/mqtt-broker/stop
pub async fn stop_broker(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.mqtt_broker() {
        return Err(AppError(anyhow::anyhow!(
            "Embedded MQTT broker requires Starter tier."
        )));
    }

    let broker = state.mqtt_broker_state.as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("MQTT broker module not initialized")))?;

    mqtt_broker::stop_broker(broker.clone()).await
        .map_err(|e| AppError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(json!({
        "message": "MQTT broker stopped",
        "status": "stopped",
    })))
}
