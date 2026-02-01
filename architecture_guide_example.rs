use axum::Router;
use std::sync::Arc;

// ====================================================
// CRATE: `database` (Located at `/crates/database`)
// ====================================================
// SHARED INFRASTRUCTURE (GENERIC)
// All database-specific logic, connection handling, and transaction
// management lives here.
pub mod database {
    use sqlx::{Postgres, Transaction, postgres::PgPoolOptions};

    pub use sqlx::Error;
    pub use sqlx::Result;

    // --- Driver Adapter Pattern ---
    pub type Driver = Postgres;
    pub type Connection = sqlx::PgConnection;
    pub type Pool = sqlx::PgPool;

    #[derive(Debug, thiserror::Error)]
    pub enum RepositoryError {
        #[error("Database error: {0}")]
        Infrastructure(sqlx::Error),
        #[error("Resource not found")]
        NotFound,
        #[error("Unique constraint violation: {0}")]
        UniqueViolation(String),
        #[error("Check constraint violation: {0}")]
        CheckViolation(String),
    }

    impl From<sqlx::Error> for RepositoryError {
        fn from(err: sqlx::Error) -> Self {
            match err {
                sqlx::Error::RowNotFound => RepositoryError::NotFound,
                _ => {
                    if let Some(db_err) = err.as_database_error() {
                        if let Some(code) = db_err.code() {
                            match code.as_ref() {
                                "23505" => {
                                    return RepositoryError::UniqueViolation(
                                        db_err.message().to_string(),
                                    );
                                }
                                "23514" => {
                                    return RepositoryError::CheckViolation(
                                        db_err.message().to_string(),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    RepositoryError::Infrastructure(err)
                }
            }
        }
    }
    #[derive(Clone)]
    pub struct Database {
        pub pool: Pool,
    }

    impl Database {
        pub async fn new(connection_string: &str) -> sqlx::Result<Self> {
            sqlx::any::install_default_drivers();
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(connection_string)
                .await?;
            Ok(Self { pool })
        }

        pub async fn run_migrations(&self) -> sqlx::Result<()> {
            sqlx::migrate!("./migrations").run(&self.pool).await?;
            Ok(())
        }

        pub async fn begin(&self) -> Result<UnitOfWork<'_>, RepositoryError> {
            let tx = self.pool.begin().await?;
            Ok(UnitOfWork { tx })
        }
    }

    pub struct UnitOfWork<'a> {
        tx: Transaction<'a, Driver>,
    }

    impl<'a> UnitOfWork<'a> {
        pub async fn commit(self) -> Result<(), RepositoryError> {
            self.tx.commit().await?;
            Ok(())
        }

        pub fn connection(&mut self) -> &mut Connection {
            &mut *self.tx
        }
    }
}

// ====================================================
// CRATE: `orders` (Located at `/crates/orders`)
// ====================================================
// DOMAIN MODULE: ORDERS
// Contains all business logic related to the "Orders" domain.
pub mod orders {
    // LAYER 1: MODELS
    pub mod models {
        use axum::{
            http::StatusCode,
            response::{IntoResponse, Response},
            Json,
        };
        use serde::{Deserialize, Serialize};
        use serde_json::json;

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        pub struct Order {
            pub id: i64,
            pub name: String,
            pub status: String,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        pub struct OrderItem {
            pub id: i64,
            pub name: String,
        }

        // ENCAPSULATION: This struct has private fields.
        // Once created via new(), it is guaranteed to be valid.
        #[derive(Debug, Serialize)]
        pub struct CreateOrderRequest {
            order_name: String,
        }

        // Raw input struct for deserialization (since CreateOrderRequest has private fields)
        #[derive(Deserialize)]
        pub struct RawCreateOrderRequest {
            pub order_name: String,
        }

        #[derive(Debug, Deserialize, Serialize)]
        pub struct AddItemRequest {
            pub item_name: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct OrderCreatedResponse {
            pub id: i64,
        }

        impl IntoResponse for OrderCreatedResponse {
            fn into_response(self) -> Response {
                let body = Json(json!({
                    "id": self.id,
                    "status": "created"
                }));
                (StatusCode::CREATED, body).into_response()
            }
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct OrderDetailsResponse {
            pub id: i64,
            pub name: String,
            pub status: String,
            pub items: Vec<OrderItem>,
        }

        impl IntoResponse for OrderDetailsResponse {
            fn into_response(self) -> Response {
                (StatusCode::OK, Json(self)).into_response()
            }
        }

        impl CreateOrderRequest {
            pub fn new(order_name: String) -> Result<Self, String> {
                if order_name.len() > 20 {
                    return Err("Order name cannot exceed 20 characters.".to_string());
                }
                Ok(Self { order_name })
            }

            pub fn order_name(&self) -> &str {
                &self.order_name
            }
        }
    }

    // LAYER 2: REPOSITORY
    mod repository {
        use crate::database::{self, RepositoryError};
        use super::models::{CreateOrderRequest, Order, OrderItem};
        use sqlx::FromRow;

        #[derive(FromRow)]
        struct OrderRecord {
            id: i64,
            name: String,
            status: String,
        }

        impl TryFrom<OrderRecord> for Order {
            type Error = RepositoryError;
            fn try_from(record: OrderRecord) -> Result<Self, Self::Error> {
                Ok(Order {
                    id: record.id,
                    name: record.name,
                    status: record.status,
                })
            }
        }

        pub(crate) struct OrderRepository<'a> {
            conn: &'a mut database::Connection,
        }

        impl<'a> OrderRepository<'a> {
            pub fn new(conn: &'a mut database::Connection) -> Self {
                Self { conn }
            }

            pub async fn create_order_parent(
                &mut self,
                order: &CreateOrderRequest,
            ) -> Result<i64, RepositoryError> {
                let id: i64 = sqlx::query_scalar(
                    "INSERT INTO orders (name, status) VALUES ($1, 'PENDING') RETURNING id",
                )
                .bind(order.order_name())
                .fetch_one(&mut *self.conn)
                .await?;
                Ok(id)
            }

            pub async fn update_status(
                &mut self,
                order_id: i64,
                status: &str,
            ) -> Result<(), RepositoryError> {
                sqlx::query("UPDATE orders SET status = $1 WHERE id = $2")
                    .bind(status)
                    .bind(order_id)
                    .execute(&mut *self.conn)
                    .await?;
                Ok(())
            }

            pub async fn find_by_id(&mut self, id: i64) -> Result<Option<Order>, RepositoryError> {
                let record = sqlx::query_as::<_, OrderRecord>(
                    "SELECT id, name, status FROM orders WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(&mut *self.conn)
                .await?;

                match record {
                    Some(r) => {
                        let order = r.try_into()?;
                        Ok(Some(order))
                    }
                    None => Ok(None),
                }
            }

            pub async fn find_items_for_order(
                &mut self,
                order_id: i64,
            ) -> Result<Vec<OrderItem>, RepositoryError> {
                let items = sqlx::query_as::<_, (i64, String)>(
                    "SELECT id, item_name FROM order_items WHERE order_id = $1",
                )
                .bind(order_id)
                .fetch_all(&mut *self.conn)
                .await?
                .into_iter()
                .map(|(id, name)| OrderItem { id, name })
                .collect();
                Ok(items)
            }

            pub async fn add_item(
                &mut self,
                order_id: i64,
                item_name: &str,
            ) -> Result<i64, RepositoryError> {
                let id: i64 = sqlx::query_scalar(
                    "INSERT INTO order_items (order_id, item_name) VALUES ($1, $2) RETURNING id",
                )
                .bind(order_id)
                .bind(item_name)
                .fetch_one(&mut *self.conn)
                .await?;
                Ok(id)
            }

            pub async fn remove_item(
                &mut self,
                order_id: i64,
                item_id: i64,
            ) -> Result<(), RepositoryError> {
                let result = sqlx::query("DELETE FROM order_items WHERE id = $1 AND order_id = $2")
                    .bind(item_id)
                    .bind(order_id)
                    .execute(&mut *self.conn)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(RepositoryError::NotFound);
                }
                Ok(())
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::database::get_test_db;

            #[tokio::test]
            async fn test_order_lifecycle() {
                let db = get_test_db().await;
                let mut uow = db.begin().await.unwrap();
                let mut repo = OrderRepository::new(uow.connection());

                let req = CreateOrderRequest::new("Lifecycle Test".to_string()).unwrap();
                let id = repo.create_order_parent(&req).await.unwrap();
                assert!(id > 0);

                let order = repo.find_by_id(id).await.unwrap().expect("Order not found");
                assert_eq!(order.name, "Lifecycle Test");
                assert_eq!(order.status, "PENDING");

                repo.update_status(id, "COMPLETED").await.unwrap();
                let updated = repo.find_by_id(id).await.unwrap().unwrap();
                assert_eq!(updated.status, "COMPLETED");

                uow.commit().await.unwrap();
            }

            #[tokio::test]
            async fn test_item_management() {
                let db = get_test_db().await;
                let mut uow = db.begin().await.unwrap();
                let mut repo = OrderRepository::new(uow.connection());

                let parent_id = repo
                    .create_order_parent(&CreateOrderRequest {
                        order_name: "Item Test".into(),
                    })
                    .await
                    .unwrap();

                let item1_id = repo.add_item(parent_id, "Item 1").await.unwrap();
                let item2_id = repo.add_item(parent_id, "Item 2").await.unwrap();

                let items = repo.find_items_for_order(parent_id).await.unwrap();
                assert_eq!(items.len(), 2);
                assert!(items.iter().any(|i| i.name == "Item 1"));

                repo.remove_item(parent_id, item1_id).await.unwrap();
                let items_after = repo.find_items_for_order(parent_id).await.unwrap();
                assert_eq!(items_after.len(), 1);
                assert_eq!(items_after[0].id, item2_id);

                let result = repo.remove_item(parent_id, 9999).await;
                assert!(matches!(result, Err(RepositoryError::NotFound)));

                uow.commit().await.unwrap();
            }
        }
    }

    // LAYER 3: SERVICE
    pub mod service {
        use crate::database::{Database, RepositoryError};
        use crate::UserContext;
        // Use super:: to access sibling modules
        use super::models::{CreateOrderRequest, OrderCreatedResponse, OrderDetailsResponse};
        use super::repository::OrderRepository; // Works because it is pub(crate)
        use axum::{
            http::StatusCode,
            response::{IntoResponse, Response},
            Json,
        };
        use serde_json::json;
        use tracing::instrument;

        #[derive(Debug)]
        pub enum OrderError {
            InvalidOrder(String),
            InfrastructureError(String),
            NotFound(String),
            Forbidden(String),
            Conflict(String),
        }

        impl From<RepositoryError> for OrderError {
            fn from(err: RepositoryError) -> Self {
                // TRANSLATION LAYER: Repository -> Service
                match err {
                    RepositoryError::Infrastructure(e) => OrderError::InfrastructureError(e.to_string()),
                    RepositoryError::NotFound => OrderError::NotFound("Resource not found".into()),
                    RepositoryError::UniqueViolation(msg) => OrderError::Conflict(msg),
                    RepositoryError::CheckViolation(msg) => OrderError::InvalidOrder(msg),
                }
            }
        }

        impl IntoResponse for OrderError {
            fn into_response(self) -> Response {
                // TRANSLATION LAYER: Service -> Handler/HTTP
                let (status, error_msg) = match self {
                    OrderError::InvalidOrder(msg) => (StatusCode::BAD_REQUEST, msg),
                    OrderError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                    OrderError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
                    OrderError::Conflict(msg) => (StatusCode::CONFLICT, msg),
                    OrderError::InfrastructureError(msg) => {
                        eprintln!("Infrastructure Error: {}", msg);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Internal Service Error".to_string(),
                        )
                    }
                };
                (status, Json(json!({ "error": error_msg }))).into_response()
            }
        }

        pub struct OrderService;

        impl OrderService {
            /// Orchestrates order creation.
            /// Note: Accepts primitives (String) to enforce that validation
            /// happens inside the service via Domain Model constructors.
            #[instrument(skip(db, ctx))]
            pub async fn create_and_process_order(
                ctx: &UserContext, // SECURITY: Context passed in
                db: &Database,
                order_name: String,
            ) -> Result<OrderCreatedResponse, OrderError> {
                // 0. AUTHORIZATION (Dummy check)
                if ctx.roles.is_empty() {
                    return Err(OrderError::Forbidden("No roles assigned".into()));
                }

                // 1. INPUT VALIDATION (Structural)
                let req = CreateOrderRequest::new(order_name).map_err(OrderError::InvalidOrder)?;

                let mut uow = db.begin().await?;
                let mut repo = OrderRepository::new(uow.connection());

                // 2. STATEFUL VALIDATION (Business Logic)
                let parent_id = repo.create_order_parent(&req).await?;

                let default_items = vec!["Widget A", "Widget B"];
                for item in default_items {
                    repo.add_item(parent_id, item).await?;
                }

                repo.update_status(parent_id, "PROCESSING").await?;

                uow.commit().await?;

                Ok(OrderCreatedResponse { id: parent_id })
            }

            #[instrument(skip(db))]
            pub async fn get_order(
                db: &Database,
                order_id: i64,
            ) -> Result<OrderDetailsResponse, OrderError> {
                let mut uow = db.begin().await?;
                let mut repo = OrderRepository::new(uow.connection());

                let order = repo
                    .find_by_id(order_id)
                    .await?
                    .ok_or_else(|| OrderError::NotFound(format!("Order {} not found", order_id)))?;

                let items = repo.find_items_for_order(order.id).await?;

                uow.commit().await?;

                Ok(OrderDetailsResponse {
                    id: order.id,
                    name: order.name,
                    status: order.status,
                    items: items
                        .into_iter()
                        .map(|i| super::models::OrderItem {
                            id: i.id,
                            name: i.name,
                        })
                        .collect(),
                })
            }

            // ... (add_item and remove_item remain similar but would also take ctx in a real app)
             pub async fn add_item_to_order(
                db: &Database,
                order_id: i64,
                item_name: String,
            ) -> Result<OrderCreatedResponse, OrderError> {
                let mut uow = db.begin().await?;
                let mut repo = OrderRepository::new(uow.connection());

                if repo.find_by_id(order_id).await?.is_none() {
                    return Err(OrderError::NotFound(format!(
                        "Order {} not found",
                        order_id
                    )));
                }

                let item_id = repo.add_item(order_id, &item_name).await?;

                uow.commit().await?;
                Ok(OrderCreatedResponse { id: item_id })
            }

            pub async fn remove_item_from_order(
                db: &Database,
                order_id: i64,
                item_id: i64,
            ) -> Result<(), OrderError> {
                let mut uow = db.begin().await?;
                let mut repo = OrderRepository::new(uow.connection());

                repo.remove_item(order_id, item_id)
                    .await
                    .map_err(|e| match e {
                        RepositoryError::NotFound => OrderError::NotFound(format!(
                            "Item {} not found in Order {}",
                            item_id, order_id
                        )),
                        _ => e.into(),
                    })?;

                uow.commit().await?;
                Ok(())
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::database::get_test_db;
            use crate::UserContext;

            fn mock_ctx() -> UserContext {
                UserContext { user_id: 1, roles: vec!["admin".into()] }
            }

            #[tokio::test]
            async fn test_create_and_process_flow() {
                let db = get_test_db().await;
                let ctx = mock_ctx();

                let resp = OrderService::create_and_process_order(&ctx, &db, "Service Order".to_string())
                    .await
                    .expect("Failed to create order");

                let details = OrderService::get_order(&db, resp.id).await.unwrap();
                assert_eq!(details.name, "Service Order");
                assert_eq!(details.status, "PROCESSING");
                assert_eq!(details.items.len(), 2);
            }

            #[tokio::test]
            async fn test_create_validation_failure() {
                let db = get_test_db().await;
                let ctx = mock_ctx();
                let err = OrderService::create_and_process_order(
                    &ctx,
                    &db,
                    "Name Too Long For The System".to_string(),
                )
                .await;
                assert!(matches!(err, Err(OrderError::InvalidOrder(_))));
            }

            #[tokio::test]
            async fn test_get_order_not_found() {
                let db = get_test_db().await;
                let err = OrderService::get_order(&db, 999).await;
                assert!(matches!(err, Err(OrderError::NotFound(_))));
            }

            #[tokio::test]
            async fn test_add_item_success() {
                let db = get_test_db().await;
                let ctx = mock_ctx();
                let resp = OrderService::create_and_process_order(&ctx, &db, "Order A".into())
                    .await
                    .unwrap();

                let item_resp = OrderService::add_item_to_order(&db, resp.id, "New Widget".into())
                    .await
                    .unwrap();
                assert!(item_resp.id > 0);

                let details = OrderService::get_order(&db, resp.id).await.unwrap();
                assert_eq!(details.items.len(), 3);
            }

            #[tokio::test]
            async fn test_add_item_order_not_found() {
                let db = get_test_db().await;
                let err = OrderService::add_item_to_order(&db, 999, "Widget".into()).await;
                assert!(matches!(err, Err(OrderError::NotFound(_))));
            }

            #[tokio::test]
            async fn test_remove_item_success() {
                let db = get_test_db().await;
                let ctx = mock_ctx();
                let resp = OrderService::create_and_process_order(&ctx, &db, "Order B".into())
                    .await
                    .unwrap();

                let details = OrderService::get_order(&db, resp.id).await.unwrap();
                let item_to_remove = details.items.first().unwrap().id;

                OrderService::remove_item_from_order(&db, resp.id, item_to_remove)
                    .await
                    .unwrap();

                let details_after = OrderService::get_order(&db, resp.id).await.unwrap();
                assert_eq!(details_after.items.len(), 1);
            }

            #[tokio::test]
            async fn test_remove_item_not_found() {
                let db = get_test_db().await;
                let ctx = mock_ctx();
                let resp = OrderService::create_and_process_order(&ctx, &db, "Order C".into())
                    .await
                    .unwrap();

                let err = OrderService::remove_item_from_order(&db, resp.id, 9999).await;
                assert!(matches!(err, Err(OrderError::NotFound(_))));
            }
        }
    }

    // LAYER 4: HANDLER
    pub mod handler {
        use super::models::{AddItemRequest, RawCreateOrderRequest, OrderItem};
        use super::service::OrderService;
        use crate::{AppState, UserContext};
        use axum::{
            extract::{Path, State},
            http::StatusCode,
            response::{Html, IntoResponse},
            routing::{delete, get, post},
            Form, Router,
        };
        use serde::Deserialize;
        use std::sync::Arc;

        // --- VIEW LAYER (TEMPLATES) ---
        // In a real app, these would be in `templates/*.html`
        // and derive `askama::Template`.

        // #[derive(Template)]
        // #[template(path = "order_details.html")]
        pub struct OrderDetailsTemplate {
            pub id: i64,
            pub name: String,
            pub status: String,
            pub items: Vec<OrderItem>,
        }

        // Mocking the Template trait behavior for this example
        impl OrderDetailsTemplate {
            fn render(&self) -> Result<String, String> {
                Ok(format!("<h1>Order {}</h1><p>Status: {}</p>", self.name, self.status))
            }
        }

        #[derive(Deserialize)]
        pub struct OrderPath {
            pub id: i64,
        }

        #[derive(Deserialize)]
        pub struct ItemPath {
            pub id: i64,
            pub item_id: i64,
        }

        pub fn orders_router(state: Arc<AppState>) -> Router {
            Router::new()
                .route("/", post(create_order_handler).with_state(state.clone()))
                .route("/{id}", get(get_order_handler).with_state(state.clone()))
                .route(
                    "/{id}/items",
                    post(add_item_handler).with_state(state.clone()),
                )
                .route(
                    "/{id}/items/{item_id}",
                    delete(remove_item_handler).with_state(state.clone()),
                )
        }

        // MOCK AUTH EXTRACTOR
        async fn mock_auth() -> UserContext {
            UserContext {
                user_id: 101,
                roles: vec!["user".to_string()],
            }
        }

        // HANDLER: Create Order (Form Submission)
        // Returns HTML (redirect or success page)
        pub async fn create_order_handler(
            State(state): State<Arc<AppState>>,
            // SSR uses Form data, not JSON
            Form(payload): Form<RawCreateOrderRequest>,
        ) -> Result<impl IntoResponse, impl IntoResponse> {
            let ctx = mock_auth().await;
            match OrderService::create_and_process_order(&ctx, &state.db, payload.order_name).await {
                Ok(created) => {
                    // In a real SSR app, we often Redirect after POST
                    // specific redirect logic omitted for brevity
                    Ok(Html(format!("Order Created: {}", created.id)))
                }
                Err(e) => Err(e),
            }
        }

        // HANDLER: Get Order (Render Template)
        pub async fn get_order_handler(
            State(state): State<Arc<AppState>>,
            Path(params): Path<OrderPath>,
        ) -> Result<impl IntoResponse, impl IntoResponse> {
            let order_dto = OrderService::get_order(&state.db, params.id).await?;

            // MAPPING: Domain DTO -> View Template
            let template = OrderDetailsTemplate {
                id: order_dto.id,
                name: order_dto.name,
                status: order_dto.status,
                items: order_dto.items,
            };

            // RENDER
            match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Template Error")),
            }
        }

        pub async fn add_item_handler(
            State(state): State<Arc<AppState>>,
            Path(params): Path<OrderPath>,
            Form(payload): Form<AddItemRequest>,
        ) -> Result<impl IntoResponse, impl IntoResponse> {
            match OrderService::add_item_to_order(&state.db, params.id, payload.item_name).await {
                Ok(_) => Ok(Html("Item Added".into())),
                Err(e) => Err(e),
            }
        }

        pub async fn remove_item_handler(
            State(state): State<Arc<AppState>>,
            Path(params): Path<ItemPath>,
        ) -> Result<impl IntoResponse, impl IntoResponse> {
            match OrderService::remove_item_from_order(&state.db, params.id, params.item_id).await {
                Ok(_) => Ok(StatusCode::NO_CONTENT), // DELETEs might still be AJAX or Form with method override
                Err(e) => Err(e),
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::database::get_test_db;
            use crate::Config;
            use axum::{
                body::Body,
                http::{Request, StatusCode},
            };
            use tower::ServiceExt;

            #[tokio::test]
            async fn test_create_order_handler_via_form() {
                let db = get_test_db().await;
                let config = Config { database_url: "mem".into(), port: 0 };
                let state = Arc::new(AppState { db, config });
                let app = orders_router(state);

                // x-www-form-urlencoded body
                let req_body = "order_name=HandlerOrder";

                let request = Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(req_body))
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();

                assert_eq!(response.status(), StatusCode::OK); // Or 201/303 depending on logic
            }

            #[tokio::test]
            async fn test_get_order_handler_renders_html() {
                let db = get_test_db().await;
                let config = Config { database_url: "mem".into(), port: 0 };
                let ctx = crate::UserContext { user_id: 1, roles: vec!["admin".into()] };

                let created = crate::orders::service::OrderService::create_and_process_order(
                    &ctx,
                    &db,
                    "Seed Order".to_string(),
                )
                .await
                .unwrap();

                let state = Arc::new(AppState { db, config });
                let app = orders_router(state);

                let uri = format!("/{}", created.id);
                let request = Request::builder().uri(&uri).body(Body::empty()).unwrap();

                let response = app.oneshot(request).await.unwrap();

                assert_eq!(response.status(), StatusCode::OK);
                // Check that we got HTML back
                // (In a real test we might inspect headers)
            }
        }
    }
}

// ====================================================
// MAIN APPLICATION ENTRY POINT
// ====================================================
// This is the main crate. It would import the routers provided by the domain crates, and set them up with the database context.
use database::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    // 12-FACTOR: Load from Env or Fail Fast
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a number"),
        }
    }
}

// SECURITY CONTEXT
#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: i64,
    pub roles: Vec<String>,
}

#[tokio::main]
async fn main() {
    // 1. Load Config (Fail Fast)
    // For this example, we manually set env vars if they aren't there to make it runnable
    if std::env::var("DATABASE_URL").is_err() {
         std::env::set_var("DATABASE_URL", "sqlite::memory:");
    }
    let config = Config::from_env();

    // 2. Initialize Infrastructure
    let db = database::new_database(&config.database_url).await.unwrap();

    let state = Arc::new(AppState { db, config: config.clone() });

    // 3. Static Assets (SSR Requirement)
    // In a real app, use `tower_http::services::ServeDir`
    // let serve_dir = tower_http::services::ServeDir::new("public");

    let app = Router::new()
        .nest("/orders", orders::handler::orders_router(state.clone()))
        // .nest_service("/public", serve_dir)
        // .layer(middleware::from_fn(auth_middleware)) // Inject UserContext here
        ;

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
