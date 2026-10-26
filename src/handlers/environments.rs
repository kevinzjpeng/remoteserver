use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use serde::Deserialize;

use crate::auth::session::SessionData;
use crate::repository::EnvironmentRepository;
use crate::AppState;

#[derive(Deserialize)]
pub struct EnvironmentForm {
    name: String,
    description: Option<String>,
    color: Option<String>,
    display_order: Option<String>,
}

#[derive(Deserialize)]
pub struct IdParam {
    id: String,
}

pub async fn list(
    state: State<AppState>,
    Extension(session): Extension<SessionData>,
) -> Result<Html<String>, StatusCode> {
    let env_repo = EnvironmentRepository::new(state.db.clone());

    let environments = env_repo.get_all().map_err(|e| {
        tracing::error!("Failed to load environments: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Environments - Server Access Manager</title>
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
        <h2>Environments ({} total)</h2>
        <ul>
        {}
        </ul>
    </main>
</body>
</html>"#,
        session.username,
        environments.len(),
        environments
            .iter()
            .map(|e| format!("<li>{} ({})</li>", e.name, e.color))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(Html(html))
}

pub async fn create(
    state: State<AppState>,
    Form(form): Form<EnvironmentForm>,
) -> Result<Response, StatusCode> {
    let env_repo = EnvironmentRepository::new(state.db.clone());

    let display_order = form
        .display_order
        .as_ref()
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(0);

    env_repo
        .create(
            &form.name,
            form.description.as_deref().unwrap_or(""),
            form.color.as_deref().unwrap_or("#6b7280"),
            display_order,
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/admin/environments").into_response())
}

pub async fn update(
    state: State<AppState>,
    Query(params): Query<IdParam>,
    Form(form): Form<EnvironmentForm>,
) -> Result<Response, StatusCode> {
    let id = params.id.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;

    let env_repo = EnvironmentRepository::new(state.db.clone());

    let display_order = form
        .display_order
        .as_ref()
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(0);

    env_repo
        .update(
            id,
            &form.name,
            form.description.as_deref().unwrap_or(""),
            form.color.as_deref().unwrap_or("#6b7280"),
            display_order,
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/admin/environments").into_response())
}

pub async fn delete(
    state: State<AppState>,
    Query(params): Query<IdParam>,
) -> Result<Response, StatusCode> {
    let id = params.id.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;

    let env_repo = EnvironmentRepository::new(state.db.clone());

    env_repo.delete(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK.into_response())
}
