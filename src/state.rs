use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub config: Settings,
}

impl AppState {
    pub fn new(config: Settings) -> Self {
        tracing::debug!("Creating application state");
        Self { config }
    }
}