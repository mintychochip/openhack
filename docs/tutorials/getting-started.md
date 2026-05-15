# Getting Started with OpenHack

**Time:** 5-15 minutes  
**Level:** Beginner  
**Prerequisites:** Docker 24+, Docker Compose 2.20+

---

## 5-Minute Quickstart

### Step 1: Install OpenHack

```bash
# Clone the repository
git clone https://github.com/opencodeai/openhack.git
cd openhack

# Copy environment file
cp .env.example .env

# Start all services
docker compose up -d

# Wait for services to be healthy (2-3 minutes)
docker compose ps
```

**Expected Output:**
```
NAME                STATUS              PORTS
openhack-gateway    Up (healthy)        0.0.0.0:8000->8000/tcp
openhack-auth-svc   Up (healthy)        3001/tcp
openhack-core-svc   Up (healthy)        3002/tcp
openhack-postgres   Up (healthy)        5432/tcp
openhack-redis      Up (healthy)        6379/tcp
```

---

### Step 2: Access the Platform

**Frontend:** http://localhost:3000  
**API Gateway:** http://localhost:8000  
**API Documentation:** http://localhost:8000/docs

---

### Step 3: Create Your First Account

```bash
# Register a new user
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "admin123",
    "name": "Admin User"
  }'

# Login
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "admin123"
  }'
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4...",
  "expires_in": 900
}
```

---

### Step 4: Make Yourself Admin

```bash
# Get your user ID from the registration response
USER_ID="your-user-id-here"

# Promote to admin
curl -X POST http://localhost:8000/api/auth/users/$USER_ID/role \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "admin"
  }'
```

---

### Step 5: Create Your First Hackathon

```bash
# Configure hackathon settings
curl -X POST http://localhost:8000/api/core/hackathon \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My First Hackathon",
    "description": "Welcome to my hackathon!",
    "start_date": "2024-06-01T00:00:00Z",
    "end_date": "2024-06-03T23:59:59Z",
    "registration_open": true
  }'
```

---

## First Hackathon Setup

### Complete Setup Checklist

**1. Configure Hackathon (10 min)**
- [ ] Set hackathon name and dates
- [ ] Write description and rules
- [ ] Upload logo and banner
- [ ] Configure registration settings
- [ ] Set timezone

**2. Set Up Judging (15 min)**
- [ ] Create judging rubric
- [ ] Define criteria and weights
- [ ] Add judges (assign judge role)
- [ ] Configure judging phases
- [ ] Set up auto-assignment

**3. Configure Events (10 min)**
- [ ] Add opening ceremony
- [ ] Schedule workshops
- [ ] Plan closing ceremony
- [ ] Enable RSVP tracking

**4. Set Up Prizes (10 min)**
- [ ] Add sponsor booths
- [ ] Create prize tiers
- [ ] Configure winner selection

**5. Test User Flow (10 min)**
- [ ] Register as participant
- [ ] Create team
- [ ] Submit project
- [ ] Cast vote
- [ ] View leaderboard

---

### Detailed Setup Steps

#### 1. Create Judging Rubric

```bash
curl -X POST http://localhost:8000/api/judging/rubrics \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Standard Rubric",
    "description": "Default judging criteria",
    "criteria": [
      {"name": "Innovation", "weight": 30, "description": "How innovative is the project?"},
      {"name": "Technical Difficulty", "weight": 30, "description": "Technical complexity"},
      {"name": "Presentation", "weight": 20, "description": "Quality of presentation"},
      {"name": "Completeness", "weight": 20, "description": "How complete is the project?"}
    ]
  }'
```

---

#### 2. Assign Judges

```bash
# First, promote users to judge role
curl -X POST http://localhost:8000/api/auth/users/JUDGE_USER_ID/role \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -d '{"role": "judge"}'

# Auto-assign judges to projects
curl -X POST http://localhost:8000/api/judging/assignments/bulk \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "judge_ids": ["judge-1", "judge-2", "judge-3"],
    "project_ids": ["project-1", "project-2", "project-3"],
    "mode": "random",
    "assignments_per_judge": 3
  }'
```

---

#### 3. Configure Leaderboard

```bash
# Set ranking formula
curl -X PUT http://localhost:8000/api/leaderboard/admin/formula \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "formula": "0.7 * judging_score + 0.3 * public_votes",
    "description": "70% judge score, 30% public votes"
  }'

# Enable public voting
curl -X PUT http://localhost:8000/api/leaderboard/admin/voting \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -d '{
    "enabled": true,
    "max_votes_per_user": 3,
    "start_time": "2024-06-03T12:00:00Z",
    "end_time": "2024-06-03T23:00:00Z"
  }'
```

---

## User Roles Guide

### Participant

**Permissions:**
- ✅ Register and login
- ✅ Create/join teams (up to 4 members)
- ✅ Submit projects
- ✅ RSVP to events
- ✅ Vote on projects (if enabled)
- ✅ View leaderboard
- ✅ Chat with AI assistant

**Cannot:**
- ❌ Judge projects
- ❌ Access admin dashboard
- ❌ Modify hackathon settings
- ❌ Manage sponsors

---

### Judge

**Permissions:**
- ✅ All participant permissions
- ✅ View assigned projects
- ✅ Submit scores using rubric
- ✅ View judging dashboard
- ✅ See score history
- ✅ Access judge-only chat

**Cannot:**
- ❌ Access admin dashboard
- ❌ Modify hackathon settings
- ❌ Manage sponsors

---

### Sponsor

**Permissions:**
- ✅ Create booth page
- ✅ Add prizes
- ✅ Review submissions for prizes
- ✅ Select winners
- ✅ Access sponsor dashboard

**Cannot:**
- ❌ Judge projects (unless also judge role)
- ❌ Access admin dashboard
- ❌ Modify hackathon settings

---

### Admin

**Permissions:**
- ✅ All permissions
- ✅ Manage users (create, update, delete, assign roles)
- ✅ Configure hackathon settings
- ✅ Manage judging rubrics and phases
- ✅ Configure leaderboard formulas
- ✅ Manage events
- ✅ View analytics dashboard
- ✅ Send announcements
- ✅ Manage webhooks
- ✅ Access all dashboards

---

## Customization Guide

### Branding

**1. Update Logo and Colors**

Edit `web/src/app/layout.tsx`:
```typescript
export const metadata = {
  title: "My Hackathon",
  description: "The best hackathon ever",
};

// Update theme colors
const theme = {
  primary: "#3B82F6",  // Blue-500
  secondary: "#10B981", // Emerald-500
};
```

**2. Customize Landing Page**

Edit `web/src/app/page.tsx`:
```typescript
export default function LandingPage() {
  return (
    <div>
      <h1>Welcome to My Hackathon!</h1>
      <p>Join us for 48 hours of coding fun</p>
    </div>
  );
}
```

---

### Feature Toggles

**Enable/Disable Features in `.env`:**
```bash
# AI Features
AI_ENABLED=true
AI_CHAT=true
AI_IDEA_GENERATOR=true
AI_TEAM_MATCHER=false
AI_CODE_REVIEW=false

# Public Voting
VOTING_ENABLED=true
VOTING_MAX_PER_USER=3

# OAuth Providers
GITHUB_ENABLED=true
GOOGLE_ENABLED=false
DISCORD_ENABLED=true

# Email
EMAIL_ENABLED=true
EMAIL_PROVIDER=smtp  # or sendgrid, ses
```

---

### Custom Email Templates

**1. Edit Template**

```bash
curl -X PUT http://localhost:8000/api/mail/admin/templates/welcome \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "Welcome to {{hackathon_name}}!",
    "body": "Hi {{name}},\n\nWelcome to {{hackathon_name}}! Get ready for an amazing event.\n\nBest,\nThe Team"
  }'
```

**2. Available Variables:**
- `{{name}}` - User's name
- `{{email}}` - User's email
- `{{hackathon_name}}` - Hackathon name
- `{{hackathon_dates}}` - Event dates
- `{{team_name}}` - Team name (if applicable)

---

### Custom OAuth

**Add New OAuth Provider:**

1. **Register application with provider** (GitHub, Google, etc.)
2. **Get client ID and secret**
3. **Add to `.env`:**
```bash
NEW_PROVIDER_CLIENT_ID=your-client-id
NEW_PROVIDER_CLIENT_SECRET=your-client-secret
NEW_PROVIDER_ENABLED=true
```

4. **Update auth service code** to handle new provider

---

## Next Steps

**Documentation:**
- [Operational Runbooks](./runbooks/)
- [Advanced Guides](./advanced/)
- [Deployment Guide](../DEPLOYMENT.md)
- [Testing Guide](../TESTING.md)

**Support:**
- GitHub Issues: https://github.com/opencodeai/openhack/issues
- Discord: https://discord.gg/openhack
- Documentation: https://docs.openhack.dev

---

**Last Updated:** May 13, 2026
