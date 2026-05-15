output "vpc_id" {
  description = "The ID of the VPC."
  value       = aws_vpc.this.id
}

output "vpc_cidr" {
  description = "The CIDR block of the VPC."
  value       = aws_vpc.this.cidr_block
}

output "public_subnet_ids" {
  description = "List of public subnet IDs (2 subnets, one per AZ). These have direct internet access via the Internet Gateway."
  value       = aws_subnet.public[*].id
}

output "private_subnet_ids" {
  description = "List of private subnet IDs (2 subnets, one per AZ). These route egress traffic through the NAT Gateway."
  value       = aws_subnet.private[*].id
}

output "isolated_subnet_ids" {
  description = "List of isolated subnet IDs (2 subnets, one per AZ). These have no route to the internet."
  value       = aws_subnet.isolated[*].id
}

output "internet_gateway_id" {
  description = "The ID of the Internet Gateway attached to the VPC."
  value       = aws_internet_gateway.this.id
}

output "nat_gateway_ids" {
  description = "List of NAT Gateway IDs (2 gateways, one per AZ)."
  value       = aws_nat_gateway.this[*].id
}

output "public_route_table_id" {
  description = "The route table ID for public subnets. Routes 0.0.0.0/0 to the Internet Gateway."
  value       = aws_route_table.public.id
}

output "private_route_table_ids" {
  description = "List of private route table IDs (2 tables, one per AZ). Each routes 0.0.0.0/0 to the corresponding NAT Gateway."
  value       = aws_route_table.private[*].id
}

output "isolated_route_table_id" {
  description = "The route table ID for isolated subnets. Contains no default route to the internet."
  value       = aws_route_table.isolated.id
}

output "alb_security_group_id" {
  description = "Security group ID for the Application Load Balancer. Allows inbound 80/443 from 0.0.0.0/0."
  value       = aws_security_group.alb.id
}

output "lambda_security_group_id" {
  description = "Security group ID for Lambda functions. Allows inbound from ALB SG, outbound 443 to VPC CIDR."
  value       = aws_security_group.lambda.id
}

output "rds_security_group_id" {
  description = "Security group ID for RDS instances. Allows inbound 5432 from Lambda SG only."
  value       = aws_security_group.rds.id
}

output "availability_zones" {
  description = "The availability zones used for subnet distribution."
  value       = var.availability_zones
}
