use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form, Json,
};
use serde::{Deserialize, Serialize};

use crate::auth::session::SessionData;
use crate::repository::CredentialRepository;
use crate::AppState;

#[derive(Deserialize)]
pub struct CredentialForm {
    name: String,
    service_type: String,
    config_json: Option<String>,
    is_active: Option<String>,
    credentials: Option<String>,
}

#[derive(Deserialize)]
pub struct IdParam {
    id: String,
}

pub async fn list(
    state: State<AppState>,
    Extension(session): Extension<SessionData>,
) -> Result<Html<String>, StatusCode> {
    let cred_repo = CredentialRepository::new(state.db.clone(), state.config.encryption_key.as_bytes().to_vec());

    let credentials = cred_repo.get_all().map_err(|e| {
        tracing::error!("Failed to load credentials: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Credentials - Server Access Manager</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <header>
        <h1>Server Access Manager</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/admin/servers">Servers</a>
            <a href="/admin/environments">Environments</a>
            <a href="/admin/credentials">Credentials</a>
            <span>Welcome, {}</span>
            <form action="/logout" method="post" style="display:inline">
                <button type="submit">Logout</button>
            </form>
        </nav>
    </header>
    <main>
        <h2>Credentials ({} total)</h2>
        <ul>
        {}
        </ul>
    </main>
</body>
</html>"#,
        session.username,
        credentials.len(),
        credentials
            .iter()
            .map(|c| format!("<li>{} ({})</li>", c.name, c.service_type))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(Html(html))
}

pub async fn create(
    state: State<AppState>,
    Form(form): Form<CredentialForm>,
) -> Result<Response, StatusCode> {
    let cred_repo = CredentialRepository::new(state.db.clone(), state.config.encryption_key.as_bytes().to_vec());

    let is_active = form.is_active.as_deref() == Some("on") || form.is_active.as_deref() == Some("true");

    let credential = crate::models::Credential {
        id: 0,
        name: form.name,
        service_type: form.service_type,
        credentials_encrypted: None,
        config_json: form.config_json,
        is_active,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let credentials_data = form
        .credentials
        .as_ref()
        .map(|c| c.as_bytes().to_vec())
        .unwrap_or_default();

    cred_repo
        .create(&credential, &credentials_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/admin/credentials").into_response())
}

pub async fn update(
    state: State<AppState>,
    Query(params): Query<IdParam>,
    Form(form): Form<CredentialForm>,
) -> Result<Response, StatusCode> {
    let id = params.id.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;

    let cred_repo = CredentialRepository::new(state.db.clone(), state.config.encryption_key.as_bytes().to_vec());

    let is_active = form.is_active.as_deref() == Some("on") || form.is_active.as_deref() == Some("true");

    let credential = crate::models::Credential {
        id,
        name: form.name,
        service_type: form.service_type,
        credentials_encrypted: None,
        config_json: form.config_json,
        is_active,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let credentials_data = form
        .credentials
        .as_ref()
        .map(|c| c.as_bytes().to_vec())
        .unwrap_or_default();

    cred_repo
        .update(&credential, &credentials_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/admin/credentials").into_response())
}

pub async fn delete(
    state: State<AppState>,
    Query(params): Query<IdParam>,
) -> Result<Response, StatusCode> {
    let id = params.id.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;

    let cred_repo = CredentialRepository::new(state.db.clone(), state.config.encryption_key.as_bytes().to_vec());

    cred_repo.delete(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK.into_response())
}
