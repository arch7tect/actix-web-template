use crate::config::Settings;
use sea_orm::DatabaseConnection;
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub config: Settings,
    pub db: DatabaseConnection,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(config: Settings, db: DatabaseConnection) -> Self {
        tracing::debug!("Creating application state");
        Self {
            config,
            db,
            start_time: Instant::now(),
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
