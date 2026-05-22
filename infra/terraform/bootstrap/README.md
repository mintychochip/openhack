# Terraform Backend Bootstrap

This directory creates the AWS resources required for Terraform remote state.

## Resources Created

- **S3 Bucket** (`openhack-terraform-state`): Stores Terraform state files with versioning and encryption
- **DynamoDB Table** (`openhack-terraform-lock`): Provides state locking to prevent concurrent applies

## Usage

```bash
# Set AWS credentials (profile, or env vars)
export AWS_PROFILE=your-profile

# Run bootstrap (once per AWS account)
bash infra/terraform/bootstrap/bootstrap.sh

# Then initialize Terraform
cd infra/terraform
terraform init
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TF_BACKEND_BUCKET` | `openhack-terraform-state` | S3 bucket name |
| `TF_BACKEND_TABLE` | `openhack-terraform-lock` | DynamoDB table name |
| `AWS_REGION` | `us-east-1` | AWS region |

## Required Variables for `terraform apply`

These variables have no defaults and must be provided:

```bash
export TF_VAR_db_password="your-database-password"
export TF_VAR_jwt_secret="your-jwt-secret-at-least-32-chars"
export TF_VAR_lambda_internal_token="your-lambda-internal-token"
```

Or pass via `-var-file`:
```bash
terraform apply -var-file=envs/dev.tfvars \
  -var="db_password=your-password" \
  -var="jwt_secret=your-secret" \
  -var="lambda_internal_token=your-token"
```
