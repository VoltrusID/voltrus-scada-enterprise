//! BACnet/IP protocol client.
//!
//! Implements device discovery, object browsing, property reading,
//! and property writing for building automation devices.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::cache::DeviceCache;
use crate::models::{BacnetDeviceConfig, BacnetDiscoveredDevice, DeviceStatus, WriteResult};

use bacnet_rs::client::BacnetClient;
use bacnet_rs::datalink::{bip::BacnetIpDataLink, DataLink, DataLinkAddress};
use bacnet_rs::network::Npdu;
use bacnet_rs::object::{
    ObjectIdentifier, ObjectType, PropertyIdentifier,
};
use bacnet_rs::service::{
    ConfirmedServiceChoice, IAmRequest, PropertyReference, ReadAccessSpecification,
    ReadPropertyMultipleRequest, UnconfirmedServiceChoice, WhoIsRequest, WritePropertyRequest,
};
use bacnet_rs::vendor::get_vendor_name;

// ── Polling ──────────────────────────────────────────────────────────

struct Backoff {
    attempt: u32,
}

impl Backoff {
    fn new() -> Self {
        Self { attempt: 0 }
    }
    fn reset(&mut self) {
        self.attempt = 0;
    }
    fn next(&mut self) -> u64 {
        let delay = 1000 * 2u64.pow(self.attempt.min(6));
        self.attempt += 1;
        delay.min(60_000)
    }
}

pub async fn poll_bacnet_device(config: BacnetDeviceConfig, cache: DeviceCache) {
    let device_id = config.id.clone();
    let mut backoff = Backoff::new();

    loop {
        match try_poll_cycle(&config).await {
            Ok(metrics) => {
                backoff.reset();
                if !metrics.is_empty() {
                    cache.update(&device_id, metrics).await;
                }
            }
            Err(e) => {
                tracing::warn!("BACnet device {} error: {e}", device_id);
                cache.set_status(&device_id, DeviceStatus::Offline).await;
                let delay = backoff.next();
                tracing::info!("BACnet device {} retrying in {delay}ms", device_id);
                tokio::time::sleep(Duration::from_millis(delay)).await;
                continue;
            }
        }

        tokio::time::sleep(Duration::from_millis(config.interval_ms)).await;
    }
}

async fn try_poll_cycle(config: &BacnetDeviceConfig) -> Result<HashMap<String, f64>> {
    let objects: Vec<(String, ObjectType, u32, PropertyIdentifier)> = config
        .objects
        .iter()
        .filter_map(|o| {
            let ot = object_type_from_str(&o.object_type)?;
            let pid = property_id_from_str(&o.property);
            Some((o.name.clone(), ot, o.object_instance, pid))
        })
        .collect();

    if objects.is_empty() {
        return Ok(HashMap::new());
    }

    let addr = format!("{}:{}", config.ip, config.port)
        .parse::<SocketAddr>()
        .context("Invalid BACnet device address")?;

    // Group by object for RPM (read multiple properties per object)
    let mut read_specs = Vec::new();
    for (_, ot, inst, pid) in &objects {
        let obj_id = ObjectIdentifier::new(*ot, *inst);
        let prop_ref = PropertyReference::new(*pid);
        read_specs.push(ReadAccessSpecification::new(obj_id, vec![prop_ref]));
    }

    let rpm_request = ReadPropertyMultipleRequest::new(read_specs);

    let metrics = tokio::task::spawn_blocking(move || -> Result<HashMap<String, f64>> {
        let client = BacnetClient::new().map_err(|e| anyhow::anyhow!("BACnet client create failed: {e}"))?;

        // Send ReadPropertyMultiple
        let mut service_data = Vec::new();
        rpm_request.encode(&mut service_data)?;

        let invoke_id: u8 = 1;
        let response_data = client_send_confirmed(&client, addr, invoke_id, service_data)?;

        // Parse response - extract property values
        let mut result = HashMap::new();

        // Simple response parsing: scan for known tags
        // PropertyValue encoding: tag byte indicates type
        // 0x44 = Real (4 bytes), 0x55 = Double (8 bytes),
        // 0x21 = Boolean, 0x31 = Unsigned (1-4 bytes), 0x33 = Enum (1-4 bytes)
        let data = &response_data;
        let mut pos = 0;
        let mut obj_idx = 0;

        while pos < data.len() && obj_idx < objects.len() {
            // Look for property value tags
            if pos + 1 < data.len() {
                let tag = data[pos];
                let value = match tag {
                    0x44 if pos + 5 <= data.len() => {
                        // Real (f32)
                        let bytes: [u8; 4] = data[pos + 1..pos + 5].try_into().map_err(|_| anyhow::anyhow!("parse f32 bytes"))?;
                        Some(f32::from_be_bytes(bytes) as f64)
                    }
                    0x55 if pos + 9 <= data.len() => {
                        // Double (f64)
                        let bytes: [u8; 8] = data[pos + 1..pos + 9].try_into().map_err(|_| anyhow::anyhow!("parse f64 bytes"))?;
                        Some(f64::from_be_bytes(bytes))
                    }
                    0x22 if pos + 3 <= data.len() => {
                        // Unsigned 2-byte
                        Some(((data[pos + 1] as u32) << 8 | data[pos + 2] as u32) as f64)
                    }
                    0x21 if pos + 2 <= data.len() => {
                        // Unsigned 1-byte
                        Some(data[pos + 1] as f64)
                    }
                    0x24 if pos + 5 <= data.len() => {
                        // Unsigned 4-byte
                        let bytes: [u8; 4] = data[pos + 1..pos + 5].try_into().map_err(|_| anyhow::anyhow!("parse u32 bytes"))?;
                        Some(u32::from_be_bytes(bytes) as f64)
                    }
                    0x31 => {
                        // Boolean true
                        Some(1.0)
                    }
                    0x30 => {
                        // Boolean false
                        Some(0.0)
                    }
                    0x32 if pos + 3 <= data.len() => {
                        // Enum 2-byte
                        Some(((data[pos + 1] as u32) << 8 | data[pos + 2] as u32) as f64)
                    }
                    _ => None,
                };

                if let Some(v) = value {
                    let (name, _, _, _) = &objects[obj_idx];
                    result.insert(name.clone(), v);
                    obj_idx += 1;
                }
            }
            pos += 1;
        }

        Ok(result)
    })
    .await
    .context("Task join failed")??;

    Ok(metrics)
}

/// Send a confirmed request via the BacnetClient's internal method.
/// This mirrors the client's send_confirmed_request but works with our config.
fn client_send_confirmed(
    _client: &BacnetClient,
    target_addr: SocketAddr,
    _invoke_id: u8,
    service_data: Vec<u8>,
) -> Result<Vec<u8>> {
    use bacnet_rs::app::{Apdu, MaxApduSize, MaxSegments};

    let apdu = Apdu::ConfirmedRequest {
        segmented: false,
        more_follows: false,
        segmented_response_accepted: true,
        max_segments: MaxSegments::Unspecified,
        max_response_size: MaxApduSize::Up1476,
        invoke_id: 1,
        sequence_number: None,
        proposed_window_size: None,
        service_choice: ConfirmedServiceChoice::ReadPropertyMultiple,
        service_data,
    };

    let apdu_data = apdu.encode();
    let mut npdu = Npdu::new();
    npdu.control.expecting_reply = true;
    let npdu_data = npdu.encode();

    let mut message = npdu_data;
    message.extend_from_slice(&apdu_data);

    // BVLC Original-Unicast-NPDU
    let mut bvlc = vec![0x81, 0x0A, 0x00, 0x00];
    bvlc.extend_from_slice(&message);
    let len = bvlc.len() as u16;
    bvlc[2] = (len >> 8) as u8;
    bvlc[3] = (len & 0xFF) as u8;

    // Access the underlying socket from BacnetClient
    // BacnetClient wraps a UdpSocket, but doesn't expose it.
    // We need to create our own socket for sending.
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")
        .context("BACnet send socket bind failed")?;
    socket
        .send_to(&bvlc, target_addr)
        .context("BACnet send failed")?;

    let mut buf = [0u8; 1500];
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(6) {
        match socket.recv_from(&mut buf) {
            Ok((len, source)) if source == target_addr => {
                // Parse BVLC header
                if len < 4 || buf[0] != 0x81 {
                    continue;
                }
                // Skip BVLC (4 bytes) and NPDU
                let bvlc_len = ((buf[2] as usize) << 8) | buf[3] as usize;
                if len != bvlc_len {
                    continue;
                }
                let npdu_start = 4;
                if let Ok((_npdu, npdu_len)) = Npdu::decode(&buf[npdu_start..len]) {
                    let apdu_start = npdu_start + npdu_len;
                    if let Ok(Apdu::ComplexAck { service_data, .. }) =
                        Apdu::decode(&buf[apdu_start..len])
                    {
                        return Ok(service_data);
                    }
                }
            }
            Ok(_) => continue,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e).context("BACnet recv failed"),
        }
    }

    anyhow::bail!("BACnet ReadPropertyMultiple timeout for {}", target_addr)
}

// ── Discovery ────────────────────────────────────────────────────────

pub async fn discover_devices(
    timeout_secs: u64,
) -> Result<Vec<BacnetDiscoveredDevice>> {
    tokio::task::spawn_blocking(move || {
        let mut datalink =
            BacnetIpDataLink::new("0.0.0.0:0").context("BACnet datalink create failed")?;

        let whois = WhoIsRequest::new();
        let mut service_data = Vec::new();
        whois.encode(&mut service_data)?;

        let mut apdu = vec![0x10, UnconfirmedServiceChoice::WhoIs as u8];
        apdu.extend_from_slice(&service_data);

        let npdu = Npdu::global_broadcast();
        let mut message = npdu.encode();
        message.extend_from_slice(&apdu);

        datalink
            .send_frame(&message, &DataLinkAddress::Broadcast)
            .context("BACnet Who-Is broadcast failed")?;

        let mut discovered = Vec::new();
        let start = std::time::Instant::now();

        while start.elapsed() < Duration::from_secs(timeout_secs) {
            match datalink.receive_frame() {
                Ok((data, source)) => {
                    let source_addr = match source {
                        DataLinkAddress::Ip(addr) => addr,
                        _ => continue,
                    };

                    if let Some(device) = parse_iam_response(&data, source_addr) {
                        discovered.push(device);
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(discovered)
    })
    .await
    .context("Task join failed")?
}

fn parse_iam_response(data: &[u8], source: SocketAddr) -> Option<BacnetDiscoveredDevice> {
    if data.len() < 4 || data[0] != 0x81 {
        return None;
    }

    let bvlc_len = ((data[2] as usize) << 8) | data[3] as usize;
    if data.len() != bvlc_len {
        return None;
    }

    let npdu_start = 4;
    let (_npdu, npdu_len) = Npdu::decode(&data[npdu_start..]).ok()?;

    let apdu_start = npdu_start + npdu_len;
    let apdu = &data[apdu_start..];

    if apdu.len() < 2 || apdu[0] != 0x10 || apdu[1] != UnconfirmedServiceChoice::IAm as u8 {
        return None;
    }

    let iam = IAmRequest::decode(&apdu[2..]).ok()?;

    Some(BacnetDiscoveredDevice {
        device_instance: iam.device_identifier.instance,
        ip: source.ip().to_string(),
        port: source.port(),
        vendor_name: Some(
            get_vendor_name(iam.vendor_identifier)
                .unwrap_or("Unknown")
                .to_string(),
        ),
        model_name: None,
        firmware_version: None,
        object_list_count: None,
    })
}

// ── Browse ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct BacnetObjectInfo {
    pub object_type: String,
    pub object_instance: u32,
    pub object_name: Option<String>,
}

pub async fn browse_objects(
    ip: &str,
    port: u16,
    device_instance: u32,
) -> Result<Vec<BacnetObjectInfo>> {
    let addr = format!("{}:{}", ip, port)
        .parse::<SocketAddr>()
        .context("Invalid BACnet address")?;

    tokio::task::spawn_blocking(move || {
        let client = BacnetClient::new().map_err(|e| anyhow::anyhow!("BACnet client failed: {e}"))?;

        let obj_ids = client
            .read_object_list(addr, device_instance)
            .map_err(|e| anyhow::anyhow!("Read object list failed: {e}"))?;

        let objects = obj_ids
            .iter()
            .filter(|id| id.object_type != ObjectType::Device)
            .map(|id| BacnetObjectInfo {
                object_type: object_type_to_string(&id.object_type),
                object_instance: id.instance,
                object_name: None,
            })
            .collect();

        Ok(objects)
    })
    .await
    .context("Task join failed")?
}

// ── Write ────────────────────────────────────────────────────────────

pub async fn write_property(
    ip: &str,
    port: u16,
    object_type: &str,
    object_instance: u32,
    property: &str,
    value: f64,
) -> WriteResult {
    let start = std::time::Instant::now();
    let addr = format!("{}:{}", ip, port);
    let object_type = object_type.to_string();
    let property = property.to_string();

    let result = tokio::task::spawn_blocking(move || {
        let addr: SocketAddr = addr.parse().map_err(|e: std::net::AddrParseError| e.to_string())?;
        let ot = object_type_from_str(&object_type)
            .ok_or_else(|| format!("Unknown object type: {}", object_type))?;
        let pid = property_id_from_str(&property);

        let obj_id = ObjectIdentifier::new(ot, object_instance);

        // Encode property value as BACnet Real (tag 0x44 + 4-byte float)
        let mut property_value = vec![0x44];
        property_value.extend_from_slice(&(value as f32).to_be_bytes());

        let pid_u32: u32 = pid.into();
        let write_req = WritePropertyRequest::new(
            obj_id,
            pid_u32,
            property_value,
        );

        let mut service_data = Vec::new();
        write_req.encode(&mut service_data).map_err(|e| e.to_string())?;

        // Send via raw socket
        let socket = std::net::UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| format!("Socket bind: {e}"))?;

        use bacnet_rs::app::{Apdu, MaxApduSize, MaxSegments};
        let apdu = Apdu::ConfirmedRequest {
            segmented: false,
            more_follows: false,
            segmented_response_accepted: true,
            max_segments: MaxSegments::Unspecified,
            max_response_size: MaxApduSize::Up1476,
            invoke_id: 1,
            sequence_number: None,
            proposed_window_size: None,
            service_choice: ConfirmedServiceChoice::WriteProperty,
            service_data,
        };

        let apdu_data = apdu.encode();
        let mut npdu = Npdu::new();
        npdu.control.expecting_reply = true;
        let npdu_data = npdu.encode();

        let mut msg = npdu_data;
        msg.extend_from_slice(&apdu_data);

        let mut bvlc = vec![0x81, 0x0A, 0x00, 0x00];
        bvlc.extend_from_slice(&msg);
        let len = bvlc.len() as u16;
        bvlc[2] = (len >> 8) as u8;
        bvlc[3] = (len & 0xFF) as u8;

        socket
            .send_to(&bvlc, addr)
            .map_err(|e| format!("Send: {e}"))?;

        // Wait for ack
        let mut buf = [0u8; 1500];
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| format!("Timeout set: {e}"))?;

        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) if src == addr => {
                    if len < 4 || buf[0] != 0x81 {
                        continue;
                    }
                    let npdu_start = 4;
                    if let Ok((_npdu, npdu_len)) = Npdu::decode(&buf[npdu_start..len]) {
                        let apdu_start = npdu_start + npdu_len;
                        if let Ok(Apdu::SimpleAck { .. }) = Apdu::decode(&buf[apdu_start..len]) {
                            return Ok(());
                        }
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }

        Err("Write ack timeout".to_string())
    })
    .await;

    let latency_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(())) => WriteResult {
            success: true,
            device_response: Some("acked".to_string()),
            error: None,
            latency_ms,
        },
        Ok(Err(e)) => WriteResult {
            success: false,
            device_response: None,
            error: Some(e),
            latency_ms,
        },
        Err(e) => WriteResult {
            success: false,
            device_response: None,
            error: Some(e.to_string()),
            latency_ms,
        },
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn object_type_from_str(s: &str) -> Option<ObjectType> {
    match s.to_lowercase().as_str() {
        "analoginput" | "ai" => Some(ObjectType::AnalogInput),
        "analogoutput" | "ao" => Some(ObjectType::AnalogOutput),
        "analogvalue" | "av" => Some(ObjectType::AnalogValue),
        "binaryinput" | "bi" => Some(ObjectType::BinaryInput),
        "binaryoutput" | "bo" => Some(ObjectType::BinaryOutput),
        "binaryvalue" | "bv" => Some(ObjectType::BinaryValue),
        "multistateinput" | "mi" => Some(ObjectType::MultiStateInput),
        "multistateoutput" | "mo" => Some(ObjectType::MultiStateOutput),
        "multistatevalue" | "mv" => Some(ObjectType::MultiStateValue),
        _ => None,
    }
}

fn object_type_to_string(ot: &ObjectType) -> String {
    match ot {
        ObjectType::AnalogInput => "analogInput".to_string(),
        ObjectType::AnalogOutput => "analogOutput".to_string(),
        ObjectType::AnalogValue => "analogValue".to_string(),
        ObjectType::BinaryInput => "binaryInput".to_string(),
        ObjectType::BinaryOutput => "binaryOutput".to_string(),
        ObjectType::BinaryValue => "binaryValue".to_string(),
        ObjectType::MultiStateInput => "multiStateInput".to_string(),
        ObjectType::MultiStateOutput => "multiStateOutput".to_string(),
        ObjectType::MultiStateValue => "multiStateValue".to_string(),
        ObjectType::Device => "device".to_string(),
        _ => format!("{:?}", ot),
    }
}

fn property_id_from_str(s: &str) -> PropertyIdentifier {
    match s.to_lowercase().as_str() {
        "presentvalue" | "present_value" => PropertyIdentifier::PresentValue,
        "description" => PropertyIdentifier::Description,
        "objectname" | "object_name" => PropertyIdentifier::ObjectName,
        "statusflags" | "status_flags" => PropertyIdentifier::StatusFlags,
        "units" => PropertyIdentifier::Units,
        "objectlist" | "object_list" => PropertyIdentifier::ObjectList,
        "objectidentifier" | "object_identifier" => PropertyIdentifier::ObjectIdentifier,
        "objecttype" | "object_type" => PropertyIdentifier::ObjectType,
        "maxapdulengthaccepted" => PropertyIdentifier::MaxApduLengthAccepted,
        "segmentationsupported" => PropertyIdentifier::SegmentationSupported,
        "vendorname" | "vendor_name" => PropertyIdentifier::VendorName,
        "vendoridentifier" | "vendor_identifier" => PropertyIdentifier::VendorIdentifier,
        "modelname" | "model_name" => PropertyIdentifier::ModelName,
        "firmwarerevision" | "firmware_revision" => PropertyIdentifier::FirmwareRevision,
        _ => PropertyIdentifier::PresentValue,
    }
}
