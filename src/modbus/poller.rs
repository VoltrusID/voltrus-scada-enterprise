use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio_modbus::prelude::*;

use crate::cache::DeviceCache;
use crate::models::*;

use super::pool::ModbusPool;
use super::types::{Backoff, parse_register};

pub async fn poll_device(
    config: DeviceConfig,
    cache: DeviceCache,
    pool: Arc<ModbusPool>,
    bind_addr: Option<std::net::SocketAddr>,
) {
    let device_id = config.id.clone();
    let mut backoff = Backoff::new();

    loop {
        match try_poll(&config, &cache, &pool, bind_addr).await {
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

async fn try_poll(
    config: &DeviceConfig,
    cache: &DeviceCache,
    pool: &ModbusPool,
    bind_addr: Option<std::net::SocketAddr>,
) -> Result<()> {
    let addr = format!("{}:{}", config.ip, config.port);
    let socket_addr: std::net::SocketAddr = addr.parse().context("Invalid address")?;

    tracing::info!("Polling device {} at {}", config.id, addr);

    loop {
        let mut ctx = pool
            .get_or_connect(&config.id, socket_addr, config.unit_id, bind_addr)
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
