use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form, Json,
};
use serde::{Deserialize, Serialize};

use crate::auth::session::SessionData;
use crate::repository::{EnvironmentRepository, ServerRepository};
use crate::AppState;

#[derive(Deserialize)]
pub struct ServerForm {
    name: String,
    hostname: String,
    port: Option<String>,
    username: String,
    environment_id: Option<String>,
    auth_type: String,
    password: Option<String>,
    private_key: Option<String>,
    passphrase: Option<String>,
    description: Option<String>,
    os_type: Option<String>,
    tags: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
pub struct IdParam {
    id: String,
}

pub async fn list(
    state: State<AppState>,
    Extension(session): Extension<SessionData>,
) -> Result<Html<String>, StatusCode> {
    let server_repo = ServerRepository::new(&state.db, state.config.encryption_key.as_bytes().to_vec());
    let env_repo = EnvironmentRepository::new(&state.db);

    let servers = server_repo.get_all().map_err(|e| {
        tracing::error!("Failed to load servers: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let environments = env_repo.get_all().map_err(|e| {
        tracing::error!("Failed to load environments: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Servers - Server Access Manager</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <header>
        <h1>Server Access Manager</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/admin/servers">Servers</a>
            <a href="/admin/environments">Environments</a>
            <span>Welcome, {}</span>
            <form action="/logout" method="post" style="display:inline">
                <button type="submit">Logout</button>
            </form>
        </nav>
    </header>
    <main>
        <h2>Servers ({} total)</h2>
        <ul>
        {}
        </ul>
    </main>
</body>
</html>"#,
        session.username,
        servers.len(),
        servers
            .iter()
            .map(|s| format!("<li>{} ({})</li>", s.name, s.hostname))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(Html(html))
}

pub async fn create(
    state: State<AppState>,
    Form(form): Form<ServerForm>,
) -> Result<Response, StatusCode> {
    let server_repo = ServerRepository::new(&state.db, state.config.encryption_key.as_bytes().to_vec());

    let port = form
        .port
        .as_ref()
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(22);

    let environment_id = form
        .environment_id
        .as_ref()
        .and_then(|e| e.parse::<i64>().ok());

    let tags = form.tags.as_ref().map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
            .join(",")
    });

    let server = crate::models::Server {
        id: 0,
        name: form.name,
        hostname: form.hostname,
        port,
        username: form.username,
        environment_id,
        auth_type: form.auth_type,
        password_encrypted: None,
        private_key_encrypted: None,
        passphrase_encrypted: None,
        description: form.description,
        os_type: form.os_type.unwrap_or_else(|| "linux".to_string()),
        tags,
        notes: form.notes,
        status: "unknown".to_string(),
        is_intranet: false,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        last_accessed: None,
    };

    let password = form.password.unwrap_or_default();
    let private_key = form.private_key.unwrap_or_default();
    let passphrase = form.passphrase.unwrap_or_default();

    server_repo
        .create(&server, &password, &private_key, &passphrase)
        .map_err(|e| {
            tracing::error!("Failed to create server: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Redirect::to("/admin/servers").into_response())
}

pub async fn update(
    state: State<AppState>,
    Query(params): Query<IdParam>,
    Form(form): Form<ServerForm>,
) -> Result<Response, StatusCode> {
    let id = params.id.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;

    let server_repo = ServerRepository::new(&state.db, state.config.encryption_key.as_bytes().to_vec());

    // Get existing server to preserve credentials if not updated
    let existing = server_repo.get_by_id(id).map_err(|_| StatusCode::NOT_FOUND)?;

    let port = form
        .port
        .as_ref()
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(existing.port);

    let environment_id = form
        .environment_id
        .as_ref()
        .and_then(|e| e.parse::<i64>().ok());

    let tags = form.tags.as_ref().map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
            .join(",")
    });

    let server = crate::models::Server {
        id,
        name: form.name,
        hostname: form.hostname,
        port,
        username: form.username,
        environment_id,
        auth_type: form.auth_type,
        password_encrypted: None,
        private_key_encrypted: None,
        passphrase_encrypted: None,
        description: form.description,
        os_type: form.os_type.unwrap_or_else(|| "linux".to_string()),
        tags,
        notes: form.notes,
        status: existing.status,
        is_intranet: existing.is_intranet,
        created_at: existing.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
        last_accessed: existing.last_accessed,
    };

    // Only update credentials if provided
    let password = if let Some(p) = form.password {
        if !p.is_empty() {
            p
        } else {
            server_repo.get_password(id).unwrap_or_default()
        }
    } else {
        server_repo.get_password(id).unwrap_or_default()
    };

    let private_key = form.private_key.unwrap_or_default();
    let passphrase = form.passphrase.unwrap_or_default();

    server_repo
        .update(&server, &password, &private_key, &passphrase)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/admin/servers").into_response())
}

pub async fn delete(
    state: State<AppState>,
    Query(params): Query<IdParam>,
) -> Result<Response, StatusCode> {
    let id = params.id.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;

    let server_repo = ServerRepository::new(&state.db, state.config.encryption_key.as_bytes().to_vec());

    server_repo.delete(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK.into_response())
}
