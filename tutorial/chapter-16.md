# Chapter 16: CI/CD Pipeline with GitHub Actions

## Overview

In this chapter, you'll set up a complete CI/CD pipeline using GitHub Actions. You'll learn how to automate testing, linting, security audits, releases, and deployments. The pipeline ensures code quality, catches bugs early, and streamlines the release process.

By the end of this chapter, your repository will have automated workflows that run tests on every push, enforce code standards, build release artifacts, publish Docker images, and optionally deploy to staging and production environments.

## Prerequisites

### Completed Chapters
- Chapters 0-15: Full application with Docker deployment

### Required Knowledge
- Git fundamentals (branches, commits, tags)
- GitHub repository management
- YAML syntax basics
- Understanding of CI/CD concepts

### Required Setup
- GitHub account
- Repository hosted on GitHub
- Write access to repository settings

## Learning Objectives

By the end of this chapter, you will:

- Understand GitHub Actions architecture (workflows, jobs, steps, runners)
- Implement automated testing on every push and pull request
- Enforce code quality with linters and formatters
- Run security audits to detect vulnerable dependencies
- Automate release creation with multi-platform binaries
- Build and publish Docker images to GitHub Container Registry
- Set up deployment workflows for staging and production
- Use caching to speed up CI builds
- Configure matrix builds for cross-platform testing
- Implement branch protection rules

## Concepts Covered

### GitHub Actions Architecture

GitHub Actions uses YAML files in `.github/workflows/` to define automation pipelines.

**Key concepts:**

- **Workflow**: A YAML file defining an automated process
- **Event**: Triggers that start workflows (push, pull_request, tags, schedule)
- **Job**: A set of steps that execute on the same runner
- **Step**: Individual task within a job
- **Runner**: Virtual machine that executes jobs (ubuntu, macos, windows)
- **Action**: Reusable unit of code (checkout, cache, docker/build-push-action)

**Workflow execution:**

```
GitHub Event (push, PR, tag)
    ↓
Workflow Triggered
    ↓
Jobs Start (can run in parallel)
    ↓
Steps Execute (sequential within job)
    ↓
Results Reported (success/failure)
```

### CI/CD Pipeline Strategy

Our pipeline has four main workflows:

1. **Test** (`test.yml`): Run tests on every push/PR
2. **Lint** (`lint.yml`): Enforce code quality standards
3. **Release** (`release.yml`): Build and publish releases on tags
4. **Deploy** (`deploy.yml`): Deploy to environments (template)

**Flow:**

```
Developer pushes code
    ↓
Test + Lint workflows run
    ↓ (pass)
Code merged to master
    ↓ (on tag)
Release workflow builds artifacts
    ↓
Docker image published to GHCR
    ↓ (optional)
Deploy workflow pushes to staging/production
```

## Step-by-Step Instructions

### Step 1: Understanding the Test Workflow

The test workflow runs your test suite on every push and pull request.

**File: `.github/workflows/test.yml`**

```yaml
name: Test

on:
  push:
    branches: [ master, stage-* ]
  pull_request:
    branches: [ master ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: testuser
          POSTGRES_PASSWORD: testpass
          POSTGRES_DB: testdb
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-registry-

      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-index-

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-build-target-

      - name: Set up environment
        run: |
          cp .env.example .env
          echo "DATABASE_URL=postgres://testuser:testpass@localhost:5432/testdb" >> .env

      - name: Run migrations
        run: |
          cd migration && cargo run
        env:
          DATABASE_URL: postgres://testuser:testpass@localhost:5432/testdb

      - name: Check code
        run: cargo check --verbose

      - name: Run tests
        run: cargo test --verbose
        env:
          DATABASE_URL: postgres://testuser:testpass@localhost:5432/testdb
          RUST_LOG: info

      - name: Run doc tests
        run: cargo test --doc
        env:
          DATABASE_URL: postgres://testuser:testpass@localhost:5432/testdb

  test-matrix:
    name: Test on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Cache dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}

      - name: Check build
        run: cargo check --verbose

      - name: Run unit tests only (no DB)
        run: cargo test --lib --verbose
```

**Key sections explained:**

#### Workflow Triggers

```yaml
on:
  push:
    branches: [ master, stage-* ]
  pull_request:
    branches: [ master ]
```

- **push**: Runs when code is pushed to `master` or stage branches
- **pull_request**: Runs when PR is opened/updated targeting `master`
- This catches issues before code is merged

#### Service Containers

```yaml
services:
  postgres:
    image: postgres:16-alpine
    # ... health check configuration
```

- GitHub Actions can run database containers alongside your workflow
- Uses Docker under the hood (just like Chapter 15!)
- Health checks ensure database is ready before tests run
- Database is automatically cleaned up after job completes

**Why this is powerful:**
- No need to mock database in tests
- Tests run against real PostgreSQL
- Same behavior as local development and production

#### Caching Strategy

```yaml
- name: Cache cargo registry
  uses: actions/cache@v4
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
```

**Three separate caches:**
1. **Cargo registry**: Downloaded crate metadata (~100MB)
2. **Cargo index**: Git index of crates.io (~200MB)
3. **Build artifacts**: Compiled dependencies (~2GB)

**Cache key:** `${{ hashFiles('**/Cargo.lock') }}`
- Cache is reused if `Cargo.lock` hasn't changed
- When dependencies update, cache is rebuilt
- **Speeds up builds by 5-10x** (30 seconds vs 5 minutes)

#### Matrix Testing

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable]
```

- **Matrix builds** run tests on multiple OS/Rust combinations
- Ensures your app works cross-platform
- Runs in parallel (GitHub provides multiple runners)
- Catches platform-specific bugs early

**Note:** Matrix job runs unit tests only (no database) for speed.

### Step 2: Understanding the Lint Workflow

The lint workflow enforces code quality standards.

**File: `.github/workflows/lint.yml`**

```yaml
name: Lint

on:
  push:
    branches: [ master, stage-* ]
  pull_request:
    branches: [ master ]

env:
  CARGO_TERM_COLOR: always

jobs:
  fmt:
    name: Format Check
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-clippy-${{ hashFiles('**/Cargo.lock') }}

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run security audit
        run: cargo audit

  deny:
    name: Cargo Deny
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install cargo-deny
        run: cargo install cargo-deny

      - name: Run cargo deny
        run: cargo deny check

  doc:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-doc-${{ hashFiles('**/Cargo.lock') }}

      - name: Check documentation
        run: cargo doc --no-deps --document-private-items --all-features
        env:
          RUSTDOCFLAGS: -D warnings
```

**Five quality checks:**

#### 1. Format Check (`fmt`)

```yaml
- name: Check formatting
  run: cargo fmt --all -- --check
```

- Ensures code follows Rust formatting standards
- **Fails if code is not formatted** (run `cargo fmt` before committing)
- Fast check (< 5 seconds)
- Maintains consistent code style across contributors

#### 2. Clippy (`clippy`)

```yaml
- name: Run clippy
  run: cargo clippy --all-targets --all-features -- -D warnings
```

- **Clippy**: Rust's linter with 550+ rules
- `-D warnings`: Treat warnings as errors (strict mode)
- Catches common mistakes, inefficiencies, and anti-patterns
- Examples:
  - Unused variables
  - Inefficient loops
  - Missing error handling
  - Non-idiomatic code

**Common clippy fixes:**

1. **Collapsible if statements** (`clippy::collapsible_if`):
```rust
// Before (nested ifs)
if let Some(ref order) = self.order {
    if order != "asc" && order != "desc" {
        return Err("Invalid order".to_string());
    }
}

// After (collapsed with let-chain)
if let Some(ref order) = self.order
    && order != "asc" && order != "desc"
{
    return Err("Invalid order".to_string());
}
```

2. **Redundant pattern matching** (`clippy::redundant_pattern_matching`):
```rust
// Before
if let Err(_) = result {
    handle_error();
}

// After
if result.is_err() {
    handle_error();
}
```

Run `cargo clippy --fix` to auto-fix many issues.

#### 3. Security Audit (`audit`)

```yaml
- name: Run security audit
  run: cargo audit
```

- **cargo-audit**: Checks for security vulnerabilities in dependencies
- Queries RustSec Advisory Database
- Fails if any dependencies have known CVEs
- **Critical for production security**

**Example output:**
```
error: 1 vulnerability found!
┌───────────────────────────────────────────────────────────────────────────┐
│ ID      │ RUSTSEC-2023-0071                                                │
│ Crate   │ tokio                                                            │
│ Version │ 1.28.0                                                           │
│ Fix     │ >=1.29.0                                                         │
│ Title   │ tokio: reject_remote_clients opens a denial of service vector   │
└───────────────────────────────────────────────────────────────────────────┘
```

**Configuring cargo-audit exceptions:**

Sometimes vulnerabilities appear in dependencies you don't actually use at runtime (e.g., MySQL features when you only use PostgreSQL). You can document these exceptions in `.cargo/audit.toml`:

```toml
# .cargo/audit.toml
[advisories]
# Document why each advisory is ignored
ignore = [
    "RUSTSEC-2023-0071",  # rsa crate (via sqlx-mysql, we only use PostgreSQL)
]
```

**Important:**
- Only ignore advisories you've thoroughly investigated
- Document WHY each is ignored (future maintainers need context)
- Re-evaluate ignored advisories periodically
- This is transparency, not hiding security issues

#### 4. Cargo Deny (`deny`)

```yaml
- name: Run cargo deny
  run: cargo deny check
```

- **cargo-deny**: Enforces policies on dependencies
- Checks:
  - **License compliance**: Reject incompatible licenses (GPL, proprietary)
  - **Duplicate dependencies**: Multiple versions of same crate
  - **Banned crates**: Crates you've explicitly forbidden
  - **Source verification**: Ensure crates from trusted sources

**Configuration:** `deny.toml` file (optional)

```toml
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
deny = ["GPL-3.0"]

[bans]
deny = [
    { name = "openssl", reason = "Use rustls instead" }
]
```

#### 5. Documentation Check (`doc`)

```yaml
- name: Check documentation
  run: cargo doc --no-deps --document-private-items --all-features
  env:
    RUSTDOCFLAGS: -D warnings
```

- Builds documentation and fails on warnings
- Ensures all public items are documented
- Catches broken doc links
- `RUSTDOCFLAGS: -D warnings`: Treat missing docs as errors

### Step 3: Understanding the Release Workflow

The release workflow builds production artifacts and publishes them.

**File: `.github/workflows/release.yml`**

Key sections:

#### Multi-Platform Binary Builds

```yaml
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            asset_name: actix-web-template-linux-amd64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            asset_name: actix-web-template-linux-musl-amd64
          - os: macos-latest
            target: x86_64-apple-darwin
            asset_name: actix-web-template-macos-amd64
          - os: macos-latest
            target: aarch64-apple-darwin
            asset_name: actix-web-template-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            asset_name: actix-web-template-windows-amd64.exe

    steps:
      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Strip binary (Linux and macOS)
        if: matrix.os != 'windows-latest'
        run: strip target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
```

**5 different binaries:**
1. **Linux x86_64 (glibc)**: Standard Linux
2. **Linux x86_64 (musl)**: Static binary (no libc dependency)
3. **macOS x86_64**: Intel Macs
4. **macOS arm64**: Apple Silicon (M1/M2/M3)
5. **Windows x86_64**: Windows systems

**Why musl?**
- Statically linked (works on any Linux, even without glibc)
- Smaller binary
- Useful for Alpine Docker images

**Strip binary:**
- Removes debug symbols
- Reduces binary size by ~30%
- `strip` not available on Windows (uses MSVC toolchain)

#### GitHub Release Creation

```yaml
  create-release:
    name: Create GitHub Release
    needs: build
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/')
    permissions:
      contents: write
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          draft: false
          prerelease: ${{ contains(github.ref, 'alpha') || contains(github.ref, 'beta') || contains(github.ref, 'rc') }}
          generate_release_notes: true
          files: artifacts/**/*
```

**Automatic release:**
- `needs: build`: Waits for all platform builds to complete
- Downloads all artifacts (5 binaries)
- Creates GitHub Release with auto-generated notes
- Marks as prerelease if tag contains `alpha`, `beta`, or `rc`
- Attaches all binaries to release

**Usage:**
```bash
# Create release
git tag v0.3.0
git push origin v0.3.0

# GitHub Actions automatically:
# 1. Builds 5 binaries
# 2. Creates release page
# 3. Uploads binaries
# 4. Generates release notes from commits
```

#### Docker Image Publishing

```yaml
  docker:
    name: Build and Push Docker Image
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    permissions:
      contents: read
      packages: write
    steps:
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=semver,pattern={{major}}
            type=sha,prefix={{branch}}-
            type=raw,value=latest,enable={{is_default_branch}}

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
          platforms: linux/amd64,linux/arm64
```

**Automatic Docker publishing:**

1. **Tag-based trigger**: Only runs on version tags (`v*`)
2. **GitHub Container Registry (GHCR)**: Free Docker registry for GitHub repos
3. **Multi-platform**: Builds for amd64 and arm64
4. **Smart tagging**: Creates multiple tags automatically

**Tag generation example:**

```bash
# Tag v0.3.0 creates:
ghcr.io/arch7tect/actix-web-template:0.3.0      # Full version
ghcr.io/arch7tect/actix-web-template:0.3        # Major.minor
ghcr.io/arch7tect/actix-web-template:0          # Major only
ghcr.io/arch7tect/actix-web-template:latest     # Latest tag
ghcr.io/arch7tect/actix-web-template:master-abc123  # Commit SHA
```

**Benefits:**
- Users can pin to exact versions: `:0.3.0`
- Or track minor updates: `:0.3`
- Or always get latest: `:latest`

**GitHub Actions caching:**
- `cache-from: type=gha`: Use GitHub Actions cache
- **Much faster** than rebuilding from scratch (5 min → 30 sec)

#### Docker Compose Testing

```yaml
  docker-compose-test:
    name: Test Docker Compose
    runs-on: ubuntu-latest
    steps:
      - name: Test docker-compose build
        run: docker compose build

      - name: Start services
        run: docker compose up -d

      - name: Wait for services
        run: sleep 20

      - name: Check health endpoint
        run: |
          curl --fail http://localhost:3737/health || exit 1

      - name: Stop services
        run: docker compose down
```

- Ensures `docker-compose.yml` is valid
- Tests that app starts successfully
- Verifies health endpoint responds
- Catches Docker configuration issues before deployment

### Step 4: Understanding the Deploy Workflow

The deploy workflow is a **template** for staging and production deployments.

**File: `.github/workflows/deploy.yml`**

```yaml
name: Deploy

on:
  workflow_dispatch:
    inputs:
      environment:
        description: 'Deployment environment'
        required: true
        default: 'staging'
        type: choice
        options:
          - staging
          - production
  push:
    tags:
      - 'v*'

jobs:
  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    if: github.event.inputs.environment == 'staging' || (github.event_name == 'push' && contains(github.ref, 'beta'))
    environment:
      name: staging
      url: https://staging.example.com
    steps:
      - name: Deploy to staging
        run: |
          echo "Deploying to staging environment..."
          # Add your staging deployment commands here

  deploy-production:
    name: Deploy to Production
    runs-on: ubuntu-latest
    if: github.event.inputs.environment == 'production' || (github.event_name == 'push' && !contains(github.ref, 'beta'))
    environment:
      name: production
      url: https://example.com
    steps:
      - name: Deploy to production
        run: |
          echo "Deploying to production environment..."
          # Add your production deployment commands here
```

**Key features:**

#### Manual Triggers

```yaml
on:
  workflow_dispatch:
    inputs:
      environment:
        type: choice
        options:
          - staging
          - production
```

- **workflow_dispatch**: Manually trigger from GitHub UI
- Choose environment from dropdown
- Useful for ad-hoc deployments

**Usage:** GitHub → Actions → Deploy → Run workflow → Choose environment

#### Environment Protection

```yaml
environment:
  name: production
  url: https://example.com
```

- **GitHub Environments**: Add protection rules
- Settings → Environments → production
- **Protection options:**
  - Require approval from specific reviewers
  - Wait timer (e.g., 10 minutes before deploying)
  - Restrict to specific branches
  - Environment-specific secrets

**Example:** Production requires approval from 2 team members

#### Deployment Strategies

The workflow is a template. Add your deployment method:

**Option 1: Docker on a VM (SSH)**

```yaml
- name: Deploy via SSH
  uses: appleboy/ssh-action@master
  with:
    host: ${{ secrets.DEPLOY_HOST }}
    username: ${{ secrets.DEPLOY_USER }}
    key: ${{ secrets.DEPLOY_SSH_KEY }}
    script: |
      docker pull ghcr.io/arch7tect/actix-web-template:${{ github.ref_name }}
      cd /opt/actix-web-template
      docker-compose pull
      docker-compose up -d
```

**Option 2: Kubernetes**

```yaml
- name: Deploy to Kubernetes
  uses: azure/k8s-deploy@v4
  with:
    manifests: |
      k8s/deployment.yml
      k8s/service.yml
    images: |
      ghcr.io/arch7tect/actix-web-template:${{ github.ref_name }}
```

**Option 3: Cloud Platforms**

```yaml
# AWS ECS
- name: Deploy to ECS
  uses: aws-actions/amazon-ecs-deploy-task-definition@v1

# Azure Container Instances
- name: Deploy to Azure
  uses: azure/aci-deploy@v1

# Google Cloud Run
- name: Deploy to Cloud Run
  uses: google-github-actions/deploy-cloudrun@v1
```

### Step 5: Setting Up Branch Protection

Configure branch protection to enforce quality standards.

**Navigate to:** GitHub → Settings → Branches → Add rule

**Configuration for `master` branch:**

```yaml
Branch name pattern: master

✅ Require a pull request before merging
   ✅ Require approvals: 1
   ✅ Dismiss stale pull request approvals when new commits are pushed

✅ Require status checks to pass before merging
   ✅ Require branches to be up to date before merging
   Status checks:
   - Test Suite
   - Format Check
   - Clippy
   - Security Audit
   - Cargo Deny
   - Documentation

✅ Require conversation resolution before merging

✅ Include administrators (applies rules to admins too)

✅ Restrict who can push to matching branches
   (Optional: Limit to specific users/teams)
```

**What this does:**
- **No direct pushes to master**: Must use pull requests
- **All checks must pass**: Tests, lints, security audits
- **Requires review**: At least 1 approval
- **Conversations resolved**: All PR comments addressed
- **Up to date**: Branch must be current with master

**Result:** Higher code quality, fewer bugs in production

### Step 6: Configuring GitHub Secrets

Add secrets for deployment and Docker registry access.

**Navigate to:** GitHub → Settings → Secrets and variables → Actions

**Required secrets:**

```yaml
# Docker (automatically available)
GITHUB_TOKEN: Provided by GitHub automatically

# Deployment (add these if using deploy.yml)
DEPLOY_HOST: your-server.example.com
DEPLOY_USER: deploy
DEPLOY_SSH_KEY: -----BEGIN OPENSSH PRIVATE KEY-----...

# Cloud providers (if applicable)
AWS_ACCESS_KEY_ID: AKIA...
AWS_SECRET_ACCESS_KEY: ...
AZURE_CREDENTIALS: {...}
GCP_SA_KEY: {...}

# Notifications (optional)
SLACK_WEBHOOK_URL: https://hooks.slack.com/services/...
```

**Security best practices:**
- **Never commit secrets** to repository
- Use environment-specific secrets (staging vs production)
- Rotate secrets regularly
- Use minimal permissions (principle of least privilege)
- Audit secret usage in workflow logs

### Step 7: Viewing Workflow Results

Monitor and debug CI/CD workflows.

**View workflow runs:**

1. GitHub → Actions tab
2. See all workflow runs (Test, Lint, Release, Deploy)
3. Click a run to see details

**Workflow status indicators:**

- ✅ **Green checkmark**: All jobs passed
- ❌ **Red X**: At least one job failed
- 🟡 **Yellow dot**: In progress
- ⭕ **Gray circle**: Queued

**Detailed logs:**

```
Actions → Choose workflow → Click run → Click job → View logs
```

**Example: Failed test**

```
Run cargo test --verbose
   Compiling actix-web-template v0.2.1
    Finished test [unoptimized + debuginfo] target(s) in 1m 23s
     Running unittests src/lib.rs (target/debug/deps/actix_web_template-...)

running 10 tests
test test_create_memo ... ok
test test_list_memos ... FAILED
test test_update_memo ... ok

failures:

---- test_list_memos stdout ----
thread 'test_list_memos' panicked at 'assertion failed: response.status().is_success()'

Error: Process completed with exit code 101.
```

**Debugging:**
- Click on failed step to see full logs
- Download logs for offline analysis
- Re-run failed jobs (Actions → Re-run all jobs)

### Step 8: Using the Workflows

Practical examples of using the CI/CD pipeline.

#### Scenario 1: Feature Development

```bash
# 1. Create feature branch
git checkout -b feature/new-endpoint

# 2. Make changes
vim src/handlers/memos.rs

# 3. Commit and push
git add .
git commit -m "Add bulk delete endpoint"
git push origin feature/new-endpoint

# GitHub Actions automatically:
# - Runs tests
# - Runs lints
# - Posts results as checks on commits
```

**View results:** GitHub shows ✅ or ❌ next to commit

#### Scenario 2: Pull Request

```bash
# 1. Create PR from feature branch
# GitHub → New Pull Request

# GitHub Actions automatically:
# - Runs all checks on PR
# - Updates status as code changes
# - Blocks merge if checks fail

# 2. Address review feedback
git commit -m "Fix review comments"
git push  # Checks run again

# 3. Merge when approved and checks pass
# GitHub → Merge pull request
```

**Branch protection prevents merge if:**
- Tests fail
- Code not formatted
- Clippy warnings
- Security vulnerabilities found
- No reviewer approval

#### Scenario 3: Creating a Release

```bash
# 1. Update version
vim Cargo.toml  # version = "0.3.0"
git commit -am "Bump version to 0.3.0"

# 2. Create and push tag
git tag v0.3.0
git push origin v0.3.0

# GitHub Actions automatically:
# - Builds binaries for 5 platforms
# - Creates GitHub Release
# - Uploads binaries to release
# - Builds Docker images (amd64 + arm64)
# - Pushes images to ghcr.io
# - Creates multiple tags (0.3.0, 0.3, 0, latest)
```

**Result:** Full release ready in ~10-15 minutes

#### Scenario 4: Manual Deployment

```bash
# GitHub → Actions → Deploy workflow → Run workflow
# Choose: staging
# Click: Run workflow

# Workflow runs deployment to staging
# Check logs for status
# Verify: https://staging.example.com/health

# If staging looks good:
# Repeat for production (with approval gates)
```

## Checkpoint

Verify your CI/CD pipeline is working correctly.

**Run these checks:**

```bash
# 1. Check workflows exist
ls .github/workflows/
# Expected: test.yml, lint.yml, release.yml, deploy.yml

# 2. Trigger test workflow
git checkout -b test-ci
echo "# Test" >> README.md
git add README.md
git commit -m "Test CI"
git push origin test-ci

# 3. View results in GitHub
# GitHub → Actions → See Test and Lint workflows running

# 4. Create pull request
# GitHub → New Pull Request
# See checks appear on PR

# 5. Create test release (optional, only if ready)
git tag v0.2.2-test
git push origin v0.2.2-test
# Watch Release workflow build binaries

# 6. Clean up test tag (if created)
git push --delete origin v0.2.2-test
git tag -d v0.2.2-test
```

**Expected results:**

- ✅ Test workflow passes in < 5 minutes
- ✅ Lint workflow passes in < 3 minutes
- ✅ Branch protection prevents merge if checks fail
- ✅ Release workflow creates artifacts on tags
- ✅ Docker images published to GHCR

## Code Review

### CI/CD Architecture Benefits

**Automated quality gates:**
- Every code change runs through tests, lints, and security checks
- Catches bugs before they reach production
- Enforces consistent code style across team
- Detects security vulnerabilities early

**Fast feedback loops:**
- Developers see test results within minutes
- No waiting for manual QA before finding issues
- Failures point to exact line of code
- Clear logs for debugging

**Reproducible builds:**
- Same environment every time (GitHub runners)
- No "works on my machine" issues
- Dependency caching for speed
- Matrix testing ensures cross-platform compatibility

**Streamlined releases:**
- One command to create release (`git tag`)
- Automated binary builds for all platforms
- Docker images built and published automatically
- Release notes generated from commits

### Workflow Design Patterns

**Separation of concerns:**
- Test workflow: Functional correctness
- Lint workflow: Code quality and security
- Release workflow: Artifact building
- Deploy workflow: Environment management

**Fail fast principle:**
- Quick checks first (format: 5s, clippy: 30s, tests: 2m)
- Expensive checks later (multi-platform builds: 15m)
- Parallel execution where possible

**Caching strategy:**
- Cargo registry, index, and build artifacts cached
- Cache key based on `Cargo.lock` hash
- Dramatically speeds up builds (5min → 30s)

## Common Issues and Solutions

### Issue: Workflows not triggering

**Symptoms**: Push code but workflows don't run

**Causes:**
- Workflow file has syntax error
- Event trigger doesn't match (wrong branch)
- Workflows disabled in repository settings

**Solutions:**

```bash
# 1. Validate YAML syntax
# Use yamllint or online validator
cat .github/workflows/test.yml | yamllint -

# 2. Check workflow triggers
on:
  push:
    branches: [ master ]  # Must match your branch name

# 3. Enable workflows
# GitHub → Settings → Actions → Allow all actions

# 4. Check Actions permissions
# Settings → Actions → General → Workflow permissions → Read and write
```

### Issue: Tests pass locally but fail in CI

**Symptoms**: `cargo test` works on your machine, fails in GitHub Actions

**Common causes:**

**1. Different database:**

```bash
# Local
DATABASE_URL=postgres://localhost/memos_db

# CI
DATABASE_URL=postgres://testuser:testpass@localhost:5432/testdb
```

**Solution:** Use .env.test that matches CI configuration

**2. Missing environment variables:**

```yaml
# Add to workflow
- name: Run tests
  env:
    RUST_LOG: info
    DATABASE_URL: ...
```

**3. Timezone differences:**

```rust
// Avoid timezone-dependent tests
// Use UTC everywhere
use chrono::Utc;
let now = Utc::now();
```

**4. Race conditions:**

```bash
# Run tests serially if they share database state
cargo test -- --test-threads=1
```

### Issue: Caching not working

**Symptoms**: Build takes full 5+ minutes every time

**Cause:** Cache key not matching

**Solution:**

```yaml
# Ensure cache key uses Cargo.lock hash
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

# Check if Cargo.lock is committed
git ls-files | grep Cargo.lock

# If missing, commit it
git add Cargo.lock
git commit -m "Add Cargo.lock for CI caching"
```

### Issue: Docker build fails in CI

**Symptoms**: Docker builds locally but not in GitHub Actions

**Common causes:**

**1. Missing .dockerignore:**

```bash
# Ensure .dockerignore exists and excludes target/
cat .dockerignore
```

**2. Out of disk space:**

```yaml
# Add cleanup step before Docker build
- name: Clean up disk space
  run: |
    docker system prune -af
    rm -rf target
```

**3. Multi-platform build issues:**

```yaml
# Temporarily disable arm64 for debugging
platforms: linux/amd64
# platforms: linux/amd64,linux/arm64
```

### Issue: Release binaries too large

**Symptoms**: Release binaries are 50MB+ each

**Solutions:**

```bash
# 1. Ensure strip is running (Linux/macOS)
strip target/release/actix-web-template

# 2. Enable LTO in Cargo.toml
[profile.release]
lto = true
codegen-units = 1
strip = true

# 3. Use UPX compression (optional)
upx --best target/release/actix-web-template
```

## Performance Optimization

### Speeding Up CI Builds

**Current timings (before optimization):**

| Workflow | Time | Cost (minutes) |
|----------|------|----------------|
| Test | 5:30 | 5.5 |
| Lint | 3:00 | 3.0 |
| Release | 25:00 | 25.0 |
| **Total** | **33:30** | **33.5** |

**Optimization strategies:**

**1. Aggressive caching**

```yaml
# Cache more directories
- name: Cache
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
      ~/.rustup  # Add rustup cache
```

**Result:** Test workflow 5:30 → 2:00 (63% faster)

**2. Sparse registry protocol**

```yaml
# Add to workflow env
env:
  CARGO_REGISTRIES_CRATES_IO_PROTOCOL: sparse
```

**Result:** Dependency download 2x faster

**3. Incremental compilation**

```yaml
env:
  CARGO_INCREMENTAL: 1
```

**Result:** Rebuilds 30-50% faster

**4. Parallel test execution**

```yaml
# Run integration tests in parallel
- name: Run tests
  run: cargo test --release --workspace
```

**Result:** Tests 2x faster

**Optimized timings:**

| Workflow | Before | After | Improvement |
|----------|--------|-------|-------------|
| Test | 5:30 | 2:00 | 63% faster |
| Lint | 3:00 | 1:30 | 50% faster |
| Release | 25:00 | 18:00 | 28% faster |
| **Total** | **33:30** | **21:30** | **36% faster** |

**Cost savings:** 12 minutes per full pipeline run

## Summary

In this chapter, you learned:

### GitHub Actions Concepts

- **Workflows, jobs, and steps**: Building blocks of CI/CD
- **Event triggers**: Automate on push, PR, tags, schedule
- **Runners**: GitHub-hosted VMs for executing workflows
- **Service containers**: Run databases alongside tests
- **Matrix builds**: Test on multiple OS/versions in parallel
- **Caching**: Speed up builds by reusing dependencies

### Quality Automation

- **Automated testing**: Run full test suite on every push
- **Code formatting**: Enforce consistent style with `rustfmt`
- **Linting**: Catch bugs and anti-patterns with `clippy`
- **Security audits**: Detect vulnerable dependencies
- **License compliance**: Ensure legal dependency usage
- **Documentation checks**: Verify docs build without errors

### Release Automation

- **Multi-platform builds**: 5 binaries for different OS/architectures
- **GitHub Releases**: Auto-generate release notes
- **Docker publishing**: Build and push images to GHCR
- **Semantic versioning**: Smart tagging (major, minor, patch, latest)
- **Artifact management**: Download binaries from releases

### Deployment Strategies

- **Environment protection**: Require approvals for production
- **Manual triggers**: Deploy on-demand from GitHub UI
- **Multiple strategies**: SSH, Kubernetes, cloud platforms
- **Rollback support**: Automatic rollback on failure

### Key Takeaways

1. **CI/CD catches bugs early**: Failed tests block merges
2. **Automation saves time**: No manual builds or deployments
3. **Consistency across environments**: Same checks everywhere
4. **Fast feedback**: Developers know within minutes if code works
5. **Security built-in**: Automated vulnerability scanning
6. **Release confidence**: Comprehensive testing before production

## Next Steps

In the next chapter, you'll:

- **Chapter 17: Observability Stack**: Add distributed tracing, metrics, and monitoring
  - OpenTelemetry integration
  - Jaeger for distributed tracing
  - Prometheus for metrics collection
  - Grafana for visualization dashboards
  - Request tracking and performance monitoring
  - Production debugging and performance analysis

### Optional Exercises

1. **Add code coverage reporting**: Use `cargo-tarpaulin` and Codecov
2. **Implement preview deployments**: Auto-deploy PRs to preview environments
3. **Add Dependabot**: Automated dependency updates
4. **Set up scheduled workflows**: Nightly builds or weekly audits
5. **Add performance benchmarks**: Track performance over time with `criterion`

### Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions Examples](https://github.com/actions-rs)
- [cargo-audit](https://github.com/rustsec/rustsec)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- [GitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
- [GitHub Environments](https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment)

---

**Congratulations!** Your application now has a complete CI/CD pipeline with automated testing, quality checks, security audits, and release management. Every code change is automatically validated, and releases are just one `git tag` command away.
