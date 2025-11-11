# Actix Web Memos Application

[![Test](https://github.com/arch7tect/actix-web-template/actions/workflows/test.yml/badge.svg)](https://github.com/arch7tect/actix-web-template/actions/workflows/test.yml)
[![Lint](https://github.com/arch7tect/actix-web-template/actions/workflows/lint.yml/badge.svg)](https://github.com/arch7tect/actix-web-template/actions/workflows/lint.yml)
[![Release](https://github.com/arch7tect/actix-web-template/actions/workflows/release.yml/badge.svg)](https://github.com/arch7tect/actix-web-template/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A production-ready web application built with Rust and Actix Web for managing memos. Features a complete REST API, server-side rendered web UI, PostgreSQL database, comprehensive testing, security features, and Docker deployment.

**Built as a comprehensive tutorial** - Learn to build production-ready Rust web applications from scratch.

## Tech Stack

- **[Actix Web 4](https://actix.rs/)** - Fast, pragmatic web framework
- **[PostgreSQL 16](https://www.postgresql.org/)** + **[SeaORM 1.0](https://www.sea-ql.org/SeaORM/)** - Database and async ORM
- **[Askama](https://djc.github.io/askama/)** - Type-safe compile-time templates
- **[utoipa](https://github.com/juhaku/utoipa)** - Auto-generated OpenAPI specs
- **[Tokio](https://tokio.rs/)** - Async runtime

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
