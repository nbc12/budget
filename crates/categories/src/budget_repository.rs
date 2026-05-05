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

    pub async fn copy_budgets(&mut self, source_month: &str, target_month: &str) -> Result<u64, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE month = $1")
            .bind(target_month)
            .fetch_one(&mut *self.conn)
            .await?;

        if count > 0 {
            return Ok(0);
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use database::get_test_db;
    use crate::repository::CategoryRepository;
    use crate::models::CreateCategoryRequest;

    #[tokio::test]
    async fn test_upsert_budget() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        
        // Need a category first
        let mut cat_repo = CategoryRepository::new(uow.connection());
        let cat_id = cat_repo.create(&CreateCategoryRequest {
            name: "Test".to_string(),
            color: "#000".to_string(),
            is_income: false,
            is_active: true,
        }).await.unwrap();

        let mut repo = MonthlyBudgetRepository::new(uow.connection());
        let req = CreateMonthlyBudgetRequest {
            category_id: cat_id,
            month: "2026-01".to_string(),
            limit_amount: 5000,
        };
        
        repo.upsert(&req).await.unwrap();
        
        let budgets = repo.get_for_month("2026-01").await.unwrap();
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].limit_amount, 5000);

        // Test update via upsert
        let update_req = CreateMonthlyBudgetRequest {
            category_id: cat_id,
            month: "2026-01".to_string(),
            limit_amount: 7500,
        };
        repo.upsert(&update_req).await.unwrap();
        let budgets = repo.get_for_month("2026-01").await.unwrap();
        assert_eq!(budgets[0].limit_amount, 7500);
    }

    #[tokio::test]
    async fn test_copy_budgets() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        
        let mut cat_repo = CategoryRepository::new(uow.connection());
        let cat_id = cat_repo.create(&CreateCategoryRequest {
            name: "Test".to_string(),
            color: "#000".to_string(),
            is_income: false,
            is_active: true,
        }).await.unwrap();

        let mut repo = MonthlyBudgetRepository::new(uow.connection());
        repo.upsert(&CreateMonthlyBudgetRequest {
            category_id: cat_id,
            month: "2026-01".to_string(),
            limit_amount: 5000,
        }).await.unwrap();

        let affected = repo.copy_budgets("2026-01", "2026-02").await.unwrap();
        assert_eq!(affected, 1);

        let budgets = repo.get_for_month("2026-02").await.unwrap();
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].limit_amount, 5000);
    }
}