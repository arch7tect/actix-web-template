use crate::{dto::TagResponseDto, error::AppError, repository::TagRepository, state::AppState};
use actix_web::{HttpResponse, Result, get, web};

/// List all tags with usage counts
///
/// Returns all tags in the system sorted by usage count (descending)
#[utoipa::path(
    get,
    path = "/api/v1/tags",
    tag = "Tags",
    responses(
        (status = 200, description = "List of all tags with usage counts", body = Vec<TagResponseDto>),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/api/v1/tags")]
#[tracing::instrument(name = "GET /api/v1/tags", skip(state))]
pub async fn list_tags(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    tracing::debug!("Fetching all tags");

    let tags_with_counts = TagRepository::get_all_tags_with_counts(&state.db).await?;

    let response: Vec<TagResponseDto> = tags_with_counts
        .into_iter()
        .map(|(name, count)| TagResponseDto { name, count })
        .collect();

    tracing::info!(tag_count = response.len(), "Successfully fetched tags");

    Ok(HttpResponse::Ok().json(response))
}
