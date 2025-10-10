use actix_web::{HttpResponse, delete, get, patch, post, put, web};
use askama::Template;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::{MemoResponseDto, PaginationParams},
    error::AppError,
    services::MemoService,
    state::AppState,
};

#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    pub memos: Vec<MemoResponseDto>,
}

#[derive(Template)]
#[template(path = "components/memo_list.html")]
pub struct MemoListTemplate {
    pub memos: Vec<MemoResponseDto>,
}

#[derive(Template)]
#[template(path = "components/memo_item.html")]
pub struct MemoItemTemplate {
    pub memo: MemoResponseDto,
}

#[derive(Template)]
#[template(path = "components/memo_form.html")]
pub struct MemoFormTemplate {
    pub memo: Option<MemoResponseDto>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct WebCreateMemoForm {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub description: Option<String>,
    pub date_to: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct WebUpdateMemoForm {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub description: Option<String>,
    pub date_to: String,
    pub completed: Option<String>,
}

#[get("/")]
pub async fn index(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    tracing::debug!("Rendering index page");

    let service = MemoService::new(state.db.clone());
    let params = PaginationParams::default();

    let result = service.get_all_memos(params).await?;

    let template = IndexTemplate { memos: result.data };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render index template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}

#[get("/web/memos")]
pub async fn get_memos_list(
    state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, AppError> {
    tracing::debug!("Fetching memos list for web");

    let service = MemoService::new(state.db.clone());

    let result = service.get_all_memos(query.into_inner()).await?;

    let template = MemoListTemplate { memos: result.data };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo list template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}

#[get("/web/memos/new")]
pub async fn get_new_memo_form() -> Result<HttpResponse, AppError> {
    tracing::debug!("Rendering new memo form");

    let template = MemoFormTemplate { memo: None };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo form template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}

#[post("/web/memos")]
pub async fn create_memo_web(
    state: web::Data<AppState>,
    form: web::Form<WebCreateMemoForm>,
) -> Result<HttpResponse, AppError> {
    tracing::debug!("Creating memo from web form");

    form.validate()
        .map_err(|e| AppError::Validation(format!("Validation failed: {}", e)))?;

    let date_to: DateTime<Utc> =
        chrono::NaiveDateTime::parse_from_str(&form.date_to, "%Y-%m-%dT%H:%M")
            .map_err(|_| {
                AppError::Validation("Invalid date format. Expected YYYY-MM-DDTHH:MM".to_string())
            })?
            .and_utc();

    let service = MemoService::new(state.db.clone());

    let dto = crate::dto::CreateMemoDto {
        title: form.title.clone(),
        description: form.description.clone(),
        date_to,
    };

    let _memo = service.create_memo(dto).await?;

    let params = PaginationParams::default();
    let result = service.get_all_memos(params).await?;

    let template = MemoListTemplate { memos: result.data };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo list template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}

#[get("/web/memos/{id}/edit")]
pub async fn get_edit_memo_form(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    tracing::debug!(memo_id = %id, "Rendering edit memo form");

    let service = MemoService::new(state.db.clone());
    let memo = service.get_memo_by_id(id).await?;

    let template = MemoFormTemplate { memo: Some(memo) };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo form template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}

#[put("/web/memos/{id}")]
pub async fn update_memo_web(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    form: web::Form<WebUpdateMemoForm>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    tracing::debug!(memo_id = %id, "Updating memo from web form");

    form.validate()
        .map_err(|e| AppError::Validation(format!("Validation failed: {}", e)))?;

    let date_to: DateTime<Utc> =
        chrono::NaiveDateTime::parse_from_str(&form.date_to, "%Y-%m-%dT%H:%M")
            .map_err(|_| {
                AppError::Validation("Invalid date format. Expected YYYY-MM-DDTHH:MM".to_string())
            })?
            .and_utc();

    let completed = form.completed.is_some();

    let service = MemoService::new(state.db.clone());

    let dto = crate::dto::UpdateMemoDto {
        title: form.title.clone(),
        description: form.description.clone(),
        date_to,
        completed,
    };

    let memo = service.update_memo(id, dto).await?;

    let template = MemoItemTemplate { memo };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo item template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}

#[delete("/web/memos/{id}")]
pub async fn delete_memo_web(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    tracing::debug!(memo_id = %id, "Deleting memo from web");

    let service = MemoService::new(state.db.clone());
    service.delete_memo(id).await?;

    Ok(HttpResponse::Ok().body(""))
}

#[patch("/web/memos/{id}/toggle")]
pub async fn toggle_memo_complete_web(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    tracing::debug!(memo_id = %id, "Toggling memo completion status");

    let service = MemoService::new(state.db.clone());
    let memo = service.toggle_complete(id).await?;

    let template = MemoItemTemplate { memo };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo item template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}
