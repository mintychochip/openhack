variable "name" {
  type        = string
  description = "Name of the ECR repository."
}

variable "immutable_tags" {
  type        = bool
  default     = false
  description = "Whether image tags are immutable. Use true for prod, false for dev."
}

variable "scan_on_push" {
  type        = bool
  default     = true
  description = "Enable image scanning on push."
}

variable "keep_last_n" {
  type        = number
  default     = 10
  description = "Number of tagged images to keep."
}

variable "expire_untagged_days" {
  type        = number
  default     = 7
  description = "Days after which untagged images are expired."
}

variable "github_actions_role_arn" {
  type        = string
  default     = ""
  description = "ARN of the GitHub Actions OIDC role allowed to push images. Empty string disables the policy."
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Tags applied to all resources."
}
