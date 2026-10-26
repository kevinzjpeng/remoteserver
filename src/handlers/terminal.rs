use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::session::SessionData;
use crate::AppState;

#[derive(Deserialize)]
pub struct TerminalParams {
    server_id: i64,
    session: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsMessage {
    #[serde(rename = "resize")]
    Resize { cols: u16, rows: u16 },
    #[serde(rename = "ping")]
    Ping { t: i64 },
    #[serde(rename = "pong")]
    Pong { t: i64 },
}

#[derive(Serialize)]
pub struct SessionListResponse {
    sessions: Vec<SessionInfo>,
}

#[derive(Serialize)]
pub struct SessionInfo {
    id: String,
    name: String,
    created_at: String,
    last_active: String,
    client_count: usize,
    active: bool,
}

pub async fn terminal_ws(
    State(state): State<AppState>,
    Extension(session_data): Extension<SessionData>,
    ws: WebSocketUpgrade,
    Query(params): Query<TerminalParams>,
) -> impl IntoResponse {
    let state = state.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, state, session_data, params))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: AppState,
    session_data: SessionData,
    params: TerminalParams,
) {
    // TODO: Implement full SSH session multiplexing
    // For now, this is a placeholder that echoes input back
    tracing::info!(
        "Terminal WebSocket connection: server_id={}, session={}",
        params.server_id,
        params.session
    );

    // Simple echo implementation for now
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                // Try to parse as control message
                if let Ok(control) = serde_json::from_str::<WsMessage>(&text) {
                    match control {
                        WsMessage::Ping { t } => {
                            let pong = serde_json::json!({"type": "pong", "t": t});
                            let _ = socket.send(Message::Text(pong.to_string())).await;
                        }
                        _ => {}
                    }
                }
            }
            Message::Binary(data) => {
                // Echo back for now
                let _ = socket.send(Message::Binary(data)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(session_data): Extension<SessionData>,
    Query(params): Query<TerminalParams>,
) -> impl IntoResponse {
    let sessions = state.ssh_session_manager.list_sessions(params.server_id);

    let response = SessionListResponse {
        sessions: sessions
            .into_iter()
            .map(|s| SessionInfo {
                id: s.id,
                name: s.name,
                created_at: s.created_at.to_rfc3339(),
                last_active: s.last_active.to_rfc3339(),
                client_count: s.client_count,
                active: s.active,
            })
            .collect(),
    };

    Json(response)
}

pub async fn close_session(
    State(state): State<AppState>,
    Extension(session_data): Extension<SessionData>,
    Query(params): Query<TerminalParams>,
) -> Result<Response, StatusCode> {
    state
        .ssh_session_manager
        .close_session(params.server_id, &params.session)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK.into_response())
}