use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use sqlx::{Transaction, Sqlite};
use std::str::FromStr;

pub use sqlx::Error;
pub use sqlx::Result;

// --- Driver Adapter Pattern ---
pub type Driver = Sqlite;
pub type Connection = sqlx::SqliteConnection;
pub type Pool = SqlitePool;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Infrastructure(sqlx::Error),
    #[error("Resource not found")]
    NotFound,
    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),
    #[error("Check constraint violation: {0}")]
    CheckViolation(String),
}

impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            _ => {
                if let Some(db_err) = err.as_database_error() {
                    if let Some(code) = db_err.code() {
                        match code.as_ref() {
                            "2067" | "1555" => {
                                return RepositoryError::UniqueViolation(
                                    db_err.message().to_string(),
                                );
                            }
                            "275" => {
                                return RepositoryError::CheckViolation(
                                    db_err.message().to_string(),
                                );
                            }
                            _ => {}
                        }
                    }
                }
                RepositoryError::Infrastructure(err)
            }
        }
    }
}

#[derive(Clone)]
pub struct Database {
    pub pool: Pool,
}

impl Database {
    pub async fn new(connection_string: &str) -> sqlx::Result<Self> {
        let options = SqliteConnectOptions::from_str(connection_string)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await?;
        
        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running migrations...");
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        println!("Migrations complete.");
        Ok(())
    }

    pub async fn begin(&self) -> Result<UnitOfWork<'_>, RepositoryError> {
        let tx = self.pool.begin().await?;
        Ok(UnitOfWork { tx })
    }
}

pub struct UnitOfWork<'a> {
    tx: Transaction<'a, Driver>,
}

impl<'a> UnitOfWork<'a> {
    pub async fn commit(self) -> Result<(), RepositoryError> {
        self.tx.commit().await?;
        Ok(())
    }

    pub fn connection(&mut self) -> &mut Connection {
        &mut *self.tx
    }
}

#[cfg(test)]
pub async fn get_test_db() -> Database {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true)
        .shared_cache(true);
        
    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .expect("Failed to create test database pool");

    let db = Database { pool };
    db.run_migrations().await.expect("Failed to run migrations");
    
    db
}
