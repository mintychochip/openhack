terraform {
  required_version = ">= 1.7"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket         = "openhack-terraform-state"
    key            = "openhack-dev.tfstate"
    region         = "us-east-1"
    dynamodb_table = "openhack-terraform-lock"
    encrypt        = true
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "openhack"
      Environment = var.environment
      ManagedBy   = "terraform"
    }
  }
}
