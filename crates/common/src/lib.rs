use clap::Parser;
use database::Database;

pub mod auth;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:budget.db")]
    pub database_url: String,

    #[arg(long, env = "PORT", default_value = "3000")]
    pub port: u16,

    #[arg(long, env = "APP_PASSWORD")]
    pub app_password: Option<String>,
}
