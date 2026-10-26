use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension,
};

use crate::auth::session::SessionData;
use crate::repository::{EnvironmentRepository, ServerRepository};
use crate::AppState;

pub async fn dashboard(
    state: State<AppState>,
    Extension(session): Extension<SessionData>,
) -> Result<Html<String>, StatusCode> {
    let server_repo = ServerRepository::new(state.db.clone(), state.config.encryption_key.as_bytes().to_vec());
    let env_repo = EnvironmentRepository::new(&state.db);

    let servers = server_repo.get_all().map_err(|e| {
        tracing::error!("Failed to load servers: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let environments = env_repo.get_all().map_err(|e| {
        tracing::error!("Failed to load environments: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // For now, serve a simple HTML page
    // In production, you'd use a template engine like askama
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Dashboard - Server Access Manager</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <header>
        <h1>Server Access Manager</h1>
        <nav>
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
