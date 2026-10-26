mod auth;
mod config;
mod crypto;
mod database;
mod handlers;
mod middleware;
mod models;
mod repository;

use std::sync::Arc;

use auth::github::GitHubOAuthClient;
use auth::session::SessionManager;
use database::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<config::Config>,
    pub github_oauth: Arc<GitHubOAuthClient>,
    pub session_manager: Arc<SessionManager>,
}

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Server Access Manager starting");
}
