# Observability Stack

This document describes the observability stack for the Actix Web Memos application, including metrics, tracing, and logging.

## Overview

The application provides comprehensive observability through:

- **Metrics**: Prometheus metrics exposed at `/metrics`
- **Tracing**: OpenTelemetry traces exported to Jaeger
- **Logging**: Structured logs with tracing-subscriber
- **Dashboards**: Pre-configured Grafana dashboards

## Table of Contents

1. [Quick Start](#quick-start)
2. [Metrics](#metrics)
3. [Distributed Tracing](#distributed-tracing)
4. [Logging](#logging)
5. [Grafana Dashboards](#grafana-dashboards)
6. [Configuration](#configuration)
7. [Troubleshooting](#troubleshooting)

## Quick Start

### Running with Observability Stack

1. Uncomment the observability services in `docker-compose.yml`:
   - `jaeger` (distributed tracing)
   - `prometheus` (metrics collection)
   - `grafana` (visualization)
   - `loki` (log aggregation)

2. Uncomment the volumes:
   - `prometheus_data`
   - `grafana_data`
   - `loki_data`

3. Add `OTLP_ENDPOINT` to the `app` service environment:
   ```yaml
   OTLP_ENDPOINT: http://jaeger:4317
   ```

4. Start the stack:
   ```bash
   docker-compose up -d
   ```

5. Access the UIs:
   - Application: http://localhost:3737
   - Metrics: http://localhost:3737/metrics
   - Jaeger UI: http://localhost:16686
   - Prometheus: http://localhost:9090
   - Grafana: http://localhost:3000 (admin/admin)

### Running Locally with Metrics Only

The application always exposes Prometheus metrics at `/metrics` endpoint, even without the full observability stack:

```bash
cargo run
curl http://localhost:3737/metrics
```

## Metrics

### Prometheus Metrics Endpoint

The application exposes metrics in Prometheus format at `/metrics`:

```bash
curl http://localhost:3737/metrics
```

### Available Metrics

The metrics exporter uses OpenTelemetry Prometheus exporter with the following configuration:

- **Histogram boundaries**: 0.001, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0 seconds

#### Built-in Metrics

- `process_*`: Process-level metrics (CPU, memory, etc.)
- `rust_*`: Rust runtime metrics

#### Application Metrics

To add custom application metrics, use the OpenTelemetry Metrics API:

```rust
use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry::KeyValue;

// Get the meter from metrics exporter
let meter = metrics_exporter.meter_provider().meter("actix-web-memos");

// Create a counter
let counter = meter.u64_counter("requests_total").init();
counter.add(1, &[KeyValue::new("endpoint", "/api/v1/memos")]);

// Create a histogram
let histogram = meter.f64_histogram("request_duration_seconds").init();
histogram.record(0.123, &[KeyValue::new("endpoint", "/api/v1/memos")]);
```

### Prometheus Configuration

The Prometheus scrape configuration is located in `observability/prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'actix-web-memos'
    scrape_interval: 10s
    static_configs:
      - targets: ['app:3737']
    metrics_path: '/metrics'
```

## Distributed Tracing

### OpenTelemetry Integration

The application supports OpenTelemetry tracing with OTLP export to Jaeger.

### Enabling Tracing

Set the `OTLP_ENDPOINT` environment variable:

```bash
export OTLP_ENDPOINT=http://localhost:4317
cargo run
```

Or in Docker Compose:

```yaml
environment:
  OTLP_ENDPOINT: http://jaeger:4317
```

### Viewing Traces

1. Open Jaeger UI: http://localhost:16686
2. Select service: `actix-web-memos`
3. Click "Find Traces"

### Trace Context

Traces are automatically created for:
- HTTP requests (via `tracing-actix-web` middleware)
- Database queries (via SeaORM with `sqlx_logging`)
- Custom spans using `#[tracing::instrument]`

### Adding Custom Spans

```rust
use tracing::{info, instrument};

#[instrument(skip(db))]
async fn process_memo(db: &DatabaseConnection, id: Uuid) -> Result<Memo, AppError> {
    info!("Processing memo {}", id);
    // Implementation
}
```

## Logging

### Structured Logging

The application uses `tracing-subscriber` for structured logging with:

- **Format**: JSON or pretty (configurable via `LOG_FORMAT`)
- **Level**: Controlled by `RUST_LOG` environment variable
- **Fields**: timestamp, level, target, message, span context

### Log Levels

```bash
# Info level (default)
RUST_LOG=info cargo run

# Debug level for specific modules
RUST_LOG=actix_web_template=debug,sea_orm=debug cargo run

# Trace level for all
RUST_LOG=trace cargo run
```

### Log Aggregation with Loki

When Loki is enabled, logs can be queried in Grafana:

1. Open Grafana: http://localhost:3000
2. Select Loki datasource
3. Use LogQL queries:
   ```logql
   {container="memos-app"} |= "error"
   {container="memos-app"} | json | level="ERROR"
   ```

## Grafana Dashboards

### Pre-configured Dashboards

The observability stack includes a pre-configured dashboard for the Memos application:

**Memos App Dashboard** (`memos-app-dashboard`)
- HTTP request rate
- Success rate (availability)
- Request latency (p50, p95, p99)

### Accessing Dashboards

1. Open Grafana: http://localhost:3000
2. Login: `admin` / `admin`
3. Navigate to Dashboards → Memos App Dashboard

### Dashboard Provisioning

Dashboards are auto-provisioned from `observability/grafana/dashboards/`:
- Place JSON dashboard files in this directory
- They will be automatically imported on Grafana startup
- Dashboards can be edited in the UI

### Creating Custom Dashboards

1. Create dashboard in Grafana UI
2. Export as JSON (Share → Export → Save to file)
3. Save to `observability/grafana/dashboards/my-dashboard.json`
4. Restart Grafana

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `OTLP_ENDPOINT` | OpenTelemetry OTLP endpoint | None | No |
| `RUST_LOG` | Log level filter | `info` | No |
| `LOG_FORMAT` | Log format (pretty/json) | `pretty` | No |

### Observability Stack Components

| Component | Port | Description |
|-----------|------|-------------|
| Application | 3737 | Main application and metrics endpoint |
| Jaeger UI | 16686 | Distributed tracing UI |
| Jaeger OTLP | 4317 | OTLP gRPC receiver |
| Prometheus | 9090 | Metrics storage and query |
| Grafana | 3000 | Visualization and dashboards |
| Loki | 3100 | Log aggregation |

### File Locations

```
observability/
├── prometheus.yml                          # Prometheus configuration
├── grafana/
│   ├── provisioning/
│   │   ├── datasources/
│   │   │   └── datasources.yml            # Auto-provision datasources
│   │   └── dashboards/
│   │       └── dashboards.yml             # Dashboard provider config
│   └── dashboards/
│       └── memos-app.json                 # Memos dashboard
```

## Troubleshooting

### Metrics Endpoint Returns Empty

Check that the metrics exporter is initialized:

```rust
let metrics_exporter = MetricsExporter::default();
```

And added to app data:

```rust
.app_data(web::Data::new(metrics_exporter.clone()))
```

### Traces Not Appearing in Jaeger

1. Verify `OTLP_ENDPOINT` is set correctly
2. Check Jaeger is running: `docker ps | grep jaeger`
3. Verify connectivity: `curl http://localhost:16686`
4. Check application logs for OpenTelemetry errors

### Prometheus Not Scraping Metrics

1. Check Prometheus targets: http://localhost:9090/targets
2. Verify app service is accessible from Prometheus container
3. Check `observability/prometheus.yml` configuration
4. Verify `/metrics` endpoint is accessible

### Grafana Dashboard Not Loading

1. Check datasource configuration: Grafana → Configuration → Data Sources
2. Test Prometheus connection in Grafana
3. Verify dashboard JSON is valid
4. Check Grafana logs: `docker logs memos-grafana`

### High Cardinality Metrics

Avoid high-cardinality labels (e.g., user IDs, UUIDs) in metrics:

```rust
// Bad - high cardinality
counter.add(1, &[KeyValue::new("memo_id", memo.id.to_string())]);

// Good - low cardinality
counter.add(1, &[KeyValue::new("operation", "create_memo")]);
```

## Best Practices

### Metrics

1. **Use appropriate metric types**:
   - Counter: Monotonically increasing values (requests, errors)
   - Gauge: Values that can go up and down (active connections, queue size)
   - Histogram: Distribution of values (latency, response size)

2. **Keep cardinality low**: Limit the number of unique label combinations

3. **Use consistent naming**: Follow Prometheus naming conventions
   - `_total` suffix for counters
   - `_seconds` suffix for durations
   - Base unit names (seconds, bytes, not milliseconds or megabytes)

### Tracing

1. **Use spans judiciously**: Don't create spans for every function
2. **Add relevant attributes**: Context that helps debugging
3. **Skip sensitive data**: Use `skip` in `#[instrument]` for passwords, tokens
4. **Correlate logs and traces**: Use span IDs in log messages

### Logging

1. **Use appropriate log levels**:
   - ERROR: Actionable errors requiring attention
   - WARN: Potential issues, degraded functionality
   - INFO: Important business events
   - DEBUG: Detailed diagnostic information
   - TRACE: Very verbose, fine-grained details

2. **Include context**: Add relevant fields to log messages
3. **Avoid sensitive data**: Don't log passwords, tokens, PII
4. **Use structured logging**: JSON format for production

## Further Reading

- [OpenTelemetry Rust Documentation](https://docs.rs/opentelemetry/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Grafana Documentation](https://grafana.com/docs/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Tracing Documentation](https://docs.rs/tracing/)