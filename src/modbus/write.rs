use tokio_modbus::prelude::*;

use crate::models::*;

pub async fn write_tag(
    pool: &crate::modbus::pool::ModbusPool,
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
            };
        }
    };

    if value < 0.0 || value > 65535.0 {
        return WriteResult {
            success: false,
            device_response: None,
            error: Some(format!(
                "Value {value} out of range for u16 register (0-65535)"
            )),
            latency_ms: start.elapsed().as_millis() as u64,
        };
    }

    let raw_value = value.round() as u16;

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

    let write_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ctx.write_single_register(address, raw_value),
    )
    .await;

    match write_result {
        Ok(Ok(_)) => {
            pool.return_connection(&config.id, ctx).await;
            WriteResult {
                success: true,
                device_response: Some(format!(
                    "Wrote register {} = {} (raw: {raw_value})",
                    address, value
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
