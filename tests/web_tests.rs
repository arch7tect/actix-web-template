mod common;

use actix_web::{App, test, web};
use actix_web_template::{
    handlers::web::{
        create_memo_web, delete_memo_web, get_edit_memo_form, get_memos_list, get_new_memo_form,
        index, toggle_memo_complete_web, update_memo_web,
    },
    services::MemoService,
};
use chrono::Utc;
use common::{fixtures::create_test_memo_dto, setup_test_state};

#[tokio::test]
async fn test_index_page() {
    let state = setup_test_state().await;

    let app = test::init_service(App::new().app_data(web::Data::new(state)).service(index)).await;

    let req = test::TestRequest::get().uri("/").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("<!DOCTYPE html") || html.contains("<html"));
}

#[tokio::test]
async fn test_get_memos_list() {
    let state = setup_test_state().await;

    let service = MemoService::new(state.db.clone());
    let dto = create_test_memo_dto("Web List Test", None);
    let created = service.create_memo(dto).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(get_memos_list),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/web/memos?limit=10&offset=0")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Web List Test"));

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_get_new_memo_form() {
    let app = test::init_service(App::new().service(get_new_memo_form)).await;

    let req = test::TestRequest::get().uri("/web/memos/new").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("<form") || html.contains("form"));
}

#[tokio::test]
async fn test_create_memo_web() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(create_memo_web),
    )
    .await;

    let date_str = Utc::now().format("%Y-%m-%dT%H:%M").to_string();

    let req = test::TestRequest::post()
        .uri("/web/memos")
        .set_form([
            ("title", "Web Created Memo"),
            ("description", "Test description"),
            ("date_to", &date_str),
        ])
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let params = actix_web_template::dto::PaginationParams::default();
    let result = service.get_all_memos(params).await.unwrap();
    let created = result
        .data
        .iter()
        .find(|m| m.title == "Web Created Memo")
        .expect("Memo should exist");

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_get_edit_memo_form() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    let dto = create_test_memo_dto("Edit Form Test", None);
    let created = service.create_memo(dto).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(get_edit_memo_form),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/web/memos/{}/edit", created.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Edit Form Test"));

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_update_memo_web() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    let dto = create_test_memo_dto("Original Web Title", None);
    let created = service.create_memo(dto).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(update_memo_web),
    )
    .await;

    let date_str = Utc::now().format("%Y-%m-%dT%H:%M").to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/web/memos/{}", created.id))
        .set_form([
            ("title", "Updated Web Title"),
            ("description", "Updated description"),
            ("date_to", &date_str),
            ("completed", "on"),
        ])
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let updated = service.get_memo_by_id(created.id).await.unwrap();
    assert_eq!(updated.title, "Updated Web Title");
    assert!(updated.completed);

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_delete_memo_web() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    let dto = create_test_memo_dto("To Delete Web", None);
    let created = service.create_memo(dto).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(delete_memo_web),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/web/memos/{}", created.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result = service.get_memo_by_id(created.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_toggle_memo_complete_web() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    let dto = create_test_memo_dto("Toggle Web Test", None);
    let created = service.create_memo(dto).await.unwrap();
    assert!(!created.completed);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(toggle_memo_complete_web),
    )
    .await;

    let req = test::TestRequest::patch()
        .uri(&format!("/web/memos/{}/toggle", created.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let toggled = service.get_memo_by_id(created.id).await.unwrap();
    assert!(toggled.completed);

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_create_memo_web_validation_error() {
    let state = setup_test_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(create_memo_web),
    )
    .await;

    let date_str = Utc::now().format("%Y-%m-%dT%H:%M").to_string();

    let req = test::TestRequest::post()
        .uri("/web/memos")
        .set_form([("title", ""), ("date_to", &date_str)])
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}
