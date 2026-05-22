# OpenHack Infrastructure (Terraform)

AWS infrastructure for the OpenHack serverless stack.

## Architecture

- **VPC**: Private subnets for Lambda, public subnets for API Gateway
- **Aurora Serverless v2**: PostgreSQL with pgvector extension
- **ElastiCache Redis**: Optional, for event caching (not required — services degrade gracefully)
- **S3**: Media file storage
- **Lambda**: 12 service functions (auth, core, gateway, judging, leaderboard, mail, notify, ai, analytics, sponsors, media, discord-bot)
- **API Gateway**: HTTP API routing to Lambda functions
- **ECR**: Container image registry for Lambda

## Quick Start

### 1. Bootstrap Backend (first time only)

```bash
bash infra/terraform/bootstrap/bootstrap.sh
```

### 2. Initialize

```bash
cd infra/terraform
terraform init
```

### 3. Set Required Variables

These variables have no defaults and must be provided:

```bash
export TF_VAR_db_password="your-secure-password"
export TF_VAR_jwt_secret="your-jwt-secret-at-least-32-characters"
export TF_VAR_lambda_internal_token="your-internal-token"
```

### 4. Deploy

```bash
terraform workspace new dev    # or: terraform workspace select dev
terraform apply -var-file=envs/dev.tfvars
```

## Environments

| Environment | tfvars | State Key |
|-------------|--------|-----------|
| Dev | `envs/dev.tfvars` | `openhack-dev.tfstate` |
| Staging | `envs/staging.tfvars` | `openhack-staging.tfstate` |
| Production | `envs/prod.tfvars` | `openhack-prod.tfstate` |

## Optional Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `redis_url` | `""` | Redis URL — services work without it |
| `domain_name` | `""` | Custom domain for API Gateway |
| `openai_api_key` | `""` | Required only for AI service |
| `discord_webhook_url` | `""` | Required only for Discord notifications |
