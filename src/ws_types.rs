use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Auth { token: String },
    Subscribe { tag_ids: Vec<String> },
    Unsubscribe { tag_ids: Vec<String> },
    TagWrite {
        tag_id: String,
        value: f64,
        command_id: Option<String>,
    },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    AuthOk {
        user: String,
        permissions: Vec<String>,
    },
    AuthError {
        message: String,
    },
    TagUpdate {
        tag_id: String,
        value: f64,
        quality: String,
        ts: i64,
        simulated: bool,
    },
    CommandAck {
        command_id: String,
        status: String,
        device_response: Option<String>,
        latency_ms: Option<u64>,
    },
    DeviceStatus {
        device_id: String,
        online: bool,
        latency_ms: Option<u64>,
    },
    Alarm {
        alarm_id: i64,
        tag_id: String,
        severity: String,
        message: String,
        value: f64,
        ts: i64,
    },
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_roundtrip() {
        let msg = ClientMessage::Subscribe {
            tag_ids: vec!["PLC01.voltage".to_string(), "PLC01.current".to_string()],
        };
        let bytes = rmp_serde::to_vec(&msg).unwrap();
        let decoded: ClientMessage = rmp_serde::from_slice(&bytes).unwrap();
        match decoded {
            ClientMessage::Subscribe { tag_ids } => {
                assert_eq!(tag_ids.len(), 2);
                assert_eq!(tag_ids[0], "PLC01.voltage");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_roundtrip() {
        let msg = ServerMessage::TagUpdate {
            tag_id: "PLC01.voltage".to_string(),
            value: 230.5,
            quality: "good".to_string(),
            ts: 1700000000,
            simulated: false,
        };
        let bytes = rmp_serde::to_vec(&msg).unwrap();
        let decoded: ServerMessage = rmp_serde::from_slice(&bytes).unwrap();
        match decoded {
            ServerMessage::TagUpdate { tag_id, value, .. } => {
                assert_eq!(tag_id, "PLC01.voltage");
                assert!((value - 230.5).abs() < 0.001);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_all_client_message_types() {
        let messages = vec![
            ClientMessage::Auth { token: "test-jwt".to_string() },
            ClientMessage::Subscribe { tag_ids: vec![] },
            ClientMessage::Unsubscribe { tag_ids: vec![] },
            ClientMessage::TagWrite { tag_id: "t1".to_string(), value: 42.0, command_id: None },
            ClientMessage::Ping,
        ];
        for msg in &messages {
            let bytes = rmp_serde::to_vec(msg).unwrap();
            let _: ClientMessage = rmp_serde::from_slice(&bytes).unwrap();
        }
    }

    #[test]
    fn test_all_server_message_types() {
        let messages = vec![
            ServerMessage::AuthOk { user: "admin".to_string(), permissions: vec![] },
            ServerMessage::AuthError { message: "bad token".to_string() },
            ServerMessage::TagUpdate { tag_id: "t".to_string(), value: 1.0, quality: "good".to_string(), ts: 0, simulated: false },
            ServerMessage::CommandAck { command_id: "c1".to_string(), status: "acked".to_string(), device_response: None, latency_ms: Some(50) },
            ServerMessage::DeviceStatus { device_id: "d1".to_string(), online: true, latency_ms: Some(10) },
            ServerMessage::Alarm { alarm_id: 1, tag_id: "t".to_string(), severity: "warning".to_string(), message: "high".to_string(), value: 99.0, ts: 0 },
            ServerMessage::Pong,
        ];
        for msg in &messages {
            let bytes = rmp_serde::to_vec(msg).unwrap();
            let _: ServerMessage = rmp_serde::from_slice(&bytes).unwrap();
        }
    }

    #[test]
    fn test_messagepack_size_efficiency() {
        let msg = ServerMessage::TagUpdate {
            tag_id: "PLC01.voltage".to_string(),
            value: 230.5,
            quality: "good".to_string(),
            ts: 1700000000,
            simulated: false,
        };
        let msgpack_bytes = rmp_serde::to_vec(&msg).unwrap();
        let json_bytes = serde_json::to_vec(&msg).unwrap();
        assert!(msgpack_bytes.len() < json_bytes.len(), "MessagePack should be smaller than JSON");
    }
}