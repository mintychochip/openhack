# Dev environment — cheapest possible
# Estimated cost: ~$15/mo

aws_region                = "us-east-1"
environment               = "dev"
project_name              = "openhack"

# VPC
vpc_cidr                  = "10.0.0.0/16"

# Database — smallest provisioned instance
db_instance_class         = "db.t4g.micro"
db_serverless_min_capacity = 0.5
db_serverless_max_capacity = 1

# No Redis (services degrade gracefully)
redis_url                 = ""

# Lambda — minimal resources
lambda_memory_size        = 256
lambda_timeout            = 30

# ECR
ecr_image_tag             = "latest"
