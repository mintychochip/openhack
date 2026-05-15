variable "name" {
  description = "Name of the HTTP API Gateway."
  type        = string
}

variable "lambda_integrations" {
  description = "Map of service name to Lambda ARN for routing. Each key becomes a route at /api/{key}/{proxy+}. Example: { auth = \"arn:aws:lambda:...:function:auth\", core = \"arn:aws:lambda:...:function:core\" }"
  type        = map(string)
}

variable "domain_name" {
  description = "Custom domain name for the API Gateway (e.g., api.example.com). Leave empty to skip custom domain."
  type        = string
  default     = ""
}

variable "certificate_arn" {
  description = "ACM certificate ARN for the custom domain. Required if domain_name is set."
  type        = string
  default     = ""
}

variable "cors_allow_origins" {
  description = "List of allowed origins for CORS. Use [\"*\"] for open access."
  type        = list(string)
  default     = ["*"]
}

variable "tags" {
  description = "Tags applied to all resources created by this module."
  type        = map(string)
  default     = {}
}
