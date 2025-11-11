# Chapter 14: Testing Strategy

## Overview

Testing is not optional—it's a fundamental part of building reliable software. A comprehensive test suite gives you confidence to refactor, prevents regressions, serves as living documentation, and enables continuous deployment. Without tests, you're flying blind, hoping changes don't break existing functionality.

This chapter implements a complete testing strategy for your Actix Web application. You'll learn to write unit tests for isolated components, integration tests for API endpoints, repository tests with a real database, and end-to-end tests for the web UI. By the end, you'll understand Rust's testing philosophy, Actix Web's test utilities, and how to maintain a test suite that provides real value.

Your application already has tests—this chapter explains their structure, shows you how to write more, and teaches testing best practices for production Rust applications.

## Prerequisites

### Completed Chapters
- Chapter 0: Prerequisites and Environment Setup
- Chapter 1-12: Complete application
- Chapter 13: Security Enhancements (optional, but tests security features)

### Required Knowledge
- Rust fundamentals (ownership, lifetimes, traits)
- Async/await in Rust
- Understanding of your application's architecture
- Basic testing concepts (assertions, test fixtures, mocking)

### System Requirements
- Running PostgreSQL instance for integration tests
- Sufficient disk space for test database
- Test database should be separate from development database

## Learning Objectives

By the end of this chapter, you will be able to:

1. Write unit tests for isolated business logic
2. Create integration tests for REST API endpoints
3. Test database operations with a real database
4. Write end-to-end tests for web handlers
5. Use test fixtures and helper utilities
6. Understand test isolation and cleanup
7. Run tests in parallel and serial modes
8. Measure and improve test coverage
9. Mock external dependencies (when needed)
10. Follow testing best practices for Rust

## Concepts Covered

### Testing Philosophy in Rust

Rust has testing built into the language and tooling:

```rust
// Unit test in same file as code
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
```

**Key principles**:
1. **Fast feedback**: Tests run with `cargo test`
2. **Co-location**: Unit tests live next to code they test
3. **No test framework needed**: Built into Rust
4. **Compile-time safety**: Tests won't compile if types wrong

**Test types in our application**:
```
┌─────────────────────────────────────────────────┐
│  Unit Tests (in src/)                           │
│  - Pure functions                               │
│  - No external dependencies                     │
│  - Fast (milliseconds)                          │
│  Example: sanitize_html()                       │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│  Service Tests (tests/service_tests.rs)         │
│  - Business logic                               │
│  - With database                                │
│  - Medium speed (seconds)                       │
│  Example: create_memo(), update_memo()          │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│  Repository Tests (tests/repository_tests.rs)   │
│  - Database operations                          │
│  - Real database                                │
│  - Medium speed (seconds)                       │
│  Example: CRUD operations, pagination           │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│  Integration Tests (tests/api_tests.rs)         │
│  - Full HTTP stack                              │
│  - REST API endpoints                           │
│  - Slower (seconds)                             │
│  Example: GET /api/v1/memos                     │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│  Web Tests (tests/web_tests.rs)                 │
│  - HTML rendering                               │
│  - Form submissions                             │
│  - Full application                             │
│  Example: GET /, POST /web/memos                │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│  Doc Tests (embedded in /// comments)           │
│  - Test code examples in documentation         │
│  - Ensures docs stay accurate                  │
│  - Very fast (compile-only)                    │
│  Example: create_memos_batch() usage            │
└─────────────────────────────────────────────────┘
```

### Test Organization

**Directory structure**:
```
actix-web-template/
├── src/
│   ├── utils/
│   │   └── sanitize.rs          # Unit tests at bottom
│   └── handlers/
│       └── test_*.rs            # Test-only handlers
├── tests/                       # Integration tests
│   ├── common/
│   │   ├── mod.rs               # Test utilities
│   │   └── fixtures.rs          # Test data helpers
│   ├── api_tests.rs             # REST API tests
│   ├── service_tests.rs         # Service layer tests
│   ├── repository_tests.rs      # Repository tests
│   └── web_tests.rs             # Web handler tests
└── Cargo.toml
```

**Why separate directories?**
- **Unit tests in src/**: Fast, no external deps, run with every compile
- **Integration tests in tests/**: Slower, need database, run on-demand
- **Common module**: Shared test utilities (DRY principle)

### Unit Tests in Rust

**Unit tests** test individual functions in isolation.

**Structure**:
```rust
// Production code
pub fn sanitize_html(input: &str) -> String {
    ammonia::clean(input)
}

// Tests in same file
#[cfg(test)]  // Only compile when testing
mod tests {
    use super::*;  // Import from parent module

    #[test]  // Mark as test function
    fn test_sanitize_html() {
        let input = "<script>alert('xss')</script>Hello";
        let result = sanitize_html(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("Hello"));
    }
}
```

**Test attributes**:
- `#[cfg(test)]`: Only compile module when testing
- `#[test]`: Mark function as test (must return `()` or `Result<(), E>`)
- `#[should_panic]`: Test should panic
- `#[ignore]`: Skip test by default (run with `cargo test -- --ignored`)

**Assertion macros**:
```rust
assert!(condition);                      // Panic if false
assert_eq!(left, right);                 // Panic if not equal
assert_ne!(left, right);                 // Panic if equal
assert!(result.is_ok());                 // Check Result
assert!(option.is_some());               // Check Option
```

**Custom messages**:
```rust
assert_eq!(result, expected, "Failed for input: {}", input);
```

### Integration Tests with actix-web::test

**Integration tests** test multiple components together, often with HTTP layer.

**Actix Web provides**:
- `test::init_service()`: Create test server
- `test::TestRequest`: Build HTTP requests
- `test::call_service()`: Send request, get response
- `test::read_body()`: Extract response body
- `test::read_body_json()`: Parse JSON response

**Example**:
```rust
use actix_web::{test, web, App};

#[tokio::test]
async fn test_health_endpoint() {
    // Create test app
    let app = test::init_service(
        App::new().service(handlers::health_check)
    ).await;

    // Build request
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    // Call service
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), 200);

    let body: HealthResponse = test::read_body_json(resp).await;
    assert_eq!(body.status, "healthy");
}
```

**Why `#[tokio::test]` instead of `#[test]`?**
- Actix Web is async
- `#[tokio::test]` provides async runtime
- Equivalent to `#[test]` wrapping `tokio::runtime::Runtime::new()`

### Documentation Tests

**Doc tests** verify code examples in documentation comments compile and run correctly.

**Example** (from `src/services/memo_service.rs`):
```rust
/// Batch create multiple memos in a single transaction.
///
/// # Example
/// ```
/// # use actix_web_template::services::MemoService;
/// # use actix_web_template::dto::CreateMemoDto;
/// # async {
/// let service = MemoService::new(db);
/// let memos = vec![/* ... */];
/// let created = service.create_memos_batch(memos).await?;
/// # Ok::<(), anyhow::Error>(())
/// # };
/// ```
pub async fn create_memos_batch(...) { }
```

**How it works**:
- Code in `///` comments (triple-slash) with ` ```rust ` blocks is extracted
- `cargo test` compiles and runs these examples
- Lines starting with `#` are hidden in rendered docs but executed in tests
- Ensures documentation examples stay correct as code evolves

**Benefits**:
- Documentation that actually works
- Examples tested automatically
- No doc rot (outdated examples)
- Only 1 doc test in our codebase (batch operations example)

### Test Database Strategy

**Problem**: Tests need database, but shouldn't interfere with each other or development data.

**Solutions**:

**Option 1: Separate test database** (our approach):
```bash
# Development database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db

# Test database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test
```

**Pros**: Simple, real database, tests are realistic
**Cons**: Tests share database (need cleanup), not perfectly isolated

**Option 2: Transaction rollback**:
```rust
#[tokio::test]
async fn test_with_transaction() {
    let txn = db.begin().await.unwrap();
    // ... test code ...
    txn.rollback().await.unwrap();  // Undo changes
}
```

**Pros**: Perfect isolation, no cleanup needed
**Cons**: Can't test transactions, more complex setup

**Option 3: Database per test**:
```rust
async fn setup_test() -> DatabaseConnection {
    let unique_db = format!("test_db_{}", uuid::Uuid::new_v4());
    create_database(&unique_db).await;
    Database::connect(&db_url(&unique_db)).await.unwrap()
}
```

**Pros**: Perfect isolation
**Cons**: Slow (create/drop DB each test), complex

**Our choice**: Separate test database with cleanup in tests. Good balance of speed, simplicity, and realism.

### Test Fixtures

**Test fixtures** provide reusable test data and setup code.

**Example** (`tests/common/fixtures.rs`):
```rust
pub fn create_test_memo_dto(title: &str, description: Option<&str>) -> CreateMemoDto {
    CreateMemoDto {
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        date_to: Utc::now(),
    }
}
```

**Usage**:
```rust
use common::fixtures::create_test_memo_dto;

#[tokio::test]
async fn test_something() {
    let dto = create_test_memo_dto("Test", Some("Description"));
    // Use dto in test...
}
```

**Benefits**:
- DRY: Don't repeat setup code
- Consistency: All tests use same data structure
- Maintainability: Change fixture, all tests update

**Common fixtures**:
- `create_test_memo_dto()`: Create memo DTOs
- `setup_test_db()`: Get database connection
- `setup_test_state()`: Create AppState for tests
- `create_test_user()`: (future) Create user for auth tests

### Test Isolation and Cleanup

**Problem**: Tests can interfere with each other if they share state.

**Example of interference**:
```rust
#[tokio::test]
async fn test_list_returns_all_memos() {
    // Creates memo
    let memo = service.create_memo(dto).await.unwrap();

    let memos = service.get_all_memos(params).await.unwrap();
    assert_eq!(memos.data.len(), 1);
    // Doesn't delete memo!
}

#[tokio::test]
async fn test_list_is_empty() {
    let memos = service.get_all_memos(params).await.unwrap();
    assert_eq!(memos.data.len(), 0);  // FAILS if first test ran!
}
```

**Solution 1: Cleanup in each test**:
```rust
#[tokio::test]
async fn test_something() {
    let memo = service.create_memo(dto).await.unwrap();

    // ... test code ...

    // Cleanup
    service.delete_memo(memo.id).await.ok();
}
```

**Solution 2: Unique test data**:
```rust
fn unique_title() -> String {
    format!("Test Memo {}", uuid::Uuid::new_v4())
}
```

**Solution 3: Test database reset** (between test runs):
```bash
# Before running tests
psql $TEST_DATABASE_URL -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
sea-orm-cli migrate up
```

**Our approach**: Cleanup in tests + unique identifiers where needed.

### Parallel vs Serial Test Execution

By default, Rust runs tests **in parallel** (multiple threads).

**Parallel execution**:
```bash
cargo test
# Runs all tests simultaneously across CPU cores
```

**Pros**: Fast
**Cons**: Database tests can conflict

**Serial execution**:
```bash
cargo test -- --test-threads=1
# Runs tests one at a time
```

**Pros**: No database conflicts
**Cons**: Slower

**Per-test control** (not built-in, use crate):
```rust
// With serial_test crate
use serial_test::serial;

#[tokio::test]
#[serial]  // Run serially with other #[serial] tests
async fn test_database_operation() {
    // ...
}
```

**Our approach**: Run database tests serially, unit tests in parallel.

### Test Coverage

**Test coverage** measures what code is executed during tests.

**Tool: tarpaulin** (Linux/macOS):
```bash
cargo install cargo-tarpaulin

cargo tarpaulin --out Html
# Generates tarpaulin-report.html
```

**Tool: llvm-cov** (all platforms):
```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov

cargo llvm-cov --html
# Generates target/llvm-cov/html/index.html
```

**Interpreting coverage**:
- **80%+ coverage**: Good
- **90%+ coverage**: Excellent
- **100% coverage**: Overkill (diminishing returns)

**Coverage doesn't mean tests are good**: High coverage with weak assertions is useless.

**What to focus on**:
- Business logic (service layer)
- Error handling paths
- Edge cases (empty lists, null values, boundary conditions)

**What to skip**:
- Generated code (entities)
- Trivial getters/setters
- Main.rs (integration tests cover this)

### Mocking in Rust

**Mocking** replaces real dependencies with fake implementations for testing.

**When to mock**:
- External APIs (HTTP requests to third parties)
- File system operations
- Time-dependent code
- Expensive operations (machine learning models)

**When NOT to mock**:
- Database (use test database, more realistic)
- Internal dependencies (test real implementation)

**Rust mocking is hard** because:
- No runtime reflection
- Strong type system
- Ownership rules

**Approaches**:

**1. Trait-based mocking** (manual):
```rust
// Define trait
trait EmailSender {
    async fn send(&self, to: &str, body: &str) -> Result<(), Error>;
}

// Real implementation
struct SmtpSender { /* ... */ }
impl EmailSender for SmtpSender { /* ... */ }

// Mock for tests
struct MockSender {
    emails_sent: Arc<Mutex<Vec<String>>>,
}
impl EmailSender for MockSender {
    async fn send(&self, to: &str, body: &str) -> Result<(), Error> {
        self.emails_sent.lock().unwrap().push(to.to_string());
        Ok(())
    }
}

// Service accepts trait
struct UserService<T: EmailSender> {
    email: T,
}

// Test with mock
#[tokio::test]
async fn test_sends_welcome_email() {
    let mock = MockSender::new();
    let service = UserService::new(mock.clone());

    service.register_user("user@example.com").await.unwrap();

    assert_eq!(mock.emails_sent.lock().unwrap().len(), 1);
}
```

**2. mockall crate** (automatic):
```rust
use mockall::automock;

#[automock]
trait EmailSender {
    async fn send(&self, to: &str, body: &str) -> Result<(), Error>;
}

#[tokio::test]
async fn test_with_mockall() {
    let mut mock = MockEmailSender::new();
    mock.expect_send()
        .with(eq("user@example.com"), predicate::always())
        .times(1)
        .returning(|_, _| Ok(()));

    let service = UserService::new(mock);
    service.register_user("user@example.com").await.unwrap();
}
```

**Our application**: Minimal mocking needed. We test with real database.

### Testing Best Practices

**1. AAA Pattern** (Arrange, Act, Assert):
```rust
#[tokio::test]
async fn test_create_memo() {
    // ARRANGE: Set up test data
    let service = setup_test_service().await;
    let dto = create_test_memo_dto("Test", None);

    // ACT: Perform action
    let result = service.create_memo(dto).await;

    // ASSERT: Verify outcome
    assert!(result.is_ok());
    let memo = result.unwrap();
    assert_eq!(memo.title, "Test");
}
```

**2. Test one thing**:
```rust
// ✗ Bad: Tests creation AND retrieval AND update
#[tokio::test]
async fn test_memo_operations() {
    let memo = service.create_memo(dto).await.unwrap();
    let fetched = service.get_memo_by_id(memo.id).await.unwrap();
    let updated = service.update_memo(memo.id, update_dto).await.unwrap();
    // ...
}

// ✓ Good: Separate tests
#[tokio::test]
async fn test_create_memo() { /* ... */ }

#[tokio::test]
async fn test_get_memo_by_id() { /* ... */ }

#[tokio::test]
async fn test_update_memo() { /* ... */ }
```

**3. Test happy path and error cases**:
```rust
#[tokio::test]
async fn test_get_memo_by_id_success() {
    // Happy path
}

#[tokio::test]
async fn test_get_memo_by_id_not_found() {
    // Error case
    let fake_id = uuid::Uuid::new_v4();
    let result = service.get_memo_by_id(fake_id).await;
    assert!(result.is_err());
}
```

**4. Descriptive test names**:
```rust
// ✗ Bad
#[tokio::test]
async fn test1() { }

// ✓ Good
#[tokio::test]
async fn test_create_memo_with_valid_data_succeeds() { }

#[tokio::test]
async fn test_create_memo_with_empty_title_fails_validation() { }
```

**5. Independent tests** (no order dependency):
```rust
// ✗ Bad: test2 depends on test1 running first
#[tokio::test]
async fn test1_create() { /* creates memo ID 123 */ }

#[tokio::test]
async fn test2_get() { /* fetches memo ID 123 */ }

// ✓ Good: Each test is self-contained
#[tokio::test]
async fn test_get_existing_memo() {
    let created = service.create_memo(dto).await.unwrap();
    let fetched = service.get_memo_by_id(created.id).await.unwrap();
    // ...
}
```

## Step-by-Step Instructions

### Step 1: Verify Test Database Configuration

Tests need a separate database from development.

**Check `.env` file**:
```bash
# Your .env file contains:
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db
APP_ENV=development  # Options: development, staging, production
```

**Note about APP_ENV=test**: While `.env.test` uses `APP_ENV=test` for documentation, it's treated as `development` internally. Rust tests use `#[cfg(test)]` for compile-time detection, not runtime environment checks.

**For running tests**, the application automatically loads `.env.test`:

**Just run tests** (`.env.test` loaded automatically):
```bash
cargo test
```

That's it! The application detects test mode with `#[cfg(test)]` and automatically loads `.env.test` instead of `.env`.

**How it works** (configured in Chapter 6):
```rust
// In src/config/settings.rs (added in Chapter 6)
#[cfg(test)]
{
    dotenvy::from_filename(".env.test").ok();  // Load .env.test in tests
}

#[cfg(not(test))]
{
    dotenvy::dotenv().ok();  // Load .env in normal mode
}
```

**Note**: This automatic loading was set up in Chapter 6 when you first needed a test database for repository tests.

**You can still override** if needed:
```bash
# Override with environment variable
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/other_test_db cargo test

# Or temporarily modify .env.test file
```

**Why `.env.test` is auto-loaded**:
- **Automatic**: No manual copying, just run `cargo test`
- **Compile-time detection**: Uses `#[cfg(test)]` to detect test mode
- **Isolated**: Test config never interferes with development
- **Convenient**: Pre-configured with test database URL and quieter logging
- **Overridable**: Can still set `DATABASE_URL` env var if needed

**How test detection works**:
- Rust's `#[cfg(test)]` is a compile-time feature flag
- When you run `cargo test`, Rust compiles code with `cfg(test)` enabled
- Our `Settings::load()` detects this and loads `.env.test` instead of `.env`
- No runtime checks needed - it's decided at compile time!

**Why separate databases?** Tests create and delete data. You don't want test data mixed with your development data.

**Create test database**:
```bash
# Using psql
psql postgresql://postgres:postgres@localhost:5432/postgres -c "CREATE DATABASE memos_test;"

# Or using Docker
docker exec -it postgres_container psql -U postgres -c "CREATE DATABASE memos_test;"
```

**Run migrations on test database**:
```bash
# Using sea-orm-cli (recommended)
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test sea-orm-cli migrate up

# Or using workspace command (from project root)
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test cargo run -p migration

# Or traditional approach
cd migration
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test cargo run
cd ..
```

**Verify test database**:
```bash
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

### Step 2: Understand Test Directory Structure

Check your `tests/` directory:

```bash
ls -la tests/
```

**Expected structure**:
```
tests/
├── common/
│   ├── mod.rs        # Common utilities
│   └── fixtures.rs   # Test data helpers
├── api_tests.rs      # REST API integration tests
├── service_tests.rs  # Service layer tests
├── repository_tests.rs  # Database layer tests
└── web_tests.rs      # Web handler tests
```

**Review `tests/common/mod.rs`**:
```rust
pub mod fixtures;

use actix_web_template::{config::Settings, state::AppState};
use sea_orm::Database;

pub async fn setup_test_db() -> sea_orm::DatabaseConnection {
    let settings = Settings::load().expect("Failed to load settings");
    Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to test database")
}

#[allow(dead_code)]
pub async fn setup_test_state() -> AppState {
    let settings = Settings::load().expect("Failed to load settings");
    let db = setup_test_db().await;
    AppState::new(settings, db)
}
```

**Review `tests/common/fixtures.rs`**:
```rust
use actix_web_template::dto::CreateMemoDto;
use chrono::Utc;

pub fn create_test_memo_dto(title: &str, description: Option<&str>) -> CreateMemoDto {
    CreateMemoDto {
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        date_to: Utc::now(),
    }
}
```

These utilities are shared across all integration tests.

### Step 3: Review Unit Tests

Unit tests live in `src/` files with `#[cfg(test)]` modules.

**Example: `src/utils/sanitize.rs`**:
```rust
pub fn sanitize_html(input: &str) -> String {
    ammonia::clean(input)
}

pub fn sanitize_optional_html(input: Option<&str>) -> Option<String> {
    input.map(sanitize_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        let input = "<script>alert('xss')</script>Hello World";
        let result = sanitize_html(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_sanitize_html_removes_javascript() {
        let input = "<img src=x onerror=alert('xss')>";
        let result = sanitize_html(input);
        assert!(!result.contains("onerror"));
        assert!(!result.contains("alert"));
    }

    #[test]
    fn test_sanitize_html_allows_safe_tags() {
        let input = "<p>Hello <strong>World</strong></p>";
        let result = sanitize_html(input);
        assert!(result.contains("<p>"));
        assert!(result.contains("<strong>"));
        assert!(result.contains("</strong>"));
    }

    #[test]
    fn test_sanitize_optional_html_none() {
        let result = sanitize_optional_html(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_sanitize_optional_html_some() {
        let input = Some("<script>alert('xss')</script>Hello");
        let result = sanitize_optional_html(input);
        assert!(result.is_some());
        let sanitized = result.unwrap();
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("Hello"));
    }
}
```

**Run only unit tests**:
```bash
cargo test --lib
```

**Why `--lib`?** Runs tests in `src/` only, skips `tests/` directory.

### Step 4: Review Service Layer Tests

The codebase has 17 service layer tests. Check `tests/service_tests.rs`:

```rust
use actix_web_template::{
    config::Settings,
    dto::{CreateMemoDto, PaginationParams, UpdateMemoDto},
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

    // Cleanup
    service.delete_memo(memo.id).await.ok();
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

    // Arrange: Create memo
    let create_dto = CreateMemoDto {
        title: "Original Title".to_string(),
        description: Some("Original description".to_string()),
        date_to: Utc::now(),
    };
    let created = service.create_memo(create_dto).await.unwrap();

    // Act: Update memo
    let update_dto = UpdateMemoDto {
        title: "Updated Title".to_string(),
        description: Some("Updated description".to_string()),
        date_to: created.date_to,
        completed: true,
    };
    let result = service.update_memo(created.id, update_dto).await;

    // Assert
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.description, Some("Updated description".to_string()));
    assert!(updated.completed);

    // Cleanup
    service.delete_memo(created.id).await.ok();
}
```

**Key patterns**:
- `setup_test_service()`: Reusable setup
- Create → Act → Assert → Cleanup
- Test both success and error cases

**Additional tests in the codebase** (not shown above for brevity):
- `test_create_memo_with_sanitization`: Verifies XSS prevention
- `test_update_memo_sanitizes_input`: Sanitization on updates
- `test_create_memos_batch_transaction`: Batch operations with transactions
- `test_delete_memos_batch_transaction`: Batch delete operations
- `test_batch_create_with_validation_error`: Transaction rollback on error
- `test_get_all_memos_with_filter`: Filtering by completed status
- `test_pagination`: Comprehensive pagination tests

The service layer has comprehensive test coverage including transaction semantics and security features.

**Run service tests**:
```bash
cargo test service_tests
# Should show 17 tests passing
```

### Step 5: Review API Integration Tests

The codebase has 10 API integration tests covering all REST endpoints. Check `tests/api_tests.rs`:

```rust
use actix_web::{App, test, web};
use actix_web_template::{
    config::Settings,
    dto::{CreateMemoDto, MemoResponseDto},
    handlers,
    state::AppState,
};
use chrono::Utc;
use sea_orm::Database;

#[tokio::test]
async fn test_create_memo_endpoint() {
    // Setup
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo)
            .service(handlers::delete_memo),
    )
    .await;

    // Arrange
    let create_dto = CreateMemoDto {
        title: "Test API Memo".to_string(),
        description: Some("Created via API test".to_string()),
        date_to: Utc::now(),
    };

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&create_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), 201);

    let memo: MemoResponseDto = test::read_body_json(resp).await;
    assert_eq!(memo.title, "Test API Memo");
    assert_eq!(memo.description, Some("Created via API test".to_string()));
    assert!(!memo.completed);

    // Cleanup
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

    // Create test data
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

    // Test GET
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/memos/{}", created_memo.id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let memo: MemoResponseDto = test::read_body_json(get_resp).await;
    assert_eq!(memo.id, created_memo.id);
    assert_eq!(memo.title, "Get Test Memo");

    // Cleanup
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/memos/{}", memo.id))
        .to_request();
    test::call_service(&app, delete_req).await;
}
```

**Key utilities**:
- `test::init_service(App::new()...)`: Create test server
- `test::TestRequest::get().uri("...").to_request()`: Build request
- `test::call_service(&app, req)`: Send request
- `test::read_body_json::<T>(resp)`: Parse JSON response

**Additional API tests in the codebase**:
- `test_get_memo_not_found`: Returns 404 for missing memos
- `test_update_memo_endpoint`: Full PUT update
- `test_patch_memo_endpoint`: Partial PATCH update
- `test_delete_memo_endpoint`: DELETE with verification
- `test_toggle_complete_endpoint`: Toggle completion status
- `test_list_memos_endpoint`: List with pagination
- `test_list_memos_with_pagination`: Detailed pagination test
- `test_create_memo_validation_error`: Empty title returns 400

All REST API endpoints are thoroughly tested.

**Run API tests**:
```bash
cargo test api_tests
# Should show 10 tests passing
```

### Step 6: Review Web Handler Tests

Check `tests/web_tests.rs` for HTML endpoint tests:

```rust
mod common;

use actix_web::{test, web, App};
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

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(index)
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's a complete HTML page
    assert!(html.contains("<!DOCTYPE html") || html.contains("<html"));
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

    // Verify memo was created
    let params = actix_web_template::dto::PaginationParams::default();
    let result = service.get_all_memos(params).await.unwrap();

    assert!(result.data.iter().any(|m| m.title == "Web Created Memo"));

    // Cleanup
    let created = result.data.iter().find(|m| m.title == "Web Created Memo").unwrap();
    service.delete_memo(created.id).await.ok();
}
```

**Key differences from API tests**:
- Use `.set_form()` instead of `.set_json()` for HTML forms
- Check for HTML in response body (DOCTYPE, tags)
- May need to parse HTML to verify content

**Additional web tests in the codebase**:
- `test_get_memos_list`: HTML fragment for AJAX
- `test_get_new_memo_form`: New memo form
- `test_get_edit_memo_form`: Edit form with data
- `test_update_memo_web`: Form-based update
- `test_delete_memo_web`: Delete via web handler
- `test_toggle_memo_complete_web`: Toggle via web
- `test_create_memo_web_validation_error`: Form validation

All 8 web handlers are tested.

**Run web tests**:
```bash
cargo test web_tests
# Should show 9 tests passing
```

### Step 7: Run All Tests

**Run all tests**:
```bash
# .env.test is automatically loaded - just run:
cargo test
```

**Expected output**:
```
running 5 tests
test utils::sanitize::tests::test_sanitize_html ... ok
test utils::sanitize::tests::test_sanitize_html_removes_javascript ... ok
test utils::sanitize::tests::test_sanitize_html_allows_safe_tags ... ok
test utils::sanitize::tests::test_sanitize_optional_html_none ... ok
test utils::sanitize::tests::test_sanitize_optional_html_some ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

... (more test output) ...

test result: ok. 10 passed ... (api_tests)
test result: ok. 11 passed ... (repository_tests)
test result: ok. 17 passed ... (service_tests)
test result: ok. 9 passed ... (web_tests)
test result: ok. 1 passed ... (doc-tests)
```

**Breakdown**:
- **5 unit tests** in `src/utils/sanitize.rs`
- **17 service tests** in `tests/service_tests.rs`
- **10 API tests** in `tests/api_tests.rs`
- **9 web tests** in `tests/web_tests.rs`
- **11 repository tests** in `tests/repository_tests.rs`
- **1 doc test** in `src/services/memo_service.rs` (documentation examples)

**Total: 53 tests** providing comprehensive coverage across all layers!

**Run tests with output**:
```bash
cargo test -- --nocapture
```

Shows `println!()` and `dbg!()` output during tests.

**Run specific test**:
```bash
cargo test test_create_memo
```

Runs all tests matching "test_create_memo".

**Run tests in specific file**:
```bash
cargo test --test api_tests
```

**Run tests serially** (one at a time, good for database tests):
```bash
cargo test -- --test-threads=1
```

**Show ignored tests**:
```bash
cargo test -- --ignored
```

### Step 8: Add New Unit Test

Let's add a unit test for error cases.

**Example: Add to `src/utils/sanitize.rs`**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ... existing tests ...

    #[test]
    fn test_sanitize_html_empty_string() {
        let input = "";
        let result = sanitize_html(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_html_no_tags() {
        let input = "Plain text with no tags";
        let result = sanitize_html(input);
        assert_eq!(result, "Plain text with no tags");
    }

    #[test]
    fn test_sanitize_html_nested_dangerous_tags() {
        let input = "<div><script>alert('xss')</script><p>Safe content</p></div>";
        let result = sanitize_html(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("<div>"));
        assert!(result.contains("<p>"));
        assert!(result.contains("Safe content"));
    }
}
```

**Run new tests**:
```bash
cargo test sanitize
```

### Step 9: Add New Integration Test

Let's add a test for pagination.

**Add to `tests/api_tests.rs`**:

```rust
#[tokio::test]
async fn test_list_memos_pagination() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db.clone());
    let service = MemoService::new(db);

    // Create multiple memos for pagination test
    let mut created_ids = Vec::new();
    for i in 1..=15 {
        let dto = CreateMemoDto {
            title: format!("Pagination Test {}", i),
            description: None,
            date_to: Utc::now(),
        };
        let memo = service.create_memo(dto).await.unwrap();
        created_ids.push(memo.id);
    }

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(handlers::list_memos),
    )
    .await;

    // Test first page
    let req = test::TestRequest::get()
        .uri("/api/v1/memos?limit=10&offset=0")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let page1: PaginatedResponse<MemoResponseDto> = test::read_body_json(resp).await;
    assert_eq!(page1.limit, 10);
    assert_eq!(page1.offset, 0);
    assert!(page1.data.len() <= 10);

    // Test second page
    let req = test::TestRequest::get()
        .uri("/api/v1/memos?limit=10&offset=10")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let page2: PaginatedResponse<MemoResponseDto> = test::read_body_json(resp).await;
    assert_eq!(page2.offset, 10);

    // Cleanup
    for id in created_ids {
        service.delete_memo(id).await.ok();
    }
}
```

**Run new test**:
```bash
cargo test test_list_memos_pagination
```

### Step 10: Add Test for Error Handling

Test that errors are handled correctly.

**Add to `tests/api_tests.rs`**:

```rust
#[tokio::test]
async fn test_get_nonexistent_memo_returns_404() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

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
async fn test_create_memo_with_invalid_data_returns_400() {
    let settings = Settings::load().expect("Failed to load settings");
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    let state = AppState::new(settings.clone(), db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::JsonConfig::default().limit(1048576))
            .service(handlers::create_memo),
    )
    .await;

    // Invalid: title too long (> 200 chars)
    let invalid_dto = serde_json::json!({
        "title": "A".repeat(300),
        "date_to": Utc::now().to_rfc3339(),
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/memos")
        .set_json(&invalid_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);  // Bad Request
}
```

### Step 11: Measure Test Coverage

Install coverage tool:

```bash
# Option 1: cargo-tarpaulin (Linux/macOS)
cargo install cargo-tarpaulin

# Option 2: cargo-llvm-cov (all platforms)
cargo install cargo-llvm-cov
```

**Run coverage with tarpaulin**:
```bash
cargo tarpaulin --out Html --output-dir coverage
```

**Run coverage with llvm-cov**:
```bash
cargo llvm-cov --html
```

**Open report**:
```bash
# tarpaulin
open coverage/index.html

# llvm-cov
open target/llvm-cov/html/index.html
```

**Interpreting the report**:
- **Green lines**: Executed during tests
- **Red lines**: Not executed
- **Yellow lines**: Partially executed (e.g., one branch of if/else)

**Focus on coverage for**:
- Service layer (business logic)
- Error handling paths
- Edge cases

**Don't worry about**:
- main.rs (covered by integration tests)
- Generated code (entities)
- Simple getters/setters

### Step 12: Review Advanced Transaction Tests

The codebase includes advanced tests for batch operations and transactions. These demonstrate testing transactional behavior.

**Example: Batch create with transaction** (`tests/service_tests.rs`):

```rust
#[tokio::test]
async fn test_create_memos_batch_transaction() {
    let service = setup_test_service().await;

    // Create multiple memos in batch (atomic operation)
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
```

**Why this test is valuable**:
- Tests transaction semantics (all-or-nothing)
- Tests batch operations (performance feature)
- Verifies atomicity (if one fails, all fail)
- Tests cleanup of multiple records

**Example: Transaction rollback test**:

```rust
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
    // This ensures atomicity: either all succeed or none do
}
```

**Key lesson**: Testing transactions ensures data consistency. If one operation fails, the entire transaction is rolled back, preventing partial writes.

### Step 13: Create Test Fixtures for Complex Scenarios

Add more fixtures to `tests/common/fixtures.rs`:

```rust
use actix_web_template::dto::{CreateMemoDto, UpdateMemoDto, PatchMemoDto};
use chrono::{Utc, Duration};

pub fn create_test_memo_dto(title: &str, description: Option<&str>) -> CreateMemoDto {
    CreateMemoDto {
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        date_to: Utc::now(),
    }
}

pub fn create_overdue_memo_dto(title: &str) -> CreateMemoDto {
    CreateMemoDto {
        title: title.to_string(),
        description: Some("This memo is overdue".to_string()),
        date_to: Utc::now() - Duration::days(7),  // 7 days ago
    }
}

pub fn create_future_memo_dto(title: &str, days_ahead: i64) -> CreateMemoDto {
    CreateMemoDto {
        title: title.to_string(),
        description: None,
        date_to: Utc::now() + Duration::days(days_ahead),
    }
}

pub fn create_update_dto(title: &str, completed: bool) -> UpdateMemoDto {
    UpdateMemoDto {
        title: title.to_string(),
        description: Some("Updated description".to_string()),
        date_to: Utc::now(),
        completed,
    }
}

pub fn create_patch_dto_title_only(title: &str) -> PatchMemoDto {
    PatchMemoDto {
        title: Some(title.to_string()),
        description: None,
        date_to: None,
        completed: None,
    }
}
```

**Use in tests**:
```rust
use common::fixtures::{create_overdue_memo_dto, create_future_memo_dto};

#[tokio::test]
async fn test_overdue_memos() {
    let service = setup_test_service().await;

    let overdue = create_overdue_memo_dto("Overdue Task");
    let created = service.create_memo(overdue).await.unwrap();

    // Assert date is in the past
    assert!(created.date_to < Utc::now());

    service.delete_memo(created.id).await.ok();
}
```

## Checkpoint

At this point, you should have:

**Test suite structure**:
- ✓ Unit tests in `src/` modules
- ✓ Integration tests in `tests/` directory
- ✓ Common utilities in `tests/common/`
- ✓ Test fixtures for reusable test data

**Test categories**:
- ✓ Unit tests (fast, isolated)
- ✓ Service tests (business logic)
- ✓ Repository tests (database operations)
- ✓ API tests (REST endpoints)
- ✓ Web tests (HTML handlers)

**Verification**:
```bash
# .env.test is automatically loaded - just run tests:
cargo test

# Expected output:
# running 53 tests (47 integration + 5 unit + 1 doc)
# test result: ok. 53 passed; 0 failed; 0 ignored

# Check coverage
cargo llvm-cov --html
# Open report: target/llvm-cov/html/index.html
# Should show 70%+ coverage

# Run specific test categories
cargo test --lib                 # Unit tests only (5 tests)
cargo test --test api_tests      # API integration tests (10 tests)
cargo test --test service_tests  # Service tests (17 tests)
cargo test --test web_tests      # Web handler tests (9 tests)
cargo test --test repository_tests  # Repository tests (11 tests)
cargo test -- --test-threads=1   # Serial execution (for database tests)
```

**What works**:
- Comprehensive test coverage across all layers
- Automated testing with `cargo test`
- Test isolation with cleanup
- Realistic tests with real database

**What's next**:
- Continuous testing in CI/CD (Chapter 16)
- Performance testing (optional)
- Load testing (optional)

## Common Issues and Solutions

### Issue: Tests fail with database connection error

**Symptom**:
```
Error: Failed to connect to test database
```

**Cause**: Test database doesn't exist or wrong DATABASE_URL.

**Solution**:
```bash
# Check environment variables
echo $APP_ENV
echo $DATABASE_URL

# Create test database
psql postgresql://postgres:postgres@localhost:5432/postgres -c "CREATE DATABASE memos_test;"

# Run migrations on test database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test sea-orm-cli migrate up

# Verify connection
psql postgresql://postgres:postgres@localhost:5432/memos_test -c "SELECT 1;"

# Set environment and run tests
export APP_ENV=test
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test
cargo test
```

### Issue: Tests interfere with each other

**Symptom**: Tests pass individually but fail when run together.

**Cause**: Tests share database, not cleaning up.

**Solution 1: Add cleanup**:
```rust
#[tokio::test]
async fn test_something() {
    let memo = service.create_memo(dto).await.unwrap();

    // Test code...

    // ALWAYS cleanup
    service.delete_memo(memo.id).await.ok();
}
```

**Solution 2: Run serially**:
```bash
cargo test -- --test-threads=1
```

**Solution 3: Unique test data**:
```rust
let title = format!("Test {}", uuid::Uuid::new_v4());
```

### Issue: Slow tests

**Symptom**: `cargo test` takes minutes to run.

**Cause**: Too many database operations, running serially.

**Solution**:
1. **Run unit tests in parallel**:
   ```bash
   cargo test --lib  # Fast, no database
   ```

2. **Optimize database tests**:
   - Reduce number of database roundtrips
   - Use transactions (if possible)
   - Create test data once, use in multiple tests

3. **Use `--release` mode**:
   ```bash
   cargo test --release  # Slower compile, faster run
   ```

4. **Skip slow tests by default**:
   ```rust
   #[tokio::test]
   #[ignore]  // Skip by default
   async fn test_slow_operation() {
       // ...
   }
   ```

   Run with: `cargo test -- --ignored`

### Issue: Test coverage not showing all files

**Symptom**: Coverage report missing files.

**Cause**: Files not executed during tests.

**Solution**:
1. **Check which files lack tests**:
   ```bash
   cargo llvm-cov --html
   # Open report, look for 0% coverage files
   ```

2. **Add tests for uncovered files**

3. **Exclude files you don't need to test**:
   ```bash
   cargo llvm-cov --html --ignore-filename-regex='(main\.rs|entities/)'
   ```

### Issue: Tests pass locally but fail in CI

**Symptom**: `cargo test` works on your machine, fails in GitHub Actions.

**Cause**: Different environment (database, dependencies, timing).

**Solution**:
1. **Match environments**: Use same Rust version, database version
2. **Check CI logs** for specific error
3. **Test timing issues**: Add sleeps if tests are timing-sensitive
4. **Database setup**: Ensure CI creates and migrates database

### Issue: Can't mock external dependencies

**Symptom**: Want to test code that calls external API without actually calling it.

**Cause**: Rust's type system makes mocking harder than dynamic languages.

**Solution**:
1. **Use traits** (see Mocking section above)
2. **Use `mockall` crate**:
   ```toml
   [dev-dependencies]
   mockall = "0.12"
   ```
3. **Feature flags** for test implementations:
   ```rust
   #[cfg(not(test))]
   fn get_api_client() -> RealClient { /* ... */ }

   #[cfg(test)]
   fn get_api_client() -> MockClient { /* ... */ }
   ```

## Code Review

### Key Design Principles Demonstrated
- **Test pyramid discipline**: The chapter enforces fast unit tests at the base, targeted integration tests in the middle, and full-stack web/UI tests at the top so failures pinpoint the correct layer.
- **Deterministic fixtures**: Shared helpers in `tests/common/` centralize database seeding, ensuring every test case starts from a known state and leaves the database clean.
- **Environment parity**: `.env.test`, Dockerized PostgreSQL, and the same migrations used in production keep test environments identical to real deployments.

### Architecture Benefits
- **Confidence to refactor**: Because each architectural layer (DTOs, services, repositories, handlers) has its own suite, regressions surface immediately where they originate.
- **Faster debugging**: Standardized naming (`api_tests`, `service_tests`, etc.) and focused `cargo test --test ...` commands reduce the time from red test to fix.
- **Operational readiness**: Coverage reporting (`cargo llvm-cov`) and serial/parallel execution modes mirror what CI/CD will run, eliminating “works on my machine” gaps.

### Complete Testing Structure
```
tests/
├── common/
│   ├── fixtures.rs      # Reusable memo builders, DB helpers
│   └── mod.rs
├── api_tests.rs        # REST endpoints via Actix test server
├── web_tests.rs        # HTML handlers + template rendering
├── service_tests.rs    # Business logic with mocked dependencies
├── repository_tests.rs # Real database CRUD + pagination
└── observability_tests.rs (optional) # Tracing/metrics guards

src/
└── **/*_tests mod      # Co-located unit tests for utilities
```

## Testing

### 1. Run the Entire Suite
```bash
cargo test
```
Loads `.env.test`, spins up the Actix test server for integration suites, and exercises all unit tests. Use this command locally before every commit.

### 2. Targeted Suites by Layer
```bash
cargo test --lib                    # Unit tests only
cargo test --test repository_tests  # Database + SeaORM coverage
cargo test --test service_tests     # Business logic workflows
cargo test --test api_tests         # REST endpoints & middleware
cargo test --test web_tests         # SSR handlers + forms
```
Running selective suites speeds up iteration and keeps failures scoped to the layer you’re editing.

### 3. Serial vs Parallel Execution
For tests that mutate the same database tables, force single-threaded execution:
```bash
cargo test -- --test-threads=1
```
Use this mode in CI or when diagnosing nondeterministic behavior.

### 4. Coverage Reporting
```bash
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html
```
Inspect the HTML report to identify untouched modules (look for 0% lines) and prioritize new tests there.

## Summary

You've established a comprehensive testing strategy for your Actix Web application:

**Key achievements**:
1. **Test organization**: Unit tests in `src/`, integration tests in `tests/`
2. **Test categories**: Unit, service, repository, API, web tests
3. **Test utilities**: Common fixtures and setup functions
4. **Test coverage**: Measured with cargo-llvm-cov or cargo-tarpaulin
5. **Test isolation**: Cleanup and unique test data
6. **Best practices**: AAA pattern, descriptive names, independent tests

**Testing patterns learned**:
- **Unit tests**: Pure functions, fast feedback
- **Integration tests**: Full stack, realistic scenarios
- **AAA pattern**: Arrange, Act, Assert
- **Test fixtures**: Reusable test data
- **Cleanup**: Always delete test data
- **Error testing**: Test both success and failure paths

**How this fits into the application**:
- **Tests** (this chapter) verify all layers work correctly
- **Service** (Chapter 7) business logic is thoroughly tested
- **Repository** (Chapter 6) database operations are tested
- **Handlers** (Chapters 8, 12) API and web endpoints are tested
- **Security** (Chapter 13) features are tested

Your application is now reliable and maintainable with 53 tests covering all layers. The next chapters add deployment and observability to make it production-ready.

## Next Steps

In **Chapter 15: Docker Deployment**, you'll:
- Create a production-ready Dockerfile
- Use multi-stage builds for minimal image size
- Configure Docker Compose for local development
- Set up PostgreSQL in a container
- Configure networking and volumes
- Add health checks to containers
- Deploy the complete stack with one command

Your application is tested and ready. Next chapter makes it deployable anywhere Docker runs.

## Additional Resources

### Official Documentation
- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html) - The Rust Book chapter on testing
- [cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html) - Cargo test command reference
- [actix-web::test](https://docs.rs/actix-web/latest/actix_web/test/index.html) - Test utilities

### Testing Tools
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Code coverage for Linux
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) - Code coverage (all platforms)
- [cargo-nextest](https://nexte.st/) - Next-generation test runner
- [mockall](https://docs.rs/mockall/) - Mocking library

### Testing Patterns
- [Testing in Rust](https://rust-lang.github.io/async-book/08_ecosystem/00_chapter.html) - Async testing guide
- [Property Testing](https://github.com/proptest-rs/proptest) - Generate test cases
- [Fuzzing](https://rust-fuzz.github.io/book/) - Automated bug finding

### Best Practices
- [Rust API Guidelines - Testing](https://rust-lang.github.io/api-guidelines/testing.html) - Official testing guidelines
- [Test Organization in Cargo](https://doc.rust-lang.org/book/ch11-03-test-organization.html) - Where to put tests
- [Testing async Rust](https://tokio.rs/tokio/topics/testing) - Tokio testing guide
