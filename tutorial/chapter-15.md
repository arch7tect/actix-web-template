# Chapter 15: Docker Deployment

## Overview

In this chapter, you'll containerize the Actix Web application using Docker and orchestrate it with Docker Compose. You'll learn about multi-stage builds for Rust applications, container networking, volume management, and health checks.

Docker provides consistent deployment across environments and simplifies the process of running the application with all its dependencies.

## Prerequisites

### Completed Chapters
- Chapters 0-14: Full application, security hardening, and test suite

### Required Knowledge
- Container fundamentals (images, layers, networks, volumes)
- Basic command-line proficiency with Docker CLI
- Understanding of environment variables and secrets from prior chapters

### Required Software
- Docker Engine 20.10 or newer
- Docker Compose v2.0 or newer (bundled with recent Docker Desktop)
- Access to a terminal capable of running shell scripts

**Verify installation:**

```bash
docker --version          # Expect Docker version 24.x or newer
docker compose version    # Expect Docker Compose version v2.x
```

## Learning Objectives

By the end of this chapter, you will:

- Understand multi-stage Docker builds for Rust applications
- Create an optimized Dockerfile for production
- Configure Docker Compose for multi-container applications
- Use .dockerignore for efficient builds
- Implement health checks in containers
- Manage data persistence with volumes
- Configure container networking
- Run migrations automatically on container startup
- Deploy the application with a single command

## Concepts Covered

### Multi-Stage Docker Builds

Multi-stage builds separate the build environment from the runtime environment, resulting in much smaller final images.

**Benefits:**
- **Smaller images**: Only runtime dependencies included (100MB vs 2GB+)
- **Faster deployments**: Less data to transfer and store
- **More secure**: Fewer attack vectors, no build tools in production
- **Cleaner**: No build artifacts or intermediate files

**How it works:**
1. **Build stage**: Use full Rust toolchain to compile application
2. **Runtime stage**: Copy only compiled binaries to minimal base image
3. **Discard**: Build tools, source code, and intermediate files are discarded

### Docker Compose

Docker Compose defines and runs multi-container applications using a YAML configuration file.

**Key concepts:**
- **Services**: Individual containers (app, database, etc.)
- **Networks**: Automatic service discovery and communication
- **Volumes**: Persistent data storage
- **Dependencies**: Define startup order with health checks
- **Environment variables**: Configure services

## Step-by-Step Instructions

### Step 1: Understanding the Dockerfile

Let's examine the multi-stage Dockerfile that's already in the project.

**File: `Dockerfile`**

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

# Build for release
RUN cargo build --release

# Build migration binary (workspace member)
RUN cargo build --release -p migration

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

# Copy the binaries from builder
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

# Create startup script to run migrations then start app
RUN echo '#!/bin/sh\n\
echo "Running database migrations..."\n\
./migration\n\
echo "Starting application..."\n\
exec ./actix-web-template' > /app/entrypoint.sh && \
    chmod +x /app/entrypoint.sh

# Run the startup script
CMD ["/app/entrypoint.sh"]
```

**Key sections explained:**

#### Build Stage

```dockerfile
FROM rust:latest as builder
```
- Uses official Rust image (includes cargo, rustc, etc.)
- Named `builder` so we can reference it later
- Size: ~2GB+ with full toolchain

```dockerfile
COPY Cargo.toml Cargo.lock ./
```
- Copy dependency manifests first
- Enables Docker layer caching for dependencies
- If dependencies don't change, this layer is reused

```dockerfile
RUN cargo build --release
```
- Compile application in release mode
- Optimizations: inlining, dead code elimination, no debug symbols
- Takes 5-10 minutes on first build

#### Runtime Stage

```dockerfile
FROM debian:bookworm-slim
```
- Minimal Debian base image (~80MB)
- Enough for running compiled Rust binaries
- Much smaller than full Rust image

```dockerfile
RUN apt-get update && \
    apt-get install -y \
        libpq5 \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*
```
- `libpq5`: PostgreSQL client library (required by SeaORM)
- `ca-certificates`: SSL/TLS certificates for HTTPS connections
- `curl`: Used for health checks
- Clean up apt cache to reduce image size

```dockerfile
COPY --from=builder /app/target/release/actix-web-template .
```
- Copy only compiled binary from build stage
- Source code and build tools are not included

#### Health Check

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3737/health || exit 1
```
- **interval**: Check every 30 seconds
- **timeout**: Health check must complete within 3 seconds
- **start-period**: Wait 5 seconds before first check (app startup time)
- **retries**: Mark unhealthy after 3 consecutive failures
- Uses `/health` endpoint we created in Chapter 4

#### Startup Script

```dockerfile
RUN echo '#!/bin/sh\n\
echo "Running database migrations..."\n\
./migration\n\
echo "Starting application..."\n\
exec ./actix-web-template' > /app/entrypoint.sh
```
- Run migrations before starting the app
- Ensures database schema is up to date
- `exec` replaces shell with app process (proper signal handling)

**Why this approach?**
- Migrations run automatically on container startup
- No manual migration step needed in deployment
- Database is always in sync with application version

### Step 2: Understanding .dockerignore

The `.dockerignore` file tells Docker which files to exclude from the build context.

**File: `.dockerignore`**

```
# Build artifacts
target/

# IDE files
.idea/
.vscode/
*.swp
*.swo
*.iml
*~

# Environment files
.env
.env.local
!.env.example
!.env.production

# Git
.git/
.gitignore

# CI/CD
.github/
.gitlab-ci.yml

# Documentation
*.md
!README.md
docs/

# OS files
.DS_Store
Thumbs.db

# Test files
tests/

# Logs
*.log
logs/

# Docker
docker-compose*.yml

# Backup files
*.bak
```

**Why exclude these files?**

1. **target/**: Build artifacts (will be built inside container)
2. **IDE files**: Not needed in container
3. **.env**: Contains secrets (use environment variables instead)
4. **.git/**: Git history not needed (large directory)
5. **tests/**: Not run in production container
6. **Documentation**: Not needed at runtime

**Performance impact:**
- Without .dockerignore: 500MB+ build context
- With .dockerignore: ~50MB build context
- **10x faster** builds when files change

### Step 3: Understanding Docker Compose Configuration

Docker Compose orchestrates multiple containers and their dependencies.

**File: `docker-compose.yml`**

```yaml
services:
  postgres:
    image: postgres:16
    container_name: memos-postgres
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: memos_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  app:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: memos-app
    environment:
      DATABASE_URL: postgresql://postgres:postgres@postgres:5432/memos_db
      SERVER_HOST: 0.0.0.0
      SERVER_PORT: 3737
      RUST_LOG: info,actix_web=debug,actix_web_template=debug
      LOG_FORMAT: pretty
      APP_ENV: development
      CORS_ALLOWED_ORIGINS: "*"
      MAX_REQUEST_SIZE: 262144
      ENABLE_SWAGGER: "true"
      DATABASE_MAX_CONNECTIONS: 10
      DATABASE_CONNECT_TIMEOUT: 30
    ports:
      - "3737:3737"
    depends_on:
      postgres:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3737/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s

volumes:
  postgres_data:
```

**Note**: The full `docker-compose.yml` includes additional services (Jaeger, Prometheus, Grafana, Loki) for observability, which will be covered in Chapter 17. For now, focus on the `postgres` and `app` services.

#### PostgreSQL Service

```yaml
postgres:
  image: postgres:16
```
- Uses official PostgreSQL 16 image
- No build needed, pulled from Docker Hub

```yaml
environment:
  POSTGRES_USER: postgres
  POSTGRES_PASSWORD: postgres
  POSTGRES_DB: memos_db
```
- Initialize database with these credentials
- **Production warning**: Use secure passwords and secrets management

```yaml
volumes:
  - postgres_data:/var/lib/postgresql/data
```
- **Named volume**: Data persists across container restarts
- Without volume: Data is lost when container stops
- Volume stored in Docker's data directory

```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U postgres"]
```
- `pg_isready`: PostgreSQL utility that checks if server is ready
- App won't start until database is healthy

#### Application Service

```yaml
app:
  build:
    context: .
    dockerfile: Dockerfile
```
- Build from local Dockerfile
- Context is current directory (for COPY commands)

```yaml
environment:
  DATABASE_URL: postgresql://postgres:postgres@postgres:5432/memos_db
```
- **Important**: Hostname is `postgres` (Docker's DNS)
- Not `localhost` - containers have separate network namespaces
- Docker automatically resolves service names to container IPs

```yaml
depends_on:
  postgres:
    condition: service_healthy
```
- **Smart dependency**: Wait until postgres is healthy
- Not just "started" - actually ready to accept connections
- Prevents "connection refused" errors on startup

```yaml
ports:
  - "3737:3737"
```
- Map container port 3737 to host port 3737
- Format: `"HOST:CONTAINER"`
- Access app at `http://localhost:3737` on host machine

#### Volumes

```yaml
volumes:
  postgres_data:
```
- Declare named volumes at top level
- Managed by Docker (survives container deletion)
- Inspect with `docker volume ls`

### Step 4: Build and Run with Docker Compose

Now let's build and run the application.

**Build the images:**

```bash
# Build all services (this will take 5-10 minutes on first build)
docker-compose build

# Watch the build process
# You'll see:
# 1. PostgreSQL image being pulled
# 2. Application being built in builder stage
# 3. Runtime image being created
```

**Start all services:**

```bash
# Start in foreground (see logs)
docker-compose up

# Or start in background
docker-compose up -d

# You'll see:
# - Creating network
# - Creating volumes
# - Starting postgres container
# - Waiting for postgres to be healthy
# - Running migrations
# - Starting application
```

**Expected output:**

```
[+] Running 3/3
 ✔ Network actix-web-template_default     Created
 ✔ Container memos-postgres               Started
 ✔ Container memos-app                    Started

memos-postgres | PostgreSQL init process complete; ready for start up.
memos-postgres | database system is ready to accept connections
memos-app      | Running database migrations...
memos-app      | Applying migration: m20250109_000001_create_memos_table
memos-app      | Migration applied successfully
memos-app      | Starting application...
memos-app      | Server running on http://0.0.0.0:3737
```

**Verify it's working:**

```bash
# Check container status
docker-compose ps

# Expected output:
# NAME            IMAGE                      STATUS         PORTS
# memos-app       actix-web-template:latest  Up (healthy)   0.0.0.0:3737->3737/tcp
# memos-postgres  postgres:16                Up (healthy)   0.0.0.0:5432->5432/tcp

# Test health endpoint
curl http://localhost:3737/health

# Expected response:
# {
#   "status": "healthy",
#   "database": "connected",
#   "timestamp": "2025-01-09T12:00:00Z"
# }

# Access Swagger UI
open http://localhost:3737/swagger-ui/
# Or visit in browser
```

### Step 5: Docker Compose Commands

Learn essential Docker Compose commands for managing your deployment.

**Starting and stopping:**

```bash
# Start all services
docker-compose up

# Start in background (detached mode)
docker-compose up -d

# Stop all services (preserves containers)
docker-compose stop

# Stop and remove containers (keeps volumes)
docker-compose down

# Stop and remove containers AND volumes (destroys data)
docker-compose down -v
```

**Viewing logs:**

```bash
# View logs from all services
docker-compose logs

# Follow logs in real-time
docker-compose logs -f

# Logs from specific service
docker-compose logs app
docker-compose logs postgres

# Last 100 lines
docker-compose logs --tail=100

# Logs with timestamps
docker-compose logs -t
```

**Rebuilding:**

```bash
# Rebuild all images
docker-compose build

# Rebuild specific service
docker-compose build app

# Rebuild without cache (clean build)
docker-compose build --no-cache

# Rebuild and restart
docker-compose up --build
```

**Inspecting:**

```bash
# List running services
docker-compose ps

# List all containers (including stopped)
docker-compose ps -a

# View service configuration
docker-compose config

# Execute command in running container
docker-compose exec app sh

# View container resource usage
docker stats memos-app memos-postgres
```

**Scaling (not applicable for our app, but useful to know):**

```bash
# Run multiple instances of a service
docker-compose up -d --scale app=3

# Note: Our app uses specific port binding (3737:3737)
# Scaling requires removing port mapping or using load balancer
```

### Step 6: Database Management in Docker

Managing the PostgreSQL database in containers.

**Access PostgreSQL CLI:**

```bash
# Connect to postgres container
docker-compose exec postgres psql -U postgres -d memos_db

# Now you're in psql:
memos_db=# \dt
# List tables: memos, seaorm_migration

memos_db=# SELECT COUNT(*) FROM memos;
# Query data

memos_db=# \q
# Exit psql
```

**Backup database:**

```bash
# Dump database to file
docker-compose exec -T postgres pg_dump -U postgres memos_db > backup.sql

# With timestamp
docker-compose exec -T postgres pg_dump -U postgres memos_db > backup-$(date +%Y%m%d-%H%M%S).sql
```

**Restore database:**

```bash
# Restore from backup
docker-compose exec -T postgres psql -U postgres -d memos_db < backup.sql
```

**Reset database:**

```bash
# Stop application
docker-compose stop app

# Drop and recreate database
docker-compose exec postgres psql -U postgres -c "DROP DATABASE IF EXISTS memos_db;"
docker-compose exec postgres psql -U postgres -c "CREATE DATABASE memos_db;"

# Restart application (migrations will run automatically)
docker-compose start app
```

**View database logs:**

```bash
# PostgreSQL logs
docker-compose logs postgres

# Follow database logs
docker-compose logs -f postgres
```

### Step 7: Troubleshooting Container Issues

Common problems and solutions.

**Container won't start:**

```bash
# Check logs
docker-compose logs app

# Common issues:
# 1. Port already in use
#    Solution: Stop other services or change port in docker-compose.yml

# 2. Database connection failed
#    Check postgres is healthy: docker-compose ps
#    Check logs: docker-compose logs postgres

# 3. Migration failed
#    Check migration logs in app output
#    Manually fix database, restart: docker-compose restart app
```

**Image build fails:**

```bash
# Check Dockerfile syntax
docker-compose config

# Build with verbose output
docker-compose build --progress=plain

# Clear build cache and rebuild
docker-compose build --no-cache
```

**Permission errors:**

```bash
# If you see permission denied errors
# Make sure Docker has access to files

# On Linux, check ownership
ls -la

# Fix ownership if needed
sudo chown -R $USER:$USER .
```

**Clean up everything:**

```bash
# Stop and remove all containers, networks, volumes
docker-compose down -v

# Remove all unused Docker resources
docker system prune -a

# Warning: This removes ALL unused containers, images, volumes, networks
# Not just for this project!
```

### Step 8: Production Considerations

Preparing your Docker setup for production.

#### 1. Security Enhancements

**Use secrets for sensitive data:**

```yaml
# docker-compose.prod.yml
services:
  postgres:
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    secrets:
      - db_password

secrets:
  db_password:
    file: ./secrets/db_password.txt
```

**Run as non-root user:**

Add to Dockerfile after runtime stage:

```dockerfile
# Create non-root user
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app

USER appuser
```

**Scan images for vulnerabilities:**

```bash
# Using Docker Scout
docker scout cves actix-web-template:latest

# Using Trivy
trivy image actix-web-template:latest
```

#### 2. Image Optimization

**Use specific Rust version:**

```dockerfile
# Instead of: FROM rust:latest
FROM rust:1.75-slim as builder
```

**Multi-arch builds:**

```bash
# Build for multiple architectures
docker buildx build --platform linux/amd64,linux/arm64 -t actix-web-template:latest .
```

**Alpine Linux (smaller but requires musl):**

```dockerfile
# Runtime stage with Alpine
FROM alpine:3.19
RUN apk add --no-cache libgcc libpq curl
# Size: ~50MB vs ~100MB with Debian
```

#### 3. Logging

**Configure JSON logging for production:**

```yaml
# docker-compose.prod.yml
services:
  app:
    environment:
      LOG_FORMAT: json
      RUST_LOG: warn,actix_web_template=info
```

**Send logs to external system:**

```bash
# Forward logs to Loki, Elasticsearch, etc.
# Covered in Chapter 17: Observability
```

#### 4. Resource Limits

**Add memory and CPU limits:**

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M
```

#### 5. Environment-Specific Configs

**Create separate compose files:**

```bash
# Development
docker-compose.yml

# Production
docker-compose.prod.yml
```

**Use override files:**

```bash
# Use both files
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up
```

## Checkpoint

Verify your Docker deployment is working correctly.

**Run these commands:**

```bash
# 1. Build and start services
docker-compose up -d --build

# 2. Wait for health checks (30 seconds)
sleep 30

# 3. Check status (both should be healthy)
docker-compose ps

# 4. Test health endpoint
curl http://localhost:3737/health

# 5. Create a memo via API
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Docker Test",
    "description": "Testing Docker deployment",
    "date_to": "2025-12-31T12:00:00Z"
  }'

# 6. View in browser
open http://localhost:3737/

# 7. Check Swagger UI
open http://localhost:3737/swagger-ui/

# 8. View logs
docker-compose logs --tail=50
```

**Expected results:**

```
# docker-compose ps output:
NAME            IMAGE                      STATUS         PORTS
memos-app       actix-web-template:latest  Up (healthy)   0.0.0.0:3737->3737/tcp
memos-postgres  postgres:16                Up (healthy)   0.0.0.0:5432->5432/tcp

# Health check response:
{
  "status": "healthy",
  "database": "connected",
  "timestamp": "2025-01-09T12:00:00Z"
}

# Create memo response:
{
  "id": "...",
  "title": "Docker Test",
  "description": "Testing Docker deployment",
  "date_to": "2025-12-31T12:00:00Z",
  "completed": false,
  "created_at": "...",
  "updated_at": "..."
}
```

**Troubleshooting if checks fail:**

```bash
# If services not healthy:
docker-compose logs app
docker-compose logs postgres

# If port already in use:
# Stop other services or change port in docker-compose.yml

# If migration fails:
docker-compose logs app | grep -i migration

# Rebuild from scratch:
docker-compose down -v
docker-compose up --build
```

## Common Issues and Solutions

### Issue: Build is very slow

**Symptoms**: Docker build takes 10+ minutes every time

**Causes**:
- Not using build cache efficiently
- Copying unnecessary files (missing .dockerignore)
- Rebuilding dependencies every time

**Solutions**:

```bash
# 1. Ensure .dockerignore is present and correct
cat .dockerignore

# 2. Use BuildKit for better caching
export DOCKER_BUILDKIT=1
docker-compose build

# 3. Check build context size
docker-compose build --progress=plain 2>&1 | grep "transferring context"
# Should be < 100MB

# 4. Use cargo-chef for better dependency caching (advanced)
# See: https://github.com/LukeMathWalker/cargo-chef
```

### Issue: Container exits immediately

**Symptoms**: `docker-compose ps` shows container as "Exited (1)"

**Causes**:
- Application crashes on startup
- Environment variables missing or incorrect
- Database not accessible

**Solutions**:

```bash
# 1. Check logs for error message
docker-compose logs app

# 2. Check environment variables
docker-compose exec app env | grep -i database

# 3. Test database connection manually
docker-compose exec app sh
# Inside container:
curl postgres:5432  # Should see "connection received"

# 4. Run migrations manually if they failed
docker-compose exec app ./migration

# 5. Check if binary is executable
docker-compose exec app ls -la
```

### Issue: Changes not reflected in container

**Symptoms**: Code changes don't appear in running container

**Cause**: Container is using old image

**Solution**:

```bash
# Rebuild and restart
docker-compose up --build

# Force rebuild without cache
docker-compose build --no-cache
docker-compose up -d
```

### Issue: Database data lost after restart

**Symptoms**: All memos disappear when stopping containers

**Cause**: Not using volumes or volumes were deleted

**Solutions**:

```bash
# 1. Check volumes exist
docker volume ls | grep postgres_data

# 2. Don't use `down -v` (it deletes volumes)
docker-compose stop   # Instead of: docker-compose down -v

# 3. Backup important data
docker-compose exec -T postgres pg_dump -U postgres memos_db > backup.sql

# 4. Verify volume is mounted
docker-compose exec postgres df -h | grep /var/lib/postgresql
```

### Issue: Port 3737 already in use

**Symptoms**: "Error starting userland proxy: bind: address already in use"

**Solutions**:

```bash
# 1. Find what's using the port
lsof -ti:3737

# 2. Kill the process
lsof -ti:3737 | xargs kill -9

# 3. Or change port in docker-compose.yml
ports:
  - "3738:3737"  # Use different host port
```

## Code Review

### Key Design Principles Demonstrated
- **Multi-stage builds** keep compilation tooling out of the runtime image, shrinking attack surface while preserving reproducible builds.
- **Container-first configuration** (env files, health checks, volumes) mirrors the twelve-factor principles established earlier, so deploying to Docker requires no code changes.
- **Fail-fast orchestration**: Compose depends_on + health check wiring ensure Postgres is healthy before the app boots, preventing race conditions or manual sequencing.

### Architecture Benefits
- **Portable deployments**: The exact same `docker-compose.yml` runs locally, in CI, or on a VM, eliminating “works on my machine” drift.
- **Operational visibility**: Health checks, structured logs, and named volumes/logical service boundaries make it trivial to debug or replace individual services.
- **Secure supply chain**: Pinning base images and copying only the compiled binaries reduces exposure to outdated build tooling and secret leakage.

### Complete Deployment Structure
```
docker-compose.yml
├── services
│   ├── app
│   │   ├── image: actix-web-template (multi-stage build output)
│   │   ├── ports: 3737:3737
│   │   ├── depends_on: postgres (condition: service_healthy)
│   │   └── volumes: ./static, ./templates (read-only)
│   └── postgres
│       ├── image: postgres:16
│       ├── volumes: postgres_data:/var/lib/postgresql/data
│       └── healthcheck: pg_isready
└── volumes
    └── postgres_data
```

## Testing

Write tests to verify Docker deployment works correctly.

**Create: `tests/docker_tests.sh`**

```bash
#!/bin/bash
set -e

echo "Starting Docker deployment tests..."

# Start services
echo "1. Starting services..."
docker-compose up -d
sleep 30  # Wait for health checks

# Test health endpoint
echo "2. Testing health endpoint..."
HEALTH=$(curl -s http://localhost:3737/health)
echo "$HEALTH" | grep -q "healthy" || (echo "Health check failed" && exit 1)

# Create a memo
echo "3. Creating test memo..."
MEMO=$(curl -s -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{"title":"Docker Test","description":"Test","date_to":"2025-12-31T00:00:00Z"}')
echo "$MEMO" | grep -q "Docker Test" || (echo "Create memo failed" && exit 1)

# List memos
echo "4. Listing memos..."
MEMOS=$(curl -s "http://localhost:3737/api/v1/memos?limit=10")
echo "$MEMOS" | grep -q "Docker Test" || (echo "List memos failed" && exit 1)

# Test Swagger UI
echo "5. Testing Swagger UI..."
SWAGGER=$(curl -s http://localhost:3737/swagger-ui/)
echo "$SWAGGER" | grep -q "swagger" || (echo "Swagger UI failed" && exit 1)

# Check logs for errors
echo "6. Checking logs for errors..."
docker-compose logs | grep -i "error" && (echo "Errors found in logs" && exit 1) || true

echo "All tests passed!"
```

**Run the tests:**

```bash
# Make executable
chmod +x tests/docker_tests.sh

# Run tests
./tests/docker_tests.sh

# Expected output:
# Starting Docker deployment tests...
# 1. Starting services...
# 2. Testing health endpoint...
# 3. Creating test memo...
# 4. Listing memos...
# 5. Testing Swagger UI...
# 6. Checking logs for errors...
# All tests passed!
```

## Performance Comparison

Compare Docker vs native performance.

**Benchmark results (example):**

| Metric | Native | Docker | Overhead |
|--------|--------|--------|----------|
| Startup time | 1.2s | 1.5s | +25% |
| Request latency (p50) | 5ms | 6ms | +20% |
| Request latency (p99) | 25ms | 30ms | +20% |
| Memory usage | 50MB | 60MB | +20% |
| Requests/sec | 10,000 | 9,500 | -5% |

**Key insights:**
- Docker overhead is minimal (< 20%)
- Mostly due to network virtualization
- Negligible for most applications
- Benefits (consistency, portability) outweigh overhead

## Summary

In this chapter, you learned:

### Docker Concepts

- **Multi-stage builds**: Separate build and runtime environments for smaller images
- **Layer caching**: Speed up builds by reusing unchanged layers
- **.dockerignore**: Reduce build context size and improve build speed
- **Health checks**: Ensure containers are actually ready, not just started

### Docker Compose

- **Service orchestration**: Define and run multi-container applications
- **Dependency management**: Use health checks for proper startup order
- **Volume management**: Persist data across container restarts
- **Network isolation**: Automatic DNS resolution between services

### Production Best Practices

- **Security**: Use secrets, non-root users, scan for vulnerabilities
- **Optimization**: Specific versions, multi-arch builds, resource limits
- **Monitoring**: Health checks, structured logging, resource usage
- **Backup**: Regular database backups, volume management

### Key Takeaways

1. **Docker simplifies deployment**: One command to run entire stack
2. **Multi-stage builds are essential**: Reduce image size by 90%+
3. **Health checks prevent issues**: Wait for services to be truly ready
4. **Volumes are critical**: Don't lose data on container restart
5. **Production requires extra care**: Security, monitoring, backups

### Architecture Integration

```
┌─────────────────────────────────────────┐
│         Docker Host                      │
│                                          │
│  ┌──────────────────────────────────┐   │
│  │  memos-app Container             │   │
│  │  ┌──────────────────────────┐    │   │
│  │  │  Actix Web Application   │    │   │
│  │  │  Port: 3737              │    │   │
│  │  └──────────────────────────┘    │   │
│  │  Health: /health                 │   │
│  └──────────────────────────────────┘   │
│             │                             │
│             ↓ (Docker network DNS)       │
│  ┌──────────────────────────────────┐   │
│  │  memos-postgres Container        │   │
│  │  ┌──────────────────────────┐    │   │
│  │  │  PostgreSQL 16           │    │   │
│  │  │  Port: 5432              │    │   │
│  │  └──────────────────────────┘    │   │
│  │  Volume: postgres_data           │   │
│  └──────────────────────────────────┘   │
│                                          │
└─────────────────────────────────────────┘
          ↕ Port mapping
┌─────────────────────────────────────────┐
│         Host Machine                     │
│  Browser → localhost:3737                │
└─────────────────────────────────────────┘
```

## Next Steps

In the next chapter, you'll:

- **Chapter 16: CI/CD Pipeline**: Automate testing and deployment
  - GitHub Actions workflows
  - Automated testing on every push
  - Docker image building and publishing
  - Deployment automation
  - Branch protection rules

### Optional Exercises

1. **Add Redis container**: For caching or session storage
2. **Implement Blue-Green deployment**: Zero-downtime deployments
3. **Add Nginx reverse proxy**: Load balancing and SSL termination
4. **Optimize build time**: Use cargo-chef for better caching
5. **Multi-platform builds**: Support ARM64 and AMD64 architectures

### Additional Resources

- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Rust Docker Guide](https://docs.docker.com/language/rust/)
- [Multi-stage Builds](https://docs.docker.com/build/building/multi-stage/)
- [Docker Security](https://docs.docker.com/engine/security/)
- [cargo-chef for faster builds](https://github.com/LukeMathWalker/cargo-chef)

---

**Congratulations!** Your application is now containerized and ready for deployment. The Docker setup provides consistency across environments and simplifies the deployment process to a single command: `docker-compose up`.
