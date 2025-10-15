# Chapter 1: Core Application Setup

## Overview

In this chapter, we'll create the foundational structure of our Actix Web application. You'll build a minimal but functional web server with environment-based configuration, application state management, and your first HTTP endpoint.

By the end of this chapter, you'll have a running web server that responds to HTTP requests and demonstrates the core patterns used throughout the rest of the tutorial.

> **Note on Tutorial Approach**: This tutorial builds the application incrementally, starting simple and adding features chapter by chapter. The actual codebase in this repository is already at Stage 18 (fully production-ready) with advanced features like OpenTelemetry tracing, Prometheus metrics, rate limiting, and comprehensive security. Each chapter will note where the tutorial code differs from the production implementation, allowing you to learn concepts step-by-step while seeing how they're applied in a real production system.

## Prerequisites

### Completed

- Chapter 0: Prerequisites and Environment Setup

### Required Knowledge

- Basic Rust project structure (Cargo.toml, src/ directory)
- Understanding of Rust modules and the `mod` keyword
- Familiarity with structs, enums, and basic traits
- Basic understanding of HTTP and web servers

### Required Software

- Rust 1.75+ installed and working
- Code editor configured
- PostgreSQL running (for future chapters)

## Learning Objectives

By completing this chapter, you will:

1. Create a new Rust project using Cargo
2. Understand the Actix Web application structure
3. Implement environment-based configuration loading
4. Set up application state with the State pattern
5. Create and configure an HTTP server
6. Build your first route handler
7. Understand the Actix Web request lifecycle
8. Test your server is working correctly

## Concepts Covered

### Actix Web Architecture

Actix Web is built on top of the Actix actor framework and uses the Tokio async runtime. Key architectural concepts:

**HttpServer**: The main server struct that binds to a TCP socket and listens for incoming connections. It can spawn multiple worker threads, each running an independent copy of your application.

**App Factory Pattern**: Instead of passing a single `App` instance, you provide a factory function that creates new `App` instances. This is necessary because Actix spawns multiple workers, and each needs its own app instance.

**Application State**: Shared data accessible to all request handlers. Wrapped in `web::Data<T>` for thread-safe access via Arc (Atomic Reference Counting).

**Handlers**: Async functions that take extractors (request data) as parameters and return something that can be converted into an HTTP response.

### Configuration Management

We follow the **12-Factor App** methodology:

- **Separation of Config and Code**: Configuration comes from environment variables, not hardcoded values
- **Environment Parity**: Same code runs in dev, staging, and production with different config
- **No Secrets in Code**: Sensitive data (database credentials, API keys) never committed to version control

### Application State Pattern

The State pattern in Actix Web allows sharing data across request handlers:

```rust
// Application state
struct AppState {
    db: DatabaseConnection,  // Database connection pool
    settings: Settings,       // Configuration
}

// Accessed in handlers via web::Data<AppState>
async fn handler(state: web::Data<AppState>) -> impl Responder {
    // Access state.db, state.settings
}
```

State is wrapped in `Arc` automatically by `web::Data`, making it cheap to clone and share across threads.

## Step-by-Step Instructions

### Step 1: Create the Rust Project

**Why**: We need a new Cargo project to hold our application code.

**How**:

1. **Navigate to your projects directory**:
   ```bash
   cd ~/projects/actix-web-tutorial
   ```

2. **Create a new binary project**:
   ```bash
   cargo new actix-memo-app --bin
   cd actix-memo-app
   ```

   This creates:
   - `Cargo.toml` - Project manifest and dependencies
   - `src/main.rs` - Application entry point
   - `.gitignore` - Git ignore file

3. **Verify the project structure**:
   ```bash
   ls -la
   ```

   Expected output:
   ```
   Cargo.toml
   src/
   .gitignore
   ```

**Verify**:
```bash
cargo build
cargo run
```

Should compile and print "Hello, world!"

---

### Step 2: Add Dependencies

**Why**: We need to add Actix Web and supporting crates to our project.

**How**:

1. **Open `Cargo.toml`** in your editor

2. **Replace the dependencies section** with:

```toml
[package]
name = "actix-memo-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
actix-web = "4.4"

# Async runtime
tokio = { version = "1.35", features = ["macros", "rt-multi-thread"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
dotenvy = "0.15"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

3. **Build to download dependencies**:
   ```bash
   cargo build
   ```

   This will download and compile all dependencies. This may take a few minutes on first run.

**Verify**:
```bash
cargo build
```

Should complete without errors. You should see compiled dependencies in `target/debug/deps/`.

---

### Step 3: Create Project Structure

**Why**: Organizing code into modules makes it maintainable and follows Rust best practices.

**How**:

1. **Create the directory structure**:
   ```bash
   mkdir -p src/config
   mkdir -p src/handlers
   mkdir -p src/utils
   ```

2. **Create module files**:
   ```bash
   touch src/config/mod.rs
   touch src/config/settings.rs
   touch src/handlers/mod.rs
   touch src/handlers/health.rs
   touch src/utils/mod.rs
   touch src/utils/tracing.rs
   touch src/state.rs
   ```

3. **Verify structure**:
   ```bash
   tree src/
   ```

   Expected output:
   ```
   src/
   ├── config/
   │   ├── mod.rs
   │   └── settings.rs
   ├── handlers/
   │   ├── mod.rs
   │   └── health.rs
   ├── utils/
   │   ├── mod.rs
   │   └── tracing.rs
   ├── state.rs
   └── main.rs
   ```

**Verify**:
```bash
ls -R src/
```

Should show all created directories and files.

---

### Step 4: Implement Configuration Module

**Why**: Load settings from environment variables in a type-safe way.

**How**:

1. **Create `src/config/settings.rs`**:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub app: AppSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    pub env: String,
}

impl Settings {
    /// Load settings from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        let _ = dotenvy::dotenv();

        let settings = Settings {
            server: ServerSettings {
                host: std::env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3737".to_string())
                    .parse()?,
            },
            app: AppSettings {
                env: std::env::var("APP_ENV")
                    .unwrap_or_else(|_| "development".to_string()),
            },
        };

        Ok(settings)
    }
}
```

2. **Create `src/config/mod.rs`**:

```rust
mod settings;

pub use settings::Settings;
```

**Verify**:
```bash
cargo check
```

Should compile without errors.

---

### Step 5: Implement Application State

**Why**: Application state holds shared resources like configuration that handlers need to access.

**How**:

1. **Create `src/state.rs`**:

```rust
use crate::config::Settings;

/// Application state shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
}

impl AppState {
    /// Create new application state with the given settings
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
}
```

**Verify**:
```bash
cargo check
```

Should compile without errors.

---

### Step 6: Implement Tracing Setup

**Why**: Structured logging helps debug and monitor your application.

**How**:

1. **Create `src/utils/tracing.rs`**:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing subscriber for structured logging
pub fn init_tracing() {
    // Set default log level to info if not specified
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

2. **Create `src/utils/mod.rs`**:

```rust
pub mod tracing;
```

**Verify**:
```bash
cargo check
```

Should compile without errors.

---

### Step 7: Implement Health Check Handler

**Why**: A basic endpoint to verify the server is running.

**How**:

1. **Create `src/handlers/health.rs`**:

```rust
use actix_web::{HttpResponse, Responder};
use serde_json::json;

/// Simple health check endpoint
///
/// Returns JSON with status "ok" and HTTP 200
#[tracing::instrument]
pub async fn health_check() -> impl Responder {
    tracing::info!("Health check requested");

    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "actix-memo-app"
    }))
}
```

2. **Create `src/handlers/mod.rs`**:

```rust
pub mod health;
```

**Verify**:
```bash
cargo check
```

Should compile without errors.

---

### Step 8: Implement Main Application

**Why**: Wire everything together and start the HTTP server.

**How**:

1. **Replace `src/main.rs`** with:

```rust
mod config;
mod handlers;
mod state;
mod utils;

use actix_web::{middleware::Logger, web, App, HttpServer};
use config::Settings;
use state::AppState;
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize tracing for structured logging
    utils::tracing::init_tracing();

    tracing::info!("Starting Actix Memo Application");

    // Load configuration from environment
    let settings = Settings::from_env()
        .expect("Failed to load settings");

    tracing::info!(
        "Configuration loaded - Environment: {}, Server: {}:{}",
        settings.app.env,
        settings.server.host,
        settings.server.port
    );

    // Create application state
    let app_state = AppState::new(settings.clone());
    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);

    tracing::info!("Starting HTTP server at {}", bind_address);

    // Create and run HTTP server
    HttpServer::new(move || {
        App::new()
            // Add application state
            .app_data(web::Data::new(app_state.clone()))
            // Add request logging middleware
            .wrap(Logger::default())
            // Register routes
            .route("/health", web::get().to(handlers::health::health_check))
            // Welcome route
            .route("/", web::get().to(welcome))
    })
    .bind(&bind_address)?
    .run()
    .await
}

/// Welcome endpoint
async fn welcome() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body("Welcome to Actix Memo App! Try /health")
}
```

**Verify**:
```bash
cargo build
```

Should compile successfully.

---

### Step 9: Create Environment File

**Why**: Configure the application without changing code.

**How**:

1. **Create `.env` file** in project root:

```bash
cat > .env << 'EOF'
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3737

# Application Configuration
APP_ENV=development

# Logging Configuration
RUST_LOG=info,actix_web=debug,actix_memo_app=debug
EOF
```

2. **Ensure .env is in .gitignore**:

```bash
echo ".env" >> .gitignore
```

**Verify**:
```bash
cat .env
```

Should display your environment variables.

---

### Step 10: Run and Test the Application

**Why**: Verify everything works correctly.

**How**:

1. **Run the application**:
   ```bash
   cargo run
   ```

   Expected output:
   ```
   2025-01-15T10:00:00.000000Z  INFO actix_memo_app: Starting Actix Memo Application
   2025-01-15T10:00:00.000000Z  INFO actix_memo_app: Configuration loaded - Environment: development, Server: 127.0.0.1:3737
   2025-01-15T10:00:00.000000Z  INFO actix_memo_app: Starting HTTP server at 127.0.0.1:3737
   2025-01-15T10:00:00.000000Z  INFO actix_server::builder: Starting 8 workers
   2025-01-15T10:00:00.000000Z  INFO actix_server::server: Actix runtime found; starting in Actix runtime
   ```

2. **In another terminal, test the endpoints**:

   **Test welcome endpoint**:
   ```bash
   curl http://localhost:3737/
   ```

   Expected output:
   ```
   Welcome to Actix Memo App! Try /health
   ```

   **Test health check endpoint**:
   ```bash
   curl http://localhost:3737/health
   ```

   Expected output:
   ```json
   {
     "status": "ok",
     "service": "actix-memo-app"
   }
   ```

3. **Check the logs** in the first terminal:
   ```
   2025-01-15T10:01:00.000000Z  INFO actix_web::middleware::logger: 127.0.0.1 "GET /health HTTP/1.1" 200 45 "-" "curl/7.79.1" 0.000123
   ```

**Verify**:
Both curl commands should return successful responses, and you should see request logs in the server terminal.

---

## Checkpoint

Run these commands to verify everything is working:

```bash
# Build should succeed
cargo build

# Check should pass
cargo check

# Clippy should have no warnings
cargo clippy

# Run the server
cargo run
```

In another terminal:

```bash
# Test endpoints
curl http://localhost:3737/
curl http://localhost:3737/health

# Test with verbose output
curl -v http://localhost:3737/health
```

### Expected Results

- Server starts without errors
- Welcome endpoint returns text response
- Health endpoint returns JSON
- Request logs appear in server terminal
- Response includes proper headers

---

## Common Issues and Solutions

### Issue: Address already in use

**Symptoms**:
```
Error: Os { code: 48, kind: AddrInUse, message: "Address already in use" }
```

**Cause**: Port 3737 is already in use by another process

**Solution**:
```bash
# Find the process using port 3737
lsof -i :3737

# Kill the process (replace PID with actual process ID)
kill -9 <PID>

# Or change the port in .env
SERVER_PORT=3738
```

---

### Issue: Failed to load settings

**Symptoms**:
```
thread 'main' panicked at 'Failed to load settings: ParseIntError'
```

**Cause**: Invalid PORT value in .env file

**Solution**:
```bash
# Ensure SERVER_PORT is a valid number
echo "SERVER_PORT=3737" >> .env

# Verify .env file
cat .env
```

---

### Issue: No request logs appearing

**Symptoms**: Requests work but no logs are printed

**Cause**: RUST_LOG environment variable not set correctly

**Solution**:
```bash
# Update .env file
echo "RUST_LOG=info,actix_web=debug" >> .env

# Restart the server
cargo run
```

---

### Issue: Compilation errors with actix-web

**Symptoms**: Type errors or missing trait implementations

**Cause**: Version mismatch or corrupted build cache

**Solution**:
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build
```

---

## Code Review

Let's review the key components we've built:

### Application Entry Point (main.rs)

```rust
#[actix_web::main]
async fn main() -> io::Result<()> {
    // 1. Initialize logging
    utils::tracing::init_tracing();

    // 2. Load configuration
    let settings = Settings::from_env()
        .expect("Failed to load settings");

    // 3. Create application state
    let app_state = AppState::new(settings.clone());

    // 4. Start HTTP server with factory pattern
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/", web::get().to(welcome))
    })
    .bind(&bind_address)?
    .run()
    .await
}
```

**Key points**:
- `#[actix_web::main]` sets up the Tokio async runtime
- `HttpServer::new` takes a factory closure that creates `App` instances
- `app_state.clone()` is cheap because AppState contains Arc-wrapped data
- `.bind()` binds to TCP socket, `.run()` starts the event loop

### Configuration (settings.rs)

Type-safe configuration loading with sensible defaults:

```rust
pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();  // Load .env if it exists

    let settings = Settings {
        server: ServerSettings {
            host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            // ... defaults for missing vars
        },
        // ...
    };

    Ok(settings)
}
```

**Key points**:
- `dotenvy::dotenv()` loads .env file
- `unwrap_or_else` provides default values
- Returns `Result` for error handling

### Handler (health.rs)

Simple async handler returning JSON:

```rust
#[tracing::instrument]
pub async fn health_check() -> impl Responder {
    tracing::info!("Health check requested");

    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "actix-memo-app"
    }))
}
```

**Key points**:
- `async fn` for asynchronous execution
- `#[tracing::instrument]` adds automatic span tracking
- `impl Responder` allows returning various response types
- `HttpResponse::Ok()` creates 200 response
- `.json()` serializes to JSON with proper Content-Type

---

## Testing

### Manual Testing

We've already tested manually with curl. Let's add a simple automated test:

1. **Create `tests/` directory**:
   ```bash
   mkdir tests
   ```

2. **Create `tests/integration_test.rs`**:

```rust
use actix_web::{test, App};

#[actix_web::test]
async fn test_health_check() {
    // Import the handler
    use actix_memo_app::handlers::health::health_check;

    // Create a test service
    let app = test::init_service(
        App::new().route("/health", actix_web::web::get().to(health_check))
    ).await;

    // Create a test request
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    // Call the service
    let resp = test::call_service(&app, req).await;

    // Assert response
    assert!(resp.status().is_success());
}
```

3. **Run tests**:
   ```bash
   cargo test
   ```

   Expected output:
   ```
   running 1 test
   test test_health_check ... ok
   ```

---

## Understanding the Request Lifecycle

When a request hits your server, here's what happens:

```
1. TCP Connection
   ↓
2. HttpServer receives request
   ↓
3. Request routed to App instance
   ↓
4. Middleware (Logger) processes request
   ↓
5. Route matcher finds handler
   ↓
6. Extractors extract data (none in our simple handler)
   ↓
7. Handler executes (health_check)
   ↓
8. Response returned
   ↓
9. Middleware processes response
   ↓
10. Response sent to client
```

### Extractors

Extractors pull data from the request:

- `web::Data<AppState>` - Application state
- `web::Json<T>` - JSON request body
- `web::Path<T>` - URL path parameters
- `web::Query<T>` - Query string parameters
- `HttpRequest` - Raw request object

We'll use these in later chapters.

---

## Summary

Congratulations! You've built a working Actix Web application. You now have:

1. **Structured project** with modules for config, handlers, utils
2. **Environment-based configuration** loading from .env file
3. **Application state** pattern for sharing data
4. **Structured logging** with tracing
5. **HTTP server** running on port 3737
6. **Health check endpoint** returning JSON
7. **Welcome endpoint** demonstrating routing
8. **Request logging** middleware

### Key Takeaways

- **HttpServer factory pattern**: Each worker gets its own App instance
- **Application state** is shared via `web::Data<T>` and Arc
- **Handlers** are async functions returning `impl Responder`
- **Configuration** comes from environment variables, not hardcoded
- **Tracing** provides structured, filterable logging
- **Middleware** wraps handlers for cross-cutting concerns

### Architecture So Far

```
┌─────────────────────────────────────┐
│        HTTP Requests                │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  HttpServer (multiple workers)      │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  App Factory (creates App instances)│
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Middleware (Logger)                │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Route Matching                     │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Handlers (health_check, welcome)   │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Response                           │
└─────────────────────────────────────┘
```

---

## What's Next

In **Chapter 2: Database Integration with SeaORM**, we'll:
- Add database connection to our application state
- Create database migrations with SeaORM CLI
- Generate entity models from database schema
- Implement connection pooling
- Verify database connectivity from our app

You'll learn how to:
- Work with PostgreSQL using SeaORM
- Manage schema changes with migrations
- Configure connection pools for production
- Handle database errors gracefully

---

## Additional Resources

### Actix Web Documentation
- [Actix Web Docs](https://actix.rs/docs/) - Official documentation
- [Actix Examples](https://github.com/actix/examples) - Official example projects
- [API Reference](https://docs.rs/actix-web/latest/actix_web/) - Complete API docs

### Related Crates
- [Tokio](https://tokio.rs/) - Async runtime
- [Serde](https://serde.rs/) - Serialization framework
- [Tracing](https://docs.rs/tracing/) - Application-level tracing

### Tutorials and Guides
- [Actix Web Tutorial](https://actix.rs/docs/getting-started/) - Official getting started
- [Rust Async Book](https://rust-lang.github.io/async-book/) - Understanding async/await
- [12-Factor App](https://12factor.net/) - Application configuration best practices

### Community
- [Actix Discord](https://discord.gg/actix) - Real-time help
- [Rust Users Forum](https://users.rust-lang.org/) - Ask questions
- [r/rust](https://www.reddit.com/r/rust/) - Rust community

---

**Ready to add database functionality? Let's move on to [Chapter 2: Database Integration with SeaORM](chapter-02.md)!**