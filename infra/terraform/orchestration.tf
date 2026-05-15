# OpenHack Infrastructure — Main Orchestration
# Wires all modules together for the full serverless stack.
# All 11 services run as Lambda — no Fargate required.
# SSE (Server-Sent Events) is replaced with frontend polling
# since Lambda does not support persistent connections.

# ================================
# Networking
# ================================

module "vpc" {
  source = "./modules/vpc"

  name        = "${var.project_name}-${var.environment}"
  vpc_cidr    = var.vpc_cidr
  common_tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}

# ================================
# Container Registry
# ================================

module "ecr" {
  source = "./modules/ecr"

  for_each = toset([
    "auth-svc", "core-svc", "mail-svc", "judging-svc",
    "leaderboard-svc", "notify-svc", "ai-svc", "analytics-svc",
    "sponsors-svc", "media-svc", "gateway-svc", "discord-bot-svc",
  ])

  name           = "${var.project_name}-${each.value}"
  immutable_tags = var.environment == "prod"
  tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}

# ================================
# Database
# ================================

module "postgres" {
  source = "./modules/postgres"

  environment               = var.environment
  serverless                = var.environment == "prod"
  instance_class            = var.db_instance_class
  serverless_min_capacity   = var.db_serverless_min_capacity
  serverless_max_capacity   = var.db_serverless_max_capacity
  username                  = var.db_username
  password                  = var.db_password
  database_name             = var.db_name
  subnet_ids                = module.vpc.isolated_subnet_ids
  allowed_security_group_id = module.vpc.lambda_security_group_id
  vpc_id                    = module.vpc.vpc_id
  backup_retention_period   = var.environment == "prod" ? 7 : 1
  skip_final_snapshot       = var.environment != "prod"
  create_extension          = true
  tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}

# ================================
# S3 — Media Storage
# ================================

module "s3" {
  source = "./modules/s3"

  name = "${var.project_name}-media-${var.environment}"

  cors_rules = var.environment == "prod" ? [
    {
      allowed_headers = ["*"]
      allowed_methods = ["GET", "PUT", "POST", "DELETE"]
      allowed_origins = ["https://${var.domain_name}"]
      expose_headers  = ["ETag"]
      max_age_seconds = 3600
    }
  ] : [
    {
      allowed_headers = ["*"]
      allowed_methods = ["GET", "PUT", "POST", "DELETE"]
      allowed_origins = ["*"]
      expose_headers  = ["ETag"]
      max_age_seconds = 3600
    }
  ]

  tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}

# ================================
# SNS — Inter-service Events
# ================================

module "sns" {
  source = "./modules/sns"

  for_each = {
    "auth-events"       = { display_name = "Auth Service Events" }
    "core-events"       = { display_name = "Core Service Events" }
    "judging-events"    = { display_name = "Judging Service Events" }
    "mail-events"       = { display_name = "Mail Service Events" }
    "leaderboard-events" = { display_name = "Leaderboard Recalculation Events" }
  }

  name         = "${var.project_name}-${each.key}-${var.environment}"
  display_name = each.value.display_name

  lambda_subscriptions = lookup(
    {
      "auth-events"       = [module.notify_lambda.function_arn]
      "core-events"       = [module.notify_lambda.function_arn]
      "judging-events"    = [module.notify_lambda.function_arn]
      "mail-events"       = [module.notify_lambda.function_arn]
      "leaderboard-events" = [module.leaderboard_lambda.function_arn]
    },
    each.key,
    []
  )

  tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}

# ================================
# SQS — Async Processing Queues
# ================================

module "recalculate_queue" {
  source = "./modules/sqs"

  name                        = "${var.project_name}-recalculate-${var.environment}"
  visibility_timeout_seconds  = 60
  dlq_max_receive_count       = 3
  lambda_arn                  = module.leaderboard_lambda.function_arn
  tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}

module "notify_event_queue" {
  source = "./modules/sqs"

  name                        = "${var.project_name}-notify-events-${var.environment}"
  visibility_timeout_seconds  = 30
  dlq_max_receive_count       = 3
  lambda_arn                  = module.notify_lambda.function_arn
  tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}

# ================================
# EventBridge — Scheduled Tasks
# ================================

module "eventbridge" {
  source = "./modules/eventbridge"

  event_bus_name = "${var.project_name}-${var.environment}"

  rules = [
    {
      name        = "phase-scheduler"
      description = "Trigger core-svc phase transition check"
      schedule    = "rate(30 seconds)"
    },
    {
      name        = "webhook-retry"
      description = "Trigger notify-svc webhook retry"
      schedule    = "rate(30 seconds)"
    },
  ]

  targets = [
    {
      rule_name = "phase-scheduler"
      id        = "core-lambda-target"
      arn       = module.core_lambda.function_arn
      input     = jsonencode({ path = "/api/core/admin/trigger-phase-check", method = "POST" })
    },
    {
      rule_name = "webhook-retry"
      id        = "notify-lambda-target"
      arn       = module.notify_lambda.function_arn
      input     = jsonencode({ path = "/api/notify/admin/retry-webhooks", method = "POST" })
    },
  ]

  lambda_permissions = [
    {
      rule_name  = "phase-scheduler"
      lambda_arn = module.core_lambda.function_arn
    },
    {
      rule_name  = "webhook-retry"
      lambda_arn = module.notify_lambda.function_arn
    },
  ]

  tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}

# ================================
# Lambda Functions (12 services)
# ================================

locals {
  db_url = "postgresql://${var.db_username}:${var.db_password}@${module.postgres.endpoint}:5432/${var.db_name}"

  common_env = {
    DATABASE_URL                   = local.db_url
    REDIS_URL                      = var.redis_url
    RUST_LOG                       = "info"
    ENVIRONMENT                    = var.environment
    AWS_LAMBDA_EVENT_ADAPTER       = "true"
    AWS_LAMBDA_EVENT_ADAPTER_ROUTE = "/lambda/event"
    LAMBDA_INTERNAL_TOKEN          = var.lambda_internal_token
  }

  common_tags = {
    Environment = var.environment
    Project     = var.project_name
  }

  lambda_services = {
    gateway = {
      image   = "${module.ecr["gateway-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {
        JWT_SECRET    = var.jwt_secret
        SERVICE_PORT  = "8080"
        SSE_ENABLED   = "false"
      }
    }
    auth = {
      image   = "${module.ecr["auth-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {
        JWT_SECRET           = var.jwt_secret
        JWT_EXPIRY           = "15m"
        REFRESH_TOKEN_EXPIRY = "7d"
      }
    }
    core = {
      image   = "${module.ecr["core-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {
        JWT_SECRET       = var.jwt_secret
        AUTH_SERVICE_URL = "http://localhost:8080"
        MEDIA_SERVICE_URL = "http://localhost:8080"
      }
    }
    judging = {
      image   = "${module.ecr["judging-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {}
    }
    leaderboard = {
      image   = "${module.ecr["leaderboard-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {}
    }
    mail = {
      image   = "${module.ecr["mail-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {
        SMTP_HOST = "email-smtp.us-east-1.amazonaws.com"
        SMTP_PORT = "587"
        MAIL_FROM = "noreply@openhack.dev"
      }
    }
    notify = {
      image   = "${module.ecr["notify-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {
        JWT_SECRET         = var.jwt_secret
        DISCORD_WEBHOOK_URL = var.discord_webhook_url
      }
    }
    ai = {
      image   = "${module.ecr["ai-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 512
      timeout = 120
      extra_env = {
        LLM_PROVIDER    = "openai"
        OPENAI_API_KEY  = var.openai_api_key
        OPENAI_MODEL    = "gpt-4-turbo"
        EMBEDDINGS_MODEL = "text-embedding-ada-002"
      }
    }
    analytics = {
      image   = "${module.ecr["analytics-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {}
    }
    sponsors = {
      image   = "${module.ecr["sponsors-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {
        JWT_SECRET = var.jwt_secret
      }
    }
    media = {
      image   = "${module.ecr["media-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {
        STORAGE_PROVIDER = "s3"
        S3_BUCKET_NAME   = module.s3.bucket_name
        S3_REGION        = var.aws_region
      }
    }
    discord_bot = {
      image   = "${module.ecr["discord-bot-svc"].repository_url}:${var.ecr_image_tag}-lambda"
      memory  = 256
      timeout = 30
      extra_env = {
        JWT_SECRET    = var.jwt_secret
        GATEWAY_URL   = "http://localhost:8080"
      }
    }
  }
}

module "auth_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-auth-${var.environment}"
  image_uri              = local.lambda_services.auth.image
  memory_size            = local.lambda_services.auth.memory
  timeout                = local.lambda_services.auth.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.auth.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "core_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-core-${var.environment}"
  image_uri              = local.lambda_services.core.image
  memory_size            = local.lambda_services.core.memory
  timeout                = local.lambda_services.core.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.core.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "judging_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-judging-${var.environment}"
  image_uri              = local.lambda_services.judging.image
  memory_size            = local.lambda_services.judging.memory
  timeout                = local.lambda_services.judging.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.judging.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "leaderboard_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-leaderboard-${var.environment}"
  image_uri              = local.lambda_services.leaderboard.image
  memory_size            = local.lambda_services.leaderboard.memory
  timeout                = local.lambda_services.leaderboard.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.leaderboard.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "mail_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-mail-${var.environment}"
  image_uri              = local.lambda_services.mail.image
  memory_size            = local.lambda_services.mail.memory
  timeout                = local.lambda_services.mail.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.mail.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "notify_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-notify-${var.environment}"
  image_uri              = local.lambda_services.notify.image
  memory_size            = local.lambda_services.notify.memory
  timeout                = local.lambda_services.notify.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.notify.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "ai_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-ai-${var.environment}"
  image_uri              = local.lambda_services.ai.image
  memory_size            = local.lambda_services.ai.memory
  timeout                = local.lambda_services.ai.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.ai.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "analytics_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-analytics-${var.environment}"
  image_uri              = local.lambda_services.analytics.image
  memory_size            = local.lambda_services.analytics.memory
  timeout                = local.lambda_services.analytics.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.analytics.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "sponsors_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-sponsors-${var.environment}"
  image_uri              = local.lambda_services.sponsors.image
  memory_size            = local.lambda_services.sponsors.memory
  timeout                = local.lambda_services.sponsors.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.sponsors.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

module "media_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-media-${var.environment}"
  image_uri              = local.lambda_services.media.image
  memory_size            = local.lambda_services.media.memory
  timeout                = local.lambda_services.media.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.media.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  policy_json            = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["s3:GetObject", "s3:PutObject", "s3:DeleteObject", "s3:ListBucket"]
        Resource = [
          module.s3.bucket_arn,
          "${module.s3.bucket_arn}/*"
        ]
      }
    ]
  })
  tags                   = local.common_tags
}

# ================================
# Discord Bot Lambda
# ================================

module "discord_bot_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-discord-bot-${var.environment}"
  image_uri              = local.lambda_services.discord_bot.image
  memory_size            = local.lambda_services.discord_bot.memory
  timeout                = local.lambda_services.discord_bot.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.discord_bot.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

# ================================
# Gateway Lambda (replaces Fargate — SSE replaced by frontend polling)
# ================================

module "gateway_lambda" {
  source = "./modules/lambda"

  function_name          = "${var.project_name}-gateway-${var.environment}"
  image_uri              = local.lambda_services.gateway.image
  memory_size            = local.lambda_services.gateway.memory
  timeout                = local.lambda_services.gateway.timeout
  environment_variables  = merge(local.common_env, local.lambda_services.gateway.extra_env)
  subnet_ids             = module.vpc.private_subnet_ids
  security_group_ids     = [module.vpc.lambda_security_group_id]
  secrets_manager_arns   = [module.postgres.secret_arn]
  tags                   = local.common_tags
}

# ================================
# API Gateway
# ================================

module "api_gateway" {
  source = "./modules/api-gateway"

  name = "${var.project_name}-api-${var.environment}"

  lambda_integrations = {
    gateway    = module.gateway_lambda.function_arn
    auth       = module.auth_lambda.function_arn
    core       = module.core_lambda.function_arn
    judging    = module.judging_lambda.function_arn
    leaderboard = module.leaderboard_lambda.function_arn
    mail       = module.mail_lambda.function_arn
    notify     = module.notify_lambda.function_arn
    ai         = module.ai_lambda.function_arn
    analytics  = module.analytics_lambda.function_arn
    sponsors   = module.sponsors_lambda.function_arn
    media      = module.media_lambda.function_arn
    discord_bot = module.discord_bot_lambda.function_arn
  }

  domain_name      = var.domain_name != "" ? var.domain_name : null
  certificate_arn  = var.certificate_arn != "" ? var.certificate_arn : null

  tags = {
    Environment = var.environment
    Project     = var.project_name
  }
}
