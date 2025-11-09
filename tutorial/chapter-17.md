# Chapter 17: Observability Stack

## Overview

In this chapter, you'll add a production-grade observability stack to your application using OpenTelemetry, Jaeger, Prometheus, Grafana, and Loki. You'll learn how to instrument your code for distributed tracing, collect metrics, visualize data in dashboards, and aggregate logs.

Observability is critical for understanding system behavior in production, debugging issues, and monitoring performance.

## Prerequisites

### Completed Chapters
- Chapters 0-16: Full application with Docker deployment and CI/CD

### Required Knowledge
- Understanding of distributed systems concepts
- Basic knowledge of metrics and tracing
- Docker Compose orchestration

### Required Software
- Docker and Docker Compose (from Chapter 15)
- All services running via docker-compose

## Learning Objectives

By the end of this chapter, you will:

- Understand the three pillars of observability (traces, metrics, logs)
- Integrate OpenTelemetry for distributed tracing
- Export traces to Jaeger for visualization
- Collect and export Prometheus metrics
- Create Grafana dashboards for monitoring
- Aggregate logs with Loki
- Implement request tracking across services
- Debug performance issues with traces
- Set up alerts based on metrics

## Concepts Covered

### The Three Pillars of Observability

**1. Traces** - Track requests through your system
- Show the path a request takes
- Measure latency at each step
- Identify bottlenecks
- Understand dependencies

**2. Metrics** - Quantitative measurements over time
- Request rates
- Error rates
- Response times (latency)
- Resource usage (CPU, memory)

**3. Logs** - Event records
- What happened
- When it happened
- Contextual information
- Error details

### OpenTelemetry

**What it is:** Vendor-neutral observability framework

**Why we use it:**
- Industry standard (CNCF project)
- Supports traces, metrics, and logs
- Works with multiple backends (Jaeger, Prometheus, etc.)
- Future-proof (no vendor lock-in)

**Architecture:**
```
Application Code
    ↓ (instrumentation)
OpenTelemetry SDK
    ↓ (export)
OTLP Exporters
    ↓ (protocol)
Backends (Jaeger, Prometheus, etc.)
```

### Distributed Tracing

**Concept:** Track a single request across multiple services/layers

**Terminology:**
- **Trace**: Complete journey of a request
- **Span**: Single operation within a trace
- **Span Context**: Metadata propagated between spans

**Example trace hierarchy:**
```
HTTP Request (root span)
  ├─ Handler execution
  │   ├─ Service method
  │   │   ├─ Repository query
  │   │   │   └─ Database query
  │   │   └─ Cache check
  │   └─ DTO conversion
  └─ Response serialization
```

## Step-by-Step Instructions

Since the observability stack is **already configured** in your `docker-compose.yml`, this chapter focuses on understanding and using it.

### Step 1: Understanding the Observability Stack Configuration

**Services already configured:**

1. **Jaeger** - Distributed tracing
   - Port 16686: Web UI
   - Port 4317: OTLP gRPC receiver
   - Port 4318: OTLP HTTP receiver

2. **Prometheus** - Metrics collection
   - Port 9090: Web UI and API
   - Scrapes metrics from `/metrics` endpoint

3. **Grafana** - Visualization dashboards
   - Port 3001: Web UI
   - Login: admin/admin
   - Pre-configured with Prometheus and Loki data sources

4. **Loki** - Log aggregation
   - Port 3100: API
   - Stores logs for querying in Grafana

**Verify the configuration exists:**

```bash
# Check docker-compose.yml has observability services
grep -A 5 "jaeger:\|prometheus:\|grafana:\|loki:" docker-compose.yml
```

**Expected:** You should see all four services defined.

### Step 2: Start the Observability Stack

**Start all services:**

```bash
# Start everything (app + databases + observability)
docker-compose up -d

# Wait for services to be healthy
sleep 30

# Check all services are running
docker-compose ps
```

**Expected output:**
```
NAME              STATUS         PORTS
memos-app         Up (healthy)   0.0.0.0:3737->3737/tcp
memos-postgres    Up (healthy)   0.0.0.0:5432->5432/tcp
memos-jaeger      Up             0.0.0.0:16686->16686/tcp, ...
memos-prometheus  Up             0.0.0.0:9090->9090/tcp
memos-grafana     Up             0.0.0.0:3001->3000/tcp
memos-loki        Up             0.0.0.0:3100->3100/tcp
```

**Verify services are accessible:**

```bash
# Check Jaeger UI
curl -s http://localhost:16686/api/services | jq

# Check Prometheus targets
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[].labels'

# Check Grafana
curl -s http://localhost:3001/api/health | jq
```

### Step 3: Understanding Existing Instrumentation

Your application **already has** OpenTelemetry dependencies. Let's understand what's there:

**Check `Cargo.toml` dependencies:**

```toml
# Observability
opentelemetry = { version = "0.31", features = ["metrics", "trace"] }
opentelemetry_sdk = { version = "0.31", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.31", features = ["metrics", "trace", "grpc-tonic"] }
tracing-opentelemetry = "0.32"
actix-web-prom = "0.10"
```

**What these provide:**
- `opentelemetry`: Core API
- `opentelemetry_sdk`: Implementation
- `opentelemetry-otlp`: Export to Jaeger/Prometheus
- `tracing-opentelemetry`: Bridge between `tracing` and OpenTelemetry
- `actix-web-prom`: Prometheus metrics for Actix Web

**Existing tracing:** Your application uses `tracing` library (from Chapter 1):

```rust
// In handlers (already there)
#[tracing::instrument(name = "GET /health", skip(state))]
pub async fn health(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // ...
}
```

These `#[tracing::instrument]` macros automatically create spans!

### Step 4: Generate Some Traffic

Let's generate traffic to see in our observability tools.

**Create some data:**

```bash
# Create several memos
for i in {1..5}; do
  curl -X POST http://localhost:3737/api/v1/memos \
    -H "Content-Type: application/json" \
    -d "{
      \"title\": \"Test Memo $i\",
      \"description\": \"Generated for observability demo\",
      \"date_to\": \"2025-12-31T12:00:00Z\"
    }"
done

# List memos (multiple times)
for i in {1..10}; do
  curl -s http://localhost:3737/api/v1/memos?limit=10 > /dev/null
done

# Access health endpoint
for i in {1..5}; do
  curl -s http://localhost:3737/health > /dev/null
done

# Access web UI
curl -s http://localhost:3737/ > /dev/null
```

### Step 5: Exploring Traces in Jaeger

**Open Jaeger UI:**

```bash
# macOS
open http://localhost:16686

# Linux
xdg-open http://localhost:16686

# Or visit in browser
# http://localhost:16686
```

**What you'll see:**

1. **Service dropdown**: Select your service (if instrumented)
2. **Operation dropdown**: Choose specific endpoints
3. **Find Traces button**: Search for traces

**Note:** If traces aren't showing, you need to add OpenTelemetry initialization (covered in next steps).

### Step 6: Add OpenTelemetry Tracing (Implementation)

Currently, the app uses `tracing` but doesn't export to Jaeger. Let's fix that.

**Create `src/observability/mod.rs`:**

```bash
mkdir -p src/observability
```

**File: `src/observability/mod.rs`**

```rust
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, TracerProvider};
use opentelemetry_sdk::Resource;
use std::time::Duration;

/// Initialize OpenTelemetry with OTLP exporter for Jaeger
pub fn init_tracer(service_name: &str, otlp_endpoint: &str) -> anyhow::Result<TracerProvider> {
    // Create OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(otlp_endpoint)
        .with_timeout(Duration::from_secs(3));

    // Create tracer provider
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    Ok(tracer_provider)
}

/// Shutdown tracing gracefully
pub fn shutdown_tracer(tracer_provider: TracerProvider) {
    if let Err(err) = tracer_provider.shutdown() {
        eprintln!("Error shutting down tracer provider: {}", err);
    }
}
```

**Update `src/lib.rs`:**

```rust
pub mod config;
pub mod docs;
pub mod dto;
pub mod entities;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod observability;  // Add this
pub mod repository;
pub mod services;
pub mod state;
pub mod utils;
```

**Update `src/main.rs` to initialize tracing:**

```rust
use actix_web::{middleware, web, App, HttpServer};
use actix_web_template::{
    config::Settings,
    docs::openapi::ApiDoc,
    handlers,
    middleware::{rate_limit, security_headers},
    observability,  // Add this
    state::AppState,
    utils::tracing::init_tracing,
};
use sea_orm::Database;
use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let settings = Arc::new(Settings::load()?);

    // Initialize tracing
    init_tracing(&settings.logging.level, &settings.logging.format);

    // Initialize OpenTelemetry tracer if endpoint is provided
    let tracer_provider = if let Ok(otlp_endpoint) = std::env::var("OTLP_ENDPOINT") {
        tracing::info!("Initializing OpenTelemetry with endpoint: {}", otlp_endpoint);
        Some(observability::init_tracer("actix-web-template", &otlp_endpoint)?)
    } else {
        tracing::warn!("OTLP_ENDPOINT not set, skipping OpenTelemetry initialization");
        None
    };

    // Set up tracing subscriber with OpenTelemetry layer
    if tracer_provider.is_some() {
        let telemetry = tracing_opentelemetry::layer()
            .with_tracer(tracer_provider.clone().unwrap().tracer("actix-web-template"));

        tracing_subscriber::registry()
            .with(telemetry)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Database connection
    let db = Database::connect(&settings.database.url).await?;
    tracing::info!("Database connected successfully");

    // Application state
    let app_state = AppState::new(settings.clone(), db);

    // Start HTTP server
    let server_host = settings.server.host.clone();
    let server_port = settings.server.port;

    tracing::info!("Starting server at {}:{}", server_host, server_port);

    let server = HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http://localhost")
                    || origin.as_bytes().starts_with(b"http://127.0.0.1")
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(security_headers::SecurityHeaders)
            .wrap(rate_limit::rate_limiter())
            .wrap(cors)
            // API routes
            .service(
                web::scope("/api/v1")
                    .service(handlers::memos::list_memos)
                    .service(handlers::memos::get_memo)
                    .service(handlers::memos::create_memo)
                    .service(handlers::memos::update_memo)
                    .service(handlers::memos::patch_memo)
                    .service(handlers::memos::delete_memo)
                    .service(handlers::memos::toggle_complete),
            )
            // Web routes
            .service(handlers::web::index)
            .service(handlers::web::list_memos_fragment)
            .service(handlers::web::new_memo_form)
            .service(handlers::web::create_memo_web)
            .service(handlers::web::get_memo_web)
            .service(handlers::web::edit_memo_form)
            .service(handlers::web::update_memo_web)
            .service(handlers::web::delete_memo_web)
            .service(handlers::web::toggle_complete_web)
            // Health and monitoring
            .service(handlers::health::health)
            .service(handlers::health::ready)
            // API documentation
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            // Static files
            .service(actix_files::Files::new("/static", "./static"))
    })
    .bind((server_host.as_str(), server_port))?
    .run();

    // Graceful shutdown
    let server_handle = server.handle();

    ctrlc::set_handler(move || {
        tracing::info!("Received shutdown signal");
        server_handle.stop(true);
    })?;

    server.await?;

    // Shutdown tracer
    if let Some(provider) = tracer_provider {
        observability::shutdown_tracer(provider);
    }

    Ok(())
}
```

**Add ctrlc dependency to `Cargo.toml`:**

```toml
[dependencies]
# ... existing dependencies ...
ctrlc = "0.8"
```

### Step 7: Verify Tracing Works

**Rebuild and restart:**

```bash
# Stop services
docker-compose down

# Rebuild app with new observability code
docker-compose build app

# Start everything
docker-compose up -d

# Wait for startup
sleep 20
```

**Generate traffic:**

```bash
# Create some requests
for i in {1..5}; do
  curl -s http://localhost:3737/api/v1/memos > /dev/null
done
```

**Check Jaeger:**

Open http://localhost:16686 and:
1. Select service: `actix-web-template`
2. Click "Find Traces"
3. You should see traces for your requests!

**Click on a trace to see:**
- Span timeline (how long each operation took)
- Span hierarchy (which operations called which)
- Tags and logs
- Latency breakdown

### Step 8: Understanding Prometheus Metrics

**Access Prometheus:**

```bash
open http://localhost:9090
```

**If metrics endpoint is configured, you'll see:**

Prometheus scrapes metrics from `http://memos-app:3737/metrics` (configured in `observability/prometheus.yml`).

**Common metrics to query:**

```promql
# Request rate (requests per second)
rate(http_requests_total[1m])

# Error rate
rate(http_requests_total{status=~"5.."}[1m])

# Request duration (p95, p99)
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Active database connections
db_connections_active
```

**Note:** The full Prometheus metrics implementation requires `actix-web-prom` middleware setup, which may not be fully configured yet. This is an optional enhancement.

### Step 9: Exploring Grafana Dashboards

**Access Grafana:**

```bash
open http://localhost:3001
# Login: admin / admin
```

**What Grafana provides:**
- Visual dashboards
- Multiple data sources (Prometheus, Loki, Jaeger)
- Alerts and notifications
- Variables and templating

**Pre-configured datasources** (in `observability/grafana/provisioning/`):
- Prometheus (metrics)
- Loki (logs)
- Jaeger (traces)

**Create a simple dashboard:**

1. Click "+" → "Dashboard"
2. Add panel
3. Select Prometheus datasource
4. Enter query: `rate(http_requests_total[1m])`
5. Set panel title: "Request Rate"
6. Click "Apply"

### Step 10: Viewing Logs in Loki

**Loki** aggregates logs from all containers.

**View logs in Grafana:**

1. Navigate to "Explore" (compass icon)
2. Select "Loki" datasource
3. Click "Log browser"
4. Select container: `{container_name="memos-app"}`
5. Run query

**LogQL query examples:**

```logql
# All logs from app
{container_name="memos-app"}

# Error logs only
{container_name="memos-app"} |= "error"

# Logs from specific endpoint
{container_name="memos-app"} |= "/api/v1/memos"

# Rate of error logs
rate({container_name="memos-app"} |= "error" [5m])
```

## Checkpoint

Verify your observability stack is working:

```bash
# 1. All services running
docker-compose ps
# Expected: All healthy/Up

# 2. Generate traffic
curl http://localhost:3737/api/v1/memos

# 3. Check Jaeger has traces
curl -s "http://localhost:16686/api/services" | jq
# Expected: ["actix-web-template"]

# 4. Check Prometheus targets
curl -s "http://localhost:9090/api/v1/targets" | jq '.data.activeTargets[].labels.job'

# 5. Check Grafana health
curl -s "http://localhost:3001/api/health" | jq
# Expected: {"database": "ok"}
```

## Common Issues and Solutions

### Issue: No traces in Jaeger

**Cause:** OpenTelemetry not initialized or OTLP_ENDPOINT not set

**Solution:**

```bash
# Check environment variable
docker-compose exec app env | grep OTLP

# Should show:
# OTLP_ENDPOINT=http://jaeger:4317

# Check app logs
docker-compose logs app | grep -i "opentelemetry\|otlp"
```

### Issue: Prometheus shows no targets

**Cause:** Prometheus config file missing or app not exposing /metrics

**Solution:**

```bash
# Check if config exists
ls observability/prometheus.yml

# Check if app exposes metrics
curl http://localhost:3737/metrics
```

### Issue: Grafana can't connect to datasources

**Cause:** Datasource provisioning files missing

**Solution:**

```bash
# Check provisioning directory exists
ls -R observability/grafana/provisioning/

# Restart Grafana
docker-compose restart grafana
```

## Summary

In this chapter, you learned:

### Observability Concepts

- **Three pillars**: Traces, Metrics, Logs
- **Distributed tracing**: Track requests across system boundaries
- **Metrics collection**: Quantitative measurements over time
- **Log aggregation**: Centralized logging for analysis

### Technology Stack

- **OpenTelemetry**: Vendor-neutral instrumentation
- **Jaeger**: Distributed tracing visualization
- **Prometheus**: Metrics collection and storage
- **Grafana**: Unified visualization dashboards
- **Loki**: Log aggregation and querying

### Implementation

- **Tracing integration**: OpenTelemetry + tracing-opentelemetry bridge
- **OTLP export**: Send traces to Jaeger via gRPC
- **Metrics endpoint**: Expose Prometheus-compatible metrics
- **Dashboard creation**: Visualize data in Grafana
- **Log queries**: Use LogQL to search logs in Loki

### Key Takeaways

1. **Observability is critical** for production systems
2. **OpenTelemetry is the standard** (vendor-neutral)
3. **Traces show request flow** and latency breakdown
4. **Metrics show trends** over time
5. **Logs provide context** for debugging
6. **Grafana unifies** all three pillars in one UI

## Next Steps

In the final chapter, you'll:

- **Chapter 18: Documentation and Next Steps**: Finalize documentation, review the complete architecture, and explore future enhancements

### Optional Exercises

1. **Add custom metrics**: Track business metrics (memos created, completion rate)
2. **Create alerts**: Set up Grafana alerts for high error rates
3. **Custom dashboards**: Build comprehensive monitoring dashboard
4. **Trace sampling**: Implement intelligent trace sampling for high-traffic apps
5. **Log correlation**: Link traces to logs using trace IDs

### Additional Resources

- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [LogQL Documentation](https://grafana.com/docs/loki/latest/logql/)
- [Rust OpenTelemetry](https://github.com/open-telemetry/opentelemetry-rust)

---

**Congratulations!** Your application now has production-grade observability. You can track every request, monitor performance metrics, and debug issues using traces and logs.
