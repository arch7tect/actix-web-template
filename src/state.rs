use crate::config::Settings;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub config: Settings,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(config: Settings, db: DatabaseConnection) -> Self {
        tracing::debug!("Creating application state");
        Self { config, db }
    }
}