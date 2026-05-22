#!/usr/bin/env bash
set -euo pipefail

# OpenHack GCP Cloud Run Deployment
# Deploys all services to Google Cloud Run using pre-built Docker images.

PROJECT_ID="${GCP_PROJECT_ID:?GCP_PROJECT_ID is required}"
REGION="${GCP_REGION:-us-central1}"
IMAGE_TAG="${ECR_IMAGE_TAG:-latest}"
REGISTRY="gcr.io/${PROJECT_ID}"

SERVICES=(
  auth-svc core-svc gateway-svc judging-svc leaderboard-svc
  mail-svc notify-svc ai-svc analytics-svc sponsors-svc
  media-svc discord-bot-svc
)

for svc in "${SERVICES[@]}"; do
  echo "Deploying ${svc} to Cloud Run..."
  gcloud run deploy "${svc}" \
    --image "${REGISTRY}/${svc}:${IMAGE_TAG}" \
    --region "${REGION}" \
    --platform managed \
    --allow-unauthenticated \
    --set-env-vars "DATABASE_URL=${DATABASE_URL},REDIS_URL=${REDIS_URL:-},JWT_SECRET=${JWT_SECRET},RUST_LOG=info" \
    --memory 256Mi \
    --cpu 1 \
    --max-instances 10 \
    --quiet
done

echo "All services deployed to Cloud Run."
echo "Gateway URL: https://gateway-svc-$(gcloud run services describe gateway-svc --region "${REGION}" --format='value(status.url)' 2>/dev/null || echo 'unknown')"
