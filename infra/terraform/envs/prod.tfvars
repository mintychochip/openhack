# Production environment — Aurora Serverless v2 + provisioned concurrency
# Estimated cost: ~$80-150/mo depending on traffic

aws_region                = "us-east-1"
environment               = "prod"
project_name              = "openhack"

# VPC
vpc_cidr                  = "10.0.0.0/16"

# Database — Aurora Serverless v2 (production capacity)
db_instance_class         = "db.serverless"
db_serverless_min_capacity = 0.5
db_serverless_max_capacity = 4

# Redis via Upstash or ElastiCache
redis_url                 = ""  # Set via CI/CD secret

# Lambda — higher resources for production
lambda_memory_size        = 512
lambda_timeout            = 60

# ECR — immutable tags in production
ecr_image_tag             = "latest"  # Override with semver tag on deploy
