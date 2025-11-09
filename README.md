# Actix Web Memos Application

[![Test](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/test.yml)
[![Lint](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/lint.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/lint.yml)
[![Release](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/release.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/release.yml)
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

Learn by building this application from scratch with our comprehensive 18-chapter tutorial.

**Completed: Chapters 0-16** | **Estimated time:** 15-23 hours total

### Part 1: Foundation (Chapters 0-4)

- [Chapter 0: Prerequisites and Environment Setup](tutorial/chapter-00.md)
- [Chapter 1: Core Application Setup](tutorial/chapter-01.md)
- [Chapter 2: Database Integration with SeaORM](tutorial/chapter-02.md)
- [Chapter 3: Error Handling and Middleware](tutorial/chapter-03.md)
- [Chapter 4: Health Checks and Monitoring](tutorial/chapter-04.md)

### Part 2: Core Architecture (Chapters 5-7)

- [Chapter 5: Data Transfer Objects and Validation](tutorial/chapter-05.md)
- [Chapter 6: Repository Layer - Database Operations](tutorial/chapter-06.md)
- [Chapter 7: Service Layer - Business Logic](tutorial/chapter-07.md)

### Part 3: REST API (Chapters 8-9)

- [Chapter 8: REST API Handlers](tutorial/chapter-08.md)
- [Chapter 9: OpenAPI Documentation](tutorial/chapter-09.md)

### Part 4: Web UI (Chapters 10-12)

- [Chapter 10: Askama Templates - Server-Side Rendering](tutorial/chapter-10.md)
- [Chapter 11: Static Assets and Styling](tutorial/chapter-11.md)
- [Chapter 12: Web Page Handlers - Building the UI](tutorial/chapter-12.md)

### Part 5: Security & Quality (Chapters 13-14)

- [Chapter 13: Security Enhancements](tutorial/chapter-13.md)
- [Chapter 14: Testing Strategy](tutorial/chapter-14.md)

### Part 6: Deployment & Operations (Chapters 15-18)

- [Chapter 15: Docker Deployment](tutorial/chapter-15.md)
- [Chapter 16: CI/CD Pipeline](tutorial/chapter-16.md)
- Chapter 17: Observability Stack *(coming soon)*
- Chapter 18: Documentation and Next Steps *(coming soon)*

## Quick Start

```bash
# Clone the repository
git clone https://github.com/arch7tect/actix-web-template.git
cd actix-web-template

# Run with Docker Compose
docker-compose up --build

# Access the application
# Web UI:     http://localhost:3737/
# API:        http://localhost:3737/api/v1/
# Swagger UI: http://localhost:3737/swagger-ui/
```

For detailed setup instructions, see [Chapter 0: Prerequisites and Environment Setup](tutorial/chapter-00.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Ready to learn?** Start with [Chapter 0: Prerequisites and Environment Setup](tutorial/chapter-00.md)

**Built with Rust 🦀 | Version 0.2.1**
