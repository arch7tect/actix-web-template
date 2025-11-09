# Chapter 7: Service Layer - Business Logic and Transactions

## Overview

The service layer sits between your handlers (HTTP layer) and repositories (data access layer), orchestrating business logic, coordinating transactions, and transforming data between DTOs and entities. This layer encapsulates domain rules, sanitizes user input, and ensures data consistency across multiple operations.

In this chapter, you'll build a complete service layer that handles business logic, coordinates database transactions, and provides a clean API for your handlers to use.

## Prerequisites

- Completed Chapter 6 (Repository Layer)
- Understanding of async/await in Rust
- Familiarity with the Result type and error handling
- Basic understanding of ACID transactions

## Learning Objectives

By the end of this chapter, you will:
- Understand the service layer's role in layered architecture
- Implement DTO to Entity conversions
- Sanitize user input to prevent XSS attacks
- Coordinate transactions across multiple operations
- Apply business logic validation
- Create a clean service API for handlers
- Test service layer logic independently

## Concepts Covered

### The Service Layer Pattern

The service layer pattern separates business logic from HTTP concerns and database operations:

```
┌──────────────────────────────────────┐
│   Handlers (HTTP Layer)              │
│   - Parse requests                   │
│   - Validate input format            │
│   - Return HTTP responses            │
└──────────────┬───────────────────────┘
               │
┌──────────────▼───────────────────────┐
│   Service Layer (THIS CHAPTER)       │
│   - Business logic                   │
│   - DTO ↔ Entity conversion          │
│   - Input sanitization               │
│   - Transaction coordination         │
│   - Business rule validation         │
└──────────────┬───────────────────────┘
               │
┌──────────────▼───────────────────────┐
│   Repository Layer                   │
│   - Database operations              │
│   - Query building                   │
│   - CRUD methods                     │
└──────────────────────────────────────┘
```

**Why separate services from repositories?**

1. **Single Responsibility**: Repositories handle data access; services handle business logic
2. **Transaction Coordination**: Services orchestrate multiple repository calls in one transaction
3. **Reusability**: Business logic can be reused across different handlers
4. **Testability**: Services can be tested with mocked repositories
5. **Maintainability**: Changes to business rules don't affect data access code

### DTO to Entity Conversion

DTOs (Data Transfer Objects) are designed for API contracts, while entities represent database structure. The service layer converts between them:

- **CreateMemoDto → ActiveModel**: Transform API input into database insert
- **UpdateMemoDto → ActiveModel**: Transform update data into database update
- **Entity → MemoResponseDto**: Transform database result into API output

### Input Sanitization

User-generated content must be sanitized to prevent XSS (Cross-Site Scripting) attacks. We'll use the `ammonia` crate to clean HTML:

```rust
// Before sanitization (dangerous)
let title = "<script>alert('XSS')</script>Memo Title";

// After sanitization (safe)
let title = "Memo Title";  // Script tags removed
```

### Transaction Coordination

Services coordinate transactions when operations must succeed or fail together:

```rust
// Example: Create memo and log activity (atomic operation)
let txn = db.begin().await?;

// Both operations must succeed
let memo = memo_repo.create_with_txn(&txn, dto).await?;
let log = activity_repo.log_with_txn(&txn, memo.id, "created").await?;

txn.commit().await?;  // Commit both or rollback both
```

### Business Logic Examples

Services enforce domain rules:
- Validate date_to is in the future
- Prevent modification of completed memos (if that's a business rule)
- Calculate derived fields
- Apply default values
- Format or normalize data

## Step-by-Step Instructions

### Step 1: Add Dependencies

First, add the `ammonia` crate for HTML sanitization.

**Why**: We need to sanitize user input to prevent XSS attacks.

**How**: Add to `Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
ammonia = "4.1"
```

**Verify**:
```bash
cargo build
```

You should see ammonia being compiled.

---

### Step 2: Create Utility Module for Sanitization

Create a reusable sanitization utility.

**Why**: Centralize sanitization logic for consistency and reusability.

**How**: Create `src/utils/sanitize.rs`:

```rust
use ammonia::clean;

/// Sanitizes HTML content to prevent XSS attacks
///
/// This function removes dangerous HTML tags and attributes while
/// preserving safe content. It uses a whitelist approach.
///
/// # Arguments
/// * `input` - The potentially unsafe HTML string
///
/// # Returns
/// A sanitized string safe for storage and display
///
/// # Examples
/// ```
/// let safe = sanitize_html("<script>alert('xss')</script>Hello");
/// assert_eq!(safe, "Hello");
/// ```
pub fn sanitize_html(input: &str) -> String {
    clean(input)
}

/// Sanitizes an optional string
///
/// Convenience function for Option<String> fields
pub fn sanitize_optional_html(input: Option<&str>) -> Option<String> {
    input.map(sanitize_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_script_tags() {
        let input = "<script>alert('xss')</script>Hello World";
        let result = sanitize_html(input);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_sanitize_img_onerror() {
        let input = r#"<img src="x" onerror="alert('xss')">Text"#;
        let result = sanitize_html(input);
        // ammonia removes dangerous attributes
        assert!(!result.contains("onerror"));
    }

    #[test]
    fn test_sanitize_safe_content() {
        let input = "Hello <b>World</b>";
        let result = sanitize_html(input);
        assert_eq!(result, "Hello <b>World</b>");
    }

    #[test]
    fn test_sanitize_optional_some() {
        let result = sanitize_optional_html(Some("<script>bad</script>Good"));
        assert_eq!(result, Some("Good".to_string()));
    }

    #[test]
    fn test_sanitize_optional_none() {
        let result = sanitize_optional_html(None);
        assert_eq!(result, None);
    }
}
```

Update `src/utils/mod.rs`:

```rust
pub mod sanitize;
pub mod tracing;
```

**Verify**:
```bash
cargo test --lib utils::sanitize
```

All sanitization tests should pass.

---

### Step 3: Create the Service Module Structure

Set up the services directory.

**Why**: Organize service layer code separately from other layers.

**How**: Create `src/services/mod.rs`:

```rust
pub mod memo_service;

pub use memo_service::MemoService;
```

---

### Step 4: Implement the MemoService

Create the core service with business logic.

**Why**: This is the heart of your business logic layer.

**How**: Create `src/services/memo_service.rs`:

```rust
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, TransactionTrait};
use uuid::Uuid;

use crate::dto::memo_dto::{
    CreateMemoDto, MemoResponseDto, PaginatedResponse, PaginationParams, PatchMemoDto,
    UpdateMemoDto,
};
use crate::entities::{memos, prelude::Memos};
use crate::error::app_error::AppError;
use crate::repository::memo_repository::MemoRepository;
use crate::utils::sanitize::{sanitize_html, sanitize_optional_html};

/// Service layer for memo business logic
///
/// The MemoService orchestrates business operations, coordinates transactions,
/// sanitizes input, and converts between DTOs and entities.
pub struct MemoService {
    db: DatabaseConnection,
}

impl MemoService {
    /// Creates a new MemoService instance
    ///
    /// # Arguments
    /// * `db` - Database connection to use for operations
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Lists memos with pagination, filtering, and sorting
    ///
    /// # Arguments
    /// * `params` - Pagination and filtering parameters
    ///
    /// # Returns
    /// Paginated list of memos as DTOs
    #[tracing::instrument(skip(self))]
    pub async fn list_memos(
        &self,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<MemoResponseDto>, AppError> {
        let repo = MemoRepository::new(self.db.clone());
        let result = repo.find_all(params).await?;

        // Convert entities to DTOs
        let items = result
            .items
            .into_iter()
            .map(Self::entity_to_dto)
            .collect();

        Ok(PaginatedResponse {
            items,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        })
    }

    /// Gets a single memo by ID
    ///
    /// # Arguments
    /// * `id` - UUID of the memo to retrieve
    ///
    /// # Returns
    /// The memo as a DTO, or NotFound error
    #[tracing::instrument(skip(self))]
    pub async fn get_memo(&self, id: Uuid) -> Result<MemoResponseDto, AppError> {
        let repo = MemoRepository::new(self.db.clone());
        let memo = repo.find_by_id(id).await?;
        Ok(Self::entity_to_dto(memo))
    }

    /// Creates a new memo with sanitized input
    ///
    /// # Arguments
    /// * `dto` - The memo creation data
    ///
    /// # Returns
    /// The created memo as a DTO
    ///
    /// # Business Rules
    /// - Title and description are sanitized to prevent XSS
    /// - date_to must be in the future (optional validation)
    #[tracing::instrument(skip(self))]
    pub async fn create_memo(&self, dto: CreateMemoDto) -> Result<MemoResponseDto, AppError> {
        // Sanitize input to prevent XSS attacks
        let sanitized_title = sanitize_html(&dto.title);
        let sanitized_description = sanitize_optional_html(dto.description.as_deref());

        // Optional: Validate business rules
        // if dto.date_to <= chrono::Utc::now() {
        //     return Err(AppError::Validation("date_to must be in the future".into()));
        // }

        let repo = MemoRepository::new(self.db.clone());

        // Create the entity
        let active_model = memos::ActiveModel {
            id: ActiveValue::NotSet,
            title: ActiveValue::Set(sanitized_title),
            description: ActiveValue::Set(sanitized_description),
            date_to: ActiveValue::Set(dto.date_to.into()),
            completed: ActiveValue::Set(false),
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
        };

        let memo = repo.create(active_model).await?;
        tracing::info!("Created memo with ID: {}", memo.id);

        Ok(Self::entity_to_dto(memo))
    }

    /// Fully updates a memo (PUT operation)
    ///
    /// # Arguments
    /// * `id` - UUID of the memo to update
    /// * `dto` - Complete update data
    ///
    /// # Returns
    /// The updated memo as a DTO
    ///
    /// # Business Rules
    /// - All fields are replaced (full update)
    /// - Input is sanitized
    #[tracing::instrument(skip(self))]
    pub async fn update_memo(
        &self,
        id: Uuid,
        dto: UpdateMemoDto,
    ) -> Result<MemoResponseDto, AppError> {
        // First, verify the memo exists
        let repo = MemoRepository::new(self.db.clone());
        let existing = repo.find_by_id(id).await?;

        // Sanitize input
        let sanitized_title = sanitize_html(&dto.title);
        let sanitized_description = sanitize_optional_html(dto.description.as_deref());

        // Create active model for update
        let mut active_model: memos::ActiveModel = existing.into();
        active_model.title = ActiveValue::Set(sanitized_title);
        active_model.description = ActiveValue::Set(sanitized_description);
        active_model.date_to = ActiveValue::Set(dto.date_to.into());
        active_model.completed = ActiveValue::Set(dto.completed);

        let memo = repo.update(active_model).await?;
        tracing::info!("Updated memo with ID: {}", memo.id);

        Ok(Self::entity_to_dto(memo))
    }

    /// Partially updates a memo (PATCH operation)
    ///
    /// # Arguments
    /// * `id` - UUID of the memo to update
    /// * `dto` - Partial update data (only provided fields are updated)
    ///
    /// # Returns
    /// The updated memo as a DTO
    ///
    /// # Business Rules
    /// - Only provided fields are updated
    /// - Other fields remain unchanged
    /// - Input is sanitized
    #[tracing::instrument(skip(self))]
    pub async fn patch_memo(
        &self,
        id: Uuid,
        dto: PatchMemoDto,
    ) -> Result<MemoResponseDto, AppError> {
        let repo = MemoRepository::new(self.db.clone());
        let existing = repo.find_by_id(id).await?;

        let mut active_model: memos::ActiveModel = existing.into();

        // Only update provided fields
        if let Some(title) = dto.title {
            let sanitized = sanitize_html(&title);
            active_model.title = ActiveValue::Set(sanitized);
        }

        if let Some(description) = dto.description {
            let sanitized = sanitize_optional_html(Some(&description));
            active_model.description = ActiveValue::Set(sanitized);
        }

        if let Some(date_to) = dto.date_to {
            active_model.date_to = ActiveValue::Set(date_to.into());
        }

        if let Some(completed) = dto.completed {
            active_model.completed = ActiveValue::Set(completed);
        }

        let memo = repo.update(active_model).await?;
        tracing::info!("Patched memo with ID: {}", memo.id);

        Ok(Self::entity_to_dto(memo))
    }

    /// Toggles the completion status of a memo
    ///
    /// # Arguments
    /// * `id` - UUID of the memo to toggle
    ///
    /// # Returns
    /// The updated memo as a DTO
    #[tracing::instrument(skip(self))]
    pub async fn toggle_complete(&self, id: Uuid) -> Result<MemoResponseDto, AppError> {
        let repo = MemoRepository::new(self.db.clone());
        let existing = repo.find_by_id(id).await?;

        let new_status = !existing.completed;
        let mut active_model: memos::ActiveModel = existing.into();
        active_model.completed = ActiveValue::Set(new_status);

        let memo = repo.update(active_model).await?;
        tracing::info!("Toggled completion for memo {}: {}", memo.id, new_status);

        Ok(Self::entity_to_dto(memo))
    }

    /// Deletes a memo
    ///
    /// # Arguments
    /// * `id` - UUID of the memo to delete
    ///
    /// # Returns
    /// Ok(()) on success, or error if memo doesn't exist
    #[tracing::instrument(skip(self))]
    pub async fn delete_memo(&self, id: Uuid) -> Result<(), AppError> {
        let repo = MemoRepository::new(self.db.clone());
        repo.delete(id).await?;
        tracing::info!("Deleted memo with ID: {}", id);
        Ok(())
    }

    /// Creates multiple memos in a single transaction
    ///
    /// This demonstrates transaction coordination at the service layer.
    /// All memos are created atomically - if any fail, all are rolled back.
    ///
    /// # Arguments
    /// * `dtos` - Vector of memo creation DTOs
    ///
    /// # Returns
    /// Vector of created memos as DTOs
    ///
    /// # Example
    /// ```rust
    /// let memos = vec![dto1, dto2, dto3];
    /// let created = service.create_memos_batch(memos).await?;
    /// // Either all 3 are created, or none are
    /// ```
    #[tracing::instrument(skip(self, dtos))]
    pub async fn create_memos_batch(
        &self,
        dtos: Vec<CreateMemoDto>,
    ) -> Result<Vec<MemoResponseDto>, AppError> {
        // Start a transaction
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| AppError::Database(e))?;

        let mut created_memos = Vec::new();

        // Create all memos within the transaction
        for dto in dtos {
            let sanitized_title = sanitize_html(&dto.title);
            let sanitized_description = sanitize_optional_html(dto.description.as_deref());

            let active_model = memos::ActiveModel {
                id: ActiveValue::NotSet,
                title: ActiveValue::Set(sanitized_title),
                description: ActiveValue::Set(sanitized_description),
                date_to: ActiveValue::Set(dto.date_to.into()),
                completed: ActiveValue::Set(false),
                created_at: ActiveValue::NotSet,
                updated_at: ActiveValue::NotSet,
            };

            // Insert within transaction
            let memo = active_model
                .insert(&txn)
                .await
                .map_err(|e| AppError::Database(e))?;

            created_memos.push(memo);
        }

        // Commit the transaction - all inserts succeed or all fail
        txn.commit().await.map_err(|e| AppError::Database(e))?;

        tracing::info!("Created {} memos in batch", created_memos.len());

        Ok(created_memos
            .into_iter()
            .map(Self::entity_to_dto)
            .collect())
    }

    /// Deletes multiple memos in a single transaction
    ///
    /// # Arguments
    /// * `ids` - Vector of memo UUIDs to delete
    ///
    /// # Returns
    /// Number of memos deleted
    #[tracing::instrument(skip(self))]
    pub async fn delete_memos_batch(&self, ids: Vec<Uuid>) -> Result<u64, AppError> {
        let repo = MemoRepository::new(self.db.clone());

        // Start transaction
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| AppError::Database(e))?;

        let mut total_deleted = 0u64;

        // Delete all memos in transaction
        for id in ids {
            // Use the transaction connection
            let result = Memos::delete_by_id(id)
                .exec(&txn)
                .await
                .map_err(|e| AppError::Database(e))?;

            total_deleted += result.rows_affected;
        }

        // Commit transaction
        txn.commit().await.map_err(|e| AppError::Database(e))?;

        tracing::info!("Deleted {} memos in batch", total_deleted);

        Ok(total_deleted)
    }

    /// Converts an entity to a DTO
    ///
    /// This is a private helper method for transforming database entities
    /// into API response objects.
    fn entity_to_dto(entity: memos::Model) -> MemoResponseDto {
        MemoResponseDto {
            id: entity.id,
            title: entity.title,
            description: entity.description,
            date_to: entity.date_to.naive_utc(),
            completed: entity.completed,
            created_at: entity.created_at.naive_utc(),
            updated_at: entity.updated_at.naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_to_dto_conversion() {
        let entity = memos::Model {
            id: Uuid::new_v4(),
            title: "Test Memo".to_string(),
            description: Some("Description".to_string()),
            date_to: chrono::Utc::now(),
            completed: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let dto = MemoService::entity_to_dto(entity.clone());

        assert_eq!(dto.id, entity.id);
        assert_eq!(dto.title, entity.title);
        assert_eq!(dto.description, entity.description);
        assert_eq!(dto.completed, entity.completed);
    }
}
```

Update `src/lib.rs` to include the services module:

```rust
pub mod config;
pub mod docs;
pub mod dto;
pub mod entities;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod repository;
pub mod services;  // Add this line
pub mod state;
pub mod utils;
```

**Verify**:
```bash
cargo build
```

The code should compile successfully.

---

### Step 5: Add Integration Tests for the Service Layer

Create comprehensive tests for service logic.

**Why**: Test business logic independently from HTTP layer.

**How**: Create `tests/service_tests.rs`:

```rust
use actix_web_template::config::settings::Settings;
use actix_web_template::dto::memo_dto::CreateMemoDto;
use actix_web_template::services::MemoService;
use chrono::Utc;
use sea_orm::Database;

#[actix_web::test]
async fn test_create_memo_with_sanitization() {
    // Setup
    let settings = Settings::new().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let service = MemoService::new(db);

    // Create memo with XSS attempt
    let dto = CreateMemoDto {
        title: "<script>alert('xss')</script>Test Memo".to_string(),
        description: Some("<img src=x onerror=alert('xss')>Description".to_string()),
        date_to: Utc::now(),
    };

    let result = service.create_memo(dto).await;
    assert!(result.is_ok());

    let memo = result.unwrap();

    // Verify XSS was sanitized
    assert!(!memo.title.contains("<script>"));
    assert_eq!(memo.title, "Test Memo");

    if let Some(desc) = &memo.description {
        assert!(!desc.contains("onerror"));
    }

    // Cleanup
    let _ = service.delete_memo(memo.id).await;
}

#[actix_web::test]
async fn test_update_memo_sanitizes_input() {
    let settings = Settings::new().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let service = MemoService::new(db);

    // Create a memo first
    let create_dto = CreateMemoDto {
        title: "Original".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();

    // Update with malicious content
    let update_dto = actix_web_template::dto::memo_dto::UpdateMemoDto {
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
    let _ = service.delete_memo(created.id).await;
}

#[actix_web::test]
async fn test_patch_memo_partial_update() {
    let settings = Settings::new().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let service = MemoService::new(db);

    // Create a memo
    let create_dto = CreateMemoDto {
        title: "Original Title".to_string(),
        description: Some("Original Description".to_string()),
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();

    // Patch only the title
    let patch_dto = actix_web_template::dto::memo_dto::PatchMemoDto {
        title: Some("Updated Title".to_string()),
        description: None,
        date_to: None,
        completed: None,
    };

    let patched = service.patch_memo(created.id, patch_dto).await.unwrap();

    // Verify only title changed
    assert_eq!(patched.title, "Updated Title");
    assert_eq!(patched.description, Some("Original Description".to_string()));
    assert_eq!(patched.date_to, created.date_to);

    // Cleanup
    let _ = service.delete_memo(created.id).await;
}

#[actix_web::test]
async fn test_toggle_complete() {
    let settings = Settings::new().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let service = MemoService::new(db);

    // Create a memo (starts as not completed)
    let create_dto = CreateMemoDto {
        title: "Toggle Test".to_string(),
        description: None,
        date_to: Utc::now(),
    };

    let created = service.create_memo(create_dto).await.unwrap();
    assert!(!created.completed);

    // Toggle to complete
    let toggled = service.toggle_complete(created.id).await.unwrap();
    assert!(toggled.completed);

    // Toggle back to incomplete
    let toggled_again = service.toggle_complete(created.id).await.unwrap();
    assert!(!toggled_again.completed);

    // Cleanup
    let _ = service.delete_memo(created.id).await;
}

#[actix_web::test]
async fn test_create_memos_batch_transaction() {
    let settings = Settings::new().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let service = MemoService::new(db);

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
        let fetched = service.get_memo(memo.id).await;
        assert!(fetched.is_ok());
    }

    // Cleanup - delete in batch
    let ids: Vec<_> = created.iter().map(|m| m.id).collect();
    let deleted_count = service.delete_memos_batch(ids).await.unwrap();
    assert_eq!(deleted_count, 3);
}

#[actix_web::test]
async fn test_delete_memos_batch_transaction() {
    let settings = Settings::new().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let service = MemoService::new(db);

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
        let result = service.get_memo(id).await;
        assert!(result.is_err());
    }
}

#[actix_web::test]
async fn test_list_memos_with_pagination() {
    let settings = Settings::new().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");

    let service = MemoService::new(db);

    // Create test memos
    let mut created_ids = Vec::new();
    for i in 1..=15 {
        let dto = CreateMemoDto {
            title: format!("Pagination Test {}", i),
            description: None,
            date_to: Utc::now(),
        };
        let created = service.create_memo(dto).await.unwrap();
        created_ids.push(created.id);
    }

    // Test pagination
    let params = actix_web_template::dto::memo_dto::PaginationParams {
        page: Some(1),
        per_page: Some(10),
        completed: None,
        sort_by: Some("created_at".to_string()),
        sort_order: Some("desc".to_string()),
    };

    let result = service.list_memos(params).await.unwrap();

    assert_eq!(result.items.len(), 10);
    assert!(result.total >= 15);
    assert_eq!(result.page, 1);
    assert_eq!(result.per_page, 10);

    // Cleanup
    let _ = service.delete_memos_batch(created_ids).await;
}
```

**Verify**:
```bash
cargo test --test service_tests
```

All service tests should pass.

---

## Checkpoint

At this point, you should have:

1. A complete service layer with business logic
2. Input sanitization preventing XSS attacks
3. Transaction coordination for batch operations
4. DTO to Entity conversions
5. Comprehensive service tests

**Verify everything works**:

```bash
# Run all tests
cargo test

# Build the project
cargo build

# Check for any warnings
cargo clippy
```

Expected output:
- All tests passing (including new service tests)
- Clean build with no errors
- Minimal or no clippy warnings

---

## Common Issues and Solutions

### Issue: "ammonia crate not found"

**Symptoms**: Compilation error about missing ammonia crate

**Cause**: Dependency not added to Cargo.toml

**Solution**:
```bash
# Add to Cargo.toml
ammonia = "4.1"

# Then update dependencies
cargo update
cargo build
```

---

### Issue: "Transaction already committed or rolled back"

**Symptoms**: Database error when using transactions

**Cause**: Trying to use a transaction after it's been committed or dropped

**Solution**: Ensure you only commit once and don't use the transaction after commit:
```rust
let txn = db.begin().await?;
// ... operations ...
txn.commit().await?;
// Don't use txn after this point!
```

---

### Issue: "Sanitization removes too much content"

**Symptoms**: Valid HTML is being stripped

**Cause**: ammonia's whitelist is strict by default

**Solution**: Configure ammonia's Builder for custom rules:
```rust
use ammonia::Builder;

pub fn sanitize_html_custom(input: &str) -> String {
    Builder::default()
        .add_tags(&["custom-tag"])
        .clean(input)
        .to_string()
}
```

---

### Issue: "Service tests fail with database errors"

**Symptoms**: Tests fail with connection or query errors

**Cause**: Database not running or wrong connection string

**Solution**:
```bash
# Check database is running
psql $DATABASE_URL

# Ensure .env is correct
cat .env | grep DATABASE_URL

# Run migrations
cd migration && cargo run
```

---

## Code Review

### Key Design Principles Demonstrated
- **Separation of concerns**: Service methods expose business workflows while handlers remain focused on HTTP transport.
- **Defensive programming**: Inputs are sanitized and validated before touching repositories, reducing the risk of XSS or malformed data.
- **Transactional safety**: Multi-step operations such as create/update wrap repository calls in a transaction to guarantee atomicity.
- **Observability**: Tracing spans decorate each public method so you can correlate service activity with inbound requests.
- **Testability**: Services operate on DTOs and repository seams, making it straightforward to stub persistence in unit tests or reuse the real repository for integration tests.

### Architecture Benefits
- **Thin handlers**: The HTTP layer parses DTOs, calls a service, and returns a result—making alternate interfaces (CLI, gRPC) easier later.
- **Reusable business logic**: Background jobs or schedulers can reuse the same services without duplicating repository calls.
- **Security posture**: Sanitization and validation live close to the data boundary, ensuring only trusted payloads persist.
- **Operational clarity**: Centralized logging and tracing in the service layer surfaces failures even when repositories succeed.

### Complete Service Layer Structure
```
src/
├── services/
│   ├── mod.rs              # Public exports and service traits
│   └── memo_service.rs     # Business logic implementation
├── repository/
│   └── memo_repository.rs  # Persistence operations (Chapter 6)
└── utils/
    └── sanitize.rs         # HTML sanitization helpers
```

```rust
#[tracing::instrument(skip(self, dto), fields(has_description = dto.description.is_some()))]
pub async fn create_memo(&self, dto: CreateMemoDto) -> Result<MemoResponseDto, AppError> {
    dto.validate()?;

    let sanitized_title = sanitize_html(&dto.title);
    let sanitized_description = sanitize_optional_html(dto.description.as_deref());

    tracing::debug!(title = %sanitized_title, "Creating new memo with sanitized input");

    let memo = MemoRepository::create(
        &self.db,
        sanitized_title,
        sanitized_description,
        dto.date_to,
    )
    .await?;

    Ok(Self::entity_to_dto(memo))
}
```

---

## Testing

### Unit Test Coverage

The service layer tests cover:

- ✅ Input sanitization (XSS prevention)
- ✅ DTO to Entity conversion
- ✅ CRUD operations
- ✅ Partial updates (PATCH)
- ✅ Toggle operations
- ✅ Transaction atomicity (batch create)
- ✅ Transaction atomicity (batch delete)
- ✅ Pagination

### Manual Testing

You can't test the service layer via HTTP yet (that's Chapter 8), but you can write a quick test binary:

Create `examples/test_service.rs`:

```rust
use actix_web_template::config::settings::Settings;
use actix_web_template::dto::memo_dto::CreateMemoDto;
use actix_web_template::services::MemoService;
use chrono::Utc;
use sea_orm::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::new()?;
    let db = Database::connect(&settings.database.url).await?;
    let service = MemoService::new(db);

    // Create a memo
    let dto = CreateMemoDto {
        title: "Test from service".to_string(),
        description: Some("Testing the service layer".to_string()),
        date_to: Utc::now(),
    };

    let created = service.create_memo(dto).await?;
    println!("Created memo: {:?}", created);

    // Toggle it
    let toggled = service.toggle_complete(created.id).await?;
    println!("Toggled memo: {:?}", toggled);

    // Delete it
    service.delete_memo(created.id).await?;
    println!("Deleted memo: {}", created.id);

    Ok(())
}
```

Run it:
```bash
cargo run --example test_service
```

---

## Summary

### What You Learned

In this chapter, you:

1. **Built a service layer** that encapsulates business logic
2. **Separated concerns** between HTTP, business logic, and data access
3. **Implemented input sanitization** to prevent XSS attacks
4. **Coordinated transactions** for atomic batch operations
5. **Converted between DTOs and Entities** maintaining clean boundaries
6. **Added tracing instrumentation** for observability
7. **Wrote comprehensive tests** for business logic

### Architecture Progress

You've now completed the core three-layer architecture:

```
✅ HTTP Layer (Handlers) - Chapter 8 next
✅ Business Logic Layer (Services) - THIS CHAPTER
✅ Data Access Layer (Repository) - Chapter 6
✅ Entity Layer (SeaORM Models) - Chapter 2
```

### Key Takeaways

1. **Services orchestrate**: They coordinate multiple operations and enforce business rules
2. **Always sanitize user input**: Use ammonia or similar for XSS prevention
3. **Transactions ensure consistency**: Use them for operations that must succeed or fail together
4. **Keep layers independent**: Services don't know about HTTP; repositories don't know about business rules
5. **Test business logic separately**: Service tests validate logic without HTTP overhead

---

## Next Steps

### Required: Chapter 8 - REST API Handlers

You'll expose the service layer through HTTP handlers that validate requests, invoke services, and shape responses for the API. Expect to translate service errors into HTTP results while keeping handlers intentionally thin.

### Optional Exercises

1. **Challenge**: Prevent updates to completed memos by enforcing a business rule in the service layer.
2. **Challenge**: Replace hard deletes with a `deleted_at` soft-delete workflow and adjust repository queries accordingly.
3. **Challenge**: Add a search method that filters memos by title or description keywords.

---

## Additional Resources

### Input Sanitization
- [ammonia crate documentation](https://docs.rs/ammonia/)
- [OWASP XSS Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)

### Transaction Patterns
- [SeaORM Transactions](https://www.sea-ql.org/SeaORM/docs/advanced-query/transaction/)
- [Database Transactions Explained](https://www.postgresql.org/docs/current/tutorial-transactions.html)

### Service Layer Pattern
- [Martin Fowler - Service Layer](https://martinfowler.com/eaaCatalog/serviceLayer.html)
- [DDD Service Layer](https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/ddd-oriented-microservice)

### Rust Best Practices
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)

---

**Ready to build the HTTP layer?** Continue to **[Chapter 8: REST API Handlers](chapter-08.md)**
