pub mod health;
pub mod memos;
pub mod metrics;
pub mod test_dto;
pub mod test_errors;
pub mod test_repository;
pub mod test_service;
pub mod web;

pub use health::{health as health_check, ready};
pub use memos::{
    create_memo, delete_memo, get_memo, list_memos, patch_memo, toggle_complete, update_memo,
};
pub use metrics::metrics as metrics_endpoint;
pub use test_dto::test_create_dto;
pub use test_errors::{test_database, test_internal, test_not_found, test_validation};
pub use test_repository::test_repository as test_repo;
pub use test_service::test_service as test_svc;
pub use web::{
    create_memo_web, delete_memo_web, get_edit_memo_form, get_memos_list, get_new_memo_form, index,
    toggle_memo_complete_web, update_memo_web,
};
