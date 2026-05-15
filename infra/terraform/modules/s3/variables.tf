variable "name" {
  type        = string
  description = "Name of the S3 bucket. Must be globally unique."
}

variable "versioning" {
  type        = bool
  default     = true
  description = "Enable versioning on the bucket."
}

variable "expire_noncurrent_days" {
  type        = number
  default     = 30
  description = "Days after which non-current versions are expired."
}

variable "cors_rules" {
  type = list(object({
    allowed_headers = list(string)
    allowed_methods = list(string)
    allowed_origins = list(string)
    expose_headers  = list(string)
    max_age_seconds = number
  }))
  default     = []
  description = "CORS rules for direct browser uploads."
}

variable "lambda_read_write_arns" {
  type        = list(string)
  default     = []
  description = "List of Lambda role ARNs that need read/write access to the bucket."
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Tags applied to all resources."
}
