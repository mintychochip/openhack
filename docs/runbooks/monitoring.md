# Monitoring & Alerting Guide

**Purpose:** Comprehensive guide to monitoring OpenHack services, dashboards, and alerting.

**Monitoring Stack:**
- **Prometheus** - Metrics collection and alerting
- **Grafana** - Visualization and dashboards
- **Alertmanager** - Alert routing and notification
- **Loki** (optional) - Log aggregation

---

## Dashboard Guide (Grafana)

### Overview Dashboard

**Purpose:** High-level view of platform health

**Panels:**
1. **Request Rate** - Total requests/sec across all services
2. **Error Rate** - Percentage of 4xx and 5xx responses
3. **Latency (p50, p95, p99)** - Response time percentiles
4. **Active Users** - Users active in last 5 minutes
5. **Service Health** - Up/Down status for each service
6. **Database Connections** - Active connections vs max
7. **Cache Hit Rate** - Redis cache effectiveness
8. **Queue Depth** - Pending jobs in background queues

**Key Metrics:**
```promql
# Request rate
sum(rate(http_requests_total[5m]))

# Error rate
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) * 100

# Latency p99
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))

# Active users
sum(rate(auth_logins_total[5m])) + sum(rate(auth_registrations_total[5m]))
```

---

### Service Health Dashboard

**Purpose:** Per-service deep dive

**Per-Service Panels:**
1. **CPU Usage** - Container CPU utilization
2. **Memory Usage** - Container memory vs limits
3. **Request Rate** - Requests/sec to this service
4. **Error Rate** - 4xx and 5xx rates
5. **Latency Heatmap** - Response time distribution
6. **Dependencies** - Status of DB, Redis, external APIs
7. **GC Pauses** (for JVM services) - Garbage collection impact
8. **Goroutine Count** (for Go services) - Concurrency level

**Key Metrics by Service:**

**Auth Service:**
```promql
# Login rate
rate(auth_logins_total[5m])

# Failed login rate
rate(auth_login_failures_total[5m])

# JWT issuance rate
rate(auth_jwt_issued_total[5m])

# Session count
redis_key_value{key="sessions:*"}
```

**Core Service:**
```promql
# Team creation rate
rate(core_teams_created_total[5m])

# Project submission rate
rate(core_projects_submitted_total[5m])

# Event RSVP rate
rate(core_event_rsvps_total[5m])
```

**Judging Service:**
```promql
# Score submission rate
rate(judging_scores_submitted_total[5m])

# Assignment completion rate
rate(judging_assignments_completed_total[5m])

# Average score by rubric
avg(judging_score_value) by (rubric_id)
```

**Leaderboard Service:**
```promql
# Vote rate
rate(leaderboard_votes_total[5m])

# Ranking updates
rate(leaderboard_rankings_updated_total[5m])

# Redis ZSET size
redis_zset_card{key="leaderboard:*"}
```

---

### Database Dashboard

**Purpose:** PostgreSQL performance and health

**Panels:**
1. **Connections** - Active, idle, waiting connections
2. **Query Rate** - Queries/sec by type (SELECT, INSERT, UPDATE, DELETE)
3. **Query Latency** - Average query time
4. **Slow Queries** - Queries >1 second
5. **Table Sizes** - Top 10 tables by size
6. **Index Usage** - Most/least used indexes
7. **Cache Hit Rate** - Buffer cache effectiveness
8. **Replication Lag** (if replicas) - Seconds behind primary
9. **Locks** - Active locks and wait times
10. **WAL Volume** - Write-ahead log generation rate

**Key Metrics:**
```promql
# Connection count
pg_stat_activity_count{state="active"}

# Query rate
rate(pg_stat_activity_count{state="active"}[5m])

# Cache hit rate
pg_stat_database_blks_hit / (pg_stat_database_blks_hit + pg_stat_database_blks_read)

# Slow queries
pg_stat_statements_seconds{query=~".*"} > 1

# Table sizes
pg_relation_size{relname=~".*"}

# Replication lag
pg_replication_lag_seconds
```

---

### Business Metrics Dashboard

**Purpose:** Hackathon-specific KPIs

**Panels:**
1. **Registrations** - Total and rate over time
2. **Team Formation** - Teams created, avg team size
3. **Project Submissions** - Projects submitted over time
4. **Judging Progress** - Scores submitted vs assignments
5. **Voting Activity** - Public votes cast
6. **Engagement** - DAU/MAU, session duration
7. **Geographic Distribution** - Users by country
8. **Conversion Funnel** - Registration → Team → Project → Submission

**Key Metrics:**
```promql
# Total registrations
auth_registrations_total

# Registration rate
rate(auth_registrations_total[1h])

# Team formation rate
rate(core_teams_created_total[1h])

# Project submission rate
rate(core_projects_submitted_total[1h])

# Judging completion
judging_scores_submitted_total / judging_assignments_created_total * 100

# Vote rate
rate(leaderboard_votes_total[1h])
```

---

## Alert Rule Reference

### Critical Alerts (SEV1)

| Alert Name | Condition | Severity | Action |
|------------|-----------|----------|--------|
| `ServiceDown` | `up{job="openhack"} == 0` for 2m | Critical | Page on-call |
| `DatabaseDown` | `pg_up == 0` for 1m | Critical | Page on-call + DBA |
| `RedisDown` | `redis_up == 0` for 1m | Critical | Page on-call |
| `HighErrorRate` | `error_rate > 10%` for 5m | Critical | Page on-call |
| `DiskFull` | `disk_usage > 95%` for 5m | Critical | Page on-call |
| `BackupFailed` | `backup_success == 0` for 24h | Critical | Page on-call |

**Prometheus Rules:**
```yaml
groups:
  - name: openhack-critical
    rules:
      - alert: ServiceDown
        expr: up{job="openhack"} == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.instance }} is down"
          description: "{{ $labels.job }} has been down for more than 2 minutes."
          
      - alert: DatabaseDown
        expr: pg_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "PostgreSQL database is down"
          description: "Database has been unreachable for more than 1 minute."
          
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) 
          / sum(rate(http_requests_total[5m])) * 100 > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }}% (threshold: 10%)"
```

---

### Warning Alerts (SEV2)

| Alert Name | Condition | Severity | Action |
|------------|-----------|----------|--------|
| `HighMemory` | `memory_usage > 80%` for 10m | Warning | Notify on-call |
| `HighCPU` | `cpu_usage > 80%` for 10m | Warning | Notify on-call |
| `SlowQueries` | `query_latency > 1s` for 10m | Warning | Notify on-call |
| `CacheHitRateLow` | `cache_hit_rate < 80%` for 30m | Warning | Notify on-call |
| `QueueBacklog` | `queue_depth > 1000` for 10m | Warning | Notify on-call |
| `CertificateExpiring` | `cert_expiry < 30d` | Warning | Notify on-call |

**Prometheus Rules:**
```yaml
groups:
  - name: openhack-warning
    rules:
      - alert: HighMemory
        expr: |
          container_memory_usage_bytes / container_spec_memory_limit_bytes * 100 > 80
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage on {{ $labels.container }}"
          description: "Memory usage is {{ $value }}% (threshold: 80%)"
          
      - alert: SlowQueries
        expr: pg_stat_statements_seconds{query=~".*"} > 1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Slow queries detected"
          description: "Queries taking longer than 1 second for 10 minutes"
```

---

### Info Alerts (SEV3/SEV4)

| Alert Name | Condition | Severity | Action |
|------------|-----------|----------|--------|
| `DiskUsageHigh` | `disk_usage > 70%` for 1h | Info | Create ticket |
| `BackupNotRun` | `backup_age > 24h` | Info | Create ticket |
| `SpikeInRegistrations` | `registration_rate > 2x normal` | Info | FYI only |
| `NewVersionAvailable` | `image_tag != latest` | Info | FYI only |

**Prometheus Rules:**
```yaml
groups:
  - name: openhack-info
    rules:
      - alert: DiskUsageHigh
        expr: node_filesystem_avail_bytes / node_filesystem_size_bytes * 100 < 30
        for: 1h
        labels:
          severity: info
        annotations:
          summary: "Disk usage above 70%"
          description: "Available disk space is {{ $value }}%"
          
      - alert: BackupNotRun
        expr: time() - backup_last_success_timestamp > 86400
        for: 1h
        labels:
          severity: info
        annotations:
          summary: "Backup has not run in 24 hours"
          description: "Last successful backup was {{ $value | humanizeDuration }} ago"
```

---

## On-Call Rotation Guide

### Rotation Schedule

**Weekly Rotation:**
- **Week 1:** Alice (Primary), Bob (Secondary)
- **Week 2:** Bob (Primary), Charlie (Secondary)
- **Week 3:** Charlie (Primary), Alice (Secondary)
- **Week 4:** Rotate as needed

**Handoff Procedure:**
1. **Monday 9 AM UTC** - Primary/Secondary swap
2. **Handoff call** (15 min) - Review open incidents, ongoing issues
3. **Update on-call doc** - Confirm new rotation in shared doc
4. **Test alerting** - Primary acknowledges test alert

---

### On-Call Responsibilities

**Primary On-Call:**
- [ ] Monitor alerts continuously
- [ ] Acknowledge alerts within 5 minutes (SEV1) / 15 minutes (SEV2)
- [ ] Triage and escalate as needed
- [ ] Lead incident response for SEV1/SEV2
- [ ] Update status page during incidents
- [ ] Create post-incident reports

**Secondary On-Call:**
- [ ] Backup for Primary (acknowledge if Primary unavailable)
- [ ] Assist with complex incidents
- [ ] Review and approve runbook updates
- [ ] Shadow Primary for training

---

### Alert Response Times

| Severity | Acknowledge | Investigate | Resolve | Escalate |
|----------|-------------|-------------|---------|----------|
| SEV1 | 5 minutes | 15 minutes | 1 hour | 30 minutes |
| SEV2 | 15 minutes | 30 minutes | 4 hours | 2 hours |
| SEV3 | 1 hour | 4 hours | 24 hours | 8 hours |
| SEV4 | 4 hours | 24 hours | 1 week | 3 days |

---

### On-Call Tools

**Alerting:**
- **PagerDuty** - Primary alerting platform
- **Slack** - Secondary notifications
- **Email** - Backup for SEV1

**Communication:**
- **Slack** - `#incidents` channel for coordination
- **Zoom** - War room bridge for SEV1
- **Status Page** - User-facing updates

**Monitoring:**
- **Grafana** - Dashboards
- **Prometheus** - Metrics and alerts
- **Loki** - Log exploration

---

## Escalation Procedures

### SEV1 Escalation Path

```
[0-5 min]  Primary On-Call acknowledges
[5-15 min] Secondary On-Call joins if no response
[15-30 min] Technical Lead escalated
[30-60 min] Engineering Manager escalated
[60+ min]  VP Engineering notified
```

**Escalation Commands:**
```bash
# PagerDuty API - escalate incident
curl -X POST https://api.pagerduty.com/incidents \
  -H "Authorization: Token token=$PAGERDUTY_TOKEN" \
  -d '{
    "incident": {
      "type": "incident",
      "title": "SEV1: Service Outage",
      "urgency": "high",
      "escalation_policy": {"id": "POLICY_ID", "type": "escalation_policy_reference"}
    }
  }'
```

---

### SEV2 Escalation Path

```
[0-15 min]  Primary On-Call acknowledges
[15-60 min] Secondary On-Call joins if stuck
[60-120 min] Technical Lead notified
[120+ min]  Engineering Manager notified
```

---

### SEV3/SEV4 Escalation

- Handled during business hours
- Create ticket in project management system
- Assign to appropriate team member
- Review in weekly team meeting

---

### External Escalation

**When to Escalate Externally:**
- Issue traced to third-party service (GitHub, AWS, etc.)
- Need vendor support for resolution
- SLA breach imminent

**External Contacts:**
| Vendor | Support Portal | Emergency Contact |
|--------|----------------|-------------------|
| AWS | https://console.aws.amazon.com/support | Enterprise support line |
| GitHub | https://support.github.com | enterprise@github.com |
| SendGrid | https://support.sendgrid.com | enterprise-support@sendgrid.com |
| Discord | https://discord.com/support | developer-support@discord.com |

---

## Runbook Links

- [Incident Response](./incident-response.md)
- [Maintenance Procedures](./maintenance.md)
- [Deployment Guide](../../DEPLOYMENT.md)
- [Testing Guide](../../TESTING.md)

---

**Last Updated:** May 13, 2026  
**Review Schedule:** Quarterly  
**Owner:** DevOps Team
