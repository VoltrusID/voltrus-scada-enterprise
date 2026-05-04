use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio_modbus::client::tcp;
use tokio_modbus::prelude::*;

use crate::cache::DeviceCache;
use crate::models::*;

// ── Connection Pool ──────────────────────────────────────────────────

struct PooledConnection {
    ctx: tokio_modbus::client::Context,
    last_used: Instant,
}

pub struct ModbusPool {
    connections: Arc<tokio::sync::Mutex<HashMap<String, PooledConnection>>>,
    max_idle: Duration,
}

impl ModbusPool {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
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
        let ctx = tcp::connect_slave(addr, Slave(unit_id)).await?;
        Ok(ctx)
    }

    pub async fn return_connection(&self, device_id: &str, ctx: tokio_modbus::client::Context) {
        let mut conns = self.connections.lock().await;
        conns.insert(
            device_id.to_string(),
            PooledConnection {
                ctx,
                last_used: Instant::now(),
            },
        );
    }

    pub async fn remove(&self, device_id: &str) {
        let mut conns = self.connections.lock().await;
        conns.remove(device_id);
    }
}

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
    }
}

fn combine_u16(high: u16, low: u16, endianness: &Endianness) -> u32 {
    match endianness {
        Endianness::Big => ((high as u32) << 16) | (low as u32),
        Endianness::Little => ((low as u32) << 16) | (high as u32),
        Endianness::Swapped => {
            let h = high.swap_bytes();
            let l = low.swap_bytes();
            ((l as u32) << 16) | (h as u32)
        }
    }
}

pub async fn poll_device(config: DeviceConfig, cache: DeviceCache, pool: Arc<ModbusPool>) {
    let device_id = config.id.clone();
    let mut backoff = Backoff::new();

    loop {
        match try_poll(&config, &cache, &pool).await {
            Ok(()) => backoff.reset(),
            Err(e) => {
                tracing::warn!("Device {device_id} error: {e}");
                cache.set_status(&device_id, DeviceStatus::Offline).await;
                pool.remove(&device_id).await;
                let delay = backoff.next();
                tracing::info!("Device {device_id} retrying in {delay}ms");
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
        }
    }
}

async fn try_poll(config: &DeviceConfig, cache: &DeviceCache, pool: &ModbusPool) -> Result<()> {
    let addr = format!("{}:{}", config.ip, config.port);
    let socket_addr: std::net::SocketAddr = addr.parse().context("Invalid address")?;

    tracing::info!("Polling device {} at {}", config.id, addr);

    loop {
        let mut ctx = pool
            .get_or_connect(&config.id, socket_addr, config.unit_id)
            .await
            .context("Modbus connection failed")?;

        let mut metrics = HashMap::new();

        for reg in &config.registers {
            let addr = reg.protocol_address();
            let count = reg.register_count();

            let read_result = if reg.register_area == "input" {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    ctx.read_input_registers(addr, count),
                ).await
            } else {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    ctx.read_holding_registers(addr, count),
                ).await
            };

            let response = read_result
                .context("Modbus read timeout")?
                .context("Modbus read failed")?
                .map_err(|code| anyhow::anyhow!("Modbus exception: {code:?}"))?;

            if response.len() < count as usize {
                anyhow::bail!(
                    "Short read for register {}: expected {count}, got {}",
                    reg.name,
                    response.len()
                );
            }

            let value = parse_register(&response, &reg.reg_type, &config.endianness) * reg.scale;
            metrics.insert(reg.name.clone(), value);
        }

        cache.update(&config.id, metrics, false).await;
        tracing::debug!("Device {} polled successfully", config.id);

        pool.return_connection(&config.id, ctx).await;
        tokio::time::sleep(std::time::Duration::from_millis(config.interval_ms)).await;
    }
}

pub async fn scan_registers(
    ip: &str,
    port: u16,
    unit_id: u8,
    start_addr: u16,
    end_addr: u16,
    endianness: &Endianness,
    register_area: &str,
    scan_delay_ms: u64,
) -> Result<Vec<ScannedRegister>> {
    let addr = format!("{}:{}", ip, port);
    let socket_addr: std::net::SocketAddr = addr.parse().context("Invalid address")?;
    let mut ctx = tcp::connect_slave(socket_addr, Slave(unit_id))
        .await
        .context("Modbus connection failed")?;

    let total = (end_addr - start_addr + 1) as usize;
    let mut results = vec![ScannedRegister {
        address: 0,
        raw_value: None,
        raw_value2: None,
        changed: false,
        readable: false,
        likely_type: "uint16".to_string(),
        interpreted_value: None,
    }; total];

    for (i, r) in results.iter_mut().enumerate() {
        r.address = start_addr + i as u16;
    }

    let batch_size: u16 = 20;
    let use_input = register_area == "input";
    let use_coils = register_area == "coil";
    let use_discrete = register_area == "discrete";

    // First pass: read in batches
    let mut addr = start_addr;
    while addr <= end_addr {
        let count = std::cmp::min(batch_size, end_addr - addr + 1);
        let read_result = if use_coils {
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                ctx.read_coils(addr, count),
            ).await.map(|r| r.map(|r| r.map(|coils| coils.iter().map(|&c| if c { 1u16 } else { 0u16 }).collect::<Vec<u16>>())))
        } else if use_discrete {
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                ctx.read_discrete_inputs(addr, count),
            ).await.map(|r| r.map(|r| r.map(|bits| bits.iter().map(|&b| if b { 1u16 } else { 0u16 }).collect::<Vec<u16>>())))
        } else if use_input {
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                ctx.read_input_registers(addr, count),
            ).await
        } else {
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                ctx.read_holding_registers(addr, count),
            ).await
        };

        match read_result {
            Ok(Ok(Ok(response))) => {
                let offset = (addr - start_addr) as usize;
                for (j, &val) in response.iter().enumerate() {
                    if offset + j < total {
                        results[offset + j].raw_value = Some(val);
                        results[offset + j].readable = true;
                    }
                }
            }
            _ => {
                // Batch failed, skip — individual registers stay unreadable
            }
        }
        addr += count;
    }

    // Wait and second pass to detect changes
    tokio::time::sleep(std::time::Duration::from_millis(scan_delay_ms)).await;

    let mut addr = start_addr;
    while addr <= end_addr {
        let count = std::cmp::min(batch_size, end_addr - addr + 1);
        let offset = (addr - start_addr) as usize;
        let has_readable = (offset..std::cmp::min(offset + count as usize, total))
            .any(|i| results[i].readable);

        if has_readable {
            let read_result = if use_coils {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    ctx.read_coils(addr, count),
                ).await.map(|r| r.map(|r| r.map(|coils| coils.iter().map(|&c| if c { 1u16 } else { 0u16 }).collect::<Vec<u16>>())))
            } else if use_discrete {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    ctx.read_discrete_inputs(addr, count),
                ).await.map(|r| r.map(|r| r.map(|bits| bits.iter().map(|&b| if b { 1u16 } else { 0u16 }).collect::<Vec<u16>>())))
            } else if use_input {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    ctx.read_input_registers(addr, count),
                ).await
            } else {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    ctx.read_holding_registers(addr, count),
                ).await
            };

            if let Ok(Ok(Ok(response))) = read_result {
                for (j, &val) in response.iter().enumerate() {
                    if offset + j < total && results[offset + j].readable {
                        results[offset + j].raw_value2 = Some(val);
                        if results[offset + j].raw_value != Some(val) {
                            results[offset + j].changed = true;
                        }
                    }
                }
            }
        }
        addr += count;
    }

    // Type heuristic: skip for coils/discrete (single-bit values)
    if !use_coils && !use_discrete {
        let mut consumed = vec![false; total];
        for i in 0..total {
            if consumed[i] || !results[i].readable {
                continue;
            }
            if i + 1 < total && results[i + 1].readable {
                let v1 = results[i].raw_value.unwrap_or(0);
                let v2 = results[i + 1].raw_value.unwrap_or(0);
                let raw = combine_u16(v1, v2, endianness);
                let f = f32::from_bits(raw);

                if f.is_finite() && f.abs() > 0.001 && f.abs() < 1_000_000.0 {
                    results[i].likely_type = "float32".to_string();
                    results[i].interpreted_value = Some(f as f64);
                    consumed[i] = true;
                    consumed[i + 1] = true;
                    results[i + 1].likely_type = "float32_lo".to_string();
                    continue;
                }

                let signed = raw as i32;
                if signed.abs() < 1_000_000 {
                    results[i].likely_type = "int32".to_string();
                    results[i].interpreted_value = Some(signed as f64);
                    consumed[i] = true;
                    consumed[i + 1] = true;
                    results[i + 1].likely_type = "int32_lo".to_string();
                    continue;
                }
            }
            consumed[i] = true;
            let v = results[i].raw_value.unwrap_or(0);
            if (v as i16) < 0 {
                results[i].likely_type = "int16".to_string();
                results[i].interpreted_value = Some((v as i16) as f64);
            } else {
                results[i].likely_type = "uint16".to_string();
                results[i].interpreted_value = Some(v as f64);
            }
        }
    } else {
        // Coils/discrete: just 0 or 1
        for r in &mut results {
            if r.readable {
                r.interpreted_value = Some(r.raw_value.unwrap_or(0) as f64);
            }
        }
    }

    // Convert to 5-digit convention and filter out "lo" registers
    let offset: u16 = match register_area {
        "input" => 30001,
        "coil" => 1,
        "discrete" => 10001,
        _ => 40001,
    };
    Ok(results
        .into_iter()
        .filter(|r| !r.likely_type.ends_with("_lo"))
        .map(|mut r| { r.address = r.address.wrapping_add(offset); r })
        .collect())
}

pub async fn write_tag(
    config: &DeviceConfig,
    address: u16,
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
            }
        }
    };

    let connect_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tcp::connect_slave(socket_addr, Slave(config.unit_id)),
    ).await;

    let mut ctx = match connect_result {
        Ok(Ok(ctx)) => ctx,
        Ok(Err(e)) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some(format!("Modbus connection failed: {e}")),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(_) => {
            return WriteResult {
                success: false,
                device_response: None,
                error: Some("Modbus connect timeout (5s)".into()),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
    };

    let raw_value = value as u16;
    let write_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ctx.write_single_register(address, raw_value),
    ).await;

    match write_result {
        Ok(Ok(_)) => WriteResult {
            success: true,
            device_response: Some(format!(
                "Wrote register {} = {} (raw: {raw_value})",
                address, value
            )),
            error: None,
            latency_ms: start.elapsed().as_millis() as u64,
        },
        Ok(Err(e)) => WriteResult {
            success: false,
            device_response: None,
            error: Some(format!("Modbus write failed: {e}")),
            latency_ms: start.elapsed().as_millis() as u64,
        },
        Err(_) => WriteResult {
            success: false,
            device_response: None,
            error: Some("Modbus write timeout (5s)".into()),
            latency_ms: start.elapsed().as_millis() as u64,
        },
    }
}

struct Backoff {
    attempt: u32,
    max_delay_ms: u64,
}

impl Backoff {
    fn new() -> Self {
        Self { attempt: 0, max_delay_ms: 30_000 }
    }

    fn reset(&mut self) { self.attempt = 0; }

    fn next(&mut self) -> u64 {
        let attempt = self.attempt.min(30);
        let delay = 1000 * 2u64.pow(attempt);
        self.attempt = self.attempt.saturating_add(1);
        delay.min(self.max_delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uint16() {
        assert_eq!(parse_register(&[1000u16], &RegisterType::Uint16, &Endianness::Big), 1000.0);
    }

    #[test]
    fn test_parse_int16_negative() {
        assert_eq!(parse_register(&[0xFFFFu16], &RegisterType::Int16, &Endianness::Big), -1.0);
    }

    #[test]
    fn test_parse_float32_big_endian() {
        let val = parse_register(&[0x435C, 0x8000], &RegisterType::Float32, &Endianness::Big);
        assert!((val - 220.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_float32_little_endian() {
        let val = parse_register(&[0x8000, 0x435C], &RegisterType::Float32, &Endianness::Little);
        assert!((val - 220.5).abs() < 0.01);
    }

    #[test]
    fn test_combine_big_endian() {
        assert_eq!(combine_u16(0x1234, 0x5678, &Endianness::Big), 0x12345678);
    }

    #[test]
    fn test_combine_little_endian() {
        assert_eq!(combine_u16(0x5678, 0x1234, &Endianness::Little), 0x12345678);
    }

    #[test]
    fn test_backoff_sequence() {
        let mut b = Backoff::new();
        assert_eq!(b.next(), 1000);
        assert_eq!(b.next(), 2000);
        assert_eq!(b.next(), 4000);
    }

    #[test]
    fn test_backoff_resets() {
        let mut b = Backoff::new();
        b.next(); b.next();
        b.reset();
        assert_eq!(b.next(), 1000);
    }

    #[test]
    fn test_backoff_max() {
        let mut b = Backoff::new();
        for _ in 0..20 { b.next(); }
        assert_eq!(b.next(), 30_000);
    }
}
