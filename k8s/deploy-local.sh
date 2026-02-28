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
