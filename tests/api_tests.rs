use actix_web::{test, web, App};
use actix_web_template::{
    config::Settings,
    dto::{CreateMemoDto, PaginatedResponse, PatchMemoDto, UpdateMemoDto, MemoResponseDto},
    handlers, state::AppState,
};
use chrono::Utc;
use sea_orm::Database;

#[tokio::test]
async fn test_create_memo_endpoint() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::delete_memo),
    )
    .await;

    let create_dto = CreateMemoDto {
        title: "Test API Memo".to_string(),
        description: Some("Created via API test".to_string()),
        date_to: Utc::now(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let memo: MemoResponseDto = test::read_body_json(resp).await;
    assert_eq!(memo.title, "Test API Memo");
    assert_eq!(memo.description, Some("Created via API test".to_string()));
    assert!(!memo.completed);

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", memo.id))
        .to_request();
    test::call_service(&app, delete_req).await;
}

#[tokio::test]
async fn test_get_memo_endpoint() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::get_memo)
            .service(handlers::delete_memo),
    )
    .await;

    let create_dto = CreateMemoDto {
        title: "Get Test Memo".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let create_req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_memo: MemoResponseDto = test::read_body_json(create_resp).await;

    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let memo: MemoResponseDto = test::read_body_json(get_resp).await;
    assert_eq!(memo.id, created_memo.id);
    assert_eq!(memo.title, "Get Test Memo");

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", memo.id))
        .to_request();
    test::call_service(&app, delete_req).await;
}

#[tokio::test]
async fn test_get_memo_not_found() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings, db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(handlers::get_memo),
    )
    .await;

    let fake_id = uuid::Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/memos/{}", fake_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_update_memo_endpoint() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::update_memo)
            .service(handlers::delete_memo),
    )
    .await;

    let create_dto = CreateMemoDto {
        title: "Original Title".to_string(),
        description: Some("Original description".to_string()),
        date_to: Utc::now(),
    };

    let create_req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_memo: MemoResponseDto = test::read_body_json(create_resp).await;

    let update_dto = UpdateMemoDto {
        title: "Updated Title".to_string(),
        description: Some("Updated description".to_string()),
        date_to: Utc::now(),
        completed: true,
    };

    let update_req = test::TestRequest::put()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .set_json(&update_dto)
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), 200);

    let updated_memo: MemoResponseDto = test::read_body_json(update_resp).await;
    assert_eq!(updated_memo.title, "Updated Title");
    assert_eq!(
        updated_memo.description,
        Some("Updated description".to_string())
    );
    assert!(updated_memo.completed);

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .to_request();
    test::call_service(&app, delete_req).await;
}

#[tokio::test]
async fn test_patch_memo_endpoint() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::patch_memo)
            .service(handlers::delete_memo),
    )
    .await;

    let create_dto = CreateMemoDto {
        title: "Original Title".to_string(),
        description: Some("Original description".to_string()),
        date_to: Utc::now(),
    };

    let create_req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_memo: MemoResponseDto = test::read_body_json(create_resp).await;

    let patch_dto = PatchMemoDto {
        title: Some("Patched Title".to_string()),
        description: None,
        date_to: None,
        completed: None,
    };

    let patch_req = test::TestRequest::patch()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .set_json(&patch_dto)
        .to_request();

    let patch_resp = test::call_service(&app, patch_req).await;
    assert_eq!(patch_resp.status(), 200);

    let patched_memo: MemoResponseDto = test::read_body_json(patch_resp).await;
    assert_eq!(patched_memo.title, "Patched Title");
    assert_eq!(
        patched_memo.description,
        Some("Original description".to_string())
    );
    assert!(!patched_memo.completed);

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .to_request();
    test::call_service(&app, delete_req).await;
}

#[tokio::test]
async fn test_delete_memo_endpoint() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::delete_memo)
            .service(handlers::get_memo),
    )
    .await;

    let create_dto = CreateMemoDto {
        title: "To Delete".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let create_req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_memo: MemoResponseDto = test::read_body_json(create_resp).await;

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[tokio::test]
async fn test_toggle_complete_endpoint() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::toggle_complete)
            .service(handlers::delete_memo),
    )
    .await;

    let create_dto = CreateMemoDto {
        title: "Toggle Test".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let create_req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_memo: MemoResponseDto = test::read_body_json(create_resp).await;
    assert!(!created_memo.completed);

    let toggle_req = test::TestRequest::patch()
        .uri(&format!("/api/v1/memos/{}/complete", created_memo.id))
        .to_request();

    let toggle_resp = test::call_service(&app, toggle_req).await;
    assert_eq!(toggle_resp.status(), 200);

    let toggled_memo: MemoResponseDto = test::read_body_json(toggle_resp).await;
    assert!(toggled_memo.completed);

    let toggle_req2 = test::TestRequest::patch()
        .uri(&format!("/api/v1/memos/{}/complete", created_memo.id))
        .to_request();

    let toggle_resp2 = test::call_service(&app, toggle_req2).await;
    let toggled_memo2: MemoResponseDto = test::read_body_json(toggle_resp2).await;
    assert!(!toggled_memo2.completed);

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .to_request();
    test::call_service(&app, delete_req).await;
}

#[tokio::test]
async fn test_list_memos_endpoint() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::list_memos)
            .service(handlers::delete_memo),
    )
    .await;

    let create_dto1 = CreateMemoDto {
        title: "List Test 1".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let create_req1 = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto1)
        .to_request();

    let create_resp1 = test::call_service(&app, create_req1).await;
    let memo1: MemoResponseDto = test::read_body_json(create_resp1).await;

    let create_dto2 = CreateMemoDto {
        title: "List Test 2".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let create_req2 = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto2)
        .to_request();

    let create_resp2 = test::call_service(&app, create_req2).await;
    let memo2: MemoResponseDto = test::read_body_json(create_resp2).await;

    let list_req = test::TestRequest::get()
        .uri("/api/v1/memos?limit=10&offset=0&sort_by=created_at&order=desc")
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let response: PaginatedResponse<MemoResponseDto> = test::read_body_json(list_resp).await;
    assert!(response.total >= 2);
    assert!(!response.data.is_empty());

    let delete_req1 = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", memo1.id))
        .to_request();
    test::call_service(&app, delete_req1).await;

    let delete_req2 = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", memo2.id))
        .to_request();
    test::call_service(&app, delete_req2).await;
}

#[tokio::test]
async fn test_list_memos_with_pagination() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::list_memos)
            .service(handlers::delete_memo),
    )
    .await;

    let mut memo_ids = Vec::new();
    for i in 0..5 {
        let create_dto = CreateMemoDto {
            title: format!("Pagination Test {}", i),
            description: None,
            date_to: Utc::now(),
        };

        let create_req = test::TestRequest::post()
            .uri("/api/v1/memos")
            .set_json(&create_dto)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        let memo: MemoResponseDto = test::read_body_json(create_resp).await;
        memo_ids.push(memo.id);
    }

    let list_req = test::TestRequest::get()
        .uri("/api/v1/memos?limit=2&offset=0")
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let response: PaginatedResponse<MemoResponseDto> = test::read_body_json(list_resp).await;
    assert_eq!(response.limit, 2);
    assert!(response.data.len() <= 2);

    for id in memo_ids {
        let delete_req = test::TestRequest::delete()
            .uri(&format!("/api/v1/memos/{}", id))
            .to_request();
        test::call_service(&app, delete_req).await;
    }
}

#[tokio::test]
async fn test_create_memo_validation_error() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings, db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo),
    )
    .await;

    let create_dto = CreateMemoDto {
        title: "".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}