use actix_web::{
    HttpResponse, Responder, delete, error::ResponseError, get, patch, post, put, web,
};
use uuid::Uuid;

use crate::{
    dto::{
        CreateMemoDto, MemoResponseDto, PaginatedMemoResponse, PaginationParams, PatchMemoDto,
        UpdateMemoDto,
    },
    error::ErrorResponse,
    services::MemoService,
    state::AppState,
};

/// List all memos
///
/// Retrieve a paginated list of memos with optional filtering by completion status and sorting by various fields
#[utoipa::path(
    get,
    path = "/api/v1/memos",
    tag = "memos",
    params(
        ("limit" = Option<u64>, Query, description = "Number of items per page (1-100, default: 10)"),
        ("offset" = Option<u64>, Query, description = "Number of items to skip (default: 0)"),
        ("completed" = Option<bool>, Query, description = "Filter by completion status"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (created_at, title, date_to, completed, updated_at)"),
        ("order" = Option<String>, Query, description = "Sort order (asc or desc, default: desc)")
    ),
    responses(
        (status = 200, description = "List of memos retrieved successfully", body = PaginatedMemoResponse),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state, params), fields(limit, offset, completed, sort_by, order))]
#[get("/api/v1/memos")]
pub async fn list_memos(
    state: web::Data<AppState>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    tracing::debug!("Listing memos with pagination");

    let service = MemoService::new(state.db.clone());
    match service.get_all_memos(params.into_inner()).await {
        Ok(response) => {
            tracing::info!(
                count = response.data.len(),
                total = response.total,
                "Memos listed successfully"
            );
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to list memos");
            e.error_response()
        }
    }
}

/// Get a memo by ID
///
/// Retrieve a single memo by its unique identifier
#[utoipa::path(
    get,
    path = "/api/v1/memos/{id}",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    responses(
        (status = 200, description = "Memo retrieved successfully", body = MemoResponseDto),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[get("/api/v1/memos/{id}")]
pub async fn get_memo(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    tracing::debug!("Getting memo by ID");

    let service = MemoService::new(state.db.clone());
    match service.get_memo_by_id(id.into_inner()).await {
        Ok(memo) => {
            tracing::info!("Memo retrieved successfully");
            HttpResponse::Ok().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get memo");
            e.error_response()
        }
    }
}

/// Create a new memo
///
/// Create a new memo with title, optional description, and due date
#[utoipa::path(
    post,
    path = "/api/v1/memos",
    tag = "memos",
    request_body = CreateMemoDto,
    responses(
        (status = 201, description = "Memo created successfully", body = MemoResponseDto),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state, dto), fields(title = %dto.title, has_description = dto.description.is_some()))]
#[post("/api/v1/memos")]
pub async fn create_memo(
    state: web::Data<AppState>,
    dto: web::Json<CreateMemoDto>,
) -> impl Responder {
    tracing::debug!("Creating new memo");

    let service = MemoService::new(state.db.clone());
    match service.create_memo(dto.into_inner()).await {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, "Memo created successfully");
            HttpResponse::Created().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create memo");
            e.error_response()
        }
    }
}

/// Update a memo
///
/// Fully update an existing memo with all fields (title, description, due date, and completion status)
#[utoipa::path(
    put,
    path = "/api/v1/memos/{id}",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    request_body = UpdateMemoDto,
    responses(
        (status = 200, description = "Memo updated successfully", body = MemoResponseDto),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state, dto), fields(memo_id = %id, title = %dto.title, has_description = dto.description.is_some(), completed = dto.completed))]
#[put("/api/v1/memos/{id}")]
pub async fn update_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateMemoDto>,
) -> impl Responder {
    tracing::debug!("Updating memo");

    let service = MemoService::new(state.db.clone());
    match service.update_memo(id.into_inner(), dto.into_inner()).await {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, "Memo updated successfully");
            HttpResponse::Ok().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to update memo");
            e.error_response()
        }
    }
}

/// Partially update a memo
///
/// Update one or more fields of an existing memo. Only provided fields will be updated.
#[utoipa::path(
    patch,
    path = "/api/v1/memos/{id}",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    request_body = PatchMemoDto,
    responses(
        (status = 200, description = "Memo partially updated successfully", body = MemoResponseDto),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state, dto), fields(memo_id = %id))]
#[patch("/api/v1/memos/{id}")]
pub async fn patch_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    dto: web::Json<PatchMemoDto>,
) -> impl Responder {
    tracing::debug!("Patching memo");

    let service = MemoService::new(state.db.clone());
    match service.patch_memo(id.into_inner(), dto.into_inner()).await {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, "Memo patched successfully");
            HttpResponse::Ok().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to patch memo");
            e.error_response()
        }
    }
}

/// Delete a memo
///
/// Permanently delete a memo by its ID
#[utoipa::path(
    delete,
    path = "/api/v1/memos/{id}",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    responses(
        (status = 204, description = "Memo deleted successfully"),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[delete("/api/v1/memos/{id}")]
pub async fn delete_memo(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    tracing::debug!("Deleting memo");

    let service = MemoService::new(state.db.clone());
    match service.delete_memo(id.into_inner()).await {
        Ok(()) => {
            tracing::info!("Memo deleted successfully");
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete memo");
            e.error_response()
        }
    }
}

/// Toggle memo completion
///
/// Toggle the completion status of a memo (completed ↔ incomplete)
#[utoipa::path(
    patch,
    path = "/api/v1/memos/{id}/complete",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    responses(
        (status = 200, description = "Memo completion status toggled successfully", body = MemoResponseDto),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[patch("/api/v1/memos/{id}/complete")]
pub async fn toggle_complete(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    tracing::debug!("Toggling memo completion status");

    let service = MemoService::new(state.db.clone());
    match service.toggle_complete(id.into_inner()).await {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, completed = memo.completed, "Memo completion toggled successfully");
            HttpResponse::Ok().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to toggle memo completion");
            e.error_response()
        }
    }
}
