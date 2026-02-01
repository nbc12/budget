use axum::{
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    extract::{Request, State},
};
use tower_sessions::Session;
use std::sync::Arc;
use crate::AppState;

pub const AUTH_SESSION_KEY: &str = "authenticated";

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    // If no password is set, authentication is disabled
    if state.config.app_password.is_none() {
        return next.run(request).await;
    }

    let authenticated: bool = session
        .get(AUTH_SESSION_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or(false);

    if authenticated {
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}