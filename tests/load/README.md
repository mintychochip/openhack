# Load Testing with k6

**Purpose:** Performance and load testing for OpenHack services.

---

## Prerequisites

```bash
# Install k6
# macOS
brew install k6

# Linux
curl https://dl.k6.io/key.gpg | gpg --dearmor -o /usr/share/keyrings/k6-archive-keyring.gpg
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | tee /etc/apt/sources.list.d/k6.list
apt-get update && apt-get install k6

# Windows
winget install k6
```

---

## Quick Start

### 1. Start OpenHack

```bash
docker compose up -d
```

### 2. Run Load Test

```bash
# Basic load test (5 minutes)
k6 run tests/load/load-test.js

# With custom base URL
BASE_URL=http://localhost:8000 k6 run tests/load/load-test.js

# With more virtual users
K6_VUS=100 k6 run tests/load/load-test.js
```

### 3. Run Stress Test

```bash
# Stress test (push to breaking point)
k6 run tests/load/stress-test.js
```

---

## Test Types

### Load Test (`load-test.js`)

**Purpose:** Verify system handles expected load

**Pattern:**
- Ramp up to 50 users → hold
- Ramp up to 100 users → hold
- Ramp up to 200 users → hold
- Spike to 500 users → hold
- Ramp down

**Thresholds:**
- 50% of requests < 500ms
- 95% of requests < 1s
- 99% of requests < 2s
- Error rate < 5%

---

### Stress Test (`stress-test.js`)

**Purpose:** Find breaking point

**Pattern:**
- Ramp to 100 → 200 → 500 → 1000 → 2000 users
- Monitor when system fails

**Thresholds:**
- 95% of requests < 2s
- 99% of requests < 5s
- Error rate < 10%

---

### Soak Test (Endurance)

**Purpose:** Verify no memory leaks over time

```bash
k6 run --duration 1h --vus 50 tests/load/load-test.js
```

---

### Spike Test

**Purpose:** Test sudden traffic surge

```bash
k6 run --vus 10 --duration 30s \
  --vus 500 --duration 1m \
  --vus 10 --duration 30s \
  tests/load/load-test.js
```

---

## Output & Reporting

### Console Output

```bash
k6 run tests/load/load-test.js
```

**Example Output:**
```
     ✓ health check: status is 200
     
     checks.........................: 100.00% ✓ 1000 / 1000
     data_received..................: 2.5 MB  41 kB/s
     data_sent......................: 1.2 MB  20 kB/s
     http_req_duration..............: avg=245ms min=50ms med=200ms p(90)=450ms p(95)=600ms p(99)=950ms
     http_req_failed................: 0.00%   ✓ 0 / 1000
     http_reqs......................: 1000    16.67/s
     iteration_duration.............: avg=1.2s  min=1s med=1.2s p(90)=1.5s p(95)=1.7s p(99)=2s
     iterations.....................: 1000    16.67/s
     vus............................: 50      min=50 max=50
     vus_max........................: 50      min=50 max=50
```

---

### JSON Output

```bash
k6 run --out json=results.json tests/load/load-test.js
```

---

### InfluxDB + Grafana

```bash
# Start InfluxDB
docker run -d -p 8086:8086 influxdb:2.0

# Run k6 with InfluxDB output
k6 run --out influxdb=http://localhost:8086/k6 tests/load/load-test.js

# View in Grafana
# Import k6 dashboard: https://grafana.com/grafana/dashboards/2587
```

---

### Cloud Reporting

```bash
# k6 Cloud (free tier available)
k6 login
k6 run --out cloud tests/load/load-test.js
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: Load Testing

on:
  push:
    branches: [main]

jobs:
  load-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Start services
        run: docker compose up -d
      
      - name: Wait for services
        run: |
          sleep 30
          curl -f http://localhost:8000/health
      
      - name: Install k6
        run: |
          curl https://dl.k6.io/key.gpg | gpg --dearmor -o /usr/share/keyrings/k6-archive-keyring.gpg
          echo "deb [arch=amd64 signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | tee /etc/apt/sources.list.d/k6.list
          apt-get update && apt-get install k6
      
      - name: Run load test
        run: k6 run tests/load/load-test.js
      
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: k6-results
          path: results.json
```

---

## Performance Targets

| Endpoint | P50 | P95 | P99 | Max Errors |
|----------|-----|-----|-----|------------|
| `/health` | 50ms | 100ms | 200ms | 0% |
| `GET /api/*` | 200ms | 500ms | 1s | 1% |
| `POST /api/*` | 300ms | 800ms | 2s | 2% |
| File Upload | 500ms | 2s | 5s | 5% |

---

## Troubleshooting

### High Latency

**Check:**
1. Database connection pool size
2. Redis cache hit rate
3. Service CPU/memory usage
4. Network latency

### High Error Rate

**Check:**
1. Service logs for errors
2. Database connection limits
3. Rate limiting configuration
4. Memory pressure

### Test Fails Thresholds

**Solutions:**
1. Increase resource limits
2. Optimize slow queries
3. Add caching
4. Scale horizontally

---

## Best Practices

1. **Run tests in staging** - Never load test production directly
2. **Start small** - Begin with 10 users, ramp up gradually
3. **Monitor everything** - Use Grafana dashboards
4. **Test regularly** - Add to CI/CD pipeline
5. **Document results** - Keep historical performance data
6. **Test edge cases** - Spike tests, soak tests, chaos tests

---

**Last Updated:** May 13, 2026  
**See Also:** [Monitoring Guide](../docs/runbooks/monitoring.md), [Deployment Guide](../DEPLOYMENT.md)
