# Troubleshooting Guide

This guide covers common issues and their solutions when working with the Actix Web Memos application.

## Table of Contents

- [Server Issues](#server-issues)
- [Database Issues](#database-issues)
- [Docker Issues](#docker-issues)
- [Build Issues](#build-issues)
- [Runtime Errors](#runtime-errors)
- [Performance Issues](#performance-issues)
- [Testing Issues](#testing-issues)

## Server Issues

### Port Already in Use

**Problem**: Server fails to start with error "Address already in use"

```
Error: Address already in use (os error 48)
```

**Solution 1**: Kill the process using port 3737

```bash
# On macOS/Linux
lsof -ti:3737 | xargs kill -9

# Or find the process ID
lsof -i:3737
# Then kill it
kill -9 <PID>
```

**Solution 2**: Change the port in `.env`

```bash
SERVER_PORT=3738
```

### Server Starts But Not Accessible

**Problem**: Server starts but cannot access http://localhost:3737/

**Solution 1**: Check if server is binding to correct address

```bash
# In .env, ensure:
SERVER_HOST=127.0.0.1
# Or bind to all interfaces:
SERVER_HOST=0.0.0.0
```

**Solution 2**: Check firewall settings

```bash
# On macOS
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /path/to/actix-web-template
```

### Graceful Shutdown Not Working

**Problem**: Ctrl+C doesn't stop the server cleanly

**Solution**: Use proper signal handling

```bash
# Send SIGTERM instead of SIGKILL
kill <PID>

# Or use Docker stop (not kill)
docker-compose stop
```

## Database Issues

### Cannot Connect to Database

**Problem**: Application fails with database connection error

```
Error: Failed to connect to database
```

**Solution 1**: Verify PostgreSQL is running

```bash
# Check if PostgreSQL container is running
docker-compose ps postgres

# Check PostgreSQL status (local install)
pg_ctl status
```

**Solution 2**: Verify DATABASE_URL is correct

```bash
# In .env, check format:
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db

# Format: postgresql://username:password@host:port/database
```

**Solution 3**: Test connection manually

```bash
psql "postgresql://postgres:postgres@localhost:5432/memos_db"
```

### Migration Errors

**Problem**: Migrations fail to run

```
Error: Migration failed: relation "memos" already exists
```

**Solution 1**: Check migration status

```bash
cd migration
sea-orm-cli migrate status
```

**Solution 2**: Reset migrations (WARNING: destroys data)

```bash
# Rollback all migrations
sea-orm-cli migrate down

# Re-run migrations
sea-orm-cli migrate up
```

**Solution 3**: Fresh database

```bash
# Drop and recreate database
docker-compose down -v  # Removes volumes
docker-compose up -d postgres

# Run migrations
cd migration
cargo run
```

### Database Connection Pool Exhausted

**Problem**: "Connection pool timeout" errors under load

```
Error: Timeout acquiring connection from pool
```

**Solution 1**: Increase pool size in `.env`

```bash
DATABASE_MAX_CONNECTIONS=20
```

**Solution 2**: Increase connect timeout

```bash
DATABASE_CONNECT_TIMEOUT=60
```

**Solution 3**: Check for connection leaks

```bash
# Enable connection pool logging
RUST_LOG=sqlx::pool=debug,info
```

### Query Timeout

**Problem**: Slow queries timing out

**Solution 1**: Add indexes to frequently queried columns

```sql
-- Check missing indexes
EXPLAIN ANALYZE SELECT * FROM memos WHERE completed = false;
```

**Solution 2**: Optimize queries in repository layer

See `src/repository/memo_repository.rs` for query optimization opportunities.

## Docker Issues

### Docker Build Fails

**Problem**: `docker build` or `docker-compose up --build` fails

**Solution 1**: Check Dockerfile syntax

```bash
# Validate Dockerfile
docker build --no-cache -t actix-web-template .
```

**Solution 2**: Clear Docker cache

```bash
docker system prune -a
docker volume prune
```

**Solution 3**: Check disk space

```bash
df -h
docker system df
```

### Container Exits Immediately

**Problem**: Container starts then exits with code 1

**Solution 1**: Check container logs

```bash
docker-compose logs app
docker logs <container_id>
```

**Solution 2**: Run container in foreground

```bash
docker-compose up app  # Without -d
```

**Solution 3**: Check environment variables

```bash
docker-compose config
```

### Cannot Connect to Database from Container

**Problem**: App container cannot reach PostgreSQL container

**Solution 1**: Verify network configuration

```bash
# Check if containers are on same network
docker network ls
docker network inspect actix-web-template_default
```

**Solution 2**: Use service name as hostname

```bash
# In docker-compose.yml, DATABASE_URL should use service name:
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/memos_db
#                                          ^^^^^^^^ service name, not localhost
```

**Solution 3**: Check depends_on configuration

```yaml
# In docker-compose.yml
services:
  app:
    depends_on:
      postgres:
        condition: service_healthy
```

### Health Check Failing

**Problem**: Docker health check reports unhealthy

**Solution 1**: Test health endpoint manually

```bash
docker exec <container_id> curl -f http://localhost:3737/health
```

**Solution 2**: Check health check configuration

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:3737/health || exit 1
```

## Build Issues

### Compilation Errors

**Problem**: `cargo build` fails with compilation errors

**Solution 1**: Update dependencies

```bash
cargo update
cargo clean
cargo build
```

**Solution 2**: Check Rust version

```bash
rustc --version  # Should be 1.75+
rustup update stable
```

**Solution 3**: Clear cargo cache

```bash
rm -rf target/
rm Cargo.lock
cargo build
```

### Linker Errors

**Problem**: Linking fails on macOS or Linux

**Solution 1**: Install system dependencies

```bash
# macOS
brew install postgresql

# Ubuntu/Debian
sudo apt-get install libpq-dev pkg-config

# Fedora/RHEL
sudo dnf install postgresql-devel
```

### Template Compilation Errors

**Problem**: Askama template compilation fails

```
Error: Failed to compile template
```

**Solution 1**: Check template syntax

- Verify all `{% %}` tags are properly closed
- Check that referenced variables exist in template struct
- Ensure template path matches file system

**Solution 2**: Clean and rebuild

```bash
cargo clean
cargo build
```

## Runtime Errors

### 404 Not Found

**Problem**: API endpoints return 404

**Solution 1**: Verify route configuration

Check `src/main.rs` for route registration:

```rust
.service(
    web::scope("/api/v1")
        .service(list_memos)
        .service(get_memo)
        // ...
)
```

**Solution 2**: Check HTTP method

```bash
# Ensure using correct method
curl -X GET http://localhost:3737/api/v1/memos  # GET, not POST
```

### 500 Internal Server Error

**Problem**: Requests fail with 500 status code

**Solution 1**: Check application logs

```bash
# Run with debug logging
RUST_LOG=debug cargo run
```

**Solution 2**: Check error handling

See `src/error/app_error.rs` for error type definitions and responses.

### Rate Limiting Errors

**Problem**: Getting 429 Too Many Requests

```
Error: Too Many Requests
```

**Solution 1**: Adjust rate limits

See `src/middleware/rate_limit.rs`:

```rust
// Increase limit
let governor_conf = GovernorConfigBuilder::default()
    .per_second(60)  // Increase from default
    .burst_size(200)  // Increase burst
```

**Solution 2**: Disable rate limiting for testing

Comment out rate limiting middleware in `src/main.rs`.

### CORS Errors

**Problem**: Browser shows CORS errors in console

```
Access to fetch at 'http://localhost:3737/api/v1/memos' from origin 'http://localhost:3000'
has been blocked by CORS policy
```

**Solution**: Configure CORS in `.env`

```bash
# Allow specific origins
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:8080

# Or allow all (development only)
CORS_ALLOWED_ORIGINS=*
```

### Input Validation Errors

**Problem**: Valid data rejected with validation errors

**Solution**: Check DTO validation rules

See `src/dto/memo_dto.rs` for validation constraints:

```rust
#[validate(length(min = 1, max = 200))]
pub title: String,
```

## Performance Issues

### Slow Response Times

**Problem**: API responses are slow

**Solution 1**: Enable response compression

Compression is enabled by default. Verify in `Cargo.toml`:

```toml
actix-web = { version = "4", features = ["compress-gzip", "compress-brotli"] }
```

**Solution 2**: Check database query performance

```bash
# Enable query logging
RUST_LOG=sqlx::query=debug cargo run
```

**Solution 3**: Add database indexes

```sql
CREATE INDEX idx_memos_completed ON memos(completed);
CREATE INDEX idx_memos_date_to ON memos(date_to);
```

### High Memory Usage

**Problem**: Application uses too much memory

**Solution 1**: Tune connection pool

```bash
# In .env
DATABASE_MAX_CONNECTIONS=5  # Reduce from default 10
```

**Solution 2**: Check for memory leaks

```bash
# Run with memory profiler
cargo install cargo-instruments
cargo instruments --release --bench my_bench
```

### High CPU Usage

**Problem**: CPU usage is constantly high

**Solution 1**: Check for inefficient loops or queries

Review code for N+1 query problems or unnecessary iterations.

**Solution 2**: Profile the application

```bash
# Install flamegraph
cargo install flamegraph

# Generate profile
cargo flamegraph
```

## Testing Issues

### Tests Fail to Connect to Database

**Problem**: Integration tests fail with database errors

**Solution 1**: Create test database

```bash
# Create separate test database
createdb memos_test

# Set TEST_DATABASE_URL
export TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_test
```

**Solution 2**: Run PostgreSQL in test mode

```bash
# Start PostgreSQL before running tests
docker-compose up -d postgres
cargo test
```

### Tests Timeout

**Problem**: Tests hang or timeout

**Solution 1**: Increase test timeout

```bash
cargo test -- --test-threads=1 --nocapture
```

**Solution 2**: Check for deadlocks

Review test code for potential database deadlocks or resource contention.

### Flaky Tests

**Problem**: Tests pass sometimes, fail other times

**Solution 1**: Isolate test database state

Ensure each test uses transactions or cleans up after itself.

**Solution 2**: Run tests serially

```bash
cargo test -- --test-threads=1
```

## Environment-Specific Issues

### Development vs Production Differences

**Problem**: Works in development but fails in production

**Solution 1**: Check environment configuration

```bash
# Ensure APP_ENV is set correctly
APP_ENV=production cargo run --release
```

**Solution 2**: Test production build locally

```bash
cargo build --release
./target/release/actix-web-template
```

### Docker vs Local Differences

**Problem**: Behavior differs between Docker and local runs

**Solution**: Ensure consistent environment variables

```bash
# Copy .env settings to docker-compose.yml environment section
docker-compose config  # Verify configuration
```

## Getting More Help

If you're still experiencing issues:

1. **Check logs**: Run with `RUST_LOG=debug` for detailed logging
2. **Search issues**: Check GitHub issues for similar problems
3. **Create issue**: Open a new issue with:
   - Rust version (`rustc --version`)
   - OS and version
   - Complete error message
   - Steps to reproduce
   - Configuration (sanitized, no passwords)

## Useful Commands

### Diagnostic Commands

```bash
# Check Rust installation
rustc --version
cargo --version

# Check PostgreSQL version
psql --version

# Check Docker version
docker --version
docker-compose --version

# View all environment variables
env | grep -E '(DATABASE|SERVER|RUST|APP)'

# Check running processes
ps aux | grep actix-web-template
lsof -i :3737

# View Docker resources
docker ps -a
docker images
docker volume ls
docker network ls

# Check system resources
top
df -h
free -h  # Linux
```

### Reset Commands

```bash
# Complete reset (WARNING: destroys all data)
docker-compose down -v
rm -rf target/
rm Cargo.lock
cargo clean
docker-compose up -d postgres
cd migration && cargo run && cd ..
cargo build
cargo run
```

### Debug Commands

```bash
# Run with maximum logging
RUST_LOG=trace,actix_web=debug,sqlx=debug cargo run

# Run tests with output
cargo test -- --nocapture --test-threads=1

# Check for unused dependencies
cargo install cargo-udeps
cargo +nightly udeps

# Security audit
cargo install cargo-audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Common Error Messages

### "thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value'"

This indicates a panic from unwrapping a Result. Check the full error message for context.

**Solution**: Run with `RUST_BACKTRACE=1` for full stack trace:

```bash
RUST_BACKTRACE=1 cargo run
```

### "FATAL: database \"memos_db\" does not exist"

Database hasn't been created.

**Solution**:

```bash
createdb memos_db
# Or with Docker:
docker-compose exec postgres createdb -U postgres memos_db
```

### "permission denied while trying to connect to the Docker daemon socket"

User doesn't have Docker permissions.

**Solution**:

```bash
# Add user to docker group
sudo usermod -aG docker $USER
# Log out and back in
```

### "cannot find -lpq"

PostgreSQL client library not found.

**Solution**:

```bash
# Install PostgreSQL development files
# macOS:
brew install postgresql

# Ubuntu/Debian:
sudo apt-get install libpq-dev
```

---

**Remember**: Most issues can be resolved by checking logs with `RUST_LOG=debug` and verifying configuration in `.env` and `docker-compose.yml`.