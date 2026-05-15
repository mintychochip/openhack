output "api_endpoint" {
  description = "Default invoke URL for the HTTP API (https://{api_id}.execute-api.{region}.amazonaws.com)."
  value       = aws_apigatewayv2_api.this.api_endpoint
}

output "api_id" {
  description = "ID of the HTTP API Gateway."
  value       = aws_apigatewayv2_api.this.id
}

output "stage_name" {
  description = "Name of the deployed stage ($default)."
  value       = aws_apigatewayv2_stage.default.name
}
