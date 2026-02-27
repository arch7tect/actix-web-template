# Kubernetes Tutorial Chapters Plan (20-23)

This document outlines four new chapters that extend the Actix Web tutorial with Kubernetes deployment. These chapters assume readers have completed chapters 0-19 (or checked out the `pre-k8s` tag).

## Code Preservation

| Tag | Points to | Purpose |
|-----|-----------|---------|
| `pre-k8s` | End of chapter 19 | Bookmark for readers who want the pre-Kubernetes codebase |
| `chapter-20-complete` | End of chapter 20 | Application hardened for multi-replica deployment |
| `chapter-21-complete` | End of chapter 21 | First working minikube deployment |
| `chapter-22-complete` | End of chapter 22 | Ingress, HPA, production patterns |
| `chapter-23-complete` | End of chapter 23 | Helm chart packaging |

---

## Chapter 20: Kubernetes Readiness — Preparing Your Application for Multi-Replica Deployment

**Tag**: `chapter-20-complete` | **Estimated time**: 90-120 min

### Learning Objectives

- Understand what breaks when you run multiple replicas behind a load balancer
- Replace in-process rate limiting with a proxy-aware strategy
- Separate database migrations from application startup
- Add graceful shutdown so in-flight requests complete before pod termination
- Externalize remaining hardcoded settings

### Chapter Outline

#### 20.1 What Changes When You Scale Horizontally

Explain the mental model shift from single-process to multi-replica. Diagram showing requests distributed across pods, each with its own in-memory state. Identify the four issues:

1. Rate limiting counts are per-process (not shared)
2. Every replica runs migrations on startup (race condition)
3. No graceful shutdown (requests dropped during rolling update)
4. Some settings are hardcoded (connection pool, security headers)

#### 20.2 Proxy-Aware Rate Limiting

**Problem**: `PeerIpKeyExtractor` sees the load balancer's IP, not the client's. All clients share one rate-limit bucket.

**Solution**: Create `ForwardedIpKeyExtractor` that reads `X-Forwarded-For` (or `X-Real-Ip` as fallback), falling back to peer IP when no proxy header exists.

> **Security note**: `X-Forwarded-For` can be spoofed by clients. This extractor must only be used when the application runs behind a trusted reverse proxy (e.g., NGINX Ingress in Kubernetes) that overwrites or sanitizes the header. The tutorial text should explain the trust boundary: in direct-to-internet deployments, stick with `PeerIpKeyExtractor`. When behind a single trusted proxy, use the *rightmost* IP in the `X-Forwarded-For` chain (the one the proxy appended), not the leftmost (which the client controls).

**Code changes**:

- `src/middleware/rate_limit.rs`:
  - Add `ForwardedIpKeyExtractor` struct implementing `KeyExtractor`
  - Extract the rightmost IP from `X-Forwarded-For` header chain (the one appended by the trusted proxy)
  - Add `TRUST_PROXY` or similar configuration flag so the extractor is only active behind a known proxy
  - Expose `create_rate_limiter()` function that returns a configured `Governor` middleware
  - Remove duplicated configuration that currently lives in both `rate_limit.rs` and `main.rs`

- `src/main.rs`:
  - Replace inline governor setup with `create_rate_limiter()` call

**Checkpoint**: Run the app with `TRUST_PROXY=true` and simulate a multi-hop forwarded chain:
```bash
# The rightmost IP (10.0.0.1, appended by the trusted proxy) should be the rate-limit key.
# The leftmost IP (spoofed-by-client) should be ignored.
curl -H "X-Forwarded-For: spoofed-by-client, 10.0.0.1" http://localhost:3737/api/v1/memos
```
Send requests exceeding the rate limit and verify that changing the rightmost IP resets the counter, while changing only the leftmost IP does not.

#### 20.3 Separating Migrations from Application Startup

**Problem**: The current `entrypoint.sh` runs `./migration` before starting `./actix-web-template` every time the container starts. If 3 replicas start simultaneously, all 3 run migrations concurrently. SeaORM migrations are idempotent but this wastes resources and can cause lock contention.

**Solution**: The repository already builds a separate `migration` binary (from the `migration/` workspace member). We leverage this by removing the `entrypoint.sh` script and instead running the app binary directly. Migrations become a separate, explicit step.

In Kubernetes (chapter 21), migrations will run as a k8s Job (using the `migration` binary) before the app Deployment rolls out.

**Code changes**:

- `Dockerfile`:
  - Remove the `entrypoint.sh` script that runs `./migration` then `./actix-web-template`
  - Default `CMD` becomes `["./actix-web-template"]` (application only, no migrations)
  - Document that `CMD ["./migration"]` can be used to run migrations separately
  - Both binaries remain in the image so the same image can serve either purpose

- `src/main.rs`:
  - Remove any migration-on-startup logic (if present)
  - The app binary now only starts the HTTP server

**Checkpoint**: Build the Docker image. `docker run <image> ./migration` runs migrations and exits. `docker run <image>` (default) starts the server without running migrations.

#### 20.4 Graceful Shutdown

**Problem**: When Kubernetes sends SIGTERM, the process exits immediately. In-flight requests get connection-reset errors.

**Solution**: Configure Actix Web's built-in shutdown timeout and flush tracing spans.

**Code changes**:

- `src/main.rs`:
  - Add `.shutdown_timeout(30)` to `HttpServer` builder
  - After `server.await?`, call `shutdown_tracing()` to log the shutdown event

- `src/observability/tracing.rs`:
  - `shutdown_tracing()` already exists in this file. Review whether the current implementation is sufficient (it logs a message; the OpenTelemetry `SdkTracerProvider` is automatically shut down when dropped since opentelemetry 0.31). If the provider is not held in a scope that drops at the right time, refactor so the provider is stored in `AppState` or a static and dropped explicitly after the server stops.

**Checkpoint**: Start the app, send a slow request (or add a sleep endpoint for testing), send SIGTERM, observe that the request completes before the process exits.

#### 20.5 Configurable Security Headers and Pool Settings

**Problem**: HSTS and frame-options are hardcoded. Connection pool parameters (min_connections, idle_timeout, max_lifetime) are hardcoded in `main.rs`.

**Solution**: Move these to `Settings`.

**Code changes**:

- `src/config/settings.rs`:
  - Add `SecurityConfig` struct: `hsts_enabled: bool`, `frame_options: String`
  - Add pool fields: `database_min_connections`, `database_idle_timeout_secs`, `database_max_lifetime_secs`
  - Provide sensible defaults via `Default` impl

- `src/middleware/security_headers.rs`:
  - Accept `SecurityConfig` (or the relevant fields) and conditionally set HSTS and frame-options

- `src/main.rs`:
  - Read pool settings from `Settings` instead of hardcoding
  - Pass security config to middleware

- `.env.example`:
  - Add new environment variables with comments

**Checkpoint**: Set `HSTS_ENABLED=false` and verify the HSTS header is absent. Change `FRAME_OPTIONS=SAMEORIGIN` and verify.

#### 20.6 Chapter Summary and What's Next

Recap all changes. The application is now ready for multi-replica deployment:
- Rate limiting works behind a proxy
- Migrations are a separate concern
- Shutdown is graceful
- All operational settings are externalized

### Files Modified (Chapter 20)

| File | Change |
|------|--------|
| `src/middleware/rate_limit.rs` | Add `ForwardedIpKeyExtractor`, `create_rate_limiter()` |
| `src/middleware/security_headers.rs` | Read config from `Settings` |
| `src/config/settings.rs` | Add `SecurityConfig`, pool fields |
| `src/main.rs` | `shutdown_timeout(30)`, use `create_rate_limiter()`, wire settings, remove migration-on-startup |
| `src/observability/tracing.rs` | Review `shutdown_tracing()`, ensure provider drops cleanly |
| `Dockerfile` | Remove `entrypoint.sh`, default CMD to `./actix-web-template`, document `./migration` CMD |
| `.env.example` | New environment variables |

---

## Chapter 21: First Kubernetes Deployment on minikube

**Tag**: `chapter-21-complete` | **Estimated time**: 2-3 hours

### Learning Objectives

- Understand Kubernetes object model (Pod, Deployment, Service, ConfigMap, Secret)
- Deploy PostgreSQL as a StatefulSet with persistent storage
- Run database migrations as a Kubernetes Job
- Deploy the application with health probes
- Access the application through a NodePort Service

### Chapter Outline

#### 21.1 Kubernetes Concepts

Brief, practical introduction to the objects we will use. ASCII diagram:

```
┌─────────────────────────────────────────────────────┐
│  Kubernetes Cluster (minikube)                      │
│                                                     │
│  ┌─────────────┐    ┌──────────────────────────┐    │
│  │  ConfigMap   │    │  Secret                  │    │
│  │  (env vars)  │    │  (DB password, app keys) │    │
│  └──────┬──────┘    └────────┬─────────────────┘    │
│         │                    │                       │
│  ┌──────▼────────────────────▼─────────────────┐    │
│  │  Deployment (app, 2 replicas)               │    │
│  │  ┌────────┐  ┌────────┐                     │    │
│  │  │ Pod 1  │  │ Pod 2  │  ← liveness/ready   │    │
│  │  └────┬───┘  └────┬───┘                     │    │
│  └───────┼───────────┼─────────────────────────┘    │
│          │           │                               │
│  ┌───────▼───────────▼─────────────────────────┐    │
│  │  Service (NodePort :30080)                  │    │
│  └─────────────────────────────────────────────┘    │
│                                                     │
│  ┌─────────────────────────────────────────────┐    │
│  │  StatefulSet (postgres, 1 replica)          │    │
│  │  ┌────────┐                                 │    │
│  │  │ Pod    │ ← PersistentVolumeClaim         │    │
│  │  └────┬───┘                                 │    │
│  └───────┼─────────────────────────────────────┘    │
│  ┌───────▼─────────────────────────────────────┐    │
│  │  Service (ClusterIP, postgres:5432)         │    │
│  └─────────────────────────────────────────────┘    │
│                                                     │
│  ┌─────────────────────────────────────────────┐    │
│  │  Job (migration) — runs once before deploy  │    │
│  └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

#### 21.2 Prerequisites and minikube Setup

- Install minikube and kubectl
- `minikube start --cpus=2 --memory=4096`
- `minikube addons enable metrics-server` (needed for HPA in chapter 22)
- Build the Docker image into minikube's registry: `minikube image build -t memos-app:latest .`

#### 21.3 Namespace, ConfigMap, and Secret

- Create `memos` namespace
- ConfigMap for non-sensitive config (SERVER_HOST, SERVER_PORT, RUST_LOG, etc.)
- Secret for DATABASE_URL (base64-encoded)
- Explain why Secrets are only base64-encoded, not encrypted (and mention Sealed Secrets as a production option)

#### 21.4 PostgreSQL StatefulSet

- Why StatefulSet instead of Deployment for databases
- PersistentVolumeClaim for data durability
- ClusterIP Service for internal access
- Readiness probe using `pg_isready`

#### 21.5 Migration Job

- Kubernetes Job runs `./migration` (the separate migration binary, same Docker image with overridden command)
- `restartPolicy: OnFailure` with `backoffLimit: 3`
- Job must complete before app Deployment (enforced by `deploy-local.sh` script)

#### 21.6 Application Deployment

- 2 replicas
- `envFrom` referencing ConfigMap and Secret
- Liveness probe: `httpGet /health` every 15s, `failureThreshold: 3`
- Readiness probe: `httpGet /ready` every 5s, `failureThreshold: 3`
- `terminationGracePeriodSeconds: 35` (slightly more than the 30s shutdown_timeout)

#### 21.7 NodePort Service

- Expose the app on a NodePort (30080)
- `minikube service memos-app -n memos` to get the URL
- Verify with `curl`

#### 21.8 The deploy-local.sh Script

A helper script that automates the deployment sequence:

```bash
#!/usr/bin/env bash
set -euo pipefail

NAMESPACE="memos"

echo "Building Docker image in minikube..."
minikube image build -t memos-app:latest .

echo "Applying base resources..."
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml  # user must create from .example

echo "Deploying PostgreSQL..."
kubectl apply -f k8s/postgres/
kubectl rollout status statefulset/postgres -n "$NAMESPACE" --timeout=120s

echo "Running migrations..."
kubectl delete job migration -n "$NAMESPACE" --ignore-not-found
kubectl apply -f k8s/migration-job.yaml
kubectl wait --for=condition=complete job/migration -n "$NAMESPACE" --timeout=120s

echo "Deploying application..."
kubectl apply -f k8s/app/
kubectl rollout status deployment/memos-app -n "$NAMESPACE" --timeout=120s

echo "Done! Access the app:"
minikube service memos-app -n "$NAMESPACE" --url
```

Each line is explained in the tutorial text.

#### 21.9 Verification and Troubleshooting

- `kubectl get pods -n memos` — all Running
- `kubectl logs -n memos deployment/memos-app` — check for startup messages
- `kubectl describe pod <name>` — check events if a pod won't start
- Common issues: image not found, secret missing, postgres not ready

### Files Created (Chapter 21)

```
k8s/
├── namespace.yaml              # Namespace definition
├── configmap.yaml              # Non-sensitive environment variables
├── secret.yaml.example         # Template for secrets (not committed)
├── postgres/
│   ├── statefulset.yaml        # PostgreSQL StatefulSet
│   ├── service.yaml            # ClusterIP Service for PostgreSQL
│   └── pvc.yaml                # PersistentVolumeClaim
├── app/
│   ├── deployment.yaml         # Application Deployment (2 replicas)
│   └── service.yaml            # NodePort Service
├── migration-job.yaml          # Migration Job
└── deploy-local.sh             # Deployment helper script (chmod +x)
```

### Tutorial file

- `tutorial/chapter-21.md`

---

## Chapter 22: Ingress, Horizontal Scaling, and Production Patterns

**Tag**: `chapter-22-complete` | **Estimated time**: 2-3 hours

### Learning Objectives

- Route external traffic through an NGINX Ingress controller
- Auto-scale the application based on CPU usage
- Protect availability during cluster maintenance with PodDisruptionBudgets
- Restrict network access to the database
- Connect the CI/CD pipeline to Kubernetes

### Chapter Outline

#### 22.1 Why Ingress

- NodePort limitations (port range 30000-32767, no hostname routing, no TLS)
- Ingress as the Kubernetes-native reverse proxy
- Install NGINX Ingress: `minikube addons enable ingress`

#### 22.2 Ingress Resource

- Route `memos.local` to the app Service
- TLS termination with a self-signed cert (for learning; mention cert-manager for production)
- Switch app Service from NodePort to ClusterIP
- Update `/etc/hosts` to point `memos.local` to minikube IP

#### 22.3 Resource Requests and Limits

- Explain requests vs limits and how they affect scheduling
- Add to app Deployment:
  ```yaml
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 500m
      memory: 256Mi
  ```
- Explain why we set requests < limits (burstable QoS)

#### 22.4 HorizontalPodAutoscaler

- Target 60% average CPU utilization
- Min 2, max 5 replicas
- `kubectl autoscale` vs declarative YAML
- Load test with `hey` or `wrk` and watch scaling:
  ```bash
  kubectl get hpa -n memos --watch
  ```

#### 22.5 PodDisruptionBudget

- What happens during `kubectl drain` (node maintenance)
- PDB ensures at least 1 pod is always available:
  ```yaml
  minAvailable: 1
  ```
- Demonstrate: drain a node and see the PDB block eviction until safe

#### 22.6 NetworkPolicy

- Restrict PostgreSQL access to only pods with label `app: memos-app`
- Default deny all ingress to postgres namespace
- Allow ingress from app pods and migration jobs

#### 22.7 CI/CD Integration

- Update `.github/workflows/deploy.yml`:
  - Build and push Docker image to a registry
  - `kubectl set image` or `kubectl apply` with new image tag
  - Wait for rollout to complete
  - Rollback on failure
- Explain the placeholder that existed since chapter 16

#### 22.8 Verification

- Access app through `https://memos.local`
- Trigger autoscale with load test
- Verify NetworkPolicy blocks unauthorized access

### Files Created/Modified (Chapter 22)

| File | Change |
|------|--------|
| `k8s/ingress.yaml` | New — NGINX Ingress resource |
| `k8s/app/hpa.yaml` | New — HorizontalPodAutoscaler |
| `k8s/app/pdb.yaml` | New — PodDisruptionBudget |
| `k8s/postgres/network-policy.yaml` | New — NetworkPolicy |
| `k8s/app/deployment.yaml` | Add resource requests/limits |
| `k8s/app/service.yaml` | Change from NodePort to ClusterIP |
| `.github/workflows/deploy.yml` | Replace placeholder with real kubectl steps |

### Tutorial file

- `tutorial/chapter-22.md`

---

## Chapter 23: Helm Chart — Parameterized Packaging

**Tag**: `chapter-23-complete`, `v0.3.0` | **Estimated time**: 2-3 hours

### Learning Objectives

- Understand Helm's architecture (chart, values, release, revision)
- Convert plain Kubernetes manifests into a reusable Helm chart
- Use `values.yaml` for defaults and environment-specific overrides
- Perform rolling upgrades and rollbacks with Helm

### Chapter Outline

#### 23.1 Why Helm

- Problem: duplicated YAML across environments (dev, staging, prod)
- Helm as a package manager: chart = package, release = installed instance
- When Helm adds value vs when plain manifests are enough
- Install Helm CLI

#### 23.2 Chart Structure

- `helm create charts/memos` as starting point, then strip defaults
- Explain each file:
  - `Chart.yaml` — metadata (name, version, appVersion)
  - `values.yaml` — default configuration values
  - `templates/` — Kubernetes manifests with Go template syntax
  - `_helpers.tpl` — shared template functions
  - `NOTES.txt` — post-install instructions

#### 23.3 Converting Manifests to Templates

Walk through converting each `k8s/` manifest:

- Replace hardcoded values with `{{ .Values.x }}` references
- Use `_helpers.tpl` for:
  - `memos.fullname` — consistent resource naming
  - `memos.labels` — standard labels (app, version, managed-by)
  - `memos.selectorLabels` — selector subset

Example transformation (deployment.yaml):
```yaml
# Before (plain manifest)
replicas: 2
image: memos-app:latest

# After (Helm template)
replicas: {{ .Values.replicaCount }}
image: "{{ .Values.image.repository }}:{{ .Values.image.tag | default .Chart.AppVersion }}"
```

#### 23.4 Values Files

- `values.yaml` — development defaults:
  ```yaml
  replicaCount: 2
  image:
    repository: memos-app
    tag: latest
  service:
    type: ClusterIP
    port: 3737
  ingress:
    enabled: false
  postgres:
    enabled: true
  ```

- `values-prod.yaml` — production overrides:
  ```yaml
  replicaCount: 3
  image:
    repository: registry.example.com/memos-app
  ingress:
    enabled: true
    host: memos.example.com
    tls: true
  resources:
    requests:
      cpu: 200m
      memory: 256Mi
    limits:
      cpu: 1000m
      memory: 512Mi
  ```

#### 23.5 Install, Upgrade, Rollback

- First install: `helm install memos charts/memos -n memos`
- Upgrade with new image: `helm upgrade memos charts/memos --set image.tag=v2`
- Production install: `helm install memos charts/memos -f charts/memos/values-prod.yaml`
- Check history: `helm history memos -n memos`
- Rollback: `helm rollback memos 1 -n memos`
- Uninstall: `helm uninstall memos -n memos`

#### 23.6 Template Debugging

- `helm template memos charts/memos` — render templates locally
- `helm install --dry-run --debug` — server-side validation
- `helm lint charts/memos` — check for common issues

#### 23.7 Chapter Summary

- Plain `k8s/` directory is retained as a learning reference
- Helm chart in `charts/memos/` is the recommended deployment method going forward
- For teams using GitOps (ArgoCD, Flux), Helm charts integrate naturally

### Files Created (Chapter 23)

```
charts/memos/
├── Chart.yaml                  # Chart metadata
├── values.yaml                 # Default values (development)
├── values-prod.yaml            # Production overrides
├── .helmignore                 # Files to ignore when packaging
└── templates/
    ├── _helpers.tpl            # Shared template helpers
    ├── namespace.yaml          # Namespace
    ├── configmap.yaml          # ConfigMap
    ├── secret.yaml             # Secret
    ├── deployment.yaml         # Application Deployment
    ├── service.yaml            # Application Service
    ├── ingress.yaml            # Ingress (conditional)
    ├── hpa.yaml                # HorizontalPodAutoscaler (conditional)
    ├── pdb.yaml                # PodDisruptionBudget
    ├── migration-job.yaml      # Migration Job (pre-install hook)
    ├── postgres-statefulset.yaml  # PostgreSQL StatefulSet
    ├── postgres-service.yaml   # PostgreSQL Service
    └── NOTES.txt               # Post-install instructions
```

### Tutorial file

- `tutorial/chapter-23.md`

---

## Appendix: Running on kind Instead of minikube

Chapters 21-23 use minikube for its beginner-friendly addon system, but every manifest works on [kind](https://kind.sigs.k8s.io/) with a few substitutions. Add this as a sidebar or appendix section in chapter 21.

### Cluster Setup

```bash
# Install kind
# macOS: brew install kind  |  Linux: go install sigs.k8s.io/kind@latest

# Create cluster with port mapping (replaces minikube's NodePort tunneling)
cat <<'EOF' | kind create cluster --name memos --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  extraPortMappings:
  - containerPort: 30080
    hostPort: 30080
    protocol: TCP
EOF
```

### Loading Images

minikube has `minikube image build`. kind equivalent:

```bash
# Build locally then load into kind (no registry needed)
docker build -t memos-app:latest .
kind load docker-image memos-app:latest --name memos
```

Add `imagePullPolicy: Never` (or `IfNotPresent`) to `deployment.yaml` so Kubernetes uses the loaded image instead of pulling from a registry.

### Ingress (Chapter 22)

Replace `minikube addons enable ingress` with the kind-specific NGINX Ingress install:

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml
kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=90s
```

The Ingress YAML itself is unchanged. Access via `localhost` (kind maps ports through `extraPortMappings`), so update `/etc/hosts` to point `memos.local` to `127.0.0.1` instead of the minikube IP.

### Metrics Server (Chapter 22 HPA)

```bash
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml
# kind runs locally so the kubelet cert won't match — patch to skip TLS verification:
kubectl patch deployment metrics-server -n kube-system \
  --type=json \
  -p='[{"op":"add","path":"/spec/template/spec/containers/0/args/-","value":"--kubelet-insecure-tls"}]'
```

### deploy-local.sh Adaptation

Replace the two minikube-specific lines:

| minikube | kind |
|----------|------|
| `minikube image build -t memos-app:latest .` | `docker build -t memos-app:latest . && kind load docker-image memos-app:latest --name memos` |
| `minikube service memos-app -n "$NAMESPACE" --url` | `echo "http://localhost:30080"` |

Everything else (`kubectl apply`, `kubectl wait`, `kubectl rollout status`) is identical.

### Summary of Differences

| Concern | minikube | kind |
|---------|----------|------|
| Cluster creation | `minikube start` | `kind create cluster --config=...` |
| Image loading | `minikube image build` | `docker build` + `kind load docker-image` |
| NodePort access | `minikube service --url` | `extraPortMappings` in cluster config |
| Ingress addon | `minikube addons enable ingress` | `kubectl apply` NGINX Ingress for kind |
| Metrics server | `minikube addons enable metrics-server` | `kubectl apply` + TLS patch |
| Cleanup | `minikube delete` | `kind delete cluster --name memos` |

---

## Implementation Order

When implementing these chapters (writing code + tutorial text):

1. **Chapter 20** first — code changes that make the app k8s-ready
2. **Chapter 21** — k8s manifests and first deployment
3. **Chapter 22** — production patterns layered on top
4. **Chapter 23** — Helm chart wrapping all manifests

Each chapter should be implemented on a branch (`chapter-20-k8s-readiness`, etc.), merged to master, and tagged.

## Estimated Total Time for Readers

| Chapter | Time |
|---------|------|
| 20 | 90-120 min |
| 21 | 2-3 hours |
| 22 | 2-3 hours |
| 23 | 2-3 hours |
| **Total** | **8-12 hours** |
