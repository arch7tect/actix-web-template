use utoipa::OpenApi;

use crate::{
    dto::{CreateMemoDto, MemoResponseDto, PaginatedMemoResponse, PatchMemoDto, UpdateMemoDto},
    error::ErrorResponse,
    handlers::{memos, metrics},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Memos API",
        version = "0.1.0",
        description = "A RESTful API for managing memos with full CRUD operations, pagination, filtering, and sorting",
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
        metrics::metrics,
    ),
    components(
        schemas(
            MemoResponseDto,
            CreateMemoDto,
            UpdateMemoDto,
            PatchMemoDto,
            PaginatedMemoResponse,
            ErrorResponse
        )
    ),
    tags(
        (name = "memos", description = "Memo management endpoints"),
        (name = "Observability", description = "Observability and monitoring endpoints")
    )
)]
pub struct ApiDoc;
