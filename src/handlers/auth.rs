use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use rand::RngCore;
use serde::Deserialize;
use tracing::error;

use crate::auth::github::GitHubOAuthClient;
use crate::auth::session::SessionManager;
use crate::repository::UserRepository;
use crate::AppState;

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

pub async fn login() -> impl IntoResponse {
    // Serve login page from static files
    axum::response::Html(include_str!("../../static/login.html"))
}

pub async fn login_redirect(state: State<AppState>) -> Response {
    let mut rng = rand::thread_rng();
    let mut state_bytes = [0u8; 32];
    rng.fill_bytes(&mut state_bytes);
    let state_str = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &state_bytes,
    );

    let auth_url = state.github_oauth.get_auth_url(&state_str);

    // Store state in cookie
    let cookie = format!(
        "oauth_state={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=600",
        state_str
    );

    (
        [(axum::http::header::SET_COOKIE, cookie)],
        Redirect::temporary(&auth_url),
    )
        .into_response()
}

pub async fn callback(
    state: State<AppState>,
    Query(params): Query<CallbackParams>,
) -> Result<Response, StatusCode> {
    tracing::info!("OAuth callback received with code: {}", params.code);

    // Exchange code for access token
    let access_token = state
        .github_oauth
        .exchange_code(&params.code)
        .await
        .map_err(|e| {
            error!("Failed to exchange code: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Successfully exchanged code for access token");

    // Get user info from GitHub
    let github_user = state.github_oauth.get_user(&access_token).await.map_err(|e| {
        error!("Failed to get GitHub user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Got GitHub user: {}", github_user.login);

    let user_repo = UserRepository::new(state.db.clone());

    // Create or get user
    let user = match user_repo.get_by_github_id(&github_user.id.to_string()) {
        Ok(u) => u,
        Err(_) => {
            // User doesn't exist, create
            user_repo
                .create_by_github(
                    &github_user.id.to_string(),
                    &github_user.login,
                    github_user.email.as_deref().unwrap_or(""),
                    &github_user.avatar_url,
                )
                .map_err(|e| {
                    error!("Failed to create user: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
        }
    };

    // Update last login
    if let Err(e) = user_repo.update_last_login(user.id) {
        error!("Failed to update last login: {}", e);
    }

    // Create session
    let session_cookie = state.session_manager.create_session(
        user.id,
        user.username.clone(),
        user.is_admin,
    );

    tracing::info!(
        "Session created successfully for user {} (ID: {}, Admin: {})",
        user.username,
        user.id,
        user.is_admin
    );

    // Set session cookie and clear state cookie
    // Secure flag is required for SameSite=None
    let session_header = format!(
        "session={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
        session_cookie,
        state.config.session_duration.as_secs()
    );

    tracing::info!("Setting session cookie: {}", session_header);

    let clear_state = "oauth_state=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";

    let response = Redirect::to("/");
    let mut response = response.into_response();

    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        session_header.parse().unwrap(),
    );
    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        clear_state.parse().unwrap(),
    );

    tracing::info!("Redirecting to / with session cookie set");

    Ok(response)
}

pub async fn logout() -> impl IntoResponse {
    let response = Redirect::to("/login");
    let mut response = response.into_response();

    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        "session=; Path=/; HttpOnly; Max-Age=0"
            .parse()
            .unwrap(),
    );

    response
}
