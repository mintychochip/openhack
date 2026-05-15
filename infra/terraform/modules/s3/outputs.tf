output "bucket_arn" {
  value       = aws_s3_bucket.this.arn
  description = "ARN of the S3 bucket."
}

output "bucket_name" {
  value       = aws_s3_bucket.this.id
  description = "Name of the S3 bucket."
}

output "lambda_access_policy_arn" {
  value       = try(aws_s3_bucket_policy.lambda_access[0].id, null)
  description = "ID of the bucket policy granting Lambda access, or null if none was created."
}
