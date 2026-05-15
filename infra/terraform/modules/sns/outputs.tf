output "topic_arn" {
  value       = aws_sns_topic.this.arn
  description = "ARN of the SNS topic."
}

output "topic_name" {
  value       = aws_sns_topic.this.name
  description = "Name of the SNS topic."
}
