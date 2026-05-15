variable "function_name" {
  description = "Name of the Lambda function. Must be unique within the AWS account and region."
  type        = string
}

variable "image_uri" {
  description = "ECR container image URI for the Lambda function (e.g., 123456789.dkr.ecr.us-east-1.amazonaws.com/repo:tag)."
  type        = string
}

variable "environment_variables" {
  description = "Map of environment variables to inject into the Lambda runtime. Keys and values must be strings."
  type        = map(string)
  default     = {}
}

variable "subnet_ids" {
  description = "List of VPC subnet IDs to attach the Lambda to. Leave empty for non-VPC (internet-accessible) Lambda."
  type        = list(string)
  default     = []
}

variable "security_group_ids" {
  description = "List of VPC security group IDs for the Lambda. Required if subnet_ids is non-empty."
  type        = list(string)
  default     = []
}

variable "provisioned_concurrency" {
  description = "Number of provisioned concurrent executions. 0 = on-demand (no provisioned concurrency). Requires publish = true on the function."
  type        = number
  default     = 0
}

variable "reserved_concurrency" {
  description = "Maximum simultaneous executions for this function. -1 = unlimited (AWS default). 0 = block all invocations."
  type        = number
  default     = -1
}

variable "memory_size" {
  description = "Memory allocated to the Lambda function in MB (128–10240, in 1 MB increments)."
  type        = number
  default     = 256
}

variable "timeout" {
  description = "Maximum execution time in seconds (1–900)."
  type        = number
  default     = 30
}

variable "secrets_manager_arns" {
  description = "List of Secrets Manager secret ARNs the execution role may read (secretsmanager:GetSecretValue). Empty list = no Secrets Manager access."
  type        = list(string)
  default     = []
}

variable "policy_json" {
  description = "Additional IAM policy document (JSON) to attach to the execution role. Use for SQS, SNS, DynamoDB, or other service permissions. Empty string = no additional policy."
  type        = string
  default     = ""
}

variable "create_dlq" {
  description = "Whether to create a Dead Letter SQS queue and attach it to the Lambda. Failed async invocations are sent to the DLQ."
  type        = bool
  default     = false
}

variable "tags" {
  description = "Tags applied to all resources created by this module."
  type        = map(string)
  default     = {}
}
