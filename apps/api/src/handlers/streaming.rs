//! Real-time packet streaming endpoints

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

/// Stream configuration
#[derive(Debug, Deserialize, ToSchema)]
pub struct StreamConfig {
    /// Filter by channel
    pub channel: Option<u8>,
    /// Filter by packet type
    pub packet_type: Option<String>,
    /// Max packets per second
    pub rate_limit: Option<u32>,
}

/// WebSocket packet streaming endpoint
#[utoipa::path(
    get,
    path = "/api/v1/stream",
    tag = "streaming",
    responses(
        (status = 101, description = "WebSocket upgrade"),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn stream_packets(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Send initial message
    if socket
        .send(axum::extract::ws::Message::Text(
            serde_json::to_string(&serde_json::json!({
                "type": "connected",
                "message": "Packet stream connected"
            }))
            .unwrap(),
        ))
        .await
        .is_err()
    {
        return;
    }

    // Main streaming loop
    loop {
        tokio::select! {
            // Receive messages from client
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(axum::extract::ws::Message::Text(text)) => {
                        tracing::debug!("Received: {}", text);

                        // Parse command
                        if text == "ping" {
                            if socket
                                .send(axum::extract::ws::Message::Text("pong".to_string()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                    Ok(axum::extract::ws::Message::Close(_)) => {
                        tracing::info!("Client disconnected");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            // Send packets from buffer (if available)
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                let buffer = state.streaming_buffer.read().await;
                if let Some(ref buf) = *buffer {
                    // Get recent packets
                    if let Ok(packets) = buf.get_recent_packets(10) {
                        if !packets.is_empty() {
                            let json = serde_json::to_string(&serde_json::json!({
                                "type": "packets",
                                "data": packets
                            }))
                            .unwrap();

                            if socket
                                .send(axum::extract::ws::Message::Text(json))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    tracing::info!("WebSocket connection closed");
}
