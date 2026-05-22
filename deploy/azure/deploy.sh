#!/usr/bin/env bash
set -euo pipefail

# OpenHack Azure Container Apps Deployment
# Deploys all services to Azure Container Apps using pre-built Docker images.

RESOURCE_GROUP="${AZURE_RESOURCE_GROUP:?AZURE_RESOURCE_GROUP is required}"
LOCATION="${AZURE_LOCATION:-eastus}"
ENVIRONMENT="${AZURE_CONTAINER_ENV:-openhack-env}"
REGISTRY="${AZURE_REGISTRY:-ghcr.io/mintychochip/openhack}"
IMAGE_TAG="${ECR_IMAGE_TAG:-latest}"

SERVICES=(
  auth-svc:3001
  core-svc:3002
  gateway-svc:8000
  judging-svc:3003
  leaderboard-svc:3004
  mail-svc:3005
  notify-svc:3006
  ai-svc:3007
  analytics-svc:3008
  sponsors-svc:3009
  media-svc:3010
  discord-bot-svc:3011
)

for entry in "${SERVICES[@]}"; do
  svc="${entry%%:*}"
  port="${entry##*:}"
  echo "Deploying ${svc} to Container Apps..."
  az containerapp create \
    --name "${svc}" \
    --resource-group "${RESOURCE_GROUP}" \
    --environment "${ENVIRONMENT}" \
    --image "${REGISTRY}/${svc}:${IMAGE_TAG}" \
    --target-port "${port}" \
    --ingress external \
    --env-vars "DATABASE_URL=${DATABASE_URL}" "REDIS_URL=${REDIS_URL:-}" "JWT_SECRET=${JWT_SECRET}" "RUST_LOG=info" \
    --cpu 0.25 \
    --memory 0.5Gi \
    --min-replicas 1 \
    --max-replicas 10 \
    --query properties.configuration.ingress.fqdn -o tsv 2>/dev/null || true
done

echo "All services deployed to Azure Container Apps."
