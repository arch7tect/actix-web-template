# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a production-ready Actix Web application for managing memos. Built with Rust, featuring a layered architecture, comprehensive testing, security features, and performance optimizations.

**Status**: Stage 18 Complete - Full-featured memo management application with REST API, web UI, documentation, tests, and Docker deployment.

## Tech Stack

- **Framework**: Actix Web 4 (async web framework)
- **Database**: PostgreSQL 16 + SeaORM 1.0 (ORM)
- **Templates**: Askama (compile-time templates)
- **API Docs**: utoipa + Swagger UI
- **Validation**: validator crate
- **Logging**: tracing + tracing-subscriber
- **Runtime**: Tokio

## Build and Development Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Run with release optimizations
cargo build --release
cargo run --release

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run linter with warnings as errors
cargo clippy -- -D warnings

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test file
cargo test --test api_tests

# Run tests serially
cargo test -- --test-threads=1
```

### Database Commands

```bash
# Run migrations
cd migration && cargo run

# Or use SeaORM CLI
sea-orm-cli migrate up
sea-orm-cli migrate down

# Generate entities from database
sea-orm-cli generate entity -o src/entities

# Create new migration
sea-orm-cli migrate generate migration_name
```

### Docker Commands

```bash
# Start all services
docker-compose up --build

# Start in background
docker-compose up -d --build

# View logs
docker-compose logs -f app

# Stop services
docker-compose down

# Remove volumes (destroys data)
docker-compose down -v
```

## Project Structure

```
actix-web-template/
├── src/
│   ├── config/              # Application configuration
│   │   ├── mod.rs
│   │   └── settings.rs      # Settings struct, env loading
│   ├── docs/                # OpenAPI documentation
│   │   ├── mod.rs
│   │   └── openapi.rs       # OpenAPI spec, Swagger config
│   ├── dto/                 # Data Transfer Objects
│   │   ├── mod.rs
│   │   └── memo_dto.rs      # CreateMemoDto, UpdateMemoDto, etc.
│   ├── entities/            # SeaORM database models
│   │   ├── mod.rs
│   │   ├── prelude.rs
│   │   └── memos.rs         # Generated memo entity
│   ├── error/               # Error handling
│   │   ├── mod.rs
│   │   └── app_error.rs     # AppError enum, ResponseError impl
│   ├── handlers/            # HTTP request handlers
│   │   ├── mod.rs
│   │   ├── health.rs        # Health check endpoints
│   │   ├── memos.rs         # REST API handlers
│   │   ├── web.rs           # HTML page handlers
│   │   ├── test_*.rs        # Handler unit tests
│   ├── middleware/          # Custom middleware
│   │   ├── mod.rs
│   │   ├── rate_limit.rs    # Rate limiting (actix-governor)
│   │   └── security_headers.rs  # Security headers
│   ├── repository/          # Database access layer
│   │   ├── mod.rs
│   │   └── memo_repository.rs   # CRUD operations
│   ├── services/            # Business logic layer
│   │   ├── mod.rs
│   │   └── memo_service.rs  # Business logic, DTO conversions
│   ├── utils/               # Utility functions
│   │   ├── mod.rs
│   │   ├── sanitize.rs      # HTML sanitization (XSS prevention)
│   │   └── tracing.rs       # Tracing setup
│   ├── state.rs             # Application state (DB connection)
│   ├── lib.rs               # Library root
│   └── main.rs              # Application entry point
├── migration/               # Database migrations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   └── m20250109_*.rs   # Migration files
│   └── Cargo.toml
├── templates/               # Askama HTML templates
│   ├── base.html            # Base layout
│   ├── pages/
│   │   ├── index.html       # Home page
│   │   └── error.html       # Error page
│   ├── components/
│   │   ├── memo_form.html   # Create/edit form
│   │   ├── memo_item.html   # Single memo display
│   │   └── memo_list.html   # Memo list
│   └── partials/
│       ├── header.html
│       └── footer.html
├── static/                  # Static assets
│   └── css/
│       └── style.css        # Application styles
├── tests/                   # Integration tests
│   ├── common/
│   │   ├── mod.rs
│   │   └── fixtures.rs      # Test helpers
│   ├── api_tests.rs         # REST API integration tests
│   ├── repository_tests.rs  # Repository layer tests
│   ├── service_tests.rs     # Service layer unit tests
│   └── web_tests.rs         # HTML endpoint tests
├── .env.example             # Example environment variables
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── docker-compose.yml       # Docker Compose configuration
├── Dockerfile               # Multi-stage Docker build
├── .dockerignore
├── CLAUDE.md                # This file
├── LICENSE                  # MIT License
├── MIGRATIONS.md            # Migration history
├── PERFORMANCE.md           # Performance docs
├── project_plan.md          # Implementation plan
├── README.md                # Main documentation
├── requirements.md          # Original requirements
├── TESTING.md               # Testing documentation
└── TROUBLESHOOTING.md       # Troubleshooting guide
```

## Architecture

### Layered Architecture

The application follows a strict layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────┐
│          HTTP Requests (REST/HTML)          │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│  HANDLERS LAYER (src/handlers/)             │
│  - HTTP request/response handling           │
│  - Input validation (validator)             │
│  - Route definitions                        │
│  - Error responses                          │
│  Files: health.rs, memos.rs, web.rs         │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│  SERVICE LAYER (src/services/)              │
│  - Business logic                           │
│  - DTO ↔ Entity conversion                  │
│  - Orchestration                            │
│  - Input sanitization                       │
│  Files: memo_service.rs                     │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│  REPOSITORY LAYER (src/repository/)         │
│  - Database operations (CRUD)               │
│  - Query building                           │
│  - Pagination, filtering, sorting           │
│  Files: memo_repository.rs                  │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│  ENTITY LAYER (src/entities/)               │
│  - SeaORM models                            │
│  - Database schema representation           │
│  Files: memos.rs (generated)                │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│          PostgreSQL Database                │
└─────────────────────────────────────────────┘
```

### Cross-Cutting Concerns

```
┌─────────────────────────────────────────────┐
│  MIDDLEWARE (src/middleware/)               │
│  - Rate limiting (actix-governor)           │
│  - Security headers                         │
│  - CORS (actix-cors)                        │
│  - Compression (gzip, brotli)               │
│  - Request logging (tracing-actix-web)      │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│  ERROR HANDLING (src/error/)                │
│  - AppError enum                            │
│  - ResponseError trait impl                 │
│  - Consistent error responses               │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│  CONFIGURATION (src/config/)                │
│  - Environment variables                    │
│  - Settings struct                          │
│  - Validation                               │
└─────────────────────────────────────────────┘
```

### Data Flow

#### REST API Request Flow

1. **HTTP Request** → Actix Web server
2. **Middleware** → Rate limiting, security headers, CORS, logging
3. **Handler** → Parse request, validate DTO
4. **Service** → Business logic, sanitization, DTO conversion
5. **Repository** → Database query (SeaORM)
6. **Database** → PostgreSQL
7. **Response** ← JSON through same layers
8. **Compression** → Gzip/Brotli if applicable
9. **HTTP Response** ← Client receives data

#### HTML Request Flow

1. **HTTP Request** → Actix Web server
2. **Middleware** → Same as REST API
3. **Web Handler** → Fetch data via service
4. **Service/Repository** → Query database
5. **Template** → Askama renders HTML
6. **Response** → HTML with vanilla JavaScript
7. **Client** → Browser renders page
8. **JavaScript** → Can make REST API calls for dynamic updates

## Key Components

### 1. Application State (`src/state.rs`)

```rust
pub struct AppState {
    pub db: DatabaseConnection,
}
```

- Holds database connection pool
- Shared across all handlers via `web::Data<AppState>`

### 2. Error Handling (`src/error/app_error.rs`)

```rust
pub enum AppError {
    Database(DbErr),
    NotFound(String),
    Validation(String),
    Internal(String),
}
```

- Centralized error handling
- Implements `ResponseError` for Actix Web
- Returns consistent JSON error responses

### 3. DTOs (`src/dto/memo_dto.rs`)

- **CreateMemoDto**: For creating new memos
- **UpdateMemoDto**: For full updates (PUT)
- **PatchMemoDto**: For partial updates (PATCH)
- **MemoResponseDto**: For responses
- **PaginationParams**: For list queries
- **PaginatedResponse<T>**: Generic paginated response

All DTOs use `validator` for input validation.

### 4. Service Layer (`src/services/memo_service.rs`)

- Business logic orchestration
- DTO to Entity conversion
- Input sanitization (XSS prevention)
- Calls repository for database operations

### 5. Repository Layer (`src/repository/memo_repository.rs`)

- CRUD operations
- Pagination with `paginate()` helper
- Filtering by `completed` status
- Sorting by various fields
- Uses SeaORM query builder

### 6. Middleware

#### Rate Limiting (`src/middleware/rate_limit.rs`)

- IP-based rate limiting
- 100 requests per minute (default)
- Uses `actix-governor`

#### Security Headers (`src/middleware/security_headers.rs`)

- X-Content-Type-Options: nosniff
- X-Frame-Options: DENY
- X-XSS-Protection: 1; mode=block
- Strict-Transport-Security (production)

### 7. API Documentation (`src/docs/openapi.rs`)

- OpenAPI 3.0 specification
- Swagger UI at `/swagger-ui/`
- Auto-generated from `utoipa` annotations
- All endpoints documented with examples

## Database Schema

### Memos Table

```sql
CREATE TABLE memos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(200) NOT NULL,
    description TEXT,
    date_to TIMESTAMP WITH TIME ZONE NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_memos_completed ON memos(completed);
CREATE INDEX idx_memos_date_to ON memos(date_to);
CREATE INDEX idx_memos_created_at ON memos(created_at);
```

## API Endpoints

### REST API (`/api/v1/`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/memos` | List memos (paginated, filtered, sorted) |
| GET | `/api/v1/memos/{id}` | Get single memo |
| POST | `/api/v1/memos` | Create new memo |
| PUT | `/api/v1/memos/{id}` | Full update |
| PATCH | `/api/v1/memos/{id}` | Partial update |
| DELETE | `/api/v1/memos/{id}` | Delete memo |
| PATCH | `/api/v1/memos/{id}/complete` | Toggle completion |

### Web UI

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Home page (memo list) |
| GET | `/memos` | Memo list component |
| GET | `/memos/new` | Create form |
| POST | `/memos` | Handle creation |
| GET | `/memos/{id}` | Single memo |
| GET | `/memos/{id}/edit` | Edit form |
| PUT | `/memos/{id}` | Handle update |
| DELETE | `/memos/{id}` | Handle deletion |
| POST | `/memos/{id}/toggle` | Toggle complete |

### Health & Monitoring

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check with DB status |
| GET | `/ready` | Kubernetes readiness probe |

### Documentation

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/swagger-ui/` | Interactive API documentation |
| GET | `/api-docs/openapi.json` | OpenAPI spec (JSON) |

## Testing Strategy

### Test Coverage

- **Unit Tests**: Embedded in handler files (`test_*.rs`)
- **Service Tests**: `tests/service_tests.rs`
- **Repository Tests**: `tests/repository_tests.rs`
- **API Integration Tests**: `tests/api_tests.rs`
- **Web Integration Tests**: `tests/web_tests.rs`

### Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test api_tests

# With output
cargo test -- --nocapture

# Serial execution (for DB tests)
cargo test -- --test-threads=1
```

See `TESTING.md` for detailed testing documentation.

## Security Features

1. **Rate Limiting**: IP-based, 100 req/min default
2. **Security Headers**: XSS, frame, content-type protection
3. **Input Sanitization**: HTML sanitization with `ammonia`
4. **CORS**: Configurable origins
5. **Request Size Limits**: 256KB default
6. **SQL Injection Prevention**: SeaORM parameterized queries
7. **Error Message Safety**: No internal details leaked

## Performance Optimizations

1. **Response Compression**: Gzip + Brotli
2. **Connection Pooling**: Database connection pool
3. **Async/Await**: Non-blocking I/O throughout
4. **Efficient Queries**: Indexed columns, pagination
5. **Compile-time Templates**: Askama (no runtime parsing)
6. **Static Asset Serving**: Efficient file serving with actix-files

See `PERFORMANCE.md` for benchmarks and tuning.

## Coding Rules

### General Guidelines

- **No emojis** in code, logs, error messages, or API responses (HTML templates are OK)
- **Prefer tracing over comments** - use `tracing::debug!()`, `tracing::info!()`, etc.
- **Comments explain "why", not "what"** - code should be self-documenting
- **Naming conventions**:
  - Functions/variables: `snake_case`
  - Types/structs: `PascalCase`
  - Constants: `SCREAMING_SNAKE_CASE`

### Error Handling

- Use `AppError` enum for all application errors
- Never use `unwrap()` or `expect()` in production code
- Log errors with appropriate tracing level
- Return safe error messages to clients

### Database Operations

- Always use SeaORM query builder (no raw SQL)
- Use transactions for multi-step operations
- Add tracing instrumentation to repository methods
- Handle `DbErr` appropriately

### Validation

- Validate all input with `validator` crate
- Sanitize user-generated content with `ammonia`
- Validate at DTO level, not in handlers

### Testing

- Write tests for all new features
- Test happy path and error cases
- Use test fixtures from `tests/common/fixtures.rs`
- Mock external dependencies

## Git Workflow

- **Never commit directly to master** - always use stage branches
- **Branching strategy**: `stage-N-short-name` for each implementation stage
- **Workflow**: Create branch → Complete work → Test (cargo build/run) → Commit → Tag → Merge to master
- **Tags**: `stage-N-complete` marks the completion of each stage

### Commit Messages

- Use descriptive, imperative commit messages
- Reference stage number if applicable
- Examples:
  - "Complete Stage 8: REST API Handlers"
  - "Fix validation error in CreateMemoDto"
  - "Add rate limiting middleware"

## Configuration

All configuration is done via environment variables. See `.env.example` for all options.

### Required Variables

- `DATABASE_URL`: PostgreSQL connection string
- `SERVER_HOST`: Bind address (default: 127.0.0.1)
- `SERVER_PORT`: Port (default: 3737)

### Optional Variables

- `RUST_LOG`: Logging level
- `LOG_FORMAT`: pretty or json
- `DATABASE_MAX_CONNECTIONS`: Connection pool size
- `CORS_ALLOWED_ORIGINS`: Comma-separated origins
- `MAX_REQUEST_SIZE`: Bytes
- `ENABLE_SWAGGER`: true/false

## Deployment

### Docker

```bash
# Build and run
docker-compose up --build

# Production deployment
docker build -t actix-web-template:latest .
docker run -p 3737:3737 --env-file .env actix-web-template:latest
```

### Health Checks

- Endpoint: `/health`
- Returns: `{"status": "healthy", "database": "connected", ...}`
- Use for Docker HEALTHCHECK and Kubernetes liveness probes

## Troubleshooting

See `TROUBLESHOOTING.md` for common issues and solutions.

### Quick Checks

```bash
# Check logs
RUST_LOG=debug cargo run

# Verify database connection
psql $DATABASE_URL

# Check port usage
lsof -i:3737

# Verify Docker services
docker-compose ps
```

## Documentation Files

- **README.md**: Main documentation, getting started, API usage
- **CLAUDE.md**: This file - architecture and development guide
- **TESTING.md**: Testing strategy and guidelines
- **PERFORMANCE.md**: Performance benchmarks and tuning
- **MIGRATIONS.md**: Database migration history
- **TROUBLESHOOTING.md**: Common issues and solutions
- **project_plan.md**: Implementation plan (18 stages)
- **requirements.md**: Original requirements

## Development Tips

### Adding New Features

1. Create stage branch: `git checkout -b stage-N-feature-name`
2. Update DTOs if needed (`src/dto/`)
3. Add repository methods (`src/repository/`)
4. Implement service logic (`src/services/`)
5. Create handlers (`src/handlers/`)
6. Add routes to `main.rs`
7. Write tests (`tests/`)
8. Update documentation
9. Run quality checks: `cargo fmt && cargo clippy && cargo test`
10. Commit and tag: `git commit -m "..." && git tag stage-N-complete`

### Debugging

```bash
# Maximum logging
RUST_LOG=trace cargo run

# With backtrace
RUST_BACKTRACE=1 cargo run

# Test specific function
cargo test test_name -- --nocapture

# Check for unused dependencies
cargo install cargo-udeps
cargo +nightly udeps
```

## Stage Completion Status

- [x] Stage 0: Project Setup & Infrastructure
- [x] Stage 1: Core Application & Configuration
- [x] Stage 2: Database Setup & ORM Integration
- [x] Stage 3: Error Handling & Middleware
- [x] Stage 4: Health Check & Monitoring
- [x] Stage 5: DTOs & Validation
- [x] Stage 6: Repository Layer
- [x] Stage 7: Service Layer
- [x] Stage 8: REST API Handlers
- [x] Stage 9: OpenAPI Documentation
- [x] Stage 10: Askama Templates Setup
- [x] Stage 11: Web Page Handlers (HTML)
- [x] Stage 12: Vanilla JavaScript Integration (skipped - integrated in Stage 11)
- [x] Stage 13: Static Assets & Styling
- [x] Stage 14: Docker & Deployment
- [x] Stage 15: Security Enhancements
- [x] Stage 16: Testing & Quality Assurance
- [x] Stage 17: Performance Optimization
- [x] Stage 18: Documentation & Finalization

## Next Steps (Optional Stages)

- [ ] Stage 19: CI/CD Pipeline (GitHub Actions)
- [ ] Stage 20: Observability Stack (Jaeger, Prometheus, Grafana)

## Resources

- [Actix Web Documentation](https://actix.rs/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Askama Documentation](https://djc.github.io/askama/)
- [Tokio Documentation](https://tokio.rs/)
- [Rust Book](https://doc.rust-lang.org/book/)