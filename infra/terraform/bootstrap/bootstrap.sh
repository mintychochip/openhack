#!/usr/bin/env bash
set -euo pipefail

# OpenHack Terraform Backend Bootstrap
# Creates the S3 bucket and DynamoDB table required for remote state.
# Run this ONCE per AWS account before running `terraform init`.

BUCKET_NAME="${TF_BACKEND_BUCKET:-openhack-terraform-state}"
DYNAMODB_TABLE="${TF_BACKEND_TABLE:-openhack-terraform-lock}"
REGION="${AWS_REGION:-us-east-1}"

echo "Creating S3 bucket: ${BUCKET_NAME}"
aws s3api create-bucket \
  --bucket "${BUCKET_NAME}" \
  --region "${REGION}" \
  --create-bucket-configuration LocationConstraint="${REGION}" 2>/dev/null || \
  echo "Bucket already exists, skipping."

echo "Enabling versioning on S3 bucket..."
aws s3api put-bucket-versioning \
  --bucket "${BUCKET_NAME}" \
  --versioning-configuration Status=Enabled

echo "Enabling server-side encryption on S3 bucket..."
aws s3api put-bucket-encryption \
  --bucket "${BUCKET_NAME}" \
  --server-side-encryption-configuration '{"Rules":[{"ApplyServerSideEncryptionByDefault":{"SSEAlgorithm":"AES256"}}]}'

echo "Blocking public access on S3 bucket..."
aws s3api put-public-access-block \
  --bucket "${BUCKET_NAME}" \
  --public-access-block-configuration BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true

echo "Creating DynamoDB table: ${DYNAMODB_TABLE}"
aws dynamodb create-table \
  --table-name "${DYNAMODB_TABLE}" \
  --attribute-definitions AttributeName=LockID,AttributeType=S \
  --key-schema AttributeName=LockID,KeyType=HASH \
  --billing-mode PAY_PER_REQUEST \
  --region "${REGION}" 2>/dev/null || \
  echo "Table already exists, skipping."

echo ""
echo "Backend resources created successfully."
echo "Now run: cd infra/terraform && terraform init"
