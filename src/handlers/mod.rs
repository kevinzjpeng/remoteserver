pub mod auth;
pub mod config_handler;
pub mod credentials;
pub mod environments;
pub mod home;
pub mod logs;
pub mod servers;
pub mod terminal;
pub mod topology;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::services::ServeDir;

use crate::AppState;
use crate::middleware;

pub fn create_router(state: AppState) -> Router {
    let static_files = ServeDir::new("static");

    // Public routes
    let public_routes = Router::new()
        .route("/login", get(auth::login))
        .route("/auth/github", get(auth::login_redirect))
        .route("/auth/github/callback", get(auth::callback))
        .route("/logout", post(auth::logout))
        .route("/api/config", get(config_handler::get_connection_config))
        .route("/favicon.ico", get(|| async { axum::response::Redirect::permanent("/static/favicon.ico") }))
        .nest_service("/static", static_files);

    // Protected routes (require auth)
    let protected_routes = Router::new()
        .route("/", get(home::dashboard))
        .route("/topology", get(topology::list))
        .route("/admin/environments", get(environments::list))
        .route("/admin/environments", post(environments::create))
        .route("/admin/environments", put(environments::update))
        .route("/admin/environments", delete(environments::delete))
        .route("/admin/servers", get(servers::list))
        .route("/admin/servers", post(servers::create))
        .route("/admin/servers", put(servers::update))
        .route("/admin/servers", delete(servers::delete))
        .route("/admin/credentials", get(credentials::list))
        .route("/admin/credentials", post(credentials::create))
        .route("/admin/credentials", put(credentials::update))
        .route("/admin/credentials", delete(credentials::delete))
        .route("/admin/logs", get(logs::list))
        .route("/terminal", get(terminal::terminal_page))
        .route("/ws/terminal", get(terminal::terminal_ws))
        .route("/api/sessions", get(terminal::list_sessions))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer(axum_middleware::from_fn(middleware::csrf::csrf_protection));

    public_routes.merge(protected_routes).with_state(state)
}
