# Chapter 6: Repository Layer - Database Operations

## Overview

In this chapter, we'll build the repository layer that handles all database operations. You'll learn the repository pattern, SeaORM query building, pagination, filtering, sorting, and database transactions.

By the end of this chapter, you'll have a clean data access layer that separates database logic from business logic.

> **Note on Tutorial Approach**: This chapter demonstrates foundational repository patterns and transaction handling. The production codebase uses these same patterns with additional optimizations and error handling. We'll build the core that scales to production.

## Prerequisites

### Completed

- Chapter 0: Prerequisites and Environment Setup
- Chapter 1: Core Application Setup
- Chapter 2: Database Integration with SeaORM
- Chapter 3: Error Handling and Middleware
- Chapter 4: Enhanced Health Checks
- Chapter 5: DTOs and Validation

### Required Knowledge

- SeaORM basics from Chapter 2
- Rust async/await
- Understanding of database concepts (CRUD, transactions)
- Basic SQL knowledge

### Required Software

- Working Actix Web application from Chapter 5
- PostgreSQL running with memos table

## Learning Objectives

By completing this chapter, you will:

1. Understand the repository pattern and its benefits
2. Build CRUD operations with SeaORM
3. Implement pagination with offset/limit
4. Add filtering by fields
5. Support sorting on multiple columns
6. Use database transactions for consistency
7. Handle transaction rollbacks and commits
8. Test repository methods
9. Understand ACID properties

## Concepts Covered

### The Repository Pattern

The **repository pattern** abstracts database access into a dedicated layer.

**Benefits**:
1. **Separation of concerns**: Database logic isolated from business logic
2. **Testability**: Can mock repository for testing services
3. **Flexibility**: Can swap database implementations
4. **Reusability**: Common queries in one place
5. **Type safety**: Compile-time query validation with SeaORM

**Structure**:
```rust
pub struct MemoRepository {
    db: DatabaseConnection,
}

impl MemoRepository {
    pub async fn create(&self, /* ... */) -> Result<Model, DbErr> { }
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Model>, DbErr> { }
    pub async fn list(&self, /* ... */) -> Result<Vec<Model>, DbErr> { }
    pub async fn update(&self, /* ... */) -> Result<Model, DbErr> { }
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> { }
}
```

### SeaORM Query Builder

SeaORM provides type-safe query building:

```rust
// SELECT * FROM memos WHERE id = ?
let memo = Memos::find_by_id(id).one(&db).await?;

// SELECT * FROM memos WHERE completed = false
let active = Memos::find()
    .filter(memos::Column::Completed.eq(false))
    .all(&db).await?;

// UPDATE memos SET completed = true WHERE id = ?
let mut active_model: memos::ActiveModel = memo.into();
active_model.completed = Set(true);
active_model.update(&db).await?;
```

### Pagination with Offset/Limit

Pagination prevents loading too much data:

```rust
// Page 1: LIMIT 10 OFFSET 0
// Page 2: LIMIT 10 OFFSET 10
// Page N: LIMIT per_page OFFSET (page - 1) * per_page

let offset = (page - 1) * per_page;
let memos = Memos::find()
    .limit(per_page)
    .offset(offset)
    .all(&db).await?;
```

### Database Transactions

**Transactions** ensure multiple operations succeed or fail together (ACID):

- **Atomicity**: All operations succeed, or none do
- **Consistency**: Database rules are enforced
- **Isolation**: Concurrent transactions don't interfere
- **Durability**: Committed changes persist

```rust
let txn = db.begin().await?;

// Multiple operations
create_memo(&txn, memo1).await?;
create_memo(&txn, memo2).await?;

// Commit (or rollback on error)
txn.commit().await?;
```

## Step-by-Step Instructions

### Step 1: Create Repository Module Structure

**Why**: Organize repository code in a dedicated module.

**How**:

1. **Create repository directory**:
   ```bash
   mkdir -p src/repository
   touch src/repository/mod.rs
   touch src/repository/memo_repository.rs
   ```

2. **Create `src/repository/mod.rs`**:

```rust
pub mod memo_repository;

pub use memo_repository::MemoRepository;
```

3. **Update `src/lib.rs`**:

```rust
pub mod config;
pub mod dto;
pub mod entities;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod repository;
pub mod state;
pub mod utils;
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 2: Create Basic Repository Structure

**Why**: Define the repository struct and basic CRUD methods.

**How**:

1. **Create `src/repository/memo_repository.rs`**:

```rust
use crate::entities::{memos, prelude::Memos};
use sea_orm::*;
use uuid::Uuid;

/// Repository for memo database operations
pub struct MemoRepository {
    db: DatabaseConnection,
}

impl MemoRepository {
    /// Create a new repository instance
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get a reference to the database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
```

**Understanding the structure**:
- Repository holds a `DatabaseConnection`
- Constructor pattern for easy instantiation
- Getter for database access when needed

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 3: Implement Create Operation

**Why**: Add ability to insert new memos into the database.

**How**:

1. **Add to `src/repository/memo_repository.rs`**:

```rust
use chrono::{DateTime, Utc};

impl MemoRepository {
    // ... existing methods ...

    /// Create a new memo
    #[tracing::instrument(skip(self), fields(title = %title))]
    pub async fn create(
        &self,
        title: String,
        description: Option<String>,
        date_to: DateTime<Utc>,
    ) -> Result<memos::Model, DbErr> {
        let now = Utc::now();

        let new_memo = memos::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(title),
            description: Set(description),
            date_to: Set(date_to.and_utc()),
            completed: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = new_memo.insert(&self.db).await?;

        tracing::debug!(memo_id = %result.id, "Created memo");

        Ok(result)
    }
}
```

**Understanding ActiveModel**:
- `ActiveModel` represents a row to be inserted/updated
- `Set(value)` marks field as changed
- `NotSet` leaves field unchanged (for updates)
- `insert()` performs the INSERT query

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 4: Implement Read Operations

**Why**: Add ability to retrieve memos from the database.

**How**:

1. **Add to `src/repository/memo_repository.rs`**:

```rust
impl MemoRepository {
    // ... existing methods ...

    /// Find a memo by ID
    #[tracing::instrument(skip(self), fields(id = %id))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<memos::Model>, DbErr> {
        let memo = Memos::find_by_id(id).one(&self.db).await?;

        if memo.is_some() {
            tracing::debug!("Found memo");
        } else {
            tracing::debug!("Memo not found");
        }

        Ok(memo)
    }

    /// List all memos with pagination
    #[tracing::instrument(skip(self))]
    pub async fn list(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<memos::Model>, u64), DbErr> {
        // Calculate offset
        let offset = (page - 1) * per_page;

        // Get total count
        let total = Memos::find().count(&self.db).await?;

        // Get paginated results
        let memos = Memos::find()
            .limit(per_page)
            .offset(offset)
            .all(&self.db)
            .await?;

        tracing::debug!(
            count = memos.len(),
            total = total,
            page = page,
            "Listed memos"
        );

        Ok((memos, total))
    }
}
```

**Understanding the queries**:
- `find_by_id()` generates `SELECT * FROM memos WHERE id = ?`
- `one()` returns `Option<Model>` (None if not found)
- `count()` returns total rows (for pagination)
- `limit()` and `offset()` for pagination
- `all()` returns `Vec<Model>`

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 5: Add Filtering and Sorting

**Why**: Support filtering by completion status and sorting results.

**How**:

1. **Add to `src/repository/memo_repository.rs`**:

```rust
impl MemoRepository {
    // ... existing methods ...

    /// List memos with filtering and sorting
    #[tracing::instrument(skip(self))]
    pub async fn list_filtered(
        &self,
        page: u64,
        per_page: u64,
        completed: Option<bool>,
        sort_by: Option<String>,
        sort_order: Option<String>,
    ) -> Result<(Vec<memos::Model>, u64), DbErr> {
        let offset = (page - 1) * per_page;

        // Build base query
        let mut query = Memos::find();

        // Apply completion filter if specified
        if let Some(completed_status) = completed {
            query = query.filter(memos::Column::Completed.eq(completed_status));
        }

        // Get total count with filters
        let total = query.clone().count(&self.db).await?;

        // Apply sorting
        query = match (sort_by.as_deref(), sort_order.as_deref()) {
            (Some("title"), Some("asc")) => query.order_by_asc(memos::Column::Title),
            (Some("title"), Some("desc")) => query.order_by_desc(memos::Column::Title),
            (Some("date_to"), Some("asc")) => query.order_by_asc(memos::Column::DateTo),
            (Some("date_to"), Some("desc")) => query.order_by_desc(memos::Column::DateTo),
            (Some("created_at"), Some("asc")) => query.order_by_asc(memos::Column::CreatedAt),
            (Some("created_at"), _) | (None, _) => {
                // Default: sort by created_at desc
                query.order_by_desc(memos::Column::CreatedAt)
            }
            _ => query.order_by_desc(memos::Column::CreatedAt),
        };

        // Apply pagination
        let memos = query.limit(per_page).offset(offset).all(&self.db).await?;

        tracing::debug!(
            count = memos.len(),
            total = total,
            page = page,
            completed = ?completed,
            "Listed filtered memos"
        );

        Ok((memos, total))
    }
}
```

**Understanding dynamic queries**:
- Build query step-by-step
- `clone()` query for count before pagination
- `filter()` adds WHERE clauses
- `order_by_asc()` / `order_by_desc()` for sorting
- Pattern matching for flexible sorting options

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 6: Implement Update Operation

**Why**: Add ability to modify existing memos.

**How**:

1. **Add to `src/repository/memo_repository.rs`**:

```rust
impl MemoRepository {
    // ... existing methods ...

    /// Update an existing memo
    #[tracing::instrument(skip(self), fields(id = %id))]
    pub async fn update(
        &self,
        id: Uuid,
        title: Option<String>,
        description: Option<Option<String>>,
        date_to: Option<DateTime<Utc>>,
        completed: Option<bool>,
    ) -> Result<memos::Model, DbErr> {
        // First, find the memo
        let memo = Memos::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Memo not found".to_string()))?;

        // Convert to ActiveModel
        let mut active: memos::ActiveModel = memo.into();

        // Update fields if provided
        if let Some(t) = title {
            active.title = Set(t);
        }

        if let Some(d) = description {
            active.description = Set(d);
        }

        if let Some(dt) = date_to {
            active.date_to = Set(dt.and_utc());
        }

        if let Some(c) = completed {
            active.completed = Set(c);
        }

        // Always update updated_at
        active.updated_at = Set(Utc::now());

        // Save changes
        let updated = active.update(&self.db).await?;

        tracing::debug!("Updated memo");

        Ok(updated)
    }

    /// Toggle memo completion status
    #[tracing::instrument(skip(self), fields(id = %id))]
    pub async fn toggle_complete(&self, id: Uuid) -> Result<memos::Model, DbErr> {
        let memo = Memos::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Memo not found".to_string()))?;

        let mut active: memos::ActiveModel = memo.into();
        active.completed = Set(!active.completed.unwrap());
        active.updated_at = Set(Utc::now());

        let updated = active.update(&self.db).await?;

        tracing::debug!(completed = updated.completed, "Toggled memo completion");

        Ok(updated)
    }
}
```

**Understanding updates**:
- Find existing memo first
- Convert `Model` to `ActiveModel` with `.into()`
- Only `Set()` fields that should change
- `update()` performs UPDATE query
- Always update `updated_at` timestamp

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 7: Implement Delete Operation

**Why**: Add ability to remove memos from the database.

**How**:

1. **Add to `src/repository/memo_repository.rs`**:

```rust
impl MemoRepository {
    // ... existing methods ...

    /// Delete a memo by ID
    #[tracing::instrument(skip(self), fields(id = %id))]
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        let result = Memos::delete_by_id(id).exec(&self.db).await?;

        if result.rows_affected > 0 {
            tracing::debug!("Deleted memo");
        } else {
            tracing::debug!("Memo not found for deletion");
        }

        Ok(result)
    }
}
```

**Understanding delete**:
- `delete_by_id()` generates DELETE query
- `exec()` executes the query
- `DeleteResult` contains `rows_affected`
- Can check if anything was actually deleted

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 8: Add Transaction Support

**Why**: Enable atomic operations across multiple database calls.

**How**:

1. **Add to `src/repository/memo_repository.rs`**:

```rust
impl MemoRepository {
    // ... existing methods ...

    /// Create multiple memos in a single transaction
    #[tracing::instrument(skip(self, memos))]
    pub async fn create_batch(
        &self,
        memos: Vec<(String, Option<String>, DateTime<Utc>)>,
    ) -> Result<Vec<memos::Model>, DbErr> {
        // Begin transaction
        let txn = self.db.begin().await?;

        let mut results = Vec::new();

        for (title, description, date_to) in memos {
            let now = Utc::now();

            let new_memo = memos::ActiveModel {
                id: Set(Uuid::new_v4()),
                title: Set(title),
                description: Set(description),
                date_to: Set(date_to.and_utc()),
                completed: Set(false),
                created_at: Set(now),
                updated_at: Set(now),
            };

            let result = new_memo.insert(&txn).await?;
            results.push(result);
        }

        // Commit transaction
        txn.commit().await?;

        tracing::debug!(count = results.len(), "Created batch of memos");

        Ok(results)
    }

    /// Delete all completed memos (using transaction)
    #[tracing::instrument(skip(self))]
    pub async fn delete_completed(&self) -> Result<u64, DbErr> {
        let txn = self.db.begin().await?;

        let result = Memos::delete_many()
            .filter(memos::Column::Completed.eq(true))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        tracing::info!(count = result.rows_affected, "Deleted completed memos");

        Ok(result.rows_affected)
    }
}
```

**Understanding transactions**:
- `begin()` starts a transaction
- All operations use `&txn` instead of `&self.db`
- `commit()` saves changes permanently
- If any operation fails, transaction auto-rolls back
- Ensures all-or-nothing semantics

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 9: Add Repository Tests

**Why**: Verify repository methods work correctly with a test database.

**How**:

1. **Create `tests/repository_tests.rs`**:

```rust
use actix_memo_app::{
    entities::prelude::*,
    repository::MemoRepository,
    utils::database,
};
use chrono::{DateTime, Utc};
use sea_orm::{Database, DatabaseConnection};
use uuid::Uuid;

/// Setup test database connection
async fn setup_db() -> DatabaseConnection {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/memos_test".to_string());

    Database::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Clean up test data
async fn cleanup(db: &DatabaseConnection, ids: Vec<Uuid>) {
    for id in ids {
        let _ = Memos::delete_by_id(id).exec(db).await;
    }
}

#[tokio::test]
async fn test_create_memo() {
    let db = setup_db().await;
    let repo = MemoRepository::new(db.clone());

    let title = "Test Memo".to_string();
    let description = Some("Test Description".to_string());
    let date_to = Utc::now();

    let result = repo.create(title.clone(), description.clone(), date_to).await;

    assert!(result.is_ok());
    let memo = result.unwrap();
    assert_eq!(memo.title, title);
    assert_eq!(memo.description, description);
    assert_eq!(memo.completed, false);

    // Cleanup
    cleanup(&db, vec![memo.id]).await;
}

#[tokio::test]
async fn test_find_by_id() {
    let db = setup_db().await;
    let repo = MemoRepository::new(db.clone());

    // Create a memo first
    let date_to = Utc::now();
    let created = repo.create("Find Test".to_string(), None, date_to).await.unwrap();

    // Find it
    let result = repo.find_by_id(created.id).await;
    assert!(result.is_ok());

    let found = result.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, created.id);

    // Test not found
    let not_found = repo.find_by_id(Uuid::new_v4()).await;
    assert!(not_found.is_ok());
    assert!(not_found.unwrap().is_none());

    // Cleanup
    cleanup(&db, vec![created.id]).await;
}

#[tokio::test]
async fn test_list_pagination() {
    let db = setup_db().await;
    let repo = MemoRepository::new(db.clone());

    let date_to = Utc::now();

    // Create test memos
    let mut ids = Vec::new();
    for i in 1..=15 {
        let memo = repo.create(format!("Memo {}", i), None, date_to).await.unwrap();
        ids.push(memo.id);
    }

    // Test first page
    let (page1, total) = repo.list(1, 10).await.unwrap();
    assert_eq!(page1.len(), 10);
    assert!(total >= 15);

    // Test second page
    let (page2, _) = repo.list(2, 10).await.unwrap();
    assert!(page2.len() >= 5);

    // Cleanup
    cleanup(&db, ids).await;
}

#[tokio::test]
async fn test_update_memo() {
    let db = setup_db().await;
    let repo = MemoRepository::new(db.clone());

    let date_to = Utc::now();
    let created = repo.create("Original".to_string(), None, date_to).await.unwrap();

    // Update title
    let updated = repo.update(
        created.id,
        Some("Updated".to_string()),
        None,
        None,
        None,
    ).await.unwrap();

    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.id, created.id);

    // Cleanup
    cleanup(&db, vec![created.id]).await;
}

#[tokio::test]
async fn test_toggle_complete() {
    let db = setup_db().await;
    let repo = MemoRepository::new(db.clone());

    let date_to = Utc::now();
    let created = repo.create("Toggle Test".to_string(), None, date_to).await.unwrap();

    assert_eq!(created.completed, false);

    // Toggle to true
    let toggled1 = repo.toggle_complete(created.id).await.unwrap();
    assert_eq!(toggled1.completed, true);

    // Toggle back to false
    let toggled2 = repo.toggle_complete(created.id).await.unwrap();
    assert_eq!(toggled2.completed, false);

    // Cleanup
    cleanup(&db, vec![created.id]).await;
}

#[tokio::test]
async fn test_delete_memo() {
    let db = setup_db().await;
    let repo = MemoRepository::new(db.clone());

    let date_to = Utc::now();
    let created = repo.create("Delete Test".to_string(), None, date_to).await.unwrap();

    // Delete
    let result = repo.delete(created.id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows_affected, 1);

    // Verify deleted
    let found = repo.find_by_id(created.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_transaction_batch_create() {
    let db = setup_db().await;
    let repo = MemoRepository::new(db.clone());

    let date_to = Utc::now();

    let batch = vec![
        ("Batch 1".to_string(), None, date_to),
        ("Batch 2".to_string(), Some("Desc".to_string()), date_to),
        ("Batch 3".to_string(), None, date_to),
    ];

    let results = repo.create_batch(batch).await.unwrap();
    assert_eq!(results.len(), 3);

    let ids: Vec<Uuid> = results.iter().map(|m| m.id).collect();

    // Cleanup
    cleanup(&db, ids).await;
}

#[tokio::test]
async fn test_list_filtered() {
    let db = setup_db().await;
    let repo = MemoRepository::new(db.clone());

    let date_to = Utc::now();

    // Create completed and incomplete memos
    let mut ids = Vec::new();
    for i in 1..=5 {
        let memo = repo.create(format!("Completed {}", i), None, date_to).await.unwrap();
        repo.toggle_complete(memo.id).await.unwrap();
        ids.push(memo.id);
    }
    for i in 1..=5 {
        let memo = repo.create(format!("Incomplete {}", i), None, date_to).await.unwrap();
        ids.push(memo.id);
    }

    // Filter by completed
    let (completed, total_completed) = repo.list_filtered(
        1, 10, Some(true), None, None
    ).await.unwrap();
    assert!(completed.len() >= 5);
    assert!(completed.iter().all(|m| m.completed));

    // Filter by incomplete
    let (incomplete, total_incomplete) = repo.list_filtered(
        1, 10, Some(false), None, None
    ).await.unwrap();
    assert!(incomplete.len() >= 5);
    assert!(incomplete.iter().all(|m| !m.completed));

    // Cleanup
    cleanup(&db, ids).await;
}
```

2. **Create `.env.test` file** for automatic test configuration:

   ```bash
   cat > .env.test << 'EOF'
# Test Environment Configuration
# AUTOMATICALLY loaded when running: cargo test
# The application detects test mode with #[cfg(test)] and loads this file
# No manual copying needed - just run: cargo test

# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3737
APP_ENV=test  # For documentation (treated as development)

# Logging Configuration (quieter for tests)
RUST_LOG=warn,actix_web_template=debug
LOG_FORMAT=compact

# Database Configuration - Test Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test
DATABASE_MAX_CONNECTIONS=5
DATABASE_CONNECT_TIMEOUT=10

# CORS Configuration (permissive for tests)
CORS_ALLOWED_ORIGINS=*

# Request Configuration
MAX_REQUEST_SIZE=262144

# API Documentation (enabled for manual testing)
ENABLE_SWAGGER=true
EOF
   ```

3. **Add automatic `.env.test` loading** to `src/config/settings.rs`:

   Update the `Settings::load()` method:

   ```rust
   impl Settings {
       pub fn load() -> anyhow::Result<Self> {
           // Automatically load .env.test when running tests
           #[cfg(test)]
           {
               dotenvy::from_filename(".env.test").ok();
           }

           #[cfg(not(test))]
           {
               dotenvy::dotenv().ok();
           }

           let server = ServerConfig {
               // ... rest unchanged
   ```

   **What this does**:
   - When you run `cargo test`, Rust compiles with `cfg(test)` flag enabled
   - This code automatically loads `.env.test` instead of `.env`
   - Your tests get the test database URL automatically
   - No manual environment variable setup needed!

4. **Setup test database** (one-time setup):

   ```bash
   # Create test database
   psql postgresql://postgres:postgres@localhost:5432/postgres -c "CREATE DATABASE memos_test;"

   # Or using Docker
   docker exec -it postgres_container psql -U postgres -c "CREATE DATABASE memos_test;"

   # Run migrations on test database (from project root)
   DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test cargo run -p migration

   # Verify tables created
   psql postgresql://postgres:postgres@localhost:5432/memos_test -c "\dt"
   ```

   **Expected output**:
   ```
               List of relations
    Schema |      Name       | Type  |  Owner
   --------+-----------------+-------+----------
    public | memos           | table | postgres
    public | seaql_migrations| table | postgres
   ```

   **Why a separate test database?** Tests create and delete data. You don't want to mix test data with your development data!

5. **Run repository tests**:
   ```bash
   cargo test repository_tests
   ```

   **That's it!** The application automatically loads `.env.test` when running tests. No need to set DATABASE_URL manually!

**Verify**:
All repository tests should pass.

---

## Checkpoint

Run these commands to verify everything works:

```bash
# Build should succeed
cargo build

# Repository tests should pass
cargo test repository_tests

# Check specific operations
cargo test test_create_memo
cargo test test_transaction_batch_create
```

### Expected Results

- All repository methods compile
- CRUD operations work correctly
- Pagination returns correct page sizes
- Filtering works for completed status
- Transactions succeed or rollback properly
- Tests pass with test database

---

## Common Issues and Solutions

### Issue: RecordNotFound error

**Symptoms**: Tests fail with "Memo not found"

**Cause**: Test data not created or already deleted

**Solution**:
```rust
// Always verify creation succeeded
let memo = repo.create(/* ... */).await?;
assert!(memo.id != Uuid::nil());

// Use separate test database
DATABASE_URL=postgresql://localhost/memos_test cargo test
```

---

### Issue: Transaction deadlock

**Symptoms**: Tests hang or timeout

**Cause**: Two transactions waiting for each other

**Solution**:
```rust
// Keep transactions short
let txn = db.begin().await?;
// Do minimal work
txn.commit().await?;

// Run tests serially
cargo test -- --test-threads=1
```

---

### Issue: Pagination returns wrong count

**Symptoms**: Total count doesn't match actual rows

**Cause**: Count query includes filters, pagination applied twice

**Solution**:
```rust
// Clone query before adding pagination
let base_query = Memos::find().filter(/* ... */);
let total = base_query.clone().count(&db).await?;
let memos = base_query.limit(per_page).offset(offset).all(&db).await?;
```

---
## Code Review

### Key Design Principles Demonstrated
- **Repository encapsulation** keeps every database interaction behind `MemoRepository`, preventing handlers and services from importing SeaORM directly.
- **Composable query builders** let filters, sorting, and pagination layer on without hand-written SQL.
- **Stateless design**: Repository methods take a `&DatabaseConnection`, which keeps the API simple and test-friendly.
- **Uniform return types**: Every method surfaces `Result<_, DbErr>`, making error conversion in the service layer predictable.

### Architecture Benefits
- **Maintainability**: Centralized query logic means schema or index changes update one module instead of every caller.
- **Test isolation**: Repository integration tests run against a real database while reusing production code paths.
- **Performance tuning**: Having a single choke point for data access makes adding tracing, caching, or metrics straightforward.
- **Separation of concerns**: Business logic stays focused on workflows while persistence details live in one place.

### Complete Repository Structure
```rust
pub struct MemoRepository;

impl MemoRepository {
    pub async fn find_all(
        db: &DatabaseConnection,
        limit: u64,
        offset: u64,
        completed: Option<bool>,
        sort_by: &str,
        order: &str,
    ) -> Result<(Vec<memos::Model>, u64), DbErr> {
        let mut query = Memos::find();
        if let Some(completed_filter) = completed {
            query = query.filter(memos::Column::Completed.eq(completed_filter));
        }
        let sort_column = match sort_by {
            "title" => memos::Column::Title,
            "date_to" => memos::Column::DateTo,
            "completed" => memos::Column::Completed,
            "updated_at" => memos::Column::UpdatedAt,
            _ => memos::Column::CreatedAt,
        };
        query = if order == "asc" {
            query.order_by_asc(sort_column)
        } else {
            query.order_by_desc(sort_column)
        };

        let total = query.clone().count(db).await?;
        let memos = query.limit(limit).offset(offset).all(db).await?;
        Ok((memos, total))
    }

    pub async fn create(
        db: &DatabaseConnection,
        title: String,
        description: Option<String>,
        date_to: DateTime<Utc>,
    ) -> Result<memos::Model, DbErr> {
        use sea_orm::ActiveValue::Set;

        let now = Utc::now();
        let new_memo = memos::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(title),
            description: Set(description),
            date_to: Set(date_to.into()),
            completed: Set(false),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        new_memo.insert(db).await
    }
}
```

---

## Understanding ACID Properties

### Atomicity
All operations in a transaction succeed or fail together:
```rust
let txn = db.begin().await?;
create_memo(&txn, memo1).await?; // Succeeds
create_memo(&txn, memo2).await?; // Fails
// Both rolled back - database unchanged
```

### Consistency
Database constraints are enforced:
```rust
// Title constraint: NOT NULL
let memo = memos::ActiveModel {
    title: Set("".to_string()), // Violates constraint
    // ...
};
// Insert fails, database stays consistent
```

### Isolation
Concurrent transactions don't interfere:
```rust
// Transaction 1: Reading memos
// Transaction 2: Creating memo
// Transaction 1 sees snapshot, not affected
```

### Durability
Committed changes persist even after crashes:
```rust
txn.commit().await?; // Data saved to disk
// Server crashes
// Data still there on restart
```

---

## Testing

### 1. Dedicated Repository Suite
Run the focused repository tests against your isolated test database:
```bash
cargo test repository_tests
```
These cover create/read/update/delete paths, pagination metadata, and filtering logic, ensuring every SeaORM query behaves exactly as documented.

### 2. Transaction Workflows
Keep a serialized run for transaction-heavy tests so concurrency issues surface immediately:
```bash
cargo test test_transaction_batch_create -- --test-threads=1
```
Running single-threaded forces the test to exercise commit/rollback logic deterministically.

### 3. Pagination & Filtering Regression Tests
Use substring filters to re-run only the pagination scenarios when you tweak queries:
```bash
cargo test pagination_filtering
```
Assertions should validate counts, offsets, and sort orders for multiple parameter combinations.

### 4. Manual Verification Against Test DB
Inspect the test database after running the suite to ensure cleanup executes:
```bash
psql "$TEST_DATABASE_URL" -c "SELECT id, title, completed FROM memos ORDER BY updated_at DESC LIMIT 5"
```
Seeing only test data (or an empty result) confirms fixtures don't leak into development databases.

## Summary

Congratulations! You've built a complete repository layer. You now have:

1. **Repository pattern** - Clean data access layer
2. **CRUD operations** - Create, Read, Update, Delete
3. **Pagination** - Offset/limit for large datasets
4. **Filtering** - Query by completion status
5. **Sorting** - Order by multiple columns
6. **Transactions** - Atomic multi-step operations
7. **Error handling** - Proper DbErr propagation
8. **Comprehensive tests** - Verified with test database
9. **Type safety** - Compile-time query validation

### Key Takeaways

- **Repository pattern** separates data access from business logic
- **SeaORM** provides type-safe query building
- **Pagination** prevents loading too much data
- **Transactions** ensure data consistency
- **Testing** requires separate test database
- **ACID** properties guarantee reliability

### Architecture So Far

```
┌─────────────────────────────────────┐
│        HTTP Requests                │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Handlers                           │
│  - Validate DTOs                    │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Service Layer (Next Chapter)       │
│  - Business logic                   │
│  - DTO ↔ Entity conversion          │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Repository Layer (This Chapter!)   │
│  - MemoRepository                   │
│  - CRUD operations                  │
│  - Pagination, filtering, sorting   │
│  - Transactions                     │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Entities & SeaORM                  │
│  - Database models                  │
│  - Query builder                    │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  PostgreSQL Database                │
└─────────────────────────────────────┘
```

---

## Next Steps

### Required: Chapter 7 - Service Layer - Business Logic and Transactions

You'll compose repositories into cohesive business workflows, manage transactions that span multiple operations, and ensure DTO conversions stay consistent. Expect to plug validation and persistence together so handlers can remain thin.

### Optional Exercises

1. **Challenge**: Add a repository method that performs text search across memo content using `contains` or full-text indexing.
2. **Challenge**: Experiment with wrapping repository calls in explicit transactions and observe how nested transactions behave.
3. **Challenge**: Prototype a simple in-memory cache (e.g., `DashMap`) around read operations to profile the impact on repeated queries.

---

## Additional Resources

### Repository Pattern
- [Martin Fowler - Repository](https://martinfowler.com/eaaCatalog/repository.html) - Pattern explanation
- [Repository Pattern in Rust](https://doc.rust-lang.org/book/ch17-03-oo-design-patterns.html) - Rust Book

### SeaORM
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/docs/index) - Official docs
- [Query Builder](https://www.sea-ql.org/SeaORM/docs/basic-crud/select) - SELECT queries
- [Transactions](https://www.sea-ql.org/SeaORM/docs/advanced-query/transaction) - Transaction guide

### Database Concepts
- [ACID Properties](https://en.wikipedia.org/wiki/ACID) - Wikipedia
- [Transaction Isolation](https://www.postgresql.org/docs/current/transaction-iso.html) - PostgreSQL docs
- [Pagination Best Practices](https://www.citusdata.com/blog/2016/03/30/five-ways-to-paginate/) - Citus blog

---

**Ready to add business logic? Let's move on to [Chapter 7: Service Layer](chapter-07.md)!**
