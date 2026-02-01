use crate::models::{Card, CreateCardRequest, UpdateCardRequest};
use crate::repository::CardRepository;
use database::{RepositoryError, Database};
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum CardError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Database error: {0}")]
    Infrastructure(String),
    #[error("Card not found")]
    NotFound,
    #[error("Card name already exists")]
    Conflict(String),
}

impl From<RepositoryError> for CardError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => CardError::NotFound,
            RepositoryError::UniqueViolation(msg) => CardError::Conflict(msg),
            RepositoryError::Infrastructure(e) => CardError::Infrastructure(e.to_string()),
            _ => CardError::Infrastructure(err.to_string()),
        }
    }
}

pub struct CardService;

impl CardService {
    #[instrument(skip(db))]
    pub async fn create_card(db: &Database, name: String) -> Result<i64, CardError> {
        if name.trim().is_empty() {
            return Err(CardError::InvalidInput("Card name cannot be empty".into()));
        }

        let req = CreateCardRequest { name: name.trim().to_string() };
        
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CardRepository::new(uow.connection());
        
        let id = repo.create(&req).await?;
        
        uow.commit().await.map_err(RepositoryError::from)?;
        
        Ok(id)
    }

    #[instrument(skip(db))]
    pub async fn list_cards(db: &Database) -> Result<Vec<Card>, CardError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CardRepository::new(uow.connection());
        
        let cards = repo.list().await?;
        Ok(cards)
    }

    #[instrument(skip(db))]
    pub async fn list_active_cards(db: &Database) -> Result<Vec<Card>, CardError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CardRepository::new(uow.connection());
        
        let cards = repo.list_active().await?;
        Ok(cards)
    }

    #[instrument(skip(db))]
    pub async fn update_card(db: &Database, id: i64, name: String, is_active: bool) -> Result<(), CardError> {
        if name.trim().is_empty() {
            return Err(CardError::InvalidInput("Card name cannot be empty".into()));
        }

        let req = UpdateCardRequest { name: name.trim().to_string(), is_active };
        
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CardRepository::new(uow.connection());
        
        repo.update(id, &req).await?;
        
        uow.commit().await.map_err(RepositoryError::from)?;
        Ok(())
    }
}