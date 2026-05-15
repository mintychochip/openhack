output "event_bus_arn" {
  value       = aws_cloudwatch_event_bus.this.arn
  description = "ARN of the EventBridge event bus."
}

output "event_bus_name" {
  value       = aws_cloudwatch_event_bus.this.name
  description = "Name of the EventBridge event bus."
}

output "rule_arns" {
  value       = { for r in aws_cloudwatch_event_rule.this : r.name => r.arn }
  description = "Map of rule name to rule ARN."
}
