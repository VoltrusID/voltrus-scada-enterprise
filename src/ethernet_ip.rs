//! Allen-Bradley EtherNet/IP + CIP protocol client.
//!
//! Implements session registration, tag read/write, and tag browsing
//! for ControlLogix/CompactLogix PLCs over TCP port 44818.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::cache::DeviceCache;
use crate::models::{DeviceStatus, EnipDeviceConfig};

// ── EtherNet/IP command codes ──

const CMD_REGISTER_SESSION: u16 = 0x0065;
const CMD_UNREGISTER_SESSION: u16 = 0x0066;
const CMD_SEND_RR_DATA: u16 = 0x006F;
const _CMD_SEND_UNIT_DATA: u16 = 0x0070;

// ── CIP service codes ──

const CIP_READ_TAG: u8 = 0x4C;
const CIP_READ_TAG_FRAGMENTED: u8 = 0x52;
const CIP_WRITE_TAG: u8 = 0x4D;
const CIP_GET_ATTRIBUTES_ALL: u8 = 0x01;
const _CIP_FORWARD_OPEN: u8 = 0x54;

// ── CIP data type codes ──

const CIP_TYPE_BOOL: u8 = 0xC1;
const CIP_TYPE_SINT: u8 = 0xC2;
const CIP_TYPE_INT: u8 = 0xC3;
const CIP_TYPE_DINT: u8 = 0xC4;
const CIP_TYPE_REAL: u8 = 0xCA;
const CIP_TYPE_STRING: u8 = 0xD0;
const CIP_TYPE_DWORD: u8 = 0xD3;
const CIP_TYPE_LINT: u8 = 0xC5;

// ── CIP status codes ──

const CIP_STATUS_SUCCESS: u8 = 0x00;
const CIP_STATUS_PARTIAL: u8 = 0x06;

// ── Encapsulation header size ──

const ENIP_HEADER_SIZE: usize = 24;

// ── CIP path for Symbol class ──

const SYMBOL_CLASS_ID: u8 = 0x6B;

// ── Public types ──

#[derive(Debug, Clone, PartialEq)]
pub enum CipDataType {
    Bool,
    Sint,
    Int,
    Dint,
    Real,
    String,
    Dword,
    Lint,
}

impl CipDataType {
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            CIP_TYPE_BOOL => Some(Self::Bool),
            CIP_TYPE_SINT => Some(Self::Sint),
            CIP_TYPE_INT => Some(Self::Int),
            CIP_TYPE_DINT => Some(Self::Dint),
            CIP_TYPE_REAL => Some(Self::Real),
            CIP_TYPE_STRING => Some(Self::String),
            CIP_TYPE_DWORD => Some(Self::Dword),
            CIP_TYPE_LINT => Some(Self::Lint),
            _ => None,
        }
    }

    pub fn code(&self) -> u8 {
        match self {
            Self::Bool => CIP_TYPE_BOOL,
            Self::Sint => CIP_TYPE_SINT,
            Self::Int => CIP_TYPE_INT,
            Self::Dint => CIP_TYPE_DINT,
            Self::Real => CIP_TYPE_REAL,
            Self::String => CIP_TYPE_STRING,
            Self::Dword => CIP_TYPE_DWORD,
            Self::Lint => CIP_TYPE_LINT,
        }
    }

    pub fn element_size(&self) -> u16 {
        match self {
            Self::Bool => 1,
            Self::Sint => 1,
            Self::Int => 2,
            Self::Dint => 4,
            Self::Real => 4,
            Self::Dword => 4,
            Self::Lint => 8,
            Self::String => 88, // CIP STRING struct: 4 bytes len + 82 bytes data + 2 pad
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bool => "BOOL",
            Self::Sint => "SINT",
            Self::Int => "INT",
            Self::Dint => "DINT",
            Self::Real => "REAL",
            Self::String => "STRING",
            Self::Dword => "DWORD",
            Self::Lint => "LINT",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CipValue {
    Bool(bool),
    Sint(i8),
    Int(i16),
    Dint(i32),
    Real(f32),
    String(String),
    Bytes(Vec<u8>),
}

impl CipValue {
    /// Convert CIP value to f64 for cache storage.
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Self::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Self::Sint(v) => Some(*v as f64),
            Self::Int(v) => Some(*v as f64),
            Self::Dint(v) => Some(*v as f64),
            Self::Real(v) => Some(*v as f64),
            Self::Bytes(_) => None,
            Self::String(_) => None,
        }
    }

    /// Get a display-friendly string representation.
    pub fn display_value(&self) -> String {
        match self {
            Self::Bool(b) => b.to_string(),
            Self::Sint(v) => v.to_string(),
            Self::Int(v) => v.to_string(),
            Self::Dint(v) => v.to_string(),
            Self::Real(v) => format!("{:.4}", v),
            Self::String(s) => s.clone(),
            Self::Bytes(b) => format!("{:02x?}", b),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CipTag {
    pub name: String,
    pub data_type: CipDataType,
    pub element_count: u16,
}

#[derive(Debug)]
pub struct EthernetIpClient {
    stream: Option<TcpStream>,
    url: String,
    session_handle: u32,
    connection_id: u32,
    sequence_counter: u16,
}

impl EthernetIpClient {
    pub fn new(url: &str) -> Self {
        Self {
            stream: None,
            url: url.to_string(),
            session_handle: 0,
            connection_id: 0,
            sequence_counter: 0,
        }
    }

    /// Connect to the PLC and register an EtherNet/IP session.
    pub async fn connect(&mut self) -> Result<()> {
        let stream = TcpStream::connect(&self.url)
            .await
            .with_context(|| format!("Failed to connect to {} for EtherNet/IP", self.url))?;
        stream.set_nodelay(true)?;
        self.stream = Some(stream);

        self.register_session().await?;
        Ok(())
    }

    /// Read a single CIP tag by name.
    pub async fn read_tag(&mut self, tag_name: &str) -> Result<CipValue> {
        let request = build_read_tag_request(tag_name);
        let response = self.send_rr_data(request).await?;
        parse_read_tag_response(&response)
    }

    /// Read multiple CIP tags.
    pub async fn read_tags(&mut self, tag_names: &[&str]) -> Result<Vec<CipValue>> {
        let mut values = Vec::with_capacity(tag_names.len());
        for name in tag_names {
            match self.read_tag(name).await {
                Ok(v) => values.push(v),
                Err(e) => {
                    tracing::warn!("Failed to read tag '{}': {}", name, e);
                    values.push(CipValue::Bytes(Vec::new()));
                }
            }
        }
        Ok(values)
    }

    /// Write a value to a CIP tag.
    pub async fn write_tag(&mut self, tag_name: &str, value: &CipValue) -> Result<()> {
        let request = build_write_tag_request(tag_name, value);
        let response = self.send_rr_data(request).await?;
        parse_write_tag_response(&response)
    }

    /// Browse available tags using GetAttributeAll on the Symbol class (0x6B).
    pub async fn browse_tags(&mut self) -> Result<Vec<CipTag>> {
        let mut tags = Vec::new();
        let mut instance_id: u32 = 1;

        // Iterate symbol class instances to discover tags
        loop {
            let request = build_get_attribute_all_request(SYMBOL_CLASS_ID, instance_id);
            let response = match self.send_rr_data(request).await {
                Ok(r) => r,
                Err(_) => break, // No more instances
            };

            if response.len() < 4 {
                break;
            }

            // Check CIP status
            let status = response[2];
            if status != CIP_STATUS_SUCCESS {
                break;
            }

            match parse_symbol_instance(&response[4..]) {
                Some(tag) => tags.push(tag),
                None => break,
            }

            instance_id += 1;

            // Safety limit: prevent infinite loop on malformed data
            if instance_id > 10000 {
                break;
            }
        }

        Ok(tags)
    }

    /// Disconnect and unregister the session.
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            // Send UnregisterSession
            let packet = build_enip_header(CMD_UNREGISTER_SESSION, self.session_handle, 0, &[]);
            let _ = stream.write_all(&packet).await;
            let _ = stream.shutdown().await;
        }
        self.session_handle = 0;
        self.connection_id = 0;
        Ok(())
    }

    // ── Internal methods ──

    async fn register_session(&mut self) -> Result<()> {
        // Protocol version 1, options 0
        let mut body = [0u8; 4];
        body[0..2].copy_from_slice(&1u16.to_le_bytes()); // protocol version
        body[2..4].copy_from_slice(&0u16.to_le_bytes()); // options

        let packet = build_enip_header(CMD_REGISTER_SESSION, 0, 0, &body);

        let stream = self.stream.as_mut().context("Not connected")?;
        stream.write_all(&packet).await.context("Write RegisterSession failed")?;
        stream.flush().await?;

        let mut header_buf = [0u8; ENIP_HEADER_SIZE];
        stream.read_exact(&mut header_buf).await.context("Read RegisterSession header failed")?;

        let status = u32::from_le_bytes(header_buf[8..12].try_into().unwrap());
        if status != 0 {
            anyhow::bail!("RegisterSession failed with status 0x{:08X}", status);
        }

        self.session_handle = u32::from_le_bytes(header_buf[4..8].try_into().unwrap());

        let length = u16::from_le_bytes(header_buf[2..4].try_into().unwrap()) as usize;
        if length > 0 {
            let mut body_buf = vec![0u8; length];
            stream.read_exact(&mut body_buf).await.context("Read RegisterSession body failed")?;
        }

        tracing::debug!("EtherNet/IP session registered: handle=0x{:08X}", self.session_handle);
        Ok(())
    }

    async fn send_rr_data(&mut self, cip_payload: Vec<u8>) -> Result<Vec<u8>> {
        // Build SendRRData wrapper
        let mut rr_data = Vec::with_capacity(cip_payload.len() + 6);
        rr_data.extend_from_slice(&0u32.to_le_bytes()); // interface handle (CIP = 0)
        rr_data.extend_from_slice(&0u16.to_le_bytes()); // timeout = 0
        let item_count: u16 = 2; // connected address + data
        rr_data.extend_from_slice(&item_count.to_le_bytes());

        // Null address item (unconnected)
        rr_data.extend_from_slice(&0x0000u16.to_le_bytes()); // type ID: null
        rr_data.extend_from_slice(&0u16.to_le_bytes()); // length: 0

        // Unconnected data item
        rr_data.extend_from_slice(&0x00B2u16.to_le_bytes()); // type ID: unconnected data
        rr_data.extend_from_slice(&(cip_payload.len() as u16).to_le_bytes());
        rr_data.extend_from_slice(&cip_payload);

        let packet = build_enip_header(CMD_SEND_RR_DATA, self.session_handle, 0, &rr_data);

        let stream = self.stream.as_mut().context("Not connected")?;
        stream.write_all(&packet).await.context("Write SendRRData failed")?;
        stream.flush().await?;

        // Read response header
        let mut header_buf = [0u8; ENIP_HEADER_SIZE];
        stream.read_exact(&mut header_buf).await.context("Read SendRRData header failed")?;

        let enip_status = u32::from_le_bytes(header_buf[8..12].try_into().unwrap());
        if enip_status != 0 {
            anyhow::bail!("SendRRData ENIP status 0x{:08X}", enip_status);
        }

        let length = u16::from_le_bytes(header_buf[2..4].try_into().unwrap()) as usize;
        let mut body_buf = vec![0u8; length];
        if length > 0 {
            stream.read_exact(&mut body_buf).await.context("Read SendRRData body failed")?;
        }

        // Parse SendRRData response to extract CIP reply
        // Skip: interface handle (4) + timeout (2) + item count (2) + address item (4 or 6) + data item header (4)
        if body_buf.len() < 12 {
            anyhow::bail!("SendRRData response too short");
        }

        // Skip to data item
        let offset = 6; // past interface handle + timeout
        let _item_count = u16::from_le_bytes(body_buf[offset..offset + 2].try_into().unwrap());

        let mut pos = offset + 2;
        // Skip address item
        if pos + 4 <= body_buf.len() {
            let addr_type = u16::from_le_bytes(body_buf[pos..pos + 2].try_into().unwrap());
            let addr_len = u16::from_le_bytes(body_buf[pos + 2..pos + 4].try_into().unwrap()) as usize;
            pos += 4 + addr_len;
        }

        // Data item
        if pos + 4 > body_buf.len() {
            anyhow::bail!("No data item in SendRRData response");
        }

        let _data_type = u16::from_le_bytes(body_buf[pos..pos + 2].try_into().unwrap());
        let data_len = u16::from_le_bytes(body_buf[pos + 2..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        if pos + data_len > body_buf.len() {
            anyhow::bail!("Data item truncated in SendRRData response");
        }

        Ok(body_buf[pos..pos + data_len].to_vec())
    }

    fn next_sequence(&mut self) -> u16 {
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        self.sequence_counter
    }
}

// ── Packet construction helpers ──

fn build_enip_header(command: u16, session_handle: u32, _options: u32, body: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(ENIP_HEADER_SIZE + body.len());
    buf.extend_from_slice(&command.to_le_bytes());
    buf.extend_from_slice(&(body.len() as u16).to_le_bytes());
    buf.extend_from_slice(&session_handle.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // status
    buf.extend_from_slice(&0u64.to_le_bytes()); // sender context
    buf.extend_from_slice(&0u32.to_le_bytes()); // options
    buf.extend_from_slice(body);
    buf
}

/// Build a CIP Read Tag request for explicit messaging.
fn build_read_tag_request(tag_name: &str) -> Vec<u8> {
    let mut request = Vec::new();

    // CIP service: Read Tag
    request.push(CIP_READ_TAG);

    // Request path: encode tag name as symbolic segment
    let path = build_symbolic_path(tag_name);
    request.push((path.len() / 2) as u8); // path size in words
    request.extend_from_slice(&path);

    // Number of elements to read
    request.extend_from_slice(&1u16.to_le_bytes());

    request
}

/// Build a CIP Write Tag request.
fn build_write_tag_request(tag_name: &str, value: &CipValue) -> Vec<u8> {
    let mut request = Vec::new();

    // CIP service: Write Tag
    request.push(CIP_WRITE_TAG);

    // Request path
    let path = build_symbolic_path(tag_name);
    request.push((path.len() / 2) as u8);
    request.extend_from_slice(&path);

    // Data type
    let (type_code, data) = encode_cip_value(value);
    request.extend_from_slice(&(type_code as u16).to_le_bytes());

    // Number of elements
    request.extend_from_slice(&1u16.to_le_bytes());

    // Value data
    request.extend_from_slice(&data);

    request
}

/// Build a CIP GetAttributeAll request for a class instance.
fn build_get_attribute_all_request(class_id: u8, instance_id: u32) -> Vec<u8> {
    let mut request = Vec::new();

    // CIP service: GetAttributeAll
    request.push(CIP_GET_ATTRIBUTES_ALL);

    // Path: class + instance
    request.push(0x03); // 3 words

    // Class segment: 0x20 = logical class segment, 8-bit
    request.push(0x20);
    request.push(class_id);
    request.push(0x00); // padding

    // Instance segment: 0x24 = logical instance segment, 8-bit
    if instance_id <= 255 {
        request.push(0x24);
        request.push(instance_id as u8);
        request.push(0x00);
        request.push(0x00);
    } else {
        // 16-bit instance
        request[1] = 0x04; // 4 words
        request.push(0x25); // logical instance segment, 16-bit
        request.push(0x00); // padding
        request.extend_from_slice(&(instance_id as u16).to_le_bytes());
    }

    request
}

/// Encode a tag name as a CIP symbolic path segment.
fn build_symbolic_path(tag_name: &str) -> Vec<u8> {
    let bytes = tag_name.as_bytes();
    let len = bytes.len();

    let mut path = Vec::with_capacity(len + 2);

    if len <= 255 {
        // Extended symbolic segment: type 0x91
        path.push(0x91);
        path.push(len as u8);
        path.extend_from_slice(bytes);

        // Pad to even length
        if len % 2 != 0 {
            path.push(0x00);
        }
    } else {
        // Long symbolic segment (unlikely but handle it)
        path.push(0x91);
        path.push(0x00); // placeholder
        path.extend_from_slice(&(len as u16).to_le_bytes());
        path.extend_from_slice(bytes);
        while path.len() % 2 != 0 {
            path.push(0x00);
        }
    }

    path
}

/// Encode a CipValue to (type_code, data bytes).
fn encode_cip_value(value: &CipValue) -> (u8, Vec<u8>) {
    match value {
        CipValue::Bool(b) => (CIP_TYPE_BOOL, vec![if *b { 0xFF } else { 0x00 }, 0x00, 0x00, 0x00]),
        CipValue::Sint(v) => (CIP_TYPE_SINT, vec![*v as u8, 0x00, 0x00, 0x00]),
        CipValue::Int(v) => (CIP_TYPE_INT, v.to_le_bytes().to_vec()),
        CipValue::Dint(v) => (CIP_TYPE_DINT, v.to_le_bytes().to_vec()),
        CipValue::Real(v) => (CIP_TYPE_REAL, v.to_le_bytes().to_vec()),
        CipValue::String(s) => {
            let bytes = s.as_bytes();
            let mut data = Vec::with_capacity(88);
            let len = bytes.len().min(82) as u32;
            data.extend_from_slice(&len.to_le_bytes());
            data.extend_from_slice(&bytes[..len as usize]);
            while data.len() < 88 {
                data.push(0x00);
            }
            (CIP_TYPE_STRING, data)
        }
        CipValue::Bytes(b) => (CIP_TYPE_DWORD, {
            let mut d = b.clone();
            while d.len() < 4 {
                d.push(0x00);
            }
            d
        }),
    }
}

/// Parse a CIP Read Tag response into a CipValue.
fn parse_read_tag_response(data: &[u8]) -> Result<CipValue> {
    if data.len() < 4 {
        anyhow::bail!("Read tag response too short ({} bytes)", data.len());
    }

    // CIP reply service (should be 0xCC = Read Tag reply)
    let service = data[0];
    if service != CIP_READ_TAG | 0x80 {
        anyhow::bail!("Unexpected CIP service reply: 0x{:02X}", service);
    }

    // Check for extended status
    let _reserved = data[1];
    let status = data[2];
    if status != CIP_STATUS_SUCCESS && status != CIP_STATUS_PARTIAL {
        let ext_status = if data.len() > 4 { data[3] } else { 0 };
        anyhow::bail!(
            "CIP Read Tag error: status=0x{:02X}, ext=0x{:02X}",
            status,
            ext_status
        );
    }

    if data.len() < 6 {
        anyhow::bail!("Read tag response missing type/data");
    }

    let type_code = u16::from_le_bytes(data[4..6].try_into().unwrap());
    let type_low = type_code as u8;

    // For partial reads, skip to data after the type field
    let data_start = if status == CIP_STATUS_PARTIAL { 6 } else { 6 };
    let value_data = &data[data_start..];

    decode_cip_value(type_low, value_data)
}

/// Parse a CIP Write Tag response.
fn parse_write_tag_response(data: &[u8]) -> Result<()> {
    if data.len() < 4 {
        anyhow::bail!("Write tag response too short");
    }

    let service = data[0];
    if service != CIP_WRITE_TAG | 0x80 {
        anyhow::bail!("Unexpected CIP service reply: 0x{:02X}", service);
    }

    let status = data[2];
    if status != CIP_STATUS_SUCCESS {
        let ext_status = if data.len() > 4 { data[3] } else { 0 };
        anyhow::bail!(
            "CIP Write Tag error: status=0x{:02X}, ext=0x{:02X}",
            status,
            ext_status
        );
    }

    Ok(())
}

/// Decode a CIP value from type code and raw data.
fn decode_cip_value(type_code: u8, data: &[u8]) -> Result<CipValue> {
    match type_code {
        CIP_TYPE_BOOL => {
            if data.is_empty() {
                anyhow::bail!("No data for BOOL");
            }
            Ok(CipValue::Bool(data[0] != 0))
        }
        CIP_TYPE_SINT => {
            if data.is_empty() {
                anyhow::bail!("No data for SINT");
            }
            Ok(CipValue::Sint(data[0] as i8))
        }
        CIP_TYPE_INT => {
            if data.len() < 2 {
                anyhow::bail!("Not enough data for INT");
            }
            Ok(CipValue::Int(i16::from_le_bytes(
                data[0..2].try_into().unwrap(),
            )))
        }
        CIP_TYPE_DINT => {
            if data.len() < 4 {
                anyhow::bail!("Not enough data for DINT");
            }
            Ok(CipValue::Dint(i32::from_le_bytes(
                data[0..4].try_into().unwrap(),
            )))
        }
        CIP_TYPE_REAL => {
            if data.len() < 4 {
                anyhow::bail!("Not enough data for REAL");
            }
            Ok(CipValue::Real(f32::from_le_bytes(
                data[0..4].try_into().unwrap(),
            )))
        }
        CIP_TYPE_STRING => {
            if data.len() < 4 {
                anyhow::bail!("Not enough data for STRING header");
            }
            let str_len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
            let str_data = if data.len() >= 4 + str_len {
                &data[4..4 + str_len]
            } else {
                &data[4..]
            };
            Ok(CipValue::String(
                String::from_utf8_lossy(str_data).to_string(),
            ))
        }
        CIP_TYPE_DWORD => {
            if data.len() < 4 {
                anyhow::bail!("Not enough data for DWORD");
            }
            Ok(CipValue::Bytes(data[0..4].to_vec()))
        }
        CIP_TYPE_LINT => {
            if data.len() < 8 {
                anyhow::bail!("Not enough data for LINT");
            }
            // Store as bytes since we don't have a Lint variant in CipValue
            Ok(CipValue::Bytes(data[0..8].to_vec()))
        }
        _ => {
            // Unknown type — return raw bytes
            Ok(CipValue::Bytes(data.to_vec()))
        }
    }
}

/// Parse a Symbol class instance to extract tag info.
fn parse_symbol_instance(data: &[u8]) -> Option<CipTag> {
    // The symbol instance data structure from GetAttributeAll:
    // Attribute 1: name (STRING)
    // Attribute 2: type (UINT)
    // The actual layout varies but we try a best-effort parse.

    if data.len() < 8 {
        return None;
    }

    // Try to read name length (first 4 bytes for CIP STRING len)
    let name_len = u32::from_le_bytes(data[0..4].try_into().ok()?) as usize;
    if name_len == 0 || name_len > 255 || data.len() < 4 + name_len + 2 {
        return None;
    }

    let name_bytes = &data[4..4 + name_len];
    let name = String::from_utf8(name_bytes.to_vec()).ok()?;
    if name.is_empty() {
        return None;
    }

    // After the name, skip padding to even boundary, then read type info
    let type_offset = 4 + name_len + (name_len % 2);
    if data.len() < type_offset + 4 {
        return None;
    }

    // Type word: upper byte has element count flags, lower byte has type code
    let type_word = u16::from_le_bytes(data[type_offset..type_offset + 2].try_into().ok()?);
    let type_code = (type_word & 0xFF) as u8;
    let element_count = ((type_word >> 8) & 0xFF) as u16;

    let data_type = CipDataType::from_code(type_code)?;

    Some(CipTag {
        name,
        data_type,
        element_count: if element_count == 0 { 1 } else { element_count },
    })
}

// ── Poller ──

/// Runtime config for an EtherNet/IP device poller.
#[derive(Debug, Clone)]
pub struct EnipDeviceRuntimeConfig {
    pub id: String,
    pub url: String,
    pub slot: u8,
    pub interval_ms: u64,
    pub tags: Vec<EnipTagConfig>,
}

/// Tag configuration for the poller.
#[derive(Debug, Clone)]
pub struct EnipTagConfig {
    pub tag_name: String,
    pub name: String,
    pub data_type: String,
}

/// Main entry point for EtherNet/IP device polling.
///
/// Spawned as `tokio::spawn(ethernet_ip::poll_enip_device(config, cache))`.
/// Runs an infinite loop with exponential backoff on connection failure.
pub async fn poll_enip_device(config: EnipDeviceRuntimeConfig, cache: DeviceCache) {
    let device_id = config.id.clone();
    let mut backoff = Backoff::new();

    loop {
        match try_poll_cycle(&config, &cache).await {
            Ok(()) => {
                backoff.reset();
            }
            Err(e) => {
                tracing::warn!("EtherNet/IP device {} error: {e}", device_id);
                cache.set_status(&device_id, DeviceStatus::Offline).await;
                let delay = backoff.next();
                tracing::info!(
                    "EtherNet/IP device {} retrying in {delay}ms",
                    device_id
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
                continue;
            }
        }

        // Wait for the configured interval
        tokio::time::sleep(Duration::from_millis(config.interval_ms)).await;
    }
}

/// Attempt one poll cycle: connect, read all tags, update cache.
async fn try_poll_cycle(config: &EnipDeviceRuntimeConfig, cache: &DeviceCache) -> Result<()> {
    let mut client = EthernetIpClient::new(&config.url);
    client.connect().await.context("EtherNet/IP connect failed")?;

    let mut metrics = HashMap::new();

    for tag in &config.tags {
        match client.read_tag(&tag.tag_name).await {
            Ok(value) => {
                if let Some(fv) = value.to_f64() {
                    metrics.insert(tag.name.clone(), fv);
                }
            }
            Err(e) => {
                tracing::warn!(
                    "EtherNet/IP tag read '{}' failed: {}",
                    tag.tag_name,
                    e
                );
            }
        }
    }

    client.disconnect().await.ok();

    if !metrics.is_empty() {
        cache.update(&config.id, metrics).await;
    }

    Ok(())
}

// ── Backoff helper ──

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

// ── Tag name validation ──

/// Validate a CIP tag name. Valid tag names contain alphanumeric characters,
/// underscores, and may use dotted paths for structure members (e.g., "MyTag.Member").
pub fn validate_tag_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Tag name cannot be empty");
    }
    if name.len() > 255 {
        anyhow::bail!("Tag name too long (max 255 chars)");
    }

    for (i, ch) in name.chars().enumerate() {
        if i == 0 && ch.is_ascii_digit() {
            anyhow::bail!("Tag name cannot start with a digit");
        }
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.' && ch != '[' && ch != ']' {
            anyhow::bail!(
                "Tag name contains invalid character '{}' at position {}",
                ch,
                i
            );
        }
    }

    Ok(())
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    // ── Packet construction tests ──

    #[test]
    fn test_build_enip_header_register_session() {
        let body = [0u8; 4]; // protocol version + options
        let packet = build_enip_header(CMD_REGISTER_SESSION, 0, 0, &body);
        assert_eq!(packet.len(), ENIP_HEADER_SIZE + 4);
        // Command
        assert_eq!(u16::from_le_bytes([packet[0], packet[1]]), CMD_REGISTER_SESSION);
        // Length
        assert_eq!(u16::from_le_bytes([packet[2], packet[3]]), 4);
        // Session handle
        assert_eq!(u32::from_le_bytes([packet[4], packet[5], packet[6], packet[7]]), 0);
        // Status
        assert_eq!(u32::from_le_bytes([packet[8], packet[9], packet[10], packet[11]]), 0);
    }

    #[test]
    fn test_build_enip_header_with_session() {
        let body = vec![1, 2, 3, 4];
        let packet = build_enip_header(CMD_SEND_RR_DATA, 0x12345678, 0, &body);
        assert_eq!(packet.len(), ENIP_HEADER_SIZE + 4);
        assert_eq!(u16::from_le_bytes([packet[0], packet[1]]), CMD_SEND_RR_DATA);
        assert_eq!(u32::from_le_bytes([packet[4], packet[5], packet[6], packet[7]]), 0x12345678);
    }

    #[test]
    fn test_build_read_tag_request() {
        let request = build_read_tag_request("MyTag");
        assert_eq!(request[0], CIP_READ_TAG);
        // Path size should be > 0
        assert!(request[1] > 0);
        // Number of elements = 1
        let len = request.len();
        assert_eq!(u16::from_le_bytes([request[len - 2], request[len - 1]]), 1);
    }

    #[test]
    fn test_build_symbolic_path_short() {
        let path = build_symbolic_path("Tag1");
        assert_eq!(path[0], 0x91); // extended symbolic segment
        assert_eq!(path[1], 4); // length
        assert_eq!(&path[2..6], b"Tag1");
    }

    #[test]
    fn test_build_symbolic_path_padded() {
        let path = build_symbolic_path("Tag"); // 3 bytes -> needs padding
        assert_eq!(path[0], 0x91);
        assert_eq!(path[1], 3);
        assert_eq!(&path[2..5], b"Tag");
        assert_eq!(path[5], 0x00); // padding
    }

    #[test]
    fn test_build_symbolic_path_long() {
        let path = build_symbolic_path("VeryLongTagName123");
        assert_eq!(path[0], 0x91);
        assert_eq!(path[1], 18);
    }

    // ── CIP data type tests ──

    #[test]
    fn test_cip_data_type_from_code() {
        assert_eq!(CipDataType::from_code(CIP_TYPE_BOOL), Some(CipDataType::Bool));
        assert_eq!(CipDataType::from_code(CIP_TYPE_SINT), Some(CipDataType::Sint));
        assert_eq!(CipDataType::from_code(CIP_TYPE_INT), Some(CipDataType::Int));
        assert_eq!(CipDataType::from_code(CIP_TYPE_DINT), Some(CipDataType::Dint));
        assert_eq!(CipDataType::from_code(CIP_TYPE_REAL), Some(CipDataType::Real));
        assert_eq!(CipDataType::from_code(CIP_TYPE_STRING), Some(CipDataType::String));
        assert_eq!(CipDataType::from_code(CIP_TYPE_DWORD), Some(CipDataType::Dword));
        assert_eq!(CipDataType::from_code(CIP_TYPE_LINT), Some(CipDataType::Lint));
        assert_eq!(CipDataType::from_code(0xFF), None);
    }

    #[test]
    fn test_cip_data_type_code_roundtrip() {
        for dt in &[
            CipDataType::Bool,
            CipDataType::Sint,
            CipDataType::Int,
            CipDataType::Dint,
            CipDataType::Real,
            CipDataType::String,
            CipDataType::Dword,
            CipDataType::Lint,
        ] {
            assert_eq!(CipDataType::from_code(dt.code()), Some(dt.clone()));
        }
    }

    #[test]
    fn test_cip_data_type_element_sizes() {
        assert_eq!(CipDataType::Bool.element_size(), 1);
        assert_eq!(CipDataType::Sint.element_size(), 1);
        assert_eq!(CipDataType::Int.element_size(), 2);
        assert_eq!(CipDataType::Dint.element_size(), 4);
        assert_eq!(CipDataType::Real.element_size(), 4);
        assert_eq!(CipDataType::Lint.element_size(), 8);
    }

    #[test]
    fn test_cip_data_type_as_str() {
        assert_eq!(CipDataType::Bool.as_str(), "BOOL");
        assert_eq!(CipDataType::Sint.as_str(), "SINT");
        assert_eq!(CipDataType::Int.as_str(), "INT");
        assert_eq!(CipDataType::Dint.as_str(), "DINT");
        assert_eq!(CipDataType::Real.as_str(), "REAL");
        assert_eq!(CipDataType::String.as_str(), "STRING");
        assert_eq!(CipDataType::Dword.as_str(), "DWORD");
        assert_eq!(CipDataType::Lint.as_str(), "LINT");
    }

    // ── CIP value encoding/decoding tests ──

    #[test]
    fn test_encode_cip_value_bool() {
        let (code, data) = encode_cip_value(&CipValue::Bool(true));
        assert_eq!(code, CIP_TYPE_BOOL);
        assert_eq!(data[0], 0xFF);

        let (code, data) = encode_cip_value(&CipValue::Bool(false));
        assert_eq!(code, CIP_TYPE_BOOL);
        assert_eq!(data[0], 0x00);
    }

    #[test]
    fn test_encode_cip_value_int() {
        let (code, data) = encode_cip_value(&CipValue::Int(-1234));
        assert_eq!(code, CIP_TYPE_INT);
        assert_eq!(i16::from_le_bytes([data[0], data[1]]), -1234);
    }

    #[test]
    fn test_encode_cip_value_dint() {
        let (code, data) = encode_cip_value(&CipValue::Dint(42));
        assert_eq!(code, CIP_TYPE_DINT);
        assert_eq!(i32::from_le_bytes([data[0], data[1], data[2], data[3]]), 42);
    }

    #[test]
    fn test_encode_cip_value_real() {
        let (code, data) = encode_cip_value(&CipValue::Real(3.14));
        assert_eq!(code, CIP_TYPE_REAL);
        let f = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        assert!((f - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_encode_cip_value_string() {
        let (code, data) = encode_cip_value(&CipValue::String("Hello".to_string()));
        assert_eq!(code, CIP_TYPE_STRING);
        let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        assert_eq!(len, 5);
        assert_eq!(&data[4..9], b"Hello");
    }

    #[test]
    fn test_decode_cip_value_bool() {
        let val = decode_cip_value(CIP_TYPE_BOOL, &[0xFF, 0, 0, 0]).unwrap();
        assert_eq!(val, CipValue::Bool(true));

        let val = decode_cip_value(CIP_TYPE_BOOL, &[0x00, 0, 0, 0]).unwrap();
        assert_eq!(val, CipValue::Bool(false));
    }

    #[test]
    fn test_decode_cip_value_int() {
        let data = (-1234i16).to_le_bytes();
        let val = decode_cip_value(CIP_TYPE_INT, &data).unwrap();
        assert_eq!(val, CipValue::Int(-1234));
    }

    #[test]
    fn test_decode_cip_value_dint() {
        let data = 42i32.to_le_bytes();
        let val = decode_cip_value(CIP_TYPE_DINT, &data).unwrap();
        assert_eq!(val, CipValue::Dint(42));
    }

    #[test]
    fn test_decode_cip_value_real() {
        let data = 3.14f32.to_le_bytes();
        let val = decode_cip_value(CIP_TYPE_REAL, &data).unwrap();
        match val {
            CipValue::Real(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Real"),
        }
    }

    #[test]
    fn test_decode_cip_value_string() {
        let mut data = Vec::new();
        data.extend_from_slice(&5u32.to_le_bytes());
        data.extend_from_slice(b"Hello");
        let val = decode_cip_value(CIP_TYPE_STRING, &data).unwrap();
        assert_eq!(val, CipValue::String("Hello".to_string()));
    }

    #[test]
    fn test_decode_cip_value_unknown_type() {
        let data = vec![0xAA, 0xBB];
        let val = decode_cip_value(0xFE, &data).unwrap();
        assert!(matches!(val, CipValue::Bytes(_)));
    }

    // ── CipValue conversion tests ──

    #[test]
    fn test_cip_value_to_f64() {
        assert_eq!(CipValue::Bool(true).to_f64(), Some(1.0));
        assert_eq!(CipValue::Bool(false).to_f64(), Some(0.0));
        assert_eq!(CipValue::Sint(-5).to_f64(), Some(-5.0));
        assert_eq!(CipValue::Int(-100).to_f64(), Some(-100.0));
        assert_eq!(CipValue::Dint(42).to_f64(), Some(42.0));
        assert!((CipValue::Real(3.14).to_f64().unwrap() - 3.14).abs() < 0.001);
        assert_eq!(CipValue::String("hi".into()).to_f64(), None);
        assert_eq!(CipValue::Bytes(vec![]).to_f64(), None);
    }

    #[test]
    fn test_cip_value_display() {
        assert_eq!(CipValue::Bool(true).display_value(), "true");
        assert_eq!(CipValue::Sint(-3).display_value(), "-3");
        assert_eq!(CipValue::Int(-100).display_value(), "-100");
        assert_eq!(CipValue::Dint(42).display_value(), "42");
        assert_eq!(CipValue::Real(3.14).display_value(), "3.1400");
        assert_eq!(CipValue::String("hello".into()).display_value(), "hello");
    }

    // ── Tag name validation tests ──

    #[test]
    fn test_validate_tag_name_valid() {
        assert!(validate_tag_name("MyTag").is_ok());
        assert!(validate_tag_name("tag1").is_ok());
        assert!(validate_tag_name("_private").is_ok());
        assert!(validate_tag_name("Tag.Structure").is_ok());
        assert!(validate_tag_name("Array[0]").is_ok());
        assert!(validate_tag_name("a").is_ok());
    }

    #[test]
    fn test_validate_tag_name_invalid() {
        assert!(validate_tag_name("").is_err());
        assert!(validate_tag_name("123tag").is_err()); // starts with digit
        assert!(validate_tag_name("tag name").is_err()); // contains space
        assert!(validate_tag_name("tag-name").is_err()); // contains hyphen
    }

    #[test]
    fn test_validate_tag_name_too_long() {
        let long_name = "A".repeat(256);
        assert!(validate_tag_name(&long_name).is_err());
    }

    // ── Backoff tests ──

    #[test]
    fn test_backoff_sequence() {
        let mut b = Backoff::new();
        assert_eq!(b.next(), 1000);
        assert_eq!(b.next(), 2000);
        assert_eq!(b.next(), 4000);
        assert_eq!(b.next(), 8000);
        assert_eq!(b.next(), 16000);
        assert_eq!(b.next(), 32000);
        assert_eq!(b.next(), 64000); // capped at 60000
    }

    #[test]
    fn test_backoff_reset() {
        let mut b = Backoff::new();
        b.next();
        b.next();
        b.reset();
        assert_eq!(b.next(), 1000);
    }

    // ── Parse read tag response tests ──

    #[test]
    fn test_parse_read_tag_response_dint() {
        // Simulate a successful DINT read response
        let mut data = vec![CIP_READ_TAG | 0x80, 0x00, CIP_STATUS_SUCCESS, 0x00];
        data.extend_from_slice(&CIP_TYPE_DINT.to_le_bytes()); // type
        data.extend_from_slice(&42i32.to_le_bytes()); // value
        let val = parse_read_tag_response(&data).unwrap();
        assert_eq!(val, CipValue::Dint(42));
    }

    #[test]
    fn test_parse_read_tag_response_error() {
        let data = vec![CIP_READ_TAG | 0x80, 0x00, 0x04, 0x00]; // error status
        let result = parse_read_tag_response(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_read_tag_response_wrong_service() {
        let data = vec![0x99, 0x00, CIP_STATUS_SUCCESS, 0x00];
        let result = parse_read_tag_response(&data);
        assert!(result.is_err());
    }

    // ── Parse write tag response tests ──

    #[test]
    fn test_parse_write_tag_response_success() {
        let data = vec![CIP_WRITE_TAG | 0x80, 0x00, CIP_STATUS_SUCCESS, 0x00];
        assert!(parse_write_tag_response(&data).is_ok());
    }

    #[test]
    fn test_parse_write_tag_response_error() {
        let data = vec![CIP_WRITE_TAG | 0x80, 0x00, 0x04, 0x00];
        assert!(parse_write_tag_response(&data).is_err());
    }

    // ── Parse symbol instance tests ──

    #[test]
    fn test_parse_symbol_instance_valid() {
        let mut data = Vec::new();
        data.extend_from_slice(&5u32.to_le_bytes()); // name len
        data.extend_from_slice(b"MyTag1"); // name (5 bytes + padding)
        data.extend_from_slice(&0x00u8.to_le_bytes()); // padding
        data.extend_from_slice(&((1u16 << 8) | CIP_TYPE_DINT as u16).to_le_bytes()); // type: 1 element, DINT

        let tag = parse_symbol_instance(&data).unwrap();
        assert_eq!(tag.name, "MyTag1");
        assert_eq!(tag.data_type, CipDataType::Dint);
    }

    #[test]
    fn test_parse_symbol_instance_empty() {
        assert!(parse_symbol_instance(&[]).is_none());
        assert!(parse_symbol_instance(&[0, 0, 0, 0]).is_none());
    }
}
