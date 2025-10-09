use actix_web::{get, HttpResponse, Result};
use crate::error::AppError;

#[get("/test/error/notfound")]
pub async fn test_not_found() -> Result<HttpResponse, AppError> {
    Err(AppError::NotFound("Test resource".to_string()))
}

#[get("/test/error/validation")]
pub async fn test_validation() -> Result<HttpResponse, AppError> {
    Err(AppError::Validation("Invalid test data".to_string()))
}

#[get("/test/error/internal")]
pub async fn test_internal() -> Result<HttpResponse, AppError> {
    Err(AppError::Internal("Test internal error".to_string()))
}

#[get("/test/error/database")]
pub async fn test_database() -> Result<HttpResponse, AppError> {
    Err(AppError::Database(sea_orm::DbErr::RecordNotFound(
        "Test database error".to_string(),
    )))
}