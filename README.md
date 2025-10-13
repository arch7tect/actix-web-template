# Actix Web Memos Application

[![Test](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/test.yml)
[![Lint](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/lint.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/lint.yml)
[![Release](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/release.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A modern, production-ready web application built with Rust and Actix Web for managing memos. Features a full REST API, server-side rendered web UI with vanilla JavaScript enhancements, comprehensive testing, and robust security features.

## Features

- **Full REST API** with pagination, filtering, and sorting
- **Server-side rendered HTML** with Askama templates
- **Vanilla JavaScript** for progressive enhancement
- **PostgreSQL database** with SeaORM ORM
- **OpenAPI/Swagger documentation** at `/swagger-ui/`
- **Health checks** for monitoring and Kubernetes readiness
- **Security features**:
  - Rate limiting per IP address
  - Security headers (XSS, Frame Options, etc.)
  - Input sanitization (XSS prevention)
  - CORS configuration
  - Request size limits
- **Performance optimizations**:
  - Gzip and Brotli compression
  - Connection pooling
  - Async/await throughout
- **Comprehensive testing**:
  - Unit tests for services
  - Integration tests for API and web handlers
  - Repository tests with test database
- **Structured logging** with tracing
- **Docker support** with multi-stage builds

## Tech Stack

- **Backend Framework**: [Actix Web 4](https://actix.rs/) - Fast, pragmatic web framework for Rust
- **Database**: [PostgreSQL 16](https://www.postgresql.org/) - Reliable, feature-rich relational database
- **ORM**: [SeaORM 1.0](https://www.sea-ql.org/SeaORM/) - Async & dynamic ORM with compile-time safety
- **Templates**: [Askama](https://djc.github.io/askama/) - Type-safe compile-time templates
- **API Documentation**: [utoipa](https://github.com/juhaku/utoipa) - Auto-generated OpenAPI specs
- **Validation**: [validator](https://github.com/Keats/validator) - Struct validation with derive macros
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) - Application-level tracing
- **Runtime**: [Tokio](https://tokio.rs/) - Async runtime for Rust

## Project Structure

```
actix-web-template/
├── src/
│   ├── config/          # Application configuration
│   ├── docs/            # OpenAPI documentation
│   ├── dto/             # Data Transfer Objects with validation
│   ├── entities/        # SeaORM database entities
│   ├── error/           # Error types and handlers
│   ├── handlers/        # HTTP request handlers (API + Web)
│   ├── middleware/      # Custom middleware (rate limiting, security)
│   ├── repository/      # Database access layer
│   ├── services/        # Business logic layer
│   ├── utils/           # Utility functions (sanitization, tracing)
│   ├── state.rs         # Application state
│   ├── lib.rs           # Library root
│   └── main.rs          # Application entry point
├── migration/           # Database migrations
├── templates/           # Askama HTML templates
│   ├── base.html        # Base layout
│   ├── pages/           # Full page templates
│   ├── components/      # Reusable components
│   └── partials/        # Header, footer, etc.
├── static/              # Static assets (CSS, JS)
│   └── css/
│       └── style.css
├── tests/               # Integration and repository tests
├── .env.example         # Example environment variables
├── Cargo.toml           # Rust dependencies
├── Dockerfile           # Multi-stage Docker build
├── docker-compose.yml   # Docker Compose configuration
└── README.md            # This file
```

## Getting Started

### Prerequisites

- **Rust 1.75+** - Install from [rustup.rs](https://rustup.rs/)
- **PostgreSQL 16** - Or use Docker Compose
- **SeaORM CLI** - For database migrations

```bash
# Install SeaORM CLI
cargo install sea-orm-cli
```

### Installation

1. **Clone the repository**

```bash
git clone <your-repo-url>
cd actix-web-template
```

2. **Set up environment variables**

```bash
cp .env.example .env
# Edit .env with your configuration
```

3. **Start PostgreSQL** (using Docker Compose)

```bash
docker-compose up -d postgres
```

4. **Run database migrations**

```bash
cd migration
cargo run
# Or use: sea-orm-cli migrate up
```

5. **Build and run the application**

```bash
cargo build --release
cargo run --release
```

The application will be available at:
- **Web UI**: http://localhost:3737/
- **API**: http://localhost:3737/api/v1/
- **Swagger UI**: http://localhost:3737/swagger-ui/
- **Health Check**: http://localhost:3737/health

## Development

### Running in Development Mode

```bash
# Start database
docker-compose up -d postgres

# Run with hot-reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Or run normally
cargo run
```

### Database Commands

```bash
# Create a new migration
sea-orm-cli migrate generate migration_name

# Run migrations
sea-orm-cli migrate up

# Rollback last migration
sea-orm-cli migrate down

# Generate entities from database
sea-orm-cli generate entity -o src/entities
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run all checks
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Testing

The project includes comprehensive test coverage across multiple layers:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test file
cargo test --test api_tests

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Test Categories

- **Unit Tests**: Service layer business logic (`tests/service_tests.rs`)
- **Repository Tests**: Database operations (`tests/repository_tests.rs`)
- **API Tests**: REST API endpoints (`tests/api_tests.rs`)
- **Web Tests**: HTML endpoints and forms (`tests/web_tests.rs`)

See [TESTING.md](TESTING.md) for detailed testing documentation.

## API Documentation

### Interactive Documentation

Visit http://localhost:3737/swagger-ui/ for interactive API documentation powered by Swagger UI.

### API Endpoints

#### Memos API

```
GET    /api/v1/memos              List memos (with pagination, filtering, sorting)
GET    /api/v1/memos/{id}         Get memo by ID
POST   /api/v1/memos              Create new memo
PUT    /api/v1/memos/{id}         Update memo (full update)
PATCH  /api/v1/memos/{id}         Partial update memo
DELETE /api/v1/memos/{id}         Delete memo
PATCH  /api/v1/memos/{id}/complete Toggle memo completion status
```

#### Health & Monitoring

```
GET    /health                     Health check with database status
GET    /ready                      Kubernetes readiness probe
```

### Example API Usage

**Create a memo:**

```bash
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Buy groceries",
    "description": "Milk, eggs, bread",
    "date_to": "2025-10-15T12:00:00Z"
  }'
```

**List memos with pagination:**

```bash
curl "http://localhost:3737/api/v1/memos?limit=10&offset=0&completed=false&sort_by=date_to&order=asc"
```

**Update a memo:**

```bash
curl -X PUT http://localhost:3737/api/v1/memos/{id} \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Buy groceries - Updated",
    "description": "Milk, eggs, bread, cheese",
    "date_to": "2025-10-15T12:00:00Z",
    "completed": false
  }'
```

**Toggle completion:**

```bash
curl -X PATCH http://localhost:3737/api/v1/memos/{id}/complete
```

**Delete a memo:**

```bash
curl -X DELETE http://localhost:3737/api/v1/memos/{id}
```

## Web UI

The application provides a server-side rendered web interface with vanilla JavaScript enhancements:

- **Homepage** (`/`): List all memos with filtering and sorting
- **Create Form**: Add new memos
- **Edit Form**: Modify existing memos
- **Toggle Complete**: Mark memos as done/undone
- **Delete**: Remove memos with confirmation

The web UI uses:
- **Askama** templates for server-side rendering
- **Vanilla JavaScript** for dynamic interactions
- **CSS** for styling with responsive design

## Configuration

### Environment Variables

All configuration is done through environment variables. See `.env.example` for all available options.

#### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `127.0.0.1` | Server bind address |
| `SERVER_PORT` | `3737` | Server port |
| `APP_ENV` | `development` | Environment: development/production |

#### Database Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | - | PostgreSQL connection string |
| `DATABASE_MAX_CONNECTIONS` | `10` | Max database connections in pool |
| `DATABASE_CONNECT_TIMEOUT` | `30` | Connection timeout in seconds |

#### Logging Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Logging level (trace/debug/info/warn/error) |
| `LOG_FORMAT` | `pretty` | Log format: pretty/json |

#### Security Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `CORS_ALLOWED_ORIGINS` | `*` | CORS allowed origins (comma-separated) |
| `MAX_REQUEST_SIZE` | `262144` | Max request body size in bytes (256KB) |

#### Features

| Variable | Default | Description |
|----------|---------|-------------|
| `ENABLE_SWAGGER` | `true` | Enable Swagger UI documentation |

## Docker Deployment

### Using Docker Compose (Recommended)

```bash
# Build and start all services
docker-compose up --build

# Run in background
docker-compose up -d --build

# View logs
docker-compose logs -f app

# Stop services
docker-compose down
```

The Docker Compose setup includes:
- PostgreSQL database with persistent volume
- Application container with health checks
- Automatic database initialization
- Network configuration

### Using Docker Manually

```bash
# Build the image
docker build -t actix-web-template .

# Run PostgreSQL
docker run -d \
  --name postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=memos_db \
  -p 5432:5432 \
  postgres:16-alpine

# Run the application
docker run -d \
  --name actix-app \
  -p 3737:3737 \
  -e DATABASE_URL=postgresql://postgres:postgres@postgres:5432/memos_db \
  --link postgres:postgres \
  actix-web-template
```

## Performance

The application includes several performance optimizations:

- **Compression**: Gzip and Brotli compression for responses
- **Connection Pooling**: Database connection pooling with SeaORM
- **Async/Await**: Non-blocking I/O throughout the application
- **Efficient Queries**: Optimized database queries with indexes
- **Static Asset Caching**: Efficient serving of CSS/JS files

See [PERFORMANCE.md](PERFORMANCE.md) for detailed performance documentation and benchmarks.

## Security

Security features implemented:

- **Rate Limiting**: IP-based rate limiting (100 requests per minute by default)
- **Security Headers**:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `X-XSS-Protection: 1; mode=block`
  - `Strict-Transport-Security` (in production)
- **Input Sanitization**: XSS prevention with HTML sanitization
- **CORS**: Configurable CORS policies
- **Request Size Limits**: Protection against large payload attacks
- **SQL Injection Prevention**: SeaORM parameterized queries
- **Error Handling**: Safe error messages (no internal details leaked)

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues and solutions.

### Quick Fixes

**Port already in use:**
```bash
# Kill process on port 3737
lsof -ti:3737 | xargs kill -9
```

**Database connection failed:**
```bash
# Check if PostgreSQL is running
docker-compose ps postgres

# Restart PostgreSQL
docker-compose restart postgres
```

**Migration errors:**
```bash
# Reset database (WARNING: destroys all data)
sea-orm-cli migrate down
sea-orm-cli migrate up
```

## Architecture

The application follows a layered architecture:

1. **Handlers Layer** (`src/handlers/`): HTTP request/response handling
2. **Service Layer** (`src/services/`): Business logic and orchestration
3. **Repository Layer** (`src/repository/`): Database access
4. **Entity Layer** (`src/entities/`): Database models (SeaORM)
5. **DTO Layer** (`src/dto/`): Data transfer objects with validation

This separation ensures:
- Clear separation of concerns
- Easy testing of each layer
- Flexibility to change implementations
- Maintainable and scalable codebase

See [CLAUDE.md](CLAUDE.md) for detailed architecture notes.

## Database Schema

### Memos Table

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `title` | VARCHAR(200) | Memo title (required) |
| `description` | TEXT | Optional detailed description |
| `date_to` | TIMESTAMP WITH TIME ZONE | Due date |
| `completed` | BOOLEAN | Completion status (default: false) |
| `created_at` | TIMESTAMP WITH TIME ZONE | Creation timestamp |
| `updated_at` | TIMESTAMP WITH TIME ZONE | Last update timestamp |

**Indexes:**
- Primary key on `id`
- Index on `completed` for filtering
- Index on `date_to` for sorting by due date
- Index on `created_at` for sorting by creation time

See [MIGRATIONS.md](MIGRATIONS.md) for migration history.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and quality checks (`cargo test && cargo clippy && cargo fmt`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Code Style

- Follow Rust naming conventions (snake_case, PascalCase, SCREAMING_SNAKE_CASE)
- Use `tracing` for logging, not println!
- Write meaningful commit messages
- Add tests for new features
- Update documentation as needed
- No emojis in code, logs, or API responses

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Actix Web](https://actix.rs/)
- Database ORM by [SeaORM](https://www.sea-ql.org/SeaORM/)
- Templates powered by [Askama](https://djc.github.io/askama/)
- API documentation by [utoipa](https://github.com/juhaku/utoipa)

## Support

For issues, questions, or contributions, please:
- Open an issue on GitHub
- Check the [TROUBLESHOOTING.md](TROUBLESHOOTING.md) guide
- Review existing issues and discussions

## Roadmap

Potential future enhancements:

- [ ] User authentication and authorization (JWT)
- [ ] Tags/categories for memos
- [ ] Full-text search
- [ ] File attachments
- [ ] Email notifications for due dates
- [ ] WebSocket support for real-time updates
- [ ] GraphQL API
- [ ] Mobile app (React Native or Flutter)
- [x] CI/CD pipeline (GitHub Actions)
- [ ] Observability stack (Jaeger, Prometheus, Grafana)

---

**Built with Rust 🦀 | Version 0.1.0**