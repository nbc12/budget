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

    pub async fn find_by_id(&mut self, id: i64) -> Result<Option<Card>, RepositoryError> {
        let record = sqlx::query_as::<_, CardRecord>(
            "SELECT id, name, is_active FROM cards WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *self.conn)
        .await?;

        Ok(record.map(|r| r.into()))
    }

    pub async fn delete(&mut self, id: i64) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM cards WHERE id = $1")
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

    #[tokio::test]
    async fn test_create_card() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let mut repo = CardRepository::new(uow.connection());
        
        let req = CreateCardRequest { name: "Test Card".to_string() };
        let id = repo.create(&req).await.unwrap();
        assert!(id > 0);
        
        let card = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(card.name, "Test Card");
        assert!(card.is_active);
    }

    #[tokio::test]
    async fn test_read_cards() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let mut repo = CardRepository::new(uow.connection());
        
        let initial_count = repo.list().await.unwrap().len();
        
        repo.create(&CreateCardRequest { name: "Card 1".to_string() }).await.unwrap();
        repo.create(&CreateCardRequest { name: "Card 2".to_string() }).await.unwrap();
        
        let cards = repo.list().await.unwrap();
        assert_eq!(cards.len(), initial_count + 2);
    }

    #[tokio::test]
    async fn test_list_active_cards() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let mut repo = CardRepository::new(uow.connection());
        
        let id1 = repo.create(&CreateCardRequest { name: "Active Card".to_string() }).await.unwrap();
        let id2 = repo.create(&CreateCardRequest { name: "Inactive Card".to_string() }).await.unwrap();
        
        repo.update(id2, &UpdateCardRequest { name: "Inactive Card".to_string(), is_active: false }).await.unwrap();
        
        let active_cards = repo.list_active().await.unwrap();
        assert!(active_cards.iter().any(|c| c.id == id1));
        assert!(!active_cards.iter().any(|c| c.id == id2));
    }

    #[tokio::test]
    async fn test_update_card() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let mut repo = CardRepository::new(uow.connection());
        
        let id = repo.create(&CreateCardRequest { name: "Original Name".to_string() }).await.unwrap();
        
        let update_req = UpdateCardRequest {
            name: "Updated Name".to_string(),
            is_active: false,
        };
        repo.update(id, &update_req).await.unwrap();
        
        let card = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(card.name, "Updated Name");
        assert!(!card.is_active);
    }

    #[tokio::test]
    async fn test_delete_card() {
        let db = get_test_db().await;
        let mut uow = db.begin().await.unwrap();
        let mut repo = CardRepository::new(uow.connection());
        
        let id = repo.create(&CreateCardRequest { name: "To Be Deleted".to_string() }).await.unwrap();
        assert!(repo.find_by_id(id).await.unwrap().is_some());
        
        repo.delete(id).await.unwrap();
        assert!(repo.find_by_id(id).await.unwrap().is_none());
    }
}
