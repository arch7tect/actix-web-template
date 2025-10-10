use actix_web::{get, web, HttpResponse};
use askama::Template;
use crate::{dto::MemoResponseDto, state::AppState};

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

#[get("/")]
pub async fn index(_state: web::Data<AppState>) -> HttpResponse {
    tracing::debug!("Rendering index page");

    let memos = vec![];

    let template = IndexTemplate { memos };

    match template.render() {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render index template");
            HttpResponse::InternalServerError().body("Template rendering error")
        }
    }
}