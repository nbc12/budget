use crate::models::{Category, CreateCategoryRequest};
use database::{self, RepositoryError};
use sqlx::FromRow;

#[derive(FromRow)]
struct CategoryRecord {
    id: i64,
    name: String,
    color: String,
    is_income: bool,
    is_active: bool,
}

impl From<CategoryRecord> for Category {
    fn from(record: CategoryRecord) -> Self {
        Category {
            id: record.id,
            name: record.name,
            color: record.color,
            is_income: record.is_income,
            is_active: record.is_active,
        }
    }
}

pub(crate) struct CategoryRepository<'a> {
    conn: &'a mut database::Connection,
}

impl<'a> CategoryRepository<'a> {
    pub fn new(conn: &'a mut database::Connection) -> Self {
        Self { conn }
    }

    pub async fn create(&mut self, req: &CreateCategoryRequest) -> Result<i64, RepositoryError> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO categories (name, color, is_income, is_active) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(&req.name)
        .bind(&req.color)
        .bind(req.is_income)
        .bind(req.is_active)
        .fetch_one(&mut *self.conn)
        .await?;
        
        Ok(id)
    }

    pub async fn list(&mut self) -> Result<Vec<Category>, RepositoryError> {
        let records = sqlx::query_as::<_, CategoryRecord>(
            "SELECT id, name, color, is_income, is_active FROM categories ORDER BY name",
        )
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_by_id(&mut self, id: i64) -> Result<Option<Category>, RepositoryError> {
        let record = sqlx::query_as::<_, CategoryRecord>(
            "SELECT id, name, color, is_income, is_active FROM categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *self.conn)
        .await?;

        Ok(record.map(|r| r.into()))
    }

    pub async fn update(&mut self, id: i64, name: &str, color: Option<&str>, is_income: bool, is_active: bool) -> Result<(), RepositoryError> {
        if let Some(c) = color {
            let result = sqlx::query("UPDATE categories SET name = $1, color = $2, is_income = $3, is_active = $4 WHERE id = $5")
                .bind(name)
                .bind(c)
                .bind(is_income)
                .bind(is_active)
                .bind(id)
                .execute(&mut *self.conn)
                .await?;
             if result.rows_affected() == 0 { return Err(RepositoryError::NotFound); }
        } else {
            let result = sqlx::query("UPDATE categories SET name = $1, is_income = $2, is_active = $3 WHERE id = $4")
                .bind(name)
                .bind(is_income)
                .bind(is_active)
                .bind(id)
                .execute(&mut *self.conn)
                .await?;
             if result.rows_affected() == 0 { return Err(RepositoryError::NotFound); }
        }

        Ok(())
    }

    pub async fn delete(&mut self, id: i64) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(&mut *self.conn)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}