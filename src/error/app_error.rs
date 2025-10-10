use actix_web::{HttpResponse, error::ResponseError, http::StatusCode};
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
}

impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        AppError::Validation(format!("Validation failed: {}", err))
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Validation(err)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_type = match self {
            AppError::Database(_) => "DatabaseError",
            AppError::NotFound(_) => "NotFound",
            AppError::Validation(_) => "ValidationError",
            AppError::Internal(_) => "InternalError",
        };

        tracing::error!(
            error_type = error_type,
            status_code = status.as_u16(),
            message = %self,
            "Error occurred"
        );

        HttpResponse::build(status).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status: status.as_u16(),
        })
    }
}
