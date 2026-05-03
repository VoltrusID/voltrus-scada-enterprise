use std::collections::HashMap;
use std::io::{Read, Write};

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::cache::DeviceCache;
use crate::models::{DeviceStatus, WriteResult};

// ── S7 Protocol Constants ──

const TPKT_VERSION: u8 = 3;
const COTP_CR: u8 = 0xE0; // Connection Request
const COTP_CC: u8 = 0xD0; // Connection Confirm
const COTP_DT: u8 = 0xF0; // Data Transfer

const S7_PROTOCOL_ID: u8 = 0x32;
const S7_MSG_TYPE_JOB: u8 = 0x01;
const S7_MSG_TYPE_ACK_DATA: u8 = 0x03;

const S7_SETUP_COMM: u8 = 0xF0;
const S7_READ_VAR: u8 = 0x04;
const S7_WRITE_VAR: u8 = 0x05;

// Area codes
const AREA_DB: u8 = 0x84;
const AREA_INPUT: u8 = 0x81;
const AREA_OUTPUT: u8 = 0x82;
const AREA_MARKER: u8 = 0x83;
const AREA_TIMER: u8 = 0x1D;
const AREA_COUNTER: u8 = 0x1C;

// Transport sizes
const TS_BIT: u8 = 0x01;
const TS_BYTE: u8 = 0x02;
const TS_WORD: u8 = 0x04;
const TS_DWORD: u8 = 0x06;
const TS_REAL: u8 = 0x08;

// S7 error codes
const S7_ERR_NONE: u16 = 0x0000;

// Default PDU size
const DEFAULT_PDU_SIZE: u16 = 480;
const MAX_PDU_SIZE: u16 = 960;

// ── S7 Data Types ──

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum S7Area {
    DB,
    Input,
    Output,
    Marker,
    Timer,
    Counter,
}

impl S7Area {
    pub fn code(&self) -> u8 {
        match self {
            S7Area::DB => AREA_DB,
            S7Area::Input => AREA_INPUT,
            S7Area::Output => AREA_OUTPUT,
            S7Area::Marker => AREA_MARKER,
            S7Area::Timer => AREA_TIMER,
            S7Area::Counter => AREA_COUNTER,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            AREA_DB => Some(S7Area::DB),
            AREA_INPUT => Some(S7Area::Input),
            AREA_OUTPUT => Some(S7Area::Output),
            AREA_MARKER => Some(S7Area::Marker),
            AREA_TIMER => Some(S7Area::Timer),
            AREA_COUNTER => Some(S7Area::Counter),
            _ => None,
        }
    }
}

impl std::fmt::Display for S7Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S7Area::DB => write!(f, "DB"),
            S7Area::Input => write!(f, "Input"),
            S7Area::Output => write!(f, "Output"),
            S7Area::Marker => write!(f, "Marker"),
            S7Area::Timer => write!(f, "Timer"),
            S7Area::Counter => write!(f, "Counter"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum S7DataType {
    Bool,
    Byte,
    Word,
    DWord,
    Real,
    String,
}

impl S7DataType {
    pub fn transport_size(&self) -> u8 {
        match self {
            S7DataType::Bool => TS_BIT,
            S7DataType::Byte => TS_BYTE,
            S7DataType::Word => TS_WORD,
            S7DataType::DWord => TS_DWORD,
            S7DataType::Real => TS_REAL,
            S7DataType::String => TS_BYTE, // Strings are transported as byte arrays
        }
    }

    pub fn byte_size(&self) -> u16 {
        match self {
            S7DataType::Bool => 1,
            S7DataType::Byte => 1,
            S7DataType::Word => 2,
            S7DataType::DWord => 4,
            S7DataType::Real => 4,
            S7DataType::String => 1, // Per character, length determines total
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bool" | "bit" => Some(S7DataType::Bool),
            "byte" | "uint8" => Some(S7DataType::Byte),
            "word" | "uint16" | "int16" => Some(S7DataType::Word),
            "dword" | "uint32" | "int32" => Some(S7DataType::DWord),
            "real" | "float" | "float32" => Some(S7DataType::Real),
            "string" | "str" => Some(S7DataType::String),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct S7Tag {
    pub area: S7Area,
    pub db_number: u16,
    pub offset: u16,
    pub bit_offset: u8,
    pub data_type: S7DataType,
    pub length: u16,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum S7Value {
    Bool(bool),
    Byte(u8),
    Word(u16),
    DWord(u32),
    Real(f32),
    String(String),
}

impl S7Value {
    pub fn to_f64(&self) -> f64 {
        match self {
            S7Value::Bool(v) => if *v { 1.0 } else { 0.0 },
            S7Value::Byte(v) => *v as f64,
            S7Value::Word(v) => *v as f64,
            S7Value::DWord(v) => *v as f64,
            S7Value::Real(v) => *v as f64,
            S7Value::String(_) => 0.0,
        }
    }
}

// ── S7 Client ──

pub struct S7Client {
    stream: Option<TcpStream>,
    url: String,
    rack: u8,
    slot: u8,
    pdu_ref: u16,
    pdu_size: u16,
}

impl S7Client {
    pub fn new(url: &str, rack: u8, slot: u8) -> Self {
        Self {
            stream: None,
            url: url.to_string(),
            rack,
            slot,
            pdu_ref: 0,
            pdu_size: DEFAULT_PDU_SIZE,
        }
    }

    /// Connect to the S7 PLC via ISO-on-TCP (TPKT + COTP + S7 Setup Communication).
    pub async fn connect(&mut self) -> Result<()> {
        let stream = TcpStream::connect(&self.url)
            .await
            .with_context(|| format!("Failed to connect to S7 PLC at {}", self.url))?;

        self.stream = Some(stream);

        // Send COTP Connection Request
        let cotp_cr = self.build_cotp_cr()?;
        self.send_raw(&cotp_cr).await?;

        // Read COTP Connection Confirm
        let cc = self.read_cotp_cc().await?;

        // Verify it's a CC
        if cc.len() < 6 || cc[1] != COTP_CC {
            anyhow::bail!("Expected COTP Connection Confirm, got: {:02X?}", &cc[..cc.len().min(6)]);
        }

        // Send S7 Setup Communication
        let setup = self.build_s7_setup_comm();
        self.send_s7(&setup).await?;

        // Read S7 Setup Communication response
        let resp = self.read_s7_response().await?;
        self.parse_setup_comm_response(&resp)?;

        tracing::info!(
            "Connected to S7 PLC at {} (rack={}, slot={}), PDU size={}",
            self.url, self.rack, self.slot, self.pdu_size
        );

        Ok(())
    }

    /// Read a single tag from the PLC.
    pub async fn read_tag(&mut self, tag: &S7Tag) -> Result<S7Value> {
        let values = self.read_tags(&[tag.clone()]).await?;
        values.into_iter().next().context("No value returned for tag")
    }

    /// Read multiple tags in one S7 request.
    pub async fn read_tags(&mut self, tags: &[S7Tag]) -> Result<Vec<S7Value>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let request = self.build_read_var_request(tags)?;
        self.send_s7(&request).await?;

        let response = self.read_s7_response().await?;
        self.parse_read_var_response(&response, tags)
    }

    /// Write a single tag to the PLC.
    pub async fn write_tag(&mut self, tag: &S7Tag, value: &S7Value) -> Result<()> {
        let request = self.build_write_var_request(tag, value)?;
        self.send_s7(&request).await?;

        let response = self.read_s7_response().await?;
        self.parse_write_var_response(&response)
    }

    /// Disconnect from the PLC.
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        Ok(())
    }

    /// Parse an S7 tag address string into an S7Tag struct.
    ///
    /// Supported formats:
    /// - "DB1.DBD0"     -> DB1, DWord at offset 0
    /// - "DB1.DBX0.1"   -> DB1, Bool at offset 0, bit 1
    /// - "DB1.DBW0"     -> DB1, Word at offset 0
    /// - "DB1.DBB0"     -> DB1, Byte at offset 0
    /// - "DB1.DBD100"   -> DB1, DWord at offset 100
    /// - "DB10.DBX10.3" -> DB10, Bool at offset 10, bit 3
    /// - "I0.0"         -> Input, Bool at offset 0, bit 0
    /// - "IB0"          -> Input, Byte at offset 0
    /// - "IW100"        -> Input, Word at offset 100
    /// - "ID100"        -> Input, DWord at offset 100
    /// - "Q0.0"         -> Output, Bool at offset 0, bit 0
    /// - "QB0"          -> Output, Byte at offset 0
    /// - "QW200"        -> Output, Word at offset 200
    /// - "MW100"        -> Marker, Word at offset 100
    /// - "MD100"        -> Marker, DWord at offset 100
    /// - "MB100"        -> Marker, Byte at offset 100
    pub fn parse_tag_address(addr: &str) -> Result<S7Tag> {
        let addr = addr.trim().to_uppercase();

        // DB addresses: DB<n>.DB<type><offset>[.bit]
        if let Some(rest) = addr.strip_prefix("DB") {
            let parts: Vec<&str> = rest.split('.').collect();
            if parts.len() >= 2 {
                let db_number: u16 = parts[0].parse()
                    .with_context(|| format!("Invalid DB number in '{}'", addr))?;

                let type_and_offset = parts[1];
                if type_and_offset.starts_with("DBX") || type_and_offset.starts_with("DBB") ||
                   type_and_offset.starts_with("DBW") || type_and_offset.starts_with("DBD") {

                    let (data_type, offset_str) = if type_and_offset.starts_with("DBX") {
                        (S7DataType::Bool, &type_and_offset[3..])
                    } else if type_and_offset.starts_with("DBB") {
                        (S7DataType::Byte, &type_and_offset[3..])
                    } else if type_and_offset.starts_with("DBW") {
                        (S7DataType::Word, &type_and_offset[3..])
                    } else {
                        (S7DataType::DWord, &type_and_offset[3..])
                    };

                    let offset: u16 = offset_str.parse()
                        .with_context(|| format!("Invalid offset in '{}'", addr))?;

                    let bit_offset = if parts.len() >= 3 && data_type == S7DataType::Bool {
                        parts[2].parse()
                            .with_context(|| format!("Invalid bit offset in '{}'", addr))?
                    } else {
                        0
                    };

                    return Ok(S7Tag {
                        area: S7Area::DB,
                        db_number,
                        offset,
                        bit_offset,
                        data_type,
                        length: 1,
                    });
                }
            }
        }

        // I, Q, M addresses
        let (area, rest) = if addr.starts_with('I') {
            (S7Area::Input, &addr[1..])
        } else if addr.starts_with('Q') {
            (S7Area::Output, &addr[1..])
        } else if addr.starts_with('M') {
            (S7Area::Marker, &addr[1..])
        } else {
            anyhow::bail!("Unsupported tag address format: '{}'. Use DB<n>.DBx<m>, Ix, Qx, or Mx format.", addr);
        };

        // Check if it's a bit address (e.g., "I0.0", "Q0.1", "M10.3")
        if rest.contains('.') {
            let parts: Vec<&str> = rest.split('.').collect();
            if parts.len() == 2 {
                let offset: u16 = parts[0].parse()
                    .with_context(|| format!("Invalid offset in '{}'", addr))?;
                let bit_offset: u8 = parts[1].parse()
                    .with_context(|| format!("Invalid bit offset in '{}'", addr))?;
                return Ok(S7Tag {
                    area,
                    db_number: 0,
                    offset,
                    bit_offset,
                    data_type: S7DataType::Bool,
                    length: 1,
                });
            }
        }

        // Byte/Word/DWord addresses (e.g., "IB0", "IW100", "ID100", "MB10", "MW100", "QW200")
        let (data_type, offset_str) = if rest.starts_with('B') {
            (S7DataType::Byte, &rest[1..])
        } else if rest.starts_with('W') {
            (S7DataType::Word, &rest[1..])
        } else if rest.starts_with('D') {
            (S7DataType::DWord, &rest[1..])
        } else {
            // Default to byte if just a number (e.g., "I0" -> IB0)
            (S7DataType::Byte, rest)
        };

        let offset: u16 = offset_str.parse()
            .with_context(|| format!("Invalid offset in '{}'", addr))?;

        Ok(S7Tag {
            area,
            db_number: 0,
            offset,
            bit_offset: 0,
            data_type,
            length: 1,
        })
    }

    // ── Private helpers ──

    fn next_pdu_ref(&mut self) -> u16 {
        self.pdu_ref = self.pdu_ref.wrapping_add(1);
        self.pdu_ref
    }

    fn build_cotp_cr(&self) -> Result<Vec<u8>> {
        // COTP CR parameters: TSAP (Transport Service Access Point)
        // Source TSAP: 0x0100 (local)
        // Destination TSAP: 0x0100 + (rack * 0x20) + slot
        let src_tsap: u16 = 0x0100;
        let dst_tsap: u16 = 0x0100 + ((self.rack as u16) << 5) + self.slot as u16;

        let mut cotp_params = Vec::new();
        // Source TSAP reference (parameter code 0xC1)
        cotp_params.push(0xC1); // parameter code: source TSAP
        cotp_params.push(0x02); // parameter length
        cotp_params.extend_from_slice(&src_tsap.to_be_bytes());
        // Destination TSAP reference (parameter code 0xC2)
        cotp_params.push(0xC2); // parameter code: destination TSAP
        cotp_params.push(0x02); // parameter length
        cotp_params.extend_from_slice(&dst_tsap.to_be_bytes());
        // TPDU size (parameter code 0xC0)
        cotp_params.push(0xC0);
        cotp_params.push(0x01);
        cotp_params.push(0x0A); // TPDU size = 0x0A (1024)

        let mut cotp = Vec::new();
        let cotp_len = 1 + 2 + 2 + 1 + cotp_params.len(); // length byte + PDU type + dst-ref + src-ref + class + params
        cotp.push(cotp_len as u8);
        cotp.push(COTP_CR);
        cotp.extend_from_slice(&0x0001u16.to_be_bytes()); // dst-ref
        cotp.extend_from_slice(&0x0001u16.to_be_bytes()); // src-ref
        cotp.push(0x00); // class
        cotp.extend_from_slice(&cotp_params);

        // Wrap in TPKT
        let mut tpkt = Vec::new();
        let total_len = 4 + cotp.len(); // TPKT header (4 bytes) + COTP
        tpkt.push(TPKT_VERSION);
        tpkt.push(0x00); // reserved
        tpkt.extend_from_slice(&(total_len as u16).to_be_bytes());
        tpkt.extend_from_slice(&cotp);

        Ok(tpkt)
    }

    async fn read_cotp_cc(&mut self) -> Result<Vec<u8>> {
        let stream = self.stream.as_mut().context("Not connected")?;

        // Read TPKT header (4 bytes)
        let mut tpkt_header = [0u8; 4];
        stream.read_exact(&mut tpkt_header).await
            .context("Failed to read TPKT header")?;

        if tpkt_header[0] != TPKT_VERSION {
            anyhow::bail!("Invalid TPKT version: {}", tpkt_header[0]);
        }

        let packet_len = u16::from_be_bytes([tpkt_header[2], tpkt_header[3]]) as usize;
        if packet_len < 4 {
            anyhow::bail!("Invalid TPKT packet length: {}", packet_len);
        }

        let mut payload = vec![0u8; packet_len - 4];
        stream.read_exact(&mut payload).await
            .context("Failed to read COTP payload")?;

        Ok(payload)
    }

    fn build_s7_setup_comm(&mut self) -> Vec<u8> {
        let pdu_ref = self.next_pdu_ref();

        // S7 Setup Communication parameters
        let mut params = Vec::new();
        params.push(S7_SETUP_COMM); // function code
        params.push(0x00); // reserved
        params.extend_from_slice(&MAX_PDU_SIZE.to_be_bytes()); // max AMQ calling
        params.extend_from_slice(&MAX_PDU_SIZE.to_be_bytes()); // max AMQ called
        params.extend_from_slice(&DEFAULT_PDU_SIZE.to_be_bytes()); // PDU size

        let mut s7 = Vec::new();
        s7.push(S7_PROTOCOL_ID);
        s7.push(S7_MSG_TYPE_JOB);
        s7.extend_from_slice(&0x0000u16.to_be_bytes()); // reserved
        s7.extend_from_slice(&pdu_ref.to_be_bytes());
        s7.extend_from_slice(&(params.len() as u16).to_be_bytes());
        s7.extend_from_slice(&0x0000u16.to_be_bytes()); // data length = 0
        s7.extend_from_slice(&params);

        s7
    }

    fn parse_setup_comm_response(&mut self, data: &[u8]) -> Result<()> {
        if data.len() < 17 {
            anyhow::bail!("S7 Setup Communication response too short: {} bytes", data.len());
        }

        if data[0] != S7_PROTOCOL_ID {
            anyhow::bail!("Invalid S7 protocol ID: 0x{:02X}", data[0]);
        }

        let error_code = u16::from_be_bytes([data[10], data[11]]);
        if error_code != S7_ERR_NONE {
            anyhow::bail!("S7 Setup Communication error: 0x{:04X}", error_code);
        }

        // Parse negotiated PDU size from data area
        if data.len() >= 27 {
            let negotiated_pdu = u16::from_be_bytes([data[25], data[26]]);
            if negotiated_pdu > 0 {
                self.pdu_size = negotiated_pdu.min(MAX_PDU_SIZE);
            }
        }

        Ok(())
    }

    fn build_read_var_request(&mut self, tags: &[S7Tag]) -> Result<Vec<u8>> {
        let pdu_ref = self.next_pdu_ref();

        // Build parameter area: function code + item count + items
        let mut params = Vec::new();
        params.push(S7_READ_VAR);
        params.push(tags.len() as u8); // item count

        // Calculate total data size for the data header
        let mut total_data_size: u16 = 0;
        let mut items = Vec::new();

        for tag in tags {
            let transport_size = tag.data_type.transport_size();
            let byte_count = if tag.data_type == S7DataType::Bool {
                1
            } else {
                tag.data_type.byte_size() * tag.length
            };
            total_data_size += byte_count;

            // S7 Item specification (12 bytes)
            let mut item = Vec::with_capacity(12);
            item.push(0x12); // variable specification: address
            item.push(0x0A); // length of following address specification
            item.push(0x10); // syntax ID: S7 any
            item.push(transport_size);
            let num_elements = if tag.data_type == S7DataType::Bool { 1u16 } else { tag.length };
            item.extend_from_slice(&num_elements.to_be_bytes());
            item.push(tag.db_number.to_be_bytes()[0]);
            item.push(tag.db_number.to_be_bytes()[1]);
            item.push(tag.area.code());
            // Offset: 3 bytes (offset * 8 + bit_offset for bits, offset * 8 for others)
            let offset_bits = (tag.offset as u32) * 8 + (tag.bit_offset as u32);
            item.push(((offset_bits >> 16) & 0xFF) as u8);
            item.push(((offset_bits >> 8) & 0xFF) as u8);
            item.push((offset_bits & 0xFF) as u8);

            items.extend_from_slice(&item);
        }

        params.extend_from_slice(&items);

        // Build data area: data header + data
        let mut data = Vec::new();
        data.push(0x00); // return code
        data.push(0xFF); // transport size: byte/word/dword
        let mut data_values = Vec::new();
        for _ in 0..total_data_size {
            data_values.push(0x00);
        }
        data.extend_from_slice(&total_data_size.to_be_bytes());
        data.extend_from_slice(&data_values);

        // Build S7 header + params + data
        let mut s7 = Vec::new();
        s7.push(S7_PROTOCOL_ID);
        s7.push(S7_MSG_TYPE_JOB);
        s7.extend_from_slice(&0x0000u16.to_be_bytes());
        s7.extend_from_slice(&pdu_ref.to_be_bytes());
        s7.extend_from_slice(&(params.len() as u16).to_be_bytes());
        s7.extend_from_slice(&(data.len() as u16).to_be_bytes());
        s7.extend_from_slice(&params);
        s7.extend_from_slice(&data);

        Ok(s7)
    }

    fn parse_read_var_response(&self, data: &[u8], tags: &[S7Tag]) -> Result<Vec<S7Value>> {
        if data.len() < 17 {
            anyhow::bail!("S7 Read Var response too short");
        }

        if data[0] != S7_PROTOCOL_ID {
            anyhow::bail!("Invalid S7 protocol ID: 0x{:02X}", data[0]);
        }

        let error_code = u16::from_be_bytes([data[10], data[11]]);
        if error_code != S7_ERR_NONE {
            anyhow::bail!("S7 Read Var error: 0x{:04X}", error_code);
        }

        let param_len = u16::from_be_bytes([data[6], data[7]]) as usize;
        let data_start = 10 + param_len; // after header(10) + params

        if data.len() < data_start + 1 {
            anyhow::bail!("S7 Read Var response data area too short");
        }

        let mut values = Vec::new();
        let mut offset = data_start;

        for tag in tags {
            if offset >= data.len() {
                values.push(S7Value::Byte(0));
                continue;
            }

            // Read return code
            let return_code = data[offset];
            offset += 1;

            if return_code != 0xFF {
                // Error reading this item
                values.push(S7Value::Byte(0));
                // Skip transport size and data length
                if offset + 3 <= data.len() {
                    offset += 1; // transport size
                    let item_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
                    offset += 2;
                    offset += item_len;
                }
                continue;
            }

            // Transport size
            if offset + 2 > data.len() {
                values.push(S7Value::Byte(0));
                break;
            }
            offset += 1; // transport size

            let item_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            if offset + item_len > data.len() {
                values.push(S7Value::Byte(0));
                break;
            }

            let item_data = &data[offset..offset + item_len];
            let value = Self::decode_value(item_data, tag);
            values.push(value);

            offset += item_len;
        }

        Ok(values)
    }

    fn decode_value(data: &[u8], tag: &S7Tag) -> S7Value {
        match tag.data_type {
            S7DataType::Bool => {
                if data.is_empty() {
                    S7Value::Bool(false)
                } else {
                    S7Value::Bool((data[0] & (1 << tag.bit_offset)) != 0)
                }
            }
            S7DataType::Byte => {
                S7Value::Byte(data.first().copied().unwrap_or(0))
            }
            S7DataType::Word => {
                if data.len() >= 2 {
                    S7Value::Word(u16::from_be_bytes([data[0], data[1]]))
                } else {
                    S7Value::Word(0)
                }
            }
            S7DataType::DWord => {
                if data.len() >= 4 {
                    S7Value::DWord(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
                } else {
                    S7Value::DWord(0)
                }
            }
            S7DataType::Real => {
                if data.len() >= 4 {
                    let bits = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                    S7Value::Real(f32::from_bits(bits))
                } else {
                    S7Value::Real(0.0)
                }
            }
            S7DataType::String => {
                // S7 strings: first byte is max length, second byte is actual length
                if data.len() >= 2 {
                    let actual_len = data[1] as usize;
                    if data.len() >= 2 + actual_len {
                        let s = String::from_utf8_lossy(&data[2..2 + actual_len]);
                        S7Value::String(s.to_string())
                    } else {
                        S7Value::String(String::new())
                    }
                } else {
                    S7Value::String(String::new())
                }
            }
        }
    }

    fn build_write_var_request(&mut self, tag: &S7Tag, value: &S7Value) -> Result<Vec<u8>> {
        let pdu_ref = self.next_pdu_ref();

        // Build parameter: function code + item count + item
        let mut params = Vec::new();
        params.push(S7_WRITE_VAR);
        params.push(0x01); // 1 item

        let transport_size = tag.data_type.transport_size();

        // Item specification (12 bytes)
        let mut item = Vec::with_capacity(12);
        item.push(0x12); // variable specification
        item.push(0x0A); // length
        item.push(0x10); // syntax ID
        item.push(transport_size);
        let num_elements = if tag.data_type == S7DataType::Bool { 1u16 } else { tag.length };
        item.extend_from_slice(&num_elements.to_be_bytes());
        item.push(tag.db_number.to_be_bytes()[0]);
        item.push(tag.db_number.to_be_bytes()[1]);
        item.push(tag.area.code());
        let offset_bits = (tag.offset as u32) * 8 + (tag.bit_offset as u32);
        item.push(((offset_bits >> 16) & 0xFF) as u8);
        item.push(((offset_bits >> 8) & 0xFF) as u8);
        item.push((offset_bits & 0xFF) as u8);

        params.extend_from_slice(&item);

        // Build data: data header + value
        let value_bytes = Self::encode_value(value, tag);
        let mut data = Vec::new();
        data.push(0x00); // return code (success)
        data.push(0x00); // transport size
        data.extend_from_slice(&(value_bytes.len() as u16).to_be_bytes());
        data.extend_from_slice(&value_bytes);

        // Build S7 header + params + data
        let mut s7 = Vec::new();
        s7.push(S7_PROTOCOL_ID);
        s7.push(S7_MSG_TYPE_JOB);
        s7.extend_from_slice(&0x0000u16.to_be_bytes());
        s7.extend_from_slice(&pdu_ref.to_be_bytes());
        s7.extend_from_slice(&(params.len() as u16).to_be_bytes());
        s7.extend_from_slice(&(data.len() as u16).to_be_bytes());
        s7.extend_from_slice(&params);
        s7.extend_from_slice(&data);

        Ok(s7)
    }

    fn encode_value(value: &S7Value, tag: &S7Tag) -> Vec<u8> {
        match value {
            S7Value::Bool(v) => {
                let byte = if *v { 1u8 << tag.bit_offset } else { 0u8 };
                vec![byte]
            }
            S7Value::Byte(v) => vec![*v],
            S7Value::Word(v) => v.to_be_bytes().to_vec(),
            S7Value::DWord(v) => v.to_be_bytes().to_vec(),
            S7Value::Real(v) => v.to_be_bytes().to_vec(),
            S7Value::String(s) => {
                let mut bytes = Vec::new();
                bytes.push(tag.length as u8); // max length
                bytes.push(s.len() as u8); // actual length
                bytes.extend_from_slice(s.as_bytes());
                bytes
            }
        }
    }

    fn parse_write_var_response(&self, data: &[u8]) -> Result<()> {
        if data.len() < 17 {
            anyhow::bail!("S7 Write Var response too short");
        }

        if data[0] != S7_PROTOCOL_ID {
            anyhow::bail!("Invalid S7 protocol ID: 0x{:02X}", data[0]);
        }

        let error_code = u16::from_be_bytes([data[10], data[11]]);
        if error_code != S7_ERR_NONE {
            anyhow::bail!("S7 Write Var error: 0x{:04X}", error_code);
        }

        Ok(())
    }

    async fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        let stream = self.stream.as_mut().context("Not connected")?;
        stream.write_all(data).await
            .context("Failed to send data to S7 PLC")?;
        Ok(())
    }

    async fn send_s7(&mut self, s7_data: &[u8]) -> Result<()> {
        // Wrap S7 data in COTP Data Transfer + TPKT
        let mut cotp = Vec::new();
        cotp.push(0x02); // length (header length = 2 for DT)
        cotp.push(COTP_DT);
        cotp.push(0x80); // EOT flag + TPDU number

        // TPKT wrapper
        let total_len = 4 + cotp.len() + s7_data.len();
        let mut packet = Vec::with_capacity(total_len);
        packet.push(TPKT_VERSION);
        packet.push(0x00);
        packet.extend_from_slice(&(total_len as u16).to_be_bytes());
        packet.extend_from_slice(&cotp);
        packet.extend_from_slice(s7_data);

        self.send_raw(&packet).await
    }

    async fn read_s7_response(&mut self) -> Result<Vec<u8>> {
        let stream = self.stream.as_mut().context("Not connected")?;

        // Read TPKT header
        let mut tpkt_header = [0u8; 4];
        stream.read_exact(&mut tpkt_header).await
            .context("Failed to read S7 TPKT header")?;

        if tpkt_header[0] != TPKT_VERSION {
            anyhow::bail!("Invalid TPKT version in S7 response: {}", tpkt_header[0]);
        }

        let packet_len = u16::from_be_bytes([tpkt_header[2], tpkt_header[3]]) as usize;
        if packet_len < 4 {
            anyhow::bail!("Invalid TPKT packet length in S7 response: {}", packet_len);
        }

        let mut payload = vec![0u8; packet_len - 4];
        stream.read_exact(&mut payload).await
            .context("Failed to read S7 COTP/S7 payload")?;

        // Skip COTP DT header (3 bytes: length, type=0xF0, EOT=0x80)
        if payload.len() < 3 {
            anyhow::bail!("S7 COTP payload too short");
        }

        Ok(payload[3..].to_vec())
    }
}

// ── Poller ──

/// Runtime config for an S7 device, used by the poller.
#[derive(Debug, Clone)]
pub struct S7DeviceConfig {
    pub id: String,
    pub url: String,
    pub rack: u8,
    pub slot: u8,
    pub interval_ms: u64,
    pub tags: Vec<S7Tag>,
}

/// Poll an S7 device in a loop. Intended to be spawned as a tokio task.
pub async fn poll_s7_device(config: S7DeviceConfig, cache: DeviceCache) {
    let device_id = config.id.clone();
    let mut backoff_secs: u64 = 1;

    loop {
        let mut client = S7Client::new(&config.url, config.rack, config.slot);

        match client.connect().await {
            Ok(()) => {
                tracing::info!("S7 device {} connected", device_id);
                backoff_secs = 1;

                loop {
                    let tags_to_read: Vec<S7Tag> = config.tags.clone();
                    match client.read_tags(&tags_to_read).await {
                        Ok(values) => {
                            let mut metrics = HashMap::new();
                            for (i, value) in values.iter().enumerate() {
                                let tag_name = if i < config.tags.len() {
                                    format!("{}_{}", config.tags[i].area.to_string().to_lowercase(),
                                            config.tags[i].offset)
                                } else {
                                    format!("tag_{}", i)
                                };
                                metrics.insert(tag_name, value.to_f64());
                            }
                            cache.update(&device_id, metrics).await;
                        }
                        Err(e) => {
                            tracing::warn!("S7 device {} read error: {}", device_id, e);
                            cache.set_status(&device_id, DeviceStatus::Offline).await;
                            break; // Reconnect
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(config.interval_ms)).await;
                }
            }
            Err(e) => {
                tracing::warn!(
                    "S7 device {} connection failed: {}, retrying in {}s",
                    device_id, e, backoff_secs
                );
                cache.set_status(&device_id, DeviceStatus::Offline).await;
            }
        }

        let _ = client.disconnect().await;
        tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(60);
    }
}

// ── One-shot write function ──

/// Write a single f64 value to an S7 PLC tag.
///
/// Connects to the PLC at `ip:port`, resolves the area string to an S7 area,
/// writes the value as a REAL (4-byte float), and returns a `WriteResult`.
///
/// The `area` parameter accepts strings like "DB", "Input", "Output", "Marker",
/// "Timer", or "Counter".
pub async fn write_tag(
    ip: &str,
    port: u16,
    area: &str,
    db_number: u32,
    offset: u16,
    value: f64,
) -> WriteResult {
    let start = std::time::Instant::now();
    let url = format!("{}:{}", ip, port);

    // Resolve area string
    let s7_area = match parse_area_string(area) {
        Some(a) => a,
        None => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Unknown S7 area: '{}'", area)),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
    };

    // Build an S7Tag for a REAL (f32) write at the given offset
    let tag = S7Tag {
        area: s7_area,
        db_number: db_number as u16,
        offset,
        bit_offset: 0,
        data_type: S7DataType::Real,
        length: 1,
    };

    let s7_value = S7Value::Real(value as f32);

    // Connect with timeout
    let mut client = S7Client::new(&url, 0, 1);
    let connect_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.connect(),
    ).await;

    match connect_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("S7 connection failed: {e}")),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(_) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some("S7 connect timeout (5s)".into()),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    // Write with timeout
    let write_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.write_tag(&tag, &s7_value),
    ).await;

    let _ = client.disconnect().await;

    match write_result {
        Ok(Ok(())) => WriteResult {
            success: true,
            device_response: Some(format!(
                "Wrote {} {} offset {} = {} (REAL)",
                area, db_number, offset, value
            )),
            error: None,
            latency_ms: start.elapsed().as_millis() as u64,
        },
        Ok(Err(e)) => WriteResult {
            success: false,
            device_response: None,
            error: Some(format!("S7 write failed: {e}")),
            latency_ms: start.elapsed().as_millis() as u64,
        },
        Err(_) => WriteResult {
            success: false,
            device_response: None,
            error: Some("S7 write timeout (5s)".into()),
            latency_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Parse an area string (case-insensitive) to an `S7Area`.
fn parse_area_string(s: &str) -> Option<S7Area> {
    match s.to_lowercase().as_str() {
        "db" | "data" => Some(S7Area::DB),
        "input" | "i" => Some(S7Area::Input),
        "output" | "q" | "o" => Some(S7Area::Output),
        "marker" | "m" => Some(S7Area::Marker),
        "timer" | "t" => Some(S7Area::Timer),
        "counter" | "c" => Some(S7Area::Counter),
        _ => None,
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_db_dword() {
        let tag = S7Client::parse_tag_address("DB1.DBD0").unwrap();
        assert_eq!(tag.area, S7Area::DB);
        assert_eq!(tag.db_number, 1);
        assert_eq!(tag.offset, 0);
        assert_eq!(tag.data_type, S7DataType::DWord);
    }

    #[test]
    fn test_parse_db_bool_with_bit() {
        let tag = S7Client::parse_tag_address("DB1.DBX0.1").unwrap();
        assert_eq!(tag.area, S7Area::DB);
        assert_eq!(tag.db_number, 1);
        assert_eq!(tag.offset, 0);
        assert_eq!(tag.bit_offset, 1);
        assert_eq!(tag.data_type, S7DataType::Bool);
    }

    #[test]
    fn test_parse_db_word() {
        let tag = S7Client::parse_tag_address("DB1.DBW0").unwrap();
        assert_eq!(tag.area, S7Area::DB);
        assert_eq!(tag.db_number, 1);
        assert_eq!(tag.offset, 0);
        assert_eq!(tag.data_type, S7DataType::Word);
    }

    #[test]
    fn test_parse_db_byte() {
        let tag = S7Client::parse_tag_address("DB1.DBB10").unwrap();
        assert_eq!(tag.area, S7Area::DB);
        assert_eq!(tag.db_number, 1);
        assert_eq!(tag.offset, 10);
        assert_eq!(tag.data_type, S7DataType::Byte);
    }

    #[test]
    fn test_parse_db_large_offset() {
        let tag = S7Client::parse_tag_address("DB10.DBD100").unwrap();
        assert_eq!(tag.area, S7Area::DB);
        assert_eq!(tag.db_number, 10);
        assert_eq!(tag.offset, 100);
        assert_eq!(tag.data_type, S7DataType::DWord);
    }

    #[test]
    fn test_parse_input_bool() {
        let tag = S7Client::parse_tag_address("I0.0").unwrap();
        assert_eq!(tag.area, S7Area::Input);
        assert_eq!(tag.offset, 0);
        assert_eq!(tag.bit_offset, 0);
        assert_eq!(tag.data_type, S7DataType::Bool);
    }

    #[test]
    fn test_parse_input_bit_nonzero() {
        let tag = S7Client::parse_tag_address("I1.3").unwrap();
        assert_eq!(tag.area, S7Area::Input);
        assert_eq!(tag.offset, 1);
        assert_eq!(tag.bit_offset, 3);
        assert_eq!(tag.data_type, S7DataType::Bool);
    }

    #[test]
    fn test_parse_marker_word() {
        let tag = S7Client::parse_tag_address("MW100").unwrap();
        assert_eq!(tag.area, S7Area::Marker);
        assert_eq!(tag.offset, 100);
        assert_eq!(tag.data_type, S7DataType::Word);
    }

    #[test]
    fn test_parse_output_word() {
        let tag = S7Client::parse_tag_address("QW200").unwrap();
        assert_eq!(tag.area, S7Area::Output);
        assert_eq!(tag.offset, 200);
        assert_eq!(tag.data_type, S7DataType::Word);
    }

    #[test]
    fn test_parse_marker_dword() {
        let tag = S7Client::parse_tag_address("MD100").unwrap();
        assert_eq!(tag.area, S7Area::Marker);
        assert_eq!(tag.offset, 100);
        assert_eq!(tag.data_type, S7DataType::DWord);
    }

    #[test]
    fn test_parse_marker_byte() {
        let tag = S7Client::parse_tag_address("MB50").unwrap();
        assert_eq!(tag.area, S7Area::Marker);
        assert_eq!(tag.offset, 50);
        assert_eq!(tag.data_type, S7DataType::Byte);
    }

    #[test]
    fn test_parse_output_bool() {
        let tag = S7Client::parse_tag_address("Q0.0").unwrap();
        assert_eq!(tag.area, S7Area::Output);
        assert_eq!(tag.offset, 0);
        assert_eq!(tag.bit_offset, 0);
        assert_eq!(tag.data_type, S7DataType::Bool);
    }

    #[test]
    fn test_parse_invalid_address() {
        assert!(S7Client::parse_tag_address("").is_err());
        assert!(S7Client::parse_tag_address("XYZ").is_err());
    }

    #[test]
    fn test_parse_case_insensitive() {
        let tag = S7Client::parse_tag_address("db1.dbw10").unwrap();
        assert_eq!(tag.area, S7Area::DB);
        assert_eq!(tag.db_number, 1);
        assert_eq!(tag.offset, 10);
        assert_eq!(tag.data_type, S7DataType::Word);
    }

    #[test]
    fn test_tpkt_cotp_packet_construction() {
        let client = S7Client::new("127.0.0.1:102", 0, 1);
        let packet = client.build_cotp_cr().unwrap();

        // TPKT header
        assert_eq!(packet[0], 3); // version
        assert_eq!(packet[1], 0); // reserved
        let total_len = u16::from_be_bytes([packet[2], packet[3]]);
        assert_eq!(total_len as usize, packet.len());

        // COTP CR
        assert!(packet[4] > 0); // COTP length
        assert_eq!(packet[5], 0xE0); // CR PDU type
    }

    #[test]
    fn test_s7_setup_comm_construction() {
        let mut client = S7Client::new("127.0.0.1:102", 0, 1);
        let packet = client.build_s7_setup_comm();

        assert_eq!(packet[0], 0x32); // protocol ID
        assert_eq!(packet[1], S7_MSG_TYPE_JOB); // msg type
        assert_eq!(packet[6], 0x00); // param length high byte is 0 since params are small
        // Function code in params
        assert_eq!(packet[10], S7_SETUP_COMM);
    }

    #[test]
    fn test_s7_read_request_construction() {
        let mut client = S7Client::new("127.0.0.1:102", 0, 1);
        let tags = vec![S7Tag {
            area: S7Area::DB,
            db_number: 1,
            offset: 0,
            bit_offset: 0,
            data_type: S7DataType::DWord,
            length: 1,
        }];
        let packet = client.build_read_var_request(&tags).unwrap();

        assert_eq!(packet[0], 0x32); // protocol ID
        assert_eq!(packet[1], S7_MSG_TYPE_JOB);
        // Params start at byte 10
        assert_eq!(packet[10], S7_READ_VAR); // function code
        assert_eq!(packet[11], 0x01); // item count
    }

    #[test]
    fn test_s7_write_request_construction() {
        let mut client = S7Client::new("127.0.0.1:102", 0, 1);
        let tag = S7Tag {
            area: S7Area::DB,
            db_number: 1,
            offset: 0,
            bit_offset: 0,
            data_type: S7DataType::Word,
            length: 1,
        };
        let packet = client.build_write_var_request(&tag, &S7Value::Word(1234)).unwrap();

        assert_eq!(packet[0], 0x32); // protocol ID
        assert_eq!(packet[1], S7_MSG_TYPE_JOB);
        // Params start at byte 10
        assert_eq!(packet[10], S7_WRITE_VAR);
    }

    #[test]
    fn test_s7_value_bool() {
        let v = S7Value::Bool(true);
        assert!((v.to_f64() - 1.0).abs() < f64::EPSILON);
        let v = S7Value::Bool(false);
        assert!((v.to_f64() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_s7_value_word() {
        let v = S7Value::Word(65535);
        assert!((v.to_f64() - 65535.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_s7_value_real() {
        let v = S7Value::Real(3.14);
        assert!((v.to_f64() - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_s7_value_string() {
        let v = S7Value::String("hello".to_string());
        assert!((v.to_f64() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_s7_value_serde() {
        let v = S7Value::DWord(42);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "42");

        let v = S7Value::Bool(true);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "true");

        let v = S7Value::Real(1.5);
        let json = serde_json::to_string(&v).unwrap();
        assert!(!json.is_empty());

        let v = S7Value::String("test".to_string());
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"test\"");
    }

    #[test]
    fn test_s7_area_codes() {
        assert_eq!(S7Area::DB.code(), 0x84);
        assert_eq!(S7Area::Input.code(), 0x81);
        assert_eq!(S7Area::Output.code(), 0x82);
        assert_eq!(S7Area::Marker.code(), 0x83);
        assert_eq!(S7Area::Timer.code(), 0x1D);
        assert_eq!(S7Area::Counter.code(), 0x1C);

        assert_eq!(S7Area::from_code(0x84), Some(S7Area::DB));
        assert_eq!(S7Area::from_code(0xFF), None);
    }

    #[test]
    fn test_s7_data_type_transport_sizes() {
        assert_eq!(S7DataType::Bool.transport_size(), 0x01);
        assert_eq!(S7DataType::Byte.transport_size(), 0x02);
        assert_eq!(S7DataType::Word.transport_size(), 0x04);
        assert_eq!(S7DataType::DWord.transport_size(), 0x06);
        assert_eq!(S7DataType::Real.transport_size(), 0x08);
    }

    #[test]
    fn test_s7_data_type_from_str() {
        assert_eq!(S7DataType::from_str_name("bool"), Some(S7DataType::Bool));
        assert_eq!(S7DataType::from_str_name("Word"), Some(S7DataType::Word));
        assert_eq!(S7DataType::from_str_name("REAL"), Some(S7DataType::Real));
        assert_eq!(S7DataType::from_str_name("float32"), Some(S7DataType::Real));
        assert_eq!(S7DataType::from_str_name("unknown"), None);
    }

    #[test]
    fn test_decode_value_word_be() {
        let data = [0x00, 0xFF]; // 255 big-endian
        let tag = S7Tag {
            area: S7Area::DB, db_number: 1, offset: 0, bit_offset: 0,
            data_type: S7DataType::Word, length: 1,
        };
        let val = S7Client::decode_value(&data, &tag);
        assert_eq!(val, S7Value::Word(255));
    }

    #[test]
    fn test_decode_value_real() {
        let data = f32::to_be_bytes(25.5);
        let tag = S7Tag {
            area: S7Area::DB, db_number: 1, offset: 0, bit_offset: 0,
            data_type: S7DataType::Real, length: 1,
        };
        let val = S7Client::decode_value(&data, &tag);
        match val {
            S7Value::Real(f) => assert!((f - 25.5).abs() < 0.001),
            _ => panic!("Expected Real value"),
        }
    }

    #[test]
    fn test_encode_value_word() {
        let tag = S7Tag {
            area: S7Area::DB, db_number: 1, offset: 0, bit_offset: 0,
            data_type: S7DataType::Word, length: 1,
        };
        let bytes = S7Client::encode_value(&S7Value::Word(0x1234), &tag);
        assert_eq!(bytes, vec![0x12, 0x34]);
    }

    #[test]
    fn test_encode_value_bool_bit_offset() {
        let tag = S7Tag {
            area: S7Area::DB, db_number: 1, offset: 0, bit_offset: 3,
            data_type: S7DataType::Bool, length: 1,
        };
        let bytes = S7Client::encode_value(&S7Value::Bool(true), &tag);
        assert_eq!(bytes, vec![0x08]); // 1 << 3
    }
}
