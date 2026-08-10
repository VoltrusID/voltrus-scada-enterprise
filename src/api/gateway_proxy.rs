//! HTTP proxy exposing `voltrus-gateway` discovery / browse / info to the web UI.
//!
//! Gated behind the `gateway-client` cargo feature: default builds compile none
//! of this. The gateway endpoint is resolved by [`resolve_endpoint`] — persisted
//! DB setting (`gateway_endpoint` in the shared `settings` table) → `VOLTRUS_GATEWAY_ADDR`
//! env → default `http://127.0.0.1:50061`. The UI sets the DB value via `PUT /endpoint`.
//!
//! Imported devices are NOT gateway-specific: Browse returns typed tags the UI maps
//! into the normal per-protocol create payloads (`/api/v1/modbus-devices`,
//! `/api/v1/opcua/devices`), so the existing storage + poller + cache pipeline
//! serves them unchanged.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use voltrus_gateway::pb::{
    self, connect_request::Protocol, discover_request::Protocol as DiscoverProto,
};

use crate::api::{AppError, AppState};
use crate::gateway_client::gateway_endpoint;

const SETTING_KEY: &str = "gateway_endpoint";

/// Resolve the gateway endpoint: DB setting → env → built-in default.
/// Returns the endpoint string plus where it came from (`"db"` | `"env"` | `"default"`).
async fn resolve_endpoint(state: &AppState) -> (String, &'static str) {
    if let Ok(rows) = state.storage.get_settings(&[SETTING_KEY]).await {
        if let Some(row) = rows.into_iter().next() {
            if !row.value.trim().is_empty() {
                return (row.value, "db");
            }
        }
    }
    if let Ok(env) = std::env::var("VOLTRUS_GATEWAY_ADDR") {
        if !env.trim().is_empty() {
            return (env, "env");
        }
    }
    (gateway_endpoint(), "default")
}

async fn connect_gateway(endpoint: &str) -> anyhow::Result<voltrus_gateway::client::GatewayClient> {
    Ok(voltrus_gateway::client::GatewayClient::connect(endpoint).await?)
}

// ── GET /api/v1/gateway/endpoint ──────────────────────────────────────────────
pub async fn get_endpoint(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let (endpoint, source) = resolve_endpoint(&state).await;
    Ok(Json(json!({ "endpoint": endpoint, "source": source })))
}

// ── PUT /api/v1/gateway/endpoint ──────────────────────────────────────────────
#[derive(serde::Deserialize)]
pub struct SetEndpointBody {
    pub endpoint: String,
}

pub async fn set_endpoint(
    State(state): State<AppState>,
    Json(body): Json<SetEndpointBody>,
) -> Result<impl IntoResponse, AppError> {
    let endpoint = body.endpoint.trim();
    if endpoint.is_empty() {
        return Err(anyhow::anyhow!("endpoint must not be empty").into());
    }
    state.storage.upsert_setting(SETTING_KEY, endpoint).await?;
    Ok(Json(json!({ "ok": true })))
}

// ── GET /api/v1/gateway/info ──────────────────────────────────────────────────
pub async fn get_info(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let (endpoint, _) = resolve_endpoint(&state).await;
    let mut client = match connect_gateway(&endpoint).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Json(json!({
                "endpoint": endpoint,
                "reachable": false,
                "error": format!("{e:#}"),
            })));
        }
    };
    match client.get_info(pb::GetInfoRequest {}).await {
        Ok(info) => Ok(Json(json!({
            "endpoint": endpoint,
            "reachable": true,
            "version": info.version,
            "build_profile": info.build_profile,
            "uptime_ms": info.uptime_ms,
        }))),
        Err(e) => Ok(Json(json!({
            "endpoint": endpoint,
            "reachable": false,
            "error": e.to_string(),
        }))),
    }
}

// ── POST /api/v1/gateway/discover ─────────────────────────────────────────────
// Proto only supports modbus + enip discovery.
#[derive(serde::Deserialize)]
pub struct DiscoverBody {
    pub protocol: String, // "modbus" | "enip"
    pub ip: String,
    pub port: Option<u32>,
    pub unit_id: Option<u32>, // modbus
    pub slot: Option<u32>,    // enip
    pub timeout_ms: Option<u32>,
}

pub async fn discover(
    State(state): State<AppState>,
    Json(body): Json<DiscoverBody>,
) -> Result<impl IntoResponse, AppError> {
    let (endpoint, _) = resolve_endpoint(&state).await;
    let mut client = connect_gateway(&endpoint).await?;
    let timeout = body.timeout_ms.unwrap_or(2000);
    let proto = match body.protocol.as_str() {
        "modbus" => DiscoverProto::Modbus(pb::ModbusConnectConfig {
            id: "discover".into(),
            ip: body.ip,
            port: body.port.unwrap_or(502),
            unit_id: body.unit_id.unwrap_or(1),
            interval_ms: 1000,
            endianness: "big".into(),
            bind_addr: String::new(),
        }),
        "enip" => DiscoverProto::Enip(pb::EnipConnectConfig {
            id: "discover".into(),
            host: body.ip,
            port: body.port.unwrap_or(44818),
            slot: body.slot.unwrap_or(0),
            poll_interval_ms: 1000,
            assembly_instance: 0,
        }),
        other => return Err(anyhow::anyhow!("discover not supported for '{other}' (modbus/enip only)").into()),
    };
    let resp = client
        .discover(pb::DiscoverRequest {
            protocol: Some(proto),
            timeout_ms: timeout,
        })
        .await?;
    let devices: Vec<Value> = resp
        .devices
        .into_iter()
        .map(|d| {
            json!({
                "protocol": d.protocol,
                "address": d.address,
                "identity": d.identity,
            })
        })
        .collect();
    Ok(Json(json!({ "devices": devices })))
}
// ── POST /api/v1/gateway/scan ────────────────────────────────────────────────
// Modbus has no tag directory, so BrowseTags returns empty for it. A scan reads
// a register range as a candidate type (stepping by the type's word size so
// float32/int32 don't overlap) and reports every register that responds with a
// value — letting the user pick real registers (with their live values) to import.
#[derive(serde::Deserialize)]
pub struct ScanBody {
    pub ip: String,
    pub port: Option<u32>,
    pub unit_id: Option<u32>,
    pub area: Option<String>,     // holding | input | coil | discrete (default holding)
    pub reg_type: Option<String>, // float32 | uint16 | int16 | int32 | uint32 | float64
    pub start: Option<u32>,
    pub count: Option<u32>, // number of tags (not registers); capped at 200
}

fn type_step(reg_type: &str) -> u32 {
    match reg_type {
        "int16" | "uint16" => 1,
        "int32" | "uint32" | "float32" => 2,
        "float64" => 4,
        _ => 1,
    }
}

pub async fn scan(
    State(state): State<AppState>,
    Json(body): Json<ScanBody>,
) -> Result<impl IntoResponse, AppError> {
    let (endpoint, _) = resolve_endpoint(&state).await;
    let mut client = connect_gateway(&endpoint).await?;
    let connect = pb::ConnectRequest {
        session_hint: "scan".into(),
        protocol: Some(Protocol::Modbus(pb::ModbusConnectConfig {
            id: "scan".into(),
            ip: body.ip,
            port: body.port.unwrap_or(502),
            unit_id: body.unit_id.unwrap_or(1),
            interval_ms: 1000,
            endianness: "big".into(),
            bind_addr: String::new(),
        })),
    };
    let session_id = client.open_session(connect).await?.session_id;

    let area = body.area.unwrap_or_else(|| "holding".into());
    let reg_type = body.reg_type.unwrap_or_else(|| "float32".into());
    let step = type_step(&reg_type);
    let start = body.start.unwrap_or(0);
    let count = body.count.unwrap_or(50).min(200);

    let tags: Vec<pb::TagAddress> = (0..count)
        .map(|i| {
            let addr = start + i * step;
            pb::TagAddress {
                address: Some(pb::tag_address::Address::Modbus(pb::ModbusTagAddress {
                    name: format!("reg_{addr}"),
                    address: addr,
                    reg_type: reg_type.clone(),
                    scale: 1.0,
                    register_area: area.clone(),
                })),
            }
        })
        .collect();

    // Sub-batch the reads: a transport reset / panic on one register only drops
    // its chunk (each chunk is a separate gRPC stream over the reused channel),
    // so the scan still returns the registers that responded.
    let mut values: Vec<pb::TagValue> = Vec::new();
    for chunk in tags.chunks(8) {
        match client
            .read_tags(pb::ReadTagsRequest {
                session_id: session_id.clone(),
                tags: chunk.to_vec(),
            })
            .await
        {
            Ok(r) => values.extend(r.values),
            Err(e) => tracing::warn!(error = %e, "gateway scan sub-batch failed"),
        }
    }
    let _ = client
        .disconnect(pb::DisconnectRequest { session_id })
        .await;

    let out: Vec<Value> = values
        .into_iter()
        .filter_map(|tv| {
            if !tv.error.is_empty() {
                return None;
            }
            let m = match tv.tag.and_then(|t| t.address) {
                Some(pb::tag_address::Address::Modbus(m)) => m,
                _ => return None,
            };
            let value = tv.value.and_then(crate::gateway_client::value_to_f64)?;
            Some(json!({
                "protocol": "modbus",
                "name": m.name,
                "address": m.address,
                "data_type": reg_type,
                "register_area": area,
                "scale": 1.0,
                "value": value,
            }))
        })
        .collect();
    Ok(Json(json!({ "tags": out })))
}


// ── POST /api/v1/gateway/browse ───────────────────────────────────────────────
#[derive(serde::Deserialize)]
pub struct BrowseBody {
    pub protocol: String, // "modbus" | "opcua"
    pub params: Value,
}

pub async fn browse(
    State(state): State<AppState>,
    Json(body): Json<BrowseBody>,
) -> Result<impl IntoResponse, AppError> {
    let (endpoint, _) = resolve_endpoint(&state).await;
    let mut client = connect_gateway(&endpoint).await?;
    let connect = build_connect_request(&body.protocol, body.params)?;
    let session_id = client.open_session(connect).await?.session_id;

    let tags = client
        .browse_tags(pb::BrowseTagsRequest {
            session_id: session_id.clone(),
        })
        .await?
        .tags;

    let _ = client
        .disconnect(pb::DisconnectRequest { session_id })
        .await;

    let out: Vec<Value> = tags.into_iter().map(tag_to_json).collect();
    Ok(Json(json!({ "tags": out })))
}


/// Build a `ConnectRequest` for the browse call from a flat JSON params object.
fn build_connect_request(protocol: &str, p: Value) -> Result<pb::ConnectRequest, AppError> {
    fn s(p: &Value, k: &str, dflt: &str) -> String {
        p.get(k).and_then(|v| v.as_str()).unwrap_or(dflt).to_string()
    }
    fn u32v(p: &Value, k: &str, dflt: u32) -> u32 {
        p.get(k).and_then(|v| v.as_u64()).unwrap_or(dflt as u64) as u32
    }
    fn u64v(p: &Value, k: &str, dflt: u64) -> u64 {
        p.get(k).and_then(|v| v.as_u64()).unwrap_or(dflt)
    }

    let proto = match protocol {
        "modbus" => Protocol::Modbus(pb::ModbusConnectConfig {
            id: s(&p, "id", "browse"),
            ip: s(&p, "ip", ""),
            port: u32v(&p, "port", 502),
            unit_id: u32v(&p, "unit_id", 1),
            interval_ms: u64v(&p, "interval_ms", 1000),
            endianness: s(&p, "endianness", "big"),
            bind_addr: String::new(),
        }),
        "opcua" => Protocol::Opcua(pb::OpcUaConnectConfig {
            endpoint_url: s(&p, "endpoint_url", ""),
            security_policy: s(&p, "security_policy", "None"),
            security_mode: s(&p, "security_mode", "None"),
            username: s(&p, "username", ""),
            password: s(&p, "password", ""),
            poll_interval_ms: u64v(&p, "poll_interval_ms", 1000),
        }),
        other => return Err(anyhow::anyhow!("unsupported browse protocol '{other}' (modbus/opcua)").into()),
    };
    Ok(pb::ConnectRequest {
        session_hint: "browse".into(),
        protocol: Some(proto),
    })
}

/// Flatten a gateway `TagAddress` into UI-friendly JSON. Carries the per-protocol
/// fields the import step needs + a normalized `data_type` for display.
fn tag_to_json(tag: pb::TagAddress) -> Value {
    let address = tag.address;
    match address {
        Some(pb::tag_address::Address::Modbus(m)) => json!({
            "protocol": "modbus",
            "name": m.name,
            "address": m.address,
            "data_type": m.reg_type,
            "scale": m.scale,
            "register_area": m.register_area,
        }),
        Some(pb::tag_address::Address::Opcua(m)) => json!({
            "protocol": "opcua",
            "name": m.name,
            "node_id": m.node_id,
            "data_type": m.data_type,
        }),
        Some(pb::tag_address::Address::Enip(m)) => json!({
            "protocol": "enip",
            "name": m.name,
            "data_type": m.data_type,
            "element_count": m.element_count,
        }),
        Some(pb::tag_address::Address::S7(m)) => json!({
            "protocol": "s7",
            "name": m.name,
            "area": m.area,
            "db_number": m.db_number,
            "offset": m.offset,
            "data_type": m.data_type,
        }),
        Some(pb::tag_address::Address::Dnp3(m)) => json!({
            "protocol": "dnp3",
            "name": m.name,
            "group": m.group,
            "variation": m.variation,
            "index": m.index,
            "scale": m.scale,
        }),
        Some(pb::tag_address::Address::Bacnet(m)) => json!({
            "protocol": "bacnet",
            "name": m.name,
            "object_type": m.object_type,
            "object_instance": m.object_instance,
            "property": m.property,
        }),
        Some(pb::tag_address::Address::Mqtt(m)) => json!({
            "protocol": "mqtt",
            "name": m.name,
            "topic": m.topic,
            "qos": m.qos,
        }),
        None => json!({}),
    }
}
