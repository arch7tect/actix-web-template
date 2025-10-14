use crate::{error::AppError, state::AppState};
use actix_web::{HttpResponse, Result, get, web};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service health status
    status: String,
    /// Database connection status
    database: String,
    /// Application version
    version: String,
    /// Service uptime in seconds
    uptime_seconds: u64,
}

#[derive(Serialize, ToSchema)]
pub struct ReadyResponse {
    /// Service readiness status
    ready: bool,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Observability",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
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

#[utoipa::path(
    get,
    path = "/ready",
    tag = "Observability",
    responses(
        (status = 200, description = "Service is ready", body = ReadyResponse),
        (status = 503, description = "Service is not ready", body = ReadyResponse),
    )
)]
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
