use actix_web::{post, web, HttpResponse, Result};
use crate::{dto::CreateMemoDto, error::AppError};
use validator::Validate;

#[post("/test/dto/create")]
pub async fn test_create_dto(
    dto: web::Json<CreateMemoDto>,
) -> Result<HttpResponse, AppError> {
    dto.validate()
        .map_err(|e| AppError::Validation(format!("{}", e)))?;

    tracing::info!(
        title = %dto.title,
        description = ?dto.description,
        date_to = %dto.date_to,
        "Valid CreateMemoDto received"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Valid memo DTO received",
        "title": dto.title,
        "description": dto.description,
        "date_to": dto.date_to
    })))
}