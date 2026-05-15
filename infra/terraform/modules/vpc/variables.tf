variable "name" {
  description = "Project or environment name used as a prefix for all resource names and tags."
  type        = string

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{1,62}$", var.name))
    error_message = "Name must start with a lowercase letter, contain only lowercase letters, digits, and hyphens, and be 2-63 characters."
  }
}

variable "vpc_cidr" {
  description = "CIDR block for the VPC. Must be a valid /16 or larger private network range."
  type        = string
  default     = "10.0.0.0/16"

  validation {
    condition     = can(cidrsubnets(var.vpc_cidr, 4, 4, 4, 4, 4, 4))
    error_message = "VPC CIDR must be large enough to allocate at least 6 /20 subnets."
  }
}

variable "public_subnet_cidrs" {
  description = "List of exactly 2 CIDR blocks for public subnets. These subnets host the ALB and NAT Gateway and have a direct route to the Internet Gateway."
  type        = list(string)
  default     = ["10.0.0.0/20", "10.0.16.0/20"]

  validation {
    condition     = length(var.public_subnet_cidrs) == 2
    error_message = "Exactly 2 public subnet CIDRs are required."
  }
}

variable "private_subnet_cidrs" {
  description = "List of exactly 2 CIDR blocks for private subnets. These subnets host Lambda workloads and route egress traffic through the NAT Gateway."
  type        = list(string)
  default     = ["10.0.32.0/20", "10.0.48.0/20"]

  validation {
    condition     = length(var.private_subnet_cidrs) == 2
    error_message = "Exactly 2 private subnet CIDRs are required."
  }
}

variable "isolated_subnet_cidrs" {
  description = "List of exactly 2 CIDR blocks for isolated subnets. These subnets host RDS instances and have no route to the internet or NAT Gateway."
  type        = list(string)
  default     = ["10.0.64.0/20", "10.0.80.0/20"]

  validation {
    condition     = length(var.isolated_subnet_cidrs) == 2
    error_message = "Exactly 2 isolated subnet CIDRs are required."
  }
}

variable "availability_zones" {
  description = "List of exactly 2 availability zones for subnet distribution. Each tier (public, private, isolated) places one subnet per AZ."
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b"]

  validation {
    condition     = length(var.availability_zones) == 2
    error_message = "Exactly 2 availability zones are required."
  }
}

variable "common_tags" {
  description = "Additional tags to merge into every resource. The Name tag is always set per-resource and will override any Name key supplied here."
  type        = map(string)
  default     = {}
}
