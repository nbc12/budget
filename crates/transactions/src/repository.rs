use crate::models::{CreateTransactionRequest, Transaction};
use database::{self, RepositoryError};
use sqlx::FromRow;

#[derive(FromRow)]
struct TransactionRecord {
    id: i64,
    category_id: i64,
    card_id: Option<i64>,
    transaction_date: String,
    amount: i64,
    notes: Option<String>,
}

impl From<TransactionRecord> for Transaction {
    fn from(record: TransactionRecord) -> Self {
        Transaction {
            id: record.id,
            category_id: record.category_id,
            card_id: record.card_id,
            transaction_date: record.transaction_date,
            amount: record.amount,
            notes: record.notes,
        }
    }
}

pub(crate) struct TransactionRepository<'a> {
    conn: &'a mut database::Connection,
}

impl<'a> TransactionRepository<'a> {
    pub fn new(conn: &'a mut database::Connection) -> Self {
        Self { conn }
    }

    pub async fn create(&mut self, req: &CreateTransactionRequest) -> Result<i64, RepositoryError> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO transactions (category_id, card_id, transaction_date, amount, notes) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(req.category_id())
        .bind(req.card_id())
        .bind(req.transaction_date())
        .bind(req.amount())
        .bind(req.notes())
        .fetch_one(&mut *self.conn)
        .await?;
        
        Ok(id)
    }

    pub async fn update(&mut self, id: i64, req: &CreateTransactionRequest) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            "UPDATE transactions SET category_id = $1, card_id = $2, transaction_date = $3, amount = $4, notes = $5 WHERE id = $6",
        )
        .bind(req.category_id())
        .bind(req.card_id())
        .bind(req.transaction_date())
        .bind(req.amount())
        .bind(req.notes())
        .bind(id)
        .execute(&mut *self.conn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    pub async fn find_by_id(&mut self, id: i64) -> Result<Option<Transaction>, RepositoryError> {
        let record = sqlx::query_as::<_, TransactionRecord>(
            "SELECT id, category_id, card_id, transaction_date, amount, notes FROM transactions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *self.conn)
        .await?;

        Ok(record.map(|r| r.into()))
    }

    pub async fn list_by_month(&mut self, month: &str) -> Result<Vec<Transaction>, RepositoryError> {
        let records = sqlx::query_as::<_, TransactionRecord>(
            "SELECT id, category_id, card_id, transaction_date, amount, notes FROM transactions WHERE strftime('%Y-%m', transaction_date) = $1 ORDER BY transaction_date DESC",
        )
        .bind(month)
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete(&mut self, id: i64) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM transactions WHERE id = $1")
            .bind(id)
            .execute(&mut *self.conn)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
