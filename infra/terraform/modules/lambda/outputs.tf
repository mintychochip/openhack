output "function_arn" {
  description = "ARN of the Lambda function."
  value       = aws_lambda_function.this.arn
}

output "function_name" {
  description = "Name of the Lambda function."
  value       = aws_lambda_function.this.function_name
}

output "invoke_arn" {
  description = "Invoke ARN of the Lambda function (for API Gateway integration)."
  value       = aws_lambda_function.this.invoke_arn
}

output "execution_role_arn" {
  description = "ARN of the IAM execution role attached to the Lambda."
  value       = aws_iam_role.execution.arn
}
