use axum::{
    extract::State,
    http::{Request, StatusCode},
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
    request: Request<axum::body::Body>,
) -> impl IntoResponse {
    // Get client IP from X-Forwarded-For header or use a default
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string());

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
