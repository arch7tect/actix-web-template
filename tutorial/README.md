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

This tutorial is organized into 18 chapters, each building on the previous:

### Part 1: Foundation
- **[Chapter 0: Prerequisites and Environment Setup](chapter-00.md)** - Set up your development environment
- **[Chapter 1: Core Application Setup](chapter-01.md)** - Create a minimal Actix Web server
- **[Chapter 2: Database Integration](chapter-02.md)** - Connect to PostgreSQL with SeaORM
- **[Chapter 3: Error Handling and Middleware](chapter-03.md)** - Centralized errors and security
- **[Chapter 4: Health Checks](chapter-04.md)** - Monitoring endpoints

### Part 2: Core Architecture
- **[Chapter 5: DTOs and Validation](chapter-05.md)** - Type-safe data transfer objects
- **[Chapter 6: Repository Layer](chapter-06.md)** - Database access patterns
- **[Chapter 7: Service Layer](chapter-07.md)** - Business logic layer

### Part 3: REST API
- **[Chapter 8: REST API Handlers](chapter-08.md)** - Complete CRUD endpoints
- **[Chapter 9: OpenAPI Documentation](chapter-09.md)** - Auto-generated API docs

### Part 4: Web UI
- **[Chapter 10: Askama Templates](chapter-10.md)** - Server-side rendering
- **[Chapter 11: Web Page Handlers](chapter-11.md)** - HTML endpoints with progressive enhancement
- **[Chapter 12: Static Assets](chapter-12.md)** - CSS and static file serving

### Part 5: Security and Quality
- **[Chapter 13: Security Enhancements](chapter-13.md)** - Rate limiting, XSS prevention
- **[Chapter 14: Testing Strategy](chapter-14.md)** - Comprehensive test coverage

### Part 6: Deployment
- **[Chapter 15: Docker Deployment](chapter-15.md)** - Containerization and orchestration

### Part 7: CI/CD
- **[Chapter 16: CI/CD Pipeline](chapter-16.md)** - Automated testing and deployment

### Part 8: Observability
- **[Chapter 17: Observability Stack](chapter-17.md)** - Tracing and metrics

### Part 9: Documentation
- **[Chapter 18: Documentation and Next Steps](chapter-18.md)** - Final polish and future enhancements

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