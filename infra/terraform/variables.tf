variable "aws_region" {
  description = "AWS region for all resources"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Deployment environment: dev, staging, prod"
  type        = string
}

variable "project_name" {
  description = "Project name used as resource name prefix"
  type        = string
  default     = "openhack"
}

variable "db_username" {
  description = "PostgreSQL master username"
  type        = string
  default     = "openhack"
}

variable "db_password" {
  description = "PostgreSQL master password"
  type        = string
  sensitive   = true
}

variable "db_name" {
  description = "PostgreSQL database name"
  type        = string
  default     = "openhack"
}

variable "db_instance_class" {
  description = "RDS instance class. db.t4g.micro for dev, db.serverless for prod"
  type        = string
  default     = "db.t4g.micro"
}

variable "db_serverless_min_capacity" {
  description = "Aurora Serverless v2 minimum ACU (0.5 = pauses when idle)"
  type        = number
  default     = 0.5
}

variable "db_serverless_max_capacity" {
  description = "Aurora Serverless v2 maximum ACU"
  type        = number
  default     = 2
}

variable "jwt_secret" {
  description = "JWT signing secret shared across auth/core/sponsors/notify services"
  type        = string
  sensitive   = true
}

variable "openai_api_key" {
  description = "OpenAI API key for ai-svc"
  type        = string
  sensitive   = true
  default     = ""
}

variable "discord_webhook_url" {
  description = "Discord webhook URL for notify-svc"
  type        = string
  sensitive   = true
  default     = ""
}

variable "redis_url" {
  description = "Redis URL. Empty string disables Redis (services degrade gracefully)"
  type        = string
  default     = ""
}

variable "lambda_memory_size" {
  description = "Default Lambda memory in MB"
  type        = number
  default     = 256
}

variable "lambda_timeout" {
  description = "Default Lambda timeout in seconds"
  type        = number
  default     = 30
}

variable "ecr_image_tag" {
  description = "Docker image tag to deploy from ECR"
  type        = string
  default     = "latest"
}

variable "domain_name" {
  description = "Custom domain name for API Gateway (optional)"
  type        = string
  default     = ""
}

variable "certificate_arn" {
  description = "ACM certificate ARN for custom domain (optional)"
  type        = string
  default     = ""
}

variable "lambda_internal_token" {
  description = "Shared secret for internal Lambda-to-service authentication. Set this to a random 32-char hex string. Passed via X-Lambda-Internal-Token header to bypass JWT auth on admin trigger endpoints."
  type        = string
  sensitive   = true
}

variable "vpc_cidr" {
  description = "CIDR block for the VPC. Must be large enough to allocate at least 6 /20 subnets."
  type        = string
  default     = "10.0.0.0/16"
}
