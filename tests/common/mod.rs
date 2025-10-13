pub mod fixtures;

use actix_web_template::{config::Settings, state::AppState};
use sea_orm::Database;

pub async fn setup_test_db() -> sea_orm::DatabaseConnection {
    let settings = Settings::load().expect("Failed to load settings");
    Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to test database")
}

#[allow(dead_code)]
pub async fn setup_test_state() -> AppState {
    let settings = Settings::load().expect("Failed to load settings");
    let db = setup_test_db().await;
    AppState::new(settings, db)
}
