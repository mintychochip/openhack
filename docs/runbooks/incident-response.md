# Incident Response Runbook

**Purpose:** Standardized procedures for responding to production incidents in OpenHack.

**Severity Levels:**
- **SEV1 (Critical):** Complete service outage, data loss, security breach
- **SEV2 (High):** Major feature broken, significant degradation
- **SEV3 (Medium):** Minor feature broken, workaround available
- **SEV4 (Low):** Cosmetic issue, minor inconvenience

---

## Service Outage Procedures

### Gateway Service Down

**Symptoms:**
- All API requests return 502/503
- Kong container not running or unhealthy
- `docker compose ps` shows gateway as exited

**Immediate Actions:**
```bash
# Check gateway status
docker compose ps gateway

# View logs
docker compose logs --tail=100 gateway

# Restart gateway
docker compose restart gateway

# If restart fails, check config
docker compose run gateway kong check /kong.yml
```

**Escalation:** SEV1 if >5 minutes

---

### Auth Service Down

**Symptoms:**
- Login/registration failing
- JWT validation errors
- `docker compose logs auth-svc` shows crashes

**Immediate Actions:**
```bash
# Check service health
curl http://localhost:3001/health

# View logs
docker compose logs --tail=100 auth-svc

# Restart service
docker compose restart auth-svc

# Check database connectivity
docker compose exec auth-svc psql $DATABASE_URL -c "SELECT 1"
```

**Escalation:** SEV1 if >5 minutes (blocks all authentication)

---

### Core Service Down

**Symptoms:**
- Team/project creation failing
- Event management broken
- `docker compose logs core-svc` shows errors

**Immediate Actions:**
```bash
# Check service health
curl http://localhost:3002/health

# View logs
docker compose logs --tail=100 core-svc

# Restart service
docker compose restart core-svc

# Check database connectivity
docker compose exec core-svc psql $DATABASE_URL -c "SELECT 1"
```

**Escalation:** SEV2 if >10 minutes

---

### Database (PostgreSQL) Down

**Symptoms:**
- All services returning 500 errors
- `docker compose logs postgres` shows crashes
- Connection refused errors

**Immediate Actions:**
```bash
# Check database status
docker compose ps postgres

# View logs
docker compose logs --tail=100 postgres

# Check disk space
docker compose exec postgres df -h

# Check connections
docker compose exec postgres psql -U openhack -c "SELECT count(*) FROM pg_stat_activity"

# Restart database
docker compose restart postgres
```

**If Database Won't Start:**
```bash
# Check data directory permissions
ls -la docker/postgres/data

# Try to start in recovery mode
docker compose run postgres pg_ctl -D /var/lib/postgresql/data recovery

# Restore from backup (see Database Recovery section)
```

**Escalation:** SEV1 immediately

---

### Redis Down

**Symptoms:**
- Session errors
- Leaderboard not updating
- Cache miss errors in logs

**Immediate Actions:**
```bash
# Check Redis status
docker compose ps redis

# View logs
docker compose logs --tail=100 redis

# Test connectivity
docker compose exec redis redis-cli ping

# Restart Redis
docker compose restart redis

# Clear corrupted data (last resort)
docker compose exec redis redis-cli FLUSHALL
```

**Escalation:** SEV2 if >10 minutes

---

## Database Recovery Steps

### Point-in-Time Recovery

**Prerequisites:**
- WAL archiving enabled
- Backup from before the incident

**Steps:**
```bash
# 1. Stop all services
docker compose down

# 2. Restore base backup
pg_restore -U openhack -d openhack /backups/base-backup.sql

# 3. Replay WAL files
pg_wal_replay --target-time="2024-01-15 14:30:00"

# 4. Verify data integrity
psql -U openhack -d openhack -c "SELECT COUNT(*) FROM auth.users"

# 5. Restart services
docker compose up -d
```

---

### Replica Failover

**If using read replicas:**
```bash
# 1. Promote replica to primary
psql -h replica-host -U openhack -c "SELECT pg_promote()"

# 2. Update connection strings
# Edit .env file to point to new primary
# DATABASE_URL=postgresql://openhack:pass@new-primary:5432/openhack

# 3. Restart services
docker compose restart

# 4. Verify connectivity
curl http://localhost:8000/health
```

---

### Data Corruption Recovery

**Symptoms:**
- Checksum errors
- Invalid page headers
- Query failures on specific tables

**Steps:**
```bash
# 1. Identify corrupted tables
psql -U openhack -d openhack -c "SELECT * FROM pg_stat_user_tables WHERE last_error IS NOT NULL"

# 2. Dump unaffected tables
pg_dump -U openhack -d openhack -t good_table > good_table.sql

# 3. Restore from backup for corrupted tables
psql -U openhack -d openhack < backup/corrupted_table.sql

# 4. Run integrity check
psql -U openhack -d openhack -c "VACUUM ANALYZE"
```

---

## Rollback Procedures

### Helm Rollback (Kubernetes)

```bash
# View release history
helm history openhack -n openhack

# Rollback to previous revision
helm rollback openhack -n openhack --revision <REVISION_NUMBER>

# Verify rollback
helm status openhack -n openhack
kubectl get pods -n openhack

# Check service health
curl https://hackathon.example.com/health
```

---

### Docker Compose Rollback

```bash
# 1. Revert code changes
git checkout HEAD~1 docker-compose.yml

# 2. Pull previous images
docker compose pull

# 3. Restart services
docker compose down
docker compose up -d

# 4. Verify health
docker compose ps
curl http://localhost:8000/health
```

---

### Database Migration Rollback

```bash
# 1. Identify last migration
docker compose exec postgres psql -U openhack -d openhack \
  -c "SELECT * FROM schema_migrations ORDER BY version DESC LIMIT 1"

# 2. Run rollback script
./scripts/migrate rollback

# 3. Or manually rollback specific migration
docker compose exec auth-svc npm run db:rollback

# 4. Verify schema
docker compose exec postgres psql -U openhack -d openhack \
  -c "\d+ auth.users"
```

---

## Communication Templates

### Initial Incident Notification (Slack/Discord)

```
🚨 **INCIDENT ALERT** 🚨

**Severity:** SEV1/SEV2/SEV3
**Service:** [Service Name]
**Impact:** [Brief description of user impact]
**Started:** [Time incident began]
**Status:** Investigating

**Current Actions:**
- [ ] Identifying root cause
- [ ] Implementing fix
- [ ] Testing resolution

**Next Update:** [Time of next update, typically 15-30 min]

**Incident Commander:** [Name]
**Technical Lead:** [Name]
```

---

### Status Page Update

```
**Incident Update - [Service Name]**

**Time:** [Current time]
**Status:** [Investigating / Identified / Fixing / Monitoring / Resolved]

**Summary:**
[Brief description of what happened and current status]

**Impact:**
- [Affected feature 1]
- [Affected feature 2]

**Next Steps:**
[What we're doing to resolve]

**Estimated Resolution:** [Time if known]
```

---

### Post-Incident Report Template

```
# Post-Incident Report

**Incident Date:** [Date]
**Duration:** [Start time - End time]
**Severity:** SEV1/SEV2/SEV3

## Summary
[Brief description of what happened]

## Impact
- **Users Affected:** [Number/percentage]
- **Features Impacted:** [List]
- **Data Loss:** [Yes/No, details]
- **Revenue Impact:** [If applicable]

## Timeline (UTC)
- **[Time]:** Incident began
- **[Time]:** Detected by [monitoring/user report]
- **[Time]:** Incident commander assigned
- **[Time]:** Root cause identified
- **[Time]:** Fix implemented
- **[Time]:** Service restored
- **[Time]:** Incident declared resolved

## Root Cause
[Detailed technical explanation]

## Contributing Factors
- [Factor 1]
- [Factor 2]

## Resolution
[What was done to fix]

## Prevention
- [ ] [Action item 1] - Owner: [Name] - Due: [Date]
- [ ] [Action item 2] - Owner: [Name] - Due: [Date]
- [ ] [Action item 3] - Owner: [Name] - Due: [Date]

## Lessons Learned
- [What went well]
- [What could be improved]
- [Surprises encountered]

## Appendix
- Relevant logs: [Links]
- Metrics graphs: [Links]
- Related tickets: [Links]
```

---

### All-Clear Notification

```
✅ **INCIDENT RESOLVED** ✅

**Service:** [Service Name]
**Duration:** [X hours Y minutes]
**Severity:** SEV1/SEV2/SEV3

**Summary:**
[Brief description of resolution]

**Current Status:**
All systems operational. Monitoring continues for [X] hours.

**Post-Incident Review:**
Scheduled for [Date/Time]. Link to invite: [Link]

**Thank you** to everyone who helped resolve this incident!
```

---

## Emergency Contacts

| Role | Name | Phone | Email |
|------|------|-------|-------|
| Incident Commander | [Name] | [Phone] | [Email] |
| Technical Lead | [Name] | [Phone] | [Email] |
| Database Admin | [Name] | [Phone] | [Email] |
| DevOps Lead | [Name] | [Phone] | [Email] |
| Communications | [Name] | [Phone] | [Email] |

---

## External Services

| Service | Status Page | Support Contact |
|---------|-------------|-----------------|
| GitHub OAuth | https://www.githubstatus.com | support@github.com |
| Google OAuth | https://status.cloud.google.com | support@google.com |
| Discord OAuth | https://discordstatus.com | support@discord.com |
| AWS S3 | https://status.aws.amazon.com | AWS Support |
| SendGrid | https://status.sendgrid.com | support@sendgrid.com |

---

**Last Updated:** May 13, 2026  
**Review Schedule:** Quarterly
