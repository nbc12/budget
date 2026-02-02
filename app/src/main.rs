use axum::{
    routing::{get},
    Router, 
    middleware::{self},
};
use common::{AppState, Config, auth::{auth_middleware}};
use database::Database;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use rust_embed::RustEmbed;
use axum_embed::ServeEmbed;
use tower_sessions::{MemoryStore, SessionManagerLayer};

mod handlers;
use handlers::auth::{login_get, login_post, root_redirect};

#[derive(RustEmbed, Clone)]
#[folder = "public/"]
struct Assets;

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
    axum::serve(listener, app).await?;

    Ok(())
}