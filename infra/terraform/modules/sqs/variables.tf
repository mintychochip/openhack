variable "name" {
  type        = string
  description = "Name of the SQS queue."
}

variable "fifo" {
  type        = bool
  default     = false
  description = "Whether the queue is FIFO."
}

variable "dlq_max_receive_count" {
  type        = number
  default     = 5
  description = "Number of times a message is received before being moved to the DLQ."
}

variable "visibility_timeout_seconds" {
  type        = number
  default     = 30
  description = "Visibility timeout in seconds."
}

variable "message_retention_seconds" {
  type        = number
  default     = 345600
  description = "Message retention period in seconds. Default is 4 days."
}

variable "lambda_arn" {
  type        = string
  default     = ""
  description = "ARN of a Lambda function to trigger from this queue. Empty string disables event source mapping."
}

variable "batch_size" {
  type        = number
  default     = 10
  description = "Number of messages per batch for the Lambda event source mapping."
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Tags applied to all resources."
}
