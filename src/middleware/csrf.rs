use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn csrf_protection(req: Request, next: Next) -> Response {
    let method = req.method().clone();

    if method == axum::http::Method::GET || method == axum::http::Method::HEAD {
        return next.run(req).await;
    }

    let has_token = req.headers().get("X-CSRF-Token").is_some();

    if !has_token {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(req).await
}
