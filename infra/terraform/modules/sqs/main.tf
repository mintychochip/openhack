resource "aws_sqs_queue" "dlq" {
  name                       = var.fifo ? "${var.name}-dlq.fifo" : "${var.name}-dlq"
  message_retention_seconds  = var.message_retention_seconds
  tags                       = var.tags
}

resource "aws_sqs_queue" "main" {
  name                        = var.fifo ? "${var.name}.fifo" : var.name
  fifo_queue                  = var.fifo
  visibility_timeout_seconds  = var.visibility_timeout_seconds
  message_retention_seconds   = var.message_retention_seconds
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.dlq.arn
    maxReceiveCount     = var.dlq_max_receive_count
  })
  tags = var.tags
}

resource "aws_lambda_event_source_mapping" "this" {
  count            = var.lambda_arn != "" ? 1 : 0
  event_source_arn = aws_sqs_queue.main.arn
  function_name    = var.lambda_arn
  batch_size       = var.batch_size
}
