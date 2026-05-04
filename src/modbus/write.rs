use tokio_modbus::client::tcp;
use tokio_modbus::prelude::*;

use crate::models::*;

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
