use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::auth::session::{SessionData, SessionManager};
use crate::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let session_cookie = req
        .headers()
        .get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|c| {
            c.split(';')
                .find(|c| c.trim().starts_with("session="))
                .map(|c| c.trim().trim_start_matches("session=").to_string())
        });

    let session = match session_cookie {
        Some(cookie) => match state.session_manager.get_session(&cookie) {
            Ok(s) => s,
            Err(_) => {
                return axum::response::Redirect::to("/login").into_response();
            }
        },
        None => {
            return axum::response::Redirect::to("/login").into_response();
        }
    };

    req.extensions_mut().insert(session);
    next.run(req).await
}

pub async fn auth_api_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let session_cookie = req
        .headers()
        .get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|c| {
            c.split(';')
                .find(|c| c.trim().starts_with("session="))
                .map(|c| c.trim().trim_start_matches("session=").to_string())
        });

    let session = match session_cookie {
        Some(cookie) => match state.session_manager.get_session(&cookie) {
            Ok(s) => s,
            Err(_) => {
                return axum::http::StatusCode::UNAUTHORIZED.into_response();
            }
        },
        None => {
            return axum::http::StatusCode::UNAUTHORIZED.into_response();
        }
    };

    req.extensions_mut().insert(session);
    next.run(req).await
}

pub async fn admin_required(req: Request, next: Next) -> Response {
    let session = req.extensions().get::<SessionData>();

    match session {
        Some(s) if s.is_admin => next.run(req).await,
        _ => axum::http::StatusCode::FORBIDDEN.into_response(),
    }
}
