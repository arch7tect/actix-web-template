use actix_web::{get, HttpResponse};

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "Server is running"
    }))
}