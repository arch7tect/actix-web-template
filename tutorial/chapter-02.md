# Chapter 2: Database Integration with SeaORM

## Overview

In this chapter, we'll integrate PostgreSQL database into our Actix Web application using SeaORM, a modern async ORM for Rust. You'll learn how to manage database schema with migrations, generate entity models, configure connection pooling, and integrate the database connection into your application state.

By the end of this chapter, you'll have a fully functional database layer ready to support your application's data persistence needs.

> **Note on Tutorial Approach**: This chapter continues the incremental build approach. The production codebase in this repository already has all these features implemented with additional optimizations (connection pool tuning, observability integration, health checks with database status). We'll build the database layer step-by-step for learning, then note how the production code enhances these concepts.

## Prerequisites

### Completed

- Chapter 0: Prerequisites and Environment Setup
- Chapter 1: Core Application Setup

### Required Knowledge

- Basic SQL and relational database concepts
- Understanding of database migrations
- Familiarity with async/await in Rust
- Basic understanding of ORM (Object-Relational Mapping)

### Required Software

- PostgreSQL 16 running (Docker or native)
- SeaORM CLI installed (`sea-orm-cli`)
- Working Actix Web application from Chapter 1

## Learning Objectives

By completing this chapter, you will:

1. Understand SeaORM architecture and how it integrates with Actix Web
2. Create and manage database migrations
3. Generate entity models from database schema
4. Configure database connection pooling
5. Integrate DatabaseConnection into application state
6. Verify database connectivity from your application
7. Implement a health check that includes database status
8. Handle database errors appropriately

## Concepts Covered

### SeaORM Architecture

SeaORM is an async ORM built for Rust with the following key components:

**DatabaseConnection**: Connection pool manager that handles multiple database connections efficiently. Thread-safe and can be shared across async tasks.

**Entity**: Rust structs that represent database tables. Generated from schema or defined manually. Support relationships, custom methods, and type-safe queries.

**ActiveModel**: Mutable version of entities used for inserts and updates. Tracks which fields have been modified.

**Migration**: Version-controlled schema changes. Each migration has an `up()` (apply) and `down()` (rollback) method.

**Query Builder**: Type-safe, composable query construction using Rust's type system to prevent SQL injection and runtime errors.

### Database Migrations

Migrations are versioned schema changes that allow:
- **Version Control**: Track database schema in git
- **Team Collaboration**: Share schema changes across team members
- **Rollback**: Undo changes if needed
- **Reproducibility**: Same schema across dev, staging, production

### Connection Pooling

Connection pools maintain multiple database connections to:
- **Reduce Latency**: Reuse existing connections instead of creating new ones
- **Handle Concurrency**: Multiple requests can access the database simultaneously
- **Resource Management**: Limit total connections to avoid overwhelming the database
- **Auto-Recovery**: Reconnect if connections are lost

## PostgreSQL Setup with Docker Compose

Before we begin integrating the database, let's ensure PostgreSQL is properly set up. While Chapter 0 covered basic Docker setup, here we'll create a production-ready Docker Compose configuration.

### Why Docker Compose?

Docker Compose provides:
- **Reproducible environments** - Same setup across all developers
- **Easy management** - Start/stop services with simple commands
- **Volume persistence** - Data survives container restarts
- **Health checks** - Ensures database is ready before app starts
- **Network isolation** - Services communicate on private network

### Create docker-compose.yml

Create a `docker-compose.yml` file in your project root:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    container_name: actix-postgres
    restart: unless-stopped
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: memos_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
    driver: local
```

**Note**: Modern Docker Compose (v2+) doesn't require the `version` field - it's automatically inferred. We're using `docker compose` (without dash) which is the newer plugin-based CLI.

**Configuration explained**:
- `postgres:16-alpine` - Lightweight PostgreSQL 16 image
- `container_name` - Easy to reference (e.g., `docker logs actix-postgres`)
- `restart: unless-stopped` - Auto-restart on crash (except manual stop)
- `environment` - Database credentials (override with .env for production)
- `ports` - Expose 5432 to host machine
- `volumes` - Persist data and run initialization scripts
- `healthcheck` - Verify database is ready to accept connections

### Optional: Create Initialization Scripts

Create `init-scripts/` directory for scripts that run on first startup:

```bash
mkdir -p init-scripts
```

Create `init-scripts/01-create-extensions.sql`:

```sql
-- Enable UUID generation extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable full-text search (useful for search features)
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
```

These extensions provide:
- `uuid-ossp`: Generate UUIDs directly in PostgreSQL (alternative to Rust-side generation)
- `pg_trgm`: Trigram-based text search for fuzzy matching

### Start PostgreSQL

```bash
# Start PostgreSQL in detached mode (background)
docker compose up -d postgres

# View logs to verify startup
docker compose logs -f postgres
```

**Expected output**:
```
[+] Running 2/2
 ✔ Network actix-web-template_default    Created
 ✔ Container actix-postgres              Started
```

### Verify PostgreSQL is Running

```bash
# Check container status
docker compose ps

# Expected output:
# NAME              IMAGE                 STATUS          PORTS
# actix-postgres    postgres:16-alpine    Up 10 seconds   0.0.0.0:5432->5432/tcp

# Check health status
docker inspect actix-postgres --format='{{.State.Health.Status}}'
# Expected output: healthy
```

### Test Database Connection

```bash
# Connect to database
docker compose exec postgres psql -U postgres -d memos_db

# You should see:
# psql (16.x)
# Type "help" for help.
# 
# memos_db=#

# Test query
# Type in psql: SELECT version();
# Then: \q to exit
```

### Common Docker Compose Commands

```bash
# Start services
docker compose up -d

# Stop services (keeps data)
docker compose stop

# Stop and remove containers (keeps data volumes)
docker compose down

# Stop and REMOVE EVERYTHING including data
docker compose down -v

# View logs
docker compose logs -f postgres

# View last 100 lines
docker compose logs --tail=100 postgres

# Restart PostgreSQL
docker compose restart postgres

# Execute SQL command
docker compose exec postgres psql -U postgres -d memos_db -c "SELECT version();"

# Backup database
docker compose exec -T postgres pg_dump -U postgres memos_db > backup.sql

# Restore from backup
docker compose exec -T postgres psql -U postgres -d memos_db < backup.sql
```

### Troubleshooting Docker Setup

**Issue: Port 5432 already in use**

```bash
# Check what's using port 5432
lsof -i :5432
# OR
netstat -an | grep 5432

# Option 1: Stop conflicting service
# On macOS with Homebrew PostgreSQL
brew services stop postgresql@16

# On Linux with systemd
sudo systemctl stop postgresql

# Option 2: Use different port in docker-compose.yml
# Change ports to:
ports:
  - "5433:5432"

# Then update DATABASE_URL in .env:
DATABASE_URL=postgresql://postgres:postgres@localhost:5433/memos_db
```

**Issue: Container won't start**

```bash
# Check logs for errors
docker compose logs postgres

# Remove and recreate container
docker compose down
docker compose up -d postgres

# If data is corrupted, remove volume (DESTROYS DATA)
docker compose down -v
docker volume rm actix-web-template_postgres_data
docker compose up -d postgres
```

**Issue: Connection refused from application**

```bash
# Verify PostgreSQL is listening
docker compose exec postgres pg_isready -U postgres

# Check if port is exposed
docker compose port postgres 5432

# Verify .env DATABASE_URL matches docker compose config
cat .env | grep DATABASE_URL
```

### Update Your .env File

Ensure your `.env` file has the correct DATABASE_URL:

```bash
# Database Configuration
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2
DATABASE_CONNECT_TIMEOUT=30
DATABASE_IDLE_TIMEOUT=600
```

**Note**: When running the app in Docker alongside PostgreSQL (we'll do this later), change `localhost` to `postgres` (the service name):

```bash
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/memos_db
```

---

Now that PostgreSQL is running, let's integrate it with our application!

## Step-by-Step Instructions

### Step 1: Add Database Dependencies

**Why**: We need SeaORM and PostgreSQL driver crates.

**How**:

1. **Update `Cargo.toml`** to add database dependencies:

```toml
[package]
name = "actix-memo-app"
version = "0.2.1"
edition = "2024"

[dependencies]
# Web framework
actix-web = "4"

# Async runtime
tokio = { version = "1.47", features = ["full"] }

# Database - SeaORM
sea-orm = { version = "1.1", features = [
    "sqlx-postgres",
    "runtime-tokio-rustls",
    "macros",
] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
dotenvy = "0.15"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# UUID support
uuid = { version = "1.18", features = ["v4", "serde"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

2. **Build to download dependencies**:
   ```bash
   cargo build
   ```

**Verify**:
```bash
cargo build
```
Should complete without errors.

---

### Step 2: Set Up Migration Project

**Why**: SeaORM migrations are managed in a separate Cargo workspace for better organization.

**How**:

1. **Create migration directory**:
   ```bash
   mkdir migration
   cd migration
   ```

2. **Initialize migration project**:
   ```bash
   cargo init --lib
   ```

3. **Update `migration/Cargo.toml`**:

```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "migration"
path = "src/lib.rs"

[[bin]]
name = "migration"
path = "src/main.rs"

[dependencies]
async-std = { version = "1", features = ["attributes", "tokio1"] }

[dependencies.sea-orm-migration]
version = "1.0"
features = [
    "sqlx-postgres",
    "runtime-tokio-rustls",
]
```

4. **Create workspace** in root `Cargo.toml`:

Add at the top of your root `Cargo.toml`:

```toml
[workspace]
members = [".", "migration"]
```

**Verify**:
```bash
cd ..
cargo build
```
Should build both the main project and migration workspace.

---

### Step 3: Create Initial Migration

**Why**: Define the database schema for our memos table.

**How**:

1. **Use SeaORM CLI to create migration**:
   ```bash
   sea-orm-cli migrate generate create_memos_table
   ```

   This creates a new migration file in `migration/src/` with a timestamp prefix.

2. **Edit the generated migration file** (e.g., `migration/src/m20240115_000001_create_memos_table.rs`):

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Memos::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Memos::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_string()),
                    )
                    .col(ColumnDef::new(Memos::Title).string().not_null())
                    .col(ColumnDef::new(Memos::Description).text())
                    .col(
                        ColumnDef::new(Memos::DateTo)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Memos::Completed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Memos::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .col(
                        ColumnDef::new(Memos::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for better query performance
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_memos_completed")
                    .table(Memos::Table)
                    .col(Memos::Completed)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_memos_date_to")
                    .table(Memos::Table)
                    .col(Memos::DateTo)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_memos_created_at")
                    .table(Memos::Table)
                    .col(Memos::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Memos::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Memos {
    Table,
    Id,
    Title,
    Description,
    DateTo,
    Completed,
    CreatedAt,
    UpdatedAt,
}
```

3. **Register the migration** in `migration/src/lib.rs`:

```rust
pub use sea_orm_migration::prelude::*;

mod m20240115_000001_create_memos_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240115_000001_create_memos_table::Migration),
        ]
    }
}
```

4. **Update `migration/src/main.rs`**:

```rust
use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
```

**Verify**:
```bash
cargo build
```
Should compile without errors.

---

### Step 4: Update Environment Configuration

**Why**: Add database configuration to environment variables.

**How**:

1. **Update `.env`** file:

```bash
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3737

# Application Configuration
APP_ENV=development

# Database Configuration
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2
DATABASE_CONNECT_TIMEOUT=30
DATABASE_IDLE_TIMEOUT=600

# Logging Configuration
RUST_LOG=info,actix_web=debug,actix_memo_app=debug,sea_orm=debug
```

2. **Update `src/config/settings.rs`** to include database settings:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub app: AppConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub env: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

impl Settings {
    /// Load settings from environment variables
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let settings = Settings {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3737".to_string())
                    .parse()?,
            },
            app: AppConfig {
                env: std::env::var("APP_ENV")
                    .unwrap_or_else(|_| "development".to_string()),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set"),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()?,
                connect_timeout: std::env::var("DATABASE_CONNECT_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                idle_timeout: std::env::var("DATABASE_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()?,
            },
        };

        Ok(settings)
    }
}
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 5: Run Database Migration

**Why**: Create the database schema in PostgreSQL.

**How**:

1. **Ensure PostgreSQL is running**:
   ```bash
   # If using Docker
   docker ps | grep postgres

   # If not running, start it
   docker start postgres-dev
   ```

2. **Run the migration**:
   ```bash
   cd migration
   cargo run -- up
   ```

   Expected output:
   ```
   Applying migration 'm20240115_000001_create_memos_table'
   Migration 'm20240115_000001_create_memos_table' has been applied
   ```

3. **Verify the schema** in PostgreSQL:
   ```bash
   # Using Docker
   docker exec -it postgres-dev psql -U postgres -d memos_db -c "\d memos"

   # Using native PostgreSQL
   psql -U postgres -d memos_db -c "\d memos"
   ```

   Expected output:
   ```
                                           Table "public.memos"
      Column    |           Type           | Collation | Nullable |      Default
   -------------+--------------------------+-----------+----------+-------------------
    id          | uuid                     |           | not null | gen_random_uuid()
    title       | character varying        |           | not null |
    description | text                     |           |          |
    date_to     | timestamp with time zone |           | not null |
    completed   | boolean                  |           | not null | false
    created_at  | timestamp with time zone |           | not null | CURRENT_TIMESTAMP
    updated_at  | timestamp with time zone |           | not null | CURRENT_TIMESTAMP
   Indexes:
       "memos_pkey" PRIMARY KEY, btree (id)
       "idx_memos_completed" btree (completed)
       "idx_memos_created_at" btree (created_at)
       "idx_memos_date_to" btree (date_to)
   ```

**Verify**:
The table and indexes should exist in the database.

---

### Step 6: Generate Entity Models

**Why**: SeaORM can generate Rust structs from the database schema automatically.

**How**:

1. **Create entities directory**:
   ```bash
   cd ..
   mkdir -p src/entities
   ```

2. **Generate entities using SeaORM CLI**:
   ```bash
   sea-orm-cli generate entity \
     -u postgresql://postgres:postgres@localhost:5432/memos_db \
     -o src/entities
   ```

   This creates:
   - `src/entities/memos.rs` - The memo entity
   - `src/entities/mod.rs` - Module exports
   - `src/entities/prelude.rs` - Convenient re-exports

3. **Review generated `src/entities/memos.rs`**:

```rust
//! `SeaORM` Entity. Generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "memos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub date_to: DateTimeWithTimeZone,
    pub completed: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

4. **Update `src/lib.rs`** to include entities module:

```rust
pub mod config;
pub mod entities;
pub mod handlers;
pub mod state;
pub mod utils;
```

**Verify**:
```bash
cargo build
```
Should compile without errors.

---

### Step 7: Create Database Connection Helper

**Why**: Centralize database connection logic for reuse across the application.

**How**:

1. **Create `src/utils/database.rs`**:

```rust
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use crate::config::Settings;

/// Establish database connection with connection pool
pub async fn establish_connection(settings: &Settings) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(&settings.database.url);

    opt.max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)
        .connect_timeout(Duration::from_secs(settings.database.connect_timeout))
        .idle_timeout(Duration::from_secs(settings.database.idle_timeout))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    tracing::info!(
        "Connecting to database with max_connections={}, min_connections={}",
        settings.database.max_connections,
        settings.database.min_connections
    );

    let db = Database::connect(opt).await?;

    tracing::info!("Database connection established successfully");

    Ok(db)
}

/// Verify database connection by executing a simple query
pub async fn verify_connection(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::Statement;
    use sea_orm::ConnectionTrait;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "SELECT 1".to_owned(),
    ))
    .await?;

    Ok(())
}
```

2. **Update `src/utils/mod.rs`**:

```rust
pub mod database;
pub mod tracing;
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 8: Update Application State

**Why**: Add database connection to application state so handlers can access it.

**How**:

1. **Update `src/state.rs`**:

```rust
use crate::config::Settings;
use sea_orm::DatabaseConnection;

/// Application state shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub db: DatabaseConnection,
}

impl AppState {
    /// Create new application state with the given settings and database connection
    pub fn new(settings: Settings, db: DatabaseConnection) -> Self {
        Self { settings, db }
    }
}
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 9: Update Health Check with Database Status

**Why**: Health checks should verify all critical dependencies, including the database.

**How**:

1. **Update `src/handlers/health.rs`**:

```rust
use actix_web::{web, HttpResponse, Responder};
use sea_orm::DatabaseConnection;
use serde_json::json;

/// Health check endpoint with database connectivity check
///
/// Returns JSON with status "ok" and database status
#[tracing::instrument(skip(db))]  // Skip 'db' - it can't be logged/serialized
pub async fn health_check(db: web::Data<DatabaseConnection>) -> impl Responder {
    tracing::info!("Health check requested");

    // Check database connectivity
    let db_status = match check_database(&db).await {
        Ok(_) => "connected",
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            "disconnected"
        }
    };

    let status = if db_status == "connected" { "ok" } else { "degraded" };

    let status_code = if status == "ok" {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(json!({
        "status": status,
        "service": "actix-memo-app",
        "database": db_status
    }))
}

/// Simple readiness probe for Kubernetes
#[tracing::instrument]
pub async fn ready() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "ready": true
    }))
}

/// Check database connectivity
async fn check_database(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    crate::utils::database::verify_connection(db).await
}
```

**Verify**:
```bash
cargo check
```
Should compile without errors.

---

### Step 10: Update Main Application

**Why**: Wire the database connection into the application startup.

**How**:

1. **Update `src/main.rs`**:

```rust
mod config;
mod entities;
mod handlers;
mod state;
mod utils;

use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;
use config::Settings;
use state::AppState;
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize tracing for structured logging
    utils::tracing::init_tracing();

    tracing::info!("Starting Actix Memo Application");

    // Load configuration from environment
    let settings = Settings::load()
        .expect("Failed to load settings");

    tracing::info!(
        "Configuration loaded - Environment: {}, Server: {}:{}",
        settings.app.env,
        settings.server.host,
        settings.server.port
    );

    // Establish database connection
    let db = utils::database::establish_connection(&settings)
        .await
        .expect("Failed to connect to database");

    // Verify database connection
    utils::database::verify_connection(&db)
        .await
        .expect("Failed to verify database connection");

    tracing::info!("Database connection verified");

    // Create application state
    let app_state = AppState::new(settings.clone(), db.clone());
    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);

    tracing::info!("Starting HTTP server at {}", bind_address);

    // Create and run HTTP server
    HttpServer::new(move || {
        App::new()
            // Add application state
            .app_data(web::Data::new(app_state.clone()))
            // Share database connection separately for convenience
            .app_data(web::Data::new(db.clone()))
            // Add request logging middleware
            .wrap(TracingLogger::default())
            // Register routes
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/ready", web::get().to(handlers::health::ready))
            // Welcome route
            .route("/", web::get().to(welcome))
    })
    .bind(&bind_address)?
    .run()
    .await
}

/// Welcome endpoint
async fn welcome() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body(
        "Welcome to Actix Memo App! Try /health or /ready"
    )
}
```

**Verify**:
```bash
cargo build
```
Should compile without errors.

---

### Step 11: Test the Application

**Why**: Verify everything works together.

**How**:

1. **Ensure PostgreSQL is running**:
   ```bash
   docker ps | grep postgres
   ```

2. **Run the application**:
   ```bash
   cargo run
   ```

   Expected output:
   ```
   2025-01-15T12:00:00.000000Z  INFO actix_memo_app: Starting Actix Memo Application
   2025-01-15T12:00:00.000000Z  INFO actix_memo_app: Configuration loaded - Environment: development, Server: 127.0.0.1:3737
   2025-01-15T12:00:00.000000Z  INFO actix_memo_app::utils::database: Connecting to database with max_connections=10, min_connections=2
   2025-01-15T12:00:00.000000Z  INFO actix_memo_app::utils::database: Database connection established successfully
   2025-01-15T12:00:00.000000Z  INFO actix_memo_app: Database connection verified
   2025-01-15T12:00:00.000000Z  INFO actix_memo_app: Starting HTTP server at 127.0.0.1:3737
   ```

3. **Test health check endpoint**:
   ```bash
   curl http://localhost:3737/health | jq
   ```

   Expected output:
   ```json
   {
     "status": "ok",
     "service": "actix-memo-app",
     "database": "connected"
   }
   ```

4. **Test ready endpoint**:
   ```bash
   curl http://localhost:3737/ready | jq
   ```

   Expected output:
   ```json
   {
     "ready": true
   }
   ```

**Verify**:
Both endpoints should return successful responses, and database status should show "connected".

---

## Checkpoint

Run these commands to verify everything is working:

```bash
# Database should be running
docker ps | grep postgres
# OR
pg_isready -h localhost -p 5432

# Verify migration was applied
psql -U postgres -d memos_db -c "\dt"

# Build should succeed
cargo build

# Application should start without errors
cargo run
```

In another terminal:

```bash
# Test health endpoint
curl http://localhost:3737/health

# Test ready endpoint
curl http://localhost:3737/ready

# Check database status
curl http://localhost:3737/health | jq '.database'
```

### Expected Results

- PostgreSQL container running
- `memos` table exists with correct schema
- Application starts without errors
- Health check returns `status: "ok"` and `database: "connected"`
- Ready check returns `ready: true`
- Logs show successful database connection

---

## Common Issues and Solutions

### Issue: Failed to connect to database

**Symptoms**:
```
Error: Failed to connect to database
caused by: Connection refused
```

**Cause**: PostgreSQL not running or wrong connection parameters

**Solution**:
```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# If not running, start it
docker start postgres-dev

# Verify DATABASE_URL in .env matches your setup
cat .env | grep DATABASE_URL

# Test connection manually
psql -U postgres -d memos_db -c "SELECT 1"
```

---

### Issue: Migration fails with "relation already exists"

**Symptoms**:
```
Error: relation "memos" already exists
```

**Cause**: Migration was partially applied or run multiple times

**Solution**:
```bash
# Check migration status
cd migration
cargo run -- status

# Rollback and reapply
cargo run -- down
cargo run -- up

# Or drop and recreate database
psql -U postgres -c "DROP DATABASE memos_db;"
psql -U postgres -c "CREATE DATABASE memos_db;"
cargo run -- up
```

---

### Issue: Entity generation fails

**Symptoms**:
```
Error: error returned from database
```

**Cause**: Database URL incorrect or table doesn't exist

**Solution**:
```bash
# Verify database URL
echo $DATABASE_URL

# Ensure migration ran successfully
cd migration
cargo run -- up

# Try entity generation with explicit URL
sea-orm-cli generate entity \
  -u postgresql://postgres:postgres@localhost:5432/memos_db \
  -o src/entities \
  --with-serde both
```

---

### Issue: Too many database connections

**Symptoms**:
```
Error: remaining connection slots are reserved for non-replication superuser connections
```

**Cause**: Connection pool max_connections too high for your PostgreSQL setup

**Solution**:
```bash
# Reduce max_connections in .env
DATABASE_MAX_CONNECTIONS=5

# Or increase PostgreSQL max_connections
# Edit postgresql.conf
max_connections = 100

# Restart PostgreSQL
docker restart postgres-dev
```

---

### Issue: Compilation errors with sea-orm

**Symptoms**: Type errors or missing trait implementations

**Cause**: Version mismatch between dependencies

**Solution**:
```bash
# Ensure all SeaORM-related versions match
cargo update sea-orm
cargo update sea-orm-migration

# Clean and rebuild
cargo clean
cargo build
```
## Code Review

### Key Design Principles Demonstrated
- **Isolated database configuration** keeps connection options alongside other environment-driven settings in `Settings`.
- **Migration-first workflow** captures schema evolution in timestamped files, ensuring reproducibility across environments.
- **Generated entities** give a type-safe representation of each table without scattering raw SQL across the codebase.
- **Instrumented pooling** enables SQL logging during development, shortening the feedback loop when diagnosing queries.

### Architecture Benefits
- **Operational reliability**: Connection pooling with sensible timeouts shields PostgreSQL from bursts and surfaces failures quickly.
- **Auditable change history**: The ordered migrations directory documents every schema change with clear rollback paths.
- **Layered data access**: Application code touches the database through `DatabaseConnection`, paving the way for repositories and services to build on top.

### Complete Database Integration Structure
```rust
pub async fn establish_connection(settings: &Settings) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(&settings.database.url);
    opt.max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)
        // ... existing pooling and timeout configuration ...
        .sqlx_logging_level(log::LevelFilter::Debug);

    Database::connect(opt).await
}
```

```text
migration/
  ├─ 20240101000000_create_memos_table.rs  // up/down schema changes + indexes
  └─ seaql_migrations/                     // migration runtime support
entity/
  └─ memo.rs                               // Generated model and relation metadata
```

---

## Understanding Database Integration

### Connection Pool Lifecycle

```
1. Application starts
   ↓
2. establish_connection() called
   ↓
3. SeaORM creates connection pool
   ↓
4. min_connections established immediately
   ↓
5. Additional connections created on demand
   ↓
6. Connections reused across requests
   ↓
7. Idle connections closed after idle_timeout
   ↓
8. Pool maintained until application shutdown
```

### Migration Workflow

```
1. Developer creates migration
   sea-orm-cli migrate generate <name>
   ↓
2. Write up() and down() logic
   ↓
3. Run migration
   cargo run -- up
   ↓
4. SeaORM applies changes to database
   ↓
5. Migration tracked in seaql_migrations table
   ↓
6. Can rollback if needed
   cargo run -- down
```

### Entity Generation

SeaORM inspects the database schema and generates:
- Rust structs matching table structure
- Type mappings (SQL types → Rust types)
- Serde implementations for JSON
- Primary key and foreign key metadata
- Relation definitions (for joins)

---

## Testing

### Migration Round-Trip
1. Ensure `DATABASE_URL` is exported (or sourced from `.env`): `source .env`.
2. Recreate the schema from scratch:
   ```bash
   sea-orm-cli migrate refresh -u "$DATABASE_URL"
   ```
3. Inspect the status table:
   ```bash
   sea-orm-cli migrate status -u "$DATABASE_URL"
   ```
   All migrations should appear as `Applied`, confirming both `up` and `down` paths work.

### Migration Crate Smoke Test
Run the workspace binary exactly as CI will:
```bash
cargo run -p migration -- fresh
```
This validates the migration crate builds, SeaORM runtime features are correct, and the database accepts the connection options from `Settings`.

### Connectivity Probe
Use psql (or any SQL client) to verify the memos table exists:
```bash
psql "$DATABASE_URL" -c "\\d+ memos"
```
If the schema prints successfully, the connection pool settings and credentials configured earlier are ready for the repository layer.

## Summary

Congratulations! You've integrated PostgreSQL with your Actix Web application. You now have:

1. **Database dependencies** added to Cargo.toml
2. **Migration system** set up with SeaORM
3. **Database schema** created with memos table and indexes
4. **Entity models** generated from schema
5. **Database connection** with configured connection pool
6. **Application state** updated to include database
7. **Health checks** that verify database connectivity
8. **Configuration** for database settings via environment variables

### Key Takeaways

- **SeaORM** provides type-safe, async database operations
- **Migrations** enable version-controlled schema changes
- **Connection pooling** improves performance and resource usage
- **Entity models** represent database tables as Rust structs
- **Health checks** should verify all critical dependencies
- **Environment configuration** keeps credentials out of code

### Architecture So Far

```
┌─────────────────────────────────────┐
│        HTTP Requests                │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Handlers (with DB access)          │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  Application State                  │
│  - Settings                         │
│  - DatabaseConnection (pool)        │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  SeaORM Connection Pool             │
│  (manages multiple connections)     │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│  PostgreSQL Database                │
│  - memos table                      │
│  - indexes                          │
└─────────────────────────────────────┘
```

---

## Next Steps

### Required: Chapter 3 - Error Handling and Middleware

You'll introduce a typed error system, map failures to HTTP responses, and layer in middleware for security, compression, and observability. Expect to thread database errors through the new error types so your persistence layer cooperates with handlers.

### Optional Exercises

1. **Challenge**: Add a migration for a `tags` table and generate the corresponding entity.
2. **Challenge**: Experiment with different `max_connections` values in the connection pool and record the impact on startup logs.
3. **Challenge**: Write a small Rust binary that seeds a few memo rows using the SeaORM entities you generated.

---

## Additional Resources

### SeaORM Documentation
- [SeaORM Docs](https://www.sea-ql.org/SeaORM/) - Official documentation
- [SeaORM Tutorial](https://www.sea-ql.org/SeaORM/docs/index) - Getting started guide
- [SeaORM Cookbook](https://www.sea-ql.org/sea-orm-cookbook/) - Recipes and examples

### Database & Migrations
- [PostgreSQL Docs](https://www.postgresql.org/docs/) - PostgreSQL documentation
- [SeaORM Migration](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/) - Migration guide
- [Database Design](https://www.postgresql.org/docs/current/ddl.html) - PostgreSQL DDL

### Related Crates
- [SQLx](https://github.com/launchbadge/sqlx) - Alternative async SQL library
- [Diesel](https://diesel.rs/) - Synchronous ORM for Rust
- [tokio-postgres](https://docs.rs/tokio-postgres/) - Async PostgreSQL client

### Tutorials
- [Actix + SeaORM Tutorial](https://www.sea-ql.org/SeaORM/docs/tutorials/actix/) - Official integration guide
- [Database Connection Pooling](https://www.sea-ql.org/SeaORM/docs/install-and-config/connection/) - Pool configuration

---

**Ready to add robust error handling? Let's move on to [Chapter 3: Error Handling and Middleware](chapter-03.md)!**
