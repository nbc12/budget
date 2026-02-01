use crate::models::{CategoryBudgetView, UpdateCategoryRequest};
use crate::service::{CategoryError, CategoryService};
use axum::{
    extract::{State, Query, Path},
    http::StatusCode,
    response::{IntoResponse, Response, Redirect, Html},
    routing::{get, post, put},
    Form, Json, Router,
};
use common::AppState;
use std::sync::Arc;
use serde::Deserialize;
use serde_json::json;

impl IntoResponse for CategoryError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            CategoryError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            CategoryError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            CategoryError::NotFound => (StatusCode::NOT_FOUND, "Category not found".to_string()),
            CategoryError::Infrastructure(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

use askama::Template;

#[derive(Template)]
#[template(path = "manage_categories.html")]
pub struct ManageCategoriesTemplate {
    pub categories: Vec<crate::models::Category>,
    pub pastel_colors: Vec<String>,
}

pub fn categories_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_categories_view).post(create_category))
        .route("/api", get(list_categories_api))
        .route("/{id}", put(update_category).delete(delete_category))
        .route("/budget", get(get_budget_view))
        .route("/limit", post(set_limit))
        .with_state(state)
}

async fn list_categories_view(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, CategoryError> {
    let categories = CategoryService::list_categories(&state.db).await?;
    let pastel_colors = vec![
        "#FFB3BA", "#FFDFBA", "#FFFFBA", "#BAFFC9", "#BAE1FF", 
        "#E2F0CB", "#FDFD96", "#FFC3A0", "#FFD1DC", "#D4F0F0",
        "#CCE2CB", "#B6CFB6", "#97C1A9", "#FCB7AF", "#FFDAC1",
        "#E7FFAC", "#FFABAB", "#D5AAFF", "#85E3FF", "#B9F6CA"
    ].into_iter().map(|s| s.to_string()).collect();

    let template = ManageCategoriesTemplate { categories, pastel_colors };
    Ok(Html(template.render().map_err(|e| CategoryError::Infrastructure(e.to_string()))?))
}

async fn list_categories_api(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, CategoryError> {
    let categories = CategoryService::list_categories(&state.db).await?;
    Ok(Json(categories))
}

#[derive(Deserialize)]
pub struct CreateCategoryForm {
    pub name: String,
    pub monthly_limit: f64,
    pub is_income: Option<String>,
}

async fn create_category(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<CreateCategoryForm>,
) -> Result<impl IntoResponse, CategoryError> {
    let is_income = payload.is_income.as_deref() == Some("on");
    
    let id = CategoryService::create_category(
        &state.db, 
        payload.name,
        is_income,
    ).await?;
    
    // Set the initial limit for the current month
    let now = chrono::Local::now();
    let month = now.format("%Y-%m").to_string();
    
    CategoryService::set_monthly_limit(
        &state.db,
        id,
        month,
        payload.monthly_limit
    ).await?;
    
    Ok(Redirect::to("/")) 
}

async fn update_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<impl IntoResponse, CategoryError> {
    CategoryService::update_category(&state.db, id, payload.name, payload.color, payload.is_income, payload.is_active).await?;
    Ok(StatusCode::OK)
}

async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, CategoryError> {
    CategoryService::delete_category(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct BudgetQuery {
    month: String,
}

async fn get_budget_view(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BudgetQuery>,
) -> Result<Json<Vec<CategoryBudgetView>>, CategoryError> {
    let view = CategoryService::get_budget_view(&state.db, &params.month).await?;
    Ok(Json(view))
}

#[derive(Deserialize)]
struct SetLimitRequest {
    category_id: i64,
    month: String,
    limit: f64,
}

async fn set_limit(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetLimitRequest>,
) -> Result<impl IntoResponse, CategoryError> {
    CategoryService::set_monthly_limit(
        &state.db,
        payload.category_id,
        payload.month,
        payload.limit,
    ).await?;
    Ok(StatusCode::OK)
}
