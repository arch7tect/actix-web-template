# Chapter 5: Data Transfer Objects and Validation

## Overview

In this chapter, we'll create Data Transfer Objects (DTOs) with comprehensive validation for our memo API. You'll learn how to separate API contracts from database entities and validate user input declaratively using the validator crate.

By the end of this chapter, you'll have type-safe DTOs that enforce business rules at compile time and runtime.

> **Note on Tutorial Approach**: This chapter demonstrates foundational DTO and validation patterns. The production codebase extends these with additional DTOs for pagination, filtering, and more complex validation scenarios. We'll build the core patterns here that scale to production use.

## Prerequisites

### Completed

- Chapter 0: Prerequisites and Environment Setup
- Chapter 1: Core Application Setup
- Chapter 2: Database Integration with SeaORM
- Chapter 3: Error Handling and Middleware
- Chapter 4: Enhanced Health Checks

### Required Knowledge

- Rust structs and enums
- Serde serialization/deserialization
- Understanding of validation concepts
- HTTP request/response bodies

### Required Software

- Working Actix Web application from Chapter 4
- PostgreSQL running

## Learning Objectives

By completing this chapter, you will:

1. Understand the DTO pattern and its benefits
2. Separate API contracts from database entities
3. Use the validator crate for declarative validation
4. Create multiple DTO variants (Create, Update, Patch, Response)
5. Implement pagination parameters with defaults
6. Build generic paginated response wrappers
7. Handle validation errors properly

## Concepts Covered

### What are DTOs?

**Data Transfer Objects (DTOs)** are structs designed specifically for transferring data between layers of your application. They define the shape of data coming into and going out of your API.

**Why use DTOs?**

1. **Separation of concerns**: API contracts independent of database schema
2. **Validation**: Enforce rules before data reaches business logic
3. **Flexibility**: Database can change without breaking API
4. **Security**: Control exactly what data is exposed
5. **Documentation**: Clear API contracts for consumers

### DTO vs Entity

**Entity** (`src/entities/memos.rs`):
- Represents database table structure
- Generated from database schema
- Contains database-specific fields (timestamps, etc.)
- Used by ORM (SeaORM)

**DTO** (`src/dto/memo_dto.rs`):
- Represents API request/response structure
- Handwritten for API design
- Contains only fields relevant to API consumers
- Used by handlers

Example:
```rust
// Entity - all database fields
pub struct Model {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub date_to: DateTimeWithTimeZone,
    pub completed: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

// DTO - only what user provides
pub struct CreateMemoDto {
    pub title: String,
    pub description: Option<String>,
    pub date_to: DateTime<Utc>,
}
```

### The Validator Crate

The `validator` crate provides declarative validation through derive macros:

```rust
#[derive(Validate)]
pub struct CreateMemoDto {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,
}
```

**Common validators**:
- `length(min, max)` - String/Vec length
- `range(min, max)` - Numeric ranges
- `email` - Email format
- `url` - URL format
- `regex` - Custom patterns
- `custom` - Custom validation functions

### DTO Variants

Different operations need different DTOs:

1. **CreateDto**: New resource creation (POST)
   - No ID (server generates)
   - All required fields
   - No timestamps

2. **UpdateDto**: Full replacement (PUT)
   - Requires ID in path
   - All fields (replace entire resource)

3. **PatchDto**: Partial update (PATCH)
   - Requires ID in path
   - Optional fields (update only provided)

4. **ResponseDto**: Outbound data (GET)
   - Includes ID
   - Includes timestamps
   - May include computed fields

### Pagination Pattern

Pagination prevents loading too much data:

```rust
pub struct PaginationParams {
    pub page: u64,      // Page number (1-indexed)
    pub per_page: u64,  // Items per page (max 100)
}

pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
```

## Step-by-Step Instructions

### Step 1: Add Dependencies

**Why**: We need the validator crate for declarative validation on DTO fields.

**How**:

1. **Update `Cargo.toml`**:

```toml
[package]
name = "actix-memo-app"
version = "0.2.1"
edition = "2024"

[dependencies]
# Web framework
actix-web = "4"

# CORS support
actix-cors = "0.7"

# Async runtime
tokio = { version = "1.47", features = ["full"] }

# Database - SeaORM
sea-orm = { version = "1.1", features = [
    "sqlx-postgres",
    "runtime-tokio-rustls",
    "macros",
] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.20", features = ["derive"] }

# Configuration
dotenvy = "0.15"

# Error handling
thiserror = "1.0"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# UUID support
uuid = { version = "1.18", features = ["v4", "serde"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

**Verify**:
```bash
cargo check
```

Should download new dependencies and compile successfully.

---

### Step 2: Create DTO Module Structure

**Why**: Organize DTOs in a dedicated module for clarity.

**How**:

1. **Create DTO directory**:
   ```bash
   mkdir -p src/dto
   touch src/dto/mod.rs
   touch src/dto/memo_dto.rs
   ```

2. **Create `src/dto/mod.rs`**:

```rust
pub mod memo_dto;

pub use memo_dto::{
    CreateMemoDto,
    UpdateMemoDto,
    PatchMemoDto,
    MemoResponseDto,
    PaginationParams,
    PaginatedResponse,
};
```

3. **Update `src/lib.rs`**:

```rust
pub mod config;
pub mod dto;
pub mod entities;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod state;
pub mod utils;
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 3: Create Request DTOs (Create, Update, Patch)

**Why**: Define what data users can send to create or modify memos.

**How**:

1. **Create `src/dto/memo_dto.rs`**:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// DTO for creating a new memo
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateMemoDto {
    /// Memo title (1-200 characters)
    #[validate(length(min = 1, max = 200, message = "Title must be between 1 and 200 characters"))]
    pub title: String,

    /// Optional memo description (max 1000 characters)
    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,

    /// Due date and time
    pub date_to: DateTime<Utc>,
}

/// DTO for full update of a memo (PUT)
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateMemoDto {
    /// Memo title (1-200 characters)
    #[validate(length(min = 1, max = 200, message = "Title must be between 1 and 200 characters"))]
    pub title: String,

    /// Optional memo description (max 1000 characters)
    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,

    /// Due date and time
    pub date_to: DateTime<Utc>,

    /// Completion status
    pub completed: bool,
}

/// DTO for partial update of a memo (PATCH)
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PatchMemoDto {
    /// Optional memo title (1-200 characters)
    #[validate(length(min = 1, max = 200, message = "Title must be between 1 and 200 characters"))]
    pub title: Option<String>,

    /// Optional memo description (max 1000 characters)
    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,

    /// Optional due date and time
    pub date_to: Option<DateTime<Utc>>,

    /// Optional completion status
    pub completed: Option<bool>,
}
```

**Understanding the annotations**:

- `#[derive(Validate)]` - Enables validation
- `#[validate(...)]` - Validation rules for fields
- `message = "..."` - Custom validation error messages

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 4: Create Response DTO

**Why**: Define what data the API returns to clients.

**How**:

1. **Add to `src/dto/memo_dto.rs`**:

```rust
/// DTO for memo responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoResponseDto {
    /// Unique memo identifier
    pub id: Uuid,

    /// Memo title
    pub title: String,

    /// Memo description
    pub description: Option<String>,

    /// Due date and time
    pub date_to: DateTime<Utc>,

    /// Completion status
    pub completed: bool,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}
```

**Note**: Response DTOs use `Serialize` (not `Deserialize`) because they're only sent, never received.

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 5: Create Pagination DTOs

**Why**: Support paginated list endpoints for better performance.

**How**:

1. **Add to `src/dto/memo_dto.rs`**:

```rust
/// Pagination parameters for list endpoints
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    #[serde(default = "default_page")]
    pub page: u64,

    /// Items per page
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    #[serde(default = "default_per_page")]
    pub per_page: u64,

    /// Optional filter by completion status
    pub completed: Option<bool>,

    /// Optional sort field
    pub sort_by: Option<String>,

    /// Optional sort direction
    pub sort_order: Option<String>,
}

/// Default page number
fn default_page() -> u64 {
    1
}

/// Default items per page
fn default_per_page() -> u64 {
    10
}

/// Generic paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// List of items
    pub data: Vec<T>,

    /// Total number of items (across all pages)
    pub total: u64,

    /// Current page number
    pub page: u64,

    /// Items per page
    pub per_page: u64,

    /// Total number of pages
    pub total_pages: u64,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, total: u64, page: u64, per_page: u64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as u64;

        Self {
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}
```

**Understanding serde defaults**:

- `#[serde(default = "function_name")]` - Uses function return value if field is missing from JSON
- Allows optional query parameters with sensible defaults
- Example: `GET /api/v1/memos` uses page=1, per_page=10

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 6: Add Conversion Functions (Entity ↔ DTO)

**Why**: Convert between database entities and DTOs easily.

**How**:

1. **Add to `src/dto/memo_dto.rs`**:

```rust
use crate::entities::memos;

impl From<memos::Model> for MemoResponseDto {
    /// Convert a database entity to a response DTO
    fn from(model: memos::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            description: model.description,
            date_to: model.date_to.naive_utc(),
            completed: model.completed,
            created_at: model.created_at.naive_utc(),
            updated_at: model.updated_at.naive_utc(),
        }
    }
}

impl MemoResponseDto {
    /// Convert a list of entities to response DTOs
    pub fn from_models(models: Vec<memos::Model>) -> Vec<Self> {
        models.into_iter().map(Self::from).collect()
    }
}
```

**Understanding the conversions**:

- `From` trait enables `.into()` conversions
- `naive_utc()` converts timezone-aware timestamps to naive (no timezone) for API
- Helper method `from_models()` for batch conversions

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 7: Test Validation

**Why**: Verify that validation rules work correctly.

**How**:

1. **Create `src/dto/memo_dto.rs` test module**:

Add at the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_memo_validation_valid() {
        let dto = CreateMemoDto {
            title: "Valid title".to_string(),
            description: Some("Valid description".to_string()),
            date_to: Utc::now()
                .unwrap(),
        };

        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_create_memo_validation_empty_title() {
        let dto = CreateMemoDto {
            title: "".to_string(),
            description: None,
            date_to: Utc::now()
                .unwrap(),
        };

        let result = dto.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("title"));
    }

    #[test]
    fn test_create_memo_validation_title_too_long() {
        let dto = CreateMemoDto {
            title: "a".repeat(201),
            description: None,
            date_to: Utc::now()
                .unwrap(),
        };

        let result = dto.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("title"));
    }

    #[test]
    fn test_create_memo_validation_description_too_long() {
        let dto = CreateMemoDto {
            title: "Valid title".to_string(),
            description: Some("a".repeat(1001)),
            date_to: Utc::now()
                .unwrap(),
        };

        let result = dto.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn test_pagination_params_validation() {
        let params = PaginationParams {
            page: 1,
            per_page: 10,
            completed: None,
            sort_by: None,
            sort_order: None,
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_pagination_params_validation_invalid_page() {
        let params = PaginationParams {
            page: 0,
            per_page: 10,
            completed: None,
            sort_by: None,
            sort_order: None,
        };

        let result = params.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pagination_params_validation_per_page_too_large() {
        let params = PaginationParams {
            page: 1,
            per_page: 101,
            completed: None,
            sort_by: None,
            sort_order: None,
        };

        let result = params.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_paginated_response_calculation() {
        let data = vec![1, 2, 3, 4, 5];
        let response = PaginatedResponse::new(data, 42, 1, 10);

        assert_eq!(response.total, 42);
        assert_eq!(response.page, 1);
        assert_eq!(response.per_page, 10);
        assert_eq!(response.total_pages, 5); // ceil(42 / 10) = 5
    }
}
```

2. **Run tests**:
   ```bash
   cargo test dto
   ```

**Verify**:
All tests should pass.

---

## Checkpoint

Run these commands to verify everything works:

```bash
# Build should succeed
cargo build

# All DTO tests should pass
cargo test dto

# Check specific validations
cargo test test_create_memo_validation
```

### Expected Results

- All dependencies installed correctly
- DTOs compile without errors
- Validation rules enforce constraints
- Tests pass for valid and invalid data
- Conversions between Entity and DTO work

---

## Common Issues and Solutions

### Issue: Validation not working

**Symptoms**: Invalid data passes validation

**Cause**: Forgot to call `.validate()`

**Solution**:
```rust
use validator::Validate;

// In your handler
let dto: CreateMemoDto = /* deserialize from request */;
dto.validate()?; // Don't forget this!
```

---

### Issue: Compilation error with chrono types

**Symptoms**: Type mismatch between `DateTime<Utc>`

**Cause**: Entity uses timezone-aware, DTO uses naive

**Solution**:
```rust
// Convert timezone-aware to naive
let naive = model.created_at.naive_utc();

// Convert naive to timezone-aware
use chrono::Utc;
let aware = naive.and_utc();
```

---

### Issue: Serde deserialization fails for optional fields

**Symptoms**: Missing optional fields cause errors

**Cause**: Not using `#[serde(default)]`

**Solution**:
```rust
#[derive(Deserialize)]
pub struct PatchMemoDto {
    #[serde(default)]  // Use Default::default() if missing
    pub title: Option<String>,
}
```

---

## Code Review

### Key Design Principles Demonstrated
- **Contract-first design**: Request/response DTOs define the public API separately from persistence models.
- **Declarative validation**: Attribute-based constraints (`#[validate]`) capture business rules where the data is defined.
- **Type-driven conversions**: `From`/`TryFrom` implementations centralize mapping between entities, DTOs, and service responses.
- **Reusable pagination**: A generic `PaginatedResponse<T>` standardizes how list endpoints shape their payloads.

### Architecture Benefits
- **Security**: Validation guards the boundary, ensuring only sanitized data reaches deeper layers.
- **Clarity for clients**: Type-safe DTOs with doc comments clearly communicate API contracts and constraints.
- **Testability**: DTOs can be unit-tested without touching the database, providing fast feedback on rules.
- **Consistency**: Shared pagination metadata keeps every list endpoint predictable for front-end consumers.

### Complete DTO & Validation Structure
```rust
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateMemoDto {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub date_to: DateTime<Utc>,
}

impl From<memos::Model> for MemoResponseDto {
    fn from(model: memos::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            // ... existing field mapping ...
        }
    }
}
```

```rust
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    // ... derived helpers like total_pages() ...
}
```

## Understanding Validation Flow

```
1. Client sends JSON request
   ↓
2. Actix Web deserializes to DTO (serde)
   ↓
3. Handler calls dto.validate()
   ↓
4. Validator checks all rules
   ↓
5a. Valid: Continue to service layer
5b. Invalid: Return 400 Bad Request with errors
```

Example validation error response:
```json
{
  "error": {
    "message": "Validation failed: title: Title must be between 1 and 200 characters",
    "status": 400
  }
}
```

---

## Summary

Congratulations! You've built a complete DTO system with validation. You now have:

1. **Separate API contracts** - DTOs independent of database entities
2. **Request DTOs** - Create, Update, and Patch variants
3. **Response DTOs** - Controlled outbound data structure
4. **Declarative validation** - Type-safe input validation
5. **Pagination support** - Query parameters with defaults
6. **Generic responses** - Reusable paginated wrapper
7. **Entity conversions** - Easy database to API transformations
8. **Comprehensive tests** - Validation rule verification

### Key Takeaways

- **DTOs separate concerns** - API design independent of database
- **Validation prevents bad data** - Catch errors early
- **Declarative is better** - Rules in struct definitions
- **Type safety** - Compiler enforces API contracts
- **Generic code** - DRY principle with `PaginatedResponse<T>`

### Architecture So Far

```
┌─────────────────────────────────────┐
│        HTTP Requests                │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Middleware Stack                   │
│  - Security, CORS, Compression      │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Handlers                           │
│  - Deserialize JSON to DTO          │
│  - Call dto.validate()              │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  DTOs (This Chapter!)               │
│  - CreateMemoDto                    │
│  - UpdateMemoDto                    │
│  - PatchMemoDto                     │
│  - MemoResponseDto                  │
│  - Validation rules                 │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Service Layer (Next Chapter)       │
│  - Business logic                   │
│  - DTO → Entity conversion          │
└─────────────────────────────────────┘
```

---

## Next Steps

### Required: Chapter 6 - Repository Layer - Database Operations

You'll encapsulate database access behind repositories, translate DTOs into SeaORM queries, and orchestrate transactions that service methods can rely on. Expect tight integration between the validation rules you created and the persistence layer.

### Optional Exercises

1. **Challenge**: Implement a custom validator that ensures memo titles remain under a configurable length read from the environment.
2. **Challenge**: Add localized validation error messages by leveraging `validator::ValidationErrors::to_tree`.
3. **Challenge**: Design a DTO for memo search (filters + sorting) even though the repository implementation comes next.

---

## Additional Resources

### Validation
- [validator crate](https://docs.rs/validator/) - Official documentation
- [Custom validators](https://github.com/Keats/validator#custom-validation) - How to write custom validation functions
- [Validation in Actix](https://actix.rs/docs/extractors/#json) - Integration guide

### DTOs and Patterns
- [DTO Pattern](https://martinfowler.com/eaaCatalog/dataTransferObject.html) - Martin Fowler
- [API Design Patterns](https://www.manning.com/books/api-design-patterns) - Book

### Serde
- [Serde attributes](https://serde.rs/attributes.html) - All serde annotations
- [Serde derive](https://serde.rs/derive.html) - Derive macro guide

---

**Ready to build the repository layer? Let's move on to [Chapter 6: Repository Layer](chapter-06.md)!**
