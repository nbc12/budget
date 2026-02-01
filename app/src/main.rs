use axum::{
    routing::{get},
    Router, 
    response::{Redirect, IntoResponse, Html, Response},
    extract::{State},
    Form,
    middleware::{self},
};
use clap::Parser;
use common::{AppState, Config, auth::{AUTH_SESSION_KEY, auth_middleware}};
use database::Database;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use rust_embed::RustEmbed;
use axum_embed::ServeEmbed;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};
use askama::Template;
use serde::Deserialize;

#[derive(RustEmbed, Clone)]
#[folder = "public/"]
struct Assets;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    error: Option<String>,
}

#[derive(Deserialize)]
struct LoginForm {
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Load Config from CLI args
    let config = Config::parse();

    // 3. Initialize Database
    let db = Database::new(&config.database_url).await?;
    db.run_migrations().await?;

    let state = Arc::new(AppState {
        db,
        config: config.clone(),
    });

    // 4. Session Store
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false); // Set to true in production with HTTPS

    // 5. Routing
    let serve_assets = ServeEmbed::<Assets>::new();
    
    // Diagnostic: Print embedded files
    for file in Assets::iter() {
        tracing::info!("Embedded file: {}", file);
    }

    // Protected Routes
    // Ensure this router has the correct State type from the start
    let protected_routes = Router::<Arc<AppState>>::new()
        .route("/", get(root_redirect))
        .nest("/budget", transactions::handler::transactions_router(state.clone()))
        .nest("/categories", categories::handler::categories_router(state.clone()))
        .nest("/cards", cards::handler::cards_router(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Combined Application Router
    let app = Router::<Arc<AppState>>::new()
        .route("/login", get(login_get).post(login_post))
        .nest_service("/public", serve_assets)
        .merge(protected_routes)
        .with_state(state)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http());

    // 6. Start Server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    if config.app_password.is_none() {
        tracing::warn!("APP_PASSWORD is not set! Authentication is DISABLED. The site will have NO login required.");
    }
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_redirect() -> Response {
    let now = chrono::Local::now();
    let month = now.format("%Y-%m").to_string();
    Redirect::to(&format!("/budget/{}", month)).into_response()
}

async fn login_get(
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

async fn login_post(
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