use crate::{error::AppError, state::AppState};
use actix_web::{HttpResponse, Result, get, web};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    database: String,
    version: String,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct ReadyResponse {
    ready: bool,
}

#[get("/health")]
pub async fn health(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let db_status = match state.db.ping().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    tracing::debug!(
        database_status = db_status,
        uptime = state.uptime_seconds(),
        "Health check performed"
    );

    let response = HealthResponse {
        status: "healthy".to_string(),
        database: db_status.to_string(),
        version: state.config.app.version.clone(),
        uptime_seconds: state.uptime_seconds(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/ready")]
pub async fn ready(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let is_ready = state.db.ping().await.is_ok();

    tracing::debug!(ready = is_ready, "Readiness check performed");

    let response = ReadyResponse { ready: is_ready };

    if is_ready {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}
