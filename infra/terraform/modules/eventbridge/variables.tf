variable "event_bus_name" {
  type        = string
  description = "Name of the EventBridge event bus."
}

variable "rules" {
  type = list(object({
    name               = string
    description        = string
    schedule            = optional(string)
    event_pattern       = optional(string)
    enabled             = optional(bool, true)
  }))
  default     = []
  description = "EventBridge rules to create. Each rule must have a name and description. Provide either schedule or event_pattern."
}

variable "targets" {
  type = list(object({
    rule_name = string
    id        = string
    arn       = string
    role_arn  = optional(string)
    input     = optional(string)
  }))
  default     = []
  description = "EventBridge targets. Each target binds to a rule by name and specifies the Lambda ARN to invoke."
}

variable "lambda_permissions" {
  type = list(object({
    rule_name  = string
    lambda_arn = string
  }))
  default     = []
  description = "Lambda permissions for EventBridge to invoke. Must be set for each Lambda target."
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Tags applied to all resources created by this module."
}
