# Multi-Tenancy Setup

**Purpose:** Configure OpenHack to support multiple independent hackathons (tenants) on a single installation.

---

## Overview

Multi-tenancy allows you to:
- Run multiple hackathons on one infrastructure
- Isolate data per hackathon
- Custom branding per tenant
- Separate user bases
- Independent configurations

**Tenancy Models:**
1. **Database-per-tenant** - Full isolation (recommended for enterprise)
2. **Schema-per-tenant** - Logical isolation (recommended for most)
3. **Row-level isolation** - Shared tables with tenant_id (not recommended)

---

## Schema-per-Tenant Setup (Recommended)

### Architecture

```
┌─────────────────────────────────────────┐
│         Shared Infrastructure           │
│  (PostgreSQL, Redis, Services, Gateway) │
└─────────────────────────────────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
   ┌─────────┐ ┌─────────┐ ┌─────────┐
   │hack_2024│ │hack_2025│ │hack_spring│
   │ schema  │ │ schema  │ │  schema   │
   └─────────┘ └─────────┘ └─────────┘
```

---

### Step 1: Enable Multi-Tenancy

Edit `.env`:
```bash
MULTI_TENANCY_ENABLED=true
TENANCY_MODEL=schema  # schema or database
DEFAULT_TENANT=default
```

---

### Step 2: Create Tenant Schema

```bash
curl -X POST http://localhost:8000/api/core/admin/tenants \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Hackathon 2024",
    "slug": "hack-2024",
    "schema": "hack_2024",
    "config": {
      "domain": "hack2024.example.com",
      "branding": {
        "logo_url": "https://cdn.example.com/hack2024-logo.png",
        "primary_color": "#3B82F6",
        "secondary_color": "#10B981"
      },
      "features": {
        "judging": true,
        "voting": true,
        "ai_assistant": true
      }
    }
  }'
```

---

### Step 3: Initialize Tenant Schema

```bash
# Run migrations for new tenant
curl -X POST http://localhost:8000/api/core/admin/tenants/hack-2024/migrate \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Or via CLI:**
```bash
./scripts/migrate --tenant hack-2024
```

---

### Step 4: Configure Domain

**Option A: Subdomain**
```bash
# DNS Configuration
hack2024.example.com  CNAME  openhack.example.com
hack2025.example.com  CNAME  openhack.example.com
```

**Option B: Path-based**
```
example.com/hack-2024/*
example.com/hack-2025/*
```

---

### Step 5: Update Gateway

Edit `docker/kong/kong.yml`:
```yaml
plugins:
  - name: tenant-resolution
    config:
      header: X-Tenant-Slug
      query_param: tenant
      cookie: tenant_id
      default_tenant: default
      
  - name: rate-limiting
    config:
      minute: 100
      policy: redis
      redis_host: redis
      redis_port: 6379
```

---

## Tenant Resolution

### How It Works

1. **Request arrives at gateway**
2. **Gateway extracts tenant from:**
   - `X-Tenant-Slug` header
   - `?tenant=` query parameter
   - `tenant` cookie
   - Subdomain (e.g., `hack2024.example.com`)
3. **Gateway sets `X-Tenant-Schema` header**
4. **Services route to correct schema**

---

### Service Implementation

**Database Connection:**
```python
# services/core/db/connection.py

async def get_tenant_session(tenant_schema: str) -> AsyncSession:
    """Get database session for specific tenant schema."""
    
    engine = create_async_engine(
        DATABASE_URL,
        connect_args={"options": f"-c search_path={tenant_schema}"}
    )
    
    async_session = async_sessionmaker(
        engine,
        class_=AsyncSession,
        expire_on_commit=False,
    )
    
    return async_session()
```

**Middleware:**
```python
# services/core/middleware/tenant.py

@app.middleware("http")
async def tenant_middleware(request: Request, call_next):
    # Extract tenant from header
    tenant_schema = request.headers.get("X-Tenant-Schema", "public")
    
    # Set tenant in request state
    request.state.tenant_schema = tenant_schema
    
    # Continue with request
    response = await call_next(request)
    
    return response
```

---

## Tenant Configuration

### Configuration Options

```json
{
  "tenant": {
    "name": "Hackathon 2024",
    "slug": "hack-2024",
    "schema": "hack_2024",
    "status": "active",
    "created_at": "2024-01-01T00:00:00Z"
  },
  "config": {
    "domain": "hack2024.example.com",
    "timezone": "America/New_York",
    "locale": "en-US",
    
    "branding": {
      "logo_url": "https://cdn.example.com/logo.png",
      "favicon_url": "https://cdn.example.com/favicon.ico",
      "primary_color": "#3B82F6",
      "secondary_color": "#10B981",
      "banner_url": "https://cdn.example.com/banner.png"
    },
    
    "features": {
      "registration": true,
      "teams": true,
      "projects": true,
      "judging": true,
      "voting": true,
      "ai_assistant": true,
      "analytics": true,
      "sponsors": true
    },
    
    "limits": {
      "max_teams": 100,
      "max_team_size": 4,
      "max_projects": 50,
      "max_judges": 20,
      "storage_gb": 50
    },
    
    "oauth": {
      "github_enabled": true,
      "google_enabled": false,
      "discord_enabled": true
    },
    
    "email": {
      "enabled": true,
      "from_address": "noreply@hack2024.example.com",
      "support_address": "support@hack2024.example.com"
    }
  }
}
```

---

## Tenant Management API

### List Tenants

```bash
curl http://localhost:8000/api/core/admin/tenants \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Response:**
```json
{
  "tenants": [
    {
      "id": "tenant-123",
      "name": "Hackathon 2024",
      "slug": "hack-2024",
      "schema": "hack_2024",
      "status": "active",
      "created_at": "2024-01-01T00:00:00Z",
      "user_count": 150,
      "team_count": 45
    }
  ],
  "total": 1
}
```

---

### Get Tenant

```bash
curl http://localhost:8000/api/core/admin/tenants/hack-2024 \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

---

### Update Tenant

```bash
curl -X PUT http://localhost:8000/api/core/admin/tenants/hack-2024 \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "branding": {
        "primary_color": "#EF4444"
      },
      "features": {
        "ai_assistant": false
      }
    }
  }'
```

---

### Delete Tenant

```bash
curl -X DELETE http://localhost:8000/api/core/admin/tenants/hack-2024 \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Warning:** This deletes all tenant data!

---

### Archive Tenant

```bash
curl -X POST http://localhost:8000/api/core/admin/tenants/hack-2024/archive \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

Archived tenants are read-only.

---

## Custom Domains

### SSL/TLS Configuration

**Automatic (Let's Encrypt):**
```yaml
# Kubernetes Ingress
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: hack-2024-ingress
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
    - hosts:
        - hack2024.example.com
      secretName: hack-2024-tls
  rules:
    - host: hack2024.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: openhack-gateway
                port:
                  number: 8000
```

**Manual Certificate:**
```bash
# Generate certificate
openssl req -x509 -nodes -days 365 \
  -newkey rsa:2048 \
  -keyout hack2024.key \
  -out hack2024.crt \
  -subj "/CN=hack2024.example.com"

# Create Kubernetes secret
kubectl create secret tls hack-2024-tls \
  --cert=hack2024.crt \
  --key=hack2024.key \
  -n openhack
```

---

### DNS Configuration

**Subdomain approach:**
```dns
hack2024.example.com.  CNAME  openhack.example.com.
hack2025.example.com.  CNAME  openhack.example.com.
```

**Custom domain approach:**
```dns
myhackathon.com.  A  192.168.1.100
www.myhackathon.com.  CNAME  myhackathon.com.
```

---

## Data Isolation

### Query Isolation

All queries automatically include tenant schema:

```sql
-- Without multi-tenancy
SELECT * FROM auth.users;

-- With multi-tenancy (automatic)
SET search_path TO hack_2024;
SELECT * FROM auth.users;

-- Or explicit
SELECT * FROM hack_2024.auth.users;
```

---

### Cross-Tenant Queries (Admin Only)

```sql
-- Aggregate across all tenants
SELECT 
  schemaname,
  COUNT(*) as user_count
FROM pg_tables
WHERE schemaname LIKE 'hack_%'
GROUP BY schemaname;
```

---

## Tenant-Specific Customization

### Branding

Each tenant can have unique:
- Logo
- Color scheme
- Banner images
- Favicon
- Email templates

### Features

Enable/disable per tenant:
- Judging system
- Public voting
- AI assistant
- Sponsor booths
- Analytics

### Limits

Set per-tenant limits:
- Maximum users
- Maximum teams
- Maximum projects
- Storage quota
- API rate limits

---

## Monitoring Per-Tenant

### Metrics

```promql
# Requests per tenant
sum(rate(http_requests_total{tenant="hack-2024"}[5m]))

# Database connections per tenant
pg_stat_activity_count{datname="hack_2024"}

# Storage used per tenant
sum(tenant_storage_bytes{tenant="hack-2024"})
```

---

### Logs

```bash
# Filter logs by tenant
docker compose logs auth-svc | grep "tenant=hack-2024"

# Or via Loki
{tenant="hack-2024"} |= "error"
```

---

## Best Practices

### 1. Schema Naming

Use consistent naming:
```
hack_<year>_<season>
# Examples:
hack_2024_spring
hack_2024_fall
hack_2025_spring
```

### 2. Tenant Cleanup

Archive (don't delete) old tenants:
```bash
# Archive instead of delete
curl -X POST http://localhost:8000/api/core/admin/tenants/hack-2024/archive \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

### 3. Resource Quotas

Set limits per tenant:
```yaml
# Kubernetes ResourceQuota
apiVersion: v1
kind: ResourceQuota
metadata:
  name: hack-2024-quota
  namespace: openhack
spec:
  hard:
    requests.cpu: "2"
    requests.memory: 4Gi
    limits.cpu: "4"
    limits.memory: 8Gi
```

### 4. Backup Per-Tenant

```bash
# Backup specific tenant
pg_dump -h localhost -U openhack \
  --schema=hack_2024 \
  -f hack-2024-backup.sql
```

### 5. Testing

Test in staging before production:
```bash
# Create test tenant
curl -X POST http://staging.openhack.example.com/api/core/admin/tenants \
  -d '{"name": "Test", "slug": "test"}'
```

---

## Troubleshooting

### Tenant Not Found

**Check:**
1. Tenant exists: `GET /api/core/admin/tenants`
2. Schema created in database
3. Gateway routing correctly
4. `X-Tenant-Schema` header set

### Cross-Tenant Data Leak

**Verify:**
1. `search_path` set correctly per request
2. No hardcoded schema names in queries
3. Middleware applying tenant isolation
4. Tests for isolation

### Performance Issues

**Solutions:**
1. Index tenant-specific queries
2. Monitor per-tenant resource usage
3. Set appropriate resource quotas
4. Archive old tenants

---

**Last Updated:** May 13, 2026  
**See Also:** [Deployment Guide](../../DEPLOYMENT.md), [Configuration Reference](../../PROJECT.md#configuration-reference)
