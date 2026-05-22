# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 12 Rust microservices: auth, core, gateway, judging, leaderboard, mail, notify, ai, analytics, sponsors, media, discord-bot
- Next.js web frontend with dashboard, admin, sponsor, and judging views
- CLI tool (`openhack`) with TUI for init, deploy, and per-service migration
- Docker Compose deployment with `--profile full` for all services
- Kubernetes Helm chart with 12 deployments, services, and configmaps
- AWS Lambda deployment via Terraform (11 Lambda functions + API Gateway)
- SSE real-time event streaming with Redis pub/sub and polling fallback
- JWT authentication with refresh tokens, MFA, and OAuth (GitHub, Google, Discord)
- Prometheus + Grafana monitoring stack with ServiceMonitors and alerting rules
- CI/CD pipeline: lint, check, test, audit, per-service Docker builds, Helm deploy
- Security scanning: cargo-audit, Trivy (filesystem + container), TruffleHog, checkov

### Changed
- Gateway migrated from Kong to custom Rust reverse proxy with SSE support
- All services migrated from Node.js/Python to Rust for performance
- Fargate removed in favor of Lambda for serverless mode (cost optimization)
- Database schema uses `TIMESTAMP` (not `TIMESTAMPTZ`) to match Rust `NaiveDateTime`

### Fixed
- SSE storm: Redis subscriber errors no longer propagate to clients (prevents cascading reconnections)
- Gateway CPU reduced from 90%+ to <0.2% by fixing Redis URL defaults and subscriber backoff
- Login redirect loop: token stored in both localStorage and cookie for middleware visibility
- Docker build caching: `touch openhack-common/src/lib.rs` forces recompile after COPY
- pgbouncer healthcheck: port 5432 (edoburu/pgbouncer ignores PGBOUNCER_PORT)
- Postfix: migrated from deprecated `catatnight/postfix` to `juanluisbaptiste/postfix`
- Terraform: 54 broken module references fixed (variable names, types, and structure)
- SQL: all 25 migration files included and ordered correctly in init.sql
