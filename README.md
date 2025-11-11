# Actix Web Memos Application

[![Test](https://github.com/arch7tect/actix-web-template/actions/workflows/test.yml/badge.svg)](https://github.com/arch7tect/actix-web-template/actions/workflows/test.yml)
[![Lint](https://github.com/arch7tect/actix-web-template/actions/workflows/lint.yml/badge.svg)](https://github.com/arch7tect/actix-web-template/actions/workflows/lint.yml)
[![Release](https://github.com/arch7tect/actix-web-template/actions/workflows/release.yml/badge.svg)](https://github.com/arch7tect/actix-web-template/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

**A comprehensive tutorial project** teaching you how to build production-ready Rust web applications from scratch.

This memos management application serves as a complete learning resource, featuring REST API, server-side rendered web UI, PostgreSQL database with migrations, comprehensive testing, security features, observability stack, and Docker deployment.

## Tech Stack

**Core Framework:**
- **[Actix Web 4](https://actix.rs/)** - Fast, pragmatic async web framework
- **[Tokio](https://tokio.rs/)** - Async runtime powering everything

**Database:**
- **[PostgreSQL 16](https://www.postgresql.org/)** - Relational database
- **[SeaORM 1.1](https://www.sea-ql.org/SeaORM/)** - Async ORM with query builder
- **[SeaORM Migrations](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)** - Database schema version control

**Web & Templates:**
- **[Askama](https://djc.github.io/askama/)** - Type-safe compile-time HTML templates
- **[utoipa](https://github.com/juhaku/utoipa)** + **[Swagger UI](https://swagger.io/tools/swagger-ui/)** - OpenAPI spec generation and interactive docs
- **Vanilla JavaScript** - No frontend framework dependencies

**Security & Middleware:**
- **[actix-governor](https://github.com/AaronErhardt/actix-governor)** - Rate limiting
- **[actix-cors](https://docs.rs/actix-cors/)** - CORS middleware
- **[ammonia](https://github.com/rust-ammonia/ammonia)** - HTML sanitization (XSS prevention)
- **[validator](https://github.com/Keats/validator)** - Input validation

**Observability:**
- **[tracing](https://tracing.rs/)** + **[tracing-subscriber](https://docs.rs/tracing-subscriber/)** - Structured logging
- **[OpenTelemetry](https://opentelemetry.io/)** - Distributed tracing and metrics
- **[Prometheus](https://prometheus.io/)** - Metrics collection and monitoring
- **[Jaeger](https://www.jaegertracing.io/)** - Distributed tracing backend
- **[Grafana](https://grafana.com/)** - Observability dashboards
- **[Loki](https://grafana.com/oss/loki/)** - Log aggregation

**Development & Deployment:**
- **[Docker](https://www.docker.com/)** + **[Docker Compose](https://docs.docker.com/compose/)** - Containerization
- **[GitHub Actions](https://github.com/features/actions)** - CI/CD pipelines (test, lint, release)

## Tutorial

**Learn by building this application from scratch!**

This project includes a comprehensive **19-chapter tutorial** (Chapters 0-18) that guides you through building a production-ready Rust web application from the ground up.

**[📚 Start the Tutorial](tutorial/README.md)** - Complete chapter index with learning path and time estimates

**Quick links:**
- [Chapter 0: Prerequisites and Environment Setup](tutorial/chapter-00.md) - Start here
- [Chapter 18: Adding Tags and Advanced Filtering](tutorial/chapter-18.md) - Latest chapter

## Quick Start

```bash
# Clone the repository
git clone https://github.com/arch7tect/actix-web-template.git
cd actix-web-template

# Run with Docker Compose (includes full observability stack)
docker-compose up --build

# Wait for all services to start (30-60 seconds)
docker-compose ps
```

### Application Access

Once running, access these services:

**Main Application:**
- **Web UI**: http://localhost:3737/
- **REST API**: http://localhost:3737/api/v1/memos
- **Swagger UI**: http://localhost:3737/swagger-ui/
- **Health Check**: http://localhost:3737/health
- **Metrics**: http://localhost:3737/metrics

**Observability Stack:**
- **Grafana**: http://localhost:3001 (admin/admin)
  - Pre-configured dashboards for logs, metrics, and traces
  - All datasources ready: Prometheus, Loki, Jaeger
- **Prometheus**: http://localhost:9090
  - Metrics explorer and PromQL queries
- **Jaeger**: http://localhost:16686
  - Distributed tracing UI
- **Loki**: http://localhost:3100
  - Log aggregation (access via Grafana Explore)
- **PostgreSQL**: localhost:5432
  - Database: `memos_db` (postgres/postgres)

For detailed setup instructions, see [Chapter 0: Prerequisites and Environment Setup](tutorial/chapter-00.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Ready to learn?** Start with [Chapter 0: Prerequisites and Environment Setup](tutorial/chapter-00.md)

**Built with Rust 🦀 | Version 0.2.5**
