# Project Implementation Plan

This document outlines the staged implementation plan for the Actix Web memos application, following best practices and ensuring a solid foundation at each stage.

## Branching Strategy

Each stage should be completed on its own branch for easy restoration and review:

```bash
# Create branch for stage
git checkout -b stage-0-project-setup

# Complete stage work...

# Commit and tag the stage
git add .
git commit -m "Complete Stage 0: Project Setup"
git tag stage-0-complete

# Merge to main
git checkout main
git merge stage-0-project-setup

# For next stage
git checkout -b stage-1-core-application
```

### Branch Naming
- **Stage branches**: `stage-N-short-name` (e.g., `stage-1-core-application`)
- **Tags**: `stage-N-complete` (e.g., `stage-1-complete`)

### Restore Points
Each stage tag is a restore point:
```bash
# List all stage tags
git tag | grep stage

# Restore to specific stage
git checkout stage-5-complete

# Create new branch from stage
git checkout -b fix-from-stage-5 stage-5-complete
```

---

## Stage 0: Project Setup & Infrastructure

**Goal:** Set up the development environment and basic project structure

### Tasks
- [ ] Initialize Cargo.toml with Rust edition 2024
- [ ] Create comprehensive .gitignore file
- [ ] Set up project directory structure (src/, migration/, templates/, static/)
- [ ] Create docker-compose.yml with PostgreSQL service
- [ ] Create multi-stage Dockerfile
- [ ] Create .dockerignore file
- [ ] Create .env.example with all environment variables
- [ ] Update CLAUDE.md with project architecture
- [ ] Initialize git repository and make initial commit

### Files to Create
```
.gitignore
Cargo.toml
docker-compose.yml
Dockerfile
.dockerignore
.env.example
```

### .env.example Contents
```bash
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3737
APP_ENV=development

# Database Configuration
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos
DATABASE_MAX_CONNECTIONS=10
DATABASE_CONNECT_TIMEOUT=30

# Logging
RUST_LOG=info
LOG_FORMAT=pretty

# Security
CORS_ALLOWED_ORIGINS=*
MAX_REQUEST_SIZE=262144

# Features
ENABLE_SWAGGER=true
```

### Deliverables
- ✅ Working development environment
- ✅ Docker Compose starts PostgreSQL successfully
- ✅ Basic project structure in place
- ✅ All environment variables documented

**Estimated Time:** 2-4 hours

---

## Stage 1: Core Application & Configuration

**Goal:** Create the basic application skeleton with configuration management

### Tasks
- [ ] Create src/lib.rs as library root
- [ ] Create src/config/settings.rs with Settings struct
- [ ] Implement configuration loading from environment
- [ ] Add configuration validation
- [ ] Create src/main.rs with basic server setup
- [ ] Set up tracing-subscriber for structured logging
- [ ] Add simple /health endpoint for testing
- [ ] Implement graceful shutdown
- [ ] Test server starts and /health returns JSON

### Dependencies to Add
```toml
[dependencies]
actix-web = "4"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
thiserror = "1"
anyhow = "1"
```

### Configuration Structure
```rust
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub logging: LoggingSettings,
    pub security: SecuritySettings,
}
```

### Deliverables
- ✅ Server starts successfully
- ✅ Configuration loads from .env
- ✅ Structured logging works
- ✅ Graceful shutdown implemented

**Estimated Time:** 4-6 hours

---

## Stage 2: Database Setup & ORM Integration

**Goal:** Set up database connection, migrations, and ORM

### Tasks
- [ ] Add SeaORM dependencies
- [ ] Create database connection pool
- [ ] Initialize migration project (migration/ directory)
- [ ] Create migration for memos table with indexes
- [ ] Run migrations
- [ ] Generate entities from database
- [ ] Add database connection to app state
- [ ] Test database connectivity

### Dependencies to Add
```toml
sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
uuid = { version = "1", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Migration Commands
```bash
# Install SeaORM CLI
cargo install sea-orm-cli

# Initialize migration
sea-orm-cli migrate init

# Create migration
sea-orm-cli migrate generate create_memos_table

# Run migrations
sea-orm-cli migrate up

# Generate entities
sea-orm-cli generate entity -o src/entities
```

### Deliverables
- ✅ Database migrations run successfully
- ✅ Entities auto-generated
- ✅ Connection pool configured
- ✅ Database connectivity verified

**Estimated Time:** 4-6 hours

---

## Stage 3: Error Handling & Middleware

**Goal:** Implement robust error handling and essential middleware

### Tasks
- [ ] Create src/error/app_error.rs with error types
- [ ] Implement ResponseError trait for Actix Web
- [ ] Create error response DTOs
- [ ] Implement request ID middleware
- [ ] Add tracing-actix-web middleware
- [ ] Configure CORS middleware
- [ ] Add request size limits
- [ ] Test error responses return proper JSON

### Dependencies to Add
```toml
actix-cors = "0.7"
tracing-actix-web = "0.7"
```

### Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

### Deliverables
- ✅ All errors return JSON responses
- ✅ Request IDs generated and logged
- ✅ CORS configured
- ✅ All requests traced

**Estimated Time:** 4-6 hours

---

## Stage 4: Health Check & Monitoring Endpoints

**Goal:** Implement health checks and monitoring

### Tasks
- [ ] Create src/handlers/health.rs
- [ ] Implement /health endpoint with DB check
- [ ] Implement /ready endpoint for Kubernetes
- [ ] Add version info to health response
- [ ] Add health endpoints to router
- [ ] Test all health endpoints

### Health Response
```json
{
  "status": "healthy",
  "database": "connected",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

### Deliverables
- ✅ /health returns status with DB check
- ✅ /ready endpoint works
- ✅ Version info included

**Estimated Time:** 2-4 hours

---

## Stage 5: DTOs & Validation

**Goal:** Create data transfer objects with validation

### Tasks
- [ ] Create src/dto/memo_dto.rs
- [ ] Implement CreateMemoDto with validators
- [ ] Implement UpdateMemoDto with validators
- [ ] Implement PatchMemoDto for partial updates
- [ ] Implement MemoResponseDto
- [ ] Create PaginationParams
- [ ] Create PaginatedResponse<T>
- [ ] Create ErrorResponse DTO
- [ ] Test validation rules

### Dependencies to Add
```toml
validator = { version = "0.18", features = ["derive"] }
```

### DTOs
```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMemoDto {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    pub date_to: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MemoResponseDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub date_to: DateTime<Utc>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Deliverables
- ✅ All DTOs defined with validation
- ✅ Validation errors return proper responses
- ✅ DTOs separate from entities

**Estimated Time:** 3-5 hours

---

## Stage 6: Repository Layer

**Goal:** Implement database access layer

### Tasks
- [ ] Create src/repository/memo_repository.rs
- [ ] Implement find_all with pagination
- [ ] Implement filtering by completion status
- [ ] Implement sorting (by date, title, etc.)
- [ ] Implement find_by_id
- [ ] Implement create
- [ ] Implement update
- [ ] Implement delete
- [ ] Add tracing instrumentation
- [ ] Write repository tests

### Repository Interface
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
    ) -> Result<(Vec<memo::Model>, u64), DbErr>;

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<memo::Model>, DbErr>;

    pub async fn create(
        db: &DatabaseConnection,
        title: String,
        description: Option<String>,
        date_to: DateTime<Utc>,
    ) -> Result<memo::Model, DbErr>;

    pub async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        title: String,
        description: Option<String>,
        date_to: DateTime<Utc>,
        completed: bool,
    ) -> Result<memo::Model, DbErr>;

    pub async fn delete(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<bool, DbErr>;
}
```

### Deliverables
- ✅ All repository methods implemented
- ✅ Tracing instrumentation added
- ✅ Tests pass
- ✅ Pagination, filtering, sorting work

**Estimated Time:** 6-8 hours

---

## Stage 7: Service Layer

**Goal:** Implement business logic layer

### Tasks
- [ ] Create src/services/memo_service.rs
- [ ] Implement MemoService struct
- [ ] Add get_all_memos with pagination
- [ ] Add get_memo_by_id
- [ ] Add create_memo with validation
- [ ] Add update_memo
- [ ] Add patch_memo for partial updates
- [ ] Add delete_memo
- [ ] Add toggle_complete
- [ ] Convert between DTOs and entities
- [ ] Add tracing instrumentation
- [ ] Write unit tests

### Service Interface
```rust
pub struct MemoService {
    db: DatabaseConnection,
}

impl MemoService {
    pub async fn get_all_memos(
        &self,
        limit: u64,
        offset: u64,
        completed: Option<bool>,
        sort_by: String,
        order: String,
    ) -> Result<PaginatedResponse<MemoResponseDto>, AppError>;

    pub async fn get_memo_by_id(
        &self,
        id: Uuid,
    ) -> Result<MemoResponseDto, AppError>;

    pub async fn create_memo(
        &self,
        dto: CreateMemoDto,
    ) -> Result<MemoResponseDto, AppError>;

    pub async fn update_memo(
        &self,
        id: Uuid,
        dto: UpdateMemoDto,
    ) -> Result<MemoResponseDto, AppError>;

    pub async fn patch_memo(
        &self,
        id: Uuid,
        dto: PatchMemoDto,
    ) -> Result<MemoResponseDto, AppError>;

    pub async fn delete_memo(
        &self,
        id: Uuid,
    ) -> Result<(), AppError>;

    pub async fn toggle_complete(
        &self,
        id: Uuid,
    ) -> Result<MemoResponseDto, AppError>;
}
```

### Deliverables
- ✅ Service layer fully implemented
- ✅ Business logic testable
- ✅ Unit tests pass
- ✅ Tracing instrumentation works

**Estimated Time:** 6-8 hours

---

## Stage 8: REST API Handlers

**Goal:** Implement REST API endpoints

### Tasks
- [ ] Create src/handlers/memos.rs
- [ ] Implement GET /api/v1/memos (list)
- [ ] Implement GET /api/v1/memos/{id}
- [ ] Implement POST /api/v1/memos
- [ ] Implement PUT /api/v1/memos/{id}
- [ ] Implement PATCH /api/v1/memos/{id}
- [ ] Implement DELETE /api/v1/memos/{id}
- [ ] Implement PATCH /api/v1/memos/{id}/complete
- [ ] Add tracing instrumentation
- [ ] Configure routes in main.rs
- [ ] Write integration tests

### API Endpoints
```rust
// GET /api/v1/memos?limit=10&offset=0&completed=false&sort_by=created_at&order=desc
async fn list_memos(
    query: Query<PaginationParams>,
    service: Data<MemoService>,
) -> Result<Json<PaginatedResponse<MemoResponseDto>>, AppError>

// GET /api/v1/memos/{id}
async fn get_memo(
    path: Path<Uuid>,
    service: Data<MemoService>,
) -> Result<Json<MemoResponseDto>, AppError>

// POST /api/v1/memos
async fn create_memo(
    dto: Json<CreateMemoDto>,
    service: Data<MemoService>,
) -> Result<Json<MemoResponseDto>, AppError>

// PUT /api/v1/memos/{id}
async fn update_memo(
    path: Path<Uuid>,
    dto: Json<UpdateMemoDto>,
    service: Data<MemoService>,
) -> Result<Json<MemoResponseDto>, AppError>

// PATCH /api/v1/memos/{id}
async fn patch_memo(
    path: Path<Uuid>,
    dto: Json<PatchMemoDto>,
    service: Data<MemoService>,
) -> Result<Json<MemoResponseDto>, AppError>

// DELETE /api/v1/memos/{id}
async fn delete_memo(
    path: Path<Uuid>,
    service: Data<MemoService>,
) -> Result<HttpResponse, AppError>

// PATCH /api/v1/memos/{id}/complete
async fn toggle_complete(
    path: Path<Uuid>,
    service: Data<MemoService>,
) -> Result<Json<MemoResponseDto>, AppError>
```

### Deliverables
- ✅ All REST API endpoints working
- ✅ Integration tests pass
- ✅ Proper HTTP status codes
- ✅ JSON responses correct

**Estimated Time:** 6-8 hours

---

## Stage 9: OpenAPI Documentation

**Goal:** Add interactive API documentation

### Tasks
- [ ] Add utoipa dependencies
- [ ] Add ToSchema derive to all DTOs
- [ ] Add #[utoipa::path] to all handlers
- [ ] Create src/docs/openapi.rs
- [ ] Configure OpenAPI spec
- [ ] Configure Swagger UI at /swagger-ui/
- [ ] Add OpenAPI JSON at /api-docs/openapi.json
- [ ] Test Swagger UI
- [ ] Add request/response examples

### Dependencies to Add
```toml
utoipa = { version = "4", features = ["actix_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "7", features = ["actix-web"] }
```

### OpenAPI Configuration
```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_memos,
        get_memo,
        create_memo,
        update_memo,
        patch_memo,
        delete_memo,
        toggle_complete,
    ),
    components(
        schemas(MemoResponseDto, CreateMemoDto, UpdateMemoDto, PatchMemoDto, ErrorResponse)
    ),
    tags(
        (name = "memos", description = "Memo management endpoints")
    )
)]
struct ApiDoc;
```

### Deliverables
- ✅ Swagger UI at /swagger-ui/
- ✅ All endpoints documented
- ✅ Schemas visible
- ✅ Examples accurate

**Estimated Time:** 3-5 hours

---

## Stage 10: Askama Templates Setup

**Goal:** Set up Askama template engine

### Tasks
- [ ] Add Askama dependencies
- [ ] Create templates/ directory structure
- [ ] Create base.html layout with HTMX
- [ ] Create templates/pages/index.html
- [ ] Create templates/pages/error.html
- [ ] Create templates/components/memo_item.html
- [ ] Create templates/components/memo_form.html
- [ ] Create templates/components/memo_list.html
- [ ] Create templates/partials/header.html
- [ ] Create templates/partials/footer.html
- [ ] Test template compilation

### Dependencies to Add
```toml
askama = "0.12"
askama_actix = "0.14"
actix-files = "0.6"
```

### Template Structure
```
templates/
├── base.html
├── pages/
│   ├── index.html
│   └── error.html
├── components/
│   ├── memo_item.html
│   ├── memo_form.html
│   └── memo_list.html
└── partials/
    ├── header.html
    └── footer.html
```

### Base Template (base.html)
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Memos App{% endblock %}</title>
    <link rel="stylesheet" href="/static/css/style.css">
    <script src="/static/js/htmx.min.js"></script>
</head>
<body>
    {% include "partials/header.html" %}

    <main>
        {% block content %}{% endblock %}
    </main>

    {% include "partials/footer.html" %}
</body>
</html>
```

### Deliverables
- ✅ Templates compile at build time
- ✅ Base layout works
- ✅ Template inheritance works
- ✅ HTMX loaded

**Estimated Time:** 3-4 hours

---

## Stage 11: Web Page Handlers (HTML)

**Goal:** Implement server-side rendered pages

### Tasks
- [ ] Create src/web/pages.rs
- [ ] Create src/web/components.rs
- [ ] Implement GET / (home page)
- [ ] Implement GET /memos (memo list component for HTMX)
- [ ] Implement GET /memos/new (create form)
- [ ] Implement POST /memos (handle creation)
- [ ] Implement GET /memos/{id} (single memo)
- [ ] Implement GET /memos/{id}/edit (edit form)
- [ ] Implement PUT /memos/{id} (handle update)
- [ ] Implement DELETE /memos/{id} (handle deletion)
- [ ] Implement POST /memos/{id}/toggle (toggle complete)
- [ ] Add routes to main.rs
- [ ] Test all HTML endpoints

### Template Structs
```rust
use askama::Template;

#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    pub memos: Vec<MemoResponseDto>,
    pub total: u64,
    pub page: u64,
}

#[derive(Template)]
#[template(path = "components/memo_list.html")]
pub struct MemoListTemplate {
    pub memos: Vec<MemoResponseDto>,
}

#[derive(Template)]
#[template(path = "components/memo_item.html")]
pub struct MemoItemTemplate {
    pub memo: MemoResponseDto,
}

#[derive(Template)]
#[template(path = "components/memo_form.html")]
pub struct MemoFormTemplate {
    pub memo: Option<MemoResponseDto>,
    pub action: String,
}
```

### Deliverables
- ✅ Home page renders
- ✅ All HTML endpoints work
- ✅ Forms submit correctly
- ✅ Askama templates compile

**Estimated Time:** 6-8 hours

---

## Stage 12: HTMX Integration

**Goal:** Add dynamic interactions with HTMX

### Tasks
- [ ] Download and add htmx.min.js to static/js/
- [ ] Add HTMX attributes to memo list for filtering
- [ ] Add HTMX attributes for sorting
- [ ] Implement instant toggle complete
- [ ] Implement delete with confirmation
- [ ] Add HTMX for create memo (modal or inline)
- [ ] Add HTMX for edit memo (modal or inline)
- [ ] Implement search with HTMX
- [ ] Add loading indicators
- [ ] Test all HTMX interactions

### HTMX Examples
```html
<!-- Toggle complete -->
<button
    hx-post="/memos/{{ memo.id }}/toggle"
    hx-target="#memo-{{ memo.id }}"
    hx-swap="outerHTML">
    Toggle
</button>

<!-- Delete with confirmation -->
<button
    hx-delete="/memos/{{ memo.id }}"
    hx-confirm="Are you sure?"
    hx-target="#memo-{{ memo.id }}"
    hx-swap="outerHTML swap:1s">
    Delete
</button>

<!-- Filter -->
<select
    hx-get="/memos"
    hx-target="#memo-list"
    hx-trigger="change"
    name="completed">
    <option value="">All</option>
    <option value="true">Completed</option>
    <option value="false">Incomplete</option>
</select>

<!-- Search -->
<input
    type="search"
    name="search"
    hx-get="/memos"
    hx-trigger="keyup changed delay:500ms"
    hx-target="#memo-list"
    placeholder="Search memos...">
```

### Deliverables
- ✅ Filtering works without page reload
- ✅ Sorting works instantly
- ✅ Toggle complete instant
- ✅ Delete with confirmation
- ✅ Search works

**Estimated Time:** 4-6 hours

---

## Stage 13: Static Assets & Styling

**Goal:** Add CSS and improve UI

### Tasks
- [ ] Create static/css/style.css
- [ ] Add basic reset/normalize CSS
- [ ] Style the layout (header, main, footer)
- [ ] Style memo list
- [ ] Style memo items
- [ ] Style forms
- [ ] Add responsive design
- [ ] Add loading indicators
- [ ] Add success/error notifications
- [ ] Configure actix-files for static serving

### CSS Structure
```css
/* Reset & Variables */
:root {
    --primary-color: #3b82f6;
    --danger-color: #ef4444;
    --success-color: #10b981;
    --text-color: #1f2937;
    --bg-color: #f9fafb;
}

/* Layout */
/* Components */
/* Forms */
/* Responsive */
```

### Deliverables
- ✅ Static files served
- ✅ UI is styled
- ✅ Responsive design
- ✅ Loading indicators work

**Estimated Time:** 4-6 hours

---

## Stage 14: Docker & Deployment

**Goal:** Finalize Docker setup

### Tasks
- [ ] Optimize Dockerfile with multi-stage build
- [ ] Add HEALTHCHECK to Dockerfile
- [ ] Update docker-compose.yml with app service
- [ ] Configure networking between services
- [ ] Add volume for database persistence
- [ ] Test full stack with docker-compose up
- [ ] Create production .env example
- [ ] Document deployment process

### Multi-stage Dockerfile
```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY migration ./migration
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/actix-web-template .
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static
EXPOSE 3737
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:3737/health || exit 1
CMD ["./actix-web-template"]
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: memos
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  app:
    build: .
    ports:
      - "3737:3737"
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@postgres:5432/memos
      - RUST_LOG=info
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  postgres_data:
```

### Deliverables
- ✅ docker-compose up works
- ✅ App connects to PostgreSQL
- ✅ Health checks work
- ✅ Data persists

**Estimated Time:** 4-6 hours

---

## Stage 15: Security Enhancements

**Goal:** Add security features

### Tasks
- [ ] Implement rate limiting middleware
- [ ] Add security headers middleware
- [ ] Configure CORS for production
- [ ] Test request size limits
- [ ] Add input sanitization
- [ ] Review SQL injection prevention
- [ ] Add CSRF protection for forms (if needed)
- [ ] Document security configuration

### Security Headers
```rust
fn security_headers() -> Middleware {
    middleware::DefaultHeaders::new()
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("X-Frame-Options", "DENY"))
        .add(("X-XSS-Protection", "1; mode=block"))
        .add(("Strict-Transport-Security", "max-age=31536000"))
}
```

### Rate Limiting
Consider using `actix-governor` or custom middleware

### Deliverables
- ✅ Rate limiting works
- ✅ Security headers present
- ✅ CORS configured
- ✅ Security documented

**Estimated Time:** 4-6 hours

---

## Stage 16: Testing & Quality Assurance

**Goal:** Comprehensive testing coverage

### Tasks
- [ ] Write unit tests for services
- [ ] Write integration tests for REST API
- [ ] Write integration tests for HTML endpoints
- [ ] Write repository tests with test DB
- [ ] Create test fixtures and helpers
- [ ] Measure test coverage (aim >70%)
- [ ] Run clippy and fix warnings
- [ ] Run cargo fmt
- [ ] Document testing strategy

### Test Structure
```
tests/
├── common/
│   ├── mod.rs
│   └── fixtures.rs
├── integration/
│   ├── api_tests.rs
│   └── web_tests.rs
└── repository_tests.rs
```

### Test Commands
```bash
cargo test
cargo test -- --nocaptured
cargo clippy -- -D warnings
cargo fmt --check
cargo tarpaulin --out Html  # Coverage
```

### Deliverables
- ✅ Test coverage >70%
- ✅ All tests pass
- ✅ No clippy warnings
- ✅ Code formatted

**Estimated Time:** 8-12 hours

---

## Stage 17: Performance Optimization

**Goal:** Optimize application performance

### Tasks
- [ ] Add response compression middleware
- [ ] Tune database connection pool
- [ ] Optimize database queries
- [ ] Add request timeouts
- [ ] Add database query timeouts
- [ ] Test performance under load
- [ ] Document performance benchmarks

### Compression
```toml
actix-web-lab = { version = "0.20", features = ["compress"] }
```

### Deliverables
- ✅ Response compression working
- ✅ Connection pool tuned
- ✅ Performance acceptable

**Estimated Time:** 4-6 hours

---

## Stage 18: Documentation & Finalization

**Goal:** Complete all documentation

### Tasks
- [ ] Write comprehensive README.md
- [ ] Document all environment variables
- [ ] Create API usage examples
- [ ] Document deployment procedures
- [ ] Update CLAUDE.md with final architecture
- [ ] Create troubleshooting guide
- [ ] Add LICENSE file
- [ ] Final review and cleanup

### README.md Sections
- Project overview
- Features
- Tech stack
- Getting started
- Development
- Testing
- Deployment
- API documentation
- Contributing
- License

### Deliverables
- ✅ Complete documentation
- ✅ Easy onboarding
- ✅ Clear deployment instructions

**Estimated Time:** 4-6 hours

---

## Optional Stage 19: CI/CD Pipeline

**Goal:** Automated testing and deployment

### Tasks
- [ ] Create GitHub Actions workflow (or GitLab CI)
- [ ] Add automated testing on push
- [ ] Add automated linting (clippy, fmt)
- [ ] Add Docker image building
- [ ] Configure environment secrets
- [ ] Test CI/CD pipeline
- [ ] Document CI/CD process

### GitHub Actions Example
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

### Deliverables
- ✅ CI runs on commits
- ✅ All checks pass
- ✅ Docker images built

**Estimated Time:** 4-6 hours

---

## Optional Stage 20: Observability Stack

**Goal:** Add external observability tools

### Tasks
- [ ] Add Jaeger to docker-compose (commented)
- [ ] Add Prometheus to docker-compose (commented)
- [ ] Add Grafana + Loki (commented)
- [ ] Implement /metrics endpoint
- [ ] Configure OpenTelemetry exporter
- [ ] Create Grafana dashboards
- [ ] Document observability setup

### docker-compose.yml Addition
```yaml
# Uncomment to enable observability

# jaeger:
#   image: jaegertracing/all-in-one:latest
#   ports:
#     - "16686:16686"

# prometheus:
#   image: prom/prometheus:latest
#   ports:
#     - "9090:9090"

# grafana:
#   image: grafana/grafana:latest
#   ports:
#     - "3000:3000"
```

### Deliverables
- ✅ Observability stack available
- ✅ Documentation complete

**Estimated Time:** 6-8 hours

---

## Development Workflow

### Daily Development
1. Pull latest changes: `git pull`
2. Start database: `docker-compose up -d postgres`
3. Run migrations: `sea-orm-cli migrate up`
4. Start app: `cargo run`
5. Make changes
6. Run tests: `cargo test`
7. Format: `cargo fmt`
8. Lint: `cargo clippy`
9. Commit changes

### Before Committing
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

### Full Stack Development
```bash
docker-compose up --build
```

---

## Success Criteria

### Stage Completion
Each stage is complete when:
- ✅ All tasks are completed
- ✅ All tests pass
- ✅ Code is formatted and linted
- ✅ Documentation is updated
- ✅ **No empty files or directories exist**
- ✅ **No unused dependencies in Cargo.toml**
- ✅ Changes are committed

### Project Completion
The project is complete when:
- ✅ All required stages (0-18) are complete
- ✅ Application runs in Docker
- ✅ All API endpoints work
- ✅ Web UI is functional with HTMX
- ✅ Test coverage >70%
- ✅ Documentation is complete
- ✅ Security features implemented
- ✅ Performance is acceptable

---

## Timeline Estimate

| Stage | Name | Time (hours) |
|-------|------|--------------|
| 0 | Project Setup | 2-4 |
| 1 | Core Application | 4-6 |
| 2 | Database Setup | 4-6 |
| 3 | Error Handling | 4-6 |
| 4 | Health Checks | 2-4 |
| 5 | DTOs & Validation | 3-5 |
| 6 | Repository Layer | 6-8 |
| 7 | Service Layer | 6-8 |
| 8 | REST API | 6-8 |
| 9 | OpenAPI Docs | 3-5 |
| 10 | Askama Templates | 3-4 |
| 11 | Web Handlers | 6-8 |
| 12 | HTMX Integration | 4-6 |
| 13 | Static Assets | 4-6 |
| 14 | Docker | 4-6 |
| 15 | Security | 4-6 |
| 16 | Testing | 8-12 |
| 17 | Performance | 4-6 |
| 18 | Documentation | 4-6 |

**Total Core Time:** 76-120 hours (2-3 weeks full-time)

**Optional Stages:**
- Stage 19 (CI/CD): 4-6 hours
- Stage 20 (Observability): 6-8 hours

---

## Notes

- Each stage builds upon previous stages
- Testing should be continuous, not just in Stage 16
- Documentation should be updated incrementally
- Security considerations throughout development
- Performance testing early to identify bottlenecks
- Code reviews recommended after each major stage
- Use feature branches for larger changes
- Keep commits small and focused
- Update CLAUDE.md as architecture evolves

---

## Key Technologies Summary

- **Backend:** Actix Web (Rust)
- **Database:** PostgreSQL 16 + SeaORM
- **Templates:** Askama (compile-time, type-safe)
- **Frontend:** HTMX (progressive enhancement)
- **API Docs:** OpenAPI + Swagger UI
- **Logging:** tracing + tracing-subscriber
- **Testing:** cargo test + integration tests
- **Deployment:** Docker + Docker Compose
- **Security:** CORS, rate limiting, security headers