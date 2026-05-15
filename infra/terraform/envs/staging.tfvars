# Staging environment — Aurora Serverless v2
# Estimated cost: ~$40/mo

aws_region                = "us-east-1"
environment               = "staging"
project_name              = "openhack"

# VPC
vpc_cidr                  = "10.0.0.0/16"

# Database — Aurora Serverless v2 (pauses when idle)
db_instance_class         = "db.serverless"
db_serverless_min_capacity = 0.5
db_serverless_max_capacity = 2

# Redis via Upstash free tier (optional)
redis_url                 = ""

# Lambda — balanced
lambda_memory_size        = 256
lambda_timeout            = 30

# ECR
ecr_image_tag             = "latest"
