#![allow(dead_code)]
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Frame types for the replication protocol.
pub const FRAME_HEARTBEAT: u8 = 0x01;
pub const FRAME_WAL_ENTRY: u8 = 0x02;
pub const FRAME_CONFIG_CHANGE: u8 = 0x03;
pub const FRAME_COMMAND_STATE: u8 = 0x04;
pub const FRAME_ROLE_CHANGE: u8 = 0x05;
pub const FRAME_SYNC_REQUEST: u8 = 0x10;
pub const FRAME_SYNC_SNAPSHOT: u8 = 0x11;
pub const FRAME_SYNC_ACK: u8 = 0x12;

/// Write a replication frame to a TCP stream.
pub async fn write_frame(stream: &mut TcpStream, frame_type: u8, payload: &[u8]) -> io::Result<()> {
    let len = payload.len() as u32;
    stream.write_u32(len).await?;
    stream.write_u8(frame_type).await?;
    stream.write_all(payload).await?;
    Ok(())
}

/// Read a replication frame from a TCP stream.
pub async fn read_frame(stream: &mut TcpStream) -> io::Result<(u8, Vec<u8>)> {
    let len = stream.read_u32().await?;
    let frame_type = stream.read_u8().await?;
    let mut payload = vec![0u8; len as usize];
    stream.read_exact(&mut payload).await?;
    Ok((frame_type, payload))
}

/// Encode a heartbeat frame.
pub fn encode_heartbeat(ts_ms: u64) -> Vec<u8> {
    ts_ms.to_be_bytes().to_vec()
}

/// Decode a heartbeat frame.
pub fn decode_heartbeat(payload: &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&payload[..8]);
    u64::from_be_bytes(bytes)
}

use tokio::net::TcpListener;
use tokio::time::{self, Duration};

use crate::ha::HaState;
use voltrus::models::HaConfig;

/// Runs the replication server on the primary. Accepts one standby connection
/// and streams heartbeats + WAL frames.
pub async fn run_replication_server(config: &HaConfig, ha_state: HaState) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&config.bind_address).await?;
    tracing::info!("HA replication server listening on {}", config.bind_address);

    loop {
        let (mut stream, addr) = listener.accept().await?;
        tracing::info!("Standby connected from {}", addr);

        let state = ha_state.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(1000));
            loop {
                interval.tick().await;
                let ts = voltrus::timeutil::now_millis() as u64;
                let payload = encode_heartbeat(ts);
                if let Err(e) = write_frame(&mut stream, FRAME_HEARTBEAT, &payload).await {
                    tracing::warn!("Failed to send heartbeat to standby: {}", e);
                    state.set_peer_status("disconnected", None).await;
                    break;
                }
                state.set_peer_status("connected", Some(0)).await;
            }
        });
    }
}

/// Runs the replication client on the standby. Connects to primary,
/// receives heartbeats + WAL frames, monitors for failover.
pub async fn run_replication_client(config: &HaConfig, ha_state: HaState) -> anyhow::Result<()> {
    let failover_threshold = config.failover_after_missed;
    let hb_interval_ms = config.heartbeat_interval_ms;

    loop {
        tracing::info!("Connecting to primary at {}", config.peer_address);

        match TcpStream::connect(&config.peer_address).await {
            Ok(mut stream) => {
                tracing::info!("Connected to primary");
                ha_state.set_peer_status("connected", Some(0)).await;

                let mut missed = 0u32;
                let timeout = Duration::from_millis(hb_interval_ms * 3);

                loop {
                    match tokio::time::timeout(timeout, read_frame(&mut stream)).await {
                        Ok(Ok((frame_type, payload))) => {
                            missed = 0;
                            match frame_type {
                                FRAME_HEARTBEAT => {
                                    let _ts = decode_heartbeat(&payload);
                                }
                                FRAME_WAL_ENTRY | FRAME_CONFIG_CHANGE | FRAME_ROLE_CHANGE => {
                                    tracing::debug!("Received frame type 0x{:02x}", frame_type);
                                }
                                _ => {
                                    tracing::warn!("Unknown frame type: 0x{:02x}", frame_type);
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("Read error from primary: {}", e);
                            missed += 1;
                        }
                        Err(_) => {
                            missed += 1;
                            tracing::warn!("Missed heartbeat ({}/{})", missed, failover_threshold);
                        }
                    }

                    if missed >= failover_threshold {
                        tracing::warn!("Failover threshold reached! Promoting to primary");
                        ha_state.set_role(crate::ha::HaRole::Primary).await;
                        ha_state.set_peer_status("disconnected", None).await;
                        break;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Cannot connect to primary: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
