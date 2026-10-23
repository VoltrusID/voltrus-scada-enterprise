//! Voltrus Gateway client — routes SCADA device polling + writes through the
//! standalone `voltrus-gateway` gRPC binary instead of driving protocols inline.
//!
//! Phase 1 wires the Modbus path under the `gateway-modbus` feature:
//! - **Poll**: `PollerManager::start` calls [`connect_modbus`] then
//!   [`poll_via_gateway`], which subscribes to tag updates and writes them into
//!   the shared [`DeviceCache`] — so the rest of SCADA (cache, WebSocket
//!   broadcaster, expressions) is unchanged.
//! - **Write**: `command_queue::executor` calls [`write_modbus_register`], which
//!   opens a transient gateway session, writes, and disconnects. Writes are
//!   occasional control actions, so a per-call session is simple and correct.
//!
//! Tag values from the gateway are a rich `Value` oneof; SCADA collapses them to
//! `f64` here ([`value_to_f64`]) — the single place that flattening happens,
//! matching the inline poller's `metrics: HashMap<String, f64>` cache edge.
//!
//! The gateway endpoint is read from `VOLTRUS_GATEWAY_ADDR`
//! (default `http://127.0.0.1:50061`).

use std::collections::HashMap;

use tokio_stream::StreamExt;
use voltrus_gateway::pb::{self, connect_request::Protocol, tag_address::Address};

use crate::cache::DeviceCache;
use crate::models::{
    DeviceConfig, DeviceStatus, WriteResult,
    config_types::{Endianness, RegisterConfig, RegisterType},
};

/// Gateway gRPC endpoint, overridable via `VOLTRUS_GATEWAY_ADDR`.
pub fn gateway_endpoint() -> String {
    std::env::var("VOLTRUS_GATEWAY_ADDR").unwrap_or_else(|_| "http://127.0.0.1:50061".to_string())
}

/// A live gateway session for one device (used by the persistent poll loop).
pub struct GatewayHandle {
    pub session_id: String,
    pub client: voltrus_gateway::client::GatewayClient,
}

/// Map a SCADA Modbus `RegisterConfig` to a gateway `TagAddress`.
///
/// The gateway's `ModbusTagAddress.address` is the 0-based address on the wire, so a register's
/// `protocol_address()` is sent: a 5-digit reference is converted and a zero-based offset (0-65535) is kept.
pub fn register_to_tag(r: &RegisterConfig) -> pb::TagAddress {
    pb::TagAddress {
        address: Some(Address::Modbus(pb::ModbusTagAddress {
            name: r.name.clone(),
            address: r.protocol_address() as u32,
            reg_type: reg_type_str(&r.reg_type).into(),
            scale: r.scale,
            register_area: r.register_area.clone(),
        })),
    }
}

/// Open a gateway session to a Modbus device on an existing client.
pub async fn open_modbus_session(
    client: &mut voltrus_gateway::client::GatewayClient,
    config: &DeviceConfig,
) -> anyhow::Result<String> {
    let resp = client
        .open_session(pb::ConnectRequest {
            session_hint: config.id.clone(),
            protocol: Some(Protocol::Modbus(pb::ModbusConnectConfig {
                id: config.id.clone(),
                ip: config.ip.clone(),
                port: config.port as u32,
                unit_id: config.unit_id as u32,
                interval_ms: config.interval_ms,
                endianness: endianness_str(&config.endianness).into(),
                bind_addr: String::new(),
            })),
        })
        .await?;
    Ok(resp.session_id)
}

/// Connect a fresh client + open a session (used by the persistent poll path).
pub async fn connect_modbus(
    endpoint: &str,
    config: &DeviceConfig,
) -> anyhow::Result<GatewayHandle> {
    let mut client = voltrus_gateway::client::GatewayClient::connect(endpoint).await?;
    let session_id = open_modbus_session(&mut client, config).await?;
    Ok(GatewayHandle { session_id, client })
}

/// Subscribe to tag updates from the gateway and pump them into the SCADA
/// `DeviceCache`. Returns when the stream ends (gateway gone / disconnected),
/// marking the device offline.
///
/// `name_override` remaps a tag's proto name (as echoed by the gateway) to the
/// SCADA display name used as the cache key. Only EtherNet/IP needs this: its
/// `EnipTagAddress.name` is the CIP tag path (required for reads), while the
/// cache key must stay the display name. Modbus/OPC UA pass `None` (the proto
/// name already is the display name).
pub async fn poll_via_gateway(
    handle: GatewayHandle,
    device_id: String,
    tags: Vec<pb::TagAddress>,
    interval_ms: u64,
    cache: DeviceCache,
    name_override: Option<HashMap<String, String>>,
) {
    let GatewayHandle {
        session_id,
        mut client,
    } = handle;

    let stream = client
        .subscribe(pb::SubscribeRequest {
            session_id,
            tags,
            interval_ms: u32::try_from(interval_ms.max(10)).unwrap_or(1000),
        })
        .await;

    let mut stream = match stream {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(device_id = %device_id, error = %e, "gateway subscribe failed");
            cache.set_status(&device_id, DeviceStatus::Offline).await;
            return;
        }
    };

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(batch) => {
                let metrics: HashMap<String, f64> = batch
                    .values
                    .into_iter()
                    .filter_map(|tv| {
                        let raw = tv.tag.as_ref().and_then(|t| match &t.address {
                            Some(Address::Modbus(m)) => Some(m.name.clone()),
                            Some(Address::Opcua(m)) => Some(m.name.clone()),
                            Some(Address::Enip(m)) => Some(m.name.clone()),
                            Some(Address::S7(m)) => Some(m.name.clone()),
                            Some(Address::Dnp3(m)) => Some(m.name.clone()),
                            Some(Address::Bacnet(m)) => Some(m.name.clone()),
                            Some(Address::Mqtt(m)) => Some(m.name.clone()),
                            None => None,
                        })?;
                        let v = value_to_f64(tv.value?)?;
                        let key = name_override
                            .as_ref()
                            .and_then(|m| m.get(&raw))
                            .cloned()
                            .unwrap_or(raw);
                        Some((key, v))
                    })
                    .collect();
                if !metrics.is_empty() {
                    cache.update(&device_id, metrics, false).await;
                }
            }
            Err(e) => {
                tracing::warn!(device_id = %device_id, error = %e, "gateway stream error");
                break;
            }
        }
    }

    cache.set_status(&device_id, DeviceStatus::Offline).await;
}

/// Write a single Modbus register through the gateway via a transient session
/// (used by the command-queue write path).
pub async fn write_modbus_register(
    config: &DeviceConfig,
    register: &RegisterConfig,
    value: f64,
) -> WriteResult {
    use voltrus_gateway::pb::value::Kind;

    let endpoint = gateway_endpoint();
    let mut client = match voltrus_gateway::client::GatewayClient::connect(&endpoint).await {
        Ok(c) => c,
        Err(e) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("gateway connect: {e}")),
                latency_ms: 0,
            };
        }
    };

    let session_id = match open_modbus_session(&mut client, config).await {
        Ok(s) => s,
        Err(e) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("gateway open_session: {e}")),
                latency_ms: 0,
            };
        }
    };

    let req = pb::WriteTagRequest {
        session_id: session_id.clone(),
        tag: Some(register_to_tag(register)),
        value: Some(pb::Value {
            kind: Some(Kind::DoubleVal(value)),
        }),
    };

    let res = match client.write_tag(req).await {
        Ok(r) => r,
        Err(e) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("gateway write: {e}")),
                latency_ms: 0,
            };
        }
    };

    // Best-effort disconnect (transient session).
    let _ = client
        .disconnect(pb::DisconnectRequest { session_id })
        .await;

    WriteResult {
        success: res.success,
        device_response: None,
        error: if res.error.is_empty() {
            None
        } else {
            Some(res.error)
        },
        latency_ms: res.latency_ms as u64,
    }
}

fn write_err(msg: String) -> WriteResult {
    WriteResult {
        success: false,
        device_response: None,
        error: Some(msg),
        latency_ms: 0,
    }
}

// ── OPC UA ──

#[cfg(feature = "gateway-opcua")]
fn security_policy_str(p: &crate::models::OpcUaSecurityPolicy) -> &'static str {
    match p {
        crate::models::OpcUaSecurityPolicy::None => "none",
        crate::models::OpcUaSecurityPolicy::Basic256Sha256 => "basic256sha256",
    }
}

#[cfg(feature = "gateway-opcua")]
fn security_mode_str(m: &crate::models::OpcUaSecurityMode) -> &'static str {
    match m {
        crate::models::OpcUaSecurityMode::None => "none",
        crate::models::OpcUaSecurityMode::Sign => "sign",
        crate::models::OpcUaSecurityMode::SignAndEncrypt => "signandencrypt",
    }
}

/// Map a SCADA OPC-UA `OpcUaNodeConfig` to a gateway `TagAddress`.
#[cfg(feature = "gateway-opcua")]
pub fn node_to_tag(n: &crate::models::OpcUaNodeConfig) -> pb::TagAddress {
    pb::TagAddress {
        address: Some(Address::Opcua(pb::OpcUaTagAddress {
            node_id: n.node_id.clone(),
            name: n.name.clone(),
            data_type: n.data_type.clone(),
        })),
    }
}

/// Open a gateway session to an OPC-UA device on an existing client.
#[cfg(feature = "gateway-opcua")]
pub async fn open_opcua_session(
    client: &mut voltrus_gateway::client::GatewayClient,
    config: &crate::models::OpcUaDeviceConfig,
) -> anyhow::Result<String> {
    let resp = client
        .open_session(pb::ConnectRequest {
            session_hint: config.id.clone(),
            protocol: Some(Protocol::Opcua(pb::OpcUaConnectConfig {
                endpoint_url: config.url.clone(),
                security_policy: security_policy_str(&config.security_policy).into(),
                security_mode: security_mode_str(&config.security_mode).into(),
                username: config.username.clone().unwrap_or_default(),
                password: config.password.clone().unwrap_or_default(),
                poll_interval_ms: config.interval_ms,
            })),
        })
        .await?;
    Ok(resp.session_id)
}

/// Connect a fresh client + open an OPC-UA session (used by the poll path).
#[cfg(feature = "gateway-opcua")]
pub async fn connect_opcua(
    endpoint: &str,
    config: &crate::models::OpcUaDeviceConfig,
) -> anyhow::Result<GatewayHandle> {
    let mut client = voltrus_gateway::client::GatewayClient::connect(endpoint).await?;
    let session_id = open_opcua_session(&mut client, config).await?;
    Ok(GatewayHandle { session_id, client })
}

/// Write a single OPC-UA node through the gateway via a transient session
/// (used by the command-queue write path).
#[cfg(feature = "gateway-opcua")]
pub async fn write_opcua_node(
    config: &crate::models::OpcUaDeviceConfig,
    node_id: &str,
    value: f64,
) -> WriteResult {
    use voltrus_gateway::pb::value::Kind;
    let mut client =
        match voltrus_gateway::client::GatewayClient::connect(&gateway_endpoint()).await {
            Ok(c) => c,
            Err(e) => return write_err(format!("gateway connect: {e}")),
        };
    let session_id = match open_opcua_session(&mut client, config).await {
        Ok(s) => s,
        Err(e) => return write_err(format!("gateway open_session: {e}")),
    };
    let req = pb::WriteTagRequest {
        session_id: session_id.clone(),
        tag: Some(pb::TagAddress {
            address: Some(Address::Opcua(pb::OpcUaTagAddress {
                node_id: node_id.to_string(),
                name: String::new(),
                data_type: String::new(),
            })),
        }),
        value: Some(pb::Value {
            kind: Some(Kind::DoubleVal(value)),
        }),
    };
    let res = match client.write_tag(req).await {
        Ok(r) => r,
        Err(e) => return write_err(format!("gateway write: {e}")),
    };
    let _ = client
        .disconnect(pb::DisconnectRequest { session_id })
        .await;
    WriteResult {
        success: res.success,
        device_response: None,
        error: if res.error.is_empty() {
            None
        } else {
            Some(res.error)
        },
        latency_ms: res.latency_ms as u64,
    }
}

// ── EtherNet/IP ──

#[cfg(any(feature = "gateway-enip", feature = "gateway-s7"))]
fn split_host_port(url: &str, default_port: u32) -> (String, u32) {
    let url = url
        .strip_prefix("tcp://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let mut parts = url.splitn(2, ':');
    let host = parts.next().unwrap_or("").to_string();
    let port = parts
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default_port);
    (host, port)
}

/// Map a SCADA EtherNet/IP `EnipTagConfig` to a gateway `TagAddress`.
///
/// `EnipTagAddress.name` is the CIP tag path (the gateway reads by it); the
/// display name is recovered separately via `name_override` on the poll path.
#[cfg(feature = "gateway-enip")]
pub fn enip_tag_to_tag(t: &crate::ethernet_ip::EnipTagConfig) -> pb::TagAddress {
    pb::TagAddress {
        address: Some(Address::Enip(pb::EnipTagAddress {
            name: t.tag_name.clone(),
            data_type: t.data_type.clone(),
            element_count: 1,
            template_id: 0,
            assembly_offset: t.assembly_offset,
            assembly_length: t.assembly_length,
            ..Default::default()
        })),
    }
}

/// Build the gateway `ConnectRequest` for an EtherNet/IP device (pure, extracted
/// for testability). `assembly_instance > 0` selects implicit I/O at the gateway;
/// `0` keeps explicit CIP ReadTag mode.
#[cfg(feature = "gateway-enip")]
fn enip_connect_request(config: &crate::models::EnipDeviceConfig) -> pb::ConnectRequest {
    let (host, port) = split_host_port(&config.url, 44818);
    pb::ConnectRequest {
        session_hint: config.id.clone(),
        protocol: Some(Protocol::Enip(pb::EnipConnectConfig {
            id: config.id.clone(),
            host,
            port,
            slot: config.slot as u32,
            poll_interval_ms: config.interval_ms,
            assembly_instance: config.assembly_instance,
            ..Default::default()
        })),
    }
}

/// Open a gateway session to an EtherNet/IP device on an existing client.
#[cfg(feature = "gateway-enip")]
pub async fn open_enip_session(
    client: &mut voltrus_gateway::client::GatewayClient,
    config: &crate::models::EnipDeviceConfig,
) -> anyhow::Result<String> {
    let resp = client.open_session(enip_connect_request(config)).await?;
    Ok(resp.session_id)
}

/// Connect a fresh client + open an EtherNet/IP session (used by the poll path).
#[cfg(feature = "gateway-enip")]
pub async fn connect_enip(
    endpoint: &str,
    config: &crate::models::EnipDeviceConfig,
) -> anyhow::Result<GatewayHandle> {
    let mut client = voltrus_gateway::client::GatewayClient::connect(endpoint).await?;
    let session_id = open_enip_session(&mut client, config).await?;
    Ok(GatewayHandle { session_id, client })
}

/// Write a single EtherNet/IP tag through the gateway via a transient session
/// (used by the command-queue write path). Data type defaults to DINT, matching
/// the inline poller's write behavior.
///
/// Note: I/O-only devices (`assembly_instance > 0`) expose no CIP tag server, so
/// explicit writes to them fail at the gateway — reads dominate the I/O path.
#[cfg(feature = "gateway-enip")]
pub async fn write_enip_tag(
    config: &crate::models::EnipDeviceConfig,
    tag_name: &str,
    value: f64,
) -> WriteResult {
    use voltrus_gateway::pb::value::Kind;
    let mut client =
        match voltrus_gateway::client::GatewayClient::connect(&gateway_endpoint()).await {
            Ok(c) => c,
            Err(e) => return write_err(format!("gateway connect: {e}")),
        };
    let session_id = match open_enip_session(&mut client, config).await {
        Ok(s) => s,
        Err(e) => return write_err(format!("gateway open_session: {e}")),
    };
    let req = pb::WriteTagRequest {
        session_id: session_id.clone(),
        tag: Some(pb::TagAddress {
            address: Some(Address::Enip(pb::EnipTagAddress {
                name: tag_name.to_string(),
                data_type: "dint".into(),
                element_count: 1,
                template_id: 0,
                ..Default::default()
            })),
        }),
        value: Some(pb::Value {
            kind: Some(Kind::DoubleVal(value)),
        }),
    };
    let res = match client.write_tag(req).await {
        Ok(r) => r,
        Err(e) => return write_err(format!("gateway write: {e}")),
    };
    let _ = client
        .disconnect(pb::DisconnectRequest { session_id })
        .await;
    WriteResult {
        success: res.success,
        device_response: None,
        error: if res.error.is_empty() {
            None
        } else {
            Some(res.error)
        },
        latency_ms: res.latency_ms as u64,
    }
}

// ── Siemens S7 ──

#[cfg(feature = "gateway-s7")]
fn s7_area_str(a: &crate::s7::S7Area) -> &'static str {
    use crate::s7::S7Area::*;
    match a {
        DB => "db",
        Input => "input",
        Output => "output",
        Marker => "marker",
        Timer => "timer",
        Counter => "counter",
    }
}

#[cfg(feature = "gateway-s7")]
fn s7_data_type_str(d: &crate::s7::S7DataType) -> &'static str {
    use crate::s7::S7DataType::*;
    match d {
        Bool => "bool",
        Byte => "byte",
        Word => "word",
        DWord => "dword",
        Real => "real",
        String => "string",
    }
}

/// Map a SCADA S7 `S7Tag` to a gateway `TagAddress`.
#[cfg(feature = "gateway-s7")]
pub fn s7_tag_to_tag(t: &crate::s7::S7Tag) -> pb::TagAddress {
    // Fall back to "{area}_{offset}" (matching the inline poller) when the tag
    // has no display name, so the cache key stays stable.
    let name = t
        .name
        .clone()
        .unwrap_or_else(|| format!("{}_{}", s7_area_str(&t.area), t.offset));
    pb::TagAddress {
        address: Some(Address::S7(pb::S7TagAddress {
            area: s7_area_str(&t.area).into(),
            db_number: t.db_number as u32,
            offset: t.offset as u32,
            bit_offset: t.bit_offset as u32,
            data_type: s7_data_type_str(&t.data_type).into(),
            length: t.length as u32,
            name,
        })),
    }
}

/// Open a gateway session to an S7 device on an existing client.
#[cfg(feature = "gateway-s7")]
pub async fn open_s7_session(
    client: &mut voltrus_gateway::client::GatewayClient,
    config: &crate::s7::S7DeviceConfig,
) -> anyhow::Result<String> {
    let (host, port) = split_host_port(&config.url, 102);
    let resp = client
        .open_session(pb::ConnectRequest {
            session_hint: config.id.clone(),
            protocol: Some(Protocol::S7(pb::S7ConnectConfig {
                id: config.id.clone(),
                host,
                port,
                rack: config.rack as u32,
                slot: config.slot as u32,
                poll_interval_ms: config.interval_ms,
                ..Default::default()
            })),
        })
        .await?;
    Ok(resp.session_id)
}

/// Connect a fresh client + open an S7 session (used by the poll path).
#[cfg(feature = "gateway-s7")]
pub async fn connect_s7(
    endpoint: &str,
    config: &crate::s7::S7DeviceConfig,
) -> anyhow::Result<GatewayHandle> {
    let mut client = voltrus_gateway::client::GatewayClient::connect(endpoint).await?;
    let session_id = open_s7_session(&mut client, config).await?;
    Ok(GatewayHandle { session_id, client })
}

/// Write a single S7 tag through the gateway via a transient session (used by
/// the command-queue write path). Data type defaults to REAL, matching the
/// inline poller's write behavior.
#[cfg(feature = "gateway-s7")]
pub async fn write_s7_tag(
    config: &crate::s7::S7DeviceConfig,
    area: &str,
    db_number: u32,
    offset: u16,
    value: f64,
) -> WriteResult {
    use voltrus_gateway::pb::value::Kind;
    let mut client =
        match voltrus_gateway::client::GatewayClient::connect(&gateway_endpoint()).await {
            Ok(c) => c,
            Err(e) => return write_err(format!("gateway connect: {e}")),
        };
    let session_id = match open_s7_session(&mut client, config).await {
        Ok(s) => s,
        Err(e) => return write_err(format!("gateway open_session: {e}")),
    };
    let req = pb::WriteTagRequest {
        session_id: session_id.clone(),
        tag: Some(pb::TagAddress {
            address: Some(Address::S7(pb::S7TagAddress {
                area: area.to_string(),
                db_number,
                offset: offset as u32,
                bit_offset: 0,
                data_type: "real".into(),
                length: 1,
                name: String::new(),
            })),
        }),
        value: Some(pb::Value {
            kind: Some(Kind::DoubleVal(value)),
        }),
    };
    let res = match client.write_tag(req).await {
        Ok(r) => r,
        Err(e) => return write_err(format!("gateway write: {e}")),
    };
    let _ = client
        .disconnect(pb::DisconnectRequest { session_id })
        .await;
    WriteResult {
        success: res.success,
        device_response: None,
        error: if res.error.is_empty() {
            None
        } else {
            Some(res.error)
        },
        latency_ms: res.latency_ms as u64,
    }
}

// ── DNP3 ──

/// Map a SCADA DNP3 `Dnp3PointConfig` to a gateway `TagAddress`.
#[cfg(feature = "gateway-dnp3")]
pub fn dnp3_point_to_tag(p: &crate::models::Dnp3PointConfig) -> pb::TagAddress {
    pb::TagAddress {
        address: Some(Address::Dnp3(pb::Dnp3TagAddress {
            name: p.name.clone(),
            group: p.group_num as u32,
            variation: p.variation as u32,
            index: p.point_index as u32,
            scale: p.scale,
        })),
    }
}

/// Open a gateway session to a DNP3 outstation on an existing client.
#[cfg(feature = "gateway-dnp3")]
pub async fn open_dnp3_session(
    client: &mut voltrus_gateway::client::GatewayClient,
    config: &crate::models::Dnp3DeviceConfig,
) -> anyhow::Result<String> {
    let resp = client
        .open_session(pb::ConnectRequest {
            session_hint: config.id.clone(),
            protocol: Some(Protocol::Dnp3(pb::Dnp3ConnectConfig {
                id: config.id.clone(),
                host: config.ip.clone(),
                port: config.port as u32,
                master_address: config.master_address as u32,
                outstation_address: config.outstation_address as u32,
                poll_interval_ms: config.event_scan_interval_ms.unwrap_or(1000),
            })),
        })
        .await?;
    Ok(resp.session_id)
}

/// Connect a fresh client + open a DNP3 session (used by the poll path).
#[cfg(feature = "gateway-dnp3")]
pub async fn connect_dnp3(
    endpoint: &str,
    config: &crate::models::Dnp3DeviceConfig,
) -> anyhow::Result<GatewayHandle> {
    let mut client = voltrus_gateway::client::GatewayClient::connect(endpoint).await?;
    let session_id = open_dnp3_session(&mut client, config).await?;
    Ok(GatewayHandle { session_id, client })
}

/// Write (direct-operate binary output) a single DNP3 point through the gateway
/// via a transient session (used by the command-queue write path). Matches the
/// inline poller, which only supports binary control.
#[cfg(feature = "gateway-dnp3")]
pub async fn write_dnp3_point(
    config: &crate::models::Dnp3DeviceConfig,
    point_index: u16,
    value: f64,
) -> WriteResult {
    use voltrus_gateway::pb::value::Kind;
    let mut client =
        match voltrus_gateway::client::GatewayClient::connect(&gateway_endpoint()).await {
            Ok(c) => c,
            Err(e) => return write_err(format!("gateway connect: {e}")),
        };
    let session_id = match open_dnp3_session(&mut client, config).await {
        Ok(s) => s,
        Err(e) => return write_err(format!("gateway open_session: {e}")),
    };
    let req = pb::WriteTagRequest {
        session_id: session_id.clone(),
        tag: Some(pb::TagAddress {
            address: Some(Address::Dnp3(pb::Dnp3TagAddress {
                name: String::new(),
                group: 12,
                variation: 1,
                index: point_index as u32,
                scale: 1.0,
            })),
        }),
        value: Some(pb::Value {
            kind: Some(Kind::DoubleVal(value)),
        }),
    };
    let res = match client.write_tag(req).await {
        Ok(r) => r,
        Err(e) => return write_err(format!("gateway write: {e}")),
    };
    let _ = client
        .disconnect(pb::DisconnectRequest { session_id })
        .await;
    WriteResult {
        success: res.success,
        device_response: None,
        error: if res.error.is_empty() {
            None
        } else {
            Some(res.error)
        },
        latency_ms: res.latency_ms as u64,
    }
}

/// proto `Value` → `f64`. Non-numeric kinds (string, bytes) return `None`.
pub fn value_to_f64(v: pb::Value) -> Option<f64> {
    use voltrus_gateway::pb::value::Kind::*;
    match v.kind {
        Some(BoolVal(b)) => Some(if b { 1.0 } else { 0.0 }),
        Some(Int32Val(i)) => Some(i as f64),
        Some(Int64Val(i)) => Some(i as f64),
        Some(Uint32Val(u)) => Some(u as f64),
        Some(FloatVal(f)) => Some(f as f64),
        Some(DoubleVal(d)) => Some(d),
        Some(StringVal(_)) | Some(BytesVal(_)) => None,
        None => None,
    }
}

fn reg_type_str(rt: &RegisterType) -> &'static str {
    match rt {
        RegisterType::Int16 => "int16",
        RegisterType::Uint16 => "uint16",
        RegisterType::Int32 => "int32",
        RegisterType::Float32 => "float32",
        RegisterType::Float64 => "float64",
    }
}

fn endianness_str(e: &Endianness) -> &'static str {
    match e {
        Endianness::Big => "big",
        Endianness::Little => "little",
        Endianness::Swapped => "swapped",
    }
}

#[cfg(all(test, feature = "gateway-server-test"))]
mod tests {
    //! Runtime e2e: spin the gateway server + a Modbus slave in-process and drive
    //! the real SCADA glue (connect_modbus / poll_via_gateway /
    //! write_modbus_register) against a DeviceCache.

    use super::*;
    use crate::cache::DeviceCache;
    use crate::models::{DeviceConfig, Endianness, RegisterType};
    use std::time::Duration;
    use voltrus_gateway::pb;

    #[test]
    fn register_to_tag_sends_the_wire_address() {
        let wire = |address, zero_based| {
            let r = RegisterConfig {
                name: "r".into(),
                address,
                reg_type: RegisterType::Uint16,
                scale: 1.0,
                register_area: "holding".into(),
                zero_based,
            };
            match register_to_tag(&r).address {
                Some(Address::Modbus(m)) => m.address,
                other => panic!("not a Modbus tag: {other:?}"),
            }
        };
        assert_eq!(wire(40001, false), 0);
        assert_eq!(wire(40003, false), 2);
        assert_eq!(wire(12, false), 12);
        assert_eq!(wire(35000, true), 35000);
        assert_eq!(wire(65535, true), 65535);
    }

    const SLAVE_PORT: u16 = 15512;

    async fn serve_gateway() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(voltrus_gateway::server::GatewayServer::new().into_service())
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        format!("http://{addr}")
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn scada_poll_and_write_via_gateway() {
        // 1. Modbus slave with a preset register.
        let (state, _log_rx, _shutdown) =
            mt_modbus::start_slave_server(SLAVE_PORT, "127.0.0.1", 20)
                .await
                .unwrap();
        {
            let mut st = state.lock().unwrap();
            st.registers[0] = 7;
        }

        // 2. Gateway server.
        let endpoint = serve_gateway().await;
        // write_modbus_register reads the endpoint from the env.
        // SAFETY: single-threaded test setup before any reads.
        unsafe {
            std::env::set_var("VOLTRUS_GATEWAY_ADDR", endpoint.clone());
        }

        // 3. SCADA cache + device config.
        let cache = DeviceCache::new();
        let config = DeviceConfig {
            id: "gw-e2e".into(),
            ip: "127.0.0.1".into(),
            port: SLAVE_PORT,
            unit_id: 1,
            interval_ms: 100,
            endianness: Endianness::Big,
            registers: vec![RegisterConfig {
                name: "r0".into(),
                address: 0,
                reg_type: RegisterType::Uint16,
                scale: 1.0,
                register_area: "holding".into(),
                zero_based: false,
            }],
            readback: None,
            network: None,
        };

        // 4. Poll via gateway into the cache.
        let handle = connect_modbus(&endpoint, &config).await.unwrap();
        let cache_poll = cache.clone();
        let tags: Vec<pb::TagAddress> = config.registers.iter().map(register_to_tag).collect();
        let did = config.id.clone();
        tokio::spawn(async move {
            poll_via_gateway(handle, did, tags, 100, cache_poll, None).await;
        });

        // 5. Wait for the cache to populate.
        let mut populated = false;
        for _ in 0..40 {
            tokio::time::sleep(Duration::from_millis(50)).await;
            for snap in cache.get_all().await {
                if snap.device_id == "gw-e2e" {
                    if let Some(v) = snap.metrics.get("r0") {
                        if (*v - 7.0).abs() < 0.5 {
                            populated = true;
                        }
                    }
                }
            }
            if populated {
                break;
            }
        }
        assert!(populated, "cache not populated via gateway poll");

        // 6. Write via a transient gateway session, verify the slave image.
        let reg = RegisterConfig {
            name: "cmd".into(),
            address: 1,
            reg_type: RegisterType::Uint16,
            scale: 1.0,
            register_area: "holding".into(),
            zero_based: false,
        };
        let res = write_modbus_register(&config, &reg, 55.0).await;
        assert!(res.success, "write failed: {:?}", res.error);
        assert_eq!(state.lock().unwrap().registers[1], 55);
    }
}

// ── BACnet ──

/// Map a SCADA BACnet `BacnetObjectConfig` to a gateway `TagAddress`.
#[cfg(feature = "gateway-bacnet")]
pub fn bacnet_object_to_tag(o: &crate::models::BacnetObjectConfig) -> pb::TagAddress {
    pb::TagAddress {
        address: Some(Address::Bacnet(pb::BacnetTagAddress {
            name: o.name.clone(),
            object_type: o.object_type.clone(),
            object_instance: o.object_instance,
            property: o.property.clone(),
        })),
    }
}

/// Open a gateway session to a BACnet device on an existing client.
#[cfg(feature = "gateway-bacnet")]
pub async fn open_bacnet_session(
    client: &mut voltrus_gateway::client::GatewayClient,
    config: &crate::models::BacnetDeviceConfig,
) -> anyhow::Result<String> {
    let resp = client
        .open_session(pb::ConnectRequest {
            session_hint: config.id.clone(),
            protocol: Some(Protocol::Bacnet(pb::BacnetConnectConfig {
                id: config.id.clone(),
                ip: config.ip.clone(),
                port: config.port as u32,
                device_id: config.device_instance,
                poll_interval_ms: config.interval_ms,
            })),
        })
        .await?;
    Ok(resp.session_id)
}

/// Connect a fresh client + open a BACnet session (used by the poll path).
#[cfg(feature = "gateway-bacnet")]
pub async fn connect_bacnet(
    endpoint: &str,
    config: &crate::models::BacnetDeviceConfig,
) -> anyhow::Result<GatewayHandle> {
    let mut client = voltrus_gateway::client::GatewayClient::connect(endpoint).await?;
    let session_id = open_bacnet_session(&mut client, config).await?;
    Ok(GatewayHandle { session_id, client })
}

/// Write a single BACnet present-value through the gateway via a transient
/// session (used by the command-queue write path). Matches the inline poller,
/// which writes the object's `presentValue` property.
#[cfg(feature = "gateway-bacnet")]
pub async fn write_bacnet_point(
    config: &crate::models::BacnetDeviceConfig,
    object_type: &str,
    object_instance: u32,
    property: &str,
    value: f64,
) -> WriteResult {
    use voltrus_gateway::pb::value::Kind;
    let mut client =
        match voltrus_gateway::client::GatewayClient::connect(&gateway_endpoint()).await {
            Ok(c) => c,
            Err(e) => return write_err(format!("gateway connect: {e}")),
        };
    let session_id = match open_bacnet_session(&mut client, config).await {
        Ok(s) => s,
        Err(e) => return write_err(format!("gateway open_session: {e}")),
    };
    let req = pb::WriteTagRequest {
        session_id: session_id.clone(),
        tag: Some(pb::TagAddress {
            address: Some(Address::Bacnet(pb::BacnetTagAddress {
                name: String::new(),
                object_type: object_type.to_string(),
                object_instance,
                property: property.to_string(),
            })),
        }),
        value: Some(pb::Value {
            kind: Some(Kind::DoubleVal(value)),
        }),
    };
    let res = match client.write_tag(req).await {
        Ok(r) => r,
        Err(e) => return write_err(format!("gateway write: {e}")),
    };
    let _ = client
        .disconnect(pb::DisconnectRequest { session_id })
        .await;
    WriteResult {
        success: res.success,
        device_response: None,
        error: if res.error.is_empty() {
            None
        } else {
            Some(res.error)
        },
        latency_ms: res.latency_ms as u64,
    }
}

#[cfg(all(test, feature = "gateway-enip"))]
mod enip_assembly_tests {
    //! Unit: SCADA enip config -> gateway proto assembly fields propagate.
    use super::*;
    use crate::ethernet_ip::EnipTagConfig;
    use crate::models::EnipDeviceConfig;
    use voltrus_gateway::pb::{self, connect_request::Protocol, tag_address::Address};

    fn device(assembly_instance: u32) -> EnipDeviceConfig {
        EnipDeviceConfig {
            id: "enip_io".into(),
            url: "192.168.1.5:44818".into(),
            slot: 0,
            interval_ms: 100,
            assembly_instance,
            tags: Vec::new(),
        }
    }

    #[test]
    fn connect_request_carries_assembly_instance() {
        let proto = match enip_connect_request(&device(100)).protocol {
            Some(Protocol::Enip(p)) => p,
            _ => panic!("expected enip protocol"),
        };
        assert_eq!(proto.assembly_instance, 100);
        assert_eq!(proto.host, "192.168.1.5");
        assert_eq!(proto.port, 44818);
    }

    #[test]
    fn connect_request_explicit_read_mode_when_zero() {
        let proto = match enip_connect_request(&device(0)).protocol {
            Some(Protocol::Enip(p)) => p,
            _ => panic!("expected enip protocol"),
        };
        assert_eq!(proto.assembly_instance, 0, "0 = explicit CIP reads");
    }

    #[test]
    fn tag_to_tag_carries_assembly_offset_and_length() {
        let t = EnipTagConfig {
            tag_name: "Counter".into(),
            name: "Counter".into(),
            data_type: "dint".into(),
            assembly_offset: 4,
            assembly_length: 4,
        };
        let enip = match enip_tag_to_tag(&t).address {
            Some(Address::Enip(e)) => e,
            _ => panic!("expected enip address"),
        };
        assert_eq!(enip.name, "Counter");
        assert_eq!(enip.assembly_offset, 4);
        assert_eq!(enip.assembly_length, 4);
    }

    // Silence unused `pb` import if the compiler doesn't see a direct use.
    #[allow(dead_code)]
    fn _pb_anchor() -> pb::TagAddress {
        pb::TagAddress::default()
    }
}
