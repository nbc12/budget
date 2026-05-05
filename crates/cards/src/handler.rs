use crate::models::{CreateCardRequest, Card, UpdateCardRequest};
use crate::service::{CardError, CardService};
use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use common::AppState;
use std::sync::Arc;
use serde_json::json;

impl IntoResponse for CardError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            CardError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            CardError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            CardError::NotFound => (StatusCode::NOT_FOUND, "Card not found".to_string()),
            CardError::Infrastructure(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub fn cards_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_active_cards).post(create_card))
        .route("/all", get(list_all_cards))
        .route("/{id}", put(update_card))
        .with_state(state)
}

async fn list_active_cards(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Card>>, CardError> {
    let cards = CardService::list_active_cards(&state.db).await?;
    Ok(Json(cards))
}

async fn list_all_cards(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Card>>, CardError> {
    let cards = CardService::list_cards(&state.db).await?;
    Ok(Json(cards))
}

async fn create_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCardRequest>,
) -> Result<impl IntoResponse, CardError> {
    let id = CardService::create_card(&state.db, payload.name).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn update_card(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCardRequest>,
) -> Result<impl IntoResponse, CardError> {
    CardService::update_card(&state.db, id, payload.name, payload.is_active).await?;
    Ok(StatusCode::OK)
}
