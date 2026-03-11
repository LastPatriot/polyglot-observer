#!/bin/bash

# Polyglot Observer: One-Click Setup Script
# Developed for Lingo.dev Hackathon 2026

set -e

echo "🌍 Initializing Polyglot Observer Environment..."

# 1. Check Prerequisites
command -v kubectl >/dev/null 2>&1 || { echo "❌ kubectl is required. Aborting." >&2; exit 1; }
command -v helm >/dev/null 2>&1 || { echo "❌ helm is required. Aborting." >&2; exit 1; }
command -v minikube >/dev/null 2>&1 || { echo "❌ minikube is required. Aborting." >&2; exit 1; }

# 2. Start Minikube (if not running)
if ! minikube status >/dev/null 2>&1; then
    echo "🚀 Starting Minikube..."
    minikube start --driver=docker
else
    echo "✅ Minikube is already running."
fi

# 3. Install Loki Stack
echo "📦 Installing Loki & Grafana Stack..."
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update
helm upgrade --install loki-stack grafana/loki-stack 
  --set promtail.enabled=false 
  --set grafana.enabled=true

# 4. Deploy Polyglot Observer
echo "🛡 Deploying Polyglot Observer Agent..."
kubectl apply -f polyglot-observer/k8s-config.yaml
kubectl apply -f polyglot-observer/k8s-daemonset.yaml

# 5. Get Grafana Password
echo "🔑 Retrieving Grafana Admin Password..."
PASSWORD=$(kubectl get secret loki-stack-grafana -o jsonpath="{.data.admin-password}" | base64 --decode)

echo "--------------------------------------------------------"
echo "✅ SETUP COMPLETE!"
echo "--------------------------------------------------------"
echo "📊 Grafana URL: http://localhost:3000"
echo "👤 Username: admin"
echo "🔑 Password: $PASSWORD"
echo "--------------------------------------------------------"
echo "👉 Run the following to access the dashboard:"
echo "   kubectl port-forward svc/loki-stack-grafana 3000:80"
echo "--------------------------------------------------------"
echo "🚀 To test auto-discovery, run:"
echo "   kubectl create namespace test-apps"
echo "   kubectl run test-pod --namespace=test-apps --image=alpine -- /bin/sh -c "while true; do echo 'Database connection failed'; sleep 5; done""
echo "--------------------------------------------------------"
