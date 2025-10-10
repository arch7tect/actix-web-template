use crate::{
    dto::{CreateMemoDto, PaginationParams, PatchMemoDto, UpdateMemoDto},
    services::MemoService,
    state::AppState,
};
use actix_web::{HttpResponse, Responder, get, web};
use chrono::Utc;
use serde_json::json;

#[get("/test/service")]
pub async fn test_service(state: web::Data<AppState>) -> impl Responder {
    tracing::info!("Testing service layer");

    let service = MemoService::new(state.db.clone());

    let test_date = Utc::now();

    tracing::debug!("Step 1: Creating a memo via service");
    let create_dto = CreateMemoDto {
        title: "Service Layer Test Memo".to_string(),
        description: Some("Testing service layer operations".to_string()),
        date_to: test_date,
    };

    let created_memo = match service.create_memo(create_dto).await {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, "Memo created via service");
            memo
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create memo");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create memo",
                "details": e.to_string()
            }));
        }
    };

    tracing::debug!("Step 2: Getting memo by ID via service");
    let found_memo = match service.get_memo_by_id(created_memo.id).await {
        Ok(memo) => {
            tracing::info!("Memo found via service");
            memo
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to find memo");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to find memo",
                "details": e.to_string()
            }));
        }
    };

    tracing::debug!("Step 3: Updating memo via service");
    let update_dto = UpdateMemoDto {
        title: "Updated Service Test Memo".to_string(),
        description: Some("Updated via service layer".to_string()),
        date_to: test_date,
        completed: true,
    };

    let updated_memo = match service.update_memo(created_memo.id, update_dto).await {
        Ok(memo) => {
            tracing::info!("Memo updated via service");
            memo
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to update memo");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to update memo",
                "details": e.to_string()
            }));
        }
    };

    tracing::debug!("Step 4: Patching memo via service");
    let patch_dto = PatchMemoDto {
        title: Some("Patched Title".to_string()),
        description: None,
        date_to: None,
        completed: None,
    };

    let patched_memo = match service.patch_memo(created_memo.id, patch_dto).await {
        Ok(memo) => {
            tracing::info!("Memo patched via service");
            memo
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to patch memo");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to patch memo",
                "details": e.to_string()
            }));
        }
    };

    tracing::debug!("Step 5: Toggling completion status via service");
    let toggled_memo = match service.toggle_complete(created_memo.id).await {
        Ok(memo) => {
            tracing::info!(completed = memo.completed, "Memo toggled via service");
            memo
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to toggle memo");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to toggle memo",
                "details": e.to_string()
            }));
        }
    };

    tracing::debug!("Step 6: Getting all memos via service");
    let params = PaginationParams {
        limit: Some(10),
        offset: Some(0),
        completed: None,
        sort_by: Some("created_at".to_string()),
        order: Some("desc".to_string()),
    };

    let all_memos = match service.get_all_memos(params).await {
        Ok(response) => {
            tracing::info!(
                count = response.data.len(),
                total = response.total,
                "Fetched all memos via service"
            );
            response
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get all memos");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get all memos",
                "details": e.to_string()
            }));
        }
    };

    tracing::debug!("Step 7: Getting completed memos via service");
    let completed_params = PaginationParams {
        limit: Some(10),
        offset: Some(0),
        completed: Some(false),
        sort_by: Some("created_at".to_string()),
        order: Some("desc".to_string()),
    };

    let completed_memos = match service.get_all_memos(completed_params).await {
        Ok(response) => {
            tracing::info!(
                count = response.data.len(),
                total = response.total,
                "Fetched completed memos via service"
            );
            response
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get completed memos");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get completed memos",
                "details": e.to_string()
            }));
        }
    };

    tracing::debug!("Step 8: Deleting memo via service");
    match service.delete_memo(created_memo.id).await {
        Ok(_) => {
            tracing::info!("Memo deleted via service");
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete memo");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to delete memo",
                "details": e.to_string()
            }));
        }
    };

    tracing::info!("Service layer test completed successfully");

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Service layer test completed",
        "results": {
            "create": {
                "id": created_memo.id,
                "title": created_memo.title,
                "completed": created_memo.completed
            },
            "get_by_id": {
                "found": true,
                "title": found_memo.title
            },
            "update": {
                "title": updated_memo.title,
                "description": updated_memo.description,
                "completed": updated_memo.completed
            },
            "patch": {
                "title": patched_memo.title,
                "description": patched_memo.description
            },
            "toggle": {
                "completed": toggled_memo.completed
            },
            "get_all": {
                "count": all_memos.data.len(),
                "total": all_memos.total
            },
            "get_completed": {
                "count": completed_memos.data.len(),
                "total": completed_memos.total
            },
            "delete": {
                "success": true
            }
        }
    }))
}
