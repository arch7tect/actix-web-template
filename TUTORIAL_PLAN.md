# Tutorial Plan: Building a Production-Ready Actix Web Application

## Overview

This tutorial teaches you to build a complete, production-ready web application using Rust and Actix Web. You'll create a memo management system with a REST API, server-side rendered web UI, PostgreSQL database, comprehensive testing, security features, and full Docker deployment.

## Target Audience

- **Intermediate Rust developers** who understand ownership, traits, and async/await
- **Backend developers** wanting to learn production-ready Rust web development
- **Software engineers** interested in building scalable, performant web services

## Prerequisites

### Required Knowledge
- Rust fundamentals (ownership, borrowing, traits, lifetimes)
- Basic async/await understanding
- HTTP and REST API concepts
- Basic SQL and relational database concepts
- Command line proficiency

### Required Software
- Rust 1.75+ (via rustup)
- PostgreSQL 16 or Docker
- Git
- A code editor (VS Code, RustRover, or similar)

## Learning Outcomes

By completing this tutorial, you will:
- Build a full-stack web application with Rust
- Understand Actix Web's actor-based architecture
- Implement a layered architecture (handlers, services, repositories)
- Work with PostgreSQL using SeaORM
- Create RESTful APIs with OpenAPI documentation
- Implement server-side rendering with Askama templates
- Apply production-ready security features
- Write comprehensive tests (unit, integration, end-to-end)
- Deploy applications with Docker and Docker Compose
- Set up CI/CD pipelines with GitHub Actions
- Implement observability with OpenTelemetry, Jaeger, Prometheus, and Grafana

## Technology Stack

### Core Technologies

#### Actix Web 4
**What it is**: A powerful, pragmatic, and extremely fast web framework for Rust.

**Technical description**:
Actix Web is built on the Actix actor framework, providing an asynchronous, type-safe foundation for building web services. It uses Tokio as its async runtime and leverages Rust's zero-cost abstractions for exceptional performance.

**Why we chose it**:
- **Performance**: One of the fastest web frameworks across all languages (benchmarks show it handling 1M+ requests/sec)
- **Type safety**: Compile-time guarantees prevent many runtime errors
- **Async by default**: Built on Tokio, enabling efficient concurrent request handling
- **Middleware ecosystem**: Rich middleware support for logging, compression, CORS, etc.
- **Extractors**: Type-safe request data extraction with automatic validation
- **Production proven**: Used by companies like Microsoft, Discord, and others
- **Active development**: Strong community and regular updates

**Alternatives considered**:
- **Axum**: Newer, simpler API, but smaller ecosystem and less battle-tested
- **Rocket**: Great DX but uses nightly Rust (now stable but still catching up)
- **Warp**: Filter-based, more functional style, steeper learning curve

---

#### PostgreSQL 16
**What it is**: Advanced open-source relational database management system.

**Technical description**:
PostgreSQL is an ACID-compliant, feature-rich RDBMS with support for complex queries, full-text search, JSONB, window functions, CTEs, and advanced indexing strategies. Version 16 includes performance improvements and enhanced monitoring.

**Why we chose it**:
- **ACID compliance**: Ensures data integrity and consistency
- **Rich feature set**: Supports JSON, arrays, custom types, and advanced queries
- **Excellent performance**: Efficient query planning and execution
- **Strong typing**: Aligns well with Rust's type system
- **Extensibility**: Support for custom functions, operators, and extensions
- **Mature ecosystem**: Decades of development, extensive tooling
- **Open source**: No licensing costs, community-driven development

**Alternatives considered**:
- **MySQL**: Less feature-rich, different SQL dialect
- **SQLite**: Limited concurrency, not suitable for production web apps
- **MongoDB**: NoSQL, loses benefits of relational data and transactions

---

#### SeaORM 1.0
**What it is**: Async ORM for Rust with compile-time safety and dynamic query building.

**Technical description**:
SeaORM provides async/await database operations with a compile-time verified entity model. It uses the sea-query builder for dynamic SQL generation, supports migrations, and offers both active record and repository patterns.

**Why we chose it**:
- **Async/await native**: Integrates seamlessly with Actix Web's async runtime
- **Type safety**: Entities are type-checked at compile time
- **Migration support**: Built-in migration management via sea-orm-cli
- **Multiple patterns**: Supports both ActiveRecord and Repository patterns
- **Query builder**: Flexible, composable query construction
- **Mock testing**: Built-in support for testing without a database
- **Growing ecosystem**: Active development, improving tooling
- **Database agnostic**: Supports PostgreSQL, MySQL, SQLite

**Alternatives considered**:
- **Diesel**: More mature but synchronous-only (requires thread pools)
- **SQLx**: Lower-level, requires more boilerplate, but compile-time query checking
- **Raw SQL**: Maximum control but loses type safety and abstraction

---

#### Askama
**What it is**: Type-safe compile-time template engine for Rust.

**Technical description**:
Askama uses Jinja2-like syntax and compiles templates into Rust code at compile time. Templates are type-checked and benefit from Rust's ownership system, preventing XSS and other template injection attacks by default.

**Why we chose it**:
- **Compile-time verification**: Templates are checked at build time, not runtime
- **Type safety**: Variables must match expected types
- **Zero runtime overhead**: Templates compiled to native code
- **Familiar syntax**: Jinja2-like syntax is widely known
- **XSS prevention**: Auto-escaping by default
- **No runtime dependencies**: Templates are Rust code after compilation
- **Performance**: No template parsing at runtime

**Alternatives considered**:
- **Tera**: Runtime template engine, more flexible but slower and less safe
- **Handlebars**: Good but runtime-based, less Rust-idiomatic
- **JSX/React**: Requires JavaScript, loses server-side rendering benefits

---

#### utoipa
**What it is**: Auto-generate OpenAPI specifications from Rust code.

**Technical description**:
utoipa uses Rust derive macros and attributes to generate OpenAPI 3.0 specifications directly from your code. It integrates with Actix Web to provide Swagger UI and ReDoc documentation interfaces.

**Why we chose it**:
- **Code-first approach**: Documentation stays in sync with implementation
- **Derive macros**: Minimal boilerplate, declarative style
- **OpenAPI 3.0**: Industry-standard API specification format
- **Swagger UI integration**: Interactive API testing in browser
- **Type safety**: Leverages Rust's type system for accurate schemas
- **Auto-validation**: API contracts are enforced by the compiler

**Alternatives considered**:
- **Manual OpenAPI**: Tedious, error-prone, gets out of sync
- **paperclip**: Less maintained, older OpenAPI version support
- **Swagger annotations**: Not idiomatic for Rust

---

#### Validator
**What it is**: Struct validation library using derive macros.

**Technical description**:
Validator provides declarative validation rules via derive macros. Supports common validations (email, URL, length, range) and custom validators. Integrates with serde for JSON deserialization and validation in one step.

**Why we chose it**:
- **Declarative**: Validation rules in struct definitions
- **Comprehensive**: Rich set of built-in validators
- **Custom validators**: Easy to add domain-specific validation
- **Serde integration**: Works seamlessly with JSON deserialization
- **Error messages**: Customizable, localization-ready
- **Zero runtime cost**: Validation code generated at compile time

**Alternatives considered**:
- **garde**: Newer, less ecosystem support
- **Manual validation**: Verbose, scattered logic, easy to miss cases
- **JSON Schema**: Runtime validation, not type-safe

---

#### Tracing
**What it is**: Application-level structured logging and diagnostics.

**Technical description**:
Tracing provides structured, contextual logging with support for spans (operations) and events. It's designed for async applications and integrates with OpenTelemetry for distributed tracing.

**Why we chose it**:
- **Structured logging**: Key-value pairs, not string concatenation
- **Async-aware**: Tracks context across async boundaries
- **Spans and events**: Hierarchical operation tracking
- **Low overhead**: Minimal performance impact when disabled
- **Ecosystem integration**: Works with Jaeger, Prometheus, etc.
- **Flexible output**: JSON, pretty console, or custom formats
- **Industry standard**: Used throughout Rust async ecosystem

**Alternatives considered**:
- **log crate**: Simpler but unstructured, no span support
- **slog**: Good but more complex setup, less async-friendly
- **println!/dbg!**: Not production-suitable, no log levels

---

#### Tokio
**What it is**: Asynchronous runtime for Rust.

**Technical description**:
Tokio provides a multi-threaded, work-stealing scheduler for executing asynchronous tasks. It includes async I/O primitives, timers, and synchronization utilities, forming the foundation of Rust's async ecosystem.

**Why we chose it**:
- **Industry standard**: De facto async runtime for Rust
- **Performance**: Efficient work-stealing scheduler
- **Comprehensive**: I/O, timers, channels, synchronization primitives
- **Production-proven**: Powers Discord, AWS Lambda, and more
- **Great tooling**: Console debugger, tracing integration
- **Active development**: Backed by the Tokio team
- **Required by Actix**: Actix Web runs on Tokio

**Alternatives considered**:
- **async-std**: Alternative runtime, smaller ecosystem
- **smol**: Lightweight, but less feature-complete
- **Custom runtime**: Not practical for most applications

---

### Supporting Technologies

#### Docker & Docker Compose
**What it is**: Containerization platform and orchestration tool.

**Technical description**:
Docker packages applications and dependencies into containers. Docker Compose defines multi-container applications using YAML, managing networking, volumes, and service dependencies.

**Why we chose it**:
- **Consistency**: Same environment dev to production
- **Isolation**: Dependencies don't conflict with host system
- **Portability**: Runs anywhere Docker runs
- **Multi-stage builds**: Optimize image size for Rust apps
- **Ecosystem**: Industry-standard container platform
- **Simple orchestration**: Docker Compose for local development
- **Production ready**: Can deploy directly or transition to Kubernetes

---

#### GitHub Actions
**What it is**: CI/CD platform integrated with GitHub.

**Technical description**:
GitHub Actions provides workflow automation with YAML-defined pipelines. Workflows run on GitHub-hosted or self-hosted runners, supporting matrix builds, caching, and third-party actions.

**Why we chose it**:
- **Integration**: Native GitHub integration, no external setup
- **Free tier**: Generous free minutes for public repos
- **Mature**: Rich ecosystem of pre-built actions
- **Flexible**: Support for complex workflows and matrix builds
- **Caching**: Speeds up Rust builds with cargo caching
- **Secrets management**: Secure credential storage

**Alternatives considered**:
- **GitLab CI**: Requires GitLab, similar features
- **CircleCI**: Good but costs more for private repos
- **Jenkins**: Self-hosted, more complex setup

---

#### OpenTelemetry
**What it is**: Observability framework for distributed tracing and metrics.

**Technical description**:
OpenTelemetry provides vendor-neutral APIs and SDKs for collecting traces, metrics, and logs. It supports context propagation across services and exports data to various backends (Jaeger, Prometheus, etc.).

**Why we chose it**:
- **Vendor-neutral**: Not locked to specific monitoring tools
- **Industry standard**: CNCF project, wide adoption
- **Comprehensive**: Traces, metrics, and logs in one framework
- **Context propagation**: Track requests across services
- **Rich instrumentation**: Auto-instrument many libraries
- **Future-proof**: Growing ecosystem and adoption

---

#### Jaeger
**What it is**: Distributed tracing system.

**Technical description**:
Jaeger collects, stores, and visualizes distributed traces. It helps identify performance bottlenecks, understand request flows, and troubleshoot issues in microservices architectures.

**Why we chose it**:
- **CNCF project**: Industry-standard tracing solution
- **OpenTelemetry support**: Native integration
- **Powerful UI**: Trace visualization and comparison
- **Scalable**: Handles high-volume trace data
- **Service graph**: Visualize service dependencies
- **Open source**: No licensing costs

---

#### Prometheus
**What it is**: Time-series metrics database and monitoring system.

**Technical description**:
Prometheus scrapes metrics from instrumented applications, stores them in a time-series database, and provides a query language (PromQL) for analysis. Includes alerting via Alertmanager.

**Why we chose it**:
- **Pull-based**: Simple to add new metrics sources
- **Powerful queries**: PromQL for complex metric analysis
- **Time-series DB**: Efficient storage and retrieval
- **Alerting**: Built-in alert rules and notifications
- **Industry standard**: De facto metrics solution for cloud-native apps
- **OpenTelemetry support**: Receives metrics from OTEL collector

---

#### Grafana
**What it is**: Observability and visualization platform.

**Technical description**:
Grafana provides dashboards for visualizing metrics, logs, and traces from various data sources. Supports templating, alerting, and team collaboration features.

**Why we chose it**:
- **Multi-source**: Connects to Prometheus, Jaeger, and more
- **Rich visualizations**: Graphs, heatmaps, tables, and more
- **Templating**: Dynamic dashboards with variables
- **Alerting**: Visual alert configuration
- **Community dashboards**: Pre-built dashboards to import
- **Open source**: Free for most use cases

---

### Additional Libraries

#### ammonia
**What it is**: HTML sanitization library.

**Why we chose it**: Prevents XSS attacks by sanitizing user-generated HTML content. Battle-tested, uses Mozilla's whitelist approach.

---

#### actix-governor
**What it is**: Rate limiting middleware for Actix Web.

**Why we chose it**: Prevents abuse through IP-based rate limiting. Simple integration, configurable limits, production-ready.

---

#### actix-cors
**What it is**: CORS middleware for Actix Web.

**Why we chose it**: Handles Cross-Origin Resource Sharing securely. Flexible configuration, supports preflight requests, integrates seamlessly.

---

#### actix-files
**What it is**: Static file serving for Actix Web.

**Why we chose it**: Efficiently serves CSS, JS, images. Supports caching headers, range requests, and compression.

---

## Tutorial Structure

The tutorial is divided into 18 chapters, progressively building a complete application. Each chapter builds on the previous, with clear objectives and hands-on exercises.

### Part 1: Foundation (Chapters 0-4)

#### Chapter 0: Prerequisites and Environment Setup
**What you'll build**: Development environment ready for Rust web development

**Key concepts**:
- Rust toolchain installation and management
- PostgreSQL setup and configuration
- SeaORM CLI installation
- IDE setup and configuration
- Project structure understanding

**Deliverables**:
- Rust and Cargo installed and verified
- PostgreSQL running (native or Docker)
- SeaORM CLI available
- Code editor configured with Rust extensions
- Understanding of project organization

---

#### Chapter 1: Core Application Setup
**What you'll build**: Minimal working Actix Web server with configuration

**Key concepts**:
- Actix Web application structure
- HttpServer and App configuration
- Environment-based configuration with dotenvy
- Application state pattern
- Basic routing and handlers
- Settings management

**Technical focus**:
- Actix Web's factory pattern for App creation
- Shared state via `web::Data<T>`
- Type-safe configuration structs
- Environment variable loading and validation
- Server binding and worker configuration

**Deliverables**:
- Running web server on port 3737
- Configuration loaded from .env file
- Basic health check endpoint
- Application state with settings
- Understanding of Actix Web request lifecycle

---

#### Chapter 2: Database Integration with SeaORM
**What you'll build**: Database connection and entity models

**Key concepts**:
- SeaORM connection management
- Database migrations with sea-orm-cli
- Entity generation from schema
- Connection pooling
- Database configuration

**Technical focus**:
- Async database connections
- Migration versioning and rollback
- Entity model structure and annotations
- DatabaseConnection in application state
- Connection pool tuning (max connections, timeouts)
- Database URL format and security

**Deliverables**:
- Database schema with memos table
- Generated SeaORM entities
- Migration system setup
- Database connection in app state
- Verified database connectivity from app

---

#### Chapter 3: Error Handling and Middleware
**What you'll build**: Centralized error handling and security middleware

**Key concepts**:
- Custom error types with enums
- ResponseError trait implementation
- Middleware creation and chaining
- Security headers configuration
- CORS setup
- Compression middleware

**Technical focus**:
- Error type design (Database, NotFound, Validation, Internal)
- Error conversion with `From` trait
- HTTP status code mapping
- Middleware transform pattern in Actix
- Security headers (X-Content-Type-Options, X-Frame-Options, etc.)
- CORS preflight handling
- Gzip and Brotli compression

**Deliverables**:
- AppError enum with proper HTTP mapping
- Consistent JSON error responses
- Security headers on all responses
- CORS configured for allowed origins
- Response compression enabled
- Error handling tested with invalid requests

---

#### Chapter 4: Health Checks and Monitoring
**What you'll build**: Health check endpoints for monitoring

**Key concepts**:
- Health check endpoint design
- Database health verification
- Kubernetes readiness/liveness probes
- Structured health responses

**Technical focus**:
- Synchronous vs async health checks
- Database ping for connectivity verification
- JSON health response format
- HTTP status codes for health states
- Readiness vs liveness semantics

**Deliverables**:
- `/health` endpoint with DB status
- `/ready` endpoint for Kubernetes
- JSON responses with health details
- Tested health check failure scenarios

---

### Part 2: Core Architecture (Chapters 5-7)

#### Chapter 5: Data Transfer Objects and Validation
**What you'll build**: Type-safe DTOs with validation

**Key concepts**:
- DTO pattern and separation from entities
- Validation with validator crate
- Custom validation rules
- OpenAPI schema generation
- Serde serialization/deserialization

**Technical focus**:
- Struct design for API contracts
- Validator derive macros and annotations
- Length, email, URL, range validators
- Custom validation functions
- Validation error handling and messaging
- OpenAPI schema annotations with utoipa
- Option<T> for optional fields
- Default values and serde attributes

**Deliverables**:
- CreateMemoDto with validation
- UpdateMemoDto (full update)
- PatchMemoDto (partial update)
- MemoResponseDto (outbound)
- PaginationParams with defaults
- PaginatedResponse<T> generic wrapper
- Validation tested with invalid data

---

#### Chapter 6: Repository Layer - Database Operations
**What you'll build**: Database access layer with CRUD operations and transaction handling

**Key concepts**:
- Repository pattern
- SeaORM query builder
- Pagination implementation
- Filtering and sorting
- Database transactions and ACID properties
- Transaction isolation levels
- Rollback and commit strategies

**Technical focus**:
- Repository struct with DatabaseConnection
- ActiveModel for inserts and updates
- Entity queries with filters
- Pagination with offset/limit
- Dynamic query building for filters
- Order by with column selection
- Error handling for DB operations
- **Transaction management deep dive**:
  - Creating transactions with `db.begin()`
  - Transaction commit and rollback
  - Using transactions for multi-step operations
  - Transaction scope and lifecycle
  - Error handling in transactions (automatic rollback)
  - Nested transactions and savepoints
  - Transaction isolation levels in PostgreSQL
  - When to use transactions vs. single operations
  - Testing transactional behavior

**Deliverables**:
- MemoRepository with CRUD methods
- Paginated list queries
- Filtering by completion status
- Sorting by multiple fields
- Transaction examples for:
  - Batch operations (create multiple memos atomically)
  - Complex updates requiring consistency
  - Operations that must succeed or fail together
- Repository tests with test database
- Transaction rollback tests (verify data not persisted on error)
- Understanding of SeaORM query patterns and transaction semantics

---

#### Chapter 7: Service Layer - Business Logic and Transactions
**What you'll build**: Service layer orchestrating business logic with transaction coordination

**Key concepts**:
- Service layer pattern
- DTO to Entity conversion
- Input sanitization (XSS prevention)
- Business logic encapsulation
- Service composition
- **Transaction orchestration at service layer**
- Multi-repository coordination

**Technical focus**:
- Service struct with repository dependency
- DTO to ActiveModel conversion
- Entity to DTO mapping
- HTML sanitization with ammonia
- Business rule enforcement
- Service method signatures (async, Result<T, E>)
- Separation of concerns (service vs repository)
- **Transaction coordination**:
  - Passing transaction context to repository methods
  - Coordinating multiple repository calls in one transaction
  - Service-level transaction boundaries
  - Complex business operations requiring atomicity
  - Transaction error handling and propagation

**Deliverables**:
- MemoService with business methods
- DTO conversion utilities
- XSS sanitization for user input
- Service methods demonstrating transaction usage:
  - Bulk create with validation (all-or-nothing)
  - Complex updates involving multiple entities
  - Business operations requiring data consistency
- Service unit tests (mocking repository)
- Transaction integration tests
- Clean service API for handlers

---

### Part 3: REST API (Chapters 8-9)

#### Chapter 8: REST API Handlers
**What you'll build**: Complete REST API with all CRUD operations

**Key concepts**:
- RESTful API design principles
- HTTP methods and status codes
- Request extraction (JSON, path, query)
- JSON response formatting
- API versioning

**Technical focus**:
- Actix Web handler functions
- web::Json<T> extractor for request bodies
- web::Path<T> for path parameters
- web::Query<T> for query strings
- web::Data<T> for app state access
- Status code selection (200, 201, 204, 404, 400, 500)
- Response builders and HttpResponse
- Error propagation with `?` operator
- Route configuration and scoping

**Deliverables**:
- GET /api/v1/memos (list with pagination, filtering, sorting)
- GET /api/v1/memos/{id} (get single memo)
- POST /api/v1/memos (create new memo)
- PUT /api/v1/memos/{id} (full update)
- PATCH /api/v1/memos/{id} (partial update)
- DELETE /api/v1/memos/{id} (delete)
- PATCH /api/v1/memos/{id}/complete (toggle completion)
- Integration tests for all endpoints
- Tested with curl or HTTP client

---

#### Chapter 9: OpenAPI Documentation
**What you'll build**: Auto-generated API documentation with Swagger UI

**Key concepts**:
- OpenAPI 3.0 specification
- Code-first documentation
- Swagger UI integration
- API schema generation

**Technical focus**:
- utoipa derive macros on DTOs and handlers
- #[utoipa::path] annotations
- Component schemas with #[derive(ToSchema)]
- OpenAPI struct composition
- SwaggerUi service configuration
- JSON spec endpoint
- Documentation tags and descriptions
- Example values in schemas

**Deliverables**:
- OpenAPI spec at /api-docs/openapi.json
- Interactive Swagger UI at /swagger-ui/
- All endpoints documented with examples
- Request/response schemas auto-generated
- Try-it-out functionality working

---

### Part 4: Web UI (Chapters 10-11)

#### Chapter 10: Askama Templates - Server-Side Rendering
**What you'll build**: HTML templates with type-safe rendering

**Key concepts**:
- Template engine fundamentals
- Template inheritance
- Component composition
- Compile-time template checking

**Technical focus**:
- Askama template syntax (Jinja2-like)
- Template struct definitions
- Template trait implementation
- Template inheritance with {% extends %}
- Blocks for content replacement
- Includes for reusable components
- Variable interpolation and escaping
- Control flow (if, for, match)
- Filters for formatting

**Deliverables**:
- base.html layout template
- pages/index.html (homepage)
- pages/error.html (error page)
- components/memo_form.html (create/edit)
- components/memo_list.html (list view)
- components/memo_item.html (single item)
- partials/header.html, footer.html
- Template structs in Rust code
- Templates compile without errors

---

#### Chapter 11: Web Page Handlers - Building the UI
**What you'll build**: Server-rendered web pages with progressive enhancement

**Key concepts**:
- Server-side rendering patterns
- HTML form handling
- Progressive enhancement with vanilla JS
- Client-side interactivity without framework

**Technical focus**:
- Actix Web HTML responses
- Template rendering in handlers
- Form submission handling (POST, PUT, DELETE)
- Form validation and error display
- Redirect after post pattern
- Vanilla JavaScript for AJAX calls
- Fetch API for REST communication
- DOM manipulation without jQuery
- Event delegation for dynamic content

**Deliverables**:
- GET / (homepage with memo list)
- GET /memos/new (create form)
- POST /memos (handle creation)
- GET /memos/{id} (memo detail)
- GET /memos/{id}/edit (edit form)
- PUT /memos/{id} (handle update)
- DELETE /memos/{id} (handle deletion)
- POST /memos/{id}/toggle (toggle complete)
- JavaScript for delete confirmation
- AJAX updates without page reload
- Integration tests for web handlers

---

### Part 5: Static Assets (Chapter 12)

#### Chapter 12: Static Assets and Styling
**What you'll build**: CSS styling and static file serving

**Key concepts**:
- Static file serving in Actix
- CSS organization
- Responsive design basics
- Asset optimization

**Technical focus**:
- actix-files configuration
- Static file routes
- Cache headers for assets
- CSS structure and naming
- Responsive layouts with flexbox/grid
- Modern CSS features (custom properties, etc.)
- Asset compression in production

**Deliverables**:
- static/css/style.css with app styling
- Responsive design for mobile/desktop
- Static file serving configured
- Cache headers for performance
- Clean, maintainable CSS

---

### Part 6: Security and Quality (Chapters 13-14)

#### Chapter 13: Security Enhancements
**What you'll build**: Production-ready security features

**Key concepts**:
- Rate limiting strategies
- XSS prevention
- CSRF protection considerations
- Input validation defense-in-depth
- Security headers deep dive

**Technical focus**:
- actix-governor rate limiter setup
- IP-based rate limiting
- Custom rate limit keys
- ammonia HTML sanitization
- Content Security Policy headers
- HSTS in production
- Request size limits
- Secure cookie configuration (future auth)

**Deliverables**:
- Rate limiting at 100 req/min per IP
- HTML sanitization on all user input
- Comprehensive security headers
- Request size limited to 256KB
- Security testing with attack vectors
- Security checklist completed

---

#### Chapter 14: Testing Strategy
**What you'll build**: Comprehensive test suite

**Key concepts**:
- Unit testing in Rust
- Integration testing
- Repository testing with test DB
- HTTP endpoint testing
- Test fixtures and utilities

**Technical focus**:
- #[cfg(test)] modules
- #[actix_web::test] for async tests
- Test database setup and teardown
- Mock service implementations
- TestRequest builder for HTTP tests
- Assertion patterns
- Test coverage measurement
- Parallel vs serial test execution

**Deliverables**:
- Service unit tests (business logic)
- Repository tests (DB operations)
- API integration tests (REST endpoints)
- Web handler tests (HTML pages)
- Test fixtures and helpers
- High test coverage (>80%)
- All tests passing with `cargo test`

---

### Part 7: Deployment (Chapter 15)

#### Chapter 15: Docker Deployment
**What you'll build**: Containerized application with orchestration

**Key concepts**:
- Docker multi-stage builds
- Docker Compose orchestration
- Container networking
- Volume management
- Health checks in containers

**Technical focus**:
- Rust Docker best practices
- Multi-stage builds (builder + runtime)
- Minimal runtime images (distroless/alpine)
- Layer caching for faster builds
- Docker Compose service definition
- PostgreSQL container configuration
- Volume persistence for database
- Network creation and isolation
- Health check configuration
- Environment variable injection
- Production-ready container setup

**Deliverables**:
- Optimized Dockerfile with multi-stage build
- docker-compose.yml with app + PostgreSQL
- .dockerignore for efficient builds
- Volume for database persistence
- Health checks configured
- Application running in containers
- Docker networking functional
- Tested deployment workflow

---

### Part 8: CI/CD (Chapter 16)

#### Chapter 16: CI/CD Pipeline
**What you'll build**: Automated testing and deployment pipeline

**Key concepts**:
- Continuous integration
- Continuous deployment
- GitHub Actions workflows
- Automated testing
- Release automation

**Technical focus**:
- Workflow YAML syntax
- Matrix builds for multiple Rust versions
- Cargo caching strategies
- Test job configuration
- Lint job with clippy
- Format check with rustfmt
- Release workflow with tagging
- Docker image building and publishing
- GitHub Container Registry
- Secrets management
- Branch protection rules

**Deliverables**:
- .github/workflows/test.yml (run tests on PR)
- .github/workflows/lint.yml (clippy + fmt)
- .github/workflows/release.yml (build + publish)
- Automated Docker image publishing
- Badge in README for build status
- Working CI/CD pipeline

---

### Part 9: Observability (Chapter 17)

#### Chapter 17: Observability Stack
**What you'll build**: Distributed tracing and metrics monitoring

**Key concepts**:
- Observability pillars (traces, metrics, logs)
- Distributed tracing
- Metrics collection and visualization
- Dashboard creation

**Technical focus**:
- OpenTelemetry SDK integration
- Tracing instrumentation with macros
- Span creation and context propagation
- Jaeger exporter configuration
- Prometheus metrics exporter
- Metric types (counter, gauge, histogram)
- Custom metrics creation
- Grafana data source configuration
- Dashboard JSON structure
- Query language (PromQL)
- Trace visualization and analysis

**Deliverables**:
- OpenTelemetry integrated in app
- Traces exported to Jaeger
- Jaeger UI accessible
- Metrics exported to Prometheus
- Prometheus scraping app metrics
- Grafana dashboards for:
  - Request rate and latency
  - Error rates
  - Database query performance
  - System metrics (CPU, memory)
- Docker Compose with full stack
- End-to-end observability working

---

### Part 10: Advanced Topics (Chapter 18)

#### Chapter 18: Documentation and Next Steps
**What you'll build**: Comprehensive project documentation

**Key concepts**:
- Documentation as code
- README best practices
- Architecture documentation
- Deployment guides

**Technical focus**:
- Markdown documentation
- Code comments and rustdoc
- API documentation completeness
- Architecture diagrams
- Troubleshooting guides
- Performance tuning guides

**Deliverables**:
- Updated README.md
- ARCHITECTURE.md with diagrams
- DEPLOYMENT.md for production
- API.md for API consumers
- Complete rustdoc comments
- Contributing guidelines

**Future enhancements discussed**:
- User authentication (JWT)
- Authorization and permissions
- WebSocket support for real-time updates
- Background job processing
- Caching with Redis
- Full-text search
- File uploads and storage
- Multi-tenancy
- GraphQL API alternative
- Frontend SPA (React/Vue)

---

## Tutorial Delivery Format

### Chapter Structure

Each chapter follows this consistent format:

```markdown
# Chapter N: [Title]

## Overview
Brief description of what you'll build and why it matters.

## Prerequisites
- Completed chapters
- Required knowledge
- Required software

## Learning Objectives
- Concrete skill 1
- Concrete skill 2
- Concrete skill 3

## Concepts Covered
Detailed explanation of concepts with technical depth.

## Step-by-Step Instructions

### Step 1: [Task Name]
**Why**: Explanation of purpose
**How**: Detailed instructions
**Code**: Complete code snippets
**Verify**: How to test this step

### Step 2: [Next Task]
...

## Checkpoint
Run these commands to verify everything works:
```bash
# verification commands
```

Expected output:
```
# expected results
```

## Common Issues and Solutions

### Issue: [Problem description]
**Symptoms**: What you see
**Cause**: Why it happens
**Solution**: How to fix it

## Code Review
Complete code for this chapter with explanations.

## Testing
How to test what you built in this chapter.

## Summary
- What you learned
- How it fits in the architecture
- Key takeaways

## Next Steps
Preview of the next chapter and optional exercises.

## Additional Resources
- Links to docs
- Related articles
- Video tutorials
```

---

## Repository Structure for Tutorial

```
actix-web-template/
├── tutorial/
│   ├── README.md                    # Tutorial index
│   ├── chapter-00.md                # Environment setup
│   ├── chapter-01.md                # Core application
│   ├── chapter-02.md                # Database integration
│   ├── chapter-03.md                # Error handling
│   ├── chapter-04.md                # Health checks
│   ├── chapter-05.md                # DTOs and validation
│   ├── chapter-06.md                # Repository layer
│   ├── chapter-07.md                # Service layer
│   ├── chapter-08.md                # REST API handlers
│   ├── chapter-09.md                # OpenAPI docs
│   ├── chapter-10.md                # Askama templates
│   ├── chapter-11.md                # Web handlers
│   ├── chapter-12.md                # Static assets
│   ├── chapter-13.md                # Security
│   ├── chapter-14.md                # Testing
│   ├── chapter-15.md                # Docker deployment
│   ├── chapter-16.md                # CI/CD pipeline
│   ├── chapter-17.md                # Observability
│   └── chapter-18.md                # Documentation
├── TUTORIAL_PLAN.md                 # This file
└── ... (rest of project files)
```

---

## Success Criteria

By the end of this tutorial, students will have:

1. **Working Application**: Fully functional memo management system
2. **Production Skills**: Real-world patterns and best practices
3. **Testing Knowledge**: Comprehensive test coverage
4. **Deployment Experience**: Docker and CI/CD setup
5. **Monitoring Setup**: Observability with tracing and metrics
6. **Portfolio Project**: Showcase-worthy Rust web application

---

## Estimated Time Investment

- **Foundation (Chapters 0-4)**: 2-3 hours
- **Architecture (Chapters 5-7)**: 2-3 hours
- **API Development (Chapters 8-9)**: 2-3 hours
- **Web UI (Chapters 10-12)**: 2-3 hours
- **Security & Testing (Chapters 13-14)**: 2-3 hours
- **Deployment (Chapter 15)**: 1-2 hours
- **CI/CD (Chapter 16)**: 1-2 hours
- **Observability (Chapter 17)**: 1-2 hours
- **Documentation (Chapter 18)**: 1 hour

**Total**: 14-21 hours (varies by experience level)

---

## Support and Community

- **GitHub Issues**: Report problems or ask questions
- **Discussions**: General questions and showcase projects
- **Discord/Slack**: Real-time community support (if available)
- **Stack Overflow**: Tag questions with `actix-web`, `rust`, `seaorm`

---

## License

This tutorial is provided under the MIT License, same as the project code.

---

**Ready to begin? Start with [Chapter 0: Prerequisites and Environment Setup](tutorial/chapter-00.md)**