use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension,
};
use serde::Deserialize;

use crate::auth::session::SessionData;
use crate::repository::AccessLogRepository;
use crate::AppState;

#[derive(Deserialize)]
pub struct LogParams {
    limit: Option<i64>,
}

pub async fn list(
    state: State<AppState>,
    Extension(session): Extension<SessionData>,
    Query(params): Query<LogParams>,
) -> Result<Html<String>, StatusCode> {
    let log_repo = AccessLogRepository::new(state.db.clone());

    let limit = params.limit.unwrap_or(100);

    let logs = log_repo.get_all(limit).map_err(|e| {
        tracing::error!("Failed to load logs: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Access Logs - Server Access Manager</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <header>
        <h1>Server Access Manager</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/admin/logs">Logs</a>
            <span>Welcome, {}</span>
            <form action="/logout" method="post" style="display:inline">
                <button type="submit">Logout</button>
            </form>
        </nav>
    </header>
    <main>
        <h2>Access Logs (showing {} most recent)</h2>
        <table>
            <thead>
                <tr>
                    <th>Time</th>
                    <th>Server ID</th>
                    <th>User ID</th>
                    <th>Type</th>
                    <th>Success</th>
                    <th>Notes</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
    </main>
</body>
</html>"#,
        session.username,
        logs.len(),
        logs.iter()
            .map(|log| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    log.accessed_at,
                    log.server_id,
                    log.user_id,
                    log.connection_type,
                    if log.success { "✓" } else { "✗" },
                    log.notes.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(Html(html))
}
