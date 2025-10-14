use utoipa::OpenApi;

use crate::{
    dto::{CreateMemoDto, MemoResponseDto, PaginatedMemoResponse, PatchMemoDto, UpdateMemoDto},
    error::ErrorResponse,
    handlers::{health, memos},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Memos API",
        version = "0.1.0",
        description = "A RESTful API for managing memos with full CRUD operations, pagination, filtering, and sorting. Includes observability endpoints for health checks and metrics.",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
        )
    ),
    paths(
        memos::list_memos,
        memos::get_memo,
        memos::create_memo,
        memos::update_memo,
        memos::patch_memo,
        memos::delete_memo,
        memos::toggle_complete,
        health::health,
        health::ready,
    ),
    components(
        schemas(
            MemoResponseDto,
            CreateMemoDto,
            UpdateMemoDto,
            PatchMemoDto,
            PaginatedMemoResponse,
            ErrorResponse,
            health::HealthResponse,
            health::ReadyResponse,
        )
    ),
    tags(
        (name = "memos", description = "Memo management endpoints"),
        (name = "Observability", description = "Health checks and monitoring endpoints. Metrics available at /metrics endpoint (Prometheus format).")
    )
)]
pub struct ApiDoc;
