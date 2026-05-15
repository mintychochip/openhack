# Webhook Integration Guide

**Purpose:** Configure outbound webhooks to integrate OpenHack with external services (Discord, Slack, custom apps).

---

## Overview

OpenHack sends real-time webhook notifications for key events:
- User registration
- Team creation
- Project submission
- Score submission
- Winner announcement

**Features:**
- HMAC signature for security
- Automatic retries (3 attempts)
- Dead letter queue for failures
- Event filtering (subscribe to specific events)
- Delivery logging

---

## Quick Start: Discord Notifications

### Step 1: Create Discord Webhook

1. Go to your Discord server
2. Edit a channel → Integrations → Webhooks
3. Click "New Webhook"
4. Copy the webhook URL

### Step 2: Configure in OpenHack

```bash
curl -X POST http://localhost:8000/api/notify/admin/webhooks \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Discord Notifications",
    "url": "https://discord.com/api/webhooks/YOUR_WEBHOOK_URL",
    "events": [
      "user.registered",
      "team.created",
      "project.submitted"
    ],
    "active": true
  }'
```

### Step 3: Test Webhook

```bash
curl -X POST http://localhost:8000/api/notify/admin/webhooks/WEBHOOK_ID/test \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

---

## Quick Start: Slack Notifications

### Step 1: Create Slack App

1. Go to https://api.slack.com/apps
2. Click "Create New App"
3. Add "Incoming Webhooks" feature
4. Activate and copy webhook URL

### Step 2: Configure in OpenHack

```bash
curl -X POST http://localhost:8000/api/notify/admin/webhooks \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Slack Announcements",
    "url": "https://hooks.slack.com/services/YOUR/WEBHOOK/URL",
    "events": ["announcement.sent"],
    "active": true
  }'
```

---

## Webhook Configuration

### Create Webhook via API

**Endpoint:** `POST /api/notify/admin/webhooks`

**Request:**
```json
{
  "name": "My Webhook",
  "url": "https://example.com/webhook",
  "events": [
    "user.registered",
    "project.submitted"
  ],
  "active": true,
  "secret": "your-hmac-secret",
  "headers": {
    "X-Custom-Header": "value"
  },
  "retry_count": 3,
  "timeout": 30
}
```

**Response:**
```json
{
  "id": "webhook-123",
  "name": "My Webhook",
  "url": "https://example.com/webhook",
  "events": ["user.registered", "project.submitted"],
  "active": true,
  "created_at": "2024-01-15T10:30:00Z"
}
```

---

### List Webhooks

**Endpoint:** `GET /api/notify/admin/webhooks`

**Response:**
```json
{
  "webhooks": [
    {
      "id": "webhook-123",
      "name": "Discord Notifications",
      "url": "https://discord.com/api/webhooks/...",
      "events": ["user.registered"],
      "active": true,
      "delivery_count": 150,
      "failure_count": 2
    }
  ],
  "total": 1
}
```

---

### Update Webhook

**Endpoint:** `PUT /api/notify/admin/webhooks/:id`

```json
{
  "name": "Updated Name",
  "active": false
}
```

---

### Delete Webhook

**Endpoint:** `DELETE /api/notify/admin/webhooks/:id`

---

## Event Reference

### User Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `user.registered` | New user registration | `{user_id, email, name, provider}` |
| `user.login` | User logs in | `{user_id, email, timestamp}` |
| `user.updated` | Profile updated | `{user_id, changes}` |

### Team Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `team.created` | New team created | `{team_id, name, creator_id, members}` |
| `team.updated` | Team details changed | `{team_id, changes}` |
| `team.joined` | User joins team | `{team_id, user_id, joined_at}` |
| `team.left` | User leaves team | `{team_id, user_id, left_at}` |

### Project Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `project.submitted` | Project submitted | `{project_id, team_id, title, submitted_at}` |
| `project.updated` | Project edited | `{project_id, changes}` |
| `project.deleted` | Project deleted | `{project_id, team_id, deleted_at}` |

### Judging Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `score.submitted` | Judge submits score | `{assignment_id, project_id, judge_id, scores}` |
| `assignment.created` | Judge assigned | `{assignment_id, judge_id, project_id}` |
| `phase.started` | Judging phase begins | `{phase_id, name, started_at}` |
| `phase.ended` | Judging phase ends | `{phase_id, name, ended_at}` |

### Leaderboard Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `vote.cast` | User votes | `{vote_id, project_id, voter_id}` |
| `rankings.updated` | Leaderboard recalculated | `{timestamp, top_3}` |
| `snapshot.frozen` | Leaderboard frozen | `{snapshot_id, frozen_at}` |

### System Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `announcement.sent` | Broadcast sent | `{announcement_id, recipient_count}` |
| `webhook.failed` | Webhook delivery failed | `{webhook_id, event, error}` |

---

## Payload Format

### Standard Format

```json
{
  "id": "evt_abc123",
  "type": "user.registered",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "user_id": "user-123",
    "email": "jane@example.com",
    "name": "Jane Doe",
    "provider": "github"
  },
  "metadata": {
    "hackathon_id": "hack-456",
    "ip_address": "192.168.1.1"
  }
}
```

---

## Security: HMAC Signatures

### Configuring HMAC

When creating a webhook, set a `secret`:

```bash
curl -X POST http://localhost:8000/api/notify/admin/webhooks \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -d '{
    "name": "Secure Webhook",
    "url": "https://example.com/webhook",
    "events": ["user.registered"],
    "secret": "your-secret-key-here"
  }'
```

### Verifying Signatures

**OpenHack sends:**
- `X-OpenHack-Signature`: `sha256=abc123...`
- `X-OpenHack-Timestamp`: Unix timestamp

**Verify in your code:**

**Node.js Example:**
```javascript
const crypto = require('crypto');

function verifySignature(payload, signature, secret) {
  const expected = crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');
  
  return crypto.timingSafeEqual(
    Buffer.from(`sha256=${expected}`),
    Buffer.from(signature)
  );
}

// In your webhook handler
app.post('/webhook', (req, res) => {
  const signature = req.headers['x-openhack-signature'];
  const timestamp = req.headers['x-openhack-timestamp'];
  const payload = JSON.stringify(req.body);
  
  // Check timestamp (prevent replay attacks)
  const age = Date.now() / 1000 - parseInt(timestamp);
  if (age > 300) {
    return res.status(400).send('Timestamp too old');
  }
  
  // Verify signature
  if (!verifySignature(payload, signature, WEBHOOK_SECRET)) {
    return res.status(401).send('Invalid signature');
  }
  
  // Process webhook
  console.log('Webhook received:', req.body);
  res.send('OK');
});
```

**Python Example:**
```python
import hmac
import hashlib
import time
from flask import Flask, request, abort

app = Flask(__name__)

def verify_signature(payload, signature, secret):
    expected = hmac.new(
        secret.encode(),
        payload.encode(),
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(f'sha256={expected}', signature)

@app.route('/webhook', methods=['POST'])
def webhook():
    signature = request.headers.get('X-OpenHack-Signature')
    timestamp = request.headers.get('X-OpenHack-Timestamp')
    payload = request.get_data(as_text=True)
    
    # Check timestamp
    age = time.time() - int(timestamp)
    if age > 300:
        abort(400, 'Timestamp too old')
    
    # Verify signature
    if not verify_signature(payload, signature, WEBHOOK_SECRET):
        abort(401, 'Invalid signature')
    
    # Process webhook
    data = request.json
    print(f'Webhook received: {data}')
    
    return 'OK'
```

---

## Retry Logic

### Automatic Retries

OpenHack automatically retries failed webhooks:

| Attempt | Delay | Backoff |
|---------|-------|---------|
| 1 | Immediate | - |
| 2 | 1 second | 1x |
| 3 | 5 seconds | 5x |
| 4 | 30 seconds | 6x |

After 3 retries (4 total attempts), the webhook is marked as failed and logged.

---

### Dead Letter Queue

Failed webhooks are stored in the dead letter queue for manual inspection.

**View Failed Webhooks:**
```bash
curl http://localhost:8000/api/notify/admin/webhooks/failed \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Retry Failed Webhook:**
```bash
curl -X POST http://localhost:8000/api/notify/admin/webhooks/failed/WEBHOOK_ID/retry \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

---

## Monitoring

### Delivery Statistics

**Endpoint:** `GET /api/notify/admin/webhooks/:id/stats`

**Response:**
```json
{
  "webhook_id": "webhook-123",
  "total_deliveries": 150,
  "successful": 148,
  "failed": 2,
  "success_rate": 98.67,
  "avg_response_time_ms": 245,
  "last_delivery": {
    "timestamp": "2024-01-15T10:30:00Z",
    "status": 200,
    "response_time_ms": 198
  }
}
```

---

### Delivery Logs

**Endpoint:** `GET /api/notify/admin/webhooks/:id/deliveries`

**Query Parameters:**
- `status`: Filter by status code (200, 500, etc.)
- `limit`: Number of records (default: 50)
- `offset`: Pagination offset

**Response:**
```json
{
  "deliveries": [
    {
      "id": "delivery-789",
      "webhook_id": "webhook-123",
      "event": "user.registered",
      "status": 200,
      "response_time_ms": 198,
      "attempt": 1,
      "timestamp": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 150
}
```

---

## Best Practices

### 1. Use HTTPS

Always use HTTPS for webhook URLs to encrypt data in transit.

### 2. Validate Timestamps

Prevent replay attacks by checking the `X-OpenHack-Timestamp` header.

### 3. Respond Quickly

Return `200 OK` immediately. Process webhook data asynchronously.

### 4. Idempotency

Handle duplicate webhook deliveries gracefully using the event `id`.

### 5. Monitor Failures

Set up alerts for high webhook failure rates.

---

## Troubleshooting

### Webhook Not Receiving Events

**Check:**
1. Webhook is active: `GET /api/notify/admin/webhooks`
2. Subscribed to correct events
3. URL is publicly accessible (not localhost)
4. Firewall allows incoming requests

### HMAC Verification Failing

**Check:**
1. Secret matches exactly (no extra spaces)
2. Using correct encoding (UTF-8)
3. Comparing full signature string (`sha256=...`)

### High Failure Rate

**Check:**
1. Endpoint is healthy and responding
2. Response timeout is sufficient (default: 30s)
3. SSL certificate is valid
4. Server has capacity to handle requests

---

**Last Updated:** May 13, 2026  
**See Also:** [Notification Service](../../PROJECT.md#notification-service), [Admin Dashboard](../../PROJECT.md#admin-dashboard)
