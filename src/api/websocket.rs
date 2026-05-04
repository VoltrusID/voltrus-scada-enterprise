use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use super::AppState;
use crate::ws_types::{ClientMessage, ServerMessage};

/// GET /api/v1/ws -- WebSocket upgrade endpoint.
///
/// The client must send an `Auth` message (MessagePack-encoded `ClientMessage::Auth`)
/// within 30 seconds of connecting. After successful authentication the connection
/// is registered with `WsManager` and the client can subscribe to tags and issue
/// tag writes.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// -- Socket handler --------------------------------------------------------

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = socket.split();

    // 1. Wait for the first message (auth) with a 30-second timeout.
    let first_msg: Message = match timeout(Duration::from_secs(30), stream.next()).await {
        Ok(Some(Ok(msg))) => msg,
        Ok(Some(Err(e))) => {
            tracing::warn!("WS: error receiving first message: {}", e);
            return;
        }
        Ok(None) | Err(_) => {
            tracing::info!("WS: client disconnected before authentication");
            return;
        }
    };

    // 2. Decode the first message as a ClientMessage::Auth.
    let data = match first_msg.into_data() {
        d if !d.is_empty() => d,
        _ => {
            let _ = send_msg(&mut sink, &ServerMessage::AuthError {
                message: "First message must be a binary Auth message".to_string(),
            })
            .await;
            return;
        }
    };

    let client_msg: ClientMessage = match rmp_serde::from_slice(&data) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("WS: failed to decode first message: {}", e);
            let _ = send_msg(&mut sink, &ServerMessage::AuthError {
                message: format!("Invalid MessagePack: {}", e),
            })
            .await;
            return;
        }
    };

    let token = match client_msg {
        ClientMessage::Auth { token } => token,
        _ => {
            let _ = send_msg(&mut sink, &ServerMessage::AuthError {
                message: "First message must be Auth".to_string(),
            })
            .await;
            return;
        }
    };

    // 3. Validate the JWT token.
    let claims =
        match crate::api::auth::verify_token(&token, &state.auth_secret) {
            Some(c) => c,
            None => {
                let _ = send_msg(&mut sink, &ServerMessage::AuthError {
                    message: "Invalid or expired token".to_string(),
                })
                .await;
                return;
            }
        };

    let username = claims.username.clone();
    let role = claims.role.clone();

    // 4. Determine permissions from role.
    let permissions = role_to_permissions(&role);

    // 5. Generate a unique connection ID.
    let conn_id = format!(
        "ws-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    tracing::info!("WS: user={} role={} conn={}", username, role, conn_id);

    // 6. Register with WsManager and get the receiver for outgoing messages.
    let mut rx = state
        .ws_manager
        .connect(conn_id.clone(), username.clone(), permissions.clone());

    // 7. Send AuthOk.
    if let Err(e) = send_msg(
        &mut sink,
        &ServerMessage::AuthOk {
            user: username.clone(),
            permissions: permissions.clone(),
        },
    )
    .await
    {
        tracing::warn!("WS: failed to send AuthOk: {}", e);
        state.ws_manager.disconnect(&conn_id);
        return;
    }

    // 8. Main loop: select between outgoing WsManager messages and
    //    incoming WebSocket frames from the client.
    let conn_id_loop = conn_id.clone();
    let ws_manager_loop = state.ws_manager.clone();
    let username_loop = username.clone();
    let role_loop = role.clone();
    let state_loop = state.clone();

    loop {
        tokio::select! {
            // Outgoing message from WsManager broadcast channel
            msg = rx.recv() => {
                match msg {
                    Some(server_msg) => {
                        if send_msg(&mut sink, &server_msg).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        // Channel closed -- manager dropped
                        break;
                    }
                }
            }
            // Incoming WebSocket frame from client
            frame = stream.next() => {
                match frame {
                    Some(Ok(Message::Binary(data))) => {
                        let client_msg = match rmp_serde::from_slice::<ClientMessage>(&data) {
                            Ok(m) => m,
                            Err(e) => {
                                tracing::warn!("WS: decode error: {}", e);
                                continue;
                            }
                        };
                        handle_client_message(
                            &client_msg,
                            &conn_id_loop,
                            &ws_manager_loop,
                            &state_loop,
                            &username_loop,
                            &role_loop,
                        ).await;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sink.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(Message::Text(_))) => {
                        tracing::debug!(
                            "WS: ignoring text frame (expected binary MessagePack)"
                        );
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Pong responses are handled automatically by tungstenite
                    }
                    Some(Err(e)) => {
                        tracing::warn!("WS: recv error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    ws_manager_loop.disconnect(&conn_id_loop);
    tracing::info!("WS: disconnected conn={}", conn_id);
}

// -- Client message dispatch -----------------------------------------------

async fn handle_client_message(
    msg: &ClientMessage,
    conn_id: &str,
    ws_manager: &Arc<crate::ws_manager::WsManager>,
    state: &AppState,
    username: &str,
    role: &str,
) {
    match msg {
        ClientMessage::Subscribe { tag_ids } => {
            ws_manager.subscribe(conn_id, tag_ids);
            tracing::debug!(
                "WS: user={} subscribed to {} tags",
                username,
                tag_ids.len()
            );
        }

        ClientMessage::Unsubscribe { tag_ids } => {
            ws_manager.unsubscribe(conn_id, tag_ids);
            tracing::debug!(
                "WS: user={} unsubscribed from {} tags",
                username,
                tag_ids.len()
            );
        }

        ClientMessage::TagWrite {
            tag_id,
            value,
            command_id,
        } => {
            // Permission check: only admins can write via WebSocket
            if role != "admin" {
                tracing::warn!(
                    "WS: user={} (role={}) denied tag write to {}",
                    username,
                    role,
                    tag_id
                );
                return;
            }

            handle_tag_write(state, username, tag_id, *value, command_id).await;
        }

        ClientMessage::Ping => {
            // Application-level ping -- WS protocol ping/pong handled separately.
        }

        ClientMessage::Auth { .. } => {
            tracing::debug!("WS: ignoring duplicate Auth message");
        }
    }
}

// -- Tag write handling (stub -- full integration in Task 5) ----------------

async fn handle_tag_write(
    state: &AppState,
    username: &str,
    tag_id: &str,
    value: f64,
    command_id: &Option<String>,
) {
    tracing::info!(
        "WS: user={} tag write {} = {} (cmd_id={:?})",
        username,
        tag_id,
        value,
        command_id
    );

    // Try to parse tag_id as an integer (data_source_id). If not, skip.
    let data_source_id: i64 = match tag_id.parse() {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!(
                "WS: tag_id '{}' is not a numeric data_source_id, skipping write",
                tag_id
            );
            return;
        }
    };

    // Look up write policy and device info
    let (writable, device_id, protocol) =
        match state.storage.get_data_source_device_id(data_source_id).await {
            Ok(Some((dev_id, proto))) => {
                let policy_ok = state
                    .storage
                    .get_write_policy(data_source_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|p| p.writable)
                    .unwrap_or(false);
                (policy_ok, dev_id, proto)
            }
            _ => {
                tracing::warn!(
                    "WS: data_source {} not found for tag write",
                    data_source_id
                );
                return;
            }
        };

    if !writable {
        tracing::warn!("WS: data_source {} is not writable", data_source_id);
        return;
    }

    // Generate command ID
    let cmd_id = command_id
        .clone()
        .unwrap_or_else(|| {
            use rand::Rng;
            rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(21)
                .map(char::from)
                .collect::<String>()
                .to_lowercase()
        });

    let now = crate::timeutil::now_secs();
    let cmd = crate::models::Command {
        id: cmd_id.clone(),
        data_source_id,
        device_id,
        protocol,
        value: value.to_string(),
        status: crate::models::CommandStatus::Pending.as_str().to_string(),
        policy: "immediate".to_string(),
        scheduled_at: None,
        retry_count: 0,
        max_retries: 3,
        error_message: None,
        created_at: now,
        sent_at: None,
        acked_at: None,
        resolved_at: None,
        created_by: username.to_string(),
    };

    if let Err(e) = state.storage.insert_command(&cmd).await {
        tracing::warn!("WS: failed to queue command: {}", e);
    } else {
        tracing::info!("WS: queued command {}", cmd_id);
    }
}

// -- Helpers ----------------------------------------------------------------

/// Map a user role to a list of permission strings.
fn role_to_permissions(role: &str) -> Vec<String> {
    match role {
        "admin" => vec![
            "read".to_string(),
            "write".to_string(),
            "admin".to_string(),
        ],
        "api" => vec!["read".to_string(), "write".to_string()],
        _ => vec!["read".to_string()],
    }
}

/// Encode a `ServerMessage` as MessagePack and send it as a binary WS frame.
async fn send_msg(
    sink: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    msg: &ServerMessage,
) -> Result<(), axum::Error> {
    let bytes = rmp_serde::to_vec(msg).map_err(|e| {
        axum::Error::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e,
        ))
    })?;
    sink.send(Message::Binary(bytes.into())).await
}
