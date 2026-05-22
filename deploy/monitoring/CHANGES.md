# Monitoring Stack Improvements - P0 + P1 + Phase 7 Implementation

**Date:** May 15, 2026  
**Status:** Complete (All Phases)

## Summary

Implemented comprehensive monitoring improvements covering:
- **Phases 1-6:** Critical fixes, infrastructure exporters, alerting, logging, and dashboard rebuilds
- **Phase 7:** Business domain metrics in 4 Rust services (auth, core, judging, leaderboard)

All business KPIs now emit to Prometheus and are visible in Grafana dashboards.

---

## Phase 1: Quick Fixes (Complete)

### 1a. Watchdog Alert
- **File:** `prometheus/rules.yaml`
- **Change:** Added `Watchdog` alert that always fires to prove alerting pipeline health
- **Impact:** On-call can verify alerting works end-to-end

### 1b. HighErrorRate Alert Annotation
- **File:** `prometheus/rules.yaml`
- **Change:** Updated description to clarify "failed" = all HTTP >=400 status codes
- **Impact:** Operators understand what the alert measures

### 1c. DeploymentReplicasMismatch Fix
- **File:** `prometheus/rules.yaml`
- **Change:** Fixed annotation that incorrectly referenced `$labels.value` twice
- **Impact:** Correct alert description displayed

### 1d. Histogram _sum Unit Fix
- **File:** `services/openhack-common/src/metrics.rs`
- **Change:** Fixed `http_request_duration_seconds_sum` to output seconds (was milliseconds)
- **Impact:** Average latency calculations now correct

---

## Phase 2: Status Code Labels (Complete)

### Metrics Middleware Refactor
- **Files:** 
  - `services/openhack-common/Cargo.toml` - Added `prometheus = "0.13"` crate
  - `services/openhack-common/src/metrics.rs` - Complete rewrite using prometheus crate
  - All 11 service `main.rs` files - Updated to use `MetricsMiddleware::new("service-name")`
- **Changes:**
  - `http_requests_total{service, status}` - Now labeled by status code
  - `http_requests_failed{service, status}` - Now labeled by status code
  - `http_request_duration_seconds{service}` - Histogram with proper labels
  - `process_uptime_seconds` - Gauge
- **Impact:** 
  - Can now filter by status code: `http_requests_total{status=~"5.."}`
  - Proper 5xx-only alerting possible
  - Per-status-code dashboards possible

---

## Phase 3: Infrastructure Exporters (Complete)

### 3a. PostgreSQL Exporter
- **File:** `exporters/postgres.yaml`
- **Deploys:** Deployment, Service, ServiceMonitor for postgres_exporter
- **Metrics:** `pg_up`, `pg_stat_activity_count`, `pg_stat_database_*`, `pg_stat_statements_*`, etc.
- **Connection:** Uses `postgresql://openhack:openhack_pass@postgres:5432/openhack?sslmode=disable`

### 3b. Redis Exporter
- **File:** `exporters/redis.yaml`
- **Deploys:** Deployment, Service, ServiceMonitor for redis_exporter
- **Metrics:** `redis_up`, `redis_keyspace_hits_total`, `redis_keyspace_misses_total`, `redis_memory_*`, etc.
- **Connection:** Uses `redis://redis:6379`

### 3c. Blackbox Exporter
- **File:** `exporters/blackbox.yaml`
- **Deploys:** ConfigMap, Deployment, Service, ServiceMonitor for blackbox_exporter
- **Modules:** HTTP 2xx check, TCP connect, SSL certificate check
- **Impact:** Enables `CertificateExpiringSoon` alert

### 3d. NGINX Ingress ServiceMonitor
- **File:** `prometheus/servicemonitors.yaml`
- **Change:** Added ServiceMonitor for ingress-nginx controller
- **Metrics:** `nginx_ingress_controller_requests`, `nginx_ingress_controller_*`
- **Impact:** Visibility into ingress-level traffic and errors

---

## Phase 4: Missing Alert Rules (Complete)

### New Alerts Added
- **File:** `prometheus/rules.yaml`

| Alert | Severity | Expression | For |
|-------|----------|------------|-----|
| `DatabaseDown` | critical | `pg_up == 0` | 1m |
| `RedisDown` | critical | `redis_up == 0` | 1m |
| `SlowQueries` | warning | `pg_stat_statements_mean_exec_seconds > 1` | 10m |
| `CacheHitRateLow` | warning | `redis_keyspace_hits_total / (hits+misses) < 0.8` | 30m |
| `CertificateExpiringSoon` | info | `probe_ssl_earliest_cert_expiry - time() < 30d` | 1h |

---

## Phase 5: Loki + Promtail (Complete)

### Configuration
- **File:** `values-local.yaml`
- **Changes:**
  - Added `loki-stack` Helm chart configuration
  - Added `promtail` DaemonSet configuration
  - Added Loki as Grafana datasource
- **Impact:** Centralized log aggregation and search

### Install Script Update
- **File:** `install.sh`
- **Changes:**
  - Added Loki stack installation step
  - Added exporters application step
  - Updated "Next steps" section with Loki and exporter verification commands

---

## Phase 6: Dashboard Rebuilds (Complete)

### 6a. Database Dashboard (Complete Rebuild)
- **File:** `grafana/database.json`
- **Before:** 6 panels showing pod CPU/memory (not PostgreSQL metrics)
- **After:** 12 panels with real PostgreSQL metrics:
  1. Database Connections (active/idle/idle in transaction)
  2. Query Rate
  3. Query Latency (mean exec/plan time)
  4. Slow Queries (>1s)
  5. Cache Hit Rate (gauge)
  6. Table Sizes (Top 10 bar chart)
  7. Replication Lag
  8. Dead Tuples
  9. Database Size
  10. PostgreSQL Exporter Health
  11. Pod Resource Usage (memory)
  12. Postgres CPU Usage

### 6b. Overview Dashboard (Enhanced)
- **File:** `grafana/overview.json`
- **Changes:**
  - Added Cache Hit Rate panel (Redis)
  - Added Latency percentiles (p50, p95, p99) in single panel
  - Added NGINX Ingress Request Rate panel
  - Added NGINX Ingress 5xx Errors panel
  - Added Database Health panel (pg_up, redis_up)
  - Added Pod Restarts panel
- **Before:** 5 panels
- **After:** 10 panels

### 6c. Services Dashboard (Enhanced)
- **File:** `grafana/services.json`
- **Changes:**
  - Added HTTP Errors by Status Class (4xx vs 5xx breakdown)
  - Added Error Rate by Service (gauge)
  - Added Latency Heatmap
  - Added Request Distribution by Status (pie chart)
  - Added Top Endpoints by Traffic (placeholder - needs path label)
- **Before:** 6 panels
- **After:** 10 panels

### 6d. Business Dashboard (Placeholders + Documentation)
- **File:** `grafana/business.json`
- **Changes:**
  - Added panels for business KPIs (will show "No data" until Phase 7)
  - Added "Implementation Notes" text panel documenting required metrics
- **Panels:**
  1. User Registrations
  2. Registration Rate
  3. Teams Created
  4. Avg Team Size
  5. Project Submissions Over Time
  6. Judging Progress (gauge)
  7. Voting Activity
  8. Total Votes Cast
  9-11. HTTP proxy metrics (while business metrics are implemented)
  12. Implementation Notes (text panel)

---

## Installation

### Apply Changes

```bash
# Navigate to monitoring directory
cd deploy/monitoring

# Reinstall monitoring stack with new values
helm upgrade --install openhack-monitoring prometheus-community/kube-prometheus-stack \
  --namespace openhack \
  --values values-local.yaml \
  --timeout 10m \
  --wait

# Apply exporters
kubectl apply -f exporters/

# Apply ServiceMonitors and alert rules
kubectl apply -f prometheus/servicemonitors.yaml
kubectl apply -f prometheus/rules.yaml
```

### Verify Installation

```bash
# Check exporters are running
kubectl get pods -l app=postgres-exporter,redis-exporter,blackbox-exporter

# Check Loki is running
kubectl get pods -l app=loki

# Check Prometheus targets
kubectl port-forward svc/openhack-monitoring-prometheus 9090:80 -n openhack
# Visit http://localhost:9090/targets

# Verify new targets:
# - postgres-exporter: UP
# - redis-exporter: UP
# - blackbox-exporter: UP
# - nginx-ingress-controller: UP (in ingress-nginx namespace)

# Check alerts loaded
kubectl port-forward svc/openhack-monitoring-prometheus 9090:80 -n openhack
# Visit http://localhost:9090/alerts
# Should see: Watchdog, DatabaseDown, RedisDown, SlowQueries, CacheHitRateLow, CertificateExpiringSoon
```

---

## Metrics Now Available

### PostgreSQL Metrics (via postgres_exporter)
- `pg_up` - Exporter health (1=healthy)
- `pg_stat_activity_count{state}` - Connections by state
- `pg_stat_database_blks_hit` - Cache hits
- `pg_stat_database_blks_read` - Cache misses
- `pg_stat_statements_calls_total` - Query count
- `pg_stat_statements_mean_exec_seconds` - Mean query time
- `pg_relation_size_bytes{relname}` - Table sizes
- `pg_replication_lag_seconds` - Replication lag
- `pg_stat_user_tables_n_dead_tup` - Dead tuples
- `pg_database_size_bytes{datname}` - Database size

### Redis Metrics (via redis_exporter)
- `redis_up` - Exporter health (1=healthy)
- `redis_keyspace_hits_total` - Cache hits
- `redis_keyspace_misses_total` - Cache misses
- `redis_memory_used_bytes` - Memory usage
- `redis_connected_clients` - Client connections
- `redis_uptime_in_seconds` - Redis uptime

### NGINX Ingress Metrics
- `nginx_ingress_controller_requests` - Request count
- `nginx_ingress_controller_request_duration_seconds` - Request latency
- `nginx_ingress_controller_bytes_sent` - Bytes sent

### SSL Certificate Metrics (via blackbox_exporter)
- `probe_ssl_earliest_cert_expiry` - Certificate expiry timestamp
- `probe_ssl_last_chain_expiry_timestamp_seconds` - Chain expiry

---

## Phase 7: Business Domain Metrics (Complete)

### Business Metrics Implementation

Added custom business counters to 4 services via `openhack-common::metrics::register_business_counter()` and `inc_business_counter()`.

| Service | Metrics Implemented | Location |
|---------|---------------------|----------|
| auth-svc | `auth_registrations_total`, `auth_logins_total`, `auth_login_failures_total`, `auth_jwt_issued_total` | `main.rs` (registration), `routes/register.rs`, `routes/login.rs` (increments) |
| core-svc | `core_teams_created_total`, `core_projects_submitted_total`, `core_event_rsvps_total` | `main.rs` (registration), `routes/team.rs`, `routes/project.rs`, `routes/event.rs` (increments) |
| judging-svc | `judging_scores_submitted_total`, `judging_assignments_completed_total` | `main.rs` (registration), `routes/scores.rs` (increments) |
| leaderboard-svc | `leaderboard_votes_total`, `leaderboard_rankings_updated_total` | `main.rs` (registration), `routes/votes.rs` (increments) |

**Pattern:**
```rust
// In main.rs at startup
openhack_common::metrics::register_business_counter("metric_name", "Help text");

// In route handlers on success
openhack_common::metrics::inc_business_counter("metric_name");
```

**Verification:**
```bash
curl http://localhost:3001/metrics | grep auth_
# Should show:
# auth_registrations_total X
# auth_logins_total X
# auth_login_failures_total X
# auth_jwt_issued_total X
```

---

## Files Modified

### Configuration Files
- `deploy/monitoring/values-local.yaml` - Added Loki, Promtail, Grafana datasource config
- `deploy/monitoring/install.sh` - Added Loki installation, exporter application
- `deploy/monitoring/prometheus/rules.yaml` - Added 5 new alerts, fixed existing bugs
- `deploy/monitoring/prometheus/servicemonitors.yaml` - Added NGINX ingress ServiceMonitor

### New Files
- `deploy/monitoring/exporters/postgres.yaml` - PostgreSQL exporter
- `deploy/monitoring/exporters/redis.yaml` - Redis exporter
- `deploy/monitoring/exporters/blackbox.yaml` - Blackbox exporter
- `deploy/monitoring/CHANGES.md` - This document

### Rust Code
- `services/openhack-common/Cargo.toml` - Added prometheus crate
- `services/openhack-common/src/metrics.rs` - Complete rewrite with labeled metrics
- `services/auth-rust/src/main.rs` - Updated middleware usage
- `services/core-rust/src/main.rs` - Updated middleware usage
- `services/gateway-rust/src/main.rs` - Updated middleware usage
- `services/judging-rust/src/main.rs` - Updated middleware usage
- `services/leaderboard/src/main.rs` - Updated middleware usage
- `services/mail-rust/src/main.rs` - Updated middleware usage
- `services/notify-rust/src/main.rs` - Updated middleware usage
- `services/ai-rust/src/main.rs` - Updated middleware usage
- `services/analytics-rust/src/main.rs` - Updated middleware usage
- `services/sponsors-rust/src/main.rs` - Updated middleware usage
- `services/media-rust/src/main.rs` - Updated middleware usage
- `services/discord-bot-rust/src/main.rs` - Updated middleware usage

### Dashboards
- `deploy/monitoring/grafana/database.json` - Complete rebuild with PostgreSQL metrics
- `deploy/monitoring/grafana/overview.json` - Enhanced with ingress, cache, latency panels
- `deploy/monitoring/grafana/services.json` - Enhanced with status breakdown, heatmap
- `deploy/monitoring/grafana/business.json` - Placeholders + implementation notes

---

## Testing

### Compile Rust Services

```bash
cd services/openhack-common
cargo check

cd ../auth-rust
cargo check

# Repeat for all services
```

### Verify Metrics Format

After deploying, check metrics output:

```bash
kubectl port-forward svc/openhack-auth-svc 3001:3001 -n openhack
curl http://localhost:3001/metrics
```

Expected output includes:
```
http_requests_total{service="auth",status="200"} 142
http_requests_total{service="auth",status="404"} 5
http_requests_failed{service="auth",status="404"} 5
http_request_duration_seconds_sum{service="auth"} 2.345
http_request_duration_seconds_count{service="auth"} 142
```

---

## Alert Coverage

### Before: 9 alerts
### After: 15 alerts

| Alert | Severity | Status |
|-------|----------|--------|
| Watchdog | none | NEW |
| ServiceDown | critical | Existing |
| HighErrorRate | critical | Fixed (annotation) |
| DiskFull | critical | Existing |
| DatabaseDown | critical | NEW |
| RedisDown | critical | NEW |
| HighMemory | warning | Existing |
| HighCPU | warning | Existing |
| PodCrashLooping | warning | Existing |
| HighLatency | warning | Existing |
| SlowQueries | warning | NEW |
| CacheHitRateLow | warning | NEW |
| DiskUsageHigh | info | Existing |
| DeploymentReplicasMismatch | info | Fixed (annotation) |
| CertificateExpiringSoon | info | NEW |

---

## Dashboard Coverage

### Before
- Overview: 5 panels (40% of runbook spec)
- Services: 6 panels (45% of runbook spec)
- Database: 6 panels (0% PostgreSQL - showed pod metrics)
- Business: 4 panels (0% business KPIs - showed HTTP metrics)

### After
- Overview: 10 panels (includes ingress, cache, latency percentiles)
- Services: 10 panels (includes status breakdown, heatmap)
- Database: 12 panels (100% PostgreSQL metrics)
- Business: 12 panels (placeholders + documentation)

---

## Known Limitations

1. **Business metrics** - Require Rust code changes (Phase 7)
2. **Path-based metrics** - Middleware doesn't capture request path (would require additional label)
3. **Method-based metrics** - Middleware doesn't capture HTTP method (GET/POST/etc.)
4. **Distributed tracing** - Still not implemented (requires OTEL collector, Tempo/Jaeger)
5. **SLI/SLO tracking** - No error budget or burn rate alerts
6. **Horizontal Pod Autoscaler** - No custom metrics pipeline for scaling

---

## Production Deployment Notes

**DO NOT apply `values-local.yaml` to production clusters.**

For production:
1. Use base `values.yaml` without kind-specific overrides
2. Re-enable control-plane ServiceMonitors (kubeEtcd, kubeScheduler, etc.)
3. Configure proper TLS for PostgreSQL and Redis connections
4. Use managed services (RDS, ElastiCache) where applicable
5. Configure Alertmanager with real Slack/PagerDuty webhooks
6. Increase resource limits for production workloads
7. Enable persistence with appropriate storage classes
8. Configure backup and disaster recovery procedures
