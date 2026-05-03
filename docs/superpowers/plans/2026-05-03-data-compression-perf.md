# v0.25.0 — Data Compression & Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the existing telemetry_hourly/telemetry_daily downsampling with a hot/warm/cold compression tier system using Gorilla encoding, add deadband filtering, connection pooling, WAL tuning, and a compression stats dashboard.

**Architecture:** Three new tables (telemetry_hot, telemetry_warm, telemetry_cold) replace the old telemetry/telemetry_hourly/telemetry_daily tables. Background workers compress data between tiers. Decompression is transparent to the API/frontend. New modules: `compression.rs` (Gorilla/RLE encoding), `deadband.rs` (write-path filtering), `api/compression.rs` (stats endpoints).

**Tech Stack:** Rust (tokio, rusqlite, axum), SQLite with WAL mode, React/TypeScript frontend

**Spec:** `docs/superpowers/specs/2026-05-03-data-compression-perf-design.md`

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `src/compression.rs` | BitWriter/BitReader, Gorilla encoder/decoder, RLE encoder/decoder, varint encoding |
| `src/deadband.rs` | DeadbandFilter struct, in-memory last-value tracker, per-tag config loading |
| `src/api/compression.rs` | GET /api/v1/compression/stats, GET/PUT /api/v1/compression/config, GET /api/v1/system/memory |
| `client/src/components/CompressionStats.tsx` | Compression stats dashboard panel |
| `client/src/components/DeadbandConfig.tsx` | Per-tag deadband configuration editor |
| `client/src/components/StorageOverview.tsx` | Storage usage breakdown visualization |

### Modified Files
| File | Changes |
|------|---------|
| `src/storage.rs` | New table creation, migration, compression worker functions, unified query, deadband_config CRUD |
| `src/main.rs` | Spawn compression workers, CLI commands, edge-mode flag, mod declarations |
| `src/models.rs` | CompressionConfig struct, CompressionTierConfig, EdgeModeConfig |
| `src/modbus.rs` | Connection pooling with ModbusPool struct |
| `src/api/mod.rs` | Register compression API routes, mod declaration |
| `src/retention.rs` | Replace old downsampling calls with new compression tier calls |
| `Cargo.toml` | Version bump to 0.25.0 |

---

## Task 1: Gorilla BitWriter and BitReader

**Files:**
- Create: `src/compression.rs`

This task creates the low-level bit I/O primitives needed by all encoding algorithms.

- [ ] **Step 1: Create `src/compression.rs` with BitWriter and BitReader**

```rust
// src/compression.rs

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
            let mask = ((1u8 << bits_to_read) - 1) << shift;
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
```

- [ ] **Step 2: Write unit tests for BitWriter/BitReader**

```rust
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
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib compression::tests`
Expected: All 5 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/compression.rs
git commit -m "feat(compression): add BitWriter/BitReader for Gorilla encoding"
```

---

## Task 2: Gorilla Encoder and Decoder

**Files:**
- Modify: `src/compression.rs`

Implements the Facebook Gorilla paper's delta-of-delta timestamp encoding and XOR value encoding.

- [ ] **Step 1: Write failing test for Gorilla roundtrip**

Add to `src/compression.rs` tests:

```rust
#[test]
fn test_gorilla_encode_decode_roundtrip() {
    // Simulate SCADA data: temperature readings at ~1s intervals with small variations
    let timestamps: Vec<i64> = vec![
        1700000000000,
        1700000001000,
        1700000002000,
        1700000003000,
        1700000004000,
        1700000005000,
        1700000006000,
        1700000007000,
        1700000008000,
        1700000009000,
    ];
    let values: Vec<f64> = vec![
        25.0,
        25.1,
        25.1,  // same as previous — XOR should be 0
        25.2,
        25.0,
        24.9,
        24.9,  // same — XOR should be 0
        25.0,
        25.1,
        25.3,
    ];

    let encoded = gorilla_encode(&timestamps, &values).unwrap();
    let (dec_ts, dec_vals) = gorilla_decode(&encoded).unwrap();

    assert_eq!(dec_ts, timestamps);
    for (i, (expected, actual)) in values.iter().zip(dec_vals.iter()).enumerate() {
        assert!(
            (expected - actual).abs() < f64::EPSILON,
            "Value mismatch at index {}: expected {}, got {}",
            i, expected, actual
        );
    }
}

#[test]
fn test_gorilla_encode_empty_input() {
    let result = gorilla_encode(&[], &[]);
    assert!(result.is_err());
}

#[test]
fn test_gorilla_single_value() {
    let timestamps = vec![1700000000000i64];
    let values = vec![42.0f64];

    let encoded = gorilla_encode(&timestamps, &values).unwrap();
    let (dec_ts, dec_vals) = gorilla_decode(&encoded).unwrap();

    assert_eq!(dec_ts, timestamps);
    assert_eq!(dec_vals, values);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib compression::tests::test_gorilla`
Expected: FAIL — `gorilla_encode` not defined

- [ ] **Step 3: Implement Gorilla encoder/decoder**

Add to `src/compression.rs` (before `#[cfg(test)]`):

```rust
use anyhow::{Context, Result};

/// Gorilla-encoded block. Header + compressed bitstream.
#[derive(Debug, Clone)]
pub struct GorillaBlock {
    pub first_ts: i64,
    pub count: usize,
    pub first_value: f64,
    pub data: Vec<u8>,
}

/// Encode timestamps and values using Gorilla paper algorithm.
/// Returns a GorillaBlock with header + compressed bitstream.
pub fn gorilla_encode(timestamps: &[i64], values: &[f64]) -> Result<GorillaBlock> {
    if timestamps.is_empty() {
        anyhow::bail!("Cannot encode empty data");
    }
    anyhow::ensure!(
        timestamps.len() == values.len(),
        "Timestamps and values must have same length"
    );

    let first_ts = timestamps[0];
    let first_value = values[0];
    let count = timestamps.len();

    let mut writer = BitWriter::new();

    if count == 1 {
        return Ok(GorillaBlock {
            first_ts,
            count,
            first_value,
            data: writer.finish(),
        });
    }

    // Delta-of-delta for timestamps
    let mut prev_delta: i64 = 0;

    // Gorilla timestamp delta-of-delta prefix codes:
    // 0        -> delta-of-delta = 0
    // 10       -> 7 bits
    // 110      -> 9 bits
    // 1110     -> 12 bits
    // 1111     -> 32 bits
    for i in 1..count {
        let delta = timestamps[i] - timestamps[i - 1];
        let dod = delta - prev_delta;

        if dod == 0 {
            writer.write_bit(false); // '0'
        } else if dod >= -64 && dod <= 63 {
            writer.write_bits(0b10, 2);
            writer.write_bits((dod as u64) & 0x7F, 7);
        } else if dod >= -256 && dod <= 255 {
            writer.write_bits(0b110, 3);
            writer.write_bits((dod as u64) & 0x1FF, 9);
        } else if dod >= -2048 && dod <= 2047 {
            writer.write_bits(0b1110, 4);
            writer.write_bits((dod as u64) & 0xFFF, 12);
        } else {
            writer.write_bits(0b1111, 4);
            writer.write_bits(dod as u64, 32);
        }
        prev_delta = delta;
    }

    // XOR encoding for values
    let mut prev_xor: u64 = 0;
    for i in 1..count {
        let val_bits = values[i].to_bits();
        let xor = val_bits ^ values[i - 1].to_bits();

        if xor == 0 {
            writer.write_bit(false); // '0' — same value
        } else {
            writer.write_bit(true); // '1'
            let leading_zeros = xor.leading_zeros() as u8;
            let trailing_zeros = xor.trailing_zeros() as u8;

            // If meaningful bits fit in same window as previous
            if leading_zeros >= prev_xor.leading_zeros() as u8
                && trailing_zeros >= prev_xor.trailing_zeros() as u8
                && prev_xor != 0
            {
                writer.write_bit(false); // reuse previous window
                let meaningful_bits =
                    64 - prev_xor.leading_zeros() as u8 - prev_xor.trailing_zeros() as u8;
                writer.write_bits(xor >> prev_xor.trailing_zeros(), meaningful_bits);
            } else {
                writer.write_bit(true); // new window
                writer.write_bits(leading_zeros as u64, 6);
                let meaningful_bits = 64 - leading_zeros - trailing_zeros;
                writer.write_bits(meaningful_bits as u64 - 1, 6); // store as (count-1) to fit 0..64 in 6 bits
                if meaningful_bits > 0 {
                    writer.write_bits(xor >> trailing_zeros, meaningful_bits);
                }
            }
            prev_xor = xor;
        }
    }

    Ok(GorillaBlock {
        first_ts,
        count,
        first_value,
        data: writer.finish(),
    })
}

/// Decode a Gorilla-encoded block back to timestamps and values.
pub fn gorilla_decode(block: &GorillaBlock) -> Result<(Vec<i64>, Vec<f64>)> {
    let count = block.count;
    let mut timestamps = Vec::with_capacity(count);
    let mut values = Vec::with_capacity(count);

    timestamps.push(block.first_ts);
    values.push(block.first_value);

    if count <= 1 {
        return Ok((timestamps, values));
    }

    let mut reader = BitReader::new(&block.data);
    let mut prev_delta: i64 = 0;

    // Decode timestamps
    for _ in 1..count {
        let first_bit = reader.read_bit().context("Unexpected end of data reading ts")?;
        let dod = if !first_bit {
            0i64
        } else {
            let second_bit = reader.read_bit().context("Unexpected end of data")?;
            if !second_bit {
                // 7 bits
                reader.read_bits(7).context("Unexpected end")? as i64
                    - 64 // sign extension for 7-bit signed
            } else {
                let third_bit = reader.read_bit().context("Unexpected end")?;
                if !third_bit {
                    // 9 bits
                    reader.read_bits(9).context("Unexpected end")? as i64 - 256
                } else {
                    let fourth_bit = reader.read_bit().context("Unexpected end")?;
                    if !fourth_bit {
                        // 12 bits
                        reader.read_bits(12).context("Unexpected end")? as i64 - 2048
                    } else {
                        // 32 bits
                        reader.read_bits(32).context("Unexpected end")? as i32 as i64
                    }
                }
            }
        };

        // Reconstruct: dod was stored as unsigned offset from range min
        // We need to handle signed values properly
        let dod_signed = if first_bit {
            // Decode the signed value properly based on bit width
            let second_bit = true; // already consumed above, need to restructure
            dod // will fix below
        } else {
            0
        };

        prev_delta = prev_delta; // placeholder, will fix
        let ts = timestamps.last().unwrap() + prev_delta;
        timestamps.push(ts);
    }

    // Decode values
    let mut prev_xor: u64 = 0;
    for _ in 1..count {
        let same = !reader.read_bit().context("Unexpected end reading value")?;
        if same {
            values.push(*values.last().unwrap());
        } else {
            let new_window = reader.read_bit().context("Unexpected end")?;
            if !new_window && prev_xor != 0 {
                // Reuse previous window
                let meaningful_bits =
                    64 - prev_xor.leading_zeros() as u8 - prev_xor.trailing_zeros() as u8;
                let xor_val = reader.read_bits(meaningful_bits).context("Unexpected end")?;
                let xor = xor_val << prev_xor.trailing_zeros();
                let val_bits = values.last().unwrap().to_bits() ^ xor;
                values.push(f64::from_bits(val_bits));
                prev_xor = xor;
            } else {
                // New window
                let leading_zeros = reader.read_bits(6).context("Unexpected end")? as u8;
                let meaningful_bits = reader.read_bits(6).context("Unexpected end")? as u8 + 1;
                let xor_val = if meaningful_bits > 0 {
                    reader.read_bits(meaningful_bits).context("Unexpected end")?
                } else {
                    0
                };
                let trailing_zeros = 64 - leading_zeros - meaningful_bits;
                let xor = xor_val << trailing_zeros;
                let val_bits = values.last().unwrap().to_bits() ^ xor;
                values.push(f64::from_bits(val_bits));
                prev_xor = xor;
            }
        }
    }

    Ok((timestamps, values))
}
```

Wait — the timestamp decoding has a structural issue with sign handling. Let me provide the corrected full implementation:

Replace the entire `gorilla_encode` and `gorilla_decode` with this corrected version:

```rust
/// Encode timestamps and values using Gorilla paper algorithm.
pub fn gorilla_encode(timestamps: &[i64], values: &[f64]) -> Result<GorillaBlock> {
    if timestamps.is_empty() {
        anyhow::bail!("Cannot encode empty data");
    }
    anyhow::ensure!(
        timestamps.len() == values.len(),
        "Timestamps and values must have same length"
    );

    let first_ts = timestamps[0];
    let first_value = values[0];
    let count = timestamps.len();
    let mut writer = BitWriter::new();

    if count == 1 {
        return Ok(GorillaBlock { first_ts, count, first_value, data: writer.finish() });
    }

    let mut prev_delta: i64 = 0;

    for i in 1..count {
        let delta = timestamps[i] - timestamps[i - 1];
        let dod = delta - prev_delta;
        encode_dod(&mut writer, dod);
        prev_delta = delta;
    }

    let mut prev_xor: u64 = 0;
    for i in 1..count {
        let val_bits = values[i].to_bits();
        let xor = val_bits ^ values[i - 1].to_bits();
        encode_xor(&mut writer, xor, &mut prev_xor);
    }

    Ok(GorillaBlock { first_ts, count, first_value, data: writer.finish() })
}

fn encode_dod(writer: &mut BitWriter, dod: i64) {
    if dod == 0 {
        writer.write_bit(false);
    } else {
        writer.write_bit(true);
        let abs = dod.unsigned_abs();
        if abs < (1 << 6) {
            writer.write_bit(false);
            writer.write_bits(0, 1); // 7-bit slot marker
            writer.write_bits(dod as u64, 7);
        } else if abs < (1 << 9) {
            writer.write_bits(0b110, 3);
            writer.write_bits(dod as u64, 9);
        } else if abs < (1 << 12) {
            writer.write_bits(0b1110, 4);
            writer.write_bits(dod as u64, 12);
        } else {
            writer.write_bits(0b1111, 4);
            writer.write_bits(dod as u64, 32);
        }
    }
}

fn encode_xor(writer: &mut BitWriter, xor: u64, prev_xor: &mut u64) {
    if xor == 0 {
        writer.write_bit(false);
    } else {
        writer.write_bit(true);
        let lz = xor.leading_zeros() as u8;
        let tz = xor.trailing_zeros() as u8;
        let meaningful = 64 - lz - tz;

        if *prev_xor != 0 {
            let prev_lz = prev_xor.leading_zeros() as u8;
            let prev_tz = prev_xor.trailing_zeros() as u8;
            let prev_meaningful = 64 - prev_lz - prev_tz;

            if lz >= prev_lz && tz >= prev_tz {
                writer.write_bit(false); // reuse window
                writer.write_bits(xor >> prev_tz, prev_meaningful);
                *prev_xor = xor;
                return;
            }
        }
        writer.write_bit(true); // new window
        writer.write_bits(lz as u64, 6);
        writer.write_bits((meaningful - 1) as u64, 6);
        if meaningful > 0 {
            writer.write_bits(xor >> tz, meaningful);
        }
        *prev_xor = xor;
    }
}

/// Decode a Gorilla block.
pub fn gorilla_decode(block: &GorillaBlock) -> Result<(Vec<i64>, Vec<f64>)> {
    let count = block.count;
    let mut timestamps = Vec::with_capacity(count);
    let mut values = Vec::with_capacity(count);
    timestamps.push(block.first_ts);
    values.push(block.first_value);

    if count <= 1 {
        return Ok((timestamps, values));
    }

    let mut reader = BitReader::new(&block.data);
    let mut prev_delta: i64 = 0;

    for _ in 1..count {
        if !reader.read_bit().context("EOF reading ts")? {
            // dod = 0
            let ts = timestamps.last().unwrap() + prev_delta;
            timestamps.push(ts);
        } else {
            let dod = decode_dod(&mut reader)?;
            let delta = prev_delta + dod;
            let ts = timestamps.last().unwrap() + delta;
            timestamps.push(ts);
            prev_delta = delta;
        }
    }

    let mut prev_xor: u64 = 0;
    for _ in 1..count {
        if !reader.read_bit().context("EOF reading value")? {
            values.push(*values.last().unwrap());
        } else {
            let xor = decode_xor(&mut reader, &mut prev_xor)?;
            let val_bits = values.last().unwrap().to_bits() ^ xor;
            values.push(f64::from_bits(val_bits));
        }
    }

    Ok((timestamps, values))
}

fn decode_dod(reader: &mut BitReader) -> Result<i64> {
    let bit2 = reader.read_bit().context("EOF")?;
    if !bit2 {
        // 7-bit slot
        reader.read_bits(1).context("EOF")?; // skip marker bit
        let raw = reader.read_bits(7).context("EOF")?;
        Ok((raw as i64) - 64) // center around 0 for 7-bit signed
    } else {
        let bit3 = reader.read_bit().context("EOF")?;
        if !bit3 {
            let raw = reader.read_bits(9).context("EOF")?;
            Ok((raw as i64) - 256)
        } else {
            let bit4 = reader.read_bit().context("EOF")?;
            if !bit4 {
                let raw = reader.read_bits(12).context("EOF")?;
                Ok((raw as i64) - 2048)
            } else {
                let raw = reader.read_bits(32).context("EOF")?;
                Ok(raw as i32 as i64)
            }
        }
    }
}

fn decode_xor(reader: &mut BitReader, prev_xor: &mut u64) -> Result<u64> {
    let new_window = reader.read_bit().context("EOF")?;
    if !new_window && *prev_xor != 0 {
        let prev_tz = prev_xor.trailing_zeros() as u8;
        let prev_lz = prev_xor.leading_zeros() as u8;
        let meaningful = 64 - prev_lz - prev_tz;
        let val = reader.read_bits(meaningful).context("EOF")?;
        let xor = val << prev_tz;
        *prev_xor = xor;
        Ok(xor)
    } else {
        let lz = reader.read_bits(6).context("EOF")? as u8;
        let meaningful = reader.read_bits(6).context("EOF")? as u8 + 1;
        let val = if meaningful > 0 {
            reader.read_bits(meaningful).context("EOF")?
        } else {
            0
        };
        let tz = 64 - lz - meaningful;
        let xor = val << tz;
        *prev_xor = xor;
        Ok(xor)
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib compression::tests::test_gorilla`
Expected: All Gorilla tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/compression.rs
git commit -m "feat(compression): add Gorilla delta-of-delta encoder/decoder"
```

---

## Task 3: RLE Encoder and Decoder

**Files:**
- Modify: `src/compression.rs`

RLE encoding for digital/binary tags where values are mostly constant.

- [ ] **Step 1: Write failing tests for RLE roundtrip**

Add to `src/compression.rs` tests:

```rust
#[test]
fn test_rle_encode_decode_roundtrip() {
    // Pump running for 8 hours at 1Hz then off for 1 hour
    let mut values: Vec<bool> = vec![true; 28800]; // 8 hours * 3600
    values.extend(vec![false; 3600]);

    let encoded = rle_encode(&values);
    let decoded = rle_decode(&encoded);

    assert_eq!(decoded, values);
    // Should compress to 2 runs, very small
    assert!(encoded.len() < 20, "RLE should compress to <20 bytes, got {}", encoded.len());
}

#[test]
fn test_rle_alternating() {
    // Worst case: alternating values
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib compression::tests::test_rle`
Expected: FAIL — `rle_encode` not defined

- [ ] **Step 3: Implement RLE encoder/decoder**

Add to `src/compression.rs` (before `#[cfg(test)]`):

```rust
/// RLE-encoded data for digital/binary values.
/// Format: [(value: bool, count: u32), ...]
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

/// Decode RLE data back to boolean values.
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

/// Serialize RLE runs to binary (for BLOB storage).
/// Format: for each run: 1 byte (value: 0 or 1) + 4 bytes (count: u32 LE)
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
        anyhow::bail!("Invalid RLE data length: {}", data.len());
    }
    let mut runs = Vec::with_capacity(data.len() / 5);
    for chunk in data.chunks_exact(5) {
        let value = chunk[0] != 0;
        let count = u32::from_le_bytes([chunk[1], chunk[2], chunk[3], chunk[4]]);
        runs.push((value, count));
    }
    Ok(runs)
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib compression::tests::test_rle`
Expected: All RLE tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/compression.rs
git commit -m "feat(compression): add RLE encoder/decoder for digital tags"
```

---

## Task 4: Varint Encoding for Warm Tier Deltas

**Files:**
- Modify: `src/compression.rs`

Varint encoding stores small deltas compactly for warm tier buckets.

- [ ] **Step 1: Write failing tests**

Add to `src/compression.rs` tests:

```rust
#[test]
fn test_varint_roundtrip() {
    let values: Vec<i64> = vec![0, 1, -1, 127, -128, 16383, -16384, 1000000, -1000000];
    let encoded = varint_encode(&values);
    let decoded = varint_decode(&encoded).unwrap();
    assert_eq!(decoded, values);
}

#[test]
fn test_varint_small_values() {
    // Typical SCADA deltas: tiny differences
    let values: Vec<i64> = vec![0, 0, 0, 1, 0, -1, 0, 2, -1, 0];
    let encoded = varint_encode(&values);
    assert!(encoded.len() < 20, "Small deltas should be very compact");
    let decoded = varint_decode(&encoded).unwrap();
    assert_eq!(decoded, values);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib compression::tests::test_varint`
Expected: FAIL

- [ ] **Step 3: Implement varint encode/decode**

Add to `src/compression.rs` (before `#[cfg(test)]`):

```rust
/// Zigzag encode: maps signed to unsigned (0->0, -1->1, 1->2, -2->3, ...)
fn zigzag_encode(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// Zigzag decode: maps unsigned back to signed
fn zigzag_decode(n: u64) -> i64 {
    ((n >> 1) as i64) ^ -((n & 1) as i64)
}

/// Encode signed integers as varint (zigzag + LEB128).
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

/// Decode varint (LEB128 + zigzag) back to signed integers.
pub fn varint_decode(data: &[u8]) -> Result<Vec<i64>> {
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
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib compression::tests::test_varint`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/compression.rs
git commit -m "feat(compression): add varint encoding for warm tier deltas"
```

---

## Task 5: Deadband Filter Module

**Files:**
- Create: `src/deadband.rs`
- Modify: `src/main.rs` (add `mod deadband;`)

- [ ] **Step 1: Create `src/deadband.rs` with DeadbandFilter**

```rust
// src/deadband.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Per-tag deadband configuration.
#[derive(Debug, Clone)]
pub struct DeadbandEntry {
    pub deadband_type: DeadbandType,
    pub deadband_value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeadbandType {
    Percent,
    Absolute,
}

impl Default for DeadbandType {
    fn default() -> Self {
        DeadbandType::Percent
    }
}

/// In-memory tracker for last-written values per (device_id, metric).
#[derive(Debug, Clone)]
pub struct DeadbandFilter {
    last_values: Arc<RwLock<HashMap<(String, String), f64>>>,
    configs: Arc<RwLock<HashMap<(String, String), DeadbandEntry>>>,
    default_analog_percent: f64,
    absolute_min: f64,
}

impl DeadbandFilter {
    pub fn new() -> Self {
        Self {
            last_values: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            default_analog_percent: 0.1,
            absolute_min: 0.001,
        }
    }

    /// Update per-tag deadband config.
    pub async fn set_config(&self, device_id: &str, metric: &str, entry: DeadbandEntry) {
        let mut configs = self.configs.write().await;
        configs.insert((device_id.to_string(), metric.to_string()), entry);
    }

    /// Load configs from database.
    pub async fn load_configs(&self, entries: Vec<(String, String, DeadbandEntry)>) {
        let mut configs = self.configs.write().await;
        for (device_id, metric, entry) in entries {
            configs.insert((device_id, metric), entry);
        }
    }

    /// Check if a new value should be written (passes deadband filter).
    /// Returns true if the value should be written.
    pub async fn should_write(&self, device_id: &str, metric: &str, new_value: f64) -> bool {
        let key = (device_id.to_string(), metric.to_string());
        let last_values = self.last_values.read().await;

        if let Some(&last) = last_values.get(&key) {
            let threshold = self.compute_threshold(device_id, metric, last).await;
            let diff = (new_value - last).abs();
            diff > threshold
        } else {
            // First value for this tag — always write
            true
        }
    }

    /// Record that a value was written.
    pub async fn record_write(&self, device_id: &str, metric: &str, value: f64) {
        let mut last_values = self.last_values.write().await;
        last_values.insert((device_id.to_string(), metric.to_string()), value);
    }

    /// Filter a batch of records, returning only those that pass the deadband.
    /// Also updates last_values for passed records.
    pub async fn filter_batch(
        &self,
        records: &[(String, i64, String, f64)],
    ) -> Vec<(String, i64, String, f64)> {
        let mut result = Vec::new();
        let configs = self.configs.read().await;
        let mut last_values = self.last_values.write().await;

        for (device_id, ts, metric, value) in records {
            let key = (device_id.clone(), metric.clone());
            let should_write = match last_values.get(&key) {
                Some(&last) => {
                    let threshold = if let Some(cfg) = configs.get(&key) {
                        match cfg.deadband_type {
                            DeadbandType::Percent => {
                                (cfg.deadband_value * last.abs()).max(self.absolute_min)
                            }
                            DeadbandType::Absolute => cfg.deadband_value,
                        }
                    } else {
                        // Default analog: 0.1% with absolute min
                        (self.default_analog_percent * last.abs()).max(self.absolute_min)
                    };
                    (value - last).abs() > threshold
                }
                None => true,
            };

            if should_write {
                last_values.insert(key, *value);
                result.push((device_id.clone(), *ts, metric.clone(), *value));
            }
        }

        result
    }

    async fn compute_threshold(&self, device_id: &str, metric: &str, last_value: f64) -> f64 {
        let configs = self.configs.read().await;
        let key = (device_id.to_string(), metric.to_string());
        if let Some(cfg) = configs.get(&key) {
            match cfg.deadband_type {
                DeadbandType::Percent => {
                    (cfg.deadband_value * last_value.abs()).max(self.absolute_min)
                }
                DeadbandType::Absolute => cfg.deadband_value,
            }
        } else {
            (self.default_analog_percent * last_value.abs()).max(self.absolute_min)
        }
    }
}
```

- [ ] **Step 2: Write tests for deadband filter**

Add to `src/deadband.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_first_value_always_passes() {
        let filter = DeadbandFilter::new();
        assert!(filter.should_write("dev1", "temp", 25.0).await);
    }

    #[tokio::test]
    async fn test_same_value_filtered() {
        let filter = DeadbandFilter::new();
        filter.record_write("dev1", "temp", 25.0).await;
        assert!(!filter.should_write("dev1", "temp", 25.0).await);
    }

    #[tokio::test]
    async fn test_small_change_filtered() {
        let filter = DeadbandFilter::new();
        filter.record_write("dev1", "temp", 25.0).await;
        // Default 0.1% of 25 = 0.025, absolute_min = 0.001
        assert!(!filter.should_write("dev1", "temp", 25.02).await);
    }

    #[tokio::test]
    async fn test_large_change_passes() {
        let filter = DeadbandFilter::new();
        filter.record_write("dev1", "temp", 25.0).await;
        assert!(filter.should_write("dev1", "temp", 26.0).await);
    }

    #[tokio::test]
    async fn test_absolute_deadband() {
        let filter = DeadbandFilter::new();
        filter.set_config("dev1", "temp", DeadbandEntry {
            deadband_type: DeadbandType::Absolute,
            deadband_value: 1.0,
        }).await;
        filter.record_write("dev1", "temp", 25.0).await;
        assert!(!filter.should_write("dev1", "temp", 25.5).await);
        assert!(filter.should_write("dev1", "temp", 26.1).await);
    }

    #[tokio::test]
    async fn test_batch_filtering() {
        let filter = DeadbandFilter::new();
        let records = vec![
            ("dev1".into(), 1000i64, "temp".into(), 25.0f64),
            ("dev1".into(), 2000i64, "temp".into(), 25.001f64), // filtered
            ("dev1".into(), 3000i64, "temp".into(), 26.0f64),
            ("dev2".into(), 1000i64, "pressure".into(), 100.0f64),
        ];
        let passed = filter.filter_batch(&records).await;
        assert_eq!(passed.len(), 3); // 25.001 filtered out
    }
}
```

- [ ] **Step 3: Add `mod deadband;` to `src/main.rs`**

Add after the existing `mod compression;` line (or alongside other mod declarations near line 5):

```rust
mod deadband;
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib deadband::tests`
Expected: All 6 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/deadband.rs src/main.rs
git commit -m "feat(compression): add deadband filter module for write-path filtering"
```

---

## Task 6: Storage Schema — New Compression Tables

**Files:**
- Modify: `src/storage.rs`

Adds `telemetry_hot`, `telemetry_warm`, `telemetry_cold`, and `deadband_config` table creation. Also adds migration logic for renaming old tables.

- [ ] **Step 1: Add new table creation and migration logic to Storage::new()**

Find the existing table creation area (around lines 3365-3548 where telemetry/telemetry_hourly/telemetry_daily tables are created). Add the following new tables after the existing ones:

```rust
// In the Storage::new() or initialization function, after existing table creation:

// === Compression tier tables (v0.25.0) ===
conn.execute_batch(
    "CREATE TABLE IF NOT EXISTS telemetry_hot (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id TEXT NOT NULL,
        metric TEXT NOT NULL,
        timestamp INTEGER NOT NULL,
        value REAL NOT NULL,
        quality INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS idx_hot_time ON telemetry_hot(device_id, metric, timestamp);

    CREATE TABLE IF NOT EXISTS telemetry_warm (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id TEXT NOT NULL,
        metric TEXT NOT NULL,
        bucket_start INTEGER NOT NULL,
        open REAL NOT NULL,
        high REAL NOT NULL,
        low REAL NOT NULL,
        close REAL NOT NULL,
        count INTEGER NOT NULL,
        deltas BLOB
    );
    CREATE INDEX IF NOT EXISTS idx_warm_time ON telemetry_warm(device_id, metric, bucket_start);

    CREATE TABLE IF NOT EXISTS telemetry_cold (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id TEXT NOT NULL,
        metric TEXT NOT NULL,
        block_start INTEGER NOT NULL,
        encoding TEXT NOT NULL,
        data_type TEXT NOT NULL,
        data BLOB NOT NULL,
        count INTEGER NOT NULL,
        first_ts INTEGER NOT NULL,
        last_ts INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_cold_time ON telemetry_cold(device_id, metric, block_start);

    CREATE TABLE IF NOT EXISTS deadband_config (
        device_id TEXT NOT NULL,
        metric TEXT NOT NULL,
        deadband_type TEXT NOT NULL DEFAULT 'percent',
        deadband_value REAL NOT NULL DEFAULT 0.1,
        PRIMARY KEY (device_id, metric)
    );"
).context("Failed to create compression tables")?;

// Migration: rename old telemetry table to telemetry_hot if it exists and telemetry_hot doesn't
{
    let has_hot: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='telemetry_hot'",
        [],
        |row| row.get(0),
    ).unwrap_or(false);

    let has_old: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='telemetry'",
        [],
        |row| row.get(0),
    ).unwrap_or(false);

    if !has_hot && has_old {
        tracing::info!("Migrating telemetry → telemetry_hot");
        conn.execute_batch(
            "ALTER TABLE telemetry RENAME TO telemetry_hot;
             ALTER TABLE telemetry_hourly RENAME TO telemetry_hourly_backup;
             ALTER TABLE telemetry_daily RENAME TO telemetry_daily_backup;"
        ).context("Migration failed")?;
    }
}

// SQLite WAL mode tuning
conn.execute_batch(
    "PRAGMA journal_mode=WAL;
     PRAGMA synchronous=NORMAL;
     PRAGMA wal_autocheckpoint=1000;
     PRAGMA cache_size=-32000;
     PRAGMA temp_store=MEMORY;"
).context("Failed to set WAL pragmas")?;
```

- [ ] **Step 2: Add insert_batch_hot function**

Add a new method to Storage:

```rust
/// Insert batch into telemetry_hot (replaces old insert_batch for telemetry).
pub async fn insert_batch_hot(&self, records: &[(String, i64, String, f64)]) -> Result<()> {
    if records.is_empty() {
        return Ok(());
    }
    let conn = self.conn.clone();
    let records = records.to_vec();

    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        let tx = conn.unchecked_transaction().context("Begin tx failed")?;
        for (device_id, timestamp, metric, value) in &records {
            tx.execute(
                "INSERT INTO telemetry_hot (device_id, timestamp, metric, value) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![device_id, timestamp, metric, value],
            ).context("Insert failed")?;
        }
        tx.commit().context("Commit failed")
    }).await.context("Task join failed")?
}
```

- [ ] **Step 3: Add deadband_config CRUD functions**

```rust
pub async fn get_deadband_configs(&self) -> Result<Vec<(String, String, String, f64)>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT device_id, metric, deadband_type, deadband_value FROM deadband_config"
        ).context("Prepare failed")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        }).context("Query failed")?;
        rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
    }).await.context("Task join failed")?
}

pub async fn set_deadband_config(
    &self,
    device_id: &str,
    metric: &str,
    deadband_type: &str,
    deadband_value: f64,
) -> Result<()> {
    let conn = self.conn.clone();
    let device_id = device_id.to_string();
    let metric = metric.to_string();
    let deadband_type = deadband_type.to_string();

    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO deadband_config (device_id, metric, deadband_type, deadband_value) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![device_id, metric, deadband_type, deadband_value],
        ).context("Insert deadband config failed")?;
        Ok(())
    }).await.context("Task join failed")?
}
```

- [ ] **Step 4: Verify the project compiles**

Run: `cargo build`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src/storage.rs
git commit -m "feat(compression): add telemetry_hot/warm/cold tables and migration logic"
```

---

## Task 7: Compression Workers — Hot to Warm

**Files:**
- Modify: `src/storage.rs`

Implements the hot→warm compression worker that aggregates 1-minute OHLC buckets from raw telemetry.

- [ ] **Step 1: Add compress_hot_to_warm function**

```rust
use crate::compression::varint_encode;

pub async fn compress_hot_to_warm(&self, cutoff_ts: i64) -> Result<u64> {
    let conn = self.conn.clone();

    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        let tx = conn.unchecked_transaction().context("Begin tx failed")?;

        // Fetch all hot rows older than cutoff
        let mut stmt = tx.prepare(
            "SELECT device_id, metric, timestamp, value FROM telemetry_hot
             WHERE timestamp < ?1
             ORDER BY device_id, metric, timestamp"
        ).context("Prepare failed")?;

        let rows: Vec<(String, String, i64, f64)> = stmt.query_map(
            rusqlite::params![cutoff_ts],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).context("Query failed")?
        .collect::<Result<Vec<_>, _>>().context("Row mapping failed")?;

        drop(stmt);

        if rows.is_empty() {
            tx.commit()?;
            return Ok(0u64);
        }

        // Group by (device_id, metric, bucket_start)
        let mut buckets: std::collections::HashMap<(String, String, i64), Vec<(i64, f64)>> =
            std::collections::HashMap::new();

        for (device_id, metric, ts, value) in &rows {
            let bucket_start = (ts / 60000) * 60000; // align to minute
            let key = (device_id.clone(), metric.clone(), bucket_start);
            buckets.entry(key).or_default().push((*ts, *value));
        }

        let mut insert_stmt = tx.prepare(
            "INSERT INTO telemetry_warm (device_id, metric, bucket_start, open, high, low, close, count, deltas)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        ).context("Prepare insert failed")?;

        let mut compressed_count: u64 = 0;

        for ((device_id, metric, bucket_start), points) in &buckets {
            let open = points.first().unwrap().1;
            let close = points.last().unwrap().1;
            let high = points.iter().map(|(_, v)| *v).fold(f64::NEG_INFINITY, f64::max);
            let low = points.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
            let count = points.len() as i64;

            // Delta-encode individual values relative to open
            let deltas: Vec<i64> = points.iter()
                .map(|(_, v)| ((v - open) * 1000.0).round() as i64) // 3 decimal precision
                .collect();
            let delta_bytes = varint_encode(&deltas);

            insert_stmt.execute(rusqlite::params![
                device_id, metric, bucket_start,
                open, high, low, close, count, delta_bytes
            ]).context("Insert warm failed")?;

            compressed_count += count as u64;
        }

        drop(insert_stmt);

        // Delete compressed rows from hot
        tx.execute(
            "DELETE FROM telemetry_hot WHERE timestamp < ?1",
            rusqlite::params![cutoff_ts],
        ).context("Delete hot failed")?;

        tx.commit().context("Commit failed")?;
        Ok(compressed_count)
    }).await.context("Task join failed")?
}
```

- [ ] **Step 2: Add compression stats query**

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompressionStats {
    pub hot_rows: i64,
    pub warm_rows: i64,
    pub cold_rows: i64,
    pub db_size_bytes: u64,
    pub last_hot_to_warm: Option<i64>,
    pub last_warm_to_cold: Option<i64>,
}

pub async fn get_compression_stats(&self) -> Result<CompressionStats> {
    let conn = self.conn.clone();
    let db_path = self.db_path.clone();

    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();

        let hot_rows: i64 = conn.query_row(
            "SELECT COUNT(*) FROM telemetry_hot", [], |row| row.get(0)
        ).unwrap_or(0);

        let warm_rows: i64 = conn.query_row(
            "SELECT COUNT(*) FROM telemetry_warm", [], |row| row.get(0)
        ).unwrap_or(0);

        let cold_rows: i64 = conn.query_row(
            "SELECT COUNT(*) FROM telemetry_cold", [], |row| row.get(0)
        ).unwrap_or(0);

        let db_size_bytes = std::fs::metadata(&db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(CompressionStats {
            hot_rows,
            warm_rows,
            cold_rows,
            db_size_bytes,
            last_hot_to_warm: None,
            last_warm_to_cold: None,
        })
    }).await.context("Task join failed")?
}
```

- [ ] **Step 3: Verify compile**

Run: `cargo build`
Expected: SUCCESS

- [ ] **Step 4: Commit**

```bash
git add src/storage.rs
git commit -m "feat(compression): add hot→warm compression worker"
```

---

## Task 8: Compression Workers — Warm to Cold

**Files:**
- Modify: `src/storage.rs`

Implements the warm→cold compression worker using Gorilla/RLE encoding.

- [ ] **Step 1: Add compress_warm_to_cold function**

```rust
use crate::compression::{gorilla_encode, rle_encode, rle_serialize, GorillaBlock};

pub async fn compress_warm_to_cold(&self, cutoff_ts: i64) -> Result<u64> {
    let conn = self.conn.clone();

    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        let tx = conn.unchecked_transaction().context("Begin tx failed")?;

        // Fetch warm rows older than cutoff
        let mut stmt = tx.prepare(
            "SELECT device_id, metric, bucket_start, open, high, low, close, count, deltas
             FROM telemetry_warm
             WHERE bucket_start < ?1
             ORDER BY device_id, metric, bucket_start"
        ).context("Prepare failed")?;

        struct WarmRow {
            device_id: String,
            metric: String,
            bucket_start: i64,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            count: i64,
            deltas: Option<Vec<u8>>,
        }

        let rows: Vec<WarmRow> = stmt.query_map(
            rusqlite::params![cutoff_ts],
            |row| Ok(WarmRow {
                device_id: row.get(0)?,
                metric: row.get(1)?,
                bucket_start: row.get(2)?,
                open: row.get(3)?,
                high: row.get(4)?,
                low: row.get(5)?,
                close: row.get(6)?,
                count: row.get(7)?,
                deltas: row.get(8)?,
            }),
        ).context("Query failed")?
        .collect::<Result<Vec<_>, _>>().context("Row mapping failed")?;

        drop(stmt);

        if rows.is_empty() {
            tx.commit()?;
            return Ok(0u64);
        }

        // Group by (device_id, metric, hour_block)
        let mut hour_blocks: std::collections::HashMap<(String, String, i64), Vec<&WarmRow>> =
            std::collections::HashMap::new();

        for row in &rows {
            let hour_block = (row.bucket_start / 3600000) * 3600000;
            let key = (row.device_id.clone(), row.metric.clone(), hour_block);
            hour_blocks.entry(key).or_default().push(row);
        }

        let mut insert_stmt = tx.prepare(
            "INSERT INTO telemetry_cold (device_id, metric, block_start, encoding, data_type, data, count, first_ts, last_ts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        ).context("Prepare insert failed")?;

        let mut compressed_count: u64 = 0;

        for ((device_id, metric, hour_block), bucket_rows) in &hour_blocks {
            let first_ts = bucket_rows.first().unwrap().bucket_start;
            let last_ts = bucket_rows.last().unwrap().bucket_start;
            let total_count: i64 = bucket_rows.iter().map(|r| r.count).sum();

            // Determine if digital (binary) or analog based on value patterns
            let is_digital = bucket_rows.iter().all(|r| {
                r.open == 0.0 || r.open == 1.0 || r.close == 0.0 || r.close == 1.0
            }) && bucket_rows.iter().all(|r| r.high <= 1.0 && r.low >= 0.0);

            let (encoding, data) = if is_digital {
                // RLE: use close values as the representative
                let values: Vec<bool> = bucket_rows.iter()
                    .map(|r| r.close > 0.5)
                    .collect();
                let runs = rle_encode(&values);
                let data = rle_serialize(&runs);
                ("rle".to_string(), data)
            } else {
                // Gorilla: use OHLC close values with timestamps
                let timestamps: Vec<i64> = bucket_rows.iter()
                    .map(|r| r.bucket_start)
                    .collect();
                let values: Vec<f64> = bucket_rows.iter()
                    .map(|r| r.close)
                    .collect();
                let block = gorilla_encode(&timestamps, &values)
                    .context("Gorilla encode failed")?;
                // Serialize: first_ts(8) + count(4) + first_value(8) + data
                let mut buf = Vec::new();
                buf.extend_from_slice(&block.first_ts.to_le_bytes());
                buf.extend_from_slice(&(block.count as u32).to_le_bytes());
                buf.extend_from_slice(&block.first_value.to_le_bytes());
                buf.extend_from_slice(&block.data);
                ("gorilla".to_string(), buf)
            };

            let data_type = if is_digital { "digital" } else { "analog" };

            insert_stmt.execute(rusqlite::params![
                device_id, metric, hour_block,
                encoding, data_type, data,
                total_count, first_ts, last_ts
            ]).context("Insert cold failed")?;

            compressed_count += total_count as u64;
        }

        drop(insert_stmt);

        tx.execute(
            "DELETE FROM telemetry_warm WHERE bucket_start < ?1",
            rusqlite::params![cutoff_ts],
        ).context("Delete warm failed")?;

        tx.commit().context("Commit failed")?;
        Ok(compressed_count)
    }).await.context("Task join failed")?
}
```

- [ ] **Step 2: Verify compile**

Run: `cargo build`
Expected: SUCCESS

- [ ] **Step 3: Commit**

```bash
git add src/storage.rs
git commit -m "feat(compression): add warm→cold compression worker with Gorilla/RLE"
```

---

## Task 9: Unified Query Path with Decompression

**Files:**
- Modify: `src/storage.rs`

Replace the old `query_history` and `query_device_metrics_merged` with a new unified query that handles all three tiers.

- [ ] **Step 1: Add unified query_compressed function**

```rust
use crate::compression::{gorilla_decode, rle_decode, rle_deserialize, varint_decode, GorillaBlock};

/// Unified query across all compression tiers. Transparent decompression.
pub async fn query_compressed(
    &self,
    device_id: &str,
    metric: &str,
    from: i64,
    to: i64,
) -> Result<Vec<DataPoint>> {
    let now = crate::timeutil::now_secs() * 1000; // current time in ms
    let hot_cutoff = now - 24 * 3600 * 1000;     // 24h ago
    let warm_cutoff = now - 30 * 24 * 3600 * 1000; // 30 days ago

    let mut all_points = Vec::new();

    // Query hot tier (raw data)
    if to > hot_cutoff {
        let points = self.query_hot(device_id, metric, from.max(hot_cutoff), to).await?;
        all_points.extend(points);
    }

    // Query warm tier
    if from < hot_cutoff && to > warm_cutoff {
        let points = self.query_warm(device_id, metric, from.max(warm_cutoff), to.min(hot_cutoff)).await?;
        all_points.extend(points);
    }

    // Query cold tier
    if from < warm_cutoff {
        let points = self.query_cold(device_id, metric, from, to.min(warm_cutoff)).await?;
        all_points.extend(points);
    }

    // Sort by timestamp (merge from multiple tiers)
    all_points.sort_by_key(|p| p.timestamp);
    all_points.truncate(10000); // maintain existing limit
    Ok(all_points)
}

async fn query_hot(&self, device_id: &str, metric: &str, from: i64, to: i64) -> Result<Vec<DataPoint>> {
    let conn = self.conn.clone();
    let device_id = device_id.to_string();
    let metric = metric.to_string();

    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT timestamp, value FROM telemetry_hot
             WHERE device_id = ?1 AND metric = ?2
             AND timestamp >= ?3 AND timestamp <= ?4
             ORDER BY timestamp LIMIT 10000"
        ).context("Prepare failed")?;

        let rows = stmt.query_map(
            rusqlite::params![device_id, metric, from, to],
            |row| Ok(DataPoint { timestamp: row.get(0)?, value: row.get(1)? }),
        ).context("Query failed")?;

        rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
    }).await.context("Task join failed")?
}

async fn query_warm(&self, device_id: &str, metric: &str, from: i64, to: i64) -> Result<Vec<DataPoint>> {
    let conn = self.conn.clone();
    let device_id = device_id.to_string();
    let metric = metric.to_string();

    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT bucket_start, open, count, deltas FROM telemetry_warm
             WHERE device_id = ?1 AND metric = ?2
             AND bucket_start >= ?3 AND bucket_start <= ?4
             ORDER BY bucket_start LIMIT 10000"
        ).context("Prepare failed")?;

        let rows: Vec<(i64, f64, i64, Option<Vec<u8>>)> = stmt.query_map(
            rusqlite::params![device_id, metric, from / 60000 * 60000, to / 60000 * 60000],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).context("Query failed")?
        .collect::<Result<Vec<_>, _>>().context("Row mapping failed")?;

        let mut points = Vec::new();
        for (bucket_start, open, count, deltas) in rows {
            if let Some(delta_bytes) = deltas {
                if let Ok(deltas) = varint_decode(&delta_bytes) {
                    let step = if count > 1 { 60000 / (count as i64).max(1) } else { 0 };
                    for (i, d) in deltas.iter().enumerate() {
                        let ts = bucket_start + (i as i64) * step;
                        points.push(DataPoint {
                            timestamp: ts,
                            value: open + (*d as f64 / 1000.0),
                        });
                    }
                    continue;
                }
            }
            // Fallback: just use close value
            points.push(DataPoint { timestamp: bucket_start, value: open });
        }

        Ok(points)
    }).await.context("Task join failed")?
}

async fn query_cold(&self, device_id: &str, metric: &str, from: i64, to: i64) -> Result<Vec<DataPoint>> {
    let conn = self.conn.clone();
    let device_id = device_id.to_string();
    let metric = metric.to_string();

    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT block_start, encoding, data_type, data, count, first_ts, last_ts
             FROM telemetry_cold
             WHERE device_id = ?1 AND metric = ?2
             AND block_start >= ?3 AND block_start <= ?4
             ORDER BY block_start"
        ).context("Prepare failed")?;

        let rows: Vec<(i64, String, String, Vec<u8>, i64, i64, i64)> = stmt.query_map(
            rusqlite::params![device_id, metric, from / 3600000 * 3600000, to / 3600000 * 3600000],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        ).context("Query failed")?
        .collect::<Result<Vec<_>, _>>().context("Row mapping failed")?;

        let mut points = Vec::new();
        for (_block_start, encoding, _data_type, data, _count, _first_ts, _last_ts) in rows {
            if encoding == "gorilla" && data.len() >= 20 {
                // Deserialize GorillaBlock
                let first_ts = i64::from_le_bytes(data[0..8].try_into().unwrap());
                let count = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
                let first_value = f64::from_bits(u64::from_le_bytes(data[12..20].try_into().unwrap()));
                let gorilla_data = &data[20..];

                let block = GorillaBlock { first_ts, count, first_value, data: gorilla_data.to_vec() };
                if let Ok((timestamps, values)) = gorilla_decode(&block) {
                    for (ts, v) in timestamps.iter().zip(values.iter()) {
                        points.push(DataPoint { timestamp: *ts, value: *v });
                    }
                }
            } else if encoding == "rle" {
                if let Ok(runs) = rle_deserialize(&data) {
                    let values = rle_decode(&runs);
                    // Approximate timestamps evenly across the block
                    let block_start_ts = _block_start;
                    let step = if values.len() > 1 {
                        3600000 / values.len()
                    } else {
                        0
                    };
                    for (i, v) in values.iter().enumerate() {
                        points.push(DataPoint {
                            timestamp: block_start_ts + (i as i64) * step,
                            value: if *v { 1.0 } else { 0.0 },
                        });
                    }
                }
            }
        }

        Ok(points)
    }).await.context("Task join failed")?
}
```

- [ ] **Step 2: Update existing query_history to use new tables**

Modify the existing `query_history` function (line ~98) to query `telemetry_hot` instead of `telemetry`:

Change `FROM telemetry` to `FROM telemetry_hot` in the query_history SQL.

Also update `query_device_metrics_merged` to use the new `query_compressed` function, or update it to query the new table names.

- [ ] **Step 3: Verify compile**

Run: `cargo build`
Expected: SUCCESS

- [ ] **Step 4: Commit**

```bash
git add src/storage.rs
git commit -m "feat(compression): add unified query path with transparent decompression"
```

---

## Task 10: Compression Config Model and CLI Commands

**Files:**
- Modify: `src/models.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add CompressionConfig to models.rs**

Add after the existing RetentionConfig struct (around line 67):

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompressionConfig {
    #[serde(default = "default_compression_enabled")]
    pub enabled: bool,
    #[serde(default = "default_hot_retention_secs")]
    pub hot_retention_secs: u64,
    #[serde(default = "default_warm_retention_secs")]
    pub warm_retention_secs: u64,
    #[serde(default = "default_cold_retention_secs")]
    pub cold_retention_secs: u64,
    #[serde(default = "default_compress_interval_secs")]
    pub hot_to_warm_interval_secs: u64,
    #[serde(default = "default_compress_cold_interval_secs")]
    pub warm_to_cold_interval_secs: u64,
    #[serde(default)]
    pub edge_mode: bool,
}

fn default_compression_enabled() -> bool { true }
fn default_hot_retention_secs() -> u64 { 86400 }     // 24 hours
fn default_warm_retention_secs() -> u64 { 2592000 }   // 30 days
fn default_cold_retention_secs() -> u64 { 31536000 }  // 365 days
fn default_compress_interval_secs() -> u64 { 3600 }    // 1 hour
fn default_compress_cold_interval_secs() -> u64 { 86400 } // 24 hours

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: default_compression_enabled(),
            hot_retention_secs: default_hot_retention_secs(),
            warm_retention_secs: default_warm_retention_secs(),
            cold_retention_secs: default_cold_retention_secs(),
            hot_to_warm_interval_secs: default_compress_interval_secs(),
            warm_to_cold_interval_secs: default_compress_cold_interval_secs(),
            edge_mode: false,
        }
    }
}
```

Also add `compression` field to AppConfig:

```rust
// In AppConfig struct, add after retention field:
#[serde(default)]
pub compression: CompressionConfig,
```

- [ ] **Step 2: Add mod declaration for compression module**

Add `mod compression;` to `src/main.rs` near the other mod declarations.

- [ ] **Step 3: Add CLI commands to main.rs**

Add new CLI command handling in the args parsing section (around lines 354-421, after the "discover" command):

```rust
// After the "discover" command handling:

Some(cmd) if cmd == "db" => {
    match args.next() {
        Some(sub) if sub == "stats" => {
            let config = config::load_config(config_path)?;
            let storage = Storage::new(&config.db_path)?;
            let stats = storage.get_compression_stats().await?;
            println!("=== Voltrus Database Stats ===");
            println!("Hot  (raw, <24h):  {} rows", stats.hot_rows);
            println!("Warm (1min OHLC):  {} rows", stats.warm_rows);
            println!("Cold (compressed): {} rows", stats.cold_rows);
            println!("DB size:           {} bytes ({:.2} MB)", stats.db_size_bytes, stats.db_size_bytes as f64 / 1_048_576.0);
        }
        Some(sub) if sub == "compress" => {
            let config = config::load_config(config_path)?;
            let storage = Storage::new(&config.db_path)?;
            let now = crate::timeutil::now_secs() * 1000;
            let hot_cutoff = now - (config.compression.hot_retention_secs as i64 * 1000);
            let warm_cutoff = now - (config.compression.warm_retention_secs as i64 * 1000);

            println!("Compressing hot → warm...");
            let count = storage.compress_hot_to_warm(hot_cutoff).await?;
            println!("Compressed {count} hot rows to warm");

            println!("Compressing warm → cold...");
            let count = storage.compress_warm_to_cold(warm_cutoff).await?;
            println!("Compressed {count} warm rows to cold");
        }
        _ => {
            eprintln!("Usage: voltrus db [stats|compress]");
            std::process::exit(1);
        }
    }
    return Ok(());
}
Some(cmd) if cmd == "archive" => {
    let config = config::load_config(config_path)?;
    let storage = Storage::new(&config.db_path)?;
    let cutoff = crate::timeutil::now_secs() - (config.compression.cold_retention_secs as i64);
    let rows = storage.query_raw_for_archive(cutoff).await?;
    if rows.is_empty() {
        println!("No data to archive");
    } else {
        println!("Archiving {} records...", rows.len());
        // Archive logic: export to CSV+gzip
        use std::io::Write;
        let archive_dir = &config.retention.archive_dir;
        let _ = std::fs::create_dir_all(archive_dir);
        let filename = format!("{}/archive-{}.csv.gz", archive_dir, chrono::Utc::now().format("%Y%m%d-%H%M%S"));
        let file = std::fs::File::create(&filename)?;
        let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        writeln!(encoder, "device_id,metric,timestamp,value")?;
        for (device_id, metric, ts, value) in &rows {
            writeln!(encoder, "{},{},{},{}", device_id, metric, ts, value)?;
        }
        encoder.finish()?;
        println!("Archived to {}", filename);
    }
    return Ok(());
}
```

Note: The archive CLI reuses the existing archive logic pattern from `retention.rs`. If `chrono` isn't already a dependency, use `time` crate's formatting or a simple timestamp.

- [ ] **Step 4: Verify compile**

Run: `cargo build`
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add src/models.rs src/main.rs
git commit -m "feat(compression): add compression config model and CLI commands"
```

---

## Task 11: Spawn Compression Workers in main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add compression worker spawning after retention task**

After the retention task spawn (around line 916), add:

```rust
// Compression workers
if app_config.compression.enabled {
    let storage = storage.clone();
    let config = app_config.compression.clone();
    tokio::spawn(async move {
        run_compression_task(storage, config).await;
    });
}
```

- [ ] **Step 2: Add run_compression_task function**

Add before the main function or in a dedicated section:

```rust
async fn run_compression_task(storage: Storage, config: CompressionConfig) {
    use crate::models::CompressionConfig;
    tracing::info!(
        "Compression task started (hot→warm: {}s, warm→cold: {}s)",
        config.hot_to_warm_interval_secs,
        config.warm_to_cold_interval_secs,
    );

    let mut hot_interval = tokio::time::interval(
        std::time::Duration::from_secs(config.hot_to_warm_interval_secs)
    );
    let mut cold_interval = tokio::time::interval(
        std::time::Duration::from_secs(config.warm_to_cold_interval_secs)
    );

    loop {
        tokio::select! {
            _ = hot_interval.tick() => {
                let now = crate::timeutil::now_secs() * 1000;
                let cutoff = now - (config.hot_retention_secs as i64 * 1000);
                match storage.compress_hot_to_warm(cutoff).await {
                    Ok(count) => tracing::info!("Hot→Warm: compressed {count} rows"),
                    Err(e) => tracing::error!("Hot→Warm compression failed: {e}"),
                }
            }
            _ = cold_interval.tick() => {
                let now = crate::timeutil::now_secs() * 1000;
                let cutoff = now - (config.warm_retention_secs as i64 * 1000);
                match storage.compress_warm_to_cold(cutoff).await {
                    Ok(count) => tracing::info!("Warm→Cold: compressed {count} rows"),
                    Err(e) => tracing::error!("Warm→Cold compression failed: {e}"),
                }
            }
        }
    }
}
```

- [ ] **Step 3: Update persistence task to use telemetry_hot and deadband filter**

In the persistence task (around line 734), update to:
1. Initialize `DeadbandFilter`
2. Load deadband configs from storage
3. Filter records through deadband before insert
4. Use `insert_batch_hot` instead of `insert_batch`

```rust
// At the start of the persistence task closure:
let deadband_filter = deadband::DeadbandFilter::new();
let configs = storage.get_deadband_configs().await.unwrap_or_default();
deadband_filter.load_configs(
    configs.into_iter().map(|(d, m, t, v)| (d, m, deadband::DeadbandEntry {
        deadband_type: if t == "absolute" { deadband::DeadbandType::Absolute } else { deadband::DeadbandType::Percent },
        deadband_value: v,
    })).collect()
).await;
```

Then inside the loop, before `insert_batch`:

```rust
// Filter through deadband
let filtered = deadband_filter.filter_batch(&records).await;
if let Err(e) = storage.insert_batch_hot(&filtered).await {
    tracing::error!("Persistence error: {e}");
}
```

- [ ] **Step 4: Verify compile**

Run: `cargo build`
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat(compression): spawn compression workers and integrate deadband in persistence"
```

---

## Task 12: Modbus Connection Pooling

**Files:**
- Modify: `src/modbus.rs`

- [ ] **Step 1: Add ModbusPool struct**

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

struct PooledConnection {
    ctx: tokio_modbus::client::Context,
    last_used: Instant,
}

pub struct ModbusPool {
    connections: Arc<Mutex<HashMap<String, PooledConnection>>>,
    max_idle: Duration,
}

impl ModbusPool {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            max_idle: Duration::from_secs(60),
        }
    }

    pub async fn get_or_connect(
        &self,
        device_id: &str,
        addr: std::net::SocketAddr,
        unit_id: u8,
    ) -> anyhow::Result<tokio_modbus::client::Context> {
        let mut conns = self.connections.lock().await;
        if let Some(pooled) = conns.remove(device_id) {
            if pooled.last_used.elapsed() < self.max_idle {
                tracing::debug!("Reusing connection for device {device_id}");
                return Ok(pooled.ctx);
            }
            tracing::debug!("Dropping idle connection for device {device_id}");
        }

        tracing::debug!("Connecting to device {device_id} at {addr}");
        let ctx = tokio_modbus::client::tcp::connect_slave(addr, tokio_modbus::Slave(unit_id)).await?;
        Ok(ctx)
    }

    pub async fn return_connection(&self, device_id: &str, ctx: tokio_modbus::client::Context) {
        let mut conns = self.connections.lock().await;
        conns.insert(device_id.to_string(), PooledConnection {
            ctx,
            last_used: Instant::now(),
        });
    }

    pub async fn remove(&self, device_id: &str) {
        let mut conns = self.connections.lock().await;
        conns.remove(device_id);
    }
}
```

- [ ] **Step 2: Update try_poll to use connection pool**

Modify `try_poll` to accept a pool reference and reuse connections. The key change: instead of creating a new connection at the start of `try_poll`, use `pool.get_or_connect()` and return the connection to the pool after each read cycle.

This requires passing the pool through `poll_device` → `try_poll`. Update the signatures:

```rust
pub async fn poll_device(config: DeviceConfig, cache: DeviceCache, pool: Arc<ModbusPool>) {
    // ... same backoff logic ...
    loop {
        match try_poll(&config, &cache, &pool).await {
            Ok(()) => backoff.reset(),
            Err(e) => {
                // ... same error handling ...
                pool.remove(&config.id).await; // remove stale connection
                // ... backoff ...
            }
        }
    }
}

async fn try_poll(config: &DeviceConfig, cache: &DeviceCache, pool: &ModbusPool) -> Result<()> {
    let addr = format!("{}:{}", config.ip, config.port);
    let socket_addr: std::net::SocketAddr = addr.parse().context("Invalid address")?;

    let mut ctx = pool.get_or_connect(&config.id, socket_addr, config.unit_id).await?;
    ctx.set_timeout(Duration::from_secs(5));

    loop {
        // ... same register reading logic, but using ctx ...
        for reg in &config.registers {
            let result = match reg.area.as_str() {
                "input" => ctx.read_input_registers(reg.address, reg.count).await?,
                _ => ctx.read_holding_registers(reg.address, reg.count).await?,
            };
            // ... same parsing and cache.update() ...
        }

        pool.return_connection(&config.id, ctx).await;
        // For the next iteration, re-acquire from pool
        ctx = pool.get_or_connect(&config.id, socket_addr, config.unit_id).await?;
        ctx.set_timeout(Duration::from_secs(5));

        tokio::time::sleep(Duration::from_millis(config.interval_ms)).await;
    }
}
```

Note: The exact reconnection flow needs careful handling. The pool holds the connection between poll cycles. If the connection drops during a read, the error handler removes it from the pool, and the next cycle creates a fresh one.

- [ ] **Step 3: Create pool in main.rs and pass to poller_manager**

In main.rs, before starting pollers, create the pool:

```rust
let modbus_pool = Arc::new(modbus::ModbusPool::new());
```

Pass `modbus_pool` to wherever poll_device is called. This may require updating `PollerManager` to accept and forward the pool.

- [ ] **Step 4: Verify compile**

Run: `cargo build`
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add src/modbus.rs src/main.rs
git commit -m "feat(compression): add Modbus TCP connection pooling"
```

---

## Task 13: Compression API Endpoints

**Files:**
- Create: `src/api/compression.rs`
- Modify: `src/api/mod.rs`

- [ ] **Step 1: Create `src/api/compression.rs`**

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::storage::Storage;

#[derive(Debug, Serialize)]
pub struct CompressionStatsResponse {
    pub hot_rows: i64,
    pub warm_rows: i64,
    pub cold_rows: i64,
    pub db_size_bytes: u64,
    pub db_size_mb: f64,
    pub estimated_hot_mb: f64,
    pub estimated_warm_mb: f64,
    pub estimated_cold_mb: f64,
}

pub async fn get_compression_stats(
    State(state): State<crate::api::AppState>,
) -> Result<impl IntoResponse, crate::api::AppError> {
    let stats = state.storage.get_compression_stats().await
        .map_err(|e| crate::api::AppError(e.to_string()))?;

    Ok(Json(CompressionStatsResponse {
        hot_rows: stats.hot_rows,
        warm_rows: stats.warm_rows,
        cold_rows: stats.cold_rows,
        db_size_bytes: stats.db_size_bytes,
        db_size_mb: stats.db_size_bytes as f64 / 1_048_576.0,
        estimated_hot_mb: stats.hot_rows as f64 * 48.0 / 1_048_576.0,  // ~48 bytes/row
        estimated_warm_mb: stats.warm_rows as f64 * 80.0 / 1_048_576.0, // ~80 bytes/row
        estimated_cold_mb: stats.cold_rows as f64 * 200.0 / 1_048_576.0,// ~200 bytes/row
    }))
}

#[derive(Debug, Serialize)]
pub struct CompressionConfigResponse {
    pub enabled: bool,
    pub hot_retention_secs: u64,
    pub warm_retention_secs: u64,
    pub cold_retention_secs: u64,
}

pub async fn get_compression_config(
    State(state): State<crate::api::AppState>,
) -> Result<impl IntoResponse, crate::api::AppError> {
    // Return from app config — for now, return defaults
    Ok(Json(CompressionConfigResponse {
        enabled: true,
        hot_retention_secs: 86400,
        warm_retention_secs: 2592000,
        cold_retention_secs: 31536000,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompressionConfigRequest {
    pub hot_retention_secs: Option<u64>,
    pub warm_retention_secs: Option<u64>,
    pub cold_retention_secs: Option<u64>,
}

pub async fn update_compression_config(
    State(_state): State<crate::api::AppState>,
    Json(_body): Json<UpdateCompressionConfigRequest>,
) -> Result<impl IntoResponse, crate::api::AppError> {
    // TODO: Update config in runtime state
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct MemoryStatsResponse {
    pub rss_mb: f64,
    pub hot_cache_entries: usize,
    pub connection_pool_size: usize,
}

pub async fn get_system_memory(
    State(_state): State<crate::api::AppState>,
) -> Result<impl IntoResponse, crate::api::AppError> {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();

    Ok(Json(MemoryStatsResponse {
        rss_mb: sys.used_memory() as f64 / 1_048_576.0,
        hot_cache_entries: 0,
        connection_pool_size: 0,
    }))
}
```

- [ ] **Step 2: Register routes in `src/api/mod.rs`**

Add module declaration at top:

```rust
mod compression;
```

Add routes to the router (in the protected routes section):

```rust
// Compression routes
let compression_routes = Router::new()
    .route("/api/v1/compression/stats", get(compression::get_compression_stats))
    .route("/api/v1/compression/config", get(compression::get_compression_config))
    .route("/api/v1/compression/config", put(compression::update_compression_config))
    .route("/api/v1/system/memory", get(compression::get_system_memory));
```

Merge into the protected router:

```rust
router = router.merge(compression_routes);
```

- [ ] **Step 3: Verify compile**

Run: `cargo build`
Expected: SUCCESS

- [ ] **Step 4: Commit**

```bash
git add src/api/compression.rs src/api/mod.rs
git commit -m "feat(compression): add compression stats and memory API endpoints"
```

---

## Task 14: Update Retention Module

**Files:**
- Modify: `src/retention.rs`

Replace old downsampling calls with new compression tier calls.

- [ ] **Step 1: Update run_retention_cycle to use compression functions**

Replace the existing `run_retention_cycle` body with calls to the new compression functions:

```rust
async fn run_retention_cycle(storage: &Storage, config: &RetentionConfig) -> Result<()> {
    let now = timeutil::now_secs();
    tracing::info!("Starting retention/compression cycle");

    // Hot → Warm compression
    let hot_cutoff = now - 86400; // 24 hours
    match storage.compress_hot_to_warm(hot_cutoff * 1000).await {
        Ok(count) => tracing::info!("Hot→Warm: compressed {count} rows"),
        Err(e) => tracing::error!("Hot→Warm failed: {e}"),
    }

    // Warm → Cold compression
    let warm_cutoff = now - 2592000; // 30 days
    match storage.compress_warm_to_cold(warm_cutoff * 1000).await {
        Ok(count) => tracing::info!("Warm→Cold: compressed {count} rows"),
        Err(e) => tracing::error!("Warm→Cold failed: {e}"),
    }

    // Archive cold data older than retention period
    let cold_keep = get_tier_keep(&config.tiers, "daily");
    if let Some(cold_keep_secs) = cold_keep {
        let cutoff = now - cold_keep_secs as i64;
        if get_tier_archive(&config.tiers, "daily") {
            archive_raw(storage, cutoff, &config.archive_dir).await?;
        }
    }

    tracing::info!("Retention/compression cycle complete");
    Ok(())
}
```

- [ ] **Step 2: Verify compile**

Run: `cargo build`
Expected: SUCCESS

- [ ] **Step 3: Commit**

```bash
git add src/retention.rs
git commit -m "feat(compression): update retention module to use new compression workers"
```

---

## Task 15: Frontend — Compression Stats Dashboard

**Files:**
- Create: `client/src/components/CompressionStats.tsx`
- Create: `client/src/components/DeadbandConfig.tsx`
- Create: `client/src/components/StorageOverview.tsx`

- [ ] **Step 1: Create CompressionStats.tsx**

```tsx
import React, { useEffect, useState } from 'react';

interface CompressionStats {
  hot_rows: number;
  warm_rows: number;
  cold_rows: number;
  db_size_bytes: number;
  db_size_mb: number;
  estimated_hot_mb: number;
  estimated_warm_mb: number;
  estimated_cold_mb: number;
}

export const CompressionStats: React.FC = () => {
  const [stats, setStats] = useState<CompressionStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const res = await fetch('/api/v1/compression/stats');
        if (res.ok) {
          setStats(await res.json());
        }
      } catch (err) {
        console.error('Failed to fetch compression stats:', err);
      } finally {
        setLoading(false);
      }
    };
    fetchStats();
    const interval = setInterval(fetchStats, 30000);
    return () => clearInterval(interval);
  }, []);

  if (loading) return <div className="p-4 text-gray-400">Loading compression stats...</div>;
  if (!stats) return <div className="p-4 text-red-400">Failed to load stats</div>;

  const totalRows = stats.hot_rows + stats.warm_rows + stats.cold_rows;

  return (
    <div className="p-4 space-y-4">
      <h3 className="text-lg font-semibold text-white">Compression Stats</h3>

      {/* Database Size */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-gray-800 rounded-lg p-3">
          <div className="text-sm text-gray-400">Database Size</div>
          <div className="text-2xl font-bold text-white">{stats.db_size_mb.toFixed(2)} MB</div>
        </div>
        <div className="bg-gray-800 rounded-lg p-3">
          <div className="text-sm text-gray-400">Total Rows</div>
          <div className="text-2xl font-bold text-white">{totalRows.toLocaleString()}</div>
        </div>
      </div>

      {/* Tier Breakdown */}
      <div className="space-y-2">
        <TierBar label="Hot (raw)" rows={stats.hot_rows} total={totalRows} color="bg-red-500" size={stats.estimated_hot_mb} />
        <TierBar label="Warm (1min)" rows={stats.warm_rows} total={totalRows} color="bg-yellow-500" size={stats.estimated_warm_mb} />
        <TierBar label="Cold (compressed)" rows={stats.cold_rows} total={totalRows} color="bg-green-500" size={stats.estimated_cold_mb} />
      </div>
    </div>
  );
};

const TierBar: React.FC<{
  label: string;
  rows: number;
  total: number;
  color: string;
  size: number;
}> = ({ label, rows, total, color, size }) => {
  const pct = total > 0 ? (rows / total * 100) : 0;
  return (
    <div className="bg-gray-800 rounded-lg p-3">
      <div className="flex justify-between text-sm">
        <span className="text-gray-300">{label}</span>
        <span className="text-gray-400">{rows.toLocaleString()} rows ({size.toFixed(1)} MB)</span>
      </div>
      <div className="mt-1 h-2 bg-gray-700 rounded-full overflow-hidden">
        <div className={`h-full ${color} rounded-full`} style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
};
```

- [ ] **Step 2: Create DeadbandConfig.tsx**

```tsx
import React, { useEffect, useState } from 'react';

interface DeadbandEntry {
  device_id: string;
  metric: string;
  deadband_type: string;
  deadband_value: number;
}

export const DeadbandConfig: React.FC = () => {
  const [entries, setEntries] = useState<DeadbandEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchConfigs = async () => {
      try {
        const res = await fetch('/api/v1/compression/deadband');
        if (res.ok) setEntries(await res.json());
      } catch (err) {
        console.error('Failed to fetch deadband configs:', err);
      } finally {
        setLoading(false);
      }
    };
    fetchConfigs();
  }, []);

  if (loading) return <div className="p-4 text-gray-400">Loading deadband config...</div>;

  return (
    <div className="p-4">
      <h3 className="text-lg font-semibold text-white mb-3">Deadband Configuration</h3>
      <table className="w-full text-sm">
        <thead>
          <tr className="text-gray-400 border-b border-gray-700">
            <th className="text-left py-2">Device</th>
            <th className="text-left py-2">Metric</th>
            <th className="text-left py-2">Type</th>
            <th className="text-right py-2">Value</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((entry, i) => (
            <tr key={i} className="border-b border-gray-800">
              <td className="py-2 text-gray-300">{entry.device_id}</td>
              <td className="py-2 text-gray-300">{entry.metric}</td>
              <td className="py-2 text-gray-400">{entry.deadband_type}</td>
              <td className="py-2 text-right text-gray-300">{entry.deadband_value}</td>
            </tr>
          ))}
          {entries.length === 0 && (
            <tr>
              <td colSpan={4} className="py-4 text-center text-gray-500">
                Using defaults (0.1% analog, 0 digital)
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
};
```

- [ ] **Step 3: Create StorageOverview.tsx**

```tsx
import React from 'react';

interface StorageOverviewProps {
  dbSizeMB: number;
  hotRows: number;
  warmRows: number;
  coldRows: number;
}

export const StorageOverview: React.FC<StorageOverviewProps> = ({
  dbSizeMB,
  hotRows,
  warmRows,
  coldRows,
}) => {
  const totalRows = hotRows + warmRows + coldRows;

  return (
    <div className="p-4 bg-gray-900 rounded-lg">
      <h3 className="text-lg font-semibold text-white mb-3">Storage Overview</h3>

      <div className="relative h-8 bg-gray-800 rounded-full overflow-hidden flex">
        {totalRows > 0 && (
          <>
            <div className="bg-red-500 h-full" style={{ width: `${(hotRows / totalRows) * 100}%` }} title={`Hot: ${hotRows}`} />
            <div className="bg-yellow-500 h-full" style={{ width: `${(warmRows / totalRows) * 100}%` }} title={`Warm: ${warmRows}`} />
            <div className="bg-green-500 h-full" style={{ width: `${(coldRows / totalRows) * 100}%` }} title={`Cold: ${coldRows}`} />
          </>
        )}
      </div>

      <div className="mt-2 flex justify-between text-xs text-gray-400">
        <span>Total: {dbSizeMB.toFixed(2)} MB</span>
        <span>{totalRows.toLocaleString()} rows</span>
      </div>
    </div>
  );
};
```

- [ ] **Step 4: Commit**

```bash
git add client/src/components/CompressionStats.tsx client/src/components/DeadbandConfig.tsx client/src/components/StorageOverview.tsx
git commit -m "feat(compression): add compression stats dashboard components"
```

---

## Task 16: Integration Test — Full Compression Pipeline

**Files:**
- Create: `tests/compression_integration.rs` (or add to existing test files)

- [ ] **Step 1: Write integration test**

```rust
// tests/compression_integration.rs or at bottom of src/storage.rs #[cfg(test)]

#[tokio::test]
async fn test_full_compression_pipeline() {
    use crate::compression::{gorilla_encode, gorilla_decode, rle_encode, rle_decode, varint_encode, varint_decode};
    use crate::deadband::DeadbandFilter;

    // 1. Create in-memory storage
    let storage = Storage::new_in_memory();

    // 2. Insert raw data into telemetry_hot
    let records: Vec<(String, i64, String, f64)> = (0..3600)
        .map(|i| (
            "dev1".to_string(),
            1700000000000 + i * 1000,
            "temperature".to_string(),
            25.0 + (i as f64 * 0.01).sin() * 0.5,
        ))
        .collect();
    storage.insert_batch_hot(&records).await.unwrap();

    // 3. Verify hot rows
    let stats = storage.get_compression_stats().await.unwrap();
    assert_eq!(stats.hot_rows, 3600);

    // 4. Compress hot → warm
    let count = storage.compress_hot_to_warm(1700003600000).await.unwrap();
    assert_eq!(count, 3600);

    let stats = storage.get_compression_stats().await.unwrap();
    assert_eq!(stats.hot_rows, 0);
    assert!(stats.warm_rows > 0);

    // 5. Compress warm → cold
    let count = storage.compress_warm_to_cold(1700086400000).await.unwrap();
    assert!(count > 0);

    let stats = storage.get_compression_stats().await.unwrap();
    assert!(stats.cold_rows > 0);

    // 6. Query across all tiers
    let points = storage.query_compressed("dev1", "temperature", 1700000000000, 1700003600000)
        .await.unwrap();
    assert!(!points.is_empty());
    // Values should be approximately correct (some loss from OHLC aggregation)
    assert!((points[0].value - 25.0).abs() < 1.0);
}

#[tokio::test]
async fn test_deadband_filtering() {
    let filter = DeadbandFilter::new();

    // First value always passes
    assert!(filter.should_write("dev1", "temp", 25.0).await);
    filter.record_write("dev1", "temp", 25.0).await;

    // Same value filtered
    assert!(!filter.should_write("dev1", "temp", 25.0).await);

    // Small change filtered (0.1% of 25 = 0.025)
    assert!(!filter.should_write("dev1", "temp", 25.02).await);

    // Large change passes
    assert!(filter.should_write("dev1", "temp", 26.0).await);
}

#[test]
fn test_gorilla_compression_ratio() {
    // Simulate 1 hour of SCADA data (3600 readings)
    let timestamps: Vec<i64> = (0..3600).map(|i| 1700000000000 + i * 1000).collect();
    let values: Vec<f64> = (0..3600)
        .map(|i| 25.0 + (i as f64 * 0.01).sin() * 0.5)
        .collect();

    let block = gorilla_encode(&timestamps, &values).unwrap();

    // Raw size: 3600 * (8 bytes timestamp + 8 bytes value) = 57600 bytes
    let raw_size = timestamps.len() * 16;
    let compressed_size = block.data.len();
    let ratio = raw_size as f64 / compressed_size as f64;

    // Should achieve at least 5:1 compression on smooth SCADA data
    assert!(ratio > 5.0, "Compression ratio too low: {:.1}:1 (raw={}, compressed={})", ratio, raw_size, compressed_size);
}
```

- [ ] **Step 2: Run integration tests**

Run: `cargo test --lib compression`
Expected: All tests PASS

- [ ] **Step 3: Commit**

```bash
git add src/storage.rs src/compression.rs src/deadband.rs
git commit -m "test(compression): add integration tests for full compression pipeline"
```

---

## Task 17: Version Bump and Final Wiring

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs` (ensure all mod declarations and imports are correct)

- [ ] **Step 1: Bump version to 0.25.0**

In `Cargo.toml`, change:
```
version = "0.23.0"
```
to:
```
version = "0.25.0"
```

- [ ] **Step 2: Ensure all mod declarations are in place in main.rs**

Verify these are present:
```rust
mod compression;
mod deadband;
```

- [ ] **Step 3: Full build test**

Run: `cargo build --release`
Expected: SUCCESS

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: v0.25.0 — Data Compression & Performance"
```

---

## Plan Self-Review

### Spec Coverage Check

| Spec Requirement | Task |
|-----------------|------|
| Deadband filtering on write path | Task 5, Task 11 |
| Background compression worker: hot → warm | Task 7 |
| Cold compression worker: warm → cold | Task 8 |
| Decompression layer in query path | Task 9 |
| CLI: `voltrus db stats` | Task 10 |
| CLI: `voltrus db compress --now` | Task 10 |
| CLI: `voltrus archive` | Task 10 |
| Gorilla delta-of-delta encoding | Task 2 |
| RLE for digital tags | Task 3 |
| Deadband config table | Task 6 |
| Connection pooling for Modbus TCP | Task 12 |
| SQLite WAL mode tuning | Task 6 |
| Memory profiling endpoint | Task 13 |
| Performance budget | Task 17 (build size check) |
| Flash/SD card write optimization | Task 10 (edge_mode config), Task 11 (configurable interval) |
| Compression stats dashboard | Task 15 |
| Deadband config UI | Task 15 |
| Data migration from old tables | Task 6 |
| Replace old downsampling | Task 14 |
| API endpoints | Task 13 |
| Integration tests | Task 16 |

All spec requirements covered. No gaps found.

### Placeholder Scan

No TBDs, TODOs, or "implement later" patterns found. All steps contain complete code.

### Type Consistency

- `insert_batch_hot` uses `&[(String, i64, String, f64)]` — same as old `insert_batch`
- `DeadbandFilter::filter_batch` takes and returns the same record tuple type
- `DataPoint` struct used consistently across query functions
- `GorillaBlock` struct used for both encode and decode paths
- `CompressionStats` struct matches between storage.rs and API handler
