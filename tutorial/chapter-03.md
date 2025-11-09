# Chapter 3: Error Handling and Middleware

## Overview

In this chapter, we'll build a robust error handling system and add essential middleware to our Actix Web application. You'll learn how to create custom error types, map them to HTTP status codes, and implement middleware for security, CORS, and compression.

By the end of this chapter, you'll have a production-ready error handling system and a suite of middleware that enhances security, performance, and developer experience.

> **Note on Tutorial Approach**: This chapter demonstrates foundational error handling and middleware patterns. The production codebase in this repository extends these concepts with additional features like rate limiting (actix-governor), OpenTelemetry integration, and enhanced security headers. We'll build the core patterns here that can be extended in later chapters.

## Prerequisites

### Completed

- Chapter 0: Prerequisites and Environment Setup
- Chapter 1: Core Application Setup
- Chapter 2: Database Integration with SeaORM

### Required Knowledge

- Rust error handling (Result, Error trait)
- Understanding of HTTP status codes
- Basic knowledge of middleware patterns
- Familiarity with async/await

### Required Software

- Working Actix Web application from Chapter 2
- PostgreSQL running

## Learning Objectives

By completing this chapter, you will:

1. Understand error handling best practices in Actix Web
2. Create a centralized error type with `enum`
3. Implement the `ResponseError` trait for custom errors
4. Map errors to appropriate HTTP status codes
5. Create custom middleware for cross-cutting concerns
6. Configure security headers middleware
7. Set up CORS for cross-origin requests
8. Add response compression (Gzip, Brotli)
9. Handle database errors gracefully

## Concepts Covered

### Error Handling in Actix Web

Actix Web uses the `ResponseError` trait to convert errors into HTTP responses:

```rust
pub trait ResponseError: std::error::Error {
    fn status_code(&self) -> StatusCode { ... }
    fn error_response(&self) -> HttpResponse { ... }
}
```

Any type implementing this trait can be returned from handlers as `Result<T, E>`.

### Custom Error Types

Instead of using generic error types, we create a custom `AppError` enum:

**Benefits**:
- Type-safe error handling
- Centralized error response formatting
- Easy mapping of errors to HTTP status codes
- Consistent error messages to clients
- Internal error details hidden from users

### Middleware in Actix Web

Middleware wraps handlers to add functionality:

```
Request → Middleware 1 → Middleware 2 → Handler
                                          ↓
Response ← Middleware 1 ← Middleware 2 ← Handler
```

**Common uses**:
- Logging and tracing
- Authentication and authorization
- Security headers
- Compression
- Rate limiting
- CORS
- Request/response modification

### Security Headers

HTTP headers that improve security:
- `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
- `X-Frame-Options: DENY` - Prevents clickjacking
- `X-XSS-Protection: 1; mode=block` - Enables XSS filter
- `Strict-Transport-Security` - Forces HTTPS (production)
- `Content-Security-Policy` - Controls resource loading

### CORS (Cross-Origin Resource Sharing)

Allows controlled access from different origins (domains). Essential for:
- Frontend apps on different domains
- Mobile applications
- Third-party integrations

## Step-by-Step Instructions

### Step 1: Create Error Module

**Why**: Centralize all error types and error handling logic.

**How**:

1. **Create error directory**:
   ```bash
   mkdir -p src/error
   touch src/error/mod.rs
   touch src/error/app_error.rs
   ```

2. **Create `src/error/app_error.rs`**:

```rust
use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use sea_orm::DbErr;
use std::fmt;

/// Main application error type
#[derive(Debug)]
pub enum AppError {
    /// Database errors
    Database(DbErr),
    /// Resource not found errors
    NotFound(String),
    /// Validation errors (bad input)
    Validation(String),
    /// Internal server errors
    Internal(String),
    /// Unauthorized access
    Unauthorized(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        // Log internal errors but don't expose details to client
        if matches!(self, AppError::Database(_) | AppError::Internal(_)) {
            tracing::error!("Internal error occurred: {}", self);
        }

        // Create error response body
        let error_message = match self {
            // Don't expose internal error details
            AppError::Database(_) => "A database error occurred".to_string(),
            AppError::Internal(_) => "An internal server error occurred".to_string(),
            // Safe to expose these messages
            AppError::NotFound(msg) => msg.clone(),
            AppError::Validation(msg) => msg.clone(),
            AppError::Unauthorized(msg) => msg.clone(),
        };

        HttpResponse::build(status)
            .insert_header(ContentType::json())
            .json(serde_json::json!({
                "error": {
                    "message": error_message,
                    "status": status.as_u16(),
                }
            }))
    }
}

/// Convert SeaORM DbErr to AppError
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(_) => AppError::NotFound("Record not found".to_string()),
            _ => AppError::Database(err),
        }
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
```

3. **Create `src/error/mod.rs`**:

```rust
mod app_error;

pub use app_error::{AppError, AppResult};
```

4. **Update `src/lib.rs`** to include error module:

```rust
pub mod config;
pub mod entities;
pub mod error;
pub mod handlers;
pub mod state;
pub mod utils;
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 2: Update Handlers to Use AppError

**Why**: Demonstrate how to use the custom error type in handlers.

**How**:

1. **Update `src/handlers/health.rs`**:

```rust
use actix_web::{web, HttpResponse, Responder};
use sea_orm::DatabaseConnection;
use serde_json::json;
use crate::error::{AppError, AppResult};

/// Health check endpoint with database connectivity check
///
/// Returns JSON with status "ok" and database status
#[tracing::instrument(skip(db))]
pub async fn health_check(db: web::Data<DatabaseConnection>) -> AppResult<impl Responder> {
    tracing::info!("Health check requested");

    // Check database connectivity
    let db_status = match check_database(&db).await {
        Ok(_) => "connected",
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            "disconnected"
        }
    };

    let status = if db_status == "connected" { "ok" } else { "degraded" };

    let status_code = if status == "ok" {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    Ok(HttpResponse::build(status_code).json(json!({
        "status": status,
        "service": "actix-memo-app",
        "database": db_status
    })))
}

/// Simple readiness probe for Kubernetes
#[tracing::instrument]
pub async fn ready() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "ready": true
    }))
}

/// Check database connectivity
async fn check_database(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    crate::utils::database::verify_connection(db).await
}

/// Example error endpoint for testing error handling
#[tracing::instrument]
pub async fn trigger_error() -> AppResult<impl Responder> {
    Err(AppError::Internal("This is a test error".to_string()))
}
```

2. **Update `src/main.rs`** to add error test endpoint:

```rust
mod config;
mod entities;
mod error;
mod handlers;
mod state;
mod utils;

use actix_web::{web, App, HttpServer};
use config::Settings;
use state::AppState;
use std::io;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize tracing for structured logging
    utils::tracing::init_tracing();

    tracing::info!("Starting Actix Memo Application");

    // Load configuration from environment
    let settings = Settings::load()
        .expect("Failed to load settings");

    tracing::info!(
        "Configuration loaded - Environment: {}, Server: {}:{}",
        settings.app.env,
        settings.server.host,
        settings.server.port
    );

    // Establish database connection
    let db = utils::database::establish_connection(&settings)
        .await
        .expect("Failed to connect to database");

    // Verify database connection
    utils::database::verify_connection(&db)
        .await
        .expect("Failed to verify database connection");

    tracing::info!("Database connection verified");

    // Create application state
    let app_state = AppState::new(settings.clone(), db.clone());
    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);

    tracing::info!("Starting HTTP server at {}", bind_address);

    // Create and run HTTP server
    HttpServer::new(move || {
        App::new()
            // Add application state
            .app_data(web::Data::new(app_state.clone()))
            // Share database connection separately for convenience
            .app_data(web::Data::new(db.clone()))
            // Add request logging middleware
            .wrap(TracingLogger::default())
            // Register routes
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/ready", web::get().to(handlers::health::ready))
            .route("/error", web::get().to(handlers::health::trigger_error))
            // Welcome route
            .route("/", web::get().to(welcome))
    })
    .bind(&bind_address)?
    .run()
    .await
}

/// Welcome endpoint
async fn welcome() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body(
        "Welcome to Actix Memo App! Try /health, /ready, or /error"
    )
}
```

**Verify**:
```bash
cargo run
```

In another terminal:
```bash
# Test error endpoint
curl http://localhost:3737/error

# Expected output:
# {"error":{"message":"An internal server error occurred","status":500}}
```

---

### Step 3: Create Security Headers Middleware

**Why**: Add security headers to all responses to protect against common vulnerabilities.

#### Understanding Middleware in Actix Web

Before we create custom middleware, let's understand what middleware is and how it works.

**What is Middleware?**

Middleware is code that sits between the HTTP server and your request handlers. It can intercept and process requests before they reach handlers, and responses before they're sent to clients.

**Common use cases**:
- **Logging**: Record requests and responses (what we've been using with `TracingLogger`)
- **Security**: Add security headers, validate authentication
- **Modification**: Compress responses, parse request bodies
- **Rate limiting**: Prevent abuse
- **CORS**: Handle cross-origin requests

**How it works**:

```
Client Request
     ↓
SecurityHeaders Middleware (adds headers)
     ↓
CORS Middleware (handles cross-origin)
     ↓
TracingLogger Middleware (logs request, creates span)
     ↓
Your Handler (processes request)
     ↓
TracingLogger Middleware (logs response, completes span)
     ↓
CORS Middleware (adds CORS headers)
     ↓
SecurityHeaders Middleware (adds security headers)
     ↓
Client Response
```

**Key concepts**:
- Middleware wraps other middleware and handlers
- Executes in "layers" - like an onion
- Request flow: outer → inner
- Response flow: inner → outer

**Order matters!**

Middleware wraps handlers in layers. The order you add middleware with `.wrap()` determines the execution order:

```rust
App::new()
    .wrap(SecurityHeaders)  // Outer layer - executes FIRST on request
    .wrap(CORS)             // Middle layer
    .wrap(TracingLogger)    // Inner layer - executes LAST on request
    // Handler
```

Execution flow:
- **Request**: First wrapped → Last wrapped → Handler
- **Response**: Handler → Last wrapped → First wrapped

In the example above:
- Request: SecurityHeaders → CORS → TracingLogger → Handler
- Response: Handler → TracingLogger → CORS → SecurityHeaders

This is why compression should come before logging (so logs show compressed size), and security headers should be outermost (so they're always added, even if other middleware fails)

**The Transform trait**:

Custom middleware in Actix Web implements two traits:
1. **`Transform`**: Factory that creates middleware instances
2. **`Service`**: The actual middleware logic that processes requests

**Why this pattern?**

Actix spawns multiple worker threads, each needing its own middleware instance. The `Transform` trait creates these instances efficiently.

**About TracingLogger**:

We're using `TracingLogger::default()` from Chapter 1, which integrates seamlessly with our tracing infrastructure. This middleware:
- Creates a tracing span for each HTTP request
- Captures method, path, status code, and duration automatically
- Integrates with distributed tracing systems
- Provides structured, contextual logging

This is what production applications use, and we started with it from day one!

Now let's create our first custom middleware:

**How**:

1. **Create middleware directory**:
   ```bash
   mkdir -p src/middleware
   touch src/middleware/mod.rs
   touch src/middleware/security_headers.rs
   ```

2. **Create `src/middleware/security_headers.rs`**:

```rust
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

/// Middleware to add security headers to all responses
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            let headers = res.headers_mut();

            // Prevent MIME type sniffing
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );

            // Prevent clickjacking
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );

            // Enable XSS filter
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );

            // Referrer policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                actix_web::http::header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Content Security Policy (basic policy, adjust for your needs)
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
                ),
            );

            Ok(res)
        })
    }
}
```

3. **Update `src/middleware/mod.rs`**:

```rust
pub mod security_headers;

pub use security_headers::SecurityHeaders;
```

4. **Update `src/lib.rs`**:

```rust
pub mod config;
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

### Step 4: Add CORS and Compression Middleware

**Why**: Enable cross-origin requests and compress responses for better performance.

**How**:

1. **Add dependencies to `Cargo.toml`**:

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

# Compression
actix-web-lab = { version = "0.20", features = ["compression"] }

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

# Configuration
dotenvy = "0.15"

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

2. **Update `.env`** to add CORS configuration:

```bash
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3737

# Application Configuration
APP_ENV=development

# Database Configuration
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2
DATABASE_CONNECT_TIMEOUT=30
DATABASE_IDLE_TIMEOUT=600

# CORS Configuration
CORS_ALLOWED_ORIGINS=*

# Logging Configuration
RUST_LOG=info,actix_web=debug,actix_memo_app=debug,sea_orm=debug
```

3. **Update `src/config/settings.rs`** to include CORS settings:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub cors: CorsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub env: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: String,
}

impl Settings {
    /// Load settings from environment variables
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let settings = Settings {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3737".to_string())
                    .parse()?,
            },
            app: AppConfig {
                env: std::env::var("APP_ENV")
                    .unwrap_or_else(|_| "development".to_string()),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set"),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()?,
                connect_timeout: std::env::var("DATABASE_CONNECT_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                idle_timeout: std::env::var("DATABASE_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()?,
            },
            cors: CorsConfig {
                allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "*".to_string()),
            },
        };

        Ok(settings)
    }
}
```

4. **Update `src/main.rs`** to add all middleware:

```rust
mod config;
mod entities;
mod error;
mod handlers;
mod middleware;
mod state;
mod utils;

use actix_cors::Cors;
use actix_web::{middleware::Compress, web, App, HttpServer};
use config::Settings;
use state::AppState;
use std::io;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize tracing for structured logging
    utils::tracing::init_tracing();

    tracing::info!("Starting Actix Memo Application");

    // Load configuration from environment
    let settings = Settings::load()
        .expect("Failed to load settings");

    tracing::info!(
        "Configuration loaded - Environment: {}, Server: {}:{}",
        settings.app.env,
        settings.server.host,
        settings.server.port
    );

    // Establish database connection
    let db = utils::database::establish_connection(&settings)
        .await
        .expect("Failed to connect to database");

    // Verify database connection
    utils::database::verify_connection(&db)
        .await
        .expect("Failed to verify database connection");

    tracing::info!("Database connection verified");

    // Create application state
    let app_state = AppState::new(settings.clone(), db.clone());
    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);

    tracing::info!("Starting HTTP server at {}", bind_address);

    // Create and run HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = if settings.cors.allowed_origins == "*" {
            Cors::permissive()
        } else {
            let mut cors = Cors::default();
            for origin in settings.cors.allowed_origins.split(',') {
                cors = cors.allowed_origin(origin.trim());
            }
            cors.allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .max_age(3600)
        };

        App::new()
            // Add application state
            .app_data(web::Data::new(app_state.clone()))
            // Share database connection separately for convenience
            .app_data(web::Data::new(db.clone()))
            // Add middleware (order matters!)
            .wrap(middleware::SecurityHeaders) // Security headers first
            .wrap(cors)                         // CORS second
            .wrap(Compress::default())          // Compression third
            .wrap(TracingLogger::default())     // Logging last to capture all
            // Register routes
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/ready", web::get().to(handlers::health::ready))
            .route("/error", web::get().to(handlers::health::trigger_error))
            // Welcome route
            .route("/", web::get().to(welcome))
    })
    .bind(&bind_address)?
    .run()
    .await
}

/// Welcome endpoint
async fn welcome() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body(
        "Welcome to Actix Memo App! Try /health, /ready, or /error"
    )
}
```

**Verify**:
```bash
cargo build
```
Should compile without errors.

---

### Step 5: Test Error Handling and Middleware

**Why**: Verify all components work together correctly.

**How**:

1. **Run the application**:
   ```bash
   cargo run
   ```

2. **Test error handling**:

   ```bash
   # Test internal error
   curl -v http://localhost:3737/error

   # Expected response:
   # HTTP/1.1 500 Internal Server Error
   # {"error":{"message":"An internal server error occurred","status":500}}
   ```

3. **Test security headers**:

   ```bash
   curl -v http://localhost:3737/health
   ```

   Look for headers in response:
   ```
   x-content-type-options: nosniff
   x-frame-options: DENY
   x-xss-protection: 1; mode=block
   referrer-policy: strict-origin-when-cross-origin
   content-security-policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'
   ```

4. **Test CORS**:

   ```bash
   curl -v -H "Origin: http://example.com" http://localhost:3737/health
   ```

   Look for CORS headers:
   ```
   access-control-allow-origin: *
   ```

5. **Test compression**:

   ```bash
   curl -v -H "Accept-Encoding: gzip" http://localhost:3737/health
   ```

   Look for:
   ```
   content-encoding: gzip
   ```

**Verify**:
All tests should return appropriate responses with correct headers.

---

## Checkpoint

Run these commands to verify everything is working:

```bash
# Build should succeed
cargo build

# Application should start
cargo run
```

In another terminal:

```bash
# Test endpoints
curl http://localhost:3737/health
curl http://localhost:3737/error

# Test with verbose output to see headers
curl -v http://localhost:3737/health

# Test CORS
curl -v -H "Origin: http://example.com" http://localhost:3737/health

# Test compression
curl -v -H "Accept-Encoding: gzip" http://localhost:3737/health
```

### Expected Results

- Error endpoint returns 500 with JSON error message
- Health endpoint includes security headers
- CORS headers present when Origin header sent
- Responses compressed when Accept-Encoding header sent
- All endpoints work correctly

---

## Common Issues and Solutions

### Issue: Middleware not applying headers

**Symptoms**: Security headers missing from responses

**Cause**: Middleware order or middleware not registered

**Solution**:
```rust
// Ensure SecurityHeaders is wrapped in App
App::new()
    .wrap(middleware::SecurityHeaders)
    // ... other middleware and routes
```

Middleware order matters! Earlier middleware wraps later ones.

---

### Issue: CORS preflight requests failing

**Symptoms**: OPTIONS requests return 404 or wrong headers

**Cause**: CORS middleware not configured correctly

**Solution**:
```rust
let cors = Cors::default()
    .allowed_origin("http://localhost:3000")
    .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
    .allowed_headers(vec![
        actix_web::http::header::AUTHORIZATION,
        actix_web::http::header::ACCEPT,
        actix_web::http::header::CONTENT_TYPE,
    ])
    .max_age(3600);
```

---

### Issue: Compilation errors with middleware

**Symptoms**: Complex type errors in middleware implementation

**Cause**: Incorrect trait bounds or future types

**Solution**:
```bash
# Ensure actix-web version is compatible
cargo update actix-web

# Check that all middleware examples match your actix-web version
cargo clean
cargo build
```

---

### Issue: Error responses not formatted correctly

**Symptoms**: Plain text errors instead of JSON

**Cause**: Missing `error_response` implementation

**Solution**:
Ensure `ResponseError` trait is fully implemented:
```rust
impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode { /* ... */ }
    fn error_response(&self) -> HttpResponse { /* ... */ }
}
```

---

## Code Review

### Key Design Principles Demonstrated
- **Centralized error taxonomy**: `AppError` owns every failure mode (database, validation, auth, internal) so handlers propagate a single type.
- **Separation of concerns**: Error serialization (`ResponseError`) and logging happen in one place, keeping handlers focused on business logic.
- **Layered middleware ordering**: Security headers, CORS, compression, logging, and tracing execute in a deliberate sequence to guarantee safe defaults.
- **Fail-safe security posture**: Default denial CORS settings and strict headers block common exploits without extra work in handlers.

### Architecture Benefits
- **Consistent HTTP contract**: Clients always receive the same JSON error structure, simplifying front-end integrations.
- **Improved observability**: Tracing spans and structured logging capture the root cause even when responses hide internals.
- **Composable cross-cutting concerns**: Middleware lives in reusable modules, so future services can adopt the same policies.
- **Operational resilience**: Mapping database errors to 500s while preserving context in logs makes production incidents easier to debug.

### Complete Error Handling & Middleware Structure
```rust
impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Database(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        // ... existing logging + JSON payload construction ...
    }
}
```

```rust
HttpServer::new(move || {
    let cors = Cors::default()
        .allowed_origin_fn(|origin, _req_head| origin.as_bytes().starts_with(b"http://localhost"))
        .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
        // ... existing policy configuration ...
        ;

    App::new()
        .app_data(app_state.clone())
        .wrap(TracingLogger::default())
        .wrap(SecurityHeaders)
        .wrap(cors)
        .wrap(Compress::default())
        // ... existing routes ...
})
```

## Testing Error Handling

Create a simple test to verify error handling:

1. **Create `tests/error_tests.rs`**:

```rust
use actix_web::{test, App, web};
use actix_memo_app::error::AppError;
use actix_memo_app::handlers;

#[actix_web::test]
async fn test_error_response() {
    let app = test::init_service(
        App::new().route("/error", web::get().to(handlers::health::trigger_error))
    ).await;

    let req = test::TestRequest::get()
        .uri("/error")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return 500 status
    assert_eq!(resp.status(), 500);

    // Response should be JSON
    assert!(resp.headers().get("content-type").unwrap().to_str().unwrap().contains("json"));
}
```

2. **Run tests**:
   ```bash
   cargo test
   ```

---

## Refactoring with `thiserror` (Production Approach)

Now that you understand how error handling works under the hood, let's refactor to use `thiserror` - the industry-standard approach used in the production codebase.

### Why We Learned the Manual Way First

Understanding the manual implementation helps you:
- **Know what happens** behind the macros
- **Debug issues** when things go wrong
- **Make informed decisions** about error design
- **Appreciate the tools** that simplify your work

### Why Use `thiserror` in Production

`thiserror` reduces boilerplate by auto-generating:
- `Display` implementation from `#[error(...)]` attributes
- `Error` trait implementation
- `From` trait conversions with `#[from]`
- Less code = fewer bugs

### Step-by-Step Refactoring

**1. Add `thiserror` dependency to `Cargo.toml`**:

```toml
[dependencies]
# ... existing dependencies ...

# Error handling
thiserror = "1.0"
```

**2. Refactor `src/error/app_error.rs`**:

```rust
use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use sea_orm::DbErr;
use thiserror::Error;

/// Main application error type
#[derive(Debug, Error)]
pub enum AppError {
    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    /// Resource not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Validation errors (bad input)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Internal server errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Unauthorized access
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        // Log internal errors but don't expose details to client
        if matches!(self, AppError::Database(_) | AppError::Internal(_)) {
            tracing::error!("Internal error occurred: {}", self);
        }

        // Create error response body
        let error_message = match self {
            // Don't expose internal error details
            AppError::Database(_) => "A database error occurred".to_string(),
            AppError::Internal(_) => "An internal server error occurred".to_string(),
            // Safe to expose these messages
            AppError::NotFound(msg) => msg.clone(),
            AppError::Validation(msg) => msg.clone(),
            AppError::Unauthorized(msg) => msg.clone(),
        };

        HttpResponse::build(status)
            .insert_header(ContentType::json())
            .json(serde_json::json!({
                "error": {
                    "message": error_message,
                    "status": status.as_u16(),
                }
            }))
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
```

**3. Build and verify**:

```bash
cargo build
```

### Before and After Comparison

**Before (Manual - ~80 lines)**:
```rust
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            // ... more matches
        }
    }
}

impl std::error::Error for AppError {}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(_) => AppError::NotFound("Record not found".to_string()),
            _ => AppError::Database(err),
        }
    }
}
```

**After (with `thiserror` - ~35 lines)**:
```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),  // Auto-generates From<DbErr>

    #[error("Not found: {0}")]
    NotFound(String),

    // ... other variants
}
```

### What We Gained

✅ **Less code**: ~50% reduction in boilerplate
✅ **Auto-generated traits**: `Display` and `Error` automatically implemented
✅ **Auto-conversions**: `#[from]` generates `From` trait implementations
✅ **Better error messages**: Consistent formatting via `#[error(...)]`
✅ **Maintainability**: Less code to maintain and test
✅ **Industry standard**: Used in production Rust codebases

### What We Still Control

We still manually implement `ResponseError` because:
- It's Actix-specific (not a standard trait)
- We need custom HTTP status code mapping
- We want to control error response format
- We need to hide internal error details for security

### Production Code Note

The actual `actix-web-template` production codebase uses exactly this pattern. You can see it in `src/error/app_error.rs`:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
    // ...
}
```

This is the approach you should use in real projects!

---

## Summary

Congratulations! You've built a production-ready error handling system and middleware stack. You now have:

1. **Custom error type** (AppError enum) for type-safe error handling
2. **ResponseError implementation** that maps errors to HTTP status codes
3. **Error response formatting** with JSON and safe error messages
4. **Security headers middleware** to protect against common vulnerabilities
5. **CORS middleware** for cross-origin requests
6. **Compression middleware** for improved performance
7. **Proper middleware ordering** for optimal request/response flow
8. **Error logging** without exposing internal details

### Key Takeaways

- **Custom error types** provide type-safe, centralized error handling
- **ResponseError trait** enables automatic error-to-response conversion
- **Middleware** implements cross-cutting concerns
- **Security headers** protect against XSS, clickjacking, and MIME sniffing
- **CORS** enables controlled cross-origin access
- **Compression** reduces bandwidth and improves performance
- **Error messages** should be safe for users, detailed in logs

### Architecture So Far

```
┌─────────────────────────────────────┐
│        HTTP Requests                │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Security Headers Middleware        │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  CORS Middleware                    │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Compression Middleware             │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  TracingLogger Middleware           │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Handlers (return Result<T, AppError>)│
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  AppError → HTTP Response           │
│  (via ResponseError trait)          │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  HTTP Response with Headers         │
└─────────────────────────────────────┘
```

---

## Next Steps

### Required: Chapter 4 - Enhanced Health Checks and Readiness Probes

You'll enrich the health check endpoints with dependency awareness, readiness and liveness semantics, and structured diagnostics that operations teams rely on. Expect to reuse the error types from this chapter to surface dependency failures cleanly.

### Optional Exercises

1. **Challenge**: Introduce a custom middleware that injects a correlation ID header when one is missing.
2. **Challenge**: Add conversion implementations (`From`) for a new error source, such as Redis or an external API client, to practice extending `AppError`.
3. **Challenge**: Benchmark gzip versus brotli middleware locally and document when each compression strategy makes sense.

---

## Additional Resources

### Error Handling
- [Actix Web Error Handling](https://actix.rs/docs/errors/) - Official guide
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) - The Book
- [thiserror crate](https://docs.rs/thiserror/) - Derive Error trait

### Middleware
- [Actix Web Middleware](https://actix.rs/docs/middleware/) - Official docs
- [actix-cors](https://docs.rs/actix-cors/) - CORS middleware
- [actix-web Compress](https://docs.rs/actix-web/latest/actix_web/middleware/struct.Compress.html) - Compression

### Security
- [OWASP Security Headers](https://owasp.org/www-project-secure-headers/) - Best practices
- [Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP) - MDN docs
- [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) - MDN guide

### Testing
- [Actix Web Testing](https://actix.rs/docs/testing/) - Official guide
- [Integration Testing](https://doc.rust-lang.org/book/ch11-03-test-organization.html) - The Book

---

**Ready to add comprehensive health checks? Let's move on to [Chapter 4: Health Checks and Monitoring](chapter-04.md)!**
