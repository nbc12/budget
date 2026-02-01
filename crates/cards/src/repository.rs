use crate::models::{Card, CreateCardRequest, UpdateCardRequest};
use database::{self, RepositoryError};
use sqlx::FromRow;

#[derive(FromRow)]
struct CardRecord {
    id: i64,
    name: String,
    is_active: bool,
}

impl From<CardRecord> for Card {
    fn from(record: CardRecord) -> Self {
        Card {
            id: record.id,
            name: record.name,
            is_active: record.is_active,
        }
    }
}

pub(crate) struct CardRepository<'a> {
    conn: &'a mut database::Connection,
}

impl<'a> CardRepository<'a> {
    pub fn new(conn: &'a mut database::Connection) -> Self {
        Self { conn }
    }

    pub async fn create(&mut self, req: &CreateCardRequest) -> Result<i64, RepositoryError> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO cards (name) VALUES ($1) RETURNING id",
        )
        .bind(&req.name)
        .fetch_one(&mut *self.conn)
        .await?;
        
        Ok(id)
    }

    pub async fn list(&mut self) -> Result<Vec<Card>, RepositoryError> {
        let records = sqlx::query_as::<_, CardRecord>(
            "SELECT id, name, is_active FROM cards ORDER BY name",
        )
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_active(&mut self) -> Result<Vec<Card>, RepositoryError> {
        let records = sqlx::query_as::<_, CardRecord>(
            "SELECT id, name, is_active FROM cards WHERE is_active = 1 ORDER BY name",
        )
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update(&mut self, id: i64, req: &UpdateCardRequest) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            "UPDATE cards SET name = $1, is_active = $2 WHERE id = $3",
        )
        .bind(&req.name)
        .bind(req.is_active)
        .bind(id)
        .execute(&mut *self.conn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
