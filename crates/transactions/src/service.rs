use crate::models::{CreateTransactionRequest, Transaction, MonthlySummary};
use crate::repository::TransactionRepository;
use database::{RepositoryError, Database};
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Database error: {0}")]
    Infrastructure(String),
    #[error("Transaction not found")]
    NotFound,
}

impl From<RepositoryError> for TransactionError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => TransactionError::NotFound,
            RepositoryError::Infrastructure(e) => TransactionError::Infrastructure(e.to_string()),
            _ => TransactionError::Infrastructure(err.to_string()),
        }
    }
}

pub struct TransactionService;

impl TransactionService {
    #[instrument(skip(db))]
    pub async fn create_transaction(
        db: &Database,
        category_id: i64,
        card_id: Option<i64>,
        date: String,
        amount_dollars: f64,
        notes: Option<String>,
    ) -> Result<i64, TransactionError> {
        // Look up category to determine if it's income
        let category = categories::service::CategoryService::get_category(db, category_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get category for transaction: {:?}", e);
                TransactionError::InvalidInput("Invalid category ID".into())
            })?;

        let req = CreateTransactionRequest::new(category_id, card_id, date, amount_dollars, category.is_income, notes)
            .map_err(TransactionError::InvalidInput)?;

        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = TransactionRepository::new(uow.connection());
        
        let id = repo.create(&req).await?;
        
        uow.commit().await.map_err(RepositoryError::from)?;
        
        Ok(id)
    }

    #[instrument(skip(db))]
    pub async fn update_transaction(
        db: &Database,
        id: i64,
        category_id: i64,
        card_id: Option<i64>,
        date: String,
        amount_dollars: f64,
        notes: Option<String>,
    ) -> Result<Transaction, TransactionError> {
        // Look up category to determine if it's income
        let category = categories::service::CategoryService::get_category(db, category_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get category for transaction update: {:?}", e);
                TransactionError::InvalidInput("Invalid category ID".into())
            })?;

        let req = CreateTransactionRequest::new(category_id, card_id, date, amount_dollars, category.is_income, notes)
            .map_err(TransactionError::InvalidInput)?;

        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = TransactionRepository::new(uow.connection());
        
        repo.update(id, &req).await?;
        
        let transaction = repo.find_by_id(id).await?
            .ok_or(TransactionError::NotFound)?;
            
        uow.commit().await.map_err(RepositoryError::from)?;
        
        Ok(transaction)
    }

    #[instrument(skip(db))]
    pub async fn get_transaction(db: &Database, id: i64) -> Result<Transaction, TransactionError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = TransactionRepository::new(uow.connection());
        
        let transaction = repo.find_by_id(id).await?
            .ok_or(TransactionError::NotFound)?;
            
        Ok(transaction)
    }

    #[instrument(skip(db))]
    pub async fn get_month_view(
        db: &Database,
        month: &str, // YYYY-MM
    ) -> Result<(Vec<Transaction>, MonthlySummary), TransactionError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = TransactionRepository::new(uow.connection());
        
        let transactions = repo.list_by_month(month).await?;
        
        let mut total_income = 0;
        let mut total_expenses = 0;
        
        for t in &transactions {
            if t.amount > 0 {
                total_income += t.amount;
            } else {
                total_expenses += t.amount.abs();
            }
        }
        
        let summary = MonthlySummary {
            month: month.to_string(),
            total_income,
            total_expenses,
            net: total_income - total_expenses,
        };
        
        Ok((transactions, summary))
    }

    #[instrument(skip(db))]
    pub async fn delete_transaction(db: &Database, id: i64) -> Result<(), TransactionError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = TransactionRepository::new(uow.connection());
        
        repo.delete(id).await?;
        
        uow.commit().await.map_err(RepositoryError::from)?;
        Ok(())
    }
}
