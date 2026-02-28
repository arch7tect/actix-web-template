# Chapter 21: First Kubernetes Deployment on minikube

## Overview

In this chapter, we deploy our hardened application (from Chapter 20) onto a local Kubernetes cluster using minikube. You will create Kubernetes manifests for every component: a Namespace, ConfigMap, Secret, PostgreSQL StatefulSet, migration Job, application Deployment, and a NodePort Service to access the app from your browser.

By the end of this chapter, two replicas of the memos application will be running on Kubernetes with health probes, connected to a PostgreSQL database with persistent storage, and accessible through a stable URL.

> **Note**: This chapter creates only Kubernetes YAML manifests and a deployment script. No Rust code changes are needed -- we did all the application hardening in Chapter 20.

## Prerequisites

### Completed Chapters

- **Chapter 20: Kubernetes Readiness** (Required)
  - Proxy-aware rate limiting, separate migration binary, graceful shutdown, configurable settings

- **Chapter 15: Docker Deployment** (Required)
  - Working Dockerfile and understanding of container builds

### Required Knowledge

- Basic Docker concepts (images, containers)
- Command-line comfort with `kubectl`
- Understanding of environment variables and YAML

### Required Software

- **minikube**: Local Kubernetes cluster ([install guide](https://minikube.sigs.k8s.io/docs/start/))
- **kubectl**: Kubernetes CLI ([install guide](https://kubernetes.io/docs/tasks/tools/))
- **Docker**: Container runtime (already installed from Chapter 15)

## Learning Objectives

By completing this chapter, you will:

1. Understand the Kubernetes object model (Pod, Deployment, Service, ConfigMap, Secret, Job, StatefulSet)
2. Deploy PostgreSQL as a StatefulSet with persistent storage
3. Run database migrations as a Kubernetes Job
4. Deploy the application with liveness and readiness probes
5. Access the application through a NodePort Service
6. Use a deployment script to automate the full workflow

## Concepts Covered

### Kubernetes Object Model

Kubernetes manages your application as a set of declarative objects. You describe the desired state in YAML files, and Kubernetes continuously works to make the actual state match. Here are the objects we will use:

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
│  │  │ Pod 1  │  │ Pod 2  │  <- liveness/ready  │    │
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
│  │  │ Pod    │ <- PersistentVolumeClaim         │    │
│  │  └────┬───┘                                 │    │
│  └───────┼─────────────────────────────────────┘    │
│  ┌───────▼─────────────────────────────────────┐    │
│  │  Service (ClusterIP, postgres:5432)         │    │
│  └─────────────────────────────────────────────┘    │
│                                                     │
│  ┌─────────────────────────────────────────────┐    │
│  │  Job (migration) -- runs once before deploy │    │
│  └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

| Object | Purpose |
|--------|---------|
| **Namespace** | Isolates all memos resources into their own scope |
| **ConfigMap** | Stores non-sensitive configuration as key-value pairs |
| **Secret** | Stores sensitive data (passwords, connection strings) base64-encoded |
| **StatefulSet** | Runs PostgreSQL with stable network identity and persistent storage |
| **PersistentVolumeClaim** | Requests durable storage that survives pod restarts |
| **Service (ClusterIP)** | Internal-only DNS name for pod-to-pod communication |
| **Service (NodePort)** | Exposes a port on every cluster node for external access |
| **Job** | Runs a task to completion (migrations), then stops |
| **Deployment** | Manages stateless application replicas with rolling updates |

### Why StatefulSet for PostgreSQL

A Deployment can run multiple replicas and replace pods freely. This is ideal for stateless applications like our memos API. But databases need:

- **Stable storage**: Data must survive pod restarts. A PersistentVolumeClaim (PVC) provides this.
- **Stable network identity**: The pod name stays consistent (`postgres-0`), so other services can rely on a predictable DNS name.
- **Ordered startup/shutdown**: StatefulSets create and terminate pods one at a time, in order.

> **Production note**: Running PostgreSQL inside Kubernetes is fine for development and small deployments. For production, consider a managed database service (AWS RDS, Cloud SQL, Azure Database) or a Kubernetes operator like CloudNativePG.

### Secrets Are Not Encrypted

Kubernetes Secrets are base64-encoded, not encrypted. Anyone with `kubectl` access to the namespace can read them:

```bash
kubectl get secret memos-secret -n memos -o jsonpath='{.data.DATABASE_URL}' | base64 -d
```

For production, consider:
- **Sealed Secrets** (Bitnami) -- encrypts secrets that can be safely committed to Git
- **External Secrets Operator** -- syncs secrets from AWS Secrets Manager, HashiCorp Vault, etc.
- **RBAC** -- restrict who can read secrets in each namespace

For this tutorial, base64-encoded secrets are sufficient.

---

## 21.1 Setting Up minikube

Start a local Kubernetes cluster:

```bash
# Start minikube with enough resources
minikube start --cpus=2 --memory=4096

# Verify the cluster is running
kubectl cluster-info

# Enable the metrics-server addon (needed for HPA in Chapter 22)
minikube addons enable metrics-server
```

Build the Docker image directly inside minikube's container runtime (so Kubernetes can find it without a registry):

```bash
# Build the image inside minikube
minikube image build -t memos-app:latest .
```

This is equivalent to running `docker build` inside the minikube VM. The image is immediately available to Kubernetes without pushing to a registry. We set `imagePullPolicy: Never` in our manifests to ensure Kubernetes uses this local image.

---

## 21.2 Namespace

All memos resources live in a dedicated namespace to avoid collisions with other applications:

```yaml
# k8s/namespace.yaml

apiVersion: v1
kind: Namespace
metadata:
  name: memos
  labels:
    app.kubernetes.io/part-of: memos
```

Apply it:

```bash
kubectl apply -f k8s/namespace.yaml
# namespace/memos created
```

From now on, every `kubectl` command uses `-n memos` to target this namespace.

---

## 21.3 ConfigMap and Secret

### ConfigMap: Non-Sensitive Configuration

The ConfigMap holds environment variables that are safe to commit to version control:

```yaml
# k8s/configmap.yaml

apiVersion: v1
kind: ConfigMap
metadata:
  name: memos-config
  namespace: memos
  labels:
    app.kubernetes.io/part-of: memos
data:
  SERVER_HOST: "0.0.0.0"
  SERVER_PORT: "3737"
  RUST_LOG: "info"
  LOG_FORMAT: "json"
  APP_ENV: "production"
  TRUST_PROXY: "true"
  HSTS_ENABLED: "false"
  FRAME_OPTIONS: "DENY"
  DATABASE_MAX_CONNECTIONS: "10"
  DATABASE_CONNECT_TIMEOUT: "30"
  DATABASE_MIN_CONNECTIONS: "2"
  DATABASE_IDLE_TIMEOUT: "300"
  DATABASE_MAX_LIFETIME: "1800"
  CORS_ALLOWED_ORIGINS: "*"
  MAX_REQUEST_SIZE: "262144"
  ENABLE_SWAGGER: "true"
```

A few values differ from the `.env.example` defaults:

| Key | Local default | Kubernetes value | Why |
|-----|--------------|-----------------|-----|
| `SERVER_HOST` | `127.0.0.1` | `0.0.0.0` | Must bind all interfaces inside a container |
| `LOG_FORMAT` | `pretty` | `json` | Structured logs for log aggregation (Loki, CloudWatch, etc.) |
| `TRUST_PROXY` | `false` | `true` | App runs behind the Kubernetes Service/Ingress |
| `APP_ENV` | `development` | `production` | Production behavior (stricter CORS validation, etc.) |

### Secret: Sensitive Configuration

Create the secret file from the provided example:

```bash
cp k8s/secret.yaml.example k8s/secret.yaml
```

The example file contains:

```yaml
# k8s/secret.yaml.example

apiVersion: v1
kind: Secret
metadata:
  name: memos-secret
  namespace: memos
  labels:
    app.kubernetes.io/part-of: memos
type: Opaque
data:
  # echo -n 'postgresql://postgres:postgres@postgres.memos.svc.cluster.local:5432/memos_db' | base64
  DATABASE_URL: cG9zdGdyZXNxbDovL3Bvc3RncmVzOnBvc3RncmVzQHBvc3RncmVzLm1lbW9zLnN2Yy5jbHVzdGVyLmxvY2FsOjU0MzIvbWVtb3NfZGI=
  # echo -n 'postgres' | base64
  POSTGRES_PASSWORD: cG9zdGdyZXM=
```

The `DATABASE_URL` references `postgres.memos.svc.cluster.local` -- this is the Kubernetes DNS name for the PostgreSQL Service we will create in the next section. The format is `<service-name>.<namespace>.svc.cluster.local`.

To encode your own values:

```bash
echo -n 'your-connection-string' | base64
```

> **Important**: `k8s/secret.yaml` is listed in `.gitignore`. Never commit actual secrets to version control.

Apply both:

```bash
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
```

---

## 21.4 PostgreSQL StatefulSet

PostgreSQL needs three resources: a PersistentVolumeClaim for storage, a StatefulSet for the pod, and a ClusterIP Service for internal access.

### PersistentVolumeClaim

```yaml
# k8s/postgres/pvc.yaml

apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-data
  namespace: memos
  labels:
    app.kubernetes.io/part-of: memos
    app.kubernetes.io/component: database
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
```

`ReadWriteOnce` means the volume can be mounted by a single node at a time -- appropriate for a single-replica database. minikube's default storage provisioner creates a directory on the host, so `1Gi` is allocated from your local disk.

### StatefulSet

```yaml
# k8s/postgres/statefulset.yaml

apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: memos
  labels:
    app: postgres
    app.kubernetes.io/part-of: memos
    app.kubernetes.io/component: database
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
        - name: postgres
          image: postgres:16
          ports:
            - containerPort: 5432
          env:
            - name: POSTGRES_DB
              value: memos_db
            - name: POSTGRES_USER
              value: postgres
            - name: POSTGRES_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: memos-secret
                  key: POSTGRES_PASSWORD
            - name: PGDATA
              value: /var/lib/postgresql/data/pgdata
          volumeMounts:
            - name: postgres-storage
              mountPath: /var/lib/postgresql/data
              subPath: pgdata
          readinessProbe:
            exec:
              command:
                - pg_isready
                - -U
                - postgres
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            exec:
              command:
                - pg_isready
                - -U
                - postgres
            initialDelaySeconds: 30
            periodSeconds: 10
      volumes:
        - name: postgres-storage
          persistentVolumeClaim:
            claimName: postgres-data
```

Key details:

- **`serviceName: postgres`**: Links to the headless Service (below) for stable DNS
- **`POSTGRES_PASSWORD` from Secret**: The password is not stored in the manifest
- **`subPath: pgdata`**: Prevents the `lost+found` directory (created by ext4 filesystems) from conflicting with PostgreSQL's data directory initialization
- **Readiness probe**: `pg_isready` checks whether PostgreSQL is accepting connections. Kubernetes will not route traffic to the pod until this succeeds.
- **Liveness probe**: Same check but with a longer initial delay. If PostgreSQL stops responding, Kubernetes restarts the pod.

### Service

```yaml
# k8s/postgres/service.yaml

apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: memos
  labels:
    app: postgres
    app.kubernetes.io/part-of: memos
    app.kubernetes.io/component: database
spec:
  type: ClusterIP
  selector:
    app: postgres
  ports:
    - port: 5432
      targetPort: 5432
```

This creates the DNS name `postgres.memos.svc.cluster.local` (or just `postgres` within the namespace) that our `DATABASE_URL` references.

### Deploy PostgreSQL

```bash
kubectl apply -f k8s/postgres/
kubectl rollout status statefulset/postgres -n memos --timeout=120s
```

Verify:

```bash
kubectl get pods -n memos
# NAME         READY   STATUS    RESTARTS   AGE
# postgres-0   1/1     Running   0          30s
```

---

## 21.5 Migration Job

In Chapter 20, we separated migrations from the application startup. Now we run them as a Kubernetes Job -- a one-shot task that runs to completion:

```yaml
# k8s/migration-job.yaml

apiVersion: batch/v1
kind: Job
metadata:
  name: migration
  namespace: memos
  labels:
    app: migration
    app.kubernetes.io/part-of: memos
    app.kubernetes.io/component: migration
spec:
  backoffLimit: 3
  template:
    metadata:
      labels:
        app: migration
    spec:
      restartPolicy: OnFailure
      containers:
        - name: migration
          image: memos-app:latest
          imagePullPolicy: Never
          command: ["./migration"]
          envFrom:
            - configMapRef:
                name: memos-config
            - secretRef:
                name: memos-secret
```

Key details:

- **Same image, different command**: The `memos-app:latest` image contains both binaries. We override the default command (`./actix-web-template`) with `./migration`.
- **`imagePullPolicy: Never`**: Use the locally built image; don't try to pull from a registry.
- **`restartPolicy: OnFailure`**: If the migration fails (e.g., database not ready yet), Kubernetes retries the pod.
- **`backoffLimit: 3`**: Give up after 3 failed attempts.
- **`envFrom`**: Injects all keys from the ConfigMap and Secret as environment variables. The migration binary needs `DATABASE_URL` to connect.

### Run the Migration

```bash
# Delete any previous migration job (jobs are not idempotent in Kubernetes)
kubectl delete job migration -n memos --ignore-not-found

# Apply and wait for completion
kubectl apply -f k8s/migration-job.yaml
kubectl wait --for=condition=complete job/migration -n memos --timeout=120s
```

Check the logs:

```bash
kubectl logs job/migration -n memos
# Applying all pending migrations
# Applying migration 'm20250109_000001_create_memos_table'
# ...
# Migration done
```

---

## 21.6 Application Deployment

Now deploy the application with 2 replicas:

```yaml
# k8s/app/deployment.yaml

apiVersion: apps/v1
kind: Deployment
metadata:
  name: memos-app
  namespace: memos
  labels:
    app: memos-app
    app.kubernetes.io/part-of: memos
    app.kubernetes.io/component: app
spec:
  replicas: 2
  selector:
    matchLabels:
      app: memos-app
  template:
    metadata:
      labels:
        app: memos-app
    spec:
      terminationGracePeriodSeconds: 35
      containers:
        - name: memos-app
          image: memos-app:latest
          imagePullPolicy: Never
          ports:
            - containerPort: 3737
          envFrom:
            - configMapRef:
                name: memos-config
            - secretRef:
                name: memos-secret
          livenessProbe:
            httpGet:
              path: /health
              port: 3737
            initialDelaySeconds: 10
            periodSeconds: 15
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /ready
              port: 3737
            initialDelaySeconds: 5
            periodSeconds: 5
            failureThreshold: 3
```

### Health Probes

We configured two probes in Chapter 4:

- **`/health`** (liveness): Returns 200 if the application process is healthy. If this fails 3 times in a row, Kubernetes restarts the pod.
- **`/ready`** (readiness): Returns 200 if the application can serve traffic (database connected, etc.). If this fails, Kubernetes removes the pod from the Service's endpoint list -- it stops receiving traffic but is not restarted.

The distinction matters during startup: a pod may be alive but not yet ready (still connecting to the database). The readiness probe prevents traffic from reaching it prematurely.

### Graceful Shutdown and terminationGracePeriodSeconds

When Kubernetes wants to stop a pod (rolling update, scale-down, node drain):

```
1. Pod marked as Terminating
2. Pod removed from Service endpoints (no new traffic)
3. SIGTERM sent to the container
4. App handles in-flight requests (up to shutdown_timeout=30s)
5. App exits cleanly
6. If still running after terminationGracePeriodSeconds (35s), SIGKILL
```

We set `terminationGracePeriodSeconds: 35` to be slightly longer than our application's `shutdown_timeout(30)` (configured in Chapter 20). This gives the application time to finish graceful shutdown before Kubernetes force-kills it.

### Deploy the Application

```bash
kubectl apply -f k8s/app/
kubectl rollout status deployment/memos-app -n memos --timeout=120s
```

Verify both replicas are running:

```bash
kubectl get pods -n memos
# NAME                         READY   STATUS      RESTARTS   AGE
# memos-app-6d8f9b7c4d-abc12   1/1     Running     0          30s
# memos-app-6d8f9b7c4d-def34   1/1     Running     0          30s
# migration-xxxxx               0/1     Completed   0          2m
# postgres-0                    1/1     Running     0          3m
```

---

## 21.7 NodePort Service

To access the application from outside the cluster, we create a NodePort Service:

```yaml
# k8s/app/service.yaml

apiVersion: v1
kind: Service
metadata:
  name: memos-app
  namespace: memos
  labels:
    app: memos-app
    app.kubernetes.io/part-of: memos
    app.kubernetes.io/component: app
spec:
  type: NodePort
  selector:
    app: memos-app
  ports:
    - port: 3737
      targetPort: 3737
      nodePort: 30080
```

A NodePort Service opens port 30080 on every node in the cluster. Traffic arriving at `<node-ip>:30080` is forwarded to port 3737 on one of the matching pods. The Kubernetes Service load-balances across both replicas.

### Access the Application

```bash
# Get the URL (minikube tunnels to the node)
minikube service memos-app -n memos --url
# http://192.168.49.2:30080
```

Test it:

```bash
# Health check
curl http://$(minikube ip):30080/health

# API
curl http://$(minikube ip):30080/api/v1/memos

# Open in browser
minikube service memos-app -n memos
```

> **NodePort limitations**: NodePorts use a restricted port range (30000-32767), don't support hostname-based routing, and don't terminate TLS. In Chapter 22, we replace this with an Ingress controller that provides all of these features.

---

## 21.8 The deploy-local.sh Script

Deploying all resources in the right order with the right waits is tedious to do manually. The `deploy-local.sh` script automates the full workflow:

```bash
#!/usr/bin/env bash
set -euo pipefail

NAMESPACE="memos"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Memos Kubernetes Deployment (minikube) ==="
echo ""

# Verify minikube is running
if ! minikube status --format='{{.Host}}' 2>/dev/null | grep -q "Running"; then
  echo "Error: minikube is not running. Start it with:"
  echo "  minikube start --cpus=2 --memory=4096"
  exit 1
fi

# Verify secret.yaml exists
if [ ! -f "$SCRIPT_DIR/secret.yaml" ]; then
  echo "Error: k8s/secret.yaml not found."
  echo "Create it from the example:"
  echo "  cp k8s/secret.yaml.example k8s/secret.yaml"
  exit 1
fi

echo "[1/6] Building Docker image in minikube..."
minikube image build -t memos-app:latest "$(dirname "$SCRIPT_DIR")"

echo ""
echo "[2/6] Applying namespace, configmap, and secret..."
kubectl apply -f "$SCRIPT_DIR/namespace.yaml"
kubectl apply -f "$SCRIPT_DIR/configmap.yaml"
kubectl apply -f "$SCRIPT_DIR/secret.yaml"

echo ""
echo "[3/6] Deploying PostgreSQL..."
kubectl apply -f "$SCRIPT_DIR/postgres/"
echo "Waiting for PostgreSQL to be ready..."
kubectl rollout status statefulset/postgres -n "$NAMESPACE" --timeout=120s

echo ""
echo "[4/6] Running database migrations..."
kubectl delete job migration -n "$NAMESPACE" --ignore-not-found
kubectl apply -f "$SCRIPT_DIR/migration-job.yaml"
echo "Waiting for migration job to complete..."
kubectl wait --for=condition=complete job/migration -n "$NAMESPACE" --timeout=120s

echo ""
echo "[5/6] Deploying application..."
kubectl apply -f "$SCRIPT_DIR/app/"
echo "Waiting for application rollout..."
kubectl rollout status deployment/memos-app -n "$NAMESPACE" --timeout=120s

echo ""
echo "[6/6] Deployment complete!"
echo ""
echo "Access the application:"
minikube service memos-app -n "$NAMESPACE" --url
```

Walk through the script step by step:

1. **`set -euo pipefail`**: Exit on any error, treat unset variables as errors, propagate pipe failures
2. **Preflight checks**: Verify minikube is running and `secret.yaml` exists before doing any work
3. **Build image**: `minikube image build` builds the Docker image inside minikube's container runtime
4. **Base resources**: Namespace, ConfigMap, and Secret must exist before anything else
5. **PostgreSQL**: Apply all resources in `postgres/`, then wait for the StatefulSet to be ready
6. **Migration**: Delete any previous Job (Jobs can't be re-applied), create a new one, wait for completion
7. **Application**: Apply the Deployment and Service, wait for all replicas to be ready
8. **URL**: Print the access URL

### Running the Script

```bash
# Make it executable (already done if you cloned the repo)
chmod +x k8s/deploy-local.sh

# Run the full deployment
./k8s/deploy-local.sh
```

---

## 21.9 Verification and Troubleshooting

### Verify Everything Is Running

```bash
# All resources in the memos namespace
kubectl get all -n memos

# Expected output:
# NAME                             READY   STATUS      RESTARTS   AGE
# pod/memos-app-xxx-yyy            1/1     Running     0          60s
# pod/memos-app-xxx-zzz            1/1     Running     0          60s
# pod/migration-xxxxx              0/1     Completed   0          90s
# pod/postgres-0                   1/1     Running     0          2m
#
# NAME                TYPE        CLUSTER-IP     EXTERNAL-IP   PORT(S)          AGE
# service/memos-app   NodePort    10.96.x.x      <none>        3737:30080/TCP   60s
# service/postgres    ClusterIP   10.96.x.x      <none>        5432/TCP         2m
#
# NAME                        READY   UP-TO-DATE   AVAILABLE   AGE
# deployment.apps/memos-app   2/2     2            2           60s
#
# NAME                                   DESIRED   CURRENT   READY   AGE
# statefulset.apps/postgres              1         1         1       2m
#
# NAME                   COMPLETIONS   DURATION   AGE
# job.batch/migration    1/1           10s        90s
```

### Check Application Logs

```bash
# Logs from all app replicas
kubectl logs -n memos -l app=memos-app --tail=50

# Logs from a specific pod
kubectl logs -n memos memos-app-xxx-yyy

# Follow logs in real time
kubectl logs -n memos -l app=memos-app -f
```

### Test the API

```bash
APP_URL=$(minikube service memos-app -n memos --url)

# Health check
curl "$APP_URL/health"
# {"status":"healthy","database":"connected",...}

# Readiness check
curl "$APP_URL/ready"
# {"status":"ready"}

# Create a memo
curl -X POST "$APP_URL/api/v1/memos" \
  -H "Content-Type: application/json" \
  -d '{"title":"First k8s memo","description":"Deployed on Kubernetes!","date_to":"2026-12-31T23:59:59Z"}'

# List memos
curl "$APP_URL/api/v1/memos"
```

### Common Issues

#### 1. "ImagePullBackOff" or "ErrImageNeverPull"

The image is not available in minikube's container runtime:

```bash
# Verify the image exists
minikube image ls | grep memos-app

# If not found, rebuild it
minikube image build -t memos-app:latest .
```

#### 2. Pod stuck in "Pending"

Usually a resource issue:

```bash
kubectl describe pod <pod-name> -n memos
# Look at the Events section for scheduling errors
```

Common causes:
- Not enough CPU/memory (increase minikube resources)
- PVC cannot be bound (check storage provisioner)

#### 3. Pod in "CrashLoopBackOff"

The container starts but crashes immediately:

```bash
kubectl logs <pod-name> -n memos --previous
# Shows logs from the crashed container
```

Common causes:
- `DATABASE_URL` is wrong (check the Secret)
- PostgreSQL is not ready yet (check postgres-0 pod status)
- Missing environment variable

#### 4. Migration Job fails

```bash
kubectl logs job/migration -n memos
kubectl describe job migration -n memos
```

Common causes:
- PostgreSQL not ready (Job started before StatefulSet was healthy)
- Wrong `DATABASE_URL` in the Secret
- Schema conflict from a previous partial migration

#### 5. "connection refused" when accessing the app

```bash
# Check if pods are ready
kubectl get pods -n memos

# Check service endpoints
kubectl get endpoints memos-app -n memos
# Should list pod IPs; empty means no ready pods
```

### Useful Debugging Commands

```bash
# Detailed pod information
kubectl describe pod <pod-name> -n memos

# Execute a command inside a pod (interactive shell)
kubectl exec -it <pod-name> -n memos -- /bin/bash

# Check connectivity from inside a pod
kubectl exec -it <pod-name> -n memos -- curl localhost:3737/health

# View events in the namespace
kubectl get events -n memos --sort-by='.lastTimestamp'

# Port-forward for direct access (bypasses NodePort)
kubectl port-forward svc/memos-app -n memos 3737:3737
```

---

## Tear Down

To remove everything and start fresh:

```bash
# Delete all resources in the namespace
kubectl delete namespace memos

# Or delete minikube entirely
minikube delete
```

Deleting the namespace removes all resources within it (pods, services, secrets, PVCs, etc.).

---

## Appendix: Running on kind Instead of minikube

Every manifest in this chapter works on [kind](https://kind.sigs.k8s.io/) (Kubernetes in Docker) with a few substitutions. kind is lighter-weight and popular in CI environments.

### Cluster Setup

```bash
# Install kind (macOS: brew install kind)

# Create cluster with port mapping
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

```bash
# Build locally, then load into kind
docker build -t memos-app:latest .
kind load docker-image memos-app:latest --name memos
```

### Differences Summary

| Concern | minikube | kind |
|---------|----------|------|
| Cluster creation | `minikube start` | `kind create cluster --config=...` |
| Image loading | `minikube image build` | `docker build` + `kind load docker-image` |
| NodePort access | `minikube service --url` | `http://localhost:30080` (via extraPortMappings) |
| Cleanup | `minikube delete` | `kind delete cluster --name memos` |

The `deploy-local.sh` script would need two changes for kind:

```bash
# Replace:
minikube image build -t memos-app:latest "$(dirname "$SCRIPT_DIR")"
# With:
docker build -t memos-app:latest "$(dirname "$SCRIPT_DIR")"
kind load docker-image memos-app:latest --name memos

# Replace:
minikube service memos-app -n "$NAMESPACE" --url
# With:
echo "http://localhost:30080"
```

---

## Summary

In this chapter, we deployed the memos application on a local Kubernetes cluster:

```
┌──────────────────────────────────────────────────────────┐
│  What We Created                                         │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  k8s/namespace.yaml      Isolated namespace for memos    │
│  k8s/configmap.yaml      Non-sensitive env vars          │
│  k8s/secret.yaml.example Template for secrets            │
│                                                          │
│  k8s/postgres/pvc.yaml        1Gi persistent storage     │
│  k8s/postgres/statefulset.yaml  PostgreSQL 16 pod        │
│  k8s/postgres/service.yaml      ClusterIP on port 5432   │
│                                                          │
│  k8s/migration-job.yaml   One-shot migration runner      │
│                                                          │
│  k8s/app/deployment.yaml  2 replicas with health probes  │
│  k8s/app/service.yaml     NodePort on 30080              │
│                                                          │
│  k8s/deploy-local.sh      Automated deployment script    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### Files Created

| File | Purpose |
|------|---------|
| `k8s/namespace.yaml` | Namespace definition |
| `k8s/configmap.yaml` | Non-sensitive environment variables |
| `k8s/secret.yaml.example` | Template for secrets (not committed) |
| `k8s/postgres/pvc.yaml` | PersistentVolumeClaim for database storage |
| `k8s/postgres/statefulset.yaml` | PostgreSQL StatefulSet |
| `k8s/postgres/service.yaml` | ClusterIP Service for PostgreSQL |
| `k8s/migration-job.yaml` | Migration Job |
| `k8s/app/deployment.yaml` | Application Deployment (2 replicas) |
| `k8s/app/service.yaml` | NodePort Service |
| `k8s/deploy-local.sh` | Deployment helper script |
| `.gitignore` | Added `k8s/secret.yaml` |

### What You Learned

1. **Kubernetes objects are declarative** -- you describe the desired state, Kubernetes makes it happen
2. **StatefulSets** provide stable identity and storage for databases
3. **Jobs** run tasks to completion, ideal for one-shot operations like migrations
4. **Health probes** (liveness and readiness) let Kubernetes manage application lifecycle automatically
5. **ConfigMaps and Secrets** separate configuration from container images
6. **NodePort Services** expose applications outside the cluster for development

---

## Next Steps

In **Chapter 22: Ingress, Horizontal Scaling, and Production Patterns**, we will:

- Replace the NodePort with an NGINX Ingress controller for hostname-based routing and TLS
- Add resource requests and limits for predictable scheduling
- Configure a HorizontalPodAutoscaler to scale replicas based on CPU usage
- Add a PodDisruptionBudget to protect availability during maintenance
- Restrict database access with a NetworkPolicy

The application is running on Kubernetes. Now let's make it production-ready.

---

## Additional Resources

### Kubernetes Fundamentals

- [Kubernetes Documentation](https://kubernetes.io/docs/home/)
- [Kubernetes Concepts: Workloads](https://kubernetes.io/docs/concepts/workloads/)
- [Kubernetes Concepts: Services](https://kubernetes.io/docs/concepts/services-networking/service/)

### minikube

- [minikube Documentation](https://minikube.sigs.k8s.io/docs/)
- [minikube Drivers](https://minikube.sigs.k8s.io/docs/drivers/)

### PostgreSQL on Kubernetes

- [CloudNativePG Operator](https://cloudnative-pg.io/)
- [PostgreSQL Docker Image](https://hub.docker.com/_/postgres)

### Kubernetes Security

- [Kubernetes Secrets](https://kubernetes.io/docs/concepts/configuration/secret/)
- [Sealed Secrets](https://sealed-secrets.netlify.app/)
- [External Secrets Operator](https://external-secrets.io/)
