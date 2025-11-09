# Chapter 4: Enhanced Health Checks and Readiness Probes

## Overview

In this chapter, we'll enhance our basic health check from previous chapters to create comprehensive health and readiness endpoints suitable for production deployments, including Kubernetes environments. You'll learn how to monitor database connections, implement proper health check patterns, and prepare your application for orchestrated deployments.

By the end of this chapter, you'll have production-grade health monitoring that provides detailed status information and integrates seamlessly with container orchestration platforms.

> **Note on Tutorial Approach**: This chapter focuses on application-level health checks and readiness probes. The production codebase extends these with OpenTelemetry tracing spans and Prometheus metrics (covered in Chapter 17: Observability Stack). We'll build the foundation here that can be enhanced with full observability later.

## Prerequisites

### Completed

- Chapter 0: Prerequisites and Environment Setup
- Chapter 1: Core Application Setup
- Chapter 2: Database Integration with SeaORM
- Chapter 3: Error Handling and Middleware

### Required Knowledge

- Understanding of health check concepts
- Familiarity with Kubernetes liveness/readiness probes (helpful but not required)
- Basic async/await patterns
- HTTP status codes

### Required Software

- Working Actix Web application from Chapter 3
- PostgreSQL running

## Learning Objectives

By completing this chapter, you will:

1. Understand the difference between health and readiness checks
2. Implement comprehensive health checks with component status
3. Create Kubernetes-compatible liveness and readiness probes
4. Add startup probes for slow-starting applications
5. Monitor database connection health
6. Return structured health check responses
7. Handle partial system failures gracefully
8. Implement health check best practices

## Concepts Covered

### Health Check Types

Different health checks serve different purposes:

**Liveness Probe**:
- Question: "Is the application alive?"
- Purpose: Detect deadlocked or crashed applications
- Action on failure: Restart the container
- Should check: Core application health, not external dependencies

**Readiness Probe**:
- Question: "Is the application ready to serve traffic?"
- Purpose: Control traffic routing
- Action on failure: Stop sending traffic (but don't restart)
- Should check: Application + critical dependencies (database, cache)

**Startup Probe**:
- Question: "Has the application finished starting?"
- Purpose: Give slow-starting apps more time
- Action on failure: Restart if startup takes too long
- Used once at startup, then switches to liveness/readiness

### Health Check Best Practices

1. **Fast Response Time**: Health checks should complete in < 1 second
2. **No Side Effects**: Health checks shouldn't modify state
3. **Cached Results**: For expensive checks, cache results for a few seconds
4. **Graceful Degradation**: Partial failures should return appropriate status
5. **Detailed Information**: Include status of each component checked
6. **Proper Status Codes**:
   - 200 OK: Healthy
   - 503 Service Unavailable: Unhealthy
   - 429 Too Many Requests: Health check being called too frequently

### Health Check Response Format

Standard health check response:

```json
{
  "status": "ok" | "degraded" | "unhealthy",
  "version": "0.1.0",
  "uptime": 12345,
  "checks": {
    "database": {
      "status": "ok",
      "response_time_ms": 5
    },
    "memory": {
      "status": "ok",
      "used_mb": 256,
      "available_mb": 512
    }
  }
}
```

## Step-by-Step Instructions

### Step 1: Create Health Check Data Structures

**Why**: Define structured types for health check responses.

**How**:

1. **Create `src/handlers/health_models.rs`**:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All checks passed
    Ok,
    /// Some non-critical checks failed
    Degraded,
    /// Critical checks failed
    Unhealthy,
}

impl HealthStatus {
    /// Convert to HTTP status code
    pub fn to_status_code(&self) -> actix_web::http::StatusCode {
        match self {
            HealthStatus::Ok => actix_web::http::StatusCode::OK,
            HealthStatus::Degraded => actix_web::http::StatusCode::OK,
            HealthStatus::Unhealthy => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

/// Individual component health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentStatus {
    Ok,
    Degraded,
    Unhealthy,
}

/// Health check for an individual component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: ComponentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
}

impl ComponentHealth {
    pub fn ok() -> Self {
        Self {
            status: ComponentStatus::Ok,
            message: None,
            response_time_ms: None,
        }
    }

    pub fn ok_with_time(response_time_ms: u64) -> Self {
        Self {
            status: ComponentStatus::Ok,
            message: None,
            response_time_ms: Some(response_time_ms),
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: ComponentStatus::Degraded,
            message: Some(message.into()),
            response_time_ms: None,
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: ComponentStatus::Unhealthy,
            message: Some(message.into()),
            response_time_ms: None,
        }
    }
}

/// Complete health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HashMap<String, ComponentHealth>,
}

impl HealthResponse {
    /// Determine overall status from component checks
    pub fn determine_status(checks: &HashMap<String, ComponentHealth>) -> HealthStatus {
        let has_unhealthy = checks.values().any(|c| c.status == ComponentStatus::Unhealthy);
        let has_degraded = checks.values().any(|c| c.status == ComponentStatus::Degraded);

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Ok
        }
    }
}
```

2. **Update `src/handlers/mod.rs`**:

```rust
pub mod health;
pub mod health_models;

pub use health_models::{HealthResponse, HealthStatus, ComponentHealth, ComponentStatus};
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 2: Add Application Startup Time Tracking

**Why**: Track application uptime for health check responses.

**How**:

1. **Update `src/state.rs`** to track startup time:

```rust
use crate::config::Settings;
use sea_orm::DatabaseConnection;
use std::time::Instant;

/// Application state shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub db: DatabaseConnection,
    pub start_time: Instant,
}

impl AppState {
    /// Create new application state with the given settings and database connection
    pub fn new(settings: Settings, db: DatabaseConnection) -> Self {
        Self {
            settings,
            db,
            start_time: Instant::now(),
        }
    }

    /// Get application uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 3: Create Enhanced Health Check Handler

**Why**: Implement comprehensive health checks with detailed component status.

**How**:

1. **Replace `src/handlers/health.rs`** with enhanced version:

```rust
use actix_web::{web, HttpResponse, Responder};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::time::Instant;

use crate::error::{AppError, AppResult};
use crate::handlers::health_models::{ComponentHealth, HealthResponse, HealthStatus};
use crate::state::AppState;

/// Comprehensive health check endpoint
///
/// Checks all critical components and returns detailed status.
/// Returns 200 for healthy/degraded, 503 for unhealthy.
#[tracing::instrument(skip(state))]
pub async fn health_check(state: web::Data<AppState>) -> impl Responder {
    tracing::debug!("Health check requested");

    let mut checks = HashMap::new();

    // Check database
    let db_check = check_database(&state.db).await;
    checks.insert("database".to_string(), db_check);

    // Determine overall status
    let overall_status = HealthResponse::determine_status(&checks);

    let response = HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.uptime_seconds(),
        checks,
    };

    let status_code = overall_status.to_status_code();

    HttpResponse::build(status_code).json(response)
}

/// Liveness probe - checks if application is alive
///
/// This should NOT check external dependencies.
/// Used by Kubernetes to determine if container should be restarted.
#[tracing::instrument]
pub async fn liveness() -> impl Responder {
    tracing::debug!("Liveness probe requested");

    // Simple alive check - just return OK
    // Don't check database or other dependencies
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive"
    }))
}

/// Readiness probe - checks if application is ready to serve traffic
///
/// This SHOULD check external dependencies.
/// Used by Kubernetes to determine if traffic should be routed to this instance.
#[tracing::instrument(skip(state))]
pub async fn readiness(state: web::Data<AppState>) -> impl Responder {
    tracing::debug!("Readiness probe requested");

    // Check database connectivity
    let db_healthy = check_database_quick(&state.db).await;

    if db_healthy {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ready"
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "reason": "database_unavailable"
        }))
    }
}

/// Startup probe - checks if application has finished starting
///
/// Used during startup to give application more time before liveness checks begin.
#[tracing::instrument(skip(state))]
pub async fn startup(state: web::Data<AppState>) -> impl Responder {
    tracing::debug!("Startup probe requested");

    // Check if database is connected
    let db_healthy = check_database_quick(&state.db).await;

    if db_healthy {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "started"
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "starting",
            "reason": "waiting_for_database"
        }))
    }
}

/// Example error endpoint for testing error handling
#[tracing::instrument]
pub async fn trigger_error() -> AppResult<impl Responder> {
    Err(AppError::Internal("This is a test error".to_string()))
}

/// Check database connectivity with detailed response
async fn check_database(db: &DatabaseConnection) -> ComponentHealth {
    let start = Instant::now();

    match crate::utils::database::verify_connection(db).await {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ComponentHealth::ok_with_time(elapsed)
        }
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            ComponentHealth::unhealthy(format!("Database error: {}", e))
        }
    }
}

/// Quick database check without detailed timing
async fn check_database_quick(db: &DatabaseConnection) -> bool {
    crate::utils::database::verify_connection(db).await.is_ok()
}
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 4: Update Routes in Main Application

**Why**: Register the new health check endpoints.

**How**:

1. **Update `src/main.rs`** to add new routes:

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

    tracing::info!("Starting Actix Memo Application v{}", env!("CARGO_PKG_VERSION"));

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
            .wrap(TracingLogger::default())            // Logging last to capture all
            // Health check routes
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/health/live", web::get().to(handlers::health::liveness))
            .route("/health/ready", web::get().to(handlers::health::readiness))
            .route("/health/startup", web::get().to(handlers::health::startup))
            // Utility routes
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
    actix_web::HttpResponse::Ok().body(format!(
        "Welcome to Actix Memo App v{}! Available endpoints:\n\
        - GET /health - Comprehensive health check\n\
        - GET /health/live - Liveness probe\n\
        - GET /health/ready - Readiness probe\n\
        - GET /health/startup - Startup probe\n\
        - GET /error - Test error handling",
        env!("CARGO_PKG_VERSION")
    ))
}
```

**Verify**:
```bash
cargo build
```
Should compile without errors.

---

### Step 5: Test Health Check Endpoints

**Why**: Verify all health check endpoints work correctly.

**How**:

1. **Start PostgreSQL** (if not already running):
   ```bash
   docker compose up -d postgres
   ```

2. **Run the application**:
   ```bash
   cargo run
   ```

3. **Test comprehensive health check**:

   ```bash
   curl http://localhost:3737/health | jq
   ```

   Expected output:
   ```json
   {
     "status": "ok",
     "version": "0.1.0",
     "uptime_seconds": 42,
     "checks": {
       "database": {
         "status": "ok",
         "response_time_ms": 5
       }
     }
   }
   ```

4. **Test liveness probe**:

   ```bash
   curl http://localhost:3737/health/live | jq
   ```

   Expected output:
   ```json
   {
     "status": "alive"
   }
   ```

5. **Test readiness probe**:

   ```bash
   curl http://localhost:3737/health/ready | jq
   ```

   Expected output:
   ```json
   {
     "status": "ready"
   }
   ```

6. **Test startup probe**:

   ```bash
   curl http://localhost:3737/health/startup | jq
   ```

   Expected output:
   ```json
   {
     "status": "started"
   }
   ```

7. **Test unhealthy state** (stop PostgreSQL):

   ```bash
   # Stop database
   docker compose stop postgres

   # Wait a moment for connection pool to detect failure
   sleep 2

   # Test health check - should show unhealthy
   curl http://localhost:3737/health | jq
   ```

   Expected output:
   ```json
   {
     "status": "unhealthy",
     "version": "0.1.0",
     "uptime_seconds": 123,
     "checks": {
       "database": {
         "status": "unhealthy",
         "message": "Database error: ..."
       }
     }
   }
   ```

   Status code should be 503 (Service Unavailable).

8. **Restart database** and verify recovery:

   ```bash
   docker compose start postgres
   sleep 2
   curl http://localhost:3737/health | jq
   ```

   Should return to healthy status.

**Verify**:
All endpoints should return appropriate responses based on system state.

---

### Step 6: Create Kubernetes Deployment Configuration (Optional)

**Why**: Demonstrate how to use health checks in Kubernetes deployments.

**How**:

1. **Create `k8s/deployment.yaml`**:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: actix-memo-app
  labels:
    app: actix-memo-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: actix-memo-app
  template:
    metadata:
      labels:
        app: actix-memo-app
    spec:
      containers:
      - name: app
        image: actix-memo-app:latest
        ports:
        - containerPort: 3737
          name: http
        env:
        - name: SERVER_HOST
          value: "0.0.0.0"
        - name: SERVER_PORT
          value: "3737"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-secret
              key: url

        # Startup probe - gives app time to start
        # Checks every 10s, fails after 30 attempts (5 minutes)
        startupProbe:
          httpGet:
            path: /health/startup
            port: 3737
          initialDelaySeconds: 5
          periodSeconds: 10
          timeoutSeconds: 5
          successThreshold: 1
          failureThreshold: 30

        # Liveness probe - restarts container if fails
        # Only checks if app is alive, not dependencies
        livenessProbe:
          httpGet:
            path: /health/live
            port: 3737
          initialDelaySeconds: 0  # Startup probe handles initial delay
          periodSeconds: 10
          timeoutSeconds: 5
          successThreshold: 1
          failureThreshold: 3

        # Readiness probe - stops sending traffic if fails
        # Checks app + database connectivity
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 3737
          initialDelaySeconds: 0
          periodSeconds: 5
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3

        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

**Explanation**:
- **startupProbe**: Runs first, gives 5 minutes for app to start
- **livenessProbe**: Checks every 10s, restarts after 3 failures (30s)
- **readinessProbe**: Checks every 5s, removes from load balancer after 3 failures (15s)

**Note**: This is a reference configuration. Actual Kubernetes deployment is covered in Chapter 15.

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
# Test all health endpoints
curl http://localhost:3737/health | jq
curl http://localhost:3737/health/live | jq
curl http://localhost:3737/health/ready | jq
curl http://localhost:3737/health/startup | jq

# Test with database stopped
docker compose stop postgres
sleep 2
curl http://localhost:3737/health | jq
# Should show unhealthy status and 503 status code

# Restart database
docker compose start postgres
sleep 2
curl http://localhost:3737/health | jq
# Should return to healthy
```

### Expected Results

- `/health` returns detailed status with all checks
- `/health/live` always returns 200 (unless app is crashed)
- `/health/ready` returns 200 when healthy, 503 when database down
- `/health/startup` returns 200 when database connected
- Status changes appropriately when database is stopped/started
- Uptime counter increases over time

---

## Common Issues and Solutions

### Issue: Health check always returns unhealthy

**Symptoms**: Even with database running, health check shows unhealthy

**Cause**: Database connection pool exhausted or timeout too short

**Solution**:
```bash
# Check database is actually running
docker compose ps

# Test database directly
docker compose exec postgres pg_isready

# Increase connection pool in .env
DATABASE_MAX_CONNECTIONS=20

# Check for connection leaks in application code
```

---

### Issue: Startup probe fails in Kubernetes

**Symptoms**: Container keeps restarting with startup probe failure

**Cause**: Application takes too long to start or database not ready

**Solution**:
```yaml
# Increase startup probe failure threshold
startupProbe:
  failureThreshold: 60  # 10 minutes instead of 5
  periodSeconds: 10

# Or add init container to wait for database
initContainers:
- name: wait-for-db
  image: busybox
  command: ['sh', '-c', 'until nc -z postgres 5432; do sleep 2; done']
```

---

### Issue: Health check is slow

**Symptoms**: Health checks take > 1 second to respond

**Cause**: Database check is slow or too many checks

**Solution**:
```rust
// Add timeout to database check
use tokio::time::timeout;
use std::time::Duration;

async fn check_database(db: &DatabaseConnection) -> ComponentHealth {
    let check = timeout(
        Duration::from_millis(500),
        crate::utils::database::verify_connection(db)
    ).await;

    match check {
        Ok(Ok(_)) => ComponentHealth::ok(),
        Ok(Err(e)) => ComponentHealth::unhealthy(format!("Database error: {}", e)),
        Err(_) => ComponentHealth::unhealthy("Database check timeout".to_string()),
    }
}
```

---

### Issue: Too many health check requests

**Symptoms**: Logs flooded with health check requests

**Cause**: Kubernetes probes configured too aggressively

**Solution**:
```yaml
# Increase probe periods
readinessProbe:
  periodSeconds: 10  # Instead of 5
livenessProbe:
  periodSeconds: 30  # Instead of 10

# Or filter health check logs
# In src/utils/tracing.rs
EnvFilter::new("info,actix_web::middleware::logger=/health=warn")
```

---

## Code Review

### Key Design Principles Demonstrated
- **Purpose-specific probes**: Separate handlers for readiness, liveness, and startup make it clear which dependencies affect each signal.
- **Explicit status modeling**: `HealthStatus` and `ComponentStatus` enums guarantee only known states reach the response payload.
- **Dependency isolation**: Each component check (database, migrations, uptime) runs independently, preventing a single failure from crashing the handler.
- **Operational metadata**: Response payloads include duration, version, and messages that on-call engineers can act on immediately.

### Architecture Benefits
- **Observability-first**: Structured JSON output feeds directly into dashboards and alerting without extra transformation.
- **Resilience**: Degraded components still produce HTTP 200 while flagging issues, preventing cascading outages from transient blips.
- **Deployment readiness**: Kubernetes (or any orchestrator) can wire probes directly to these endpoints, shortening production setup time.
- **Extensibility**: Adding new component checks only requires pushing another entry into the `HashMap`, keeping the handler maintainable.

### Complete Health Monitoring Structure
```rust
pub async fn readiness(state: web::Data<AppState>) -> impl Responder {
    let mut components = HashMap::new();
    components.insert("database", check_database(&state.db).await);
    components.insert("migrations", check_migrations().await);
    // ... existing checks ...

    let status = HealthResponse::determine_status(&components);
    let payload = HealthResponse::from_components(status, components, state.uptime());

    HttpResponse::build(status.to_status_code()).json(payload)
}
```

```json
{
  "status": "degraded",
  "version": "1.3.0",
  "uptime_seconds": 742,
  "components": {
    "database": { "status": "ok", "response_time_ms": 12 },
    "migrations": { "status": "unhealthy", "message": "pending migration 20240101" }
  }
}
```

## Understanding Health Check Patterns

### Health vs Readiness vs Liveness

```
┌─────────────────────────────────────────┐
│ Application Startup                     │
│                                         │
│  startupProbe running                   │
│    ↓                                    │
│  Waiting for database...                │
│    ↓                                    │
│  startupProbe succeeds                  │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│ Normal Operation                        │
│                                         │
│  livenessProbe: Checks app is alive    │
│    Every 10s                            │
│    Failure → Restart container          │
│                                         │
│  readinessProbe: Checks ready for traffic│
│    Every 5s                             │
│    Failure → Stop routing traffic       │
│                                         │
│  Health endpoint: Detailed status      │
│    On-demand                            │
│    Returns component details            │
└─────────────────────────────────────────┘
```

### When to Use Each

| Endpoint | Check | Failure Action | Use Case |
|----------|-------|----------------|----------|
| `/health` | Detailed | None (monitoring) | Dashboards, debugging |
| `/health/live` | App alive | Restart | Kubernetes liveness |
| `/health/ready` | App + DB | Remove from LB | Kubernetes readiness |
| `/health/startup` | Initial startup | Restart if timeout | Slow app startup |

---

## Testing Health Checks

Create tests to verify health check behavior:

1. **Create `tests/health_tests.rs`**:

```rust
use actix_web::{test, App, web};
use actix_memo_app::{handlers, state::AppState};

#[actix_web::test]
async fn test_liveness_probe() {
    let app = test::init_service(
        App::new().route("/health/live", web::get().to(handlers::health::liveness))
    ).await;

    let req = test::TestRequest::get()
        .uri("/health/live")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Liveness should always return 200
    assert_eq!(resp.status(), 200);
}

// Note: Testing readiness requires database connection
// See tests/integration_tests.rs for full integration tests
```

2. **Run tests**:
   ```bash
   cargo test test_liveness_probe
   ```

---

## Summary

Congratulations! You've implemented production-ready health checks. You now have:

1. **Structured health check responses** with component details
2. **Multiple probe types** for different use cases
3. **Kubernetes-compatible probes** (startup, liveness, readiness)
4. **Comprehensive health endpoint** with detailed status
5. **Graceful degradation** for partial failures
6. **Uptime tracking** and version reporting
7. **Proper HTTP status codes** for monitoring

### Key Takeaways

- **Different probes serve different purposes** - liveness, readiness, and startup each have specific roles
- **Fast health checks** should complete in < 1 second
- **Liveness checks** should NOT check external dependencies
- **Readiness checks** SHOULD check critical dependencies
- **Structured responses** make monitoring and debugging easier
- **Graceful degradation** improves availability
- **Proper status codes** enable automated monitoring

### Architecture So Far

```
┌─────────────────────────────────────┐
│        HTTP Requests                │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Middleware Stack                   │
│  - Security Headers                 │
│  - CORS                             │
│  - Compression                      │
│  - Logging                          │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Health Check Routes                │
│  - /health (detailed)               │
│  - /health/live (liveness)          │
│  - /health/ready (readiness)        │
│  - /health/startup (startup)        │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Component Checks                   │
│  - Database connectivity            │
│  - Response time tracking           │
│  - Status aggregation               │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Health Response                    │
│  - Overall status                   │
│  - Component details                │
│  - Uptime and version               │
└─────────────────────────────────────┘
```

---

## Next Steps

### Required: Chapter 5 - Data Transfer Objects and Validation

You'll introduce DTOs that separate transport concerns from domain logic, enforce validation rules, and prepare the API for richer request/response contracts. Expect to route validation failures through the error handling stack and keep health endpoints untouched.

### Optional Exercises

1. **Challenge**: Expose a `/health/metrics` endpoint that emits Prometheus-compatible gauges for each dependency.
2. **Challenge**: Write a shell script that polls the readiness endpoint until it passes, then starts dependent services.
3. **Challenge**: Simulate a degraded dependency and document how each probe responds so your ops runbook has concrete expectations.

---

## Additional Resources

### Health Checks
- [Kubernetes Probes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/) - Official guide
- [Health Check Pattern](https://microservices.io/patterns/observability/health-check-api.html) - Microservices pattern
- [Health Check RFC](https://tools.ietf.org/html/draft-inadarei-api-health-check) - Draft standard

### Monitoring
- [Prometheus](https://prometheus.io/) - Metrics and monitoring
- [Grafana](https://grafana.com/) - Visualization
- [OpenTelemetry](https://opentelemetry.io/) - Observability framework

### Production Deployment
- [12-Factor App](https://12factor.net/) - Best practices
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/configuration/overview/) - K8s patterns

---

**Ready to build type-safe DTOs? Let's move on to [Chapter 5: DTOs and Validation](chapter-05.md)!**
