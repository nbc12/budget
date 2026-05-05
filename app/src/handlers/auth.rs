use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use common::{AppState, auth::AUTH_SESSION_KEY};
use std::sync::Arc;
use askama::Template;
use serde::Deserialize;
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub password: String,
}

pub async fn root_redirect() -> Response {
    let now = chrono::Local::now();
    let month = now.format("%Y-%m").to_string();
    Redirect::to(&format!("/budget/{}", month)).into_response()
}

pub async fn login_get(
    State(state): State<Arc<AppState>>,
) -> Response {
    if state.config.app_password.is_none() {
        return Redirect::to("/").into_response();
    }

    let template = LoginTemplate { error: None };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Template Error").into_response(),
    }
}

pub async fn login_post(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(payload): Form<LoginForm>,
) -> Response {
    if let Some(correct_password) = &state.config.app_password {
        if payload.password == *correct_password {
            let _ = session.insert(AUTH_SESSION_KEY, true).await;
            return Redirect::to("/").into_response();
        }
    }

    let template = LoginTemplate { error: Some("Invalid password".into()) };
    match template.render() {
        Ok(html) => (axum::http::StatusCode::UNAUTHORIZED, Html(html)).into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Template Error").into_response(),
    }
}
