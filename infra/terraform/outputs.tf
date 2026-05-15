output "api_gateway_url" {
  description = "Base URL for the HTTP API Gateway"
  value       = module.api_gateway.api_endpoint
}

output "rds_endpoint" {
  description = "PostgreSQL connection endpoint"
  value       = module.postgres.endpoint
  sensitive   = true
}

output "s3_media_bucket" {
  description = "S3 bucket name for media uploads"
  value       = module.s3.bucket_name
}

output "sns_event_topic_arns" {
  description = "SNS topic ARNs for inter-service events"
  value       = { for k, m in module.sns : k => m.topic_arn }
}

output "gateway_lambda_arn" {
  description = "ARN of the gateway Lambda function"
  value       = module.gateway_lambda.function_arn
}
