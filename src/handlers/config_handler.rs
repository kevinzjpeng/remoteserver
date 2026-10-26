use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::config::is_internal_client;
use crate::AppState;

#[derive(Serialize)]
pub struct ConnectionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_ip: Option<String>,
    pub is_internal: bool,
}

pub async fn get_connection_config(
    state: State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> impl IntoResponse {
    let client_ip = addr.ip().to_string();
    let is_internal = is_internal_client(&client_ip);

    let resp = ConnectionConfig {
        is_internal,
        internal_ip: if is_internal && !state.config.internal_ip.is_empty() {
            Some(state.config.internal_ip.clone())
        } else {
            None
        },
    };

    Json(resp)
}
