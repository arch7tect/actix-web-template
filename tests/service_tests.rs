use actix_web_template::{
    config::Settings,
    dto::{CreateMemoDto, PaginationParams, PatchMemoDto, UpdateMemoDto},
    services::MemoService,
};
use chrono::Utc;
use sea_orm::Database;

async fn setup_test_service() -> MemoService {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    MemoService::new(db)
}

#[tokio::test]
async fn test_create_memo() {
    let service = setup_test_service().await;

    let create_dto = CreateMemoDto {
        title: "Test Memo".to_string(),
        description: Some("Test description".to_string()),
        date_to: Utc::now(),
    };

    let result = service.create_memo(create_dto).await;
    assert!(result.is_ok());

    let memo = result.unwrap();
    assert_eq!(memo.title, "Test Memo");
    assert_eq!(memo.description, Some("Test description".to_string()));
    assert!(!memo.completed);

    service.delete_memo(memo.id).await.ok();
}

#[tokio::test]
async fn test_get_memo_by_id() {
    let service = setup_test_service().await;

    let create_dto = CreateMemoDto {
        title: "Test Get By ID".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();

    let result = service.get_memo_by_id(created.id).await;
    assert!(result.is_ok());

    let memo = result.unwrap();
    assert_eq!(memo.id, created.id);
    assert_eq!(memo.title, "Test Get By ID");

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_get_memo_by_id_not_found() {
    let service = setup_test_service().await;
    let fake_id = uuid::Uuid::new_v4();

    let result = service.get_memo_by_id(fake_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_memo() {
    let service = setup_test_service().await;

    let create_dto = CreateMemoDto {
        title: "Original Title".to_string(),
        description: Some("Original description".to_string()),
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();

    let update_dto = UpdateMemoDto {
        title: "Updated Title".to_string(),
        description: Some("Updated description".to_string()),
        date_to: Utc::now(),
        completed: true,
    };

    let result = service.update_memo(created.id, update_dto).await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.description, Some("Updated description".to_string()));
    assert!(updated.completed);

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_patch_memo() {
    let service = setup_test_service().await;

    let create_dto = CreateMemoDto {
        title: "Original Title".to_string(),
        description: Some("Original description".to_string()),
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();

    let patch_dto = PatchMemoDto {
        title: Some("Patched Title".to_string()),
        description: None,
        date_to: None,
        completed: None,
    };

    let result = service.patch_memo(created.id, patch_dto).await;
    assert!(result.is_ok());

    let patched = result.unwrap();
    assert_eq!(patched.title, "Patched Title");
    assert_eq!(
        patched.description,
        Some("Original description".to_string())
    );
    assert!(!patched.completed);

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_toggle_complete() {
    let service = setup_test_service().await;

    let create_dto = CreateMemoDto {
        title: "Toggle Test".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();
    assert!(!created.completed);

    let toggled = service.toggle_complete(created.id).await.unwrap();
    assert!(toggled.completed);

    let toggled_again = service.toggle_complete(created.id).await.unwrap();
    assert!(!toggled_again.completed);

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_delete_memo() {
    let service = setup_test_service().await;

    let create_dto = CreateMemoDto {
        title: "To Delete".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();

    let result = service.delete_memo(created.id).await;
    assert!(result.is_ok());

    let get_result = service.get_memo_by_id(created.id).await;
    assert!(get_result.is_err());
}

#[tokio::test]
async fn test_delete_memo_not_found() {
    let service = setup_test_service().await;
    let fake_id = uuid::Uuid::new_v4();

    let result = service.delete_memo(fake_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_all_memos() {
    let service = setup_test_service().await;

    let create_dto1 = CreateMemoDto {
        title: "Memo 1".to_string(),
        description: None,
        date_to: Utc::now(),
    };
    let memo1 = service.create_memo(create_dto1).await.unwrap();

    let create_dto2 = CreateMemoDto {
        title: "Memo 2".to_string(),
        description: None,
        date_to: Utc::now(),
    };
    let memo2 = service.create_memo(create_dto2).await.unwrap();

    let params = PaginationParams {
        limit: Some(10),
        offset: Some(0),
        completed: None,
        sort_by: Some("created_at".to_string()),
        order: Some("desc".to_string()),
    };

    let result = service.get_all_memos(params).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.total >= 2);
    assert!(!response.data.is_empty());

    service.delete_memo(memo1.id).await.ok();
    service.delete_memo(memo2.id).await.ok();
}

#[tokio::test]
async fn test_get_all_memos_with_filter() {
    let service = setup_test_service().await;

    let create_dto = CreateMemoDto {
        title: "Completed Memo".to_string(),
        description: None,
        date_to: Utc::now(),
    };
    let created = service.create_memo(create_dto).await.unwrap();

    let update_dto = UpdateMemoDto {
        title: created.title.clone(),
        description: None,
        date_to: created.date_to,
        completed: true,
    };
    service.update_memo(created.id, update_dto).await.unwrap();

    let params = PaginationParams {
        limit: Some(10),
        offset: Some(0),
        completed: Some(true),
        sort_by: Some("created_at".to_string()),
        order: Some("desc".to_string()),
    };

    let result = service.get_all_memos(params).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.total >= 1);
    assert!(response.data.iter().all(|m| m.completed));

    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_create_memo_validation_fails() {
    let service = setup_test_service().await;

    let create_dto = CreateMemoDto {
        title: "".to_string(), // Empty title should fail validation
        description: None,
        date_to: Utc::now(),
    };

    let result = service.create_memo(create_dto).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_pagination() {
    let service = setup_test_service().await;

    let mut created_ids = Vec::new();
    for i in 0..5 {
        let create_dto = CreateMemoDto {
            title: format!("Pagination Test {}", i),
            description: None,
            date_to: Utc::now(),
        };
        let memo = service.create_memo(create_dto).await.unwrap();
        created_ids.push(memo.id);
    }

    let params = PaginationParams {
        limit: Some(2),
        offset: Some(0),
        completed: None,
        sort_by: Some("created_at".to_string()),
        order: Some("desc".to_string()),
    };

    let result = service.get_all_memos(params).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.limit, 2);
    assert!(response.data.len() <= 2);

    for id in created_ids {
        service.delete_memo(id).await.ok();
    }
}

#[tokio::test]
async fn test_create_memo_with_sanitization() {
    let service = setup_test_service().await;

    // Create memo with XSS attempt
    let create_dto = CreateMemoDto {
        title: "<script>alert('xss')</script>Test Memo".to_string(),
        description: Some("<img src=x onerror=alert('xss')>Description".to_string()),
        date_to: Utc::now(),
    };

    let result = service.create_memo(create_dto).await;
    assert!(result.is_ok());

    let memo = result.unwrap();

    // Verify XSS was sanitized
    assert!(!memo.title.contains("<script>"));
    assert_eq!(memo.title, "Test Memo");

    if let Some(desc) = &memo.description {
        assert!(!desc.contains("onerror"));
    }

    // Cleanup
    service.delete_memo(memo.id).await.ok();
}

#[tokio::test]
async fn test_update_memo_sanitizes_input() {
    let service = setup_test_service().await;

    // Create a memo first
    let create_dto = CreateMemoDto {
        title: "Original".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();

    // Update with malicious content
    let update_dto = UpdateMemoDto {
        title: "<script>bad()</script>Updated".to_string(),
        description: Some("Safe description".to_string()),
        date_to: Utc::now(),
        completed: false,
    };

    let updated = service.update_memo(created.id, update_dto).await.unwrap();

    // Verify sanitization
    assert!(!updated.title.contains("<script>"));
    assert_eq!(updated.title, "Updated");

    // Cleanup
    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_create_memos_batch_transaction() {
    let service = setup_test_service().await;

    // Create multiple memos in batch
    let dtos = vec![
        CreateMemoDto {
            title: "Batch 1".to_string(),
            description: Some("First in batch".to_string()),
            date_to: Utc::now(),
        },
        CreateMemoDto {
            title: "Batch 2".to_string(),
            description: Some("Second in batch".to_string()),
            date_to: Utc::now(),
        },
        CreateMemoDto {
            title: "Batch 3".to_string(),
            description: Some("Third in batch".to_string()),
            date_to: Utc::now(),
        },
    ];

    let result = service.create_memos_batch(dtos).await;
    assert!(result.is_ok());

    let created = result.unwrap();
    assert_eq!(created.len(), 3);

    // Verify all were created
    for memo in &created {
        let fetched = service.get_memo_by_id(memo.id).await;
        assert!(fetched.is_ok());
    }

    // Cleanup - delete in batch
    let ids: Vec<_> = created.iter().map(|m| m.id).collect();
    let deleted_count = service.delete_memos_batch(ids).await.unwrap();
    assert_eq!(deleted_count, 3);
}

#[tokio::test]
async fn test_delete_memos_batch_transaction() {
    let service = setup_test_service().await;

    // Create several memos individually
    let mut ids = Vec::new();
    for i in 1..=5 {
        let dto = CreateMemoDto {
            title: format!("Batch Delete {}", i),
            description: None,
            date_to: Utc::now(),
        };
        let created = service.create_memo(dto).await.unwrap();
        ids.push(created.id);
    }

    // Delete all in batch
    let deleted_count = service.delete_memos_batch(ids.clone()).await.unwrap();
    assert_eq!(deleted_count, 5);

    // Verify all are deleted
    for id in ids {
        let result = service.get_memo_by_id(id).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_batch_create_with_validation_error() {
    let service = setup_test_service().await;

    // Create batch with one invalid memo (empty title)
    let dtos = vec![
        CreateMemoDto {
            title: "Valid Memo 1".to_string(),
            description: None,
            date_to: Utc::now(),
        },
        CreateMemoDto {
            title: "".to_string(), // Invalid - empty title
            description: None,
            date_to: Utc::now(),
        },
        CreateMemoDto {
            title: "Valid Memo 2".to_string(),
            description: None,
            date_to: Utc::now(),
        },
    ];

    // Should fail due to validation error
    let result = service.create_memos_batch(dtos).await;
    assert!(result.is_err());

    // Verify transaction rolled back - no memos should be created
    // (Would need to check by ID or timestamp if we stored them,
    // but validation happens before transaction)
}
