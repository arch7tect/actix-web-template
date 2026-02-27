# Chapter 20: Kubernetes Readiness - Preparing for Multi-Replica Deployment

## Overview

In this chapter, we prepare our application for deployment on Kubernetes. Before writing any Kubernetes manifests (that comes in Chapter 21), we need to fix several assumptions baked into our code that break when multiple copies of the application run behind a load balancer.

You'll learn to identify and fix five issues: broken rate limiting behind a proxy, a migration race condition, missing graceful shutdown, hardcoded security headers, and hardcoded connection pool settings. By the end of this chapter, the application will be ready for multi-replica deployment.

> **Note**: This chapter modifies only Rust code, the Dockerfile, and configuration. No Kubernetes knowledge is required yet. Think of it as "hardening" the application for any load-balanced environment, not just Kubernetes.

## Prerequisites

### Completed Chapters

- **Chapter 19: Tags Feature - Web UI** (Required)
  - All application code through tags feature
  - Docker deployment working

- **Chapter 15: Docker Deployment** (Recommended)
  - Understanding of Dockerfile and docker-compose

- **Chapter 13: Security Enhancements** (Recommended)
  - Security headers middleware
  - Rate limiting setup

### Required Knowledge

- Understanding of HTTP proxies and load balancers (conceptual)
- Docker basics (building images, running containers)
- Environment variables for configuration

### Required Software

- Working application from Chapter 19
- Docker and docker-compose installed
- `curl` for testing

## Learning Objectives

By completing this chapter, you will:

1. Understand what breaks when scaling from one process to many
2. Implement a proxy-aware rate limiter using `X-Forwarded-For`
3. Separate database migrations from application startup
4. Add graceful shutdown to avoid dropped requests during rolling updates
5. Externalize hardcoded settings into environment variables
6. Configure security headers based on deployment environment

## Concepts Covered

### The Single-Process Assumption

When developing locally, your application runs as a single process handling all requests. Many design decisions that work fine in this model break when you scale horizontally:

```
Single Process (Development)        Multiple Replicas (Production)
┌─────────────────────┐            ┌──────────────────────────────────┐
│                     │            │         Load Balancer            │
│   Client            │            │    (NGINX Ingress / ALB)         │
│     │                │            │         │                       │
│     ▼                │            │    ┌────┼────┐                  │
│   App (1 process)   │            │    ▼    ▼    ▼                  │
│   - rate limiter    │            │  Pod1  Pod2  Pod3               │
│   - migrations      │            │  each has its own:              │
│   - security hdrs   │            │  - rate limiter (not shared!)   │
│   - pool settings   │            │  - migration runner             │
│                     │            │  - hardcoded config             │
└─────────────────────┘            └──────────────────────────────────┘
```

In the multi-replica world:
- **Rate limiting** counts are per-process. A client can send 100 requests to each of 3 replicas = 300 total, tripling the intended limit.
- **Migrations** run on every container start. If 3 replicas start simultaneously, all 3 attempt migrations concurrently, causing lock contention.
- **No graceful shutdown** means in-flight requests get connection-reset errors when Kubernetes replaces a pod.
- **Hardcoded settings** prevent tuning per environment without rebuilding the image.

### X-Forwarded-For and Trust Boundaries

When your app runs behind a reverse proxy (like NGINX Ingress in Kubernetes), the proxy's IP becomes the peer IP for every request. To identify the real client, proxies add the `X-Forwarded-For` header:

```
Client (203.0.113.50) → Proxy (10.0.0.1) → App

X-Forwarded-For: 203.0.113.50
Peer IP seen by app: 10.0.0.1
```

With multiple proxies, the header becomes a comma-separated chain:

```
X-Forwarded-For: client-ip, proxy1-ip, proxy2-ip
                 ↑ leftmost              ↑ rightmost
                 (client-controlled)      (set by trusted proxy)
```

**Security rule**: Always use the **rightmost** IP (the one your trusted proxy appended), not the leftmost (which the client can set to anything). This is safe because your proxy overwrites or appends to the header, so the rightmost entry is trustworthy.

---

## 20.1 Proxy-Aware Rate Limiting

### The Problem

Our current rate limiter in `main.rs` uses `GovernorConfigBuilder::default()`, which keys on the TCP peer IP address. Behind a load balancer, every request appears to come from the proxy's IP, so all clients share a single rate-limit bucket:

```rust
// Before: main.rs (broken behind proxy)
let governor_conf = GovernorConfigBuilder::default()  // WARNING: Uses PeerIpKeyExtractor
    .milliseconds_per_request(600)
    .burst_size(100)
    .finish()
    .unwrap();
```

Meanwhile, `src/middleware/rate_limit.rs` had a `create_rate_limiter()` function with `PeerIpKeyExtractor`, but it was dead code — never exported or called.

### The Solution

We create a custom `ForwardedIpKeyExtractor` that reads client IP from proxy headers, falling back to the peer IP when no proxy headers are present. This works correctly in both direct and proxied deployments.

### Implementation

Replace the contents of `src/middleware/rate_limit.rs`:

```rust
// src/middleware/rate_limit.rs

use actix_governor::{                                           // ** NEW
    Governor, GovernorConfig, GovernorConfigBuilder,             // ** NEW
    KeyExtractor, SimpleKeyExtractionError,                     // ** NEW
    governor::middleware::NoOpMiddleware,                        // ** NEW
};                                                              // ** NEW
use actix_web::dev::ServiceRequest;                             // ** NEW

#[derive(Debug, Clone, Copy, PartialEq, Eq)]                   // ** NEW
pub struct ForwardedIpKeyExtractor;                              // ** NEW

impl KeyExtractor for ForwardedIpKeyExtractor {                  // ** NEW
    type Key = String;                                           // ** NEW
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;  // ** NEW

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // Behind a trusted proxy: use the rightmost IP from X-Forwarded-For.
        // The rightmost entry is the one appended by the proxy itself (not client-controlled).
        if let Some(forwarded) = req.headers().get("x-forwarded-for")
            && let Ok(value) = forwarded.to_str()
            && let Some(ip) = value.rsplit(',').next()
        {
            let ip = ip.trim();
            if !ip.is_empty() {
                return Ok(ip.to_string());
            }
        }

        // Fallback: X-Real-Ip header (set by some proxies like NGINX)
        if let Some(real_ip) = req.headers().get("x-real-ip")
            && let Ok(ip) = real_ip.to_str()
        {
            let ip = ip.trim();
            if !ip.is_empty() {
                return Ok(ip.to_string());
            }
        }

        // Final fallback: peer IP from the TCP connection
        Ok(req
            .peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string()))
    }
}

pub fn create_rate_limiter_config(
) -> anyhow::Result<GovernorConfig<ForwardedIpKeyExtractor, NoOpMiddleware>> {
    let mut builder = GovernorConfigBuilder::default()
        .key_extractor(ForwardedIpKeyExtractor);                // ** NEW
    builder.milliseconds_per_request(600).burst_size(100);
    builder
        .finish()
        .ok_or_else(|| anyhow::anyhow!("Failed to create rate limiter configuration"))
}

pub fn create_governor(
    config: &GovernorConfig<ForwardedIpKeyExtractor, NoOpMiddleware>,
) -> Governor<ForwardedIpKeyExtractor, NoOpMiddleware> {
    Governor::new(config)
}
```

### How `ForwardedIpKeyExtractor` Works

The extractor implements `actix_governor::KeyExtractor`, a trait that determines how rate-limit keys are derived from requests. The extraction follows a priority chain:

1. **`X-Forwarded-For`** — Standard header set by proxies. We take the rightmost IP (the one appended by the trusted proxy, not the client-controlled leftmost).
2. **`X-Real-Ip`** — Non-standard but widely used by NGINX. Contains a single IP.
3. **Peer IP** — Direct TCP connection address. Works for direct deployments.

> **Security note**: The `X-Forwarded-For` header can be spoofed by clients. This extractor should only be used behind a trusted reverse proxy (e.g., NGINX Ingress in Kubernetes) that sanitizes the header. In direct-to-internet deployments without a proxy, the peer IP fallback is used automatically and is safe.

### Update Module Exports

Update `src/middleware/mod.rs` to export the rate limiting module:

```rust
// src/middleware/mod.rs

pub mod rate_limit;                                              // ** NEW
pub mod security_headers;

pub use rate_limit::{create_governor, create_rate_limiter_config}; // ** NEW
pub use security_headers::SecurityHeaders;
```

### Checkpoint

After completing this section, verify the module compiles:

```bash
cargo check
```

You should see no errors. We'll wire the rate limiter into `main.rs` in section 20.5.

---

## 20.2 Separating Migrations from Application Startup

### The Problem

Our `Dockerfile` currently creates an `entrypoint.sh` script that runs migrations before starting the application:

```dockerfile
# Before: Dockerfile (broken with multiple replicas)
RUN echo '#!/bin/sh\n\
echo "Running database migrations..."\n\
./migration\n\
echo "Starting application..."\n\
exec ./actix-web-template' > /app/entrypoint.sh && \
    chmod +x /app/entrypoint.sh

CMD ["/app/entrypoint.sh"]
```

When 3 replicas start simultaneously, all 3 run migrations concurrently. While SeaORM migrations are idempotent (they won't corrupt data), this causes unnecessary database lock contention and wastes resources.

### The Solution

Remove the `entrypoint.sh` script and make the application binary the default command. Both binaries (`actix-web-template` and `migration`) remain in the image, so the same image can be used for either purpose:

- **Default**: `docker run <image>` starts the application (no migrations)
- **Migrations**: `docker run <image> ./migration` runs migrations and exits

In Kubernetes (Chapter 21), migrations will run as a separate Job before the application Deployment rolls out.

### Implementation

Update the `Dockerfile`:

```dockerfile
# Multi-stage Dockerfile for Actix Web Memos Application

# Build stage
FROM rust:latest as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code and resources
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY migration ./migration

# Build for release (build all workspace members)
RUN cargo build --release --all

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        libpq5 \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binaries from builder (workspace builds to shared target/)
COPY --from=builder /app/target/release/actix-web-template .
COPY --from=builder /app/target/release/migration ./migration

# Copy templates and static files
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static

# Expose the application port
EXPOSE 3737

# Add health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3737/health || exit 1

# Run migrations separately: docker run <image> ./migration        ** NEW
# Default: start the application only (no migrations on startup)   ** NEW
CMD ["./actix-web-template"]                                      # ** CHANGED
```

The key changes:
- **Removed**: The `entrypoint.sh` script and its `RUN echo ...` block
- **Changed**: `CMD` from `["/app/entrypoint.sh"]` to `["./actix-web-template"]`
- **Kept**: Both binaries in the image for flexibility

### Checkpoint

Build the Docker image and verify both commands work:

```bash
# Build the image
docker build -t memos-app:latest .

# Run migrations only (exits when done)
docker run --rm --env-file .env memos-app:latest ./migration

# Run the application (default command)
docker run --rm --env-file .env -p 3737:3737 memos-app:latest
```

---

## 20.3 Graceful Shutdown

### The Problem

When Kubernetes sends SIGTERM to stop a pod (during rolling updates, scaling down, or node draining), the process should finish handling in-flight requests before exiting. Without a shutdown timeout, requests get connection-reset errors.

### The Solution

Actix Web has built-in graceful shutdown support via `.shutdown_timeout()`. When the server receives SIGTERM:

1. It stops accepting new connections
2. It waits up to the timeout for in-flight requests to complete
3. It shuts down

We also call `shutdown_tracing()` after the server stops to flush any remaining trace spans.

### The Timeline

```
SIGTERM received
    │
    ▼
Stop accepting new connections
    │
    ▼
Wait up to 30s for in-flight requests  ← shutdown_timeout(30)
    │
    ▼
Server stops
    │
    ▼
Flush tracing spans                     ← shutdown_tracing()
    │
    ▼
Process exits
```

In Kubernetes, `terminationGracePeriodSeconds` should be slightly longer than this timeout (e.g., 35s) to give the application time to shut down before Kubernetes sends SIGKILL.

### Implementation

The change is in `main.rs` — add `.shutdown_timeout(30)` to the `HttpServer` builder chain, and call `shutdown_tracing()` after the server exits. We'll show the full `main.rs` in section 20.5 after all changes are combined.

The key additions:

```rust
// In the HttpServer builder chain:
    .shutdown_timeout(30)                                        // ** NEW

// After server.await?:
    shutdown_tracing();                                          // ** NEW
```

The `shutdown_tracing()` function in `src/observability/tracing.rs` logs the shutdown event. Since OpenTelemetry 0.31, `SdkTracerProvider` is automatically shut down when dropped, so explicit provider shutdown is not required.

### Checkpoint

Start the application, then send SIGTERM:

```bash
# Terminal 1: start the app
cargo run

# Terminal 2: send SIGTERM
kill -TERM $(pgrep actix-web-template)

# You should see "Shutting down tracing" and "Application shutdown complete" in the logs
```

---

## 20.4 Configurable Security Headers and Pool Settings

### The Problem

Several settings are hardcoded that should vary by environment:

1. **HSTS header** (`Strict-Transport-Security`) — should only be enabled behind TLS in production
2. **X-Frame-Options** — might need `SAMEORIGIN` instead of `DENY` for iframe embedding
3. **Connection pool** — `min_connections`, `idle_timeout`, `max_lifetime` are hardcoded in `main.rs`

### The Solution

Move these to `Settings` so they can be configured via environment variables without rebuilding.

### Implementation: Settings

Add `SecurityConfig` and pool fields to `src/config/settings.rs`:

```rust
// src/config/settings.rs

use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cors: CorsConfig,
    pub api: ApiConfig,
    pub app: AppConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,                                // ** NEW
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub trust_proxy: bool,                                       // ** NEW
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: u64,
    pub min_connections: u32,                                     // ** NEW
    pub idle_timeout: u64,                                       // ** NEW
    pub max_lifetime: u64,                                       // ** NEW
}

// ... (CorsConfig, ApiConfig, AppConfig, LoggingConfig unchanged)

#[derive(Debug, Clone, Deserialize)]                             // ** NEW
pub struct SecurityConfig {                                      // ** NEW
    pub hsts_enabled: bool,                                      // ** NEW
    pub frame_options: String,                                   // ** NEW
}                                                                // ** NEW
```

In the `Settings::load()` method, add the new fields:

```rust
// In Settings::load():

let server = ServerConfig {
    host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
    port: env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3737".to_string())
        .parse()?,
    trust_proxy: env::var("TRUST_PROXY")                        // ** NEW
        .unwrap_or_else(|_| "false".to_string())                // ** NEW
        .parse()                                                 // ** NEW
        .unwrap_or(false),                                       // ** NEW
};

let database = DatabaseConfig {
    url: env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
    max_connections: env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_string())
        .parse()?,
    connect_timeout: env::var("DATABASE_CONNECT_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse()?,
    min_connections: env::var("DATABASE_MIN_CONNECTIONS")        // ** NEW
        .unwrap_or_else(|_| "2".to_string())                    // ** NEW
        .parse()?,                                               // ** NEW
    idle_timeout: env::var("DATABASE_IDLE_TIMEOUT")              // ** NEW
        .unwrap_or_else(|_| "300".to_string())                  // ** NEW
        .parse()?,                                               // ** NEW
    max_lifetime: env::var("DATABASE_MAX_LIFETIME")              // ** NEW
        .unwrap_or_else(|_| "1800".to_string())                 // ** NEW
        .parse()?,                                               // ** NEW
};

// ... after logging config:

let security = SecurityConfig {                                  // ** NEW
    hsts_enabled: env::var("HSTS_ENABLED")                      // ** NEW
        .unwrap_or_else(|_| "false".to_string())                // ** NEW
        .parse()                                                 // ** NEW
        .unwrap_or(false),                                       // ** NEW
    frame_options: env::var("FRAME_OPTIONS")                     // ** NEW
        .unwrap_or_else(|_| "DENY".to_string()),                // ** NEW
};                                                               // ** NEW

Ok(Settings {
    server,
    database,
    cors,
    api,
    app,
    logging,
    security,                                                    // ** NEW
})
```

Export `SecurityConfig` from `src/config/mod.rs`:

```rust
// src/config/mod.rs

pub mod settings;

pub use settings::{SecurityConfig, Settings};                    // ** CHANGED
```

### Implementation: Security Headers Middleware

Update `src/middleware/security_headers.rs` to accept configuration instead of hardcoding values:

```rust
// src/middleware/security_headers.rs

use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::HeaderValue;
use std::future::{Ready, ready};
use std::pin::Pin;

pub struct SecurityHeaders {
    hsts_enabled: bool,                                          // ** NEW
    frame_options: HeaderValue,                                  // ** NEW
}

impl SecurityHeaders {
    pub fn new(hsts_enabled: bool, frame_options: &str) -> Self { // ** NEW
        Self {                                                    // ** NEW
            hsts_enabled,                                         // ** NEW
            frame_options: HeaderValue::from_str(frame_options)   // ** NEW
                .unwrap_or_else(|_| HeaderValue::from_static("DENY")), // ** NEW
        }                                                         // ** NEW
    }
}

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
        ready(Ok(SecurityHeadersMiddleware {
            service,
            hsts_enabled: self.hsts_enabled,                     // ** NEW
            frame_options: self.frame_options.clone(),            // ** NEW
        }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
    hsts_enabled: bool,                                          // ** NEW
    frame_options: HeaderValue,                                  // ** NEW
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        let hsts_enabled = self.hsts_enabled;                    // ** NEW
        let frame_options = self.frame_options.clone();           // ** NEW

        Box::pin(async move {
            let mut res = fut.await?;

            let headers = res.headers_mut();
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                frame_options,                                   // ** CHANGED (was hardcoded "DENY")
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            );
            if hsts_enabled {                                    // ** NEW (was always included)
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                    HeaderValue::from_static("max-age=31536000; includeSubDomains"),
                );
            }
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';",
                ),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
            );

            Ok(res)
        })
    }
}
```

### Checkpoint

Verify the new settings work:

```bash
# Default: HSTS disabled, DENY frame options
cargo run &
curl -s -I http://localhost:3737/health | grep -i "x-frame-options"
# x-frame-options: DENY

curl -s -I http://localhost:3737/health | grep -i "strict-transport"
# (no output — HSTS is disabled by default)

# With HSTS enabled and SAMEORIGIN frame options:
HSTS_ENABLED=true FRAME_OPTIONS=SAMEORIGIN cargo run &
curl -s -I http://localhost:3737/health | grep -i "strict-transport"
# strict-transport-security: max-age=31536000; includeSubDomains

curl -s -I http://localhost:3737/health | grep -i "x-frame-options"
# x-frame-options: SAMEORIGIN
```

---

## 20.5 Wiring Everything Together in main.rs

Now we combine all the changes into `src/main.rs`:

```rust
// src/main.rs

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Compress, web};
use actix_web_prom::PrometheusMetricsBuilder;
use actix_web_template::{
    config::Settings,
    docs::ApiDoc,
    handlers,
    middleware::{SecurityHeaders, create_governor, create_rate_limiter_config}, // ** CHANGED
    observability::tracing::{init_tracing_with_otlp, shutdown_tracing},        // ** CHANGED
    state::AppState,
};
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load()?;

    let otlp_endpoint = std::env::var("OTLP_ENDPOINT").ok();
    init_tracing_with_otlp("memos-api", otlp_endpoint)
        .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;

    tracing::info!(
        version = settings.app.version,
        env = ?settings.app.env,
        "Starting Actix Web Memos application"
    );

    settings.validate()?;

    tracing::info!(
        url = %settings.database.url.split('@').next_back().unwrap_or("***"),
        max_connections = settings.database.max_connections,
        min_connections = settings.database.min_connections,       // ** NEW
        connect_timeout = settings.database.connect_timeout,
        idle_timeout = settings.database.idle_timeout,             // ** NEW
        max_lifetime = settings.database.max_lifetime,             // ** NEW
        "Connecting to database with tuned connection pool"
    );

    let mut opt = ConnectOptions::new(&settings.database.url);
    opt.max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)        // ** CHANGED (was hardcoded 2)
        .connect_timeout(Duration::from_secs(settings.database.connect_timeout))
        .acquire_timeout(Duration::from_secs(settings.database.connect_timeout))
        .idle_timeout(Duration::from_secs(settings.database.idle_timeout))   // ** CHANGED (was 300)
        .max_lifetime(Duration::from_secs(settings.database.max_lifetime))   // ** CHANGED (was 1800)
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;
    tracing::info!("Database connection established with optimized pool settings");

    tracing::info!("Initializing Prometheus metrics exporter");
    let prometheus = PrometheusMetricsBuilder::new("actix_web")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let state = AppState::new(settings.clone(), db);

    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);
    tracing::info!(address = %bind_address, "Starting HTTP server");

    if settings.server.trust_proxy {                              // ** NEW
        tracing::info!(                                           // ** NEW
            "TRUST_PROXY=true: rate limiter will use X-Forwarded-For / X-Real-Ip headers"
        );                                                        // ** NEW
    }                                                             // ** NEW

    tracing::info!("Configuring rate limiting: 100 requests per minute per IP");
    let governor_conf = create_rate_limiter_config()?;            // ** CHANGED

    let security_hsts = settings.security.hsts_enabled;           // ** NEW
    let security_frame = settings.security.frame_options.clone(); // ** NEW

    HttpServer::new(move || {
        let rate_limiter = create_governor(&governor_conf);       // ** CHANGED
        let cors = if state.config.cors.allowed_origins.len() == 1
            && state.config.cors.allowed_origins[0] == "*"
        {
            Cors::permissive()
        } else {
            let mut cors = Cors::default();
            for origin in &state.config.cors.allowed_origins {
                cors = cors.allowed_origin(origin.as_str());
            }
            cors.allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .max_age(3600)
        };

        let openapi = ApiDoc::openapi();

        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::JsonConfig::default().limit(state.config.api.max_request_size))
            .app_data(web::PayloadConfig::default().limit(state.config.api.max_request_size))
            .wrap(prometheus.clone())
            .wrap(Compress::default())
            .wrap(SecurityHeaders::new(security_hsts, &security_frame)) // ** CHANGED
            .wrap(rate_limiter)
            .wrap(cors)
            .wrap(TracingLogger::default())
            .service(actix_files::Files::new("/static", "./static"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(handlers::index)
            .service(handlers::get_memos_list)
            .service(handlers::get_new_memo_form)
            .service(handlers::create_memo_web)
            .service(handlers::get_edit_memo_form)
            .service(handlers::update_memo_web)
            .service(handlers::delete_memo_web)
            .service(handlers::toggle_memo_complete_web)
            .service(handlers::health_check)
            .service(handlers::ready)
            .service(handlers::list_memos)
            .service(handlers::get_memo)
            .service(handlers::create_memo)
            .service(handlers::update_memo)
            .service(handlers::patch_memo)
            .service(handlers::delete_memo)
            .service(handlers::toggle_complete)
            .service(handlers::list_tags)
            .service(handlers::test_not_found)
            .service(handlers::test_validation)
            .service(handlers::test_internal)
            .service(handlers::test_database)
            .service(handlers::test_create_dto)
            .service(handlers::test_repo)
            .service(handlers::test_svc)
    })
    .workers(num_cpus::get() * 2)
    .keep_alive(Duration::from_secs(75))
    .client_request_timeout(Duration::from_secs(60))
    .client_disconnect_timeout(Duration::from_secs(5))
    .shutdown_timeout(30)                                        // ** NEW
    .bind(&bind_address)?
    .run()
    .await?;

    shutdown_tracing();                                          // ** NEW
    tracing::info!("Application shutdown complete");
    Ok(())
}
```

### Summary of Changes in main.rs

| Line | Before | After |
|------|--------|-------|
| Imports | `GovernorConfigBuilder` directly | `create_governor`, `create_rate_limiter_config` from middleware |
| Imports | No `shutdown_tracing` | Import `shutdown_tracing` |
| Pool settings | `.min_connections(2)` hardcoded | `.min_connections(settings.database.min_connections)` |
| Pool settings | `.idle_timeout(Duration::from_secs(300))` hardcoded | `.idle_timeout(Duration::from_secs(settings.database.idle_timeout))` |
| Pool settings | `.max_lifetime(Duration::from_secs(1800))` hardcoded | `.max_lifetime(Duration::from_secs(settings.database.max_lifetime))` |
| Rate limiter | Inline `GovernorConfigBuilder` | `create_rate_limiter_config()` |
| Security headers | `.wrap(SecurityHeaders)` | `.wrap(SecurityHeaders::new(hsts, &frame))` |
| Shutdown | No timeout | `.shutdown_timeout(30)` |
| After exit | Nothing | `shutdown_tracing()` |

---

## 20.6 Environment Variables

Update `.env.example` with the new configuration options:

```bash
# .env.example

# Development Environment Configuration
# This is the default configuration for local development
# Copy this to .env: cp .env.example .env

# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3737
APP_ENV=development

# Proxy Configuration
# Set to true when running behind a reverse proxy (e.g., NGINX Ingress in Kubernetes).
# When true, rate limiting uses X-Forwarded-For / X-Real-Ip headers to identify clients.
TRUST_PROXY=false                                                # ** NEW

# Logging Configuration
RUST_LOG=info,actix_web=debug,actix_web_template=debug
LOG_FORMAT=pretty

# Database Configuration
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db
DATABASE_MAX_CONNECTIONS=10
DATABASE_CONNECT_TIMEOUT=30

# Connection Pool Tuning                                         # ** NEW
DATABASE_MIN_CONNECTIONS=2                                       # ** NEW
DATABASE_IDLE_TIMEOUT=300                                        # ** NEW
DATABASE_MAX_LIFETIME=1800                                       # ** NEW

# CORS Configuration
CORS_ALLOWED_ORIGINS=*

# Request Configuration
MAX_REQUEST_SIZE=262144

# API Documentation
ENABLE_SWAGGER=true

# Security Headers                                               # ** NEW
# Enable HSTS header (set to true in production behind TLS)
HSTS_ENABLED=false                                               # ** NEW
# X-Frame-Options value: DENY or SAMEORIGIN
FRAME_OPTIONS=DENY                                               # ** NEW

# OpenTelemetry / Jaeger Tracing
OTLP_ENDPOINT=http://jaeger:4317
```

All new variables have sensible defaults, so existing `.env` files continue to work without changes.

---

## Verification

Run the full verification suite to confirm everything works:

```bash
# 1. Code compiles without errors
cargo check

# 2. No clippy warnings
cargo clippy -- -D warnings

# 3. All existing tests pass
cargo test

# 4. Test proxy-aware rate limiting
cargo run &

# Without proxy headers: uses peer IP
curl http://localhost:3737/api/v1/memos
# Should work normally

# With X-Forwarded-For: uses rightmost IP for rate limiting
curl -H "X-Forwarded-For: spoofed-by-client, 10.0.0.1" http://localhost:3737/api/v1/memos
# Rate limit key is "10.0.0.1" (rightmost, trusted)

# Changing only the leftmost IP does NOT reset the rate limit counter
curl -H "X-Forwarded-For: different-spoof, 10.0.0.1" http://localhost:3737/api/v1/memos
# Still keyed on "10.0.0.1"

# 5. Docker build succeeds
docker build -t memos-app:latest .

# 6. Container starts without running migrations
docker run --rm -e DATABASE_URL=postgresql://x:x@localhost/x memos-app:latest &
# Should attempt to connect to DB (and fail if not available), but NOT run migrations
```

---

## Common Issues

### 1. "Cannot find `create_rate_limiter_config`"

Make sure `src/middleware/mod.rs` exports the rate_limit module:

```rust
pub mod rate_limit;
pub use rate_limit::{create_governor, create_rate_limiter_config};
```

### 2. "Missing field `security` in initializer of `Settings`"

Ensure `SecurityConfig` is loaded in `Settings::load()` and included in the returned struct:

```rust
Ok(Settings {
    server,
    database,
    cors,
    api,
    app,
    logging,
    security,  // Don't forget this!
})
```

### 3. "Missing field `trust_proxy` in initializer of `ServerConfig`"

Add the `trust_proxy` field to the `ServerConfig` construction in `Settings::load()`:

```rust
let server = ServerConfig {
    host: ...,
    port: ...,
    trust_proxy: env::var("TRUST_PROXY")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false),
};
```

### 4. Docker image still runs migrations on startup

Make sure you removed the entire `entrypoint.sh` block from the Dockerfile and changed `CMD` to `["./actix-web-template"]`.

### 5. HSTS header still appears with `HSTS_ENABLED=false`

Check that you're using `SecurityHeaders::new(hsts, &frame)` instead of the old unit struct `SecurityHeaders` in `main.rs`.

---

## Summary

In this chapter, we identified and fixed five assumptions that break under multi-replica deployment:

```
┌─────────────────────────────────────────────────────────────┐
│  Before (Chapter 19)           After (Chapter 20)          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Rate limiter uses             ForwardedIpKeyExtractor      │
│  PeerIpKeyExtractor            reads X-Forwarded-For,       │
│  (sees proxy IP)               falls back to peer IP        │
│                                                             │
│  Migrations run on             App binary only;             │
│  every container start         migrations run separately    │
│  (race condition)              (docker run <img> ./migration)│
│                                                             │
│  No shutdown timeout           .shutdown_timeout(30)        │
│  (requests dropped)            + shutdown_tracing()          │
│                                                             │
│  HSTS always on,               HSTS_ENABLED env var,        │
│  DENY hardcoded                FRAME_OPTIONS env var        │
│                                                             │
│  Pool settings hardcoded       DATABASE_MIN_CONNECTIONS,    │
│  in main.rs                    DATABASE_IDLE_TIMEOUT,       │
│                                DATABASE_MAX_LIFETIME        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Files Modified

| File | Change |
|------|--------|
| `src/middleware/rate_limit.rs` | Rewrote with `ForwardedIpKeyExtractor`, `create_rate_limiter_config()`, `create_governor()` |
| `src/middleware/mod.rs` | Export `rate_limit` module and its public functions |
| `src/middleware/security_headers.rs` | Accept `hsts_enabled` and `frame_options` via constructor; conditional HSTS |
| `src/config/settings.rs` | Added `SecurityConfig`, pool fields (`min_connections`, `idle_timeout`, `max_lifetime`), `trust_proxy` |
| `src/config/mod.rs` | Export `SecurityConfig` |
| `src/main.rs` | Wire new rate limiter, security headers, pool settings, `shutdown_timeout(30)`, `shutdown_tracing()` |
| `Dockerfile` | Remove `entrypoint.sh`; default CMD to `./actix-web-template` |
| `.env.example` | Added `TRUST_PROXY`, `DATABASE_MIN_CONNECTIONS`, `DATABASE_IDLE_TIMEOUT`, `DATABASE_MAX_LIFETIME`, `HSTS_ENABLED`, `FRAME_OPTIONS` |

### What You Learned

1. **Horizontal scaling breaks in-process state** — rate limiters, caches, and sessions that live in a single process don't work with multiple replicas
2. **X-Forwarded-For trust model** — always read the rightmost IP (proxy-appended), never the leftmost (client-controlled)
3. **Separation of concerns** — migrations are a deployment step, not an application startup step
4. **Graceful shutdown** — critical for zero-downtime deployments; Actix Web handles it with a single configuration call
5. **Externalize configuration** — every environment-specific value should come from an environment variable

---

## Next Steps

In **Chapter 21: First Kubernetes Deployment on minikube**, we'll:

- Set up a local Kubernetes cluster with minikube
- Deploy PostgreSQL as a StatefulSet with persistent storage
- Run migrations as a Kubernetes Job (using the separate migration binary)
- Deploy our hardened application with health probes and 2 replicas
- Verify everything works with `kubectl` commands

The application is now ready. Let's deploy it.

---

## Additional Resources

### Rate Limiting

- [actix-governor documentation](https://docs.rs/actix-governor)
- [OWASP Rate Limiting](https://cheatsheetseries.owasp.org/cheatsheets/Denial_of_Service_Cheat_Sheet.html)
- [X-Forwarded-For (MDN)](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For)

### Kubernetes Readiness

- [12-Factor App: Config](https://12factor.net/config)
- [Kubernetes: Container Lifecycle Hooks](https://kubernetes.io/docs/concepts/containers/container-lifecycle-hooks/)
- [Graceful Shutdown in Kubernetes](https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-termination)

### Actix Web

- [Actix Web: Server Configuration](https://actix.rs/docs/server)
- [Actix Web: Middleware](https://actix.rs/docs/middleware)
