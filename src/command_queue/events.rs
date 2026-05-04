use super::*;
use crate::ws_types::ServerMessage;

impl CommandQueue {
    pub(super) fn publish_status_event(&self, command_id: &str, status: &str, error: Option<&str>) {
        let payload = serde_json::json!({
            "type": "command_status",
            "command_id": command_id,
            "status": status,
            "error": error,
        })
        .to_string();
        self.cache.publish_sse_payload(payload);

        // Broadcast via WebSocket
        if let Some(ref ws) = self.ws_manager {
            ws.broadcast_command_ack(ServerMessage::CommandAck {
                command_id: command_id.to_string(),
                status: status.to_string(),
                device_response: error.map(|e| e.to_string()),
                latency_ms: None,
            });
        }
    }

    pub(super) fn publish_ack_event(&self, command_id: &str, response: &str, latency_ms: u64) {
        let payload = serde_json::json!({
            "type": "command_ack",
            "command_id": command_id,
            "status": "acked",
            "device_response": response,
            "latency_ms": latency_ms,
        })
        .to_string();
        self.cache.publish_sse_payload(payload);

        // Broadcast via WebSocket
        if let Some(ref ws) = self.ws_manager {
            ws.broadcast_command_ack(ServerMessage::CommandAck {
                command_id: command_id.to_string(),
                status: "acked".to_string(),
                device_response: Some(response.to_string()),
                latency_ms: Some(latency_ms),
            });
        }
    }
}
