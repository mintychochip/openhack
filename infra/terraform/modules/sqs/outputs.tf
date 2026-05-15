output "queue_arn" {
  value       = aws_sqs_queue.main.arn
  description = "ARN of the main SQS queue."
}

output "queue_url" {
  value       = aws_sqs_queue.main.id
  description = "URL of the main SQS queue."
}

output "dlq_arn" {
  value       = aws_sqs_queue.dlq.arn
  description = "ARN of the dead letter queue."
}
