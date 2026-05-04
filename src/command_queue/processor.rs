use super::*;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cache::DeviceCache;
use crate::storage::Storage;
use crate::ws_manager::WsManager;

impl CommandQueue {
    pub fn new(storage: Storage, cache: DeviceCache) -> Self {
        Self {
            storage,
            cache,
            in_flight: Arc::new(RwLock::new(HashSet::new())),
            ws_manager: None,
            readback_trackers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Attach a WsManager so that command status changes are broadcast over WebSocket.
    pub fn with_ws_manager(mut self, ws_manager: Arc<WsManager>) -> Self {
        self.ws_manager = Some(ws_manager);
        self
    }

    /// Main loop: process the queue every 500 ms.
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(POLL_INTERVAL_MS));
        loop {
            interval.tick().await;
            if let Err(e) = self.process_queue().await {
                tracing::warn!("Command queue processor error: {}", e);
            }
        }
    }

    async fn process_queue(&self) -> anyhow::Result<()> {
        let commands = self.storage.get_queued_commands().await?;

        for cmd in &commands {
            let now = crate::timeutil::now_secs();

            // -- Expiry check --
            if cmd.status == "queued" && (cmd.created_at + COMMAND_EXPIRY_SECS) < now {
                self.publish_status_event(&cmd.id, "expired", None);
                self.storage
                    .update_command_status(
                        &cmd.id,
                        "expired",
                        Some("Command expired (30 min timeout)"),
                        "system",
                        Some("auto-expired"),
                    )
                    .await?;
                continue;
            }

            // -- Scheduled check --
            if cmd.policy == "scheduled" {
                if let Some(scheduled_at) = cmd.scheduled_at {
                    if scheduled_at > now {
                        continue; // Not yet time
                    }
                }
            }

            // -- Concurrency check: only one command per device in-flight --
            {
                let in_flight = self.in_flight.read().await;
                if in_flight.contains(&cmd.device_id) {
                    continue;
                }
            }

            // -- Device online check --
            let device_id_str = cmd.device_id.to_string();
            if !self.cache.is_device_online(&device_id_str).await {
                continue;
            }

            // Mark as in-flight
            {
                let mut in_flight = self.in_flight.write().await;
                in_flight.insert(cmd.device_id);
            }

            // -- Execute --
            self.publish_status_event(&cmd.id, "sent", None);
            self.storage
                .update_command_status(&cmd.id, "sent", None, "system", None)
                .await?;

            let result = self.execute_write(cmd).await;

            // -- Handle result --
            match result.success {
                true => {
                    let response = result
                        .device_response
                        .clone()
                        .unwrap_or_else(|| "OK".to_string());
                    self.publish_ack_event(&cmd.id, &response, result.latency_ms);
                    self.storage
                        .update_command_status(&cmd.id, "acked", None, "system", None)
                        .await?;
                }
                false => {
                    let error = result.error.as_deref().unwrap_or("Unknown error");
                    if cmd.retry_count < cmd.max_retries {
                        self.publish_status_event(&cmd.id, "queued", Some(error));
                        self.storage
                            .update_command_status(
                                &cmd.id,
                                "queued",
                                Some(error),
                                "system",
                                Some("retry"),
                            )
                            .await?;
                    } else {
                        self.publish_status_event(&cmd.id, "failed", Some(error));
                        self.storage
                            .update_command_status(&cmd.id, "failed", Some(error), "system", None)
                            .await?;
                    }
                }
            }

            // Remove from in-flight
            {
                let mut in_flight = self.in_flight.write().await;
                in_flight.remove(&cmd.device_id);
            }
        }

        Ok(())
    }
}
