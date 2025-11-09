# Chapter 8: REST API Handlers

## Overview

REST API handlers are the HTTP entry point to your application. They receive HTTP requests, extract and validate input, call service layer methods, and return JSON responses. In this chapter, you'll build a complete RESTful API for memo management with proper HTTP semantics, error handling, and observability.

You'll create seven endpoints covering all CRUD operations: list, get, create, update, patch, delete, and a custom toggle operation. Each handler follows REST principles, uses appropriate HTTP methods and status codes, and integrates with the service layer you built in Chapter 7.

## Prerequisites

### Completed
- Chapter 7: Service Layer - Business Logic and Transactions

### Required Knowledge
- HTTP methods (GET, POST, PUT, PATCH, DELETE)
- HTTP status codes (200, 201, 204, 400, 404, 500)
- REST API design principles
- JSON serialization/deserialization

### Required Software
- Rust 1.75+
- PostgreSQL running
- All dependencies from previous chapters

## Learning Objectives

By the end of this chapter, you will:
- Understand RESTful API design principles and HTTP semantics
- Implement Actix Web handler functions with extractors
- Handle request data (JSON body, path parameters, query strings)
- Return appropriate HTTP status codes and JSON responses
- Register routes with Actix Web's App
- Test REST API endpoints end-to-end

## Concepts Covered

### RESTful API Design

REST (Representational State Transfer) is an architectural style for web services using HTTP methods to perform operations on resources.

**Key Principles:**

1. **Resource-Based**: URLs represent resources (`/api/v1/memos/{id}`)
2. **HTTP Methods**: Use standard methods for operations
   - GET: Retrieve resources
   - POST: Create new resources
   - PUT: Full update of resources
   - PATCH: Partial update of resources
   - DELETE: Remove resources
3. **Stateless**: Each request contains all information needed
4. **Standard Status Codes**: Use HTTP status codes correctly
5. **JSON Format**: Consistent response format

**Our API Design:**

```
GET    /api/v1/memos           → List all memos (paginated)
GET    /api/v1/memos/{id}      → Get single memo
POST   /api/v1/memos           → Create new memo
PUT    /api/v1/memos/{id}      → Full update
PATCH  /api/v1/memos/{id}      → Partial update
DELETE /api/v1/memos/{id}      → Delete memo
PATCH  /api/v1/memos/{id}/complete → Toggle completion
```

###

 HTTP Status Codes

Correct status codes communicate success or failure clearly:

**Success Codes:**
- `200 OK`: Successful GET, PUT, PATCH
- `201 Created`: Successful POST (resource created)
- `204 No Content`: Successful DELETE (no response body)

**Client Error Codes:**
- `400 Bad Request`: Invalid input/validation error
- `404 Not Found`: Resource doesn't exist

**Server Error Codes:**
- `500 Internal Server Error`: Unexpected server error

### Actix Web Handlers

Handlers are async functions that process HTTP requests and return responses.

**Handler Signature:**
```rust
pub async fn handler_name(
    extractors...
) -> impl Responder {
    // Process request
    // Return response
}
```

**Actix Web Extractors:**

1. **web::Data<T>**: Access shared application state
2. **web::Path<T>**: Extract path parameters
3. **web::Query<T>**: Extract query string parameters
4. **web::Json<T>**: Extract and validate JSON request body

### Request Flow

```
HTTP Request
     ↓
Actix Web Server
     ↓
Route Matching
     ↓
Handler Function
     ├→ Extract Data (web::Json, web::Path, etc.)
     ├→ Call Service Layer
     └→ Build Response
     ↓
Middleware (logging, compression, etc.)
     ↓
HTTP Response (JSON)
```

### Error Handling in Handlers

Handlers convert service errors into HTTP responses:

```rust
match service.create_memo(dto).await {
    Ok(memo) => HttpResponse::Created().json(memo),
    Err(e) => e.error_response(), // AppError implements ResponseError
}
```

Our `AppError` type automatically converts to appropriate HTTP status codes.

## Step-by-Step Instructions

### Step 1: Create the Memo Handlers Module

**Why**: Organize REST API handlers separately from web UI handlers.

**How**: Create `src/handlers/memos.rs`:

```rust
use actix_web::{
    HttpResponse, Responder, delete, error::ResponseError, get, patch, post, put, web,
};
use uuid::Uuid;

use crate::{
    dto::{
        CreateMemoDto, MemoResponseDto, PaginatedMemoResponse, PaginationParams, PatchMemoDto,
        UpdateMemoDto,
    },
    error::ErrorResponse,
    services::MemoService,
    state::AppState,
};
```

This imports all necessary types for our handlers.

**Verify**:
```bash
cargo check
```

---

### Step 2: Implement List Memos Handler

**Why**: Allow clients to retrieve all memos with pagination and filtering.

**How**: Add to `src/handlers/memos.rs`:

```rust
/// List all memos with pagination and filtering
#[tracing::instrument(skip(state, params))]
#[get("/api/v1/memos")]
pub async fn list_memos(
    state: web::Data<AppState>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    tracing::debug!("Listing memos with pagination");

    let service = MemoService::new(state.db.clone());
    match service.get_all_memos(params.into_inner()).await {
        Ok(response) => {
            tracing::info!(
                count = response.data.len(),
                total = response.total,
                "Memos listed successfully"
            );
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to list memos");
            e.error_response()
        }
    }
}
```

**Key Points:**
- `#[get("/api/v1/memos")]`: Route macro
- `web::Query<PaginationParams>`: Extracts query string
- `HttpResponse::Ok().json()`: Returns 200 with JSON
- `#[tracing::instrument]`: Adds observability

**Verify**:
```bash
cargo check
```

---

### Step 3: Implement Get Single Memo Handler

**Why**: Retrieve a specific memo by its ID.

**How**: Add to `src/handlers/memos.rs`:

```rust
/// Get a memo by ID
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[get("/api/v1/memos/{id}")]
pub async fn get_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    tracing::debug!("Getting memo by ID");

    let service = MemoService::new(state.db.clone());
    match service.get_memo_by_id(id.into_inner()).await {
        Ok(memo) => {
            tracing::info!("Memo retrieved successfully");
            HttpResponse::Ok().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get memo");
            e.error_response()
        }
    }
}
```

**Key Points:**
- `web::Path<Uuid>`: Extracts `{id}` from URL
- Returns 404 automatically if not found (via `AppError`)

---

### Step 4: Implement Create Memo Handler

**Why**: Allow clients to create new memos.

**How**: Add to `src/handlers/memos.rs`:

```rust
/// Create a new memo
#[tracing::instrument(skip(state, dto), fields(title = %dto.title))]
#[post("/api/v1/memos")]
pub async fn create_memo(
    state: web::Data<AppState>,
    dto: web::Json<CreateMemoDto>,
) -> impl Responder {
    tracing::debug!("Creating new memo");

    let service = MemoService::new(state.db.clone());
    match service.create_memo(dto.into_inner()).await {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, "Memo created successfully");
            HttpResponse::Created().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create memo");
            e.error_response()
        }
    }
}
```

**Key Points:**
- `web::Json<CreateMemoDto>`: Extracts and validates JSON body
- `HttpResponse::Created()`: Returns 201 status
- Validation happens automatically (validator crate)

---

### Step 5: Implement Update Memo Handler (PUT)

**Why**: Allow full replacement of memo data.

**How**: Add to `src/handlers/memos.rs`:

```rust
/// Update a memo (full replacement)
#[tracing::instrument(skip(state, dto), fields(memo_id = %id))]
#[put("/api/v1/memos/{id}")]
pub async fn update_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateMemoDto>,
) -> impl Responder {
    tracing::debug!("Updating memo");

    let service = MemoService::new(state.db.clone());
    match service.update_memo(id.into_inner(), dto.into_inner()).await {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, "Memo updated successfully");
            HttpResponse::Ok().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to update memo");
            e.error_response()
        }
    }
}
```

**Key Points:**
- PUT requires all fields (UpdateMemoDto)
- Returns 200 with updated resource

---

### Step 6: Implement Patch Memo Handler (PATCH)

**Why**: Allow partial updates (only provided fields change).

**How**: Add to `src/handlers/memos.rs`:

```rust
/// Partially update a memo
#[tracing::instrument(skip(state, dto), fields(memo_id = %id))]
#[patch("/api/v1/memos/{id}")]
pub async fn patch_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    dto: web::Json<PatchMemoDto>,
) -> impl Responder {
    tracing::debug!("Patching memo");

    let service = MemoService::new(state.db.clone());
    match service.patch_memo(id.into_inner(), dto.into_inner()).await {
        Ok(memo) => {
            tracing::info!(memo_id = %memo.id, "Memo patched successfully");
            HttpResponse::Ok().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to patch memo");
            e.error_response()
        }
    }
}
```

**Key Points:**
- PATCH uses PatchMemoDto (all fields Optional)
- Only provided fields are updated
- Unspecified fields remain unchanged

---

### Step 7: Implement Delete Memo Handler

**Why**: Allow permanent removal of memos.

**How**: Add to `src/handlers/memos.rs`:

```rust
/// Delete a memo
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[delete("/api/v1/memos/{id}")]
pub async fn delete_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    tracing::debug!("Deleting memo");

    let service = MemoService::new(state.db.clone());
    match service.delete_memo(id.into_inner()).await {
        Ok(()) => {
            tracing::info!("Memo deleted successfully");
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete memo");
            e.error_response()
        }
    }
}
```

**Key Points:**
- DELETE returns 204 No Content (no response body)
- `.finish()` sends response without body

---

### Step 8: Implement Toggle Complete Handler

**Why**: Convenient endpoint to toggle completion status.

**How**: Add to `src/handlers/memos.rs`:

```rust
/// Toggle memo completion status
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[patch("/api/v1/memos/{id}/complete")]
pub async fn toggle_complete(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    tracing::debug!("Toggling memo completion status");

    let service = MemoService::new(state.db.clone());
    match service.toggle_complete(id.into_inner()).await {
        Ok(memo) => {
            tracing::info!(
                memo_id = %memo.id,
                completed = memo.completed,
                "Completion toggled"
            );
            HttpResponse::Ok().json(memo)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to toggle completion");
            e.error_response()
        }
    }
}
```

---

### Step 9: Export Handlers from Module

**Why**: Make handlers available to main.rs.

**How**: Update `src/handlers/mod.rs`:

```rust
pub mod memos;

// ... existing exports ...

pub use memos::{
    create_memo, delete_memo, get_memo, list_memos,
    patch_memo, toggle_complete, update_memo,
};
```

**Verify**:
```bash
cargo check
```

---

### Step 10: Register Routes in Main

**Why**: Wire up handlers to the Actix Web application.

**How**: Update `src/main.rs`:

```rust
// In the App configuration
.service(handlers::list_memos)
.service(handlers::get_memo)
.service(handlers::create_memo)
.service(handlers::update_memo)
.service(handlers::patch_memo)
.service(handlers::delete_memo)
.service(handlers::toggle_complete)
```

**Verify**:
```bash
cargo run
```

You should see the server start successfully.

---

## Checkpoint

At this point, you should have:

1. Complete REST API handlers in `src/handlers/memos.rs`
2. All 7 endpoints implemented:
   - GET `/api/v1/memos` (list)
   - GET `/api/v1/memos/{id}` (get)
   - POST `/api/v1/memos` (create)
   - PUT `/api/v1/memos/{id}` (update)
   - PATCH `/api/v1/memos/{id}` (patch)
   - DELETE `/api/v1/memos/{id}` (delete)
   - PATCH `/api/v1/memos/{id}/complete` (toggle)
3. Handlers exported and routes registered

**Verify everything works**:

```bash
# Start the server
cargo run

# In another terminal, test the API:

# Create a memo
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Memo","description":"Testing API","date_to":"2025-12-31T23:59:59Z"}'

# List memos
curl http://localhost:3737/api/v1/memos

# Get single memo (use ID from create response)
curl http://localhost:3737/api/v1/memos/{id}

# Update memo
curl -X PUT http://localhost:3737/api/v1/memos/{id} \
  -H "Content-Type: application/json" \
  -d '{"title":"Updated","description":"Modified","date_to":"2025-12-31T23:59:59Z","completed":false}'

# Patch memo
curl -X PATCH http://localhost:3737/api/v1/memos/{id} \
  -H "Content-Type: application/json" \
  -d '{"title":"Patched Title"}'

# Toggle complete
curl -X PATCH http://localhost:3737/api/v1/memos/{id}/complete

# Delete memo
curl -X DELETE http://localhost:3737/api/v1/memos/{id}
```

Expected: All requests return appropriate JSON responses with correct status codes.

---

## Common Issues and Solutions

### Issue: "Handler not found" or 404 errors

**Symptoms**: Requests return 404 even though handler exists

**Cause**: Handler not registered in main.rs or route path mismatch

**Solution**:
```rust
// Verify handler is registered in main.rs
.service(handlers::list_memos) // Must be present

// Check route macro matches
#[get("/api/v1/memos")]  // Path must be exact
```

---

### Issue: "Failed to deserialize JSON" error

**Symptoms**: 400 error when sending JSON

**Cause**: JSON doesn't match DTO structure or validation failed

**Solution**:
```bash
# Ensure JSON matches CreateMemoDto structure
{
  "title": "string",           # Required, 1-200 chars
  "description": "string",      # Optional, max 1000 chars
  "date_to": "2025-12-31T23:59:59Z"  # Required, DateTime<Utc>
}

# Check validation rules in DTOs
```

---

### Issue: Compilation error "trait Responder not implemented"

**Symptoms**: `impl Responder` doesn't compile

**Cause**: Return type doesn't implement Responder

**Solution**:
```rust
// Correct: Return HttpResponse
HttpResponse::Ok().json(data)

// Correct: Return Result that converts to HttpResponse
match result {
    Ok(data) => HttpResponse::Ok().json(data),
    Err(e) => e.error_response(),
}
```

---

### Issue: UUID parsing errors in path parameters

**Symptoms**: 404 or parsing error with valid UUID

**Cause**: UUID format mismatch or wrong path parameter name

**Solution**:
```rust
// Route definition
#[get("/api/v1/memos/{id}")]  // {id} matches parameter name

// Handler signature
pub async fn get_memo(id: web::Path<Uuid>) -> impl Responder {
                    //  ↑ Must match route parameter
}
```

---

## Code Review

### Key Design Principles Demonstrated

1. **Thin Handlers**: Handlers only handle HTTP concerns, business logic in services
2. **Extractor Pattern**: Type-safe extraction of request data
3. **Error Conversion**: AppError automatically converts to HTTP responses
4. **Observability**: Every handler has tracing instrumentation
5. **REST Semantics**: Correct HTTP methods and status codes
6. **Separation of Concerns**: Clear boundaries between layers

### Architecture Benefits

```
┌─────────────────────────────────────────┐
│ REST API Handlers (THIS CHAPTER) ✓      │
│ - HTTP request/response handling        │
│ - Input extraction & validation         │
│ - Status code mapping                   │
│ - JSON serialization                    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│ Service Layer (Chapter 7) ✓             │
│ - Business logic                        │
│ - DTO conversions                       │
│ - Transaction coordination              │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│ Repository Layer (Chapter 6) ✓          │
│ - Database operations                   │
└─────────────────────────────────────────┘
```

**Benefits:**
- Handlers can be tested independently
- Easy to add new endpoints
- Service layer reusable (web UI uses same services)
- Clear error handling flow
- Type-safe request/response handling

### Complete Handler Structure

```
src/
├── handlers/
│   ├── mod.rs          # Exports all handlers
│   ├── memos.rs        # REST API handlers (THIS CHAPTER)
│   ├── web.rs          # Web UI handlers
│   └── health.rs       # Health check handlers
```

---

## Testing

### API Integration Tests

The REST API can be tested end-to-end with integration tests.

**Test Coverage:**
- ✅ Create memo with valid data
- ✅ Create memo with invalid data (validation)
- ✅ List memos with pagination
- ✅ List memos with filtering
- ✅ Get memo by ID
- ✅ Get non-existent memo (404)
- ✅ Update memo (PUT)
- ✅ Patch memo (PATCH)
- ✅ Delete memo
- ✅ Toggle completion status

**Run API tests:**
```bash
cargo test --test api_tests
```

**Manual Testing with curl:**
```bash
# Full CRUD flow
# 1. Create
ID=$(curl -s -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","description":"Desc","date_to":"2025-12-31T23:59:59Z"}' \
  | jq -r '.id')

# 2. Get
curl http://localhost:3737/api/v1/memos/$ID | jq

# 3. Update
curl -X PUT http://localhost:3737/api/v1/memos/$ID \
  -H "Content-Type: application/json" \
  -d '{"title":"Updated","description":"Modified","date_to":"2025-12-31T23:59:59Z","completed":true}' | jq

# 4. Delete
curl -X DELETE http://localhost:3737/api/v1/memos/$ID -v
```

---

## Summary

### What You Learned

In this chapter, you:

1. **Built a complete RESTful API** with 7 endpoints covering all CRUD operations
2. **Applied REST principles** using correct HTTP methods and status codes
3. **Used Actix Web extractors** for type-safe request data extraction
4. **Integrated with service layer** keeping handlers thin and focused
5. **Implemented observability** with tracing instrumentation
6. **Registered routes** with Actix Web's application builder

### Architecture Progress

```
✅ HTTP Layer (REST API Handlers) - THIS CHAPTER
✅ Business Logic Layer (Services) - Chapter 7
✅ Data Access Layer (Repository) - Chapter 6
✅ Entity Layer (SeaORM Models) - Chapter 2
✅ Error Handling - Chapter 3
✅ DTOs & Validation - Chapter 5
```

You now have a fully functional REST API!

### Key Takeaways

1. **Handlers are thin**: They delegate business logic to services
2. **Extractors provide type safety**: `web::Json`, `web::Path`, `web::Query`
3. **HTTP status codes matter**: Use correct codes for semantic clarity
4. **Error handling is automatic**: `AppError` converts to HTTP responses
5. **REST is about resources**: URLs represent resources, methods represent actions

---

## Next Steps

### Required: Chapter 9 - OpenAPI Documentation

Now that you have working REST API handlers, you'll add comprehensive API documentation using OpenAPI 3.0 specification and Swagger UI, making your API discoverable and easy to use.

### Optional Exercises

1. **Add filtering by date range**: Support `date_from` and `date_to` query parameters
2. **Implement bulk operations**: Add endpoint for batch create/delete
3. **Add response caching**: Use `Cache-Control` headers for GET requests

---

## Additional Resources

### REST API Design
- [REST API Tutorial](https://restfulapi.net/)
- [HTTP Status Codes](https://httpstatuses.com/)
- [Richardson Maturity Model](https://martinfowler.com/articles/richardsonMaturityModel.html)

### Actix Web
- [Actix Web Extractors](https://actix.rs/docs/extractors)
- [Actix Web Responses](https://actix.rs/docs/response)
- [Actix Web Testing](https://actix.rs/docs/testing)

### Testing APIs
- [curl documentation](https://curl.se/docs/manual.html)
- [Postman](https://www.postman.com/)
- [HTTPie](https://httpie.io/)
