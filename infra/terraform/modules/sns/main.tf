resource "aws_sns_topic" "this" {
  name         = var.name
  display_name = var.display_name != "" ? var.display_name : var.name
  tags         = var.tags
}

resource "aws_sns_topic_policy" "this" {
  count = length(var.allowed_publisher_arns) > 0 ? 1 : 0
  arn   = aws_sns_topic.this.arn
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { AWS = var.allowed_publisher_arns }
      Action    = "SNS:Publish"
      Resource  = aws_sns_topic.this.arn
    }]
  })
}

resource "aws_sns_topic_subscription" "lambda" {
  count     = length(var.lambda_subscriptions)
  topic_arn = aws_sns_topic.this.arn
  protocol  = "lambda"
  endpoint  = var.lambda_subscriptions[count.index]
}

resource "aws_sns_topic_subscription" "sqs" {
  count     = length(var.sqs_subscriptions)
  topic_arn = aws_sns_topic.this.arn
  protocol  = "sqs"
  endpoint  = var.sqs_subscriptions[count.index]
}

resource "aws_sns_topic_subscription" "https" {
  count     = length(var.https_subscriptions)
  topic_arn = aws_sns_topic.this.arn
  protocol  = "https"
  endpoint  = var.https_subscriptions[count.index]
}

resource "aws_lambda_permission" "sns_invoke" {
  count         = length(var.lambda_subscriptions)
  statement_id  = "AllowSNSInvoke-${count.index}"
  action        = "lambda:InvokeFunction"
  function_name = var.lambda_subscriptions[count.index]
  principal     = "sns.amazonaws.com"
  source_arn    = aws_sns_topic.this.arn
}
