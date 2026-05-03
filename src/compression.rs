// src/compression.rs

use anyhow::{bail, Context, Result};

/// Bit-level writer for encoding compressed data.
pub struct BitWriter {
    buffer: Vec<u8>,
    current_byte: u8,
    bit_pos: u8, // 0..7, number of bits written in current_byte
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            current_byte: 0,
            bit_pos: 0,
        }
    }

    /// Write `n` bits from `value` (MSB first).
    pub fn write_bits(&mut self, value: u64, mut n: u8) {
        while n > 0 {
            let remaining = 8 - self.bit_pos;
            let bits_to_write = remaining.min(n);
            let shift = n - bits_to_write;
            let bits = ((value >> shift) & ((1u64 << bits_to_write) - 1)) as u8;
            self.current_byte |= bits << (remaining - bits_to_write);
            self.bit_pos += bits_to_write;
            n -= bits_to_write;
            if self.bit_pos == 8 {
                self.buffer.push(self.current_byte);
                self.current_byte = 0;
                self.bit_pos = 0;
            }
        }
    }

    /// Write a single bit.
    pub fn write_bit(&mut self, bit: bool) {
        self.write_bits(if bit { 1 } else { 0 }, 1);
    }

    /// Finalize and return the written bytes.
    pub fn finish(mut self) -> Vec<u8> {
        if self.bit_pos > 0 {
            self.buffer.push(self.current_byte);
        }
        self.buffer
    }
}

/// Bit-level reader for decoding compressed data.
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8, // 0..7, next bit to read in current byte
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    /// Read `n` bits and return as u64 (MSB first).
    pub fn read_bits(&mut self, mut n: u8) -> Option<u64> {
        let mut result: u64 = 0;
        while n > 0 {
            if self.byte_pos >= self.data.len() {
                return None;
            }
            let remaining = 8 - self.bit_pos;
            let bits_to_read = remaining.min(n);
            let shift = remaining - bits_to_read;
            let mask = (((1u32 << bits_to_read) - 1) << shift) as u8;
            let bits = (self.data[self.byte_pos] & mask) >> shift;
            result = (result << bits_to_read) | (bits as u64);
            self.bit_pos += bits_to_read;
            n -= bits_to_read;
            if self.bit_pos == 8 {
                self.byte_pos += 1;
                self.bit_pos = 0;
            }
        }
        Some(result)
    }

    /// Read a single bit.
    pub fn read_bit(&mut self) -> Option<bool> {
        self.read_bits(1).map(|v| v == 1)
    }

    /// Check if there are more bits to read.
    pub fn has_remaining(&self) -> bool {
        self.byte_pos < self.data.len()
    }
}

// ---------------------------------------------------------------------------
// Gorilla encoding (Facebook Gorilla paper)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GorillaBlock {
    pub first_ts: i64,
    pub count: usize,
    pub first_value: f64,
    pub data: Vec<u8>,
}

/// Encode timestamps and values using Gorilla compression.
///
/// Timestamps are encoded with delta-of-delta. Values are encoded with XOR.
pub fn gorilla_encode(timestamps: &[i64], values: &[f64]) -> Result<GorillaBlock> {
    if timestamps.is_empty() || values.is_empty() {
        bail!("empty input: cannot encode zero data points");
    }
    if timestamps.len() != values.len() {
        bail!(
            "mismatched lengths: {} timestamps vs {} values",
            timestamps.len(),
            values.len()
        );
    }
    let count = timestamps.len();
    let first_ts = timestamps[0];
    let first_value = values[0];

    if count == 1 {
        return Ok(GorillaBlock {
            first_ts,
            count,
            first_value,
            data: Vec::new(),
        });
    }

    let mut writer = BitWriter::new();

    // --- Timestamp encoding (delta-of-delta) ---
    // The implicit "previous delta" before the first real delta is 0.
    // For each element i >= 1, dod = delta_i - prev_delta.
    let mut prev_delta: i64 = 0;
    for i in 1..count {
        let delta = timestamps[i] - timestamps[i - 1];
        let dod = delta - prev_delta;
        encode_dod(&mut writer, dod);
        prev_delta = delta;
    }

    // --- Value encoding (XOR) ---
    let mut prev_xor: u64 = 0;
    let mut prev_lz: u8 = 0;
    let mut prev_tz: u8 = 0;

    for i in 1..count {
        let cur_bits = values[i].to_bits();
        let prev_bits = values[i - 1].to_bits();
        let xor = cur_bits ^ prev_bits;

        if xor == 0 {
            writer.write_bit(false);
        } else {
            writer.write_bit(true);

            let lz = xor.leading_zeros() as u8;
            let tz = xor.trailing_zeros() as u8;
            let meaningful_bits = 64 - lz - tz;

            // Try to reuse previous window
            if prev_xor != 0 && lz >= prev_lz && tz >= prev_tz {
                writer.write_bit(false);
                writer.write_bits(xor >> tz, meaningful_bits);
            } else {
                writer.write_bit(true);
                writer.write_bits(lz as u64, 6);
                writer.write_bits((meaningful_bits - 1) as u64, 6);
                writer.write_bits(xor >> tz, meaningful_bits);
            }

            prev_xor = xor;
            prev_lz = lz;
            prev_tz = tz;
        }
    }

    let data = writer.finish();

    Ok(GorillaBlock {
        first_ts,
        count,
        first_value,
        data,
    })
}

/// Decode a Gorilla block back into timestamps and values.
pub fn gorilla_decode(block: &GorillaBlock) -> Result<(Vec<i64>, Vec<f64>)> {
    let count = block.count;
    if count == 0 {
        bail!("cannot decode block with count 0");
    }

    let mut timestamps = Vec::with_capacity(count);
    let mut values = Vec::with_capacity(count);

    timestamps.push(block.first_ts);
    values.push(block.first_value);

    if count == 1 {
        return Ok((timestamps, values));
    }

    let mut reader = BitReader::new(&block.data);

    // --- Timestamp decoding ---
    let mut prev_delta: i64 = 0;
    for i in 1..count {
        let dod = decode_dod(&mut reader).context("truncated timestamp data")?;
        let delta = prev_delta + dod;
        let ts = timestamps[i - 1] + delta;
        timestamps.push(ts);
        prev_delta = delta;
    }

    // --- Value decoding ---
    let mut prev_bits = block.first_value.to_bits();
    let mut prev_lz: u8 = 0;
    let mut prev_tz: u8 = 0;

    for _ in 1..count {
        let xor_nonzero = reader
            .read_bit()
            .context("truncated value control bit")?;
        if !xor_nonzero {
            // xor == 0, same value as previous
            values.push(f64::from_bits(prev_bits));
        } else {
            let reuse_window = reader
                .read_bit()
                .context("truncated value window bit")?;

            let xor;
            if reuse_window {
                // New window
                let lz = reader
                    .read_bits(6)
                    .context("truncated leading zeros")? as u8;
                let meaningful_count = reader
                    .read_bits(6)
                    .context("truncated meaningful count")?
                    as u8
                    + 1;
                let meaningful_bits = reader
                    .read_bits(meaningful_count)
                    .context("truncated meaningful value bits")?;
                xor = meaningful_bits << (64 - lz - meaningful_count);
                prev_lz = lz;
                prev_tz = 64 - lz - meaningful_count;
            } else {
                // Reuse previous window
                let meaningful_count = 64 - prev_lz - prev_tz;
                let meaningful_bits = reader
                    .read_bits(meaningful_count)
                    .context("truncated meaningful value bits (reuse)")?;
                xor = meaningful_bits << prev_tz;
            }

            let cur_bits = prev_bits ^ xor;
            values.push(f64::from_bits(cur_bits));
            prev_bits = cur_bits;
        }
    }

    Ok((timestamps, values))
}

/// Encode a delta-of-delta value into the bit writer.
fn encode_dod(writer: &mut BitWriter, dod: i64) {
    if dod == 0 {
        // 0
        writer.write_bit(false);
    } else {
        // 1...
        writer.write_bit(true);
        let abs = dod.unsigned_abs();
        if abs < 64 {
            // 10 + 7 bits (signed, offset by 64: maps -64..63 to 0..127)
            writer.write_bit(false);
            writer.write_bits((dod as u64).wrapping_add(64) & 0x7F, 7);
        } else if abs < 256 {
            // 110 + 9 bits (signed, offset by 256: maps -256..255 to 0..511)
            writer.write_bit(true);
            writer.write_bit(false);
            writer.write_bits((dod as u64).wrapping_add(256) & 0x1FF, 9);
        } else if abs < 2048 {
            // 1110 + 12 bits (signed, offset by 2048)
            writer.write_bit(true);
            writer.write_bit(true);
            writer.write_bit(false);
            writer.write_bits((dod as u64).wrapping_add(2048) & 0xFFF, 12);
        } else {
            // 1111 + 32 bits (signed, stored as raw i32 bits)
            writer.write_bit(true);
            writer.write_bit(true);
            writer.write_bit(true);
            writer.write_bits(dod as u32 as u64, 32);
        }
    }
}

/// Decode a delta-of-delta value from the bit reader.
fn decode_dod(reader: &mut BitReader) -> Option<i64> {
    let first = reader.read_bit()?;
    if !first {
        // 0 → dod == 0
        return Some(0);
    }

    let second = reader.read_bit()?;
    if !second {
        // 10 prefix: 7 bits, signed with offset 64
        let raw = reader.read_bits(7)? as i64;
        Some(raw - 64)
    } else {
        let third = reader.read_bit()?;
        if !third {
            // 110 prefix: 9 bits, signed with offset 256
            let raw = reader.read_bits(9)? as i64;
            Some(raw - 256)
        } else {
            let fourth = reader.read_bit()?;
            if !fourth {
                // 1110 prefix: 12 bits, signed with offset 2048
                let raw = reader.read_bits(12)? as i64;
                Some(raw - 2048)
            } else {
                // 1111 prefix: 32 bits, signed i32
                let raw = reader.read_bits(32)? as u32 as i32 as i64;
                Some(raw)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// RLE (Run-Length Encoding) for digital/binary tags
// ---------------------------------------------------------------------------

/// RLE-encode boolean values into (value, count) pairs.
pub fn rle_encode(values: &[bool]) -> Vec<(bool, u32)> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = values[0];
    let mut count: u32 = 1;
    for &v in &values[1..] {
        if v == current && count < u32::MAX {
            count += 1;
        } else {
            result.push((current, count));
            current = v;
            count = 1;
        }
    }
    result.push((current, count));
    result
}

/// Decode RLE (value, count) pairs back to booleans.
pub fn rle_decode(runs: &[(bool, u32)]) -> Vec<bool> {
    let total: usize = runs.iter().map(|(_, c)| *c as usize).sum();
    let mut result = Vec::with_capacity(total);
    for (value, count) in runs {
        for _ in 0..*count {
            result.push(*value);
        }
    }
    result
}

/// Serialize RLE runs to binary: per run: 1 byte (0/1) + 4 bytes (u32 LE)
pub fn rle_serialize(runs: &[(bool, u32)]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(runs.len() * 5);
    for (value, count) in runs {
        buf.push(if *value { 1 } else { 0 });
        buf.extend_from_slice(&count.to_le_bytes());
    }
    buf
}

/// Deserialize RLE runs from binary.
pub fn rle_deserialize(data: &[u8]) -> Result<Vec<(bool, u32)>> {
    if data.len() % 5 != 0 {
        bail!("Invalid RLE data length: {}", data.len());
    }
    let mut runs = Vec::with_capacity(data.len() / 5);
    for chunk in data.chunks_exact(5) {
        let value = chunk[0] != 0;
        let count = u32::from_le_bytes([chunk[1], chunk[2], chunk[3], chunk[4]]);
        runs.push((value, count));
    }
    Ok(runs)
}

// ---------------------------------------------------------------------------
// Varint encoding (zigzag + LEB128) for warm tier deltas
// ---------------------------------------------------------------------------

fn zigzag_encode(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

fn zigzag_decode(n: u64) -> i64 {
    ((n >> 1) as i64) ^ -((n & 1) as i64)
}

pub fn varint_encode(values: &[i64]) -> Vec<u8> {
    let mut buf = Vec::new();
    for &v in values {
        let mut n = zigzag_encode(v);
        loop {
            let byte = (n & 0x7F) as u8;
            n >>= 7;
            if n == 0 {
                buf.push(byte);
                break;
            }
            buf.push(byte | 0x80);
        }
    }
    buf
}

pub fn varint_decode(data: &[u8]) -> anyhow::Result<Vec<i64>> {
    let mut values = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let mut n: u64 = 0;
        let mut shift: u32 = 0;
        loop {
            if i >= data.len() {
                anyhow::bail!("Truncated varint");
            }
            let byte = data[i];
            i += 1;
            n |= ((byte & 0x7F) as u64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
        }
        values.push(zigzag_decode(n));
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_roundtrip_single_bits() {
        let mut writer = BitWriter::new();
        for i in 0..16 {
            writer.write_bit(i % 2 == 0);
        }
        let data = writer.finish();

        let mut reader = BitReader::new(&data);
        for i in 0..16 {
            assert_eq!(reader.read_bit(), Some(i % 2 == 0));
        }
    }

    #[test]
    fn test_write_read_roundtrip_multi_bit() {
        let mut writer = BitWriter::new();
        writer.write_bits(0b1010, 4);
        writer.write_bits(0b11001100, 8);
        writer.write_bits(0b111, 3);
        let data = writer.finish();

        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_bits(4), Some(0b1010));
        assert_eq!(reader.read_bits(8), Some(0b11001100));
        assert_eq!(reader.read_bits(3), Some(0b111));
    }

    #[test]
    fn test_write_read_large_value() {
        let mut writer = BitWriter::new();
        writer.write_bits(0xDEADBEEF, 32);
        let data = writer.finish();

        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_bits(32), Some(0xDEADBEEF));
    }

    #[test]
    fn test_empty_writer() {
        let writer = BitWriter::new();
        assert!(writer.finish().is_empty());
    }

    #[test]
    fn test_reader_past_end_returns_none() {
        let mut writer = BitWriter::new();
        writer.write_bits(0xFF, 8);
        let data = writer.finish();

        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_bits(8), Some(0xFF));
        assert_eq!(reader.read_bits(1), None);
    }

    // -----------------------------------------------------------------------
    // Gorilla encoder/decoder tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_gorilla_encode_decode_roundtrip() {
        // 10 SCADA-like readings: ~1s intervals, small variations, some duplicates
        let timestamps: Vec<i64> = vec![
            1_700_000_000,
            1_700_000_001,
            1_700_000_002,
            1_700_000_003,
            1_700_000_003, // duplicate timestamp (delta-of-delta = -1)
            1_700_000_005, // 2s gap
            1_700_000_006,
            1_700_000_007,
            1_700_000_008,
            1_700_000_009,
        ];
        let values: Vec<f64> = vec![
            23.5,
            23.6,
            23.6,  // duplicate value
            23.7,
            23.7,  // duplicate value
            23.5,  // back to earlier value
            24.0,
            24.1,
            24.1,  // duplicate value
            24.2,
        ];

        let block =
            gorilla_encode(&timestamps, &values).expect("encode should succeed");
        assert_eq!(block.count, 10);
        assert_eq!(block.first_ts, 1_700_000_000);

        let (dec_ts, dec_vals) =
            gorilla_decode(&block).expect("decode should succeed");

        assert_eq!(dec_ts, timestamps);
        assert_eq!(dec_vals, values);
    }

    #[test]
    fn test_gorilla_encode_empty_input() {
        let ts: Vec<i64> = vec![];
        let vals: Vec<f64> = vec![];
        assert!(gorilla_encode(&ts, &vals).is_err());

        // Also mismatched lengths
        let ts2 = vec![1i64];
        let vals2: Vec<f64> = vec![];
        assert!(gorilla_encode(&ts2, &vals2).is_err());
    }

    #[test]
    fn test_gorilla_single_value() {
        let timestamps = vec![1234567890i64];
        let values = vec![42.0f64];

        let block =
            gorilla_encode(&timestamps, &values).expect("encode single value");
        assert_eq!(block.count, 1);
        assert_eq!(block.first_ts, 1234567890);
        assert!(block.data.is_empty());

        let (dec_ts, dec_vals) =
            gorilla_decode(&block).expect("decode single value");
        assert_eq!(dec_ts, timestamps);
        assert_eq!(dec_vals, values);
    }

    // -----------------------------------------------------------------------
    // RLE encoder/decoder tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_rle_encode_decode_roundtrip() {
        // Pump running 8 hours at 1Hz then off 1 hour
        let mut values: Vec<bool> = vec![true; 28800];
        values.extend(vec![false; 3600]);
        let encoded = rle_encode(&values);
        let decoded = rle_decode(&encoded);
        assert_eq!(decoded, values);
        assert!(
            encoded.len() < 20,
            "RLE should compress to <20 bytes, got {}",
            encoded.len()
        );
    }

    #[test]
    fn test_rle_alternating() {
        let values: Vec<bool> = (0..100).map(|i| i % 2 == 0).collect();
        let encoded = rle_encode(&values);
        let decoded = rle_decode(&encoded);
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_rle_single_value() {
        let values = vec![true];
        let encoded = rle_encode(&values);
        let decoded = rle_decode(&encoded);
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_rle_all_same() {
        let values = vec![false; 100000];
        let encoded = rle_encode(&values);
        let decoded = rle_decode(&encoded);
        assert_eq!(decoded, values);
        assert!(encoded.len() < 10);
    }

    #[test]
    fn test_rle_serialize_deserialize_roundtrip() {
        let runs = vec![(true, 28800u32), (false, 3600u32)];
        let data = rle_serialize(&runs);
        let decoded = rle_deserialize(&data).unwrap();
        assert_eq!(decoded, runs);
    }

    // -----------------------------------------------------------------------
    // Varint encoder/decoder tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_varint_roundtrip() {
        let values: Vec<i64> = vec![0, 1, -1, 127, -128, 16383, -16384, 1000000, -1000000];
        let encoded = varint_encode(&values);
        let decoded = varint_decode(&encoded).unwrap();
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_varint_small_values() {
        let values: Vec<i64> = vec![0, 0, 0, 1, 0, -1, 0, 2, -1, 0];
        let encoded = varint_encode(&values);
        assert!(encoded.len() < 20, "Small deltas should be very compact");
        let decoded = varint_decode(&encoded).unwrap();
        assert_eq!(decoded, values);
    }

    // -----------------------------------------------------------------------
    // Compression ratio integration test
    // -----------------------------------------------------------------------

    #[test]
    fn test_gorilla_compression_ratio() {
        // 1 hour of SCADA data (3600 readings) at 1s intervals.
        // Realistic sensor pattern: mostly constant temperature with occasional
        // small step changes. Gorilla encoding excels here because XOR deltas
        // are zero for repeated values and small for nearby values, while
        // constant-rate timestamps compress to ~1 bit each.
        let mut values = Vec::with_capacity(3600);
        let mut current = 25.0f64;
        for i in 0..3600 {
            // Mostly constant, with a small step every ~60s
            if i > 0 && i % 60 == 0 {
                current += 0.1;
            }
            values.push(current);
        }

        let timestamps: Vec<i64> = (0..3600).map(|i| 1700000000000 + i * 1000).collect();
        let block = gorilla_encode(&timestamps, &values).unwrap();
        let raw_size = timestamps.len() * 16; // 8 bytes ts + 8 bytes value
        let compressed_size = block.data.len();
        let ratio = raw_size as f64 / compressed_size as f64;

        assert!(ratio > 5.0, "Compression ratio too low: {:.1}:1", ratio);
    }
}
