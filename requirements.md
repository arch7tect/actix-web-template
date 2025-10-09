# Requirements Document

## Project Overview
A simple memos web application with REST API server built using Actix Web framework in Rust.

## Functional Requirements

### REST API Endpoints

#### Health & Monitoring
- `GET /health` - Health check with database connectivity
  - Response: `200 OK` with `{"status": "healthy", "database": "connected", "version": "0.1.0"}`
- `GET /ready` - Readiness check (for Kubernetes)
- `GET /metrics` - Prometheus metrics (optional, if enabled)

#### Memos Management
- `GET /api/v1/memos` - List all memos (with pagination, filtering, sorting)
  - Query params: `limit`, `offset`, `completed`, `sort_by`, `order`
- `GET /api/v1/memos/{id}` - Get a specific memo by ID
- `POST /api/v1/memos` - Create a new memo
- `PUT /api/v1/memos/{id}` - Update an existing memo (full replacement)
- `PATCH /api/v1/memos/{id}` - Partially update a memo
- `DELETE /api/v1/memos/{id}` - Delete a memo
- `PATCH /api/v1/memos/{id}/complete` - Toggle memo completion status

#### Data Models
**Memo**
```json
{
  "id": "uuid",
  "title": "string",
  "description": "string",
  "date_to": "timestamp",
  "completed": "boolean",
  "created_at": "timestamp",
  "updated_at": "timestamp"
}
```

### Web Interface
- **Server-side rendered** HTML using Askama templates (compile-time type-safe)
- **HTMX** for dynamic interactions without full page reloads
- **Progressive enhancement** - works without JavaScript, better with it

#### Pages
- `GET /` - Home page with memo list
- `GET /memos/new` - Create memo form (can be loaded in modal via HTMX)
- `GET /memos/{id}/edit` - Edit memo form (can be loaded in modal via HTMX)

#### Features
- Display paginated list of memos
- Create new memo (inline form or modal)
- Edit memos (inline or modal)
- Delete memos with confirmation
- Toggle completion status (instant update via HTMX)
- Filter by completion status (instant update via HTMX)
- Sort by date, title, etc. (instant update via HTMX)
- Search memos (instant search via HTMX)

#### Template Structure (Askama)
```
templates/
├── base.html              # Base layout with HTMX setup
├── pages/
│   ├── index.html         # Home page with memo list
│   └── error.html         # Error page
├── components/
│   ├── memo_item.html     # Single memo component
│   ├── memo_form.html     # Memo form (create/edit)
│   └── memo_list.html     # Memo list (for HTMX swaps)
└── partials/
    ├── header.html
    └── footer.html
```

#### HTMX Integration
- **hx-get**: Load memo list, filters, search results
- **hx-post**: Create/update/delete memos
- **hx-swap**: Replace parts of page without reload
- **hx-target**: Specify where to swap content
- **hx-trigger**: Define when to trigger requests
- Out-of-band swaps for notifications/toasts

### API Documentation
- OpenAPI 3.0 specification generated via `utoipa`
- Swagger UI available at `/swagger-ui/`
- Interactive API documentation and testing
- Automatic schema generation from Rust types
- Request/response examples
- Authentication documentation (if applicable)

## Technical Requirements

### Core Dependencies
- `actix-web` - Web framework
- `actix-cors` - CORS middleware
- `actix-files` - Static file serving
- `askama` - Compile-time template engine (Jinja2-like)
- `askama_actix` - Askama integration for Actix Web
- `serde` - Serialization/deserialization
- `serde_json` - JSON support
- `uuid` - Unique identifier generation
- `chrono` - Date/time handling
- `tokio` - Async runtime
- `sea-orm` - Async ORM with migration support
- `dotenv` - Environment variable management
- `thiserror` - Custom error types
- `anyhow` - Error handling
- `tracing` - Structured logging and instrumentation
- `tracing-subscriber` - Log output formatting and filtering
- `tracing-actix-web` - Actix Web tracing integration
- `validator` - Input validation
- `utoipa` - OpenAPI documentation generation
- `utoipa-swagger-ui` - Swagger UI integration for Actix Web

### Data Storage
- PostgreSQL database (version 16)
- SeaORM for database operations and migrations
- Connection pool management
- Database migrations using SeaORM CLI

### Configuration
- `SERVER_HOST`: Server host (default: `127.0.0.1`)
- `SERVER_PORT`: Server port (default: `3737`)
- `RUST_LOG`: Log level (default: `info`)
- `LOG_FORMAT`: Log format - `pretty`, `json`, `compact` (default: `pretty`)
- `DATABASE_URL`: PostgreSQL connection string (required)
- `DATABASE_MAX_CONNECTIONS`: Connection pool size (default: `10`)
- `DATABASE_CONNECT_TIMEOUT`: Connection timeout in seconds (default: `30`)
- `CORS_ALLOWED_ORIGINS`: Comma-separated allowed origins (default: `*` in dev, must be set in prod)
- `MAX_REQUEST_SIZE`: Max request body size in bytes (default: `262144` - 256KB)
- `ENABLE_SWAGGER`: Enable Swagger UI (default: `true` in dev, `false` in prod)
- `APP_ENV`: Environment - `development`, `staging`, `production` (default: `development`)
- Environment file: `.env` for local development
- Structured configuration module with validation

### Docker Setup
- `docker-compose.yml` for orchestrating services
- Core Services (required):
  - **postgres** - PostgreSQL 16 database with health checks
  - **app** - Actix Web application (multi-stage Dockerfile)
- Optional Observability Services (commented out by default):
  - **jaeger** - Distributed tracing UI (port 16686)
  - **grafana** - Metrics and logs visualization (port 3000)
  - **loki** - Log aggregation backend
  - **prometheus** - Metrics storage (port 9090)
- Volume mounts for database persistence
- Network configuration for service communication
- Environment variable management via `.env` file

### Error Handling
- Custom error types using `thiserror`
- Centralized error handling with `actix-web` error trait
- Proper HTTP status codes for all responses
- JSON error responses with meaningful messages
- Input validation for all endpoints using `validator`
- Database error mapping to application errors

### Logging and Observability
- Structured logging using `tracing` framework
- Request/response logging via `tracing-actix-web` middleware
- Application event logging with structured context (spans and events)
- Request IDs for correlation across distributed traces
- Different log levels per module (configurable via `RUST_LOG` env var)
- JSON formatted logs for production (optional)
- Performance instrumentation with tracing spans
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- Instrumentation of:
  - All HTTP handlers
  - Database queries (via repository layer)
  - Business logic operations (service layer)
  - External service calls (if any)

### Optional: External Observability Stack
For production deployments, consider adding:
- **OpenTelemetry** (`tracing-opentelemetry`) - Export traces to external systems
- **Jaeger** - Distributed tracing UI (Docker service)
- **Grafana + Loki** - Log aggregation and visualization (Docker services)
- **Prometheus** - Metrics collection (with `actix-web-prom`)

**Note:** For this initial version, we'll use console/file-based logging. External observability tools can be added later as optional Docker Compose services.

### Security
- **CORS Configuration**: Configurable CORS policies (`actix-cors`)
- **Rate Limiting**: Request throttling per IP/endpoint (consider `actix-governor` or custom middleware)
- **Request Size Limits**: Max payload size enforcement
- **SQL Injection Prevention**: Automatic via SeaORM parameterized queries
- **Input Sanitization**: Validation and sanitization of all inputs
- **Security Headers**: Add security headers (X-Content-Type-Options, X-Frame-Options, etc.)
- **HTTPS/TLS**: Support for TLS in production
- **Environment-based secrets**: Never commit secrets, use environment variables

### Authentication & Authorization (Future Enhancement)
While not in v1, design should support:
- JWT-based authentication
- Role-based access control (RBAC)
- API key authentication for external services
- Session management

### Performance & Scalability
- **Connection Pooling**: Database connection pool tuning
- **Response Compression**: Gzip/Brotli compression (`actix-web` middleware)
- **Caching Strategy**: Consider Redis for caching (future enhancement)
- **Database Indexes**: Proper indexing on frequently queried fields
- **Pagination**: All list endpoints should support pagination
- **Async I/O**: Non-blocking operations throughout

### Reliability & Resilience
- **Health Checks**: `/health` endpoint with database connectivity check
- **Graceful Shutdown**: Proper cleanup of resources on shutdown
- **Database Migrations**: Versioned, rollback-capable migrations
- **Retry Logic**: For transient failures (database, external services)
- **Circuit Breakers**: For external service calls (future enhancement)
- **Timeouts**: Request and database query timeouts

### Monitoring & Operations
- **Metrics Endpoint**: `/metrics` for Prometheus scraping (optional)
- **Request ID Tracking**: UUID for each request, propagated through logs
- **Structured Logging**: JSON logs for production, pretty logs for development
- **Log Rotation**: When logging to files (via external tools)
- **Application Versioning**: Version info in health endpoint
- **Build Info**: Include git commit, build date in binary

### Data Management
- **Soft Deletes**: Consider soft delete pattern (add `deleted_at` field)
- **Audit Logging**: Track who changed what and when (future enhancement)
- **Data Validation**: Comprehensive validation at all layers
- **Database Transactions**: Proper transaction management for multi-step operations
- **Data Backup**: Database backup strategy (external to app)
- **Data Retention**: Consider data retention policies

### API Design
- **Versioning**: API versioning strategy (e.g., `/api/v1/memos`)
- **Pagination**: Limit/offset or cursor-based pagination
- **Filtering**: Support query parameters for filtering lists
- **Sorting**: Support sorting by multiple fields
- **Partial Updates**: PATCH should support partial updates
- **Idempotency**: POST operations should be idempotent where possible
- **ETags**: For cache validation (future enhancement)

### Configuration Management
- **Environment Profiles**: dev, staging, production configurations
- **Feature Flags**: Toggle features without code changes (future enhancement)
- **Configuration Validation**: Validate all config on startup
- **Secrets Management**: Use environment variables, never hardcode

### Testing Strategy
- **Unit Tests**: Business logic in services
- **Integration Tests**: API endpoints with test database
- **Repository Tests**: Database operations
- **Contract Tests**: API contract validation
- **Load Testing**: Performance benchmarks (future)
- **Test Coverage**: Aim for >70% coverage

### DevOps & CI/CD
- **Docker Multi-stage Build**: Optimize image size
- **Health Checks**: Docker HEALTHCHECK directive
- **CI/CD Pipeline**: Automated testing and building (GitHub Actions, GitLab CI)
- **Database Migrations in CI**: Run migrations as part of deployment
- **Rolling Deployments**: Zero-downtime deployment strategy
- **Environment Variables**: Proper secret management in CI/CD

### Best Practices
- **Layered Architecture**: Handlers → Services → Repository → Database
- **Dependency Injection**: Pass dependencies through app state
- **DTOs**: Separate API models from database entities
- **Validation**: Input validation at handler level
- **Error Propagation**: Use Result types throughout
- **Testing**: Unit tests, integration tests, and API tests
- **Code Organization**: Feature-based module structure
- **Type Safety**: Leverage Rust's type system for correctness
- **Compile-time Template Safety**: Askama templates type-checked at compile time
- **Progressive Enhancement**: HTMX works without JavaScript, enhances with it
- **API Documentation**: Auto-generated OpenAPI specs with utoipa macros
- **12-Factor App**: Follow 12-factor app principles

## Project Structure
```
src/
├── main.rs                  # Application entry point
├── lib.rs                   # Library root for better testing
├── config/                  # Configuration management
│   ├── mod.rs
│   └── settings.rs
├── handlers/                # HTTP request handlers
│   ├── mod.rs
│   ├── health.rs
│   └── memos.rs
├── dto/                     # Data Transfer Objects (with utoipa schemas)
│   ├── mod.rs
│   ├── memo_dto.rs
│   └── error_response.rs
├── docs/                    # OpenAPI documentation
│   ├── mod.rs
│   └── openapi.rs
├── services/                # Business logic layer
│   ├── mod.rs
│   └── memo_service.rs
├── repository/              # Database access layer
│   ├── mod.rs
│   └── memo_repository.rs
├── entities/                # SeaORM entities (auto-generated)
│   ├── mod.rs
│   ├── memo.rs
│   └── prelude.rs
├── middleware/              # Custom middleware
│   ├── mod.rs
│   └── request_id.rs
├── error/                   # Error types
│   ├── mod.rs
│   └── app_error.rs
├── utils/                   # Utility functions
│   └── mod.rs
└── web/                     # Web page handlers (HTML via Askama)
    ├── mod.rs
    ├── pages.rs             # Full page handlers
    └── components.rs        # HTMX component handlers

templates/                   # Askama templates (compile-time checked)
├── base.html                # Base layout with HTMX
├── pages/
│   ├── index.html           # Home page
│   └── error.html           # Error page
├── components/
│   ├── memo_item.html       # Single memo (for HTMX swaps)
│   ├── memo_form.html       # Create/edit form
│   └── memo_list.html       # List of memos (for filtering/sorting)
└── partials/
    ├── header.html
    └── footer.html

static/                      # Static assets
├── css/
│   └── style.css            # Custom styles
├── js/
│   └── htmx.min.js          # HTMX library (or CDN)
└── images/

migration/                   # SeaORM migrations
├── src/
│   ├── lib.rs
│   ├── m20240101_000001_create_memos_table.rs
│   └── mod.rs

docker-compose.yml           # Docker orchestration
Dockerfile                   # Multi-stage build (includes template compilation)
.env.example                 # Example environment variables
.dockerignore               # Docker ignore patterns
README.md                    # Project documentation
```

## Database Schema

### memos table
```sql
CREATE TABLE memos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(200) NOT NULL,
    description TEXT,
    date_to TIMESTAMPTZ NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_memos_date_to ON memos(date_to);
CREATE INDEX idx_memos_completed ON memos(completed);
CREATE INDEX idx_memos_created_at ON memos(created_at);
```

## Validation Rules
- **title**: Required, 1-200 characters
- **description**: Optional, max 1000 characters
- **date_to**: Required, must be valid ISO 8601 timestamp
- **completed**: Boolean, defaults to false

## Testing Requirements
- Unit tests for business logic (services)
- Integration tests for API endpoints (handlers)
- Repository tests with test database
- Mocked dependencies for isolated testing
- Test fixtures and helpers
- CI/CD pipeline integration

## Development Commands
```bash
# Setup database
docker-compose up -d postgres

# Run migrations
sea-orm-cli migrate up

# Generate entities from database
sea-orm-cli generate entity -o src/entities

# Run application locally
cargo run

# Run with Docker Compose (all services)
docker-compose up --build

# Run tests
cargo test

# Run tests with output
cargo test -- --nocaptureed

# Format and lint
cargo fmt
cargo clippy -- -D warnings

# Access API documentation (after running the server)
# Swagger UI: http://127.0.0.1:3737/swagger-ui/
# OpenAPI JSON: http://127.0.0.1:3737/api-docs/openapi.json
```

## Tracing Implementation

All handlers, services, and repository methods should be instrumented with tracing:

**Handlers:**
```rust
use tracing::instrument;

#[instrument(skip(service), fields(request_id))]
async fn get_memos(service: Data<MemoService>) -> Result<Json<Vec<MemoDto>>, AppError> {
    tracing::info!("Fetching all memos");
    // implementation...
}
```

**Services:**
```rust
#[instrument(skip(self))]
pub async fn create_memo(&self, dto: CreateMemoDto) -> Result<MemoDto, AppError> {
    tracing::debug!(?dto, "Creating new memo");
    // implementation...
}
```

**Repository:**
```rust
#[instrument(skip(db))]
pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<memo::Model>, DbErr> {
    tracing::trace!("Querying database for all memos");
    // implementation...
}
```

## API Documentation Implementation

All DTOs and handlers should be annotated with `utoipa` macros:

**DTOs:**
```rust
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MemoDto {
    // fields...
}
```

**Handlers:**
```rust
use utoipa::path;

#[utoipa::path(
    get,
    path = "/api/memos",
    responses(
        (status = 200, description = "List all memos", body = Vec<MemoDto>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn get_memos() -> Result<Json<Vec<MemoDto>>, AppError> {
    // implementation...
}
```