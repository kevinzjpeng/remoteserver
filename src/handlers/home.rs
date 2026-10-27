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
    let env_repo = EnvironmentRepository::new(state.db.clone());

    let servers = server_repo.get_all().map_err(|e| {
        tracing::error!("Failed to load servers: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let environments = env_repo.get_all().map_err(|e| {
        tracing::error!("Failed to load environments: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let servers_html = if servers.is_empty() {
        "<p>No servers yet. <a href=\"/admin/servers\">Add your first server</a>.</p>".to_string()
    } else {
        let rows = servers
            .iter()
            .map(|s| {
                let env_name = if let Some(env_id) = s.environment_id {
                    environments
                        .iter()
                        .find(|e| e.id == env_id)
                        .map(|e| e.name.as_str())
                        .unwrap_or("-")
                } else {
                    "-"
                };
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    s.name, s.hostname, env_name, s.os_type
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<table role="grid">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Hostname</th>
                        <th>Environment</th>
                        <th>OS</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>"#,
            rows
        )
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dashboard - Server Access Manager</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css">
    <link rel="stylesheet" href="/static/style.css">
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.min.js" defer></script>
</head>
<body>
    <header class="container">
        <nav>
            <ul>
                <li><strong>🖥️ Server Access Manager</strong></li>
            </ul>
            <ul>
                <li><a href="/">Dashboard</a></li>
                <li><a href="/topology">Topology</a></li>
                <li>
                    <details class="dropdown">
                        <summary>{}</summary>
                        <dir>
                            <ul>
                                <li><a href="/admin/environments">Environments</a></li>
                                <li><a href="/admin/servers">Servers</a></li>
                                <li><a href="/admin/credentials">Credentials</a></li>
                                <li><a href="/admin/logs">Logs</a></li>
                                <li><a href="/logout" onclick="event.preventDefault(); fetch('/logout', {{method: 'POST'}}).then(() => window.location.href='/login')">Logout</a></li>
                            </ul>
                        </dir>
                    </details>
                </li>
            </ul>
        </nav>
    </header>

    <main class="container">
        <h1>Dashboard</h1>

        <div class="grid">
            <article>
                <header>Environments</header>
                <p style="font-size: 2em; font-weight: bold;">{}</p>
                <footer>
                    <a href="/admin/environments" class="secondary">Manage →</a>
                </footer>
            </article>

            <article>
                <header>Servers</header>
                <p style="font-size: 2em; font-weight: bold;">{}</p>
                <footer>
                    <a href="/admin/servers" class="secondary">Manage →</a>
                </footer>
            </article>

            <article>
                <header>Quick Actions</header>
                <ul>
                    <li><a href="/admin/servers">Add Server</a></li>
                    <li><a href="/admin/environments">Add Environment</a></li>
                    <li><a href="/topology">View Topology</a></li>
                </ul>
            </article>
        </div>

        <h2>Recent Servers</h2>
        {}
    </main>

    <footer class="container">
        <small>Server Access Manager | Built with Rust</small>
    </footer>

    <script src="/static/js/app.js"></script>
</body>
</html>"#,
        session.username,
        environments.len(),
        servers.len(),
        servers_html
    );

    Ok(Html(html))
}
