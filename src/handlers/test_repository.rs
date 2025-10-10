use crate::{repository::MemoRepository, state::AppState};
use actix_web::{HttpResponse, Responder, get, web};
use chrono::Utc;
use serde_json::json;

#[get("/test/repository")]
pub async fn test_repository(state: web::Data<AppState>) -> impl Responder {
    tracing::info!("Testing repository layer");

    let test_date = Utc::now();

    tracing::debug!("Step 1: Creating a test memo");
    let created_memo = match MemoRepository::create(
        &state.db,
        "Test Repository Memo".to_string(),
        Some("Testing repository CRUD operations".to_string()),
        test_date,
    )
    .await
    {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, "Memo created successfully");
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

    tracing::debug!("Step 2: Finding memo by ID");
    let found_memo = match MemoRepository::find_by_id(&state.db, created_memo.id).await {
        Ok(Some(memo)) => {
            tracing::info!("Memo found by ID");
            memo
        }
        Ok(None) => {
            tracing::error!("Memo not found after creation");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Memo not found after creation"
            }));
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to find memo");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to find memo",
                "details": e.to_string()
            }));
        }
    };

    tracing::debug!("Step 3: Updating the memo");
    let updated_memo = match MemoRepository::update(
        &state.db,
        created_memo.id,
        "Updated Test Memo".to_string(),
        Some("Updated description".to_string()),
        test_date,
        true,
    )
    .await
    {
        Ok(memo) => {
            tracing::info!("Memo updated successfully");
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

    tracing::debug!("Step 4: Finding all memos");
    let (all_memos, total) =
        match MemoRepository::find_all(&state.db, 10, 0, None, "created_at", "desc").await {
            Ok(result) => {
                tracing::info!(
                    count = result.0.len(),
                    total = result.1,
                    "Fetched all memos"
                );
                result
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to find all memos");
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to find all memos",
                    "details": e.to_string()
                }));
            }
        };

    tracing::debug!("Step 5: Finding completed memos only");
    let (completed_memos, completed_total) =
        match MemoRepository::find_all(&state.db, 10, 0, Some(true), "created_at", "desc").await {
            Ok(result) => {
                tracing::info!(
                    count = result.0.len(),
                    total = result.1,
                    "Fetched completed memos"
                );
                result
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to find completed memos");
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to find completed memos",
                    "details": e.to_string()
                }));
            }
        };

    tracing::debug!("Step 6: Deleting the test memo");
    let deleted = match MemoRepository::delete(&state.db, created_memo.id).await {
        Ok(result) => {
            tracing::info!(deleted = result, "Delete operation completed");
            result
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete memo");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to delete memo",
                "details": e.to_string()
            }));
        }
    };

    tracing::info!("Repository test completed successfully");

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Repository layer test completed",
        "results": {
            "create": {
                "id": created_memo.id,
                "title": created_memo.title,
                "completed": created_memo.completed
            },
            "find_by_id": {
                "found": true,
                "title": found_memo.title
            },
            "update": {
                "title": updated_memo.title,
                "description": updated_memo.description,
                "completed": updated_memo.completed
            },
            "find_all": {
                "count": all_memos.len(),
                "total": total
            },
            "find_completed": {
                "count": completed_memos.len(),
                "total": completed_total
            },
            "delete": {
                "success": deleted
            }
        }
    }))
}
