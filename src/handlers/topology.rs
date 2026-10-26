use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension,
};

use crate::auth::session::SessionData;
use crate::repository::{EnvironmentRepository, ServerRepository};
use crate::AppState;

pub async fn list(
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

    // Group servers by environment
    let mut env_servers: std::collections::HashMap<String, Vec<&crate::models::Server>> =
        std::collections::HashMap::new();

    for server in &servers {
        let env_name = if let Some(env_id) = server.environment_id {
            environments
                .iter()
                .find(|e| e.id == env_id)
                .map(|e| e.name.as_str())
                .unwrap_or("Unknown")
        } else {
            "No Environment"
        };

        env_servers
            .entry(env_name.to_string())
            .or_insert_with(Vec::new)
            .push(server);
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Topology - Server Access Manager</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <header>
        <h1>Server Access Manager</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/topology">Topology</a>
            <span>Welcome, {}</span>
            <form action="/logout" method="post" style="display:inline">
                <button type="submit">Logout</button>
            </form>
        </nav>
    </header>
    <main>
        <h2>Server Topology</h2>
        {}
    </main>
</body>
</html>"#,
        session.username,
        env_servers
            .iter()
            .map(|(env_name, servers)| {
                format!(
                    "<h3>{}</h3><ul>{}</ul>",
                    env_name,
                    servers
                        .iter()
                        .map(|s| format!("<li>{} ({}) - {}</li>", s.name, s.hostname, s.status))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(Html(html))
}
