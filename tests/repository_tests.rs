mod common;

use actix_web_template::repository::MemoRepository;
use chrono::Utc;
use common::{fixtures::create_test_memo_dto, setup_test_db};

#[tokio::test]
async fn test_repository_create() {
    let db = setup_test_db().await;
    let dto = create_test_memo_dto("Repository Test", Some("Test desc"));

    let result =
        MemoRepository::create(&db, dto.title.clone(), dto.description.clone(), dto.date_to).await;

    assert!(result.is_ok());
    let memo = result.unwrap();
    assert_eq!(memo.title, "Repository Test");
    assert_eq!(memo.description, Some("Test desc".to_string()));
    assert!(!memo.completed);

    MemoRepository::delete(&db, memo.id).await.ok();
}

#[tokio::test]
async fn test_repository_find_by_id() {
    let db = setup_test_db().await;
    let dto = create_test_memo_dto("Find By ID Test", None);

    let created =
        MemoRepository::create(&db, dto.title.clone(), dto.description.clone(), dto.date_to)
            .await
            .unwrap();

    let result = MemoRepository::find_by_id(&db, created.id).await;
    assert!(result.is_ok());

    let found = result.unwrap();
    assert!(found.is_some());
    let memo = found.unwrap();
    assert_eq!(memo.id, created.id);
    assert_eq!(memo.title, "Find By ID Test");

    MemoRepository::delete(&db, created.id).await.ok();
}

#[tokio::test]
async fn test_repository_find_by_id_not_found() {
    let db = setup_test_db().await;
    let fake_id = uuid::Uuid::new_v4();

    let result = MemoRepository::find_by_id(&db, fake_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_repository_update() {
    let db = setup_test_db().await;
    let dto = create_test_memo_dto("Original", Some("Original desc"));

    let created =
        MemoRepository::create(&db, dto.title.clone(), dto.description.clone(), dto.date_to)
            .await
            .unwrap();

    let result = MemoRepository::update(
        &db,
        created.id,
        "Updated".to_string(),
        Some("Updated desc".to_string()),
        Utc::now(),
        true,
    )
    .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.description, Some("Updated desc".to_string()));
    assert!(updated.completed);

    MemoRepository::delete(&db, created.id).await.ok();
}

#[tokio::test]
async fn test_repository_update_not_found() {
    let db = setup_test_db().await;
    let fake_id = uuid::Uuid::new_v4();

    let result =
        MemoRepository::update(&db, fake_id, "Test".to_string(), None, Utc::now(), false).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_repository_delete() {
    let db = setup_test_db().await;
    let dto = create_test_memo_dto("To Delete", None);

    let created =
        MemoRepository::create(&db, dto.title.clone(), dto.description.clone(), dto.date_to)
            .await
            .unwrap();

    let result = MemoRepository::delete(&db, created.id).await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    let found = MemoRepository::find_by_id(&db, created.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_repository_delete_not_found() {
    let db = setup_test_db().await;
    let fake_id = uuid::Uuid::new_v4();

    let result = MemoRepository::delete(&db, fake_id).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_repository_find_all_basic() {
    let db = setup_test_db().await;

    let dto1 = create_test_memo_dto("Repo List 1", None);
    let memo1 = MemoRepository::create(
        &db,
        dto1.title.clone(),
        dto1.description.clone(),
        dto1.date_to,
    )
    .await
    .unwrap();

    let dto2 = create_test_memo_dto("Repo List 2", None);
    let memo2 = MemoRepository::create(
        &db,
        dto2.title.clone(),
        dto2.description.clone(),
        dto2.date_to,
    )
    .await
    .unwrap();

    let result = MemoRepository::find_all(&db, 10, 0, None, "created_at", "desc").await;
    assert!(result.is_ok());

    let (memos, total) = result.unwrap();
    assert!(total >= 2);
    assert!(!memos.is_empty());

    MemoRepository::delete(&db, memo1.id).await.ok();
    MemoRepository::delete(&db, memo2.id).await.ok();
}

#[tokio::test]
async fn test_repository_find_all_with_pagination() {
    let db = setup_test_db().await;

    let mut ids = Vec::new();
    for i in 0..5 {
        let dto = create_test_memo_dto(&format!("Pagination {}", i), None);
        let memo = MemoRepository::create(&db, dto.title, dto.description, dto.date_to)
            .await
            .unwrap();
        ids.push(memo.id);
    }

    let result = MemoRepository::find_all(&db, 2, 0, None, "created_at", "desc").await;
    assert!(result.is_ok());

    let (memos, _) = result.unwrap();
    assert!(memos.len() <= 2);

    for id in ids {
        MemoRepository::delete(&db, id).await.ok();
    }
}

#[tokio::test]
async fn test_repository_find_all_with_completed_filter() {
    let db = setup_test_db().await;

    let dto = create_test_memo_dto("Completed Memo", None);
    let created = MemoRepository::create(&db, dto.title, dto.description, dto.date_to)
        .await
        .unwrap();

    MemoRepository::update(
        &db,
        created.id,
        created.title.clone(),
        None,
        created.date_to.with_timezone(&Utc),
        true,
    )
    .await
    .unwrap();

    let result = MemoRepository::find_all(&db, 10, 0, Some(true), "created_at", "desc").await;
    assert!(result.is_ok());

    let (memos, total) = result.unwrap();
    assert!(total >= 1);
    assert!(memos.iter().all(|m| m.completed));

    MemoRepository::delete(&db, created.id).await.ok();
}

#[tokio::test]
async fn test_repository_find_all_sorting() {
    let db = setup_test_db().await;

    let dto1 = create_test_memo_dto("AAA Sort Test", None);
    let memo1 = MemoRepository::create(&db, dto1.title, dto1.description, dto1.date_to)
        .await
        .unwrap();

    let dto2 = create_test_memo_dto("ZZZ Sort Test", None);
    let memo2 = MemoRepository::create(&db, dto2.title, dto2.description, dto2.date_to)
        .await
        .unwrap();

    let result_asc = MemoRepository::find_all(&db, 100, 0, None, "title", "asc").await;
    assert!(result_asc.is_ok());

    let (memos_asc, _) = result_asc.unwrap();
    let aaa_pos = memos_asc.iter().position(|m| m.id == memo1.id);
    let zzz_pos = memos_asc.iter().position(|m| m.id == memo2.id);

    if let (Some(aaa), Some(zzz)) = (aaa_pos, zzz_pos) {
        assert!(aaa < zzz);
    }

    MemoRepository::delete(&db, memo1.id).await.ok();
    MemoRepository::delete(&db, memo2.id).await.ok();
}
