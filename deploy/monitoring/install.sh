#!/bin/bash
# Monitoring Stack Installation Script
# Optional - installs Prometheus, Grafana, and Alertmanager

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NAMESPACE="${NAMESPACE:-openhack}"
RELEASE_NAME="${RELEASE_NAME:-openhack-monitoring}"

echo "🔍 OpenHack Monitoring Stack Installer"
echo "======================================"
echo ""

# Check if Helm is installed
if ! command -v helm &> /dev/null; then
    echo "❌ Helm is not installed. Please install Helm first."
    exit 1
fi

# Check if kubectl is configured
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ kubectl is not configured. Please configure kubectl first."
    exit 1
fi

# Prompt for confirmation
echo "This will install the following in the '$NAMESPACE' namespace:"
echo "  - Prometheus (metrics collection)"
echo "  - Grafana (dashboards)"
echo "  - Alertmanager (alerting)"
echo "  - Node Exporter (infrastructure metrics)"
echo "  - Kube-state-metrics (K8s metrics)"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Installation cancelled."
    exit 0
fi

# Create namespace if it doesn't exist
echo "📦 Creating namespace..."
kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -

# Add Prometheus community Helm repo
echo "📦 Adding Prometheus community Helm repo..."
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# Check if values file exists
VALUES_FILE="${SCRIPT_DIR}/values.yaml"
if [ ! -f "$VALUES_FILE" ]; then
    echo "❌ Values file not found: $VALUES_FILE"
    exit 1
fi

# Ask if user wants to edit values
echo ""
echo "📝 Default configuration has monitoring DISABLED."
read -p "Would you like to edit values.yaml before installing? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    ${EDITOR:-nano} "$VALUES_FILE"
fi

# Install Prometheus stack
echo "📦 Installing Prometheus stack..."
helm upgrade --install "$RELEASE_NAME" prometheus-community/kube-prometheus-stack \
  --namespace "$NAMESPACE" \
  --values "$VALUES_FILE" \
  --timeout 10m \
  --wait

# Apply ServiceMonitors for OpenHack services
echo "📦 Applying ServiceMonitors..."
kubectl apply -f "${SCRIPT_DIR}/prometheus/servicemonitors.yaml" --namespace "$NAMESPACE"

# Apply alert rules
echo "📦 Applying alert rules..."
kubectl apply -f "${SCRIPT_DIR}/prometheus/rules.yaml" --namespace "$NAMESPACE"

# Get Grafana admin password
echo ""
echo "🔑 Retrieving Grafana admin password..."
GRAFANA_PASSWORD=$(kubectl get secret --namespace "$NAMESPACE" "$RELEASE_NAME-grafana" -o jsonpath="{.data.admin-password}" | base64 --decode)

# Get Grafana URL
echo "🌐 Getting Grafana URL..."
if kubectl get ingress --namespace "$NAMESPACE" "$RELEASE_NAME-grafana" &> /dev/null; then
    GRAFANA_URL=$(kubectl get ingress --namespace "$NAMESPACE" "$RELEASE_NAME-grafana" -o jsonpath="{.spec.rules[0].host}")
    echo ""
    echo "✅ Monitoring stack installed successfully!"
    echo ""
    echo "📊 Grafana Dashboard:"
    echo "   URL: https://$GRAFANA_URL"
    echo "   Username: admin"
    echo "   Password: $GRAFANA_PASSWORD"
else
    echo ""
    echo "✅ Monitoring stack installed successfully!"
    echo ""
    echo "📊 Access Grafana:"
    echo "   kubectl port-forward svc/$RELEASE_NAME-grafana 3000:80 --namespace $NAMESPACE"
    echo "   URL: http://localhost:3000"
    echo "   Username: admin"
    echo "   Password: $GRAFANA_PASSWORD"
fi

echo ""
echo "📈 Prometheus:"
echo "   kubectl port-forward svc/$RELEASE_NAME-prometheus 9090:80 --namespace $NAMESPACE"
echo "   URL: http://localhost:9090"
echo ""
echo "🔔 Alertmanager:"
echo "   kubectl port-forward svc/$RELEASE_NAME-alertmanager 9093:80 --namespace $NAMESPACE"
echo "   URL: http://localhost:9093"
echo ""
echo "📚 Next steps:"
echo "   1. Import dashboards from deploy/monitoring/grafana/"
echo "   2. Configure alert receivers in Alertmanager"
echo "   3. Verify metrics are being collected"
echo ""
