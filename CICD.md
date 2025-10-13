# CI/CD Pipeline Documentation

This document describes the continuous integration and continuous deployment (CI/CD) pipeline for the Actix Web Memos application.

## Table of Contents

- [Overview](#overview)
- [Workflows](#workflows)
- [Setup Instructions](#setup-instructions)
- [Workflow Triggers](#workflow-triggers)
- [Secrets Configuration](#secrets-configuration)
- [Badge URLs](#badge-urls)
- [Troubleshooting](#troubleshooting)

## Overview

The CI/CD pipeline consists of four main GitHub Actions workflows:

1. **Test** - Runs automated tests on multiple platforms
2. **Lint** - Checks code quality and style
3. **Release** - Builds release binaries and Docker images
4. **Deploy** - Deploys to staging and production environments

All workflows are defined in `.github/workflows/` directory.

## Workflows

### 1. Test Workflow (.github/workflows/test.yml)

**Purpose**: Ensures code quality through automated testing.

**Triggers**:
- Push to `master` or `stage-*` branches
- Pull requests to `master`

**Jobs**:

#### test
- Runs full test suite with PostgreSQL service
- Executes on Ubuntu latest
- Includes:
  - Setup and run PostgreSQL 16
  - Run database migrations
  - Execute all tests with code and doc tests
  - Cache cargo dependencies for faster builds

#### test-matrix
- Tests build across multiple operating systems
- Matrix strategy: `ubuntu-latest`, `macos-latest`, `windows-latest`
- Runs unit tests only (no database required)
- Verifies cross-platform compatibility

**Key Features**:
- PostgreSQL service container for integration tests
- Dependency caching for faster CI runs
- Parallel execution across multiple OS platforms
- Comprehensive test coverage (unit, integration, doc tests)

**Environment Variables**:
```yaml
CARGO_TERM_COLOR: always
RUST_BACKTRACE: 1
DATABASE_URL: postgres://testuser:testpass@localhost:5432/testdb
```

### 2. Lint Workflow (.github/workflows/lint.yml)

**Purpose**: Enforces code quality standards and security practices.

**Triggers**:
- Push to `master` or `stage-*` branches
- Pull requests to `master`

**Jobs**:

#### fmt
- Checks Rust code formatting with `cargo fmt`
- Ensures consistent code style across the project

#### clippy
- Runs Clippy linter with `-D warnings` (treats warnings as errors)
- Catches common mistakes and suggests idiomatic Rust patterns

#### audit
- Performs security audit with `cargo-audit`
- Checks for known vulnerabilities in dependencies

#### deny
- Runs `cargo-deny` for license and security compliance
- Enforces dependency policy

#### doc
- Verifies documentation builds without errors
- Checks private items documentation with `--document-private-items`

**Key Features**:
- Multiple quality gates ensure high code standards
- Security vulnerability scanning
- Documentation validation
- Dependency caching for performance

### 3. Release Workflow (.github/workflows/release.yml)

**Purpose**: Builds and distributes release artifacts.

**Triggers**:
- Tags matching `v*` (e.g., v1.0.0)
- Tags matching `stage-*-complete`
- Manual workflow dispatch

**Jobs**:

#### build
- Builds release binaries for multiple targets
- Matrix strategy includes:
  - Linux (GNU and musl): x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl
  - macOS: x86_64-apple-darwin, aarch64-apple-darwin (Apple Silicon)
  - Windows: x86_64-pc-windows-msvc
- Strips binaries to reduce size
- Uploads artifacts for each platform

#### create-release
- Creates GitHub Release with all platform binaries
- Generates release notes automatically
- Marks as prerelease for alpha/beta/rc tags

#### docker
- Builds and pushes multi-platform Docker images (linux/amd64, linux/arm64)
- Publishes to GitHub Container Registry (ghcr.io)
- Tags images with:
  - Semantic version tags (v1, v1.0, v1.0.0)
  - SHA-based tags
  - `latest` for default branch
- Uses BuildKit caching for efficient builds

#### docker-compose-test
- Tests docker-compose configuration
- Verifies service startup and health checks
- Ensures deployment readiness

**Key Features**:
- Cross-platform binary distribution
- Automated GitHub Releases
- Multi-architecture Docker images
- Build caching for performance
- Deployment validation

**Docker Image Tags**:
- `ghcr.io/YOUR_USERNAME/actix-web-template:latest`
- `ghcr.io/YOUR_USERNAME/actix-web-template:v1.0.0`
- `ghcr.io/YOUR_USERNAME/actix-web-template:v1.0`
- `ghcr.io/YOUR_USERNAME/actix-web-template:v1`

### 4. Deploy Workflow (.github/workflows/deploy.yml)

**Purpose**: Deploys application to staging and production environments.

**Triggers**:
- Manual workflow dispatch with environment selection
- Automatic on version tags (production)
- Automatic on beta tags (staging)

**Jobs**:

#### deploy-staging
- Deploys to staging environment
- Runs smoke tests after deployment
- Notifies team of deployment status

#### deploy-production
- Deploys to production environment
- Requires manual approval (configured via GitHub Environments)
- Runs health checks after deployment
- Notifies team of deployment status

#### rollback
- Automatic rollback on production deployment failure
- Restores previous working version

**Key Features**:
- Environment-specific deployments
- Manual approval gates for production
- Automated smoke testing
- Rollback capability
- Deployment notifications

**Note**: This is a template workflow. You need to configure actual deployment steps based on your infrastructure (Docker, Kubernetes, SSH, cloud providers, etc.).

## Setup Instructions

### 1. Initial Setup

1. **Fork/Clone the repository**:
```bash
git clone https://github.com/YOUR_USERNAME/actix-web-template.git
cd actix-web-template
```

2. **Update README badges**:
   - Replace `YOUR_USERNAME` in the badge URLs in `README.md`
   - Line 3-5 in README.md

3. **Enable GitHub Actions**:
   - Go to repository Settings > Actions > General
   - Allow all actions and reusable workflows

### 2. GitHub Container Registry Setup

1. **Enable GitHub Packages**:
   - Go to repository Settings > Packages
   - Ensure packages are enabled for the repository

2. **Make package public** (optional):
   - After first Docker image push, go to Packages
   - Click on your package
   - Package settings > Change visibility > Public

### 3. GitHub Environments Setup

1. **Create environments**:
   - Go to Settings > Environments
   - Click "New environment"
   - Create `staging` and `production` environments

2. **Configure production protection**:
   - Select `production` environment
   - Enable "Required reviewers"
   - Add team members as reviewers
   - Optional: Set deployment branch restrictions

3. **Add environment secrets** (if needed):
   - `DEPLOY_SSH_KEY` - SSH private key for deployments
   - `DEPLOY_HOST` - Deployment server hostname
   - `DEPLOY_USER` - Deployment user
   - Cloud provider credentials (AWS, Azure, GCP)

### 4. Configure Secrets

Go to Settings > Secrets and variables > Actions:

**Repository Secrets** (optional, depending on deployment method):
- `DEPLOY_SSH_KEY` - SSH private key for server access
- `DEPLOY_HOST` - Server hostname
- `DEPLOY_USER` - SSH username
- `SLACK_WEBHOOK_URL` - For Slack notifications
- Cloud provider credentials

**Note**: `GITHUB_TOKEN` is automatically provided and has permissions for:
- Writing packages (Docker images)
- Creating releases
- Reading repository contents

### 5. cargo-deny Configuration

Create `.github/cargo-deny.toml` (optional):
```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
notice = "warn"

[licenses]
unlicensed = "deny"
copyleft = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
]

[bans]
multiple-versions = "warn"
wildcards = "warn"
```

## Workflow Triggers

### Automatic Triggers

| Workflow | Push to master | Push to stage-* | PR to master | Tags | Manual |
|----------|---------------|----------------|--------------|------|--------|
| Test     | Yes           | Yes            | Yes          | No   | No     |
| Lint     | Yes           | Yes            | Yes          | No   | No     |
| Release  | No            | No             | No           | Yes  | Yes    |
| Deploy   | No            | No             | No           | Yes  | Yes    |

### Tag-Based Releases

**Version tags** (e.g., `v1.0.0`):
```bash
git tag v1.0.0
git push origin v1.0.0
```
- Triggers: Release workflow (build binaries + Docker images)
- Triggers: Deploy workflow to production

**Stage completion tags** (e.g., `stage-19-complete`):
```bash
git tag stage-19-complete
git push origin stage-19-complete
```
- Triggers: Release workflow (build binaries only)

**Pre-release tags** (alpha, beta, rc):
```bash
git tag v1.0.0-beta.1
git push origin v1.0.0-beta.1
```
- Triggers: Release workflow
- Triggers: Deploy workflow to staging
- Marked as pre-release on GitHub

### Manual Triggers

**Release workflow**:
1. Go to Actions > Release
2. Click "Run workflow"
3. Select branch
4. Click "Run workflow"

**Deploy workflow**:
1. Go to Actions > Deploy
2. Click "Run workflow"
3. Select branch
4. Choose environment (staging/production)
5. Click "Run workflow"

## Secrets Configuration

### Required for Docker Publishing

No additional secrets required. Uses `GITHUB_TOKEN` automatically.

### Optional for Deployments

#### SSH-based Deployment

```bash
# Generate SSH key (if needed)
ssh-keygen -t ed25519 -C "github-actions" -f deploy_key

# Add public key to server's ~/.ssh/authorized_keys
cat deploy_key.pub

# Add private key to GitHub Secrets as DEPLOY_SSH_KEY
cat deploy_key
```

Then add these secrets:
- `DEPLOY_SSH_KEY` - Private SSH key
- `DEPLOY_HOST` - example.com
- `DEPLOY_USER` - deployuser

#### Kubernetes Deployment

Add these secrets:
- `KUBE_CONFIG` - Base64-encoded kubeconfig file
- `KUBE_NAMESPACE` - Kubernetes namespace

#### AWS Deployment

Add these secrets:
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_REGION`

## Badge URLs

Update these URLs in your README.md:

```markdown
[![Test](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/test.yml)
[![Lint](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/lint.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/lint.yml)
[![Release](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/release.yml/badge.svg)](https://github.com/YOUR_USERNAME/actix-web-template/actions/workflows/release.yml)
```

Replace `YOUR_USERNAME` with your actual GitHub username.

## Troubleshooting

### Common Issues

#### 1. Tests Failing in CI but Passing Locally

**Symptoms**: Tests pass with `cargo test` locally but fail in GitHub Actions.

**Solutions**:
- Check database connection string in workflow
- Verify PostgreSQL service is running
- Ensure migrations are executed before tests
- Check environment variable configuration

**Debug**:
```bash
# Run tests with same env as CI
DATABASE_URL=postgres://testuser:testpass@localhost:5432/testdb cargo test
```

#### 2. Docker Build Fails

**Symptoms**: `docker` job fails with build errors.

**Solutions**:
- Test Docker build locally: `docker build -t test .`
- Check Dockerfile for syntax errors
- Verify all dependencies are available
- Check .dockerignore file

#### 3. Cargo Deny Failures

**Symptoms**: `deny` job fails due to license or advisory issues.

**Solutions**:
- Review the failed check output
- Update dependencies: `cargo update`
- For advisories: `cargo audit fix`
- For licenses: review `.github/cargo-deny.toml`

#### 4. Permission Denied - GHCR Push

**Symptoms**: Cannot push Docker image to ghcr.io.

**Solutions**:
- Verify packages permission in workflow
- Check repository package settings
- Ensure `GITHUB_TOKEN` has write permissions

**Fix workflow permissions**:
```yaml
permissions:
  contents: read
  packages: write
```

#### 5. Matrix Build Failures on Windows/macOS

**Symptoms**: Linux builds pass but Windows/macOS fail.

**Solutions**:
- Check for platform-specific dependencies
- Review path separators (use `std::path::Path`)
- Test locally with cross-compilation:
  ```bash
  rustup target add x86_64-pc-windows-msvc
  cargo build --target x86_64-pc-windows-msvc
  ```

#### 6. Release Tag Not Triggering Workflow

**Symptoms**: Pushed tag but workflow doesn't run.

**Solutions**:
- Verify tag format matches trigger pattern
- Check that workflow file is on default branch
- Ensure Actions are enabled in repository settings

**Verify tag**:
```bash
git tag -l
git show v1.0.0
```

### Viewing Workflow Logs

1. Go to repository's Actions tab
2. Click on the workflow run
3. Click on the job name
4. Expand the step to view logs

### Re-running Failed Workflows

1. Go to the failed workflow run
2. Click "Re-run jobs" button
3. Select "Re-run failed jobs" or "Re-run all jobs"

### Local Testing

Test workflows locally with [act](https://github.com/nektos/act):

```bash
# Install act
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run test workflow
act -j test

# Run lint workflow
act -j lint

# Run specific job
act -j clippy
```

**Note**: `act` has limitations and may not perfectly replicate GitHub Actions environment.

## Best Practices

### Workflow Optimization

1. **Use caching aggressively**:
   - Cache cargo registry
   - Cache cargo git dependencies
   - Cache build artifacts
   - Cache Docker layers

2. **Parallelize independent jobs**:
   - Run tests and lints concurrently
   - Use matrix strategy for multi-platform builds

3. **Fail fast**:
   - Run quick checks (fmt, clippy) before expensive tests
   - Use `fail-fast: false` in matrix when you want all results

4. **Minimize workflow duration**:
   - Use smaller runner images when possible
   - Share artifacts between jobs instead of rebuilding
   - Skip redundant work (e.g., don't run tests on docs-only changes)

### Security

1. **Protect secrets**:
   - Use environment-specific secrets
   - Don't log secrets (GitHub automatically masks them)
   - Rotate secrets regularly

2. **Review dependencies**:
   - Run security audits regularly
   - Keep dependencies updated
   - Use Dependabot for automatic updates

3. **Require reviews**:
   - Set up branch protection rules
   - Require PR reviews before merging
   - Require status checks to pass

### Deployment

1. **Use environments**:
   - Separate staging and production
   - Require approvals for production
   - Set deployment branch restrictions

2. **Test before deploying**:
   - Run smoke tests in staging
   - Verify health checks pass
   - Monitor metrics after deployment

3. **Have rollback plan**:
   - Keep previous version artifacts
   - Automate rollback process
   - Document manual rollback steps

## Next Steps

1. **Configure Deployment**:
   - Set up actual deployment scripts in `.github/workflows/deploy.yml`
   - Configure target infrastructure (servers, Kubernetes, cloud)
   - Add deployment notifications

2. **Add Code Coverage**:
   - Integrate [codecov.io](https://codecov.io/)
   - Add coverage badge to README
   - Set coverage thresholds

3. **Set up Dependabot**:
   - Create `.github/dependabot.yml`
   - Enable automatic dependency updates
   - Configure update frequency

4. **Add Performance Benchmarks**:
   - Create benchmark workflow
   - Track performance over time
   - Alert on performance regressions

5. **Implement Deployment Strategies**:
   - Blue-green deployments
   - Canary releases
   - Rolling updates

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Workflow Syntax](https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions)
- [GitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
- [Rust CI/CD Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)

---

**Last Updated**: Stage 19 - CI/CD Pipeline Implementation