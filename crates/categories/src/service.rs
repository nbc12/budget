use crate::models::{Category, CreateCategoryRequest, CreateMonthlyBudgetRequest, CategoryBudgetView};
use crate::repository::CategoryRepository;
use crate::budget_repository::MonthlyBudgetRepository;
use database::{RepositoryError, Database};
use tracing::instrument;
use rand::seq::SliceRandom;

#[derive(Debug, thiserror::Error)]
pub enum CategoryError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Database error: {0}")]
    Infrastructure(String),
    #[error("Category already exists: {0}")]
    Conflict(String),
    #[error("Category not found")]
    NotFound,
}

impl From<RepositoryError> for CategoryError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => CategoryError::NotFound,
            RepositoryError::UniqueViolation(msg) => CategoryError::Conflict(msg),
            RepositoryError::Infrastructure(e) => CategoryError::Infrastructure(e.to_string()),
            _ => CategoryError::Infrastructure(err.to_string()),
        }
    }
}

pub struct CategoryService;

impl CategoryService {
    fn get_random_pastel_color() -> String {
        let colors = vec![
            "#FFB3BA", "#FFDFBA", "#FFFFBA", "#BAFFC9", "#BAE1FF", 
            "#E2F0CB", "#FDFD96", "#FFC3A0", "#FFD1DC", "#D4F0F0",
            "#CCE2CB", "#B6CFB6", "#97C1A9", "#FCB7AF", "#FFDAC1",
            "#E7FFAC", "#FFABAB", "#D5AAFF", "#85E3FF", "#B9F6CA"
        ];
        let mut rng = rand::thread_rng();
        colors.choose(&mut rng).unwrap_or(&"#FFFFFF").to_string()
    }

    #[instrument(skip(db))]
    pub async fn create_category(
        db: &Database,
        name: String,
        is_income: bool,
    ) -> Result<i64, CategoryError> {
        let color = Self::get_random_pastel_color();
        let mut req = CreateCategoryRequest::new(name, color, is_income)
            .map_err(CategoryError::InvalidInput)?;
        req.is_active = true;
            
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CategoryRepository::new(uow.connection());
        
        let id = repo.create(&req).await?;
        
        uow.commit().await.map_err(RepositoryError::from)?;
        
        Ok(id)
    }

    #[instrument(skip(db))]
    pub async fn update_category(
        db: &Database,
        id: i64,
        name: String,
        color: Option<String>,
        is_income: bool,
        is_active: bool,
    ) -> Result<(), CategoryError> {
        if name.trim().is_empty() {
            return Err(CategoryError::InvalidInput("Category name cannot be empty".into()));
        }

        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CategoryRepository::new(uow.connection());
        
        repo.update(id, name.trim(), color.as_deref(), is_income, is_active).await?;
        
        uow.commit().await.map_err(RepositoryError::from)?;
        Ok(())
    }

    #[instrument(skip(db))]
    pub async fn delete_category(
        db: &Database,
        id: i64,
    ) -> Result<(), CategoryError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CategoryRepository::new(uow.connection());
        
        repo.delete(id).await?;
        
        uow.commit().await.map_err(RepositoryError::from)?;
        Ok(())
    }

    #[instrument(skip(db))]
    pub async fn set_monthly_limit(
        db: &Database,
        category_id: i64,
        month: String,
        limit_dollars: f64,
    ) -> Result<(), CategoryError> {
        let req = CreateMonthlyBudgetRequest::new(category_id, month, limit_dollars)
            .map_err(CategoryError::InvalidInput)?;

        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = MonthlyBudgetRepository::new(uow.connection());

        repo.upsert(&req).await?;
        uow.commit().await.map_err(RepositoryError::from)?;
        Ok(())
    }

    #[instrument(skip(db))]
    pub async fn list_categories(db: &Database) -> Result<Vec<Category>, CategoryError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CategoryRepository::new(uow.connection());
        
        let categories = repo.list().await?;
        
        Ok(categories)
    }

    #[instrument(skip(db))]
    pub async fn get_category(db: &Database, id: i64) -> Result<Category, CategoryError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = CategoryRepository::new(uow.connection());
        
        let category = repo.find_by_id(id).await?
            .ok_or(CategoryError::NotFound)?;
            
        Ok(category)
    }

    #[instrument(skip(db))]
    pub async fn get_budget_view(db: &Database, month: &str) -> Result<Vec<CategoryBudgetView>, CategoryError> {
        tracing::info!("get_budget_view called for month: {}", month);
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        
        // 1. Get all categories
        let mut cat_repo = CategoryRepository::new(uow.connection());
        let categories = cat_repo.list().await.map_err(|e| {
            tracing::error!("Failed to list categories: {}", e);
            CategoryError::from(e)
        })?;

        // 2. Get budgets for this month
        let mut budget_repo = MonthlyBudgetRepository::new(uow.connection());
        let budgets = budget_repo.get_for_month(month).await.map_err(|e| {
            tracing::error!("Failed to get budgets for month: {}", e);
            CategoryError::from(e)
        })?;

        // 3. Build View (Merging)
        let mut views = Vec::new();
        for cat in categories {
            let budget = budgets.iter().find(|b| b.category_id == cat.id).cloned();
            
            // Only include if active OR has a budget for this month
            if cat.is_active || budget.is_some() {
                views.push(CategoryBudgetView {
                    category: cat,
                    budget,
                    spent: 0, // Placeholder
                    remaining: 0, // Placeholder
                });
            }
        }

        Ok(views)
    }

    #[instrument(skip(db))]
    pub async fn ensure_budgets_exist(db: &Database, current_month: &str, previous_month: &str) -> Result<(), CategoryError> {
        let mut uow = db.begin().await.map_err(RepositoryError::from)?;
        let mut repo = MonthlyBudgetRepository::new(uow.connection());
        
        repo.copy_budgets(previous_month, current_month).await?;
        
        uow.commit().await.map_err(RepositoryError::from)?;
        Ok(())
    }
}