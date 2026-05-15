data "aws_caller_identity" "current" {}

resource "aws_iam_role" "execution" {
  name = "${var.function_name}-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
        Action = "sts:AssumeRole"
      }
    ]
  })

  tags = var.tags
}

resource "aws_iam_role_policy_attachment" "basic_execution" {
  role       = aws_iam_role.execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy_attachment" "vpc_execution" {
  count      = length(var.subnet_ids) > 0 ? 1 : 0
  role       = aws_iam_role.execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

resource "aws_iam_role_policy" "secrets_manager" {
  count = length(var.secrets_manager_arns) > 0 ? 1 : 0
  name  = "secrets-manager-read"
  role  = aws_iam_role.execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["secretsmanager:GetSecretValue", "secretsmanager:DescribeSecret"]
        Resource = var.secrets_manager_arns
      }
    ]
  })
}

resource "aws_iam_role_policy" "additional" {
  count  = var.policy_json != "" ? 1 : 0
  name   = "additional-policy"
  role   = aws_iam_role.execution.id
  policy = var.policy_json
}

resource "aws_sqs_queue" "dlq" {
  count                     = var.create_dlq ? 1 : 0
  name                      = "${var.function_name}-dlq"
  message_retention_seconds = 1209600
  tags                      = var.tags
}

resource "aws_iam_role_policy" "dlq_send" {
  count = var.create_dlq ? 1 : 0
  name  = "dlq-send"
  role  = aws_iam_role.execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = "sqs:SendMessage"
        Resource = aws_sqs_queue.dlq[0].arn
      }
    ]
  })
}

resource "aws_cloudwatch_log_group" "lambda" {
  name              = "/aws/lambda/${var.function_name}"
  retention_in_days = 7
  tags              = var.tags
}

resource "aws_lambda_function" "this" {
  function_name = var.function_name
  role          = aws_iam_role.execution.arn
  image_uri     = var.image_uri
  package_type  = "Image"
  memory_size   = var.memory_size
  timeout       = var.timeout

  reserved_concurrent_executions = var.reserved_concurrency == -1 ? null : var.reserved_concurrency

  environment {
    variables = var.environment_variables
  }

  dynamic "vpc_config" {
    for_each = length(var.subnet_ids) > 0 ? [1] : []
    content {
      subnet_ids         = var.subnet_ids
      security_group_ids = var.security_group_ids
    }
  }

  dynamic "dead_letter_config" {
    for_each = var.create_dlq ? [1] : []
    content {
      target_arn = aws_sqs_queue.dlq[0].arn
    }
  }

  publish = var.provisioned_concurrency > 0

  depends_on = [aws_cloudwatch_log_group.lambda]

  tags = var.tags
}

resource "aws_lambda_provisioned_concurrency_config" "this" {
  count                             = var.provisioned_concurrency > 0 ? 1 : 0
  function_name                     = aws_lambda_function.this.function_name
  qualifier                         = aws_lambda_function.this.version
  provisioned_concurrent_executions = var.provisioned_concurrency
}
