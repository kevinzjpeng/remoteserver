mod auth;
mod config;
mod crypto;
mod database;
mod handlers;
mod middleware;
mod models;
mod repository;
mod service;
mod session;

use std::sync::Arc;
use std::time::Duration;

use auth::github::GitHubOAuthClient;
use auth::session::SessionManager;
use database::Database;
use session::manager::SessionManager as SSHSessionManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<config::Config>,
    pub github_oauth: Arc<GitHubOAuthClient>,
    pub session_manager: Arc<SessionManager>,
    pub ssh_session_manager: Arc<SSHSessionManager>,
}

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Server Access Manager starting");

    // Load configuration
    let config = match config::load_config() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Configuration loaded successfully");

    // Initialize database
    let db = match database::setup_database(&config.database_path) {
        Ok(d) => Arc::new(d),
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Database initialized at {}", config.database_path);

    // Initialize GitHub OAuth client
    let github_oauth = Arc::new(GitHubOAuthClient::new(
        config.github_client_id.clone(),
        config.github_client_secret.clone(),
        config.github_redirect_uri.clone(),
    ));

    // Initialize session manager
    let session_manager = Arc::new(SessionManager::new(
        config.session_secret.clone(),
        config.session_duration.as_secs() as i64,
    ));

    // Initialize SSH session manager
    let ssh_session_manager = Arc::new(SSHSessionManager::new(Duration::from_secs(300)));

    // Create server repository for health check
    let server_repo = Arc::new(repository::ServerRepository::new(
        db.clone(),
        config.encryption_key.as_bytes().to_vec(),
    ));

    // Initialize health check service
    let health_check = service::health_check::HealthCheckService::new(
        server_repo.clone(),
        config.clone(),
    );

    // Start health check service in background
    tokio::spawn(async move {
        health_check.run().await;
    });

    tracing::info!("Health check service started");

    // Create application state
    let state = AppState {
        db: db.clone(),
        config: config.clone(),
        github_oauth,
        session_manager,
        ssh_session_manager,
    };

    // Build router
    let app = handlers::create_router(state);

    // Start server
    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
