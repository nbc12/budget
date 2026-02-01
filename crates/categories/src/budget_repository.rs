use crate::models::{MonthlyBudget, CreateMonthlyBudgetRequest};
use database::{self, RepositoryError};
use sqlx::FromRow;

#[derive(FromRow)]
struct MonthlyBudgetRecord {
    id: i64,
    category_id: i64,
    month: String,
    limit_amount: i64,
}

impl From<MonthlyBudgetRecord> for MonthlyBudget {
    fn from(record: MonthlyBudgetRecord) -> Self {
        MonthlyBudget {
            id: record.id,
            category_id: record.category_id,
            month: record.month,
            limit_amount: record.limit_amount,
        }
    }
}

pub(crate) struct MonthlyBudgetRepository<'a> {
    conn: &'a mut database::Connection,
}

impl<'a> MonthlyBudgetRepository<'a> {
    pub fn new(conn: &'a mut database::Connection) -> Self {
        Self { conn }
    }

    pub async fn upsert(&mut self, req: &CreateMonthlyBudgetRequest) -> Result<(), RepositoryError> {
        // Upsert logic for SQLite
        sqlx::query(
            r#"
            INSERT INTO monthly_budgets (category_id, month, limit_amount)
            VALUES ($1, $2, $3)
            ON CONFLICT(category_id, month) DO UPDATE SET
            limit_amount = excluded.limit_amount
            "#
        )
        .bind(req.category_id)
        .bind(&req.month)
        .bind(req.limit_amount)
        .execute(&mut *self.conn)
        .await?;
        
        Ok(())
    }

    pub async fn get_for_month(&mut self, month: &str) -> Result<Vec<MonthlyBudget>, RepositoryError> {
        let records = sqlx::query_as::<_, MonthlyBudgetRecord>(
            "SELECT id, category_id, month, limit_amount FROM monthly_budgets WHERE month = $1",
        )
        .bind(month)
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    // "Auto-Copy" Logic
    // Copies limits from source_month to target_month ONLY if target_month has no entries.
    pub async fn copy_budgets(&mut self, source_month: &str, target_month: &str) -> Result<u64, RepositoryError> {
        // 1. Check if target has data
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE month = $1")
            .bind(target_month)
            .fetch_one(&mut *self.conn)
            .await?;

        if count > 0 {
            return Ok(0); // Already exists, don't overwrite
        }

        // 2. Copy
        let result = sqlx::query(
            r#"
            INSERT INTO monthly_budgets (category_id, month, limit_amount)
            SELECT category_id, $1, limit_amount FROM monthly_budgets WHERE month = $2
            "#
        )
        .bind(target_month)
        .bind(source_month)
        .execute(&mut *self.conn)
        .await?;

        Ok(result.rows_affected())
    }
}
