# Actix Web Tutorial: Build a Production-Ready Web Application

Welcome to this comprehensive tutorial on building a production-ready web application with Rust and Actix Web!

## What You'll Build

A complete memo management application featuring:
- **REST API** with full CRUD operations
- **Web UI** with server-side rendering
- **PostgreSQL** database with migrations
- **Security** features (rate limiting, XSS prevention, security headers)
- **Testing** suite (unit, integration, end-to-end)
- **Docker** deployment
- **CI/CD** pipeline with GitHub Actions
- **Observability** with Jaeger, Prometheus, and Grafana

## Prerequisites

- Basic Rust knowledge (ownership, traits, async/await)
- Understanding of web concepts (HTTP, REST, databases)
- Familiarity with command line

## Tutorial Structure

The tutorial currently ships with eighteen chapters (0-17) that guide you from environment setup through a complete production-ready application with observability. Each chapter builds on previous concepts and should be completed sequentially.

### Part 1: Foundation (Chapters 0-4)

- **[Chapter 0: Prerequisites and Environment Setup](chapter-00.md)** - Prepare your tooling, database, and project workspace
- **[Chapter 1: Core Application Setup](chapter-01.md)** - Create the baseline Actix Web application and project skeleton
- **[Chapter 2: Database Integration with SeaORM](chapter-02.md)** - Connect to PostgreSQL and configure SeaORM
- **[Chapter 3: Error Handling and Middleware](chapter-03.md)** - Centralize error responses and add essential middleware
- **[Chapter 4: Enhanced Health Checks and Readiness Probes](chapter-04.md)** - Build robust health and readiness endpoints

### Part 2: Core Architecture (Chapters 5-7)

- **[Chapter 5: Data Transfer Objects and Validation](chapter-05.md)** - Define DTOs and enforce request validation
- **[Chapter 6: Repository Layer - Database Operations](chapter-06.md)** - Encapsulate database logic in the repository layer
- **[Chapter 7: Service Layer - Business Logic and Transactions](chapter-07.md)** - Implement business logic and transactional workflows

### Part 3: REST API (Chapters 8-9)

- **[Chapter 8: REST API Handlers](chapter-08.md)** - Build REST endpoints for CRUD operations
- **[Chapter 9: OpenAPI Documentation](chapter-09.md)** - Auto-generate API documentation with Swagger UI

### Part 4: Web UI (Chapters 10-12)

- **[Chapter 10: Askama Templates - Server-Side Rendering](chapter-10.md)** - Create type-safe HTML templates
- **[Chapter 11: Static Assets and Styling](chapter-11.md)** - Add CSS and configure static file serving
- **[Chapter 12: Web Page Handlers - Building the UI](chapter-12.md)** - Implement server-rendered web pages

### Part 5: Security & Quality (Chapters 13-14)

- **[Chapter 13: Security Enhancements](chapter-13.md)** - Add rate limiting, XSS prevention, and security headers
- **[Chapter 14: Testing Strategy](chapter-14.md)** - Implement comprehensive test coverage

### Part 6: Deployment & Operations (Chapters 15-17)

- **[Chapter 15: Docker Deployment](chapter-15.md)** - Containerize your application with multi-stage builds
- **[Chapter 16: CI/CD Pipeline](chapter-16.md)** - Automate testing, linting, and releases with GitHub Actions
- **[Chapter 17: Observability Stack](chapter-17.md)** - Add distributed tracing, metrics, and log aggregation

### Coming Soon

- **Chapter 18: Documentation and Next Steps** - Final architecture documentation and future enhancements

## How to Use This Tutorial

1. **Sequential Learning**: Follow chapters in order, as each builds on previous concepts
2. **Hands-on Practice**: Type out the code yourself rather than copying
3. **Checkpoints**: Verify your work at each checkpoint before proceeding
4. **Exploration**: Experiment with variations and extensions
5. **Reference**: Use the completed code in the main repository as reference

## Estimated Time

- **Foundation**: 2-3 hours
- **Architecture**: 2-3 hours
- **API Development**: 2-3 hours
- **Web UI**: 2-3 hours
- **Security & Testing**: 2-3 hours
- **Deployment & CI/CD**: 2-3 hours
- **Observability**: 1-2 hours
- **Documentation**: 1 hour

**Total**: 14-21 hours depending on experience level

## Getting Help

- **GitHub Issues**: Report problems or ask questions
- **Code Reference**: See the main project for completed code
- **Documentation**: Check CLAUDE.md for architecture details
- **Troubleshooting**: See TROUBLESHOOTING.md for common issues

## Ready to Start?

Begin with **[Chapter 0: Prerequisites and Environment Setup](chapter-00.md)**

---

**Note**: This tutorial assumes you're building the application from scratch. If you want to explore the completed code, see the main README.md in the project root.
