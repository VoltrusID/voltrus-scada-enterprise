# Protocol Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the battle-tested protocol improvements from MacTools into the Voltrus SCADA — fixing broken Modbus writes, adding Float64 support, enabling bulk reads, and fixing the S7 write hardcode.

**Architecture:** Directly port the `value_to_words`, bulk-read grouping, and register-type-aware write dispatch from MacTools `mt-modbus` crate into the equivalent Voltrus modules. The S7 write path gets a `data_type` parameter instead of hardcoding REAL. Shared protocol types (`Backoff`) are unified.

**Tech Stack:** Rust, tokio, tokio-modbus, serde, thiserror

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `src/models/config_types.rs` | Add `Float64` to `RegisterType`, update `register_count` |
| Modify | `src/modbus/types.rs` | Add `value_to_words`, `Float64` parse_register arm |
| Modify | `src/modbus/write.rs` | Rewrite with type-aware writes (FC06/FC16/coil) |
| Modify | `src/modbus/poller.rs` | Port bulk-read grouping from MacTools |
| Modify | `src/modbus/tests.rs` | Add Float64, value_to_words, register_count tests |
| Modify | `src/s7/poller.rs` | Accept data_type param, use shared Backoff |
| No change | `src/modbus/pool.rs` | Already ahead of MacTools (bind_addr support) |

---

### Task 1: Add `RegisterType::Float64` and update `register_count`

**Files:**
- Modify: `src/models/config_types.rs:248-275`

- [ ] **Step 1: Write the failing test**

In `src/modbus/tests.rs`, add:

```rust
#[test]
fn test_register_count_float64() {
    let reg = RegisterConfig {
        name: "power".into(),
        address: 0,
        reg_type: RegisterType::Float64,
        scale: 1.0,
        register_area: "holding".into(),
    };
    assert_eq!(reg.register_count(), 4);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_register_count_float64 2>&1`
Expected: FAIL — `Float64` variant does not exist

- [ ] **Step 3: Add `Float64` variant and update `register_count`**

In `src/models/config_types.rs`, change:

```rust
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RegisterType {
    Int16,
    Uint16,
    Int32,
    Float32,
    Float64,
}
```

Update `register_count`:

```rust
impl RegisterConfig {
    pub fn register_count(&self) -> u16 {
        match self.reg_type {
            RegisterType::Int16 | RegisterType::Uint16 => 1,
            RegisterType::Int32 | RegisterType::Float32 => 2,
            RegisterType::Float64 => 4,
        }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_register_count_float64 2>&1`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/models/config_types.rs
git commit -m "feat: add RegisterType::Float64 for 64-bit register support"
```

---

### Task 2: Add `value_to_words` and `Float64` parse_register to modbus types

**Files:**
- Modify: `src/modbus/types.rs`

- [ ] **Step 1: Write the failing tests**

In `src/modbus/tests.rs`, add:

```rust
#[test]
fn test_parse_float64_big_endian() {
    // 220.5 as f64 = 0x406B900000000000
    let words: Vec<u16> = vec![0x406B, 0x9000, 0x0000, 0x0000];
    let val = parse_register(&words, &RegisterType::Float64, &Endianness::Big);
    assert!((val - 220.5).abs() < 0.001);
}

#[test]
fn test_value_to_words_uint16() {
    let words = value_to_words(42.0, &RegisterType::Uint16, &Endianness::Big);
    assert_eq!(words, vec![42]);
}

#[test]
fn test_value_to_words_float32() {
    let words = value_to_words(220.5, &RegisterType::Float32, &Endianness::Big);
    assert_eq!(words, vec![0x435C, 0x8000]);
}

#[test]
fn test_value_to_words_int32() {
    let words = value_to_words(1000.0, &RegisterType::Int32, &Endianness::Big);
    assert_eq!(words, vec![0x0000, 0x03E8]);
}

#[test]
fn test_value_to_words_float64() {
    let words = value_to_words(220.5, &RegisterType::Float64, &Endianness::Big);
    assert_eq!(words, vec![0x406B, 0x9000, 0x0000, 0x0000]);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_parse_float64 test_value_to_words 2>&1`
Expected: FAIL — `value_to_words` does not exist, `Float64` arm missing in `parse_register`

- [ ] **Step 3: Add `Float64` arm to `parse_register` and add `value_to_words`**

In `src/modbus/types.rs`, update `parse_register` to add the `Float64` arm:

```rust
pub fn parse_register(words: &[u16], reg_type: &RegisterType, endianness: &Endianness) -> f64 {
    match reg_type {
        RegisterType::Uint16 => words[0] as f64,
        RegisterType::Int16 => (words[0] as i16) as f64,
        RegisterType::Int32 => {
            let raw = combine_u16(words[0], words[1], endianness);
            raw as i32 as f64
        }
        RegisterType::Float32 => {
            let raw = combine_u16(words[0], words[1], endianness);
            f32::from_bits(raw) as f64
        }
        RegisterType::Float64 => {
            let hi = combine_u16(words[0], words[1], endianness);
            let lo = combine_u16(words[2], words[3], endianness);
            let bits = ((hi as u64) << 32) | (lo as u64);
            f64::from_bits(bits)
        }
    }
}
```

Add `value_to_words` and `split_u32` after `combine_u16`:

```rust
pub(crate) fn value_to_words(value: f64, reg_type: &RegisterType, endianness: &Endianness) -> Vec<u16> {
    match reg_type {
        RegisterType::Uint16 => vec![value as u16],
        RegisterType::Int16 => vec![(value as i16) as u16],
        RegisterType::Int32 => {
            let raw = (value as i32) as u32;
            split_u32(raw, endianness)
        }
        RegisterType::Float32 => {
            let raw = (value as f32).to_bits();
            split_u32(raw, endianness)
        }
        RegisterType::Float64 => {
            let raw = value.to_bits();
            let hi = split_u32((raw >> 32) as u32, endianness);
            let lo = split_u32((raw & 0xFFFFFFFF) as u32, endianness);
            [hi, lo].concat()
        }
    }
}

fn split_u32(raw: u32, endianness: &Endianness) -> Vec<u16> {
    match endianness {
        Endianness::Big => vec![(raw >> 16) as u16, raw as u16],
        Endianness::Little => vec![raw as u16, (raw >> 16) as u16],
        Endianness::Swapped => {
            let h = (raw >> 16) as u16;
            let l = raw as u16;
            vec![h.swap_bytes(), l.swap_bytes()]
        }
    }
}
```

- [ ] **Step 4: Update `mod.rs` to export `value_to_words` for the test module**

In `src/modbus/mod.rs`, add to the `#[cfg(test)]` block:

```rust
#[cfg(test)]
pub use types::*;
```

This line already exists — just verify `value_to_words` is accessible via `use super::*` in the test file.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test test_parse_float64 test_value_to_words 2>&1`
Expected: ALL PASS

- [ ] **Step 6: Commit**

```bash
git add src/modbus/types.rs src/modbus/tests.rs
git commit -m "feat: add Float64 parsing and value_to_words for type-aware Modbus writes"
```

---

### Task 3: Rewrite Modbus write with type-aware dispatch

**Files:**
- Modify: `src/modbus/write.rs`

- [ ] **Step 1: Write the full replacement for `src/modbus/write.rs`**

Replace the entire file with:

```rust
use tokio_modbus::prelude::*;

use crate::models::*;
use super::types::value_to_words;

pub async fn write_tag(
    pool: &crate::modbus::pool::ModbusPool,
    config: &DeviceConfig,
    register: &RegisterConfig,
    value: f64,
) -> WriteResult {
    let start = std::time::Instant::now();
    let addr = format!("{}:{}", config.ip, config.port);
    let socket_addr: std::net::SocketAddr = match addr.parse() {
        Ok(a) => a,
        Err(e) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Invalid address: {e}")),
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    // Coil writes
    if register.register_area == "coil" {
        return write_coil(pool, &config.id, socket_addr, config.unit_id, register.protocol_address(), value != 0.0, start).await;
    }

    let words = value_to_words(value, &register.reg_type, &config.endianness);

    let ctx_result = pool
        .get_or_connect(&config.id, socket_addr, config.unit_id, None)
        .await;

    let mut ctx = match ctx_result {
        Ok(ctx) => ctx,
        Err(e) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Modbus connection failed: {e}")),
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    let protocol_addr = register.protocol_address();

    let write_result = if words.len() == 1 {
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            ctx.write_single_register(protocol_addr, words[0]),
        )
        .await
    } else {
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            ctx.write_multiple_registers(protocol_addr, &words),
        )
        .await
    };

    match write_result {
        Ok(Ok(_)) => {
            pool.return_connection(&config.id, ctx).await;
            WriteResult {
                success: true,
                device_response: Some(format!(
                    "Wrote {} {} = {} (raw: {:?})",
                    register.name, protocol_addr, value, words
                )),
                error: None,
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
        Ok(Err(e)) => {
            pool.remove(&config.id).await;
            WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Modbus write failed: {e}")),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(_) => {
            pool.remove(&config.id).await;
            WriteResult {
                success: false,
                device_response: None,
                error: Some("Modbus write timeout (5s)".into()),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
    }
}

async fn write_coil(
    pool: &crate::modbus::pool::ModbusPool,
    device_id: &str,
    socket_addr: std::net::SocketAddr,
    unit_id: u8,
    address: u16,
    coil_on: bool,
    start: std::time::Instant,
) -> WriteResult {
    let ctx_result = pool
        .get_or_connect(device_id, socket_addr, unit_id, None)
        .await;

    let mut ctx = match ctx_result {
        Ok(ctx) => ctx,
        Err(e) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Modbus connection failed: {e}")),
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    // Try FC05 (Write Single Coil), fall back to FC0F (Write Multiple Coils)
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ctx.write_single_coil(address, coil_on),
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            pool.return_connection(device_id, ctx).await;
            WriteResult {
                success: true,
                device_response: Some(format!("Wrote coil {} = {}", address, coil_on)),
                error: None,
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
        _ => {
            let fallback = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                ctx.write_multiple_coils(address, &[coil_on]),
            )
            .await;

            match fallback {
                Ok(Ok(_)) => {
                    pool.return_connection(device_id, ctx).await;
                    WriteResult {
                        success: true,
                        device_response: Some(format!(
                            "Wrote coil {} = {} (FC0F fallback)",
                            address, coil_on
                        )),
                        error: None,
                        latency_ms: start.elapsed().as_millis() as u64,
                    }
                }
                Ok(Err(e)) => {
                    pool.remove(device_id).await;
                    WriteResult {
                        success: false,
                        device_response: None,
                        error: Some(format!("Modbus coil write failed: {e}")),
                        latency_ms: start.elapsed().as_millis() as u64,
                    }
                }
                Err(_) => {
                    pool.remove(device_id).await;
                    WriteResult {
                        success: false,
                        device_response: None,
                        error: Some("Modbus coil write timeout (5s)".into()),
                        latency_ms: start.elapsed().as_millis() as u64,
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Update callers of `write_tag`**

The signature changed from `write_tag(pool, config, address, value)` to `write_tag(pool, config, register, value)`. Update callers:

**`src/api/websocket.rs:406`** — find the write dispatch and change it to pass the full register config. The current code looks like:

```rust
crate::modbus::write_tag(&state.modbus_pool, &config, address, value).await
```

Change to pass the `RegisterConfig` instead of just the address. You'll need to look up the register from the device config by matching the address. Since the caller already has `address` and `value`, find the matching register:

```rust
let register = config.registers.iter().find(|r| r.protocol_address() == address).cloned().unwrap_or_else(|| {
    crate::models::RegisterConfig {
        name: "unknown".into(),
        address,
        reg_type: crate::models::RegisterType::Uint16,
        scale: 1.0,
        register_area: "holding".into(),
    }
});
crate::modbus::write_tag(&state.modbus_pool, &config, &register, value).await
```

**`src/command_queue/executor.rs:113`** — same change:

```rust
let register = crate::models::RegisterConfig {
    name: "command".into(),
    address,
    reg_type: crate::models::RegisterType::Uint16,
    scale: 1.0,
    register_area: "holding".into(),
};
crate::modbus::write_tag(&self.modbus_pool, &config, &register, value).await
```

- [ ] **Step 3: Build to verify compilation**

Run: `cargo build 2>&1`
Expected: Compiles with no errors

- [ ] **Step 4: Run existing tests**

Run: `cargo test 2>&1`
Expected: ALL PASS

- [ ] **Step 5: Commit**

```bash
git add src/modbus/write.rs src/api/websocket.rs src/command_queue/executor.rs
git commit -m "fix: rewrite Modbus write with type-aware dispatch (Float32/Float64/Int32/coils)"
```

---

### Task 4: Port bulk-read grouping into Modbus poller

**Files:**
- Modify: `src/modbus/poller.rs`

- [ ] **Step 1: Replace the per-register loop with bulk-read grouping**

In `src/modbus/poller.rs`, replace the inner polling loop (the `for reg in &config.registers` block at lines 73-106) with the MacTools grouping algorithm:

Replace the block from `let mut metrics = HashMap::new();` through `pool.return_connection(...)` with:

```rust
        let mut metrics = HashMap::new();

        // Sort registers by area then address for optimal grouping
        let mut sorted_regs: Vec<&RegisterConfig> = config.registers.iter().collect();
        sorted_regs.sort_by_key(|r| (r.register_area.clone(), r.protocol_address()));

        let mut i = 0;
        while i < sorted_regs.len() {
            let first = &sorted_regs[i];
            let area = &first.register_area;
            let start_addr = first.protocol_address();
            let mut end_addr = start_addr + first.register_count();

            let mut group_end = i + 1;
            while group_end < sorted_regs.len()
                && sorted_regs[group_end].register_area == *area
                && sorted_regs[group_end].protocol_address() <= end_addr
            {
                let t = &sorted_regs[group_end];
                let t_end = t.protocol_address() + t.register_count();
                if t_end > end_addr {
                    end_addr = t_end;
                }
                group_end += 1;
            }

            let total = (end_addr - start_addr) as u16;

            let read_result = if area == "input" {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    ctx.read_input_registers(start_addr, total),
                )
                .await
            } else {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    ctx.read_holding_registers(start_addr, total),
                )
                .await
            };

            let response = match read_result {
                Ok(Ok(Ok(words))) => words,
                Ok(Ok(Err(code))) => {
                    tracing::warn!("Device {} Modbus exception at addr {}: {:?}", config.id, start_addr, code);
                    for j in i..group_end {
                        metrics.insert(sorted_regs[j].name.clone(), f64::NAN);
                    }
                    i = group_end;
                    continue;
                }
                Ok(Err(e)) => {
                    anyhow::bail!("Modbus read failed for group starting at {}: {}", start_addr, e);
                }
                Err(_) => {
                    anyhow::bail!("Modbus read timeout for group starting at {}", start_addr);
                }
            };

            for j in i..group_end {
                let reg = &sorted_regs[j];
                let offset = (reg.protocol_address() - start_addr) as usize;
                let count = reg.register_count() as usize;
                if offset + count > response.len() {
                    metrics.insert(reg.name.clone(), f64::NAN);
                    continue;
                }
                let value = parse_register(&response[offset..offset + count], &reg.reg_type, &config.endianness) * reg.scale;
                metrics.insert(reg.name.clone(), value);
            }

            i = group_end;
        }

        cache.update(&config.id, metrics, false).await;
        tracing::debug!("Device {} polled successfully", config.id);

        pool.return_connection(&config.id, ctx).await;
```

- [ ] **Step 2: Build to verify compilation**

Run: `cargo build 2>&1`
Expected: Compiles with no errors

- [ ] **Step 3: Run existing tests**

Run: `cargo test 2>&1`
Expected: ALL PASS

- [ ] **Step 4: Commit**

```bash
git add src/modbus/poller.rs
git commit -m "perf: bulk-read contiguous Modbus registers in single request"
```

---

### Task 5: Fix S7 write hardcode — accept data_type from tag config

**Files:**
- Modify: `src/s7/poller.rs:92-186`

- [ ] **Step 1: Update `write_tag` signature and body**

Replace the S7 `write_tag` function in `src/s7/poller.rs` (lines 92-186) with:

```rust
pub async fn write_tag(
    ip: &str,
    port: u16,
    area: &str,
    db_number: u32,
    offset: u16,
    value: f64,
    data_type: Option<&str>,
) -> WriteResult {
    let start = std::time::Instant::now();
    let url = format!("{}:{}", ip, port);

    let s7_area = match parse_area_string(area) {
        Some(a) => a,
        None => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Unknown S7 area: '{}'", area)),
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    let s7_data_type = data_type
        .and_then(S7DataType::from_str_name)
        .unwrap_or(S7DataType::Real);

    let tag = S7Tag {
        area: s7_area,
        db_number: db_number as u16,
        offset,
        bit_offset: 0,
        data_type: s7_data_type.clone(),
        length: if s7_data_type == S7DataType::String { 254 } else { 1 },
        name: None,
    };

    let s7_value = match s7_data_type {
        S7DataType::Bool => S7Value::Bool(value != 0.0),
        S7DataType::Byte => S7Value::Byte(value as u8),
        S7DataType::Word => S7Value::Word(value as u16),
        S7DataType::DWord => S7Value::DWord(value as u32),
        S7DataType::Real => S7Value::Real(value as f32),
        S7DataType::String => S7Value::String(format!("{}", value)),
    };

    let mut client = S7Client::new(&url, 0, 1);
    let connect_result =
        tokio::time::timeout(std::time::Duration::from_secs(5), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("S7 connection failed: {e}")),
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
        Err(_) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some("S7 connect timeout (5s)".into()),
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }
    }

    let write_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.write_tag(&tag, &s7_value),
    )
    .await;

    let _ = client.disconnect().await;

    match write_result {
        Ok(Ok(())) => WriteResult {
            success: true,
            device_response: Some(format!(
                "Wrote {} {} offset {} = {} ({:?})",
                area, db_number, offset, value, s7_data_type
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
```

- [ ] **Step 2: Update callers of `s7::write_tag`**

**`src/command_queue/executor.rs:163`** — the current call is:

```rust
crate::s7::write_tag(&ip, port, &area, db_number, offset, value).await
```

Change to:

```rust
crate::s7::write_tag(&ip, port, &area, db_number, offset, value, None).await
```

The `None` defaults to `Real` for backward compatibility. Future callers can pass `Some("word")` etc.

- [ ] **Step 3: Build to verify compilation**

Run: `cargo build 2>&1`
Expected: Compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src/s7/poller.rs src/command_queue/executor.rs
git commit -m "fix: S7 write supports all data types instead of hardcoding REAL"
```

---

### Task 6: Unify S7 poller backoff with shared Backoff struct

**Files:**
- Modify: `src/s7/poller.rs:23-81`

- [ ] **Step 1: Replace hand-rolled backoff in `poll_s7_device`**

In `src/s7/poller.rs`, replace lines 25-26 and line 79 with the shared `Backoff` from `crate::modbus::types`. This requires making `Backoff` public at the crate level or duplicating it. Since `Backoff` is `pub(crate)`, it's already accessible.

Change `poll_s7_device`:

```rust
pub async fn poll_s7_device(config: S7DeviceConfig, cache: DeviceCache) {
    let device_id = config.id.clone();
    let mut backoff = crate::modbus::types::Backoff::new();

    loop {
        let mut client = S7Client::new(&config.url, config.rack, config.slot);

        match client.connect().await {
            Ok(()) => {
                tracing::info!("S7 device {} connected", device_id);
                backoff.reset();
                cache.set_status(&device_id, DeviceStatus::Online).await;

                loop {
                    match client.read_tags(&config.tags).await {
                        Ok(values) => {
                            let mut metrics = HashMap::new();
                            for (i, value) in values.iter().enumerate() {
                                let tag_name = if i < config.tags.len() {
                                    config.tags[i].name.clone().unwrap_or_else(|| {
                                        format!(
                                            "{}_{}",
                                            config.tags[i].area.to_string().to_lowercase(),
                                            config.tags[i].offset
                                        )
                                    })
                                } else {
                                    format!("tag_{}", i)
                                };
                                metrics.insert(tag_name, value.to_f64());
                            }
                            cache.update(&device_id, metrics, false).await;
                        }
                        Err(e) => {
                            tracing::warn!("S7 device {} read error: {}", device_id, e);
                            cache.set_status(&device_id, DeviceStatus::Offline).await;
                            break;
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(config.interval_ms)).await;
                }
            }
            Err(e) => {
                let delay = backoff.next();
                tracing::warn!(
                    "S7 device {} connection failed: {}, retrying in {}ms",
                    device_id,
                    e,
                    delay
                );
                cache.set_status(&device_id, DeviceStatus::Offline).await;
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                continue;
            }
        }

        let _ = client.disconnect().await;
        let delay = backoff.next();
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo build 2>&1`
Expected: Compiles with no errors

- [ ] **Step 3: Commit**

```bash
git add src/s7/poller.rs
git commit -m "refactor: unify S7 poller backoff with shared Backoff struct"
```

---

### Task 7: Final integration test

**Files:**
- No new files

- [ ] **Step 1: Run full test suite**

Run: `cargo test 2>&1`
Expected: ALL PASS

- [ ] **Step 2: Build release to verify no warnings**

Run: `cargo build 2>&1`
Expected: Clean build, no errors

- [ ] **Step 3: Verify all callers compile with the new signatures**

Run: `cargo check --all-features 2>&1`
Expected: No errors

---

## Self-Review Checklist

**1. Spec coverage:**
- [x] Modbus write broken (only u16) → Task 3
- [x] Missing Float64 → Task 1, 2
- [x] No bulk reads → Task 4
- [x] S7 write hardcoded REAL → Task 5
- [x] Inconsistent S7 backoff → Task 6
- [x] Missing value_to_words → Task 2
- [x] No coil write support → Task 3
- [x] No FC05/FC0F/FC16 dispatch → Task 3

**2. Placeholder scan:** No TBD, TODO, or "add appropriate X" found.

**3. Type consistency:**
- `write_tag` new signature: `(pool, config, register: &RegisterConfig, value: f64)` — callers updated in Task 3
- `s7::write_tag` new signature: adds `data_type: Option<&str>` — callers updated in Task 5
- `RegisterType::Float64` added in Task 1, used in Tasks 2 and 3
- `value_to_words` returns `Vec<u16>` — consistent with `write_single_register`/`write_multiple_registers` args
