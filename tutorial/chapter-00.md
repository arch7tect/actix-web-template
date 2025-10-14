# Chapter 0: Prerequisites and Environment Setup

## Overview

Before diving into building our production-ready Actix Web application, we need to set up a proper development environment. This chapter will guide you through installing all necessary tools and verifying your setup is ready for Rust web development.

By the end of this chapter, you'll have a fully configured development environment with Rust, PostgreSQL, and all supporting tools needed to build modern web applications.

## Prerequisites

### Required Knowledge

Before starting this tutorial, you should be comfortable with:

- **Rust fundamentals**: Ownership, borrowing, lifetimes, traits
- **Async/await**: Basic understanding of asynchronous programming in Rust
- **HTTP basics**: Understanding of HTTP methods, status codes, and REST principles
- **SQL basics**: Familiarity with relational databases and basic SQL queries
- **Command line**: Comfortable navigating and running commands in a terminal

### Experience Level

This tutorial is designed for intermediate Rust developers. If you're new to Rust, we recommend completing:
- [The Rust Book](https://doc.rust-lang.org/book/) - Official Rust documentation
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by doing
- [Async Book](https://rust-lang.github.io/async-book/) - Understanding async Rust

## Learning Objectives

By completing this chapter, you will:

1. Install and configure the Rust toolchain with rustup
2. Set up PostgreSQL database (locally or via Docker)
3. Install SeaORM CLI for database migrations
4. Configure a code editor with Rust support
5. Understand the project structure we'll be building
6. Verify all tools are working correctly

## What You'll Need

### Hardware Requirements

- **Operating System**: Linux, macOS, or Windows (with WSL2 recommended)
- **RAM**: Minimum 4GB, 8GB+ recommended for comfortable development
- **Disk Space**: At least 5GB free space for Rust toolchain, dependencies, and Docker
- **CPU**: Any modern CPU (Rust compilation can be CPU-intensive)

### Internet Connection

- Required for downloading Rust toolchain, crates, and Docker images
- Stable connection recommended for initial setup

## Step-by-Step Setup

### Step 1: Install Rust Toolchain

**Why**: Rust is our primary programming language. The rustup tool manages Rust versions and associated tools.

**How**:

1. **Install rustup** (Rust version manager):

   **Linux/macOS**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   **Windows**:
   - Download and run [rustup-init.exe](https://rustup.rs/)
   - Follow the on-screen instructions
   - For best experience, use WSL2 (Windows Subsystem for Linux 2)

2. **Configure your current shell**:
   ```bash
   source $HOME/.cargo/env
   ```

3. **Verify installation**:
   ```bash
   rustc --version
   cargo --version
   rustfmt --version
   clippy-driver --version
   ```

   Expected output (versions may vary):
   ```
   rustc 1.75.0 (82e1608df 2023-12-21)
   cargo 1.75.0 (1d8b05cdd 2023-11-20)
   rustfmt 1.7.0-stable (82e1608df 2023-12-21)
   clippy 0.1.75 (82e1608d 2023-12-21)
   ```

**Verify**:
```bash
rustc --version
```
Should print the Rust compiler version without errors.

---

### Step 2: Install Build Tools (Optional but Recommended)

**Why**: Some Rust crates have C/C++ dependencies. This project uses mostly pure-Rust dependencies, but build tools are needed for:
- **Brotli compression** (`compress-brotli` feature) - Uses C bindings
- **Some OpenTelemetry components** - Potential C dependencies in the stack
- **Future dependencies** - Many popular crates (like OpenSSL-based ones) need C tooling

**Note**: If you skip this and encounter "linker not found" errors later, you can return to this step.

**How**:

**Linux (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install build-essential pkg-config
```

**Linux (Fedora/RHEL)**:
```bash
sudo dnf install gcc gcc-c++ make
```

**macOS**:
```bash
xcode-select --install
```

**Windows**:
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- Select "Desktop development with C++" workload
- Or use WSL2 and follow Linux instructions

**Verify**:
```bash
# Linux/macOS
gcc --version

# Windows (in VS Developer Command Prompt)
cl.exe
```

**What we DON'T need** (thanks to modern Rust):
- ❌ OpenSSL development libraries - We use rustls (pure Rust TLS)
- ❌ PostgreSQL client libraries - SeaORM handles this
- ❌ Most compression libraries - Pure Rust implementations available

---

### Step 3: Install PostgreSQL

You have two options: native installation or Docker. We recommend Docker for easier setup and isolation.

#### Option A: Docker (Recommended)

**Why**: Docker provides isolated, reproducible environments and is closer to production deployment.

**How**:

1. **Install Docker Desktop**:
   - **Linux**: Follow [Docker Engine installation](https://docs.docker.com/engine/install/)
   - **macOS**: Download [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop/)
   - **Windows**: Download [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop/)

2. **Install Docker Compose**:
   - Included with Docker Desktop on macOS/Windows
   - Linux: Follow [Docker Compose installation](https://docs.docker.com/compose/install/)

3. **Verify Docker installation**:
   ```bash
   docker --version
   docker-compose --version
   ```

   Expected output:
   ```
   Docker version 24.0.7, build afdd53b
   Docker Compose version v2.23.0
   ```

4. **Start PostgreSQL with Docker**:
   ```bash
   docker run --name postgres-dev \
     -e POSTGRES_USER=postgres \
     -e POSTGRES_PASSWORD=postgres \
     -e POSTGRES_DB=memos_db \
     -p 5432:5432 \
     -d postgres:16-alpine
   ```

5. **Verify PostgreSQL is running**:
   ```bash
   docker ps
   ```

   You should see the postgres-dev container running.

6. **Test connection**:
   ```bash
   docker exec -it postgres-dev psql -U postgres -d memos_db
   ```

   You should see the PostgreSQL prompt:
   ```
   psql (16.1)
   Type "help" for help.

   memos_db=#
   ```

   Type `\q` to exit.

**Verify**:
```bash
docker ps | grep postgres-dev
```
Should show a running PostgreSQL container.

#### Option B: Native Installation

**Why**: Some developers prefer native installations for performance or existing infrastructure.

**How**:

**Linux (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**Linux (Fedora/RHEL)**:
```bash
sudo dnf install postgresql-server postgresql-contrib
sudo postgresql-setup --initdb
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**macOS** (using Homebrew):
```bash
brew install postgresql@16
brew services start postgresql@16
```

**Windows**:
- Download installer from [PostgreSQL Downloads](https://www.postgresql.org/download/windows/)
- Run installer and follow setup wizard
- Remember the password you set for the postgres user

**Create database**:
```bash
# Linux/macOS
sudo -u postgres psql -c "CREATE DATABASE memos_db;"
sudo -u postgres psql -c "CREATE USER postgres WITH PASSWORD 'postgres';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE memos_db TO postgres;"

# Or using psql directly
psql -U postgres
CREATE DATABASE memos_db;
\q
```

**Verify**:
```bash
psql -U postgres -d memos_db -c "SELECT version();"
```
Should print PostgreSQL version information.

---

### Step 4: Install SeaORM CLI

**Why**: SeaORM CLI provides tools for database migrations and entity generation, essential for our ORM workflow.

**How**:

1. **Install sea-orm-cli**:
   ```bash
   cargo install sea-orm-cli
   ```

   This may take several minutes as it compiles from source.

2. **Verify installation**:
   ```bash
   sea-orm-cli --version
   ```

   Expected output:
   ```
   sea-orm-cli 1.0.0
   ```

**Verify**:
```bash
sea-orm-cli --help
```
Should display help information with available commands.

---

### Step 5: Install Additional Tools

**Why**: These tools improve development experience and code quality.

**How**:

1. **Install cargo-watch** (auto-rebuild on file changes):
   ```bash
   cargo install cargo-watch
   ```

2. **Install cargo-tarpaulin** (code coverage - Linux only):
   ```bash
   # Linux only
   cargo install cargo-tarpaulin
   ```

3. **Install sqlx-cli** (optional, for SQL verification):
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

**Verify**:
```bash
cargo watch --version
```

---

### Step 6: Configure Your Code Editor

We'll cover setup for popular editors. Choose the one you prefer.

#### Visual Studio Code (Recommended)

**Why**: Excellent Rust support, integrated terminal, and extensive plugin ecosystem.

**How**:

1. **Install VS Code**: Download from [code.visualstudio.com](https://code.visualstudio.com/)

2. **Install Rust extensions**:
   - Open VS Code
   - Press `Ctrl+Shift+X` (or `Cmd+Shift+X` on macOS)
   - Search for and install:
     - **rust-analyzer** (Rust language server)
     - **Even Better TOML** (TOML syntax highlighting)
     - **crates** (Cargo.toml dependency management)
     - **Error Lens** (inline error display)
     - **GitLens** (Git integration)

3. **Configure settings** (optional):
   - Press `Ctrl+,` (or `Cmd+,` on macOS)
   - Search for "rust-analyzer"
   - Enable:
     - Check on save
     - Format on save
     - Inlay hints

**Verify**: Open a Rust file and you should see syntax highlighting, code completion, and inline type hints.

#### RustRover (JetBrains)

**Why**: Dedicated Rust IDE with powerful refactoring and debugging tools.

**How**:

1. **Install RustRover**: Download from [jetbrains.com/rust](https://www.jetbrains.com/rust/)
2. **Install Rust plugin** (if not bundled)
3. **Configure Rust toolchain**: Settings → Languages & Frameworks → Rust

**Verify**: Create a new Rust project and verify code completion works.

#### Vim/Neovim

**Why**: Lightweight, keyboard-driven, highly customizable.

**How**:

1. **Install rust.vim**:
   ```vim
   Plug 'rust-lang/rust.vim'
   ```

2. **Install coc.nvim** with rust-analyzer:
   ```vim
   Plug 'neovim/nvim-lspconfig'
   ```

3. **Configure rust-analyzer**:
   ```lua
   require'lspconfig'.rust_analyzer.setup{}
   ```

**Verify**: Open a Rust file and `:checkhealth` should show rust-analyzer working.

---

### Step 7: Set Up Project Directory

**Why**: Understanding the project structure helps navigate and organize code effectively.

**How**:

1. **Create project directory**:
   ```bash
   mkdir -p ~/projects/actix-web-tutorial
   cd ~/projects/actix-web-tutorial
   ```

2. **Initialize Git repository** (optional but recommended):
   ```bash
   git init
   git config user.name "Your Name"
   git config user.email "your.email@example.com"
   ```

3. **Create initial .gitignore**:
   ```bash
   cat > .gitignore << 'EOF'
   # Rust
   /target/
   **/*.rs.bk
   Cargo.lock

   # IDE
   .idea/
   .vscode/
   *.swp
   *.swo
   *~

   # Environment
   .env
   .env.local

   # OS
   .DS_Store
   Thumbs.db

   # Database
   *.db
   *.sqlite

   # Logs
   *.log
   EOF
   ```

**Verify**:
```bash
ls -la
```
Should show `.git/` and `.gitignore` files.

---

### Step 8: Understanding the Project Structure

**Why**: Knowing where each component lives makes development smoother and follows Rust best practices.

**What we'll build**:

```
actix-web-template/
├── src/
│   ├── config/              # Configuration management
│   │   ├── mod.rs
│   │   └── settings.rs      # Settings struct, env loading
│   │
│   ├── docs/                # OpenAPI documentation
│   │   ├── mod.rs
│   │   └── openapi.rs       # API spec generation
│   │
│   ├── dto/                 # Data Transfer Objects
│   │   ├── mod.rs
│   │   └── memo_dto.rs      # Request/response DTOs
│   │
│   ├── entities/            # Database entities (generated)
│   │   ├── mod.rs
│   │   ├── prelude.rs
│   │   └── memos.rs         # Memo entity model
│   │
│   ├── error/               # Error handling
│   │   ├── mod.rs
│   │   └── app_error.rs     # Custom error types
│   │
│   ├── handlers/            # HTTP request handlers
│   │   ├── mod.rs
│   │   ├── health.rs        # Health check endpoints
│   │   ├── memos.rs         # REST API handlers
│   │   └── web.rs           # HTML page handlers
│   │
│   ├── middleware/          # Custom middleware
│   │   ├── mod.rs
│   │   ├── rate_limit.rs    # Rate limiting
│   │   └── security_headers.rs  # Security headers
│   │
│   ├── repository/          # Database access layer
│   │   ├── mod.rs
│   │   └── memo_repository.rs
│   │
│   ├── services/            # Business logic layer
│   │   ├── mod.rs
│   │   └── memo_service.rs
│   │
│   ├── utils/               # Utility functions
│   │   ├── mod.rs
│   │   ├── sanitize.rs      # Input sanitization
│   │   └── tracing.rs       # Logging setup
│   │
│   ├── state.rs             # Application state
│   ├── lib.rs               # Library root
│   └── main.rs              # Application entry point
│
├── migration/               # Database migrations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   └── m20250109_create_memos_table.rs
│   └── Cargo.toml
│
├── templates/               # Askama HTML templates
│   ├── base.html
│   ├── pages/
│   ├── components/
│   └── partials/
│
├── static/                  # Static assets
│   └── css/
│       └── style.css
│
├── tests/                   # Integration tests
│   ├── common/
│   ├── api_tests.rs
│   ├── repository_tests.rs
│   ├── service_tests.rs
│   └── web_tests.rs
│
├── .env.example             # Example environment variables
├── .gitignore
├── Cargo.toml               # Project dependencies
├── Dockerfile
├── docker-compose.yml
└── README.md
```

**Layer responsibilities**:

1. **Handlers**: HTTP request/response handling, input validation
2. **Services**: Business logic, DTO conversions, orchestration
3. **Repository**: Database operations, queries
4. **Entities**: Database schema representation (SeaORM models)
5. **DTOs**: API contracts with validation rules
6. **Middleware**: Cross-cutting concerns (logging, security, rate limiting)

---

## Environment Variables Setup

**Why**: Separating configuration from code is a production best practice (12-factor app methodology).

**How**:

1. **Create .env file**:
   ```bash
   cat > .env << 'EOF'
   # Server Configuration
   SERVER_HOST=127.0.0.1
   SERVER_PORT=3737
   APP_ENV=development

   # Database Configuration
   DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db
   DATABASE_MAX_CONNECTIONS=10
   DATABASE_CONNECT_TIMEOUT=30

   # Logging Configuration
   RUST_LOG=debug
   LOG_FORMAT=pretty

   # Security Configuration
   CORS_ALLOWED_ORIGINS=*
   MAX_REQUEST_SIZE=262144

   # Features
   ENABLE_SWAGGER=true
   EOF
   ```

2. **Adjust DATABASE_URL if using Docker**:
   - If PostgreSQL is in Docker: `postgresql://postgres:postgres@localhost:5432/memos_db`
   - If using Docker Compose (we'll set up later): `postgresql://postgres:postgres@postgres:5432/memos_db`

**Verify**:
```bash
cat .env
```
Should display your environment variables.

---

## Checkpoint: Verify Your Setup

Run these commands to ensure everything is installed correctly:

```bash
# Rust toolchain
rustc --version
cargo --version
rustfmt --version
clippy-driver --version

# Database
psql -U postgres -d memos_db -c "SELECT 1;"
# OR if using Docker:
docker exec -it postgres-dev psql -U postgres -d memos_db -c "SELECT 1;"

# SeaORM CLI
sea-orm-cli --version

# Optional tools
cargo watch --version
```

### Expected Results

All commands should complete without errors and print version information or query results.

### Test Database Connection

Create a simple test to verify database connectivity:

```bash
# Using Docker
docker exec -it postgres-dev psql -U postgres -d memos_db -c "
CREATE TABLE IF NOT EXISTS test (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100)
);
INSERT INTO test (name) VALUES ('Hello, Rust!');
SELECT * FROM test;
DROP TABLE test;
"

# Using native PostgreSQL
psql -U postgres -d memos_db -c "
CREATE TABLE IF NOT EXISTS test (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100)
);
INSERT INTO test (name) VALUES ('Hello, Rust!');
SELECT * FROM test;
DROP TABLE test;
"
```

Expected output:
```
CREATE TABLE
INSERT 0 1
 id |     name
----+---------------
  1 | Hello, Rust!
(1 row)

DROP TABLE
```

---

## Common Issues and Solutions

### Issue: rustup command not found

**Symptoms**: `rustup: command not found` or similar error

**Cause**: Rust installation didn't add to PATH, or shell needs restart

**Solution**:
```bash
# Add to PATH manually
source $HOME/.cargo/env

# Or restart your shell
exec $SHELL

# Verify PATH contains cargo bin
echo $PATH | grep cargo
```

---

### Issue: PostgreSQL connection refused

**Symptoms**: `could not connect to server: Connection refused`

**Cause**: PostgreSQL not running or wrong connection parameters

**Solution**:
```bash
# If using Docker, ensure container is running
docker ps | grep postgres

# If not running, start it
docker start postgres-dev

# If using native PostgreSQL
sudo systemctl status postgresql
sudo systemctl start postgresql

# Verify port 5432 is listening
lsof -i :5432
# OR
netstat -an | grep 5432
```

---

### Issue: Permission denied on PostgreSQL

**Symptoms**: `FATAL: role "postgres" does not exist` or permission errors

**Cause**: User or database not properly created

**Solution**:
```bash
# Using Docker - recreate container
docker rm -f postgres-dev
docker run --name postgres-dev \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=memos_db \
  -p 5432:5432 \
  -d postgres:16-alpine

# Using native PostgreSQL
sudo -u postgres createuser -s $USER
sudo -u postgres createdb memos_db
```

---

### Issue: Cargo build tools missing (Linux)

**Symptoms**: `linker 'cc' not found` or similar compilation errors

**Cause**: Missing C/C++ build tools

**Solution**:
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install gcc gcc-c++ make openssl-devel
```

---

### Issue: sea-orm-cli installation fails

**Symptoms**: Compilation errors during `cargo install sea-orm-cli`

**Cause**: Usually missing system dependencies or rustc version too old

**Solution**:
```bash
# Update Rust to latest stable
rustup update stable

# Ensure OpenSSL development libraries are installed
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# macOS
brew install openssl@3

# Retry installation
cargo install sea-orm-cli --locked
```

---

## Understanding Key Concepts

Before moving to Chapter 1, let's clarify some important concepts:

### Cargo Workspace

A Cargo workspace allows multiple related packages to share dependencies and configuration. We'll use this for our main application and migration subproject.

### Environment-Based Configuration

We separate configuration (environment variables) from code to:
- Support different environments (development, staging, production)
- Keep secrets out of version control
- Follow 12-factor app methodology

### Async Runtime (Tokio)

Actix Web runs on Tokio, Rust's async runtime. This enables:
- Concurrent request handling without threads for each request
- Efficient I/O operations
- Better resource utilization

### ORM vs Raw SQL

We use SeaORM (Object-Relational Mapping) to:
- Write type-safe database queries in Rust
- Automatically generate SQL
- Handle migrations systematically
- Reduce boilerplate code

---

## Summary

Congratulations! You've completed the environment setup. You now have:

1. **Rust toolchain** installed and verified (rustc, cargo, rustfmt, clippy)
2. **PostgreSQL database** running (Docker or native)
3. **SeaORM CLI** ready for migrations
4. **Code editor** configured with Rust support
5. **Project structure** understanding
6. **Environment variables** configured
7. **All tools** verified and working

### Key Takeaways

- Rust development requires a proper toolchain (rustc, cargo, and friends)
- PostgreSQL provides our reliable, ACID-compliant database
- SeaORM CLI manages database schema evolution
- Environment variables separate configuration from code
- Docker provides consistent, isolated development environments
- The layered architecture (handlers → services → repositories → entities) keeps code organized

### What's Next

In **Chapter 1: Core Application Setup**, we'll:
- Create a new Rust project with Cargo
- Set up the basic Actix Web server
- Implement configuration loading from environment variables
- Create application state
- Build our first HTTP endpoint
- Test the server is running

---

## Additional Resources

### Official Documentation
- [Rust Book](https://doc.rust-lang.org/book/) - Complete Rust guide
- [Cargo Book](https://doc.rust-lang.org/cargo/) - Cargo package manager
- [Actix Web](https://actix.rs/) - Web framework documentation
- [SeaORM](https://www.sea-ql.org/SeaORM/) - ORM documentation
- [PostgreSQL](https://www.postgresql.org/docs/) - Database documentation

### Rust Learning Resources
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by doing
- [Rustlings](https://github.com/rust-lang/rustlings) - Small exercises
- [Async Book](https://rust-lang.github.io/async-book/) - Async programming in Rust

### Community
- [Rust Users Forum](https://users.rust-lang.org/) - Get help from the community
- [Rust Discord](https://discord.gg/rust-lang) - Real-time chat
- [r/rust](https://www.reddit.com/r/rust/) - Rust subreddit

### Tools
- [crates.io](https://crates.io/) - Rust package registry
- [docs.rs](https://docs.rs/) - Documentation for all crates
- [Rust Playground](https://play.rust-lang.org/) - Try Rust in the browser

---

**Ready to code? Let's move on to [Chapter 1: Core Application Setup](chapter-01.md)!**