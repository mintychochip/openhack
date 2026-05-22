# OpenHack Secrets in AWS Secrets Manager
# These secrets are referenced by Kubernetes External-secrets

resource "aws_secretsmanager_secret" "openhack_jwt" {
  name        = "openhack/jwt-secret"
  description = "JWT signing secret for OpenHack platform"
  
  recovery_window_in_days = 30
  
  tags = {
    Environment = var.environment
    Project     = "openhack"
  }
}

resource "aws_secretsmanager_secret_version" "openhack_jwt" {
  secret_id     = aws_secretsmanager_secret.openhack_jwt.id
  secret_string = var.jwt_secret != "" ? var.jwt_secret : random_password.jwt.result
}

resource "random_password" "jwt" {
  length  = 32
  special = true
}

resource "aws_secretsmanager_secret" "openhack_database" {
  name        = "openhack/database-password"
  description = "PostgreSQL database password for OpenHack platform"
  
  recovery_window_in_days = 30
  
  tags = {
    Environment = var.environment
    Project     = "openhack"
  }
}

resource "aws_secretsmanager_secret_version" "openhack_database" {
  secret_id     = aws_secretsmanager_secret.openhack_database.id
  secret_string = var.database_password != "" ? var.database_password : random_password.database.result
}

resource "random_password" "database" {
  length  = 32
  special = true
}

resource "aws_secretsmanager_secret" "openhack_redis" {
  name        = "openhack/redis-password"
  description = "Redis password for OpenHack platform"
  
  recovery_window_in_days = 30
  
  tags = {
    Environment = var.environment
    Project     = "openhack"
  }
}

resource "aws_secretsmanager_secret_version" "openhack_redis" {
  secret_id     = aws_secretsmanager_secret.openhack_redis.id
  secret_string = var.redis_password != "" ? var.redis_password : random_password.redis.result
}

resource "random_password" "redis" {
  length  = 32
  special = true
}

resource "aws_secretsmanager_secret" "openhack_smtp" {
  name        = "openhack/smtp-password"
  description = "SMTP password for OpenHack mail service"
  
  recovery_window_in_days = 30
  
  tags = {
    Environment = var.environment
    Project     = "openhack"
  }
}

resource "aws_secretsmanager_secret_version" "openhack_smtp" {
  secret_id     = aws_secretsmanager_secret.openhack_smtp.id
  secret_string = var.smtp_password
}

resource "aws_secretsmanager_secret" "openhack_minio" {
  name        = "openhack/minio-root-password"
  description = "MinIO root password for OpenHack media storage"
  
  recovery_window_in_days = 30
  
  tags = {
    Environment = var.environment
    Project     = "openhack"
  }
}

resource "aws_secretsmanager_secret_version" "openhack_minio" {
  secret_id     = aws_secretsmanager_secret.openhack_minio.id
  secret_string = var.minio_root_password != "" ? var.minio_root_password : random_password.minio.result
}

resource "random_password" "minio" {
  length  = 32
  special = true
}
