use actix_web::{delete, error::ResponseError, get, patch, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::{
    dto::{CreateMemoDto, PaginationParams, PatchMemoDto, UpdateMemoDto},
    services::MemoService,
    state::AppState,
};

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
            tracing::info!(count = response.data.len(), total = response.total, "Memos listed successfully");
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to list memos");
            e.error_response()
        }
    }
}

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