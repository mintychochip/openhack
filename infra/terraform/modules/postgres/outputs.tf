output "endpoint" {
  description = "PostgreSQL connection endpoint (cluster endpoint for Aurora, instance address for RDS)"
  value       = var.serverless ? aws_rds_cluster.serverless[0].endpoint : aws_db_instance.provisioned[0].address
  sensitive   = true
}

output "port" {
  description = "PostgreSQL port"
  value       = 5432
}

output "database_name" {
  description = "Name of the PostgreSQL database"
  value       = var.database_name
}

output "secret_arn" {
  description = "ARN of the AWS Secrets Manager secret containing DB credentials (username, password, host, port, dbname)"
  value       = aws_secretsmanager_secret.db_credentials.arn
}
