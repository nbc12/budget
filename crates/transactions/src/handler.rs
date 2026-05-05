use crate::models::{RawCreateTransactionRequest};
use crate::service::{TransactionError, TransactionService};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post, delete},
    Form, Json, Router,
};
use common::AppState;
use std::sync::Arc;
use askama::Template;
use serde::Deserialize;
use serde_json::json;
use categories::virtual_budget::VirtualBudgetService;

impl IntoResponse for TransactionError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            TransactionError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            TransactionError::NotFound => (StatusCode::NOT_FOUND, "Transaction not found".to_string()),
            TransactionError::Infrastructure(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

#[derive(Template)]
#[template(path = "month_view.html")]
pub struct MonthViewTemplate {
    pub month: String,
    pub month_display: String,
    pub overview: FinancialOverview,
    pub budget_rows: Vec<BudgetRowView>,
    pub virtual_rows: Vec<VirtualCategoryView>,
    pub transactions: Vec<TransactionView>,
    pub categories: Vec<categories::models::Category>,
    pub cards: Vec<cards::models::Card>,
}

pub struct FinancialOverview {
    pub total_income: String,
    pub total_expenses: String,
    pub net_balance: String,
    pub net_is_positive: bool,
}

#[derive(Clone)]
pub struct BudgetRowView {
    pub category_id: i64,
    pub category_name: String,
    pub category_color: String,
    pub limit_dollars: String,
    pub spent_dollars: String,
    pub remaining_dollars: String,
    pub percent_spent: String,
    pub percent_remaining: String,
    pub is_over_budget: bool,
    pub is_income: bool,
    pub is_active: bool,
}

pub struct VirtualCategoryView {
    pub name: String,
    pub amount_dollars: String,
    pub is_income: bool,
}

#[derive(Template)]
#[template(path = "row_snippet.html")]
pub struct TransactionRowTemplate {
    pub t: TransactionView,
}

pub struct TransactionView {
    pub id: i64,
    pub category_id: i64,
    pub card_id: i64, 
    pub category_name: String,
    pub category_color: String,
    pub card_name: String,
    pub transaction_date: String,
    pub transaction_date_display: String,
    pub amount_dollars: String,
    pub is_income: bool,
    pub notes: String,
}

#[derive(Deserialize)]
pub struct MonthParam {
    pub month: String, // YYYY-MM
}

#[derive(Deserialize)]
pub struct UpdateTransactionRequest {
    pub category_id: i64,
    pub card_id: Option<i64>,
    pub transaction_date: String,
    pub amount_dollars: f64,
    pub notes: Option<String>,
}

pub fn transactions_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // Specific routes first
        .route("/add", post(create_transaction))
        // Then parameterized routes
        .route("/{month}", get(get_month_view))
        .route("/transaction/{id}", delete(delete_transaction).put(update_transaction))
        .with_state(state)
}

async fn get_month_view(
    State(state): State<Arc<AppState>>,
    Path(params): Path<MonthParam>,
) -> Result<impl IntoResponse, TransactionError> {
    tracing::info!("Fetching month view for: {}", params.month);

    // 0. Ensure budgets exist for this month (Auto-Copy logic)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{}-01", params.month), "%Y-%m-%d") {
        let prev_date = date - chrono::Months::new(1);
        let previous_month = prev_date.format("%Y-%m").to_string();
        
        if let Err(e) = categories::service::CategoryService::ensure_budgets_exist(&state.db, &params.month, &previous_month).await {
            tracing::warn!("Auto-copy budgets failed: {}. Continuing anyway.", e);
        }
    }

    // 1. Get transactions and basic summary
    let (transactions, summary) = TransactionService::get_month_view(&state.db, &params.month).await.map_err(|e| {
        tracing::error!("get_month_view error: {:?}", e);
        e
    })?;
    
    // 2. Get categories and monthly budgets
    let budget_views = categories::service::CategoryService::get_budget_view(&state.db, &params.month)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get budget view: {}", e);
            TransactionError::Infrastructure(e.to_string())
        })?;
        
    // 3. Get cards
    let all_cards = cards::service::CardService::list_cards(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list cards: {}", e);
            TransactionError::Infrastructure(e.to_string())
        })?;
    
    // 4. Enrich budget views with actual 'spent' data
    let mut enriched_budget_rows = Vec::new();
    let mut transactions_for_virtual = Vec::new();

    for view_ref in &budget_views {
        let mut view = view_ref.clone();
        let actual: i64 = if view.category.is_income {
            // For income, sum positive amounts
            transactions.iter()
                .filter(|t| t.category_id == view.category.id && t.amount > 0)
                .map(|t| t.amount)
                .sum()
        } else {
            // For expenses, sum absolute negative amounts
            transactions.iter()
                .filter(|t| t.category_id == view.category.id && t.amount < 0)
                .map(|t| t.amount.abs())
                .sum()
        };
        
        view.spent = actual;
        let limit = view.budget.as_ref().map(|b| b.limit_amount).unwrap_or(0);
        
        if view.category.is_income {
            // For income: good if actual > budget
            view.remaining = actual - limit;
        } else {
            // For expenses: good if limit > actual
            view.remaining = limit - actual;
        }

        let (p_spent, p_rem) = if limit == 0 {
            (0.0, 0.0)
        } else {
            let spent = (actual as f64 / limit as f64) * 100.0;
            let rem = (view.remaining as f64 / limit as f64) * 100.0;
            (spent, rem)
        };

        enriched_budget_rows.push(BudgetRowView {
            category_id: view.category.id,
            category_name: view.category.name.clone(),
            category_color: view.category.color.clone(),
            limit_dollars: format!("{:.2}", limit as f64 / 100.0),
            spent_dollars: format!("{:.2}", actual as f64 / 100.0),
            remaining_dollars: format!("{:.2}", view.remaining as f64 / 100.0),
            percent_spent: format!("{:.0}", p_spent),
            percent_remaining: format!("{:.0}", p_rem),
            is_over_budget: if view.category.is_income { view.remaining < 0 } else { view.remaining < 0 }, // Both mean we are "behind" target
            is_income: view.category.is_income,
            is_active: view.category.is_active,
        });
    }

    // 5. Calculate Virtual Rows
    for t in &transactions {
        transactions_for_virtual.push((t.category_id, t.amount));
    }
    // Re-calculating with the updated 'spent' data if needed for splits
    // For now, our virtual service just takes raw transactions
    let raw_budget_views = enriched_budget_rows.iter().map(|r| {
        // Dummy conversion back for the service
        categories::models::CategoryBudgetView {
            category: categories::models::Category { 
                id: r.category_id, 
                name: r.category_name.clone(), 
                color: r.category_color.clone(),
                is_income: r.is_income,
                is_active: true // Budget rows in this view are always active or have budget
            },
            budget: None,
            spent: (r.spent_dollars.parse::<f64>().unwrap_or(0.0) * 100.0) as i64,
            remaining: 0,
        }
    }).collect::<Vec<_>>();

    let virtual_categories = VirtualBudgetService::calculate_virtual_rows(&raw_budget_views, &transactions_for_virtual);
    let virtual_rows = virtual_categories.into_iter().map(|v| VirtualCategoryView {
        name: v.name,
        amount_dollars: format!("{:.2}", v.amount as f64 / 100.0),
        is_income: v.is_income,
    }).collect();

    // 6. Map transactions for view
    let transaction_views = transactions.into_iter().map(|t| {
        let cat = enriched_budget_rows.iter()
            .find(|r| r.category_id == t.category_id);
            
        let cat_name = cat.map(|r| r.category_name.clone()).unwrap_or_else(|| "Unknown".to_string());
        let cat_color = cat.map(|r| r.category_color.clone()).unwrap_or_else(|| "#ffffff".to_string());
            
        let card_name = all_cards.iter()
            .find(|c| Some(c.id) == t.card_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Cash".to_string());
            
        let date_display = chrono::NaiveDate::parse_from_str(&t.transaction_date, "%Y-%m-%d")
            .map(|d| d.format("%e %b %Y").to_string())
            .unwrap_or_else(|_| t.transaction_date.clone());
            
        TransactionView {
            id: t.id,
            category_id: t.category_id,
            card_id: t.card_id.unwrap_or(0),
            category_name: cat_name,
            category_color: cat_color,
            card_name,
            transaction_date: t.transaction_date,
            transaction_date_display: date_display,
            amount_dollars: format!("{:.2}", t.amount.abs() as f64 / 100.0),
            is_income: t.amount > 0,
            notes: t.notes.unwrap_or_default(),
        }
    }).collect();

    let overview = FinancialOverview {
        total_income: format!("{:.2}", summary.total_income as f64 / 100.0),
        total_expenses: format!("{:.2}", summary.total_expenses as f64 / 100.0),
        net_balance: format!("{:.2}", summary.net as f64 / 100.0),
        net_is_positive: summary.net >= 0,
    };

    let month_display = chrono::NaiveDate::parse_from_str(&format!("{}-01", params.month), "%Y-%m-%d")
        .map(|d| d.format("%B %Y").to_string())
        .unwrap_or_else(|_| params.month.clone());

    let categories_for_template: Vec<categories::models::Category> = budget_views.into_iter().map(|v| v.category).collect();

    let template = MonthViewTemplate {
        month: params.month,
        month_display,
        overview,
        budget_rows: enriched_budget_rows.clone(),
        virtual_rows,
        transactions: transaction_views,
        categories: categories_for_template,
        cards: all_cards,
    };

    Ok(Html(template.render().map_err(|e| TransactionError::Infrastructure(e.to_string()))?))
}

async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<RawCreateTransactionRequest>,
) -> Result<impl IntoResponse, TransactionError> {
    let month = if payload.transaction_date.len() >= 7 {
        payload.transaction_date[0..7].to_string()
    } else {
        chrono::Local::now().format("%Y-%m").to_string()
    };

    let card_id = payload.card_id
        .as_ref()
        .and_then(|s| if s.is_empty() { None } else { s.parse::<i64>().ok() });

    TransactionService::create_transaction(
        &state.db,
        payload.category_id,
        card_id,
        payload.transaction_date,
        payload.amount_dollars,
        payload.notes,
    ).await.map_err(|e| {
        tracing::error!("create_transaction error: {:?}", e);
        e
    })?;
    
    Ok(axum::response::Redirect::to(&format!("/budget/{}", month)))
}

async fn update_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> Result<impl IntoResponse, TransactionError> {
    let transaction = TransactionService::update_transaction(
        &state.db,
        id,
        payload.category_id,
        payload.card_id,
        payload.transaction_date,
        payload.amount_dollars,
        payload.notes,
    ).await?;
    
    let categories = categories::service::CategoryService::list_categories(&state.db)
        .await
        .map_err(|e| TransactionError::Infrastructure(e.to_string()))?;
        
    let all_cards = cards::service::CardService::list_cards(&state.db)
        .await
        .map_err(|e| TransactionError::Infrastructure(e.to_string()))?;

    let cat = categories.iter()
        .find(|c| c.id == transaction.category_id);
    let cat_name = cat.map(|c| c.name.clone()).unwrap_or_else(|| "Unknown".to_string());
    let cat_color = cat.map(|c| c.color.clone()).unwrap_or_else(|| "#ffffff".to_string());
        
    let card_name = all_cards.iter()
        .find(|c| Some(c.id) == transaction.card_id)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Cash".to_string());
        
    let date_display = chrono::NaiveDate::parse_from_str(&transaction.transaction_date, "%Y-%m-%d")
        .map(|d| d.format("%e %b %Y").to_string())
        .unwrap_or_else(|_| transaction.transaction_date.clone());
        
    let view = TransactionView {
        id: transaction.id,
        category_id: transaction.category_id,
        card_id: transaction.card_id.unwrap_or(0),
        category_name: cat_name,
        category_color: cat_color,
        card_name,
        transaction_date: transaction.transaction_date,
        transaction_date_display: date_display,
        amount_dollars: format!("{:.2}", transaction.amount.abs() as f64 / 100.0),
        is_income: transaction.amount > 0,
        notes: transaction.notes.unwrap_or_default(),
    };
    
    let template = TransactionRowTemplate { t: view };
    Ok(Html(template.render().map_err(|e| TransactionError::Infrastructure(e.to_string()))?))
}

async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, TransactionError> {
    TransactionService::delete_transaction(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
