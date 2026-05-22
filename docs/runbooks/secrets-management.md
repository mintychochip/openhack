# Secrets Management

## Overview

OpenHack uses AWS Secrets Manager with the External Secrets Operator to manage production secrets securely. This approach:

- **Eliminates secrets from git** - No plaintext secrets in repository
- **Automatic rotation** - Secrets can be rotated without redeploying
- **Audit trail** - AWS CloudTrail logs all secret access
- **Fine-grained access** - IAM policies control which pods can access which secrets

## Architecture

```
AWS Secrets Manager
        ↓
External Secrets Operator (K8s)
        ↓
Kubernetes Secrets (auto-created)
        ↓
Application Pods (via secretRef)
```

## Setup

### 1. Install External Secrets Operator

```bash
helm repo add external-secrets https://charts.external-secrets.io
helm repo update
helm install external-secrets external-secrets/external-secrets \
  -n external-secrets \
  --create-namespace \
  --set installCRDs=true
```

### 2. Configure IAM for IRSA

```bash
# Create IAM policy for Secrets Manager access
aws iam create-policy \
  --policy-name ExternalSecretsPolicy \
  --policy-document file://infra/terraform/external-secrets-policy.json

# Create service account with IRSA
eksctl create iamserviceaccount \
  --name external-secrets-sa \
  --namespace openhack \
  --cluster <cluster-name> \
  --attach-policy-arn arn:aws:iam::<account>:policy/ExternalSecretsPolicy \
  --approve
```

### 3. Create Secrets in AWS

```bash
cd infra/terraform
terraform init
terraform apply -target=module.secrets
```

This creates:
- `openhack/jwt-secret`
- `openhack/database-password`
- `openhack/redis-password`
- `openhack/smtp-password`
- `openhack/minio-root-password`

### 4. Deploy ExternalSecrets CRD

```bash
kubectl apply -f deploy/helm/openhack/templates/externalsecrets.yaml
```

### 5. Update Helm Values

In `values.yaml` or `values.prod.yaml`:

```yaml
secrets:
  useExternalSecrets: true
```

## Local Development

For local development, keep `useExternalSecrets: false` in `values-local.yaml`. Secrets will be generated automatically by Helm.

**Never commit `values-local.yaml`** - it contains development secrets.

## Secret Rotation

### Manual Rotation

```bash
# Generate new secret
NEW_SECRET=$(openssl rand -base64 32)

# Update in AWS Secrets Manager
aws secretsmanager update-secret \
  --secret-id openhack/jwt-secret \
  --secret-string "$NEW_SECRET"

# External Secrets will sync within 1 hour (refreshInterval)
# Or force sync:
kubectl delete secret openhack-secrets -n openhack
```

### Automated Rotation

Configure AWS Secrets Manager rotation Lambda for automatic rotation (recommended for production).

## Migration from Plaintext Secrets

1. Create secrets in AWS Secrets Manager using Terraform
2. Install External Secrets Operator
3. Deploy ExternalSecrets CRD
4. Update `values.yaml` to set `useExternalSecrets: true`
5. Remove plaintext secrets from git history (if needed):
   ```bash
   git filter-branch --force --index-filter \
     'git rm --cached --ignore-unmatch deploy/helm/openhack/values.yaml' \
     --prune-empty --tag-name-filter cat -- --all
   ```

## Security Best Practices

- ✅ Use IRSA (IAM Roles for Service Accounts) for pod identity
- ✅ Enable CloudTrail logging for secret access
- ✅ Set appropriate `refreshInterval` (1h recommended)
- ✅ Use secret versioning for rollback capability
- ✅ Rotate secrets every 90 days minimum
- ❌ Never commit secrets to git
- ❌ Never use same secret across environments
- ❌ Never share secrets via chat/email
