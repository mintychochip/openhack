variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "jwt_secret" {
  description = "JWT signing secret (leave empty to auto-generate)"
  type        = string
  default     = ""
  sensitive   = true
}

variable "database_password" {
  description = "PostgreSQL database password (leave empty to auto-generate)"
  type        = string
  default     = ""
  sensitive   = true
}

variable "redis_password" {
  description = "Redis password (leave empty to auto-generate)"
  type        = string
  default     = ""
  sensitive   = true
}

variable "smtp_password" {
  description = "SMTP password for mail service"
  type        = string
  default     = ""
  sensitive   = true
}

variable "minio_root_password" {
  description = "MinIO root password (leave empty to auto-generate)"
  type        = string
  default     = ""
  sensitive   = true
}
