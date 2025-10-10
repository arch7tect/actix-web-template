use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateMemoDto {
    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    pub title: String,

    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,

    pub date_to: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateMemoDto {
    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    pub title: String,

    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,

    pub date_to: DateTime<Utc>,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PatchMemoDto {
    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    pub title: Option<String>,

    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,

    pub date_to: Option<DateTime<Utc>>,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoResponseDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub date_to: DateTime<Utc>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<u64>,

    #[validate(range(min = 0, message = "Offset must be non-negative"))]
    pub offset: Option<u64>,

    pub completed: Option<bool>,

    #[validate(length(max = 50, message = "Sort field must not exceed 50 characters"))]
    pub sort_by: Option<String>,

    pub order: Option<String>,
}

impl PaginationParams {
    pub fn validate_order(&self) -> Result<(), String> {
        if let Some(ref order) = self.order
            && order != "asc"
            && order != "desc"
        {
            return Err("Order must be 'asc' or 'desc'".to_string());
        }
        Ok(())
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(10),
            offset: Some(0),
            completed: None,
            sort_by: Some("created_at".to_string()),
            order: Some("desc".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: u64, limit: u64, offset: u64) -> Self {
        Self {
            data,
            total,
            limit,
            offset,
        }
    }
}
