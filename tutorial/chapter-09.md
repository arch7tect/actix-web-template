# Chapter 9: OpenAPI Documentation

## Overview

In this chapter, you'll add professional API documentation to your memo management application using OpenAPI 3.0 specification and Swagger UI. You'll learn how **utoipa** generates comprehensive API documentation directly from your Rust code, keeping your documentation and implementation perfectly synchronized.

OpenAPI documentation is essential for:
- **API consumers**: Understanding available endpoints, request/response formats, and error codes
- **Development teams**: Maintaining consistent API contracts across services
- **Testing**: Interactive API exploration and testing via Swagger UI
- **Integration**: Auto-generating client SDKs in multiple languages

By the end of this chapter, you'll have interactive API documentation at `/swagger-ui/` that stays automatically in sync with your code.

---

## Prerequisites

### Completed Chapters
- **Chapter 0**: Prerequisites and Environment Setup
- **Chapter 1**: Core Application Setup
- **Chapter 2**: Database Integration with SeaORM
- **Chapter 3**: Error Handling and Middleware
- **Chapter 4**: Health Checks and Monitoring
- **Chapter 5**: DTOs and Validation
- **Chapter 6**: Repository Layer
- **Chapter 7**: Service Layer
- **Chapter 8**: REST API Handlers

### Required Knowledge
- REST API design principles
- JSON schema concepts
- HTTP request/response structure
- Basic understanding of API documentation

### Required Software
- Working Actix Web application from Chapter 8
- All dependencies from previous chapters

---

## Learning Objectives

By the end of this chapter, you will be able to:

1. **Generate OpenAPI 3.0 specifications** from Rust code using utoipa
2. **Document DTOs** with schema definitions and examples
3. **Annotate handler functions** with request/response specifications
4. **Configure Swagger UI** for interactive API exploration
5. **Maintain documentation** that automatically stays synchronized with code changes
6. **Test APIs interactively** using the Swagger UI "Try it out" feature

---

## Concepts Covered

### What is OpenAPI?

**OpenAPI Specification (OAS)** is an industry-standard format for describing REST APIs. It provides a machine-readable contract that defines:

- Available endpoints and operations
- Request parameters (path, query, headers, body)
- Request/response payload formats and examples
- Authentication methods
- Error responses

OpenAPI 3.0 is the current standard, supporting JSON and YAML formats. The specification enables:
- **Auto-generated client libraries** in multiple languages
- **Interactive documentation** with Swagger UI or ReDoc
- **API validation** and testing tools
- **Mock servers** for development

### Why Code-First Documentation?

Traditional API documentation approaches have significant problems:

**Manual Documentation Problems**:
- Gets out of sync with code changes
- Requires duplicate effort (code + docs)
- No compile-time validation
- Easy to forget updating after changes

**Code-First Benefits**:
- Documentation lives with code
- Compiler ensures accuracy
- Automatic updates when code changes
- Single source of truth
- Type-safe schema generation

### What is utoipa?

**utoipa** is a Rust library that generates OpenAPI documentation using derive macros and attributes. It leverages Rust's type system to create accurate API specifications at compile time.

**Key features**:
- `#[derive(ToSchema)]`: Generates JSON schemas from structs
- `#[utoipa::path]`: Documents handler functions
- `#[derive(OpenApi)]`: Composes full API specification
- Compile-time validation: Invalid docs won't compile
- Swagger UI integration: Interactive documentation
- Zero runtime overhead: Specs generated at build time

### utoipa Architecture

```
┌─────────────────────────────────────────────┐
│  DTOs with #[derive(ToSchema)]              │
│  - CreateMemoDto                            │
│  - MemoResponseDto                          │
│  - ErrorResponse                            │
└──────────────┬──────────────────────────────┘
               │ (generates)
               ▼
┌─────────────────────────────────────────────┐
│  JSON Schema Definitions                    │
│  - Type information                         │
│  - Validation rules                         │
│  - Field descriptions                       │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  Handlers with #[utoipa::path]              │
│  - Endpoint paths                           │
│  - HTTP methods                             │
│  - Parameters                               │
│  - Request/response bodies                  │
└──────────────┬──────────────────────────────┘
               │ (combined by)
               ▼
┌─────────────────────────────────────────────┐
│  #[derive(OpenApi)] struct                  │
│  - API metadata (title, version)            │
│  - paths() - All endpoints                  │
│  - components() - All schemas               │
│  - tags() - Endpoint grouping               │
└──────────────┬──────────────────────────────┘
               │ (serves)
               ▼
┌─────────────────────────────────────────────┐
│  Swagger UI at /swagger-ui/                 │
│  - Interactive documentation                │
│  - "Try it out" functionality               │
│  - Schema exploration                       │
└─────────────────────────────────────────────┘
```

### OpenAPI Components

An OpenAPI specification consists of several key sections:

**1. Info Section**
- API title, version, description
- Contact information
- License details

**2. Paths Section**
- All API endpoints
- HTTP methods (GET, POST, PUT, etc.)
- Parameters (path, query, headers)
- Request/response specifications
- Status codes and error handling

**3. Components Section**
- Reusable schemas (DTOs)
- Security schemes
- Response definitions
- Parameter definitions

**4. Tags Section**
- Logical grouping of endpoints
- Organization for documentation

### Documenting with Doc Comments

utoipa automatically extracts documentation from Rust `///` doc comments:

```rust
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service health status
    status: String,
    /// Database connection status
    database: String,
    /// Application version
    version: String,
    /// Service uptime in seconds
    uptime_seconds: u64,
}
```

**What happens:**
- `///` comments become field descriptions in OpenAPI schema
- Appears in Swagger UI when exploring schemas
- Provides context for fields with technical names

**Template strategy:**
- ✅ **Health/monitoring structs**: Use doc comments (technical field names)
- ❌ **DTOs**: No doc comments (self-explanatory names like `title`, `description`)
- ✅ **Handler parameters**: Use inline `description` in `#[utoipa::path]`

**Why selective documentation?**
- `uptime_seconds` benefits from "Service uptime in seconds" explanation
- `title` is already clear without "/// The title of the memo"
- Keeps code clean while documenting where it adds value

### Type Handling and Serialization

utoipa handles Rust types automatically, but some need special consideration:

**Automatically Supported:**
- Primitive types: `String`, `i32`, `u64`, `bool`, `f64`
- Standard collections: `Vec<T>`, `Option<T>`, `HashMap<K, V>`
- DateTime and UUID types (with features enabled)

**How DateTime and UUID work:**
```rust
pub struct MemoResponseDto {
    pub id: Uuid,           // Serialized as "string" (format: "uuid")
    pub date_to: DateTime<Utc>,  // Serialized as "string" (format: "date-time")
}
```

**Why this matters:**
- OpenAPI schema shows `type: string, format: date-time`
- Clients know to send/receive ISO 8601 timestamps
- UUID shown as `type: string, format: uuid`
- utoipa's `chrono` and `uuid` features enable this automatically

**Unsupported Types:**
If you use custom types not supported by utoipa:
```rust
// This won't work without custom implementation:
pub struct CustomDate(String);

// Solution: Implement `ToSchema` manually or use serde serialization
```

### Automatic Validator Integration

utoipa automatically extracts constraints from `validator` attributes:

```rust
#[derive(Validate, ToSchema)]
pub struct CreateMemoDto {
    #[validate(length(min = 1, max = 200))]
    pub title: String,  // OpenAPI shows: minLength: 1, maxLength: 200

    #[validate(range(min = 1, max = 100))]
    pub limit: u64,     // OpenAPI shows: minimum: 1, maximum: 100
}
```

**What gets extracted:**
- `length(min, max)` → `minLength`, `maxLength`
- `range(min, max)` → `minimum`, `maximum`
- `email` → `format: email`
- `url` → `format: uri`
- `Option<T>` → `required: false`

**Why this is powerful:**
- Single source of truth (validation rules in one place)
- No duplication between runtime validation and docs
- OpenAPI consumers can validate before sending requests
- API clients can generate better validation

---

## Step-by-Step Instructions

### Step 1: Add utoipa Dependencies

**Why**: utoipa requires two crates: the core library and Swagger UI integration.

**How**: Add these dependencies to `Cargo.toml`.

**Code**:

```toml
# In Cargo.toml, add to [dependencies] section

utoipa = { version = "5.3", features = ["actix_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "8.0", features = ["actix-web"] }
```

**Explanation**:
- `utoipa`: Core OpenAPI generation library
- Features:
  - `actix_extras`: Integration with Actix Web extractors
  - `chrono`: Support for DateTime types
  - `uuid`: Support for UUID types
- `utoipa-swagger-ui`: Provides Swagger UI web interface

**Verify**:

```bash
cargo check
```

You should see:
```
Checking utoipa v5.3.0
Checking utoipa-swagger-ui v8.0.0
Finished dev [unoptimized + debuginfo] target(s)
```

---

### Step 2: Add ToSchema to DTOs

**Why**: DTOs need schema definitions so OpenAPI knows their structure, validation rules, and field types.

**How**: Add `#[derive(ToSchema)]` to all DTO structs and import the trait.

**Code**:

```rust
// In src/dto/memo_dto.rs

use utoipa::ToSchema; // Add this import

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateMemoDto {
    // ... existing fields ...
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct MemoResponseDto {
    // ... existing fields ...
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateMemoDto {
    // ... existing fields ...
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct PatchMemoDto {
    // ... existing fields ...
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedMemoResponse {
    // ... existing fields ...
}
```

**What ToSchema generates**:
- Field names and types
- Required vs optional fields (Option<T>)
- Validation constraints (from `validator` attributes)
- Nested object schemas
- Array item types

**Verify**:

```bash
cargo check
```

---

### Step 3: Add ToSchema to Error Response

**Why**: API documentation must include error response formats so clients know what to expect when requests fail. The `ErrorResponse` struct was created in Chapter 3, and now we'll add OpenAPI schema generation to it.

**How**: Add `ToSchema` derive to the existing `ErrorResponse` struct.

**Code**:

```rust
// In src/error/app_error.rs

use utoipa::ToSchema; // Add this import at the top

// Update ErrorResponse to include ToSchema
#[derive(Serialize, ToSchema)]  // Add ToSchema here
pub struct ErrorResponse {
    pub error: String,      // Error type (e.g., "NotFound", "ValidationError")
    pub message: String,    // Human-readable error message
    pub status: u16,        // HTTP status code (e.g., 404, 400, 500)
}

// No changes needed to ResponseError implementation - it already uses this struct
```

**What this provides**:
- Clear error categorization with `error` field
- Human-readable messages with `message` field
- Explicit HTTP status codes with `status` field
- OpenAPI schema generation via `ToSchema`

**Example error response**:
```json
{
  "error": "NotFound",
  "message": "Memo with ID 550e8400-e29b-41d4-a716-446655440000 not found",
  "status": 404
}
```

**Verify**:

```bash
cargo check
```

---

### Step 4: Add ToSchema and Doc Comments to Health Check Responses

**Why**: Health check endpoints need schemas for their structured JSON responses. We'll also add doc comments to document what each field represents in the OpenAPI specification.

**How**: Add `ToSchema` derive and doc comments to health response structs.

**Code**:

```rust
// In src/handlers/health.rs

use utoipa::ToSchema; // Add this import

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service health status
    status: String,
    /// Database connection status
    database: String,
    /// Application version
    version: String,
    /// Service uptime in seconds
    uptime_seconds: u64,
}

#[derive(Serialize, ToSchema)]
pub struct ReadyResponse {
    /// Service readiness status
    ready: bool,
}

// ... rest of file unchanged ...
```

**What the doc comments do:**
- `/// Service health status` → Appears as field description in OpenAPI schema
- Swagger UI displays these when exploring the HealthResponse model
- Provides context for API consumers

**Why we use doc comments here but not on DTOs:**
- Health responses have technical field names (`uptime_seconds`, `database`)
- DTOs have self-explanatory names (`title`, `description`, `date_to`)
- Doc comments add value where field purpose isn't obvious

**Verify**:

```bash
cargo check
```

---

### Step 5: Document Handler Functions with utoipa::path

**Why**: Each handler needs annotations describing its endpoint, parameters, request body, and possible responses.

**How**: Add `#[utoipa::path(...)]` attributes above each handler function.

**Code**:

```rust
// In src/handlers/memos.rs

use utoipa;

/// List all memos
///
/// Retrieve a paginated list of memos with optional filtering by completion status and sorting by various fields
#[utoipa::path(
    get,
    path = "/api/v1/memos",
    tag = "memos",
    params(
        ("limit" = Option<u64>, Query, description = "Number of items per page (1-100, default: 10)"),
        ("offset" = Option<u64>, Query, description = "Number of items to skip (default: 0)"),
        ("completed" = Option<bool>, Query, description = "Filter by completion status"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (created_at, title, date_to, completed, updated_at)"),
        ("order" = Option<String>, Query, description = "Sort order (asc or desc, default: desc)")
    ),
    responses(
        (status = 200, description = "List of memos retrieved successfully", body = PaginatedMemoResponse),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state, params))]
#[get("/api/v1/memos")]
pub async fn list_memos(
    state: web::Data<AppState>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    // ... implementation from Chapter 8 ...
}

/// Get a memo by ID
///
/// Retrieve a single memo by its unique identifier
#[utoipa::path(
    get,
    path = "/api/v1/memos/{id}",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    responses(
        (status = 200, description = "Memo retrieved successfully", body = MemoResponseDto),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[get("/api/v1/memos/{id}")]
pub async fn get_memo(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    // ... implementation from Chapter 8 ...
}

/// Create a new memo
///
/// Create a new memo with title, optional description, and due date
#[utoipa::path(
    post,
    path = "/api/v1/memos",
    tag = "memos",
    request_body = CreateMemoDto,
    responses(
        (status = 201, description = "Memo created successfully", body = MemoResponseDto),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state, dto), fields(title = %dto.title))]
#[post("/api/v1/memos")]
pub async fn create_memo(
    state: web::Data<AppState>,
    dto: web::Json<CreateMemoDto>,
) -> impl Responder {
    // ... implementation from Chapter 8 ...
}

/// Update a memo
///
/// Fully update an existing memo with all fields (title, description, due date, and completion status)
#[utoipa::path(
    put,
    path = "/api/v1/memos/{id}",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    request_body = UpdateMemoDto,
    responses(
        (status = 200, description = "Memo updated successfully", body = MemoResponseDto),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state, dto), fields(memo_id = %id))]
#[put("/api/v1/memos/{id}")]
pub async fn update_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateMemoDto>,
) -> impl Responder {
    // ... implementation from Chapter 8 ...
}

/// Partially update a memo
///
/// Update one or more fields of an existing memo. Only provided fields will be updated.
#[utoipa::path(
    patch,
    path = "/api/v1/memos/{id}",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    request_body = PatchMemoDto,
    responses(
        (status = 200, description = "Memo updated successfully", body = MemoResponseDto),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state, dto), fields(memo_id = %id))]
#[patch("/api/v1/memos/{id}")]
pub async fn patch_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    dto: web::Json<PatchMemoDto>,
) -> impl Responder {
    // ... implementation from Chapter 8 ...
}

/// Delete a memo
///
/// Permanently delete a memo by its ID
#[utoipa::path(
    delete,
    path = "/api/v1/memos/{id}",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    responses(
        (status = 204, description = "Memo deleted successfully"),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[delete("/api/v1/memos/{id}")]
pub async fn delete_memo(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    // ... implementation from Chapter 8 ...
}

/// Toggle memo completion
///
/// Toggle the completion status of a memo (completed ↔ incomplete)
#[utoipa::path(
    patch,
    path = "/api/v1/memos/{id}/complete",
    tag = "memos",
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    responses(
        (status = 200, description = "Memo completion toggled successfully", body = MemoResponseDto),
        (status = 404, description = "Memo not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[patch("/api/v1/memos/{id}/complete")]
pub async fn toggle_complete(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    // ... implementation from Chapter 8 ...
}
```

**What the doc comments do:**

The `/// Summary\n///\n/// Description` comments above each handler appear in the OpenAPI specification:
- First line becomes the operation summary (shown in endpoint list)
- Lines after blank line become the detailed description
- Appears in Swagger UI when you expand an endpoint
- Helps API consumers understand what each endpoint does

**Example**:
- `/// List all memos` → Operation summary in Swagger UI
- `/// Retrieve a paginated list of...` → Detailed description

**Key points**:
- `/// ... ///`: Doc comments extracted by utoipa into OpenAPI spec
- `get/post/put/patch/delete`: HTTP method
- `path`: API endpoint URL
- `tag`: Groups endpoints in Swagger UI
- `params()`: Path, query, and header parameters
- `request_body`: Request payload schema
- `responses()`: All possible status codes with schemas
- Position: Doc comments → #[utoipa::path] → #[tracing::instrument]

**Verify**:

```bash
cargo check
```

---

### Step 6: Document Health Check Handlers

**Why**: Health endpoints should be documented for monitoring and operations teams.

**How**: Add `#[utoipa::path]` to health check handlers.

**Code**:

```rust
// In src/handlers/health.rs

use utoipa;

#[utoipa::path(
    get,
    path = "/health",
    tag = "Observability",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
#[get("/health")]
#[tracing::instrument(name = "GET /health", skip(state))]
pub async fn health(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // ... implementation from Chapter 4 ...
}

#[utoipa::path(
    get,
    path = "/ready",
    tag = "Observability",
    responses(
        (status = 200, description = "Service is ready", body = ReadyResponse),
        (status = 503, description = "Service is not ready", body = ReadyResponse),
    )
)]
#[get("/ready")]
#[tracing::instrument(name = "GET /ready", skip(state))]
pub async fn ready(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // ... implementation from Chapter 4 ...
}
```

**Note about doc comments:**

Health handlers don't have `/// ...` doc comments above the function in the template. The inline `description` in `#[utoipa::path]` responses is sufficient for these simple endpoints. This follows the template's selective documentation strategy - doc comments where they add value, inline descriptions otherwise.

**Verify**:

```bash
cargo check
```

---

### Step 7: Create OpenAPI Documentation Structure

**Why**: You need a central struct that combines all paths, schemas, and metadata into a complete OpenAPI specification.

**How**: Create a new module `src/docs/` with the OpenAPI composition.

**Code**:

Create `src/docs/mod.rs`:

```rust
pub mod openapi;

pub use openapi::ApiDoc;
```

Create `src/docs/openapi.rs`:

```rust
use utoipa::OpenApi;

use crate::{
    dto::{CreateMemoDto, MemoResponseDto, PaginatedMemoResponse, PatchMemoDto, UpdateMemoDto},
    error::ErrorResponse,
    handlers::{health, memos},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Memos API",
        version = "0.1.0",
        description = "A RESTful API for managing memos with full CRUD operations, pagination, filtering, and sorting. Includes observability endpoints for health checks and metrics.",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
        )
    ),
    paths(
        memos::list_memos,
        memos::get_memo,
        memos::create_memo,
        memos::update_memo,
        memos::patch_memo,
        memos::delete_memo,
        memos::toggle_complete,
        health::health,
        health::ready,
    ),
    components(
        schemas(
            MemoResponseDto,
            CreateMemoDto,
            UpdateMemoDto,
            PatchMemoDto,
            PaginatedMemoResponse,
            ErrorResponse,
            health::HealthResponse,
            health::ReadyResponse,
        )
    ),
    tags(
        (name = "memos", description = "Memo management endpoints"),
        (name = "Observability", description = "Health checks and monitoring endpoints. Metrics available at /metrics endpoint (Prometheus format).")
    )
)]
pub struct ApiDoc;
```

**Explanation**:

- `#[derive(OpenApi)]`: Generates the OpenAPI specification
- `info()`: API metadata (title, version, description, contact, license)
- `paths()`: All documented handler functions
- `components(schemas())`: All DTO schemas
- `tags()`: Logical grouping and descriptions

**Add to lib.rs**:

```rust
// In src/lib.rs

pub mod docs; // Add this line
// ... existing modules ...
```

**Verify**:

```bash
cargo check
```

---

### Step 8: Configure Swagger UI

**Why**: Swagger UI provides an interactive web interface for exploring and testing your API.

**How**: Add Swagger UI service to your Actix Web application.

**Code**:

```rust
// In src/main.rs

use utoipa::OpenApi; // Add this
use utoipa_swagger_ui::SwaggerUi; // Add this

use actix_web_template::docs::ApiDoc; // Add this

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ... existing setup code ...

    HttpServer::new(move || {
        // ... existing middleware setup ...

        let openapi = ApiDoc::openapi(); // Generate OpenAPI spec

        App::new()
            // ... existing configuration ...
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            // ... rest of services ...
    })
    .bind((settings.server.host.clone(), settings.server.port))?
    .run()
    .await?;

    Ok(())
}
```

**Explanation**:
- `ApiDoc::openapi()`: Generates the OpenAPI specification
- `SwaggerUi::new("/swagger-ui/{_:.*}")`: Serves Swagger UI at this path
- `.url("/api-docs/openapi.json", openapi.clone())`: Makes JSON spec available
- Position: Register BEFORE other services for proper routing

**Verify**:

```bash
cargo check
cargo run
```

---

### Step 9: Test API Documentation

**Why**: Verify that documentation is generated correctly and Swagger UI is accessible.

**How**: Access Swagger UI in your browser and test endpoints.

**Testing steps**:

1. **Start the application**:
   ```bash
   cargo run
   ```

2. **Open Swagger UI**:
   - Navigate to `http://localhost:3737/swagger-ui/`
   - You should see the Swagger UI interface

3. **Explore API structure**:
   - View "memos" tag with 7 endpoints
   - View "Observability" tag with 2 endpoints
   - Check schemas section for DTOs

4. **Test an endpoint**:
   - Click on `GET /api/v1/memos`
   - Click "Try it out"
   - Set parameters (e.g., limit=5)
   - Click "Execute"
   - View response body, headers, and status code

5. **Check JSON specification**:
   - Navigate to `http://localhost:3737/api-docs/openapi.json`
   - View raw OpenAPI JSON specification
   - Verify all paths and schemas are present

6. **Test with curl**:
   ```bash
   # Get OpenAPI spec
   curl http://localhost:3737/api-docs/openapi.json | jq '.info'

   # Should return:
   {
     "title": "Memos API",
     "version": "0.1.0",
     "description": "A RESTful API for managing memos...",
     ...
   }
   ```

**Verify**:

All tests should pass:
- Swagger UI loads without errors
- All 9 endpoints are documented
- "Try it out" functionality works
- Schemas show validation rules
- JSON spec is valid OpenAPI 3.0

---

## Checkpoint

At this point, you should have:

1. **Working Swagger UI** at `http://localhost:3737/swagger-ui/`
2. **OpenAPI JSON spec** at `http://localhost:3737/api-docs/openapi.json`
3. **All 9 endpoints documented**:
   - 7 memo management endpoints
   - 2 observability endpoints
4. **Interactive testing** via "Try it out" functionality
5. **Schema documentation** with validation rules automatically extracted

**Verify everything works**:

```bash
# Run the application
cargo run

# In another terminal, test the OpenAPI spec
curl http://localhost:3737/api-docs/openapi.json | jq '.info.title'
# Should output: "Memos API"

# List all documented paths
curl http://localhost:3737/api-docs/openapi.json | jq '.paths | keys'
# Should show all 9 endpoints

# Check a specific endpoint
curl http://localhost:3737/api-docs/openapi.json | jq '.paths["/api/v1/memos"].get'
# Should show the list_memos documentation
```

**Expected results**:
- All commands execute without errors
- JSON output is well-formatted
- All endpoints are documented
- Schemas include validation rules

---

## Common Issues and Solutions

### Issue: Swagger UI shows "Failed to load API definition"

**Symptoms**: Swagger UI loads but displays error message about loading API definition.

**Cause**: OpenAPI JSON endpoint not accessible or returns invalid JSON.

**Solution**:
1. Check that `/api-docs/openapi.json` is accessible:
   ```bash
   curl http://localhost:3737/api-docs/openapi.json
   ```
2. Verify SwaggerUi service is registered in `main.rs`:
   ```rust
   .service(
       SwaggerUi::new("/swagger-ui/{_:.*}")
           .url("/api-docs/openapi.json", openapi.clone()),
   )
   ```
3. Ensure ApiDoc is imported and openapi() is called
4. Check for compilation errors: `cargo check`

---

### Issue: Handler not appearing in documentation

**Symptoms**: Some endpoints are missing from Swagger UI.

**Cause**: Handler not listed in `paths()` section of OpenAPI struct or missing `#[utoipa::path]` annotation.

**Solution**:
1. Verify handler has `#[utoipa::path(...)]` attribute
2. Check that handler is listed in `ApiDoc` `paths()`:
   ```rust
   paths(
       memos::list_memos,  // Make sure your handler is here
       // ... other handlers
   ),
   ```
3. Ensure handler module is public and accessible
4. Rebuild: `cargo clean && cargo build`

---

### Issue: Schema not found error

**Symptoms**: Compilation error: "cannot find struct `MyDto` in scope" or runtime "schema not found".

**Cause**: Schema used in handler but not listed in `components(schemas())` section.

**Solution**:
1. Add `#[derive(ToSchema)]` to your DTO:
   ```rust
   #[derive(Serialize, Deserialize, ToSchema)]
   pub struct MyDto { ... }
   ```
2. List it in OpenAPI components:
   ```rust
   components(
       schemas(
           MyDto,  // Add here
           // ... other schemas
       )
   )
   ```
3. Ensure DTO is imported in `openapi.rs`

---

### Issue: Validation rules not showing in docs

**Symptoms**: Swagger UI doesn't show min/max length, required fields, etc.

**Cause**: ToSchema doesn't automatically extract validator attributes.

**Solution**:

You can manually add schema attributes:
```rust
#[derive(Validate, ToSchema)]
pub struct CreateMemoDto {
    #[validate(length(min = 1, max = 200))]
    #[schema(min_length = 1, max_length = 200)]  // Add explicit schema rules
    pub title: String,
}
```

Or document validation in field descriptions:
```rust
#[schema(description = "Memo title (1-200 characters)")]
pub title: String,
```

---

### Issue: Wrong HTTP status code in documentation

**Symptoms**: Swagger UI shows incorrect status codes for responses.

**Cause**: Mismatch between handler implementation and documentation.

**Solution**:
1. Review handler implementation to see what status codes it returns
2. Update `#[utoipa::path]` responses to match:
   ```rust
   responses(
       (status = 200, description = "Success", body = MyDto),
       (status = 404, description = "Not found", body = ErrorResponse),
       (status = 500, description = "Server error", body = ErrorResponse)
   )
   ```
3. Ensure all possible error paths are documented

---

### Issue: Example values not appearing

**Symptoms**: Swagger UI doesn't show example request/response data.

**Cause**: Missing `#[schema(example = ...)]` attributes or incorrect format.

**Solution**:
1. Add examples to DTO fields:
   ```rust
   #[schema(example = "Example value")]
   pub field: String,
   ```
2. For complex types, use correct format:
   ```rust
   #[schema(example = "2025-01-06T12:00:00Z")]  // DateTime
   #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]  // UUID
   ```
3. Rebuild after adding examples

---

## Code Review

### Key Design Principles Demonstrated

This implementation demonstrates several critical API documentation principles:

**1. Documentation as Code**

OpenAPI documentation lives with your code, not in separate files:
- Schemas derived from DTOs via `#[derive(ToSchema)]`
- Endpoints documented with `#[utoipa::path]` on handlers
- Compile-time validation ensures accuracy
- Changes to code automatically update documentation

**Why this matters**: Eliminates documentation drift, reduces maintenance burden, and guarantees accuracy.

**2. Single Source of Truth**

Type definitions serve both runtime and documentation:
```rust
#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateMemoDto {
    pub title: String,  // Used at runtime AND in OpenAPI schema
    // ...
}
```

One struct definition generates:
- JSON serialization rules (serde)
- Validation logic (validator)
- OpenAPI schema (utoipa)

**Why this matters**: No duplication, no inconsistency, no manual synchronization.

**3. Compile-Time Safety**

Invalid documentation won't compile:
```rust
paths(
    memos::nonexistent_handler,  // Compilation error!
)

components(
    schemas(
        UndefinedDto,  // Compilation error!
    )
)
```

**Why this matters**: Catches documentation errors at build time, not runtime.

**4. Progressive Enhancement**

Start simple, add detail incrementally:
1. Basic: `#[derive(ToSchema)]` - generates schema from type
2. Enhanced: Add `#[schema(example = ...)]` - improves examples
3. Advanced: Add descriptions, custom formats, validation rules

**Why this matters**: Quick to implement basics, can refine over time.

---

### Architecture Benefits

**Automated Synchronization**
- Code changes automatically reflect in documentation
- No manual OpenAPI file editing
- No risk of outdated documentation

**Type Safety**
- Rust's type system ensures schema accuracy
- Validation rules automatically documented
- Optional vs required fields correctly represented

**Developer Experience**
- Interactive testing via Swagger UI
- "Try it out" with pre-filled examples
- Schema exploration for complex types

**Client Generation**
- OpenAPI spec enables auto-generated clients
- Consistent contracts across services
- Type-safe client libraries in multiple languages

**Team Collaboration**
- API contracts visible to all team members
- Backend and frontend teams aligned on types
- Non-technical stakeholders can explore APIs

---

### Complete OpenAPI Structure

Let's examine the complete architecture from all angles:

**src/docs/openapi.rs** - Central Documentation:
```rust
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Memos API",
        version = "0.1.0",
        description = "...",
        contact(name = "API Support", email = "support@example.com"),
        license(name = "MIT"),
    ),
    paths(
        // All handler functions that should appear in docs
        memos::list_memos,
        memos::get_memo,
        // ... all 9 handlers
    ),
    components(
        schemas(
            // All DTOs used in requests/responses
            MemoResponseDto,
            CreateMemoDto,
            // ... all schemas
        )
    ),
    tags(
        // Logical grouping for organization
        (name = "memos", description = "Memo management endpoints"),
        (name = "Observability", description = "Health checks..."),
    )
)]
pub struct ApiDoc;
```

**Handler Documentation Pattern**:
```rust
#[utoipa::path(
    get,                                    // HTTP method
    path = "/api/v1/memos/{id}",           // Endpoint URL
    tag = "memos",                          // Group in docs
    params(
        ("id" = Uuid, Path, description = "Memo ID")
    ),
    responses(
        (status = 200, description = "Success", body = MemoResponseDto),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip(state), fields(memo_id = %id))]
#[get("/api/v1/memos/{id}")]
pub async fn get_memo(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Implementation
}
```

**DTO Schema Pattern**:
```rust
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateMemoDto {
    #[validate(length(min = 1, max = 200))]
    #[schema(example = "Buy groceries", min_length = 1, max_length = 200)]
    pub title: String,

    #[validate(length(max = 2000))]
    #[schema(example = "Detailed description")]
    pub description: Option<String>,

    #[schema(example = "2025-01-15T10:00:00Z")]
    pub date_to: DateTime<Utc>,
}
```

**Integration in main.rs**:
```rust
let openapi = ApiDoc::openapi();

App::new()
    // ... middleware ...
    .service(
        SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", openapi.clone()),
    )
    // ... other services ...
```

---

## Testing

### Manual Testing with Swagger UI

1. **Start the application**:
   ```bash
   cargo run
   ```

2. **Open Swagger UI**: `http://localhost:3737/swagger-ui/`

3. **Test each endpoint**:
   - **List Memos**: GET `/api/v1/memos`
     - Try different query parameters (limit, offset, completed)
     - Verify pagination works
     - Check response schema

   - **Get Memo**: GET `/api/v1/memos/{id}`
     - Use an existing memo ID
     - Try invalid UUID format
     - Verify 404 response

   - **Create Memo**: POST `/api/v1/memos`
     - Use example request body
     - Try invalid data (empty title, too long description)
     - Check validation errors

   - **Update Memo**: PUT `/api/v1/memos/{id}`
     - Modify existing memo
     - Try partial data (should fail - requires all fields)

   - **Patch Memo**: PATCH `/api/v1/memos/{id}`
     - Update single field
     - Verify partial update works

   - **Delete Memo**: DELETE `/api/v1/memos/{id}`
     - Delete a memo
     - Verify 204 response
     - Try deleting again (should 404)

   - **Toggle Complete**: PATCH `/api/v1/memos/{id}/complete`
     - Toggle completion status
     - Verify boolean flips

4. **Verify schema documentation**:
   - Click on "Schemas" section
   - Expand each DTO
   - Check field types, validation rules, examples
   - Verify optional vs required fields

### Automated Testing

Add a test to verify OpenAPI spec generation:

```rust
// In tests/api_tests.rs (or create tests/openapi_tests.rs)

#[actix_web::test]
async fn test_openapi_spec_available() {
    let app = test::init_service(
        App::new().service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api-docs/openapi.json")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let spec: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify OpenAPI structure
    assert_eq!(spec["openapi"], "3.0.3");
    assert_eq!(spec["info"]["title"], "Memos API");
    assert!(spec["paths"].is_object());
    assert!(spec["components"]["schemas"].is_object());
}

#[actix_web::test]
async fn test_swagger_ui_accessible() {
    let app = test::init_service(
        App::new().service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/swagger-ui/")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Swagger UI returns OK and HTML content
    assert_eq!(resp.status(), StatusCode::OK);
}
```

**Run tests**:
```bash
cargo test test_openapi
```

### Testing with External Tools

**1. OpenAPI Validator**:
```bash
# Install openapi-generator-cli
npm install -g @openapitools/openapi-generator-cli

# Validate spec
openapi-generator-cli validate -i http://localhost:3737/api-docs/openapi.json
```

**2. Generate Client SDK**:
```bash
# Generate TypeScript client
openapi-generator-cli generate \
  -i http://localhost:3737/api-docs/openapi.json \
  -g typescript-axios \
  -o ./generated-client
```

**3. Use Postman**:
- Import OpenAPI spec from `http://localhost:3737/api-docs/openapi.json`
- Auto-generate Postman collection
- Test all endpoints with generated requests

---

## Summary

### What You Learned

In this chapter, you:

1. **Implemented OpenAPI 3.0 documentation** using utoipa for automatic generation
2. **Added schema definitions** to DTOs with `#[derive(ToSchema)]`
3. **Documented all API endpoints** with `#[utoipa::path]` annotations
4. **Configured Swagger UI** for interactive API exploration
5. **Created a central OpenAPI specification** combining all paths and schemas
6. **Added example values** to improve documentation quality
7. **Tested API documentation** both manually and programmatically

### Key Takeaways

**Code-First Documentation is Superior**:
- Eliminates documentation drift
- Single source of truth for types
- Compile-time validation
- Automatic synchronization

**utoipa's Three-Level Architecture**:
1. **DTOs with ToSchema** → JSON schemas
2. **Handlers with utoipa::path** → Endpoint docs
3. **OpenApi struct** → Complete specification

**Swagger UI Benefits**:
- Interactive testing without external tools
- "Try it out" with example data
- Schema exploration
- Team collaboration

**Production Readiness**:
- Professional API documentation
- Client SDK generation support
- Team alignment on contracts
- Integration testing capabilities

### How It Fits in the Architecture

OpenAPI documentation is a **cross-cutting concern** that enhances your entire API:

```
┌─────────────────────────────────────────────┐
│  Swagger UI (/swagger-ui/)                  │
│  Interactive documentation & testing        │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│  OpenAPI Spec (/api-docs/openapi.json)      │
│  Machine-readable API contract              │
└──────────────┬──────────────────────────────┘
               │
      ┌────────┴────────┐
      ▼                 ▼
┌──────────────┐  ┌──────────────┐
│   Handlers   │  │     DTOs     │
│ (endpoints)  │  │  (schemas)   │
└──────────────┘  └──────────────┘
```

Documentation integrates with:
- **DTOs (Chapter 5)**: Schema generation
- **Handlers (Chapter 8)**: Endpoint documentation
- **Error Handling (Chapter 3)**: Error response docs
- **Health Checks (Chapter 4)**: Monitoring endpoint docs

---

## Next Steps

### Required: Chapter 10 - Askama Templates

Now that you have a complete REST API with documentation, you'll add server-side rendered HTML pages using Askama templates. You'll learn compile-time template rendering with type-safe template composition.

### Optional Exercises

1. **Experiment with doc comments on DTOs** (Optional Enhancement):
   - Try adding `///` doc comments to DTO fields (template doesn't use them)
   - Example: `/// The title of the memo (1-200 characters)`
   - Observe how they appear in Swagger UI
   - Decide if added documentation value justifies extra lines

2. **Add example values to schemas**:
   - Use `#[schema(example = "...")]` attributes on DTO fields
   - Provide realistic example data for each field
   - Verify "Try it out" pre-fills with your examples

3. **Create API documentation page**:
   - Add a `/docs` route serving custom HTML
   - Link to Swagger UI and OpenAPI JSON
   - Include API usage examples and guides

4. **Generate client SDKs**:
   - Use openapi-generator to create TypeScript client
   - Create Python client with openapi-generator
   - Compare auto-generated vs hand-written clients

5. **Add request/response examples to handlers**:
   - Use utoipa's example feature for complex scenarios
   - Show multiple response examples (success, validation error, not found)
   - Document error cases comprehensively

---

## Additional Resources

### Official Documentation
- [utoipa documentation](https://docs.rs/utoipa/)
- [OpenAPI 3.0 Specification](https://swagger.io/specification/)
- [Swagger UI documentation](https://swagger.io/tools/swagger-ui/)

### Tutorials and Guides
- [OpenAPI Best Practices](https://oai.github.io/Documentation/best-practices.html)
- [utoipa examples](https://github.com/juhaku/utoipa/tree/master/examples)
- [API Documentation Guide](https://swagger.io/blog/api-documentation/)

### Tools
- [OpenAPI Generator](https://openapi-generator.tech/) - Generate client SDKs
- [Redoc](https://redocly.com/redoc/) - Alternative documentation UI
- [Spectral](https://stoplight.io/open-source/spectral) - OpenAPI linter
- [Swagger Editor](https://editor.swagger.io/) - Online OpenAPI editor

---

**Congratulations!** You now have professional, interactive API documentation that stays automatically synchronized with your code. Your API is ready for consumption by frontend developers, mobile apps, and third-party integrations.
