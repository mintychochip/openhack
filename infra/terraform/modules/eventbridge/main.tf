resource "aws_cloudwatch_event_bus" "this" {
  name = var.event_bus_name
  tags = var.tags
}

resource "aws_cloudwatch_event_rule" "this" {
  count               = length(var.rules)
  name                = var.rules[count.index].name
  description         = var.rules[count.index].description
  event_bus_name      = aws_cloudwatch_event_bus.this.name
  schedule_expression = try(var.rules[count.index].schedule, null)
  event_pattern       = try(var.rules[count.index].event_pattern, null)
  is_enabled          = try(var.rules[count.index].enabled, true)
  tags                = var.tags
}

resource "aws_cloudwatch_event_target" "this" {
  count          = length(var.targets)
  rule           = var.targets[count.index].rule_name
  target_id      = var.targets[count.index].id
  arn            = var.targets[count.index].arn
  role_arn      = try(var.targets[count.index].role_arn, null)
  event_bus_name = aws_cloudwatch_event_bus.this.name
  input          = try(var.targets[count.index].input, null)
  depends_on     = [aws_cloudwatch_event_rule.this]
}

resource "aws_lambda_permission" "eventbridge" {
  count         = length(var.lambda_permissions)
  statement_id  = "AllowEventBridge-${var.lambda_permissions[count.index].rule_name}"
  action        = "lambda:InvokeFunction"
  function_name = var.lambda_permissions[count.index].lambda_arn
  principal     = "events.amazonaws.com"
  source_arn    = one([for r in aws_cloudwatch_event_rule.this : r.arn if r.name == var.lambda_permissions[count.index].rule_name])
}
