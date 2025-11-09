use crate::{error::AppError, state::AppState};
use actix_web::{HttpResponse, Result, get, web};
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;
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
#[tracing::instrument(name = "GET /health", skip(state))]
pub async fn health(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // Check database with 500ms timeout
    let db_check = timeout(Duration::from_millis(500), state.db.ping()).await;

    let db_status = match db_check {
        Ok(Ok(_)) => "connected",
        Ok(Err(_)) => "disconnected",
        Err(_) => {
            tracing::warn!("Database health check timeout after 500ms");
            "timeout"
        }
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
#[tracing::instrument(name = "GET /ready", skip(state))]
pub async fn ready(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // Check database with 500ms timeout
    let db_check = timeout(Duration::from_millis(500), state.db.ping()).await;
    let is_ready = matches!(db_check, Ok(Ok(_)));

    if !is_ready && db_check.is_err() {
        tracing::warn!("Database readiness check timeout after 500ms");
    }

    tracing::debug!(ready = is_ready, "Readiness check performed");

    let response = ReadyResponse { ready: is_ready };

    if is_ready {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}
