locals {
  name_prefix = "${var.project_name}-${var.environment}"

  shared_buffers_value = var.serverless ? "262144" : (
    var.instance_class == "db.t4g.micro"   ? "393216" :  /* 1GB RAM / 4 = 256MB = 32768 * 8kB pages */
    var.instance_class == "db.t4g.small"  ? "1572864" : /* 2GB RAM / 4 = 512MB */
    var.instance_class == "db.t4g.medium" ? "3932160" : /* 4GB RAM / 4 = 1GB */
    var.instance_class == "db.r6g.large"  ? "2097152" : /* 8GB RAM / 4 = 2GB */
    var.instance_class == "db.r6g.xlarge" ? "4194304" : /* 16GB RAM / 4 = 4GB */
    "262144"
  )

  common_tags = merge(var.tags, {
    Module = "postgres"
  })
}

data "aws_rds_engine_version" "postgres" {
  engine  = "aurora-postgresql"
  version = var.engine_version
}

resource "aws_db_parameter_group" "this" {
  name_prefix = "${local.name_prefix}-pg-"
  family      = "aurora-postgresql16"
  description = "Parameter group for ${local.name_prefix} PostgreSQL with pgvector"

  tags = local.common_tags

  parameter {
    name  = "shared_buffers"
    value = local.shared_buffers_value
  }

  parameter {
    name  = "rds.logical_replication"
    value = "1"
  }

  parameter {
    name  = "pgvector.max_index_records"
    value = "1000000"
  }
}

resource "aws_db_subnet_group" "this" {
  name       = "${local.name_prefix}-postgres"
  subnet_ids = var.subnet_ids

  tags = merge(local.common_tags, {
    Name = "${local.name_prefix}-postgres-subnet-group"
  })
}

resource "aws_security_group" "postgres" {
  name_prefix = "${local.name_prefix}-postgres-"
  description = "Allow inbound PostgreSQL from the application security group"
  vpc_id      = var.vpc_id

  tags = merge(local.common_tags, {
    Name = "${local.name_prefix}-postgres-sg"
  })
}

resource "aws_security_group_rule" "postgres_ingress" {
  description       = "Allow PostgreSQL access from application"
  type              = "ingress"
  from_port         = 5432
  to_port           = 5432
  protocol          = "tcp"
  source_security_group_id = var.allowed_security_group_id
  security_group_id = aws_security_group.postgres.id
}

resource "aws_security_group_rule" "postgres_egress" {
  description       = "Allow all outbound"
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = aws_security_group.postgres.id
}

resource "aws_secretsmanager_secret" "db_credentials" {
  name_prefix = "${local.name_prefix}/postgres/"
  description = "PostgreSQL credentials for ${local.name_prefix}"

  tags = merge(local.common_tags, {
    Name = "${local.name_prefix}-postgres-credentials"
  })
}

resource "aws_secretsmanager_secret_version" "db_credentials" {
  secret_id = aws_secretsmanager_secret.db_credentials.id
  secret_string = jsonencode({
    username = var.username
    password = var.password
    engine   = "postgresql"
    host     = var.serverless ? aws_rds_cluster.serverless[0].endpoint : aws_db_instance.provisioned[0].address
    port     = 5432
    dbname   = var.database_name
  })
}

resource "aws_rds_cluster" "serverless" {
  count = var.serverless ? 1 : 0

  cluster_identifier_prefix = "${local.name_prefix}-"
  engine                    = "aurora-postgresql"
  engine_version            = var.engine_version
  database_name             = var.database_name
  master_username           = var.username
  master_password           = var.password

  db_subnet_group_name          = aws_db_subnet_group.this.name
  vpc_security_group_ids        = [aws_security_group.postgres.id]
  db_cluster_parameter_group_name = aws_db_parameter_group.this.name

  storage_type       = "aurora-iopt1"
  backup_retention_period = var.backup_retention_period
  skip_final_snapshot = var.skip_final_snapshot

  serverlessv2_scaling_configuration {
    min_capacity = var.serverless_min_capacity
    max_capacity = var.serverless_max_capacity
  }

  enabled_cloudwatch_logs_exports = ["postgresql"]

  tags = merge(local.common_tags, {
    Name = "${local.name_prefix}-aurora-cluster"
  })
}

resource "aws_rds_cluster_instance" "serverless" {
  count = var.serverless ? 1 : 0

  cluster_identifier  = aws_rds_cluster.serverless[0].cluster_identifier
  identifier_prefix   = "${local.name_prefix}-instance-"
  instance_class      = "db.serverless"
  engine              = aws_rds_cluster.serverless[0].engine
  engine_version      = aws_rds_cluster.serverless[0].engine_version

  tags = merge(local.common_tags, {
    Name = "${local.name_prefix}-aurora-instance"
  })
}

resource "aws_db_instance" "provisioned" {
  count = var.serverless ? 0 : 1

  identifier_prefix   = "${local.name_prefix}-"
  engine              = "postgresql"
  engine_version      = var.engine_version
  instance_class      = var.instance_class

  allocated_storage     = var.allocated_storage
  storage_type          = "gp3"
  storage_encrypted     = true

  db_name  = var.database_name
  username = var.username
  password = var.password

  db_subnet_group_name   = aws_db_subnet_group.this.name
  vpc_security_group_ids = [aws_security_group.postgres.id]
  parameter_group_name   = aws_db_parameter_group.this.name

  backup_retention_period = var.backup_retention_period
  skip_final_snapshot     = var.skip_final_snapshot
  deletion_protection     = !var.skip_final_snapshot

  performance_insights_enabled = true

  tags = merge(local.common_tags, {
    Name = "${local.name_prefix}-rds-instance"
  })
}

resource "null_resource" "create_pgvector" {
  count = var.create_extension ? 1 : 0

  triggers = {
    endpoint = var.serverless ? aws_rds_cluster.serverless[0].endpoint : aws_db_instance.provisioned[0].address
    db_name  = var.database_name
  }

  provisioner "local-exec" {
    command     = "PGPASSWORD=${var.password} psql -h ${var.serverless ? aws_rds_cluster.serverless[0].endpoint : aws_db_instance.provisioned[0].address} -p 5432 -U ${var.username} -d ${var.database_name} -c 'CREATE EXTENSION IF NOT EXISTS vector;'"
    interpreter = ["bash", "-c"]
  }

  depends_on = [
    aws_rds_cluster.serverless,
    aws_rds_cluster_instance.serverless,
    aws_db_instance.provisioned,
  ]
}
