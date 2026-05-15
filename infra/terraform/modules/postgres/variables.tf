variable "environment" {
  description = "Deployment environment: dev, staging, prod"
  type        = string
}

variable "project_name" {
  description = "Project name used as resource name prefix"
  type        = string
  default     = "openhack"
}

variable "serverless" {
  description = "Whether to use Aurora Serverless v2 (true) or provisioned RDS instance (false)"
  type        = bool
  default     = false
}

variable "engine_version" {
  description = "PostgreSQL engine version"
  type        = string
  default     = "16"
}

variable "instance_class" {
  description = "RDS instance class when serverless=false. db.t4g.micro for dev, db.r6g.large for prod"
  type        = string
  default     = "db.t4g.micro"
}

variable "serverless_min_capacity" {
  description = "Aurora Serverless v2 minimum ACU. 0.5 allows the cluster to pause when idle"
  type        = number
  default     = 0.5
}

variable "serverless_max_capacity" {
  description = "Aurora Serverless v2 maximum ACU"
  type        = number
  default     = 2
}

variable "allocated_storage" {
  description = "Allocated storage in GB for provisioned RDS instance (GP3)"
  type        = number
  default     = 20
}

variable "database_name" {
  description = "Name of the PostgreSQL database to create"
  type        = string
  default     = "openhack"
}

variable "username" {
  description = "PostgreSQL master username"
  type        = string
  default     = "openhack"
}

variable "password" {
  description = "PostgreSQL master password"
  type        = string
  sensitive   = true
}

variable "subnet_ids" {
  description = "List of isolated subnet IDs for the DB subnet group"
  type        = list(string)
}

variable "allowed_security_group_id" {
  description = "Security group ID allowed to connect to PostgreSQL on port 5432"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID where the security group is created"
  type        = string
}

variable "backup_retention_period" {
  description = "Backup retention period in days. 7 for prod, 1 for dev"
  type        = number
  default     = 7
}

variable "skip_final_snapshot" {
  description = "Whether to skip a final snapshot on destroy. true for dev, false for prod"
  type        = bool
  default     = false
}

variable "create_extension" {
  description = "Whether to auto-create the pgvector extension via null_resource. Requires psql on the operator machine or a Lambda"
  type        = bool
  default     = true
}

variable "tags" {
  description = "Additional tags to apply to all resources"
  type        = map(string)
  default     = {}
}
