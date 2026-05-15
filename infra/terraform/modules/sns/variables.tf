variable "name" {
  type        = string
  description = "Name of the SNS topic."
}

variable "display_name" {
  type        = string
  default     = ""
  description = "Display name for the SNS topic."
}

variable "lambda_subscriptions" {
  type        = list(string)
  default     = []
  description = "List of Lambda function ARNs to subscribe to the topic."
}

variable "sqs_subscriptions" {
  type        = list(string)
  default     = []
  description = "List of SQS queue ARNs to subscribe to the topic."
}

variable "https_subscriptions" {
  type        = list(string)
  default     = []
  description = "List of HTTPS endpoint URLs to subscribe to the topic."
}

variable "allowed_publisher_arns" {
  type        = list(string)
  default     = []
  description = "List of AWS principal ARNs allowed to publish to the topic. Empty list allows all."
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Tags applied to all resources."
}
