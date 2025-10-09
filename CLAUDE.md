# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an Actix Web template project written in Rust. Currently in early development stage with minimal dependencies and a basic structure.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Run with release optimizations
cargo build --release
cargo run --release

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapd
```

## Project Structure

- `src/main.rs` - Application entry point (currently a minimal placeholder)
- `Cargo.toml` - Project configuration and dependencies (Rust edition 2024)

## Architecture Notes

This is a template project intended for Actix Web development. The current codebase is minimal and will need to be expanded with:
- Actix Web framework setup and routing
- Application state management
- Middleware configuration
- Handler functions for endpoints
- Database integration (if needed)
- Configuration management
- Error handling patterns

When adding Actix Web functionality, follow the typical layered architecture:
- **Handlers** - HTTP request/response logic
- **Services** - Business logic layer
- **Repository** - Database access layer
- **Entities** - SeaORM database models
- **DTOs** - Data transfer objects (separate from entities)
- **Middleware** - Cross-cutting concerns (logging, auth, etc.)
- **Config** - Application configuration

## Coding Rules

- **No emojis** in code, logs, error messages, or API responses (HTML templates are OK)
- **Prefer tracing over comments** - use `tracing::debug!()`, `tracing::info!()`, etc.
- **Comments explain "why", not "what"** - code should be self-documenting
- **Naming**: functions/variables `snake_case`, types `PascalCase`, constants `SCREAMING_SNAKE_CASE`