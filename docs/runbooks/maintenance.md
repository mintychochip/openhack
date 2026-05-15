# Maintenance Procedures Runbook

**Purpose:** Standardized procedures for scheduled and unscheduled maintenance in OpenHack.

**Maintenance Windows:**
- **Primary:** Tuesday 2:00-4:00 AM UTC (low-traffic period)
- **Secondary:** Sunday 3:00-5:00 AM UTC
- **Emergency:** As needed with stakeholder approval

---

## Scheduled Maintenance Checklist

### Pre-Maintenance (1 Week Before)

- [ ] **Announce maintenance window**
  - [ ] Post to status page
  - [ ] Send email to users (for major maintenance)
  - [ ] Post to community Discord/Slack
  - [ ] Update social media if needed

- [ ] **Prepare rollback plan**
  - [ ] Document current versions
  - [ ] Create backup of database
  - [ ] Test rollback procedure in staging

- [ ] **Verify backups**
  - [ ] Confirm latest backup completed successfully
  - [ ] Test restore in staging environment
  - [ ] Document backup location and credentials

- [ ] **Prepare team**
  - [ ] Assign incident commander
  - [ ] Assign technical lead
  - [ ] Schedule on-call coverage
  - [ ] Share runbook with team

---

### Pre-Maintenance (1 Day Before)

- [ ] **Send reminder notification**
  - [ ] Status page update
  - [ ] Email reminder
  - [ ] Discord/Slack reminder

- [ ] **Final backup**
  - [ ] Create fresh database backup
  - [ ] Verify backup integrity
  - [ ] Store backup in secondary location

- [ ] **Check monitoring**
  - [ ] Verify all dashboards operational
  - [ ] Test alerting channels
  - [ ] Confirm on-call rotation active

- [ ] **Prepare communication channels**
  - [ ] Create incident channel (Slack/Discord)
  - [ ] Test screen sharing tools
  - [ ] Prepare status page templates

---

### During Maintenance

- [ ] **Begin maintenance**
  - [ ] Post "maintenance started" to status page
  - [ ] Announce in Discord/Slack
  - [ ] Start timer for maintenance window

- [ ] **Execute maintenance tasks**
  - [ ] Follow specific procedure (see below)
  - [ ] Document each step with timestamp
  - [ ] Monitor logs and metrics continuously

- [ ] **Verify completion**
  - [ ] Run health checks on all services
  - [ ] Verify key user flows work
  - [ ] Check metrics for anomalies

- [ ] **End maintenance**
  - [ ] Post "maintenance complete" to status page
  - [ ] Announce in Discord/Slack
  - [ ] Send completion email if major maintenance

---

### Post-Maintenance (Within 24 Hours)

- [ ] **Monitor closely**
  - [ ] Review error rates
  - [ ] Check performance metrics
  - [ ] Monitor user reports

- [ ] **Document outcomes**
  - [ ] Update maintenance log
  - [ ] Note any issues encountered
  - [ ] Record lessons learned

- [ ] **Follow-up tasks**
  - [ ] Complete any deferred tasks
  - [ ] Clean up temporary files/configs
  - [ ] Update documentation if needed

---

## Database Migration Procedures

### Zero-Downtime Migration Strategy

**Golden Rules:**
1. **Backwards compatible changes first** - Add columns with defaults, don't remove
2. **Deploy code before migration** - New code handles both old and new schema
3. **Migrate data in batches** - Avoid long-running transactions
4. **Remove old code after migration** - Clean up in next deployment

---

### Adding a Column (Safe)

**Step 1: Add Column (Deploy 1)**
```sql
-- Migration: 001_add_new_column.sql
ALTER TABLE auth.users 
ADD COLUMN new_field VARCHAR(255) DEFAULT '';

-- Note: Must have DEFAULT to avoid table lock
```

**Step 2: Update Code (Deploy 2)**
```typescript
// Code now writes to both old and new fields
// during transition period
```

**Step 3: Backfill Data (Deploy 3)**
```sql
-- Backfill in batches to avoid lock
UPDATE auth.users 
SET new_field = old_field 
WHERE new_field = '' 
AND id BETWEEN 1 AND 10000;

-- Repeat for each batch
```

**Step 4: Make Column Required (Deploy 4)**
```sql
-- Only after all data migrated
ALTER TABLE auth.users 
ALTER COLUMN new_field SET NOT NULL;
```

**Step 5: Remove Old Column (Deploy 5 - Next Release)**
```sql
ALTER TABLE auth.users 
DROP COLUMN old_field;
```

---

### Removing a Column (Safe)

**Step 1: Stop Writing to Column (Deploy 1)**
```typescript
// Remove code that writes to old_column
// Code only reads from old_column for now
```

**Step 2: Make Column Nullable (Deploy 2)**
```sql
ALTER TABLE auth.users 
ALTER COLUMN old_column DROP NOT NULL;
```

**Step 3: Remove Read Access (Deploy 3)**
```typescript
// Remove all references to old_column
// Code no longer reads or writes
```

**Step 4: Drop Column (Deploy 4)**
```sql
-- Only after confirming no code uses it
ALTER TABLE auth.users 
DROP COLUMN old_column;
```

---

### Data Migration (Large Tables)

**For tables >1M rows:**

```sql
-- 1. Create new table with new schema
CREATE TABLE auth.users_new (LIKE auth.users INCLUDING ALL);

-- 2. Add new columns/changes to new table
ALTER TABLE auth.users_new ADD COLUMN new_field VARCHAR(255);

-- 3. Migrate data in batches
DO $$
DECLARE
    batch_size INTEGER := 10000;
    offset_val INTEGER := 0;
BEGIN
    LOOP
        INSERT INTO auth.users_new (id, email, name, new_field)
        SELECT id, email, name, COALESCE(old_field, '')
        FROM auth.users
        ORDER BY id
        LIMIT batch_size OFFSET offset_val;
        
        offset_val := offset_val + batch_size;
        EXIT WHEN NOT FOUND;
        
        -- Commit each batch
        COMMIT;
    END LOOP;
END $$;

-- 4. Swap tables (during maintenance window)
BEGIN;
ALTER TABLE auth.users RENAME TO auth_users_old;
ALTER TABLE auth.users_new RENAME TO users;
COMMIT;

-- 5. Keep old table for rollback (drop after verification)
-- DROP TABLE auth_users_old;
```

---

### Migration Rollback

**If migration fails:**

```bash
# 1. Stop application
docker compose down

# 2. Restore from pre-migration backup
pg_restore -U openhack -d openhack /backups/pre-migration-backup.sql

# 3. Rollback code
git checkout HEAD~1

# 4. Restart services
docker compose up -d

# 5. Verify health
curl http://localhost:8000/health
```

---

## Certificate Renewal

### Let's Encrypt (cert-manager)

**Automatic Renewal (Kubernetes):**
```yaml
# cert-manager handles this automatically
# Certificates renew 30 days before expiry

# Check certificate status
kubectl get certificates -n openhack

# View certificate details
kubectl describe certificate openhack-tls -n openhack
```

**Manual Renewal (if needed):**
```bash
# 1. Install certbot
apt-get install certbot

# 2. Renew certificate
certbot renew --nginx

# 3. Or for standalone
certbot certonly --standalone -d hackathon.example.com

# 4. Reload nginx
nginx -s reload

# 5. Verify
certbot certificates
```

---

### Manual Certificate (Self-Signed or Purchased)

**Renewal Process:**
```bash
# 1. Generate new certificate
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout tls.key -out tls.crt \
  -subj "/CN=hackathon.example.com"

# 2. Create Kubernetes secret
kubectl create secret tls openhack-tls \
  --cert=tls.crt \
  --key=tls.key \
  -n openhack \
  --dry-run=client -o yaml | kubectl apply -f -

# 3. Or copy to Docker volume
cp tls.crt docker/certs/
cp tls.key docker/certs/

# 4. Restart services
docker compose restart gateway
# or
kubectl rollout restart deployment/openhack-gateway -n openhack

# 5. Verify
curl -v https://hackathon.example.com/health
```

---

### Certificate Expiry Monitoring

**Alert Configuration:**
```yaml
# Prometheus alert rule
- alert: CertificateExpiringSoon
  expr: probe_ssl_earliest_cert_expiry - time() < 86400 * 30
  for: 1h
  labels:
    severity: warning
  annotations:
    summary: "SSL certificate expiring in less than 30 days"
```

**Manual Check:**
```bash
# Check certificate expiry
echo | openssl s_client -connect hackathon.example.com:443 2>/dev/null | \
  openssl x509 -noout -dates

# Or use certbot
certbot certificates
```

---

## Backup Verification

### Daily Backup Verification

**Automated Script:**
```bash
#!/bin/bash
# scripts/verify-backup.sh

BACKUP_DIR="/backups/daily"
LATEST=$(ls -t $BACKUP_DIR/*.sql | head -1)

# 1. Check backup exists and is recent
if [ -z "$LATEST" ]; then
    echo "ERROR: No backup found"
    exit 1
fi

if [ $(find $LATEST -mtime +1) ]; then
    echo "ERROR: Backup is older than 1 day"
    exit 1
fi

# 2. Check backup size is reasonable
SIZE=$(stat -c%s "$LATEST")
if [ $SIZE -lt 1000000 ]; then
    echo "ERROR: Backup suspiciously small (<1MB)"
    exit 1
fi

# 3. Test restore in isolated container
docker run --rm -v $LATEST:/backup.sql -v $BACKUP_DIR:/backups \
  postgres:16 pg_restore -h postgres -U openhack -d openhack_test /backup.sql

if [ $? -ne 0 ]; then
    echo "ERROR: Restore test failed"
    exit 1
fi

echo "SUCCESS: Backup verified"
```

---

### Weekly Restore Test

**Procedure:**
```bash
# 1. Create test database
docker compose run postgres createdb -U openhack openhack_test

# 2. Restore latest backup
pg_restore -h localhost -U openhack -d openhack_test \
  /backups/daily/latest.sql

# 3. Run integrity checks
psql -h localhost -U openhack -d openhack_test <<EOF
-- Check row counts
SELECT 'auth.users' as table_name, COUNT(*) as row_count FROM auth.users
UNION ALL
SELECT 'core.teams', COUNT(*) FROM core.teams
UNION ALL
SELECT 'core.projects', COUNT(*) FROM core.projects;

-- Check foreign key integrity
SELECT COUNT(*) as orphaned_scores
FROM judging.scores s
LEFT JOIN judging.assignments a ON s.assignment_id = a.id
WHERE a.id IS NULL;

-- Check index health
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
EOF

# 4. Drop test database
docker compose run postgres dropdb -U openhack openhack_test
```

---

### Monthly Disaster Recovery Test

**Full DR Test Procedure:**
```bash
# 1. Spin up fresh environment
mkdir /tmp/dr-test
cd /tmp/dr-test
curl -fsSL https://openhack.dev/install.sh | bash

# 2. Restore from backup
docker compose exec postgres pg_restore -U openhack -d openhack \
  /backups/monthly/latest.sql

# 3. Verify all services
for service in auth core judging leaderboard mail notify ai analytics sponsors media; do
    curl -f http://localhost:8000/health || exit 1
done

# 4. Test key user flows
# - Register new user
# - Create team
# - Submit project
# - Cast vote

# 5. Document results
echo "DR Test: $(date)" >> /var/log/dr-tests.log
echo "Result: SUCCESS" >> /var/log/dr-tests.log

# 6. Cleanup
rm -rf /tmp/dr-test
```

---

## Maintenance Log Template

```markdown
# Maintenance Log

## [Date] - [Maintenance Type]

**Window:** [Start Time] - [End Time] UTC  
**Duration:** [X hours Y minutes]  
**Type:** Scheduled / Unscheduled / Emergency  
**Severity:** SEV1 / SEV2 / SEV3 / SEV4

### Team
- **Incident Commander:** [Name]
- **Technical Lead:** [Name]
- **On-Call:** [Name]

### Changes Made
1. [Change 1]
2. [Change 2]
3. [Change 3]

### Issues Encountered
- [Issue 1 and resolution]
- [Issue 2 and resolution]

### Verification
- [ ] All services healthy
- [ ] Key user flows tested
- [ ] Metrics normal
- [ ] No error spikes

### Rollback Performed
[If applicable, describe rollback]

### Follow-up Actions
- [ ] [Action 1] - Owner: [Name] - Due: [Date]
- [ ] [Action 2] - Owner: [Name] - Due: [Date]

### Notes
[Any additional observations or lessons learned]
```

---

**Last Updated:** May 13, 2026  
**Review Schedule:** Quarterly  
**Owner:** DevOps Team
