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

#[cfg(test)]
mod tests {
    use super::*;
    use database::get_test_db;

    async fn setup_deps(conn: &mut database::Connection) -> (i64, i64) {
        let cat_id: i64 = sqlx::query_scalar(
            "INSERT INTO categories (name, color, is_income, is_active) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind("Test Cat")
        .bind("#000")
        .bind(false)
        .bind(true)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

        let card_id: i64 = sqlx::query_scalar(
            "INSERT INTO cards (name, is_active) VALUES ($1, $2) RETURNING id",
        )
        .bind("Test Card")
        .bind(true)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

        (cat_id, card_id)
    }

    #[tokio::test]
    async fn test_create_transaction() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let (cat_id, card_id) = setup_deps(uow.connection()).await;

        let mut repo = TransactionRepository::new(uow.connection());
        let req = CreateTransactionRequest::new(cat_id, Some(card_id), "2026-01-01".to_string(), 10.0, false, Some("Notes".into())).unwrap();
        
        let id = repo.create(&req).await.unwrap();
        assert!(id > 0);

        let t = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(t.amount, -1000);
        assert_eq!(t.notes, Some("Notes".to_string()));
    }

    #[tokio::test]
    async fn test_read_transactions() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let (cat_id, card_id) = setup_deps(uow.connection()).await;

        let mut repo = TransactionRepository::new(uow.connection());
        let req = CreateTransactionRequest::new(cat_id, Some(card_id), "2026-01-01".to_string(), 10.0, false, None).unwrap();
        repo.create(&req).await.unwrap();

        let list = repo.list_by_month("2026-01").await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_update_transaction() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let (cat_id, card_id) = setup_deps(uow.connection()).await;

        let mut repo = TransactionRepository::new(uow.connection());
        let id = repo.create(&CreateTransactionRequest::new(cat_id, Some(card_id), "2026-01-01".to_string(), 10.0, false, None).unwrap()).await.unwrap();

        let update_req = CreateTransactionRequest::new(cat_id, Some(card_id), "2026-01-02".to_string(), 20.0, true, Some("Updated".into())).unwrap();
        repo.update(id, &update_req).await.unwrap();

        let t = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(t.amount, 2000);
        assert_eq!(t.transaction_date, "2026-01-02");
        assert_eq!(t.notes, Some("Updated".to_string()));
    }

    #[tokio::test]
    async fn test_delete_transaction() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let (cat_id, card_id) = setup_deps(uow.connection()).await;

        let mut repo = TransactionRepository::new(uow.connection());
        let id = repo.create(&CreateTransactionRequest::new(cat_id, Some(card_id), "2026-01-01".to_string(), 10.0, false, None).unwrap()).await.unwrap();

        assert!(repo.find_by_id(id).await.unwrap().is_some());
        repo.delete(id).await.unwrap();
        assert!(repo.find_by_id(id).await.unwrap().is_none());
    }
}
