# OpenHack API Documentation

**Complete API reference for all OpenHack services.**

---

## API Gateway

**Base URL:** `http://localhost:8000`  
**Swagger UI:** `http://localhost:8000/docs`  
**ReDoc:** `http://localhost:8000/redoc`

All API requests are routed through Kong Gateway to appropriate services.

---

## Auth Service (`/api/auth`)

**Port:** 3001  
**Swagger:** `http://localhost:3001/docs`

### Authentication Endpoints

#### `POST /api/auth/register`
Register a new user account.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "name": "John Doe",
  "github_username": "johndoe"
}
```

**Response:**
```json
{
  "user_id": "uuid",
  "email": "user@example.com",
  "email_verification_sent": true
}
```

---

#### `POST /api/auth/login`
Login with email and password.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4...",
  "expires_in": 900
}
```

---

#### `POST /api/auth/oauth/:provider`
Initiate OAuth login flow.

**Providers:** `github`, `google`, `discord`

**Response:** `302` redirect to OAuth provider

---

#### `POST /api/auth/refresh`
Refresh access token.

**Request:**
```json
{
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4..."
}
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 900
}
```

---

#### `POST /api/auth/logout`
Logout and invalidate session.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "logged_out": true
}
```

---

### Password Management

#### `POST /api/auth/forgot-password`
Request password reset email.

**Request:**
```json
{
  "email": "user@example.com"
}
```

**Response:**
```json
{
  "reset_email_sent": true
}
```

---

#### `POST /api/auth/reset-password`
Reset password with token.

**Request:**
```json
{
  "token": "reset-token-from-email",
  "new_password": "NewSecurePass123!"
}
```

**Response:**
```json
{
  "password_reset": true
}
```

---

### User Profile

#### `GET /api/auth/me`
Get current user profile.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "id": "uuid",
  "email": "user@example.com",
  "name": "John Doe",
  "avatar_url": "https://...",
  "roles": ["participant"],
  "created_at": "2024-01-01T00:00:00Z"
}
```

---

#### `PUT /api/auth/me`
Update user profile.

**Headers:** `Authorization: Bearer <token>`

**Request:**
```json
{
  "name": "Jane Doe",
  "avatar_url": "https://...",
  "github_username": "janedoe"
}
```

**Response:**
```json
{
  "updated": true,
  "user": { ... }
}
```

---

### MFA (Multi-Factor Authentication)

#### `POST /api/auth/me/mfa/enable`
Enable TOTP MFA.

**Response:**
```json
{
  "qr_code_url": "data:image/png;base64,...",
  "backup_codes": ["ABC123", "DEF456", ...]
}
```

---

#### `POST /api/auth/me/mfa/verify`
Verify TOTP code.

**Request:**
```json
{
  "code": "123456"
}
```

**Response:**
```json
{
  "mfa_enabled": true
}
```

---

#### `POST /api/auth/me/mfa/disable`
Disable MFA.

**Request:**
```json
{
  "code": "123456",
  "password": "CurrentPassword123!"
}
```

**Response:**
```json
{
  "mfa_disabled": true
}
```

---

#### `POST /api/auth/me/mfa/sms/send`
Send SMS verification code.

**Request:**
```json
{
  "phone_number": "+1234567890"
}
```

**Response:**
```json
{
  "sms_sent": true
}
```

---

### Admin Endpoints

#### `GET /api/auth/users`
List all users (admin only).

**Query Params:** `?page=1&limit=50&role=participant`

**Response:**
```json
{
  "users": [...],
  "total": 100,
  "page": 1
}
```

---

#### `DELETE /api/auth/users/:id`
Delete user (admin only).

**Response:**
```json
{
  "deleted": true
}
```

---

#### `POST /api/auth/users/:id/role`
Update user role (admin only).

**Request:**
```json
{
  "role": "judge"
}
```

**Response:**
```json
{
  "role_updated": true
}
```

---

## Core Service (`/api/core`)

**Port:** 3002  
**Swagger:** `http://localhost:3002/docs`

### Teams

#### `GET /api/core/teams`
List all teams.

**Response:**
```json
{
  "teams": [
    {
      "id": "uuid",
      "name": "Team Awesome",
      "description": "We build awesome stuff",
      "members": [...],
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 50
}
```

---

#### `POST /api/core/teams`
Create a new team.

**Request:**
```json
{
  "name": "Team Awesome",
  "description": "We build awesome stuff"
}
```

**Response:**
```json
{
  "team_id": "uuid",
  "created": true
}
```

---

#### `GET /api/core/teams/:id`
Get team details.

**Response:**
```json
{
  "id": "uuid",
  "name": "Team Awesome",
  "members": [
    {
      "user_id": "uuid",
      "name": "John Doe",
      "role": "captain"
    }
  ],
  "projects": [...]
}
```

---

#### `DELETE /api/core/teams/:id`
Delete team.

**Response:**
```json
{
  "deleted": true
}
```

---

### Projects

#### `GET /api/core/projects`
List all projects.

**Query Params:** `?status=submitted&team_id=uuid`

**Response:**
```json
{
  "projects": [
    {
      "id": "uuid",
      "title": "Awesome Project",
      "description": "An awesome project description",
      "team_id": "uuid",
      "status": "submitted",
      "repo_url": "https://github.com/...",
      "demo_url": "https://...",
      "submitted_at": "2024-01-03T00:00:00Z"
    }
  ],
  "total": 30
}
```

---

#### `POST /api/core/projects`
Create a new project.

**Request:**
```json
{
  "title": "Awesome Project",
  "description": "An awesome project description",
  "repo_url": "https://github.com/...",
  "demo_url": "https://...",
  "track": "web3"
}
```

**Response:**
```json
{
  "project_id": "uuid",
  "created": true
}
```

---

#### `PUT /api/core/projects/:id`
Update project.

**Request:**
```json
{
  "title": "Updated Title",
  "description": "Updated description",
  "status": "submitted"
}
```

**Response:**
```json
{
  "updated": true,
  "project": { ... }
}
```

---

#### `DELETE /api/core/projects/:id`
Delete project.

**Response:**
```json
{
  "deleted": true
}
```

---

### Events

#### `GET /api/core/events`
List all events.

**Response:**
```json
{
  "events": [
    {
      "id": "uuid",
      "name": "Opening Ceremony",
      "description": "Welcome to the hackathon!",
      "start_time": "2024-01-01T09:00:00Z",
      "end_time": "2024-01-01T10:00:00Z",
      "location": "Main Hall",
      "rsvp_count": 150
    }
  ],
  "total": 20
}
```

---

#### `POST /api/core/events/:id/rsvp`
RSVP to an event.

**Response:**
```json
{
  "rsvpd": true,
  "event_id": "uuid"
}
```

---

### Hackathon Configuration

#### `GET /api/core/hackathon`
Get hackathon configuration.

**Response:**
```json
{
  "name": "Hackathon 2024",
  "description": "The best hackathon ever",
  "start_date": "2024-01-01T00:00:00Z",
  "end_date": "2024-01-03T23:59:59Z",
  "registration_open": true,
  "max_team_size": 4,
  "themes": ["web3", "ai", "sustainability"]
}
```

---

#### `PUT /api/core/hackathon`
Update hackathon configuration (admin only).

**Request:**
```json
{
  "name": "Updated Name",
  "registration_open": false,
  "max_team_size": 5
}
```

**Response:**
```json
{
  "updated": true,
  "hackathon": { ... }
}
```

---

## Judging Service (`/api/judging`)

**Port:** 3003  
**Swagger:** `http://localhost:3003/docs`

### Rubrics

#### `POST /api/judging/rubrics`
Create judging rubric.

**Request:**
```json
{
  "name": "Standard Rubric",
  "description": "Default judging criteria",
  "criteria": [
    {
      "name": "Innovation",
      "weight": 30,
      "description": "How innovative is the project?"
    },
    {
      "name": "Technical Difficulty",
      "weight": 30,
      "description": "Technical complexity"
    }
  ]
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "Standard Rubric",
  "criteria": [...],
  "created_at": "2024-01-01T00:00:00Z"
}
```

---

#### `GET /api/judging/rubrics`
List all rubrics.

**Response:**
```json
{
  "rubrics": [...],
  "total": 5
}
```

---

### Assignments

#### `POST /api/judging/assignments`
Create single assignment.

**Request:**
```json
{
  "judge_id": "uuid",
  "project_id": "uuid",
  "rubric_id": "uuid"
}
```

**Response:**
```json
{
  "id": "uuid",
  "judge_id": "uuid",
  "project_id": "uuid",
  "status": "pending"
}
```

---

#### `POST /api/judging/assignments/bulk`
Bulk assign judges to projects.

**Request:**
```json
{
  "judge_ids": ["uuid1", "uuid2"],
  "project_ids": ["uuid1", "uuid2", "uuid3"],
  "mode": "random",
  "assignments_per_judge": 3
}
```

**Response:**
```json
{
  "assignments": [...],
  "total": 6
}
```

---

#### `GET /api/judging/assignments`
List assignments.

**Query Params:** `?judge_id=uuid&status=pending`

**Response:**
```json
{
  "assignments": [...],
  "total": 10
}
```

---

### Scores

#### `POST /api/judging/scores`
Submit score for an assignment.

**Request:**
```json
{
  "assignment_id": "uuid",
  "scores": {
    "Innovation": 85,
    "Technical Difficulty": 90,
    "Presentation": 75
  },
  "comments": "Great project!"
}
```

**Response:**
```json
{
  "id": "uuid",
  "assignment_id": "uuid",
  "total_score": 250,
  "submitted_at": "2024-01-03T00:00:00Z"
}
```

---

#### `GET /api/judging/scores`
List scores.

**Query Params:** `?project_id=uuid&judge_id=uuid`

**Response:**
```json
{
  "scores": [...],
  "total": 20
}
```

---

### Dashboard

#### `GET /api/judging/dashboard`
Get judging dashboard stats.

**Response:**
```json
{
  "total_assignments": 30,
  "completed_assignments": 25,
  "pending_assignments": 5,
  "average_score": 82.5,
  "completion_rate": 83.33
}
```

---

### Phases

#### `POST /api/judging/phases`
Create judging phase.

**Request:**
```json
{
  "name": "Preliminary",
  "order": 1,
  "start_date": "2024-01-02T00:00:00Z",
  "end_date": "2024-01-03T00:00:00Z"
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "Preliminary",
  "order": 1,
  "status": "active"
}
```

---

#### `GET /api/judging/phases`
List all phases.

**Response:**
```json
{
  "phases": [...],
  "total": 3
}
```

---

### Normalization

#### `POST /api/judging/normalize`
Recalculate scores with normalization.

**Request:**
```json
{
  "method": "zscore",
  "phase_id": "uuid"
}
```

**Response:**
```json
{
  "normalized": true,
  "scores_updated": 50
}
```

---

## Leaderboard Service (`/api/leaderboard`)

**Port:** 3004  
**Swagger:** `http://localhost:3004/docs`

### Rankings

#### `GET /api/leaderboard/rankings`
Get current leaderboard rankings.

**Query Params:** `?limit=10&category=overall`

**Response:**
```json
{
  "rankings": [
    {
      "rank": 1,
      "project_id": "uuid",
      "project_name": "Awesome Project",
      "team_name": "Team Awesome",
      "score": 95.5,
      "normalized_score": 92.3
    }
  ],
  "total": 30,
  "last_updated": "2024-01-03T12:00:00Z"
}
```

---

#### `GET /api/leaderboard/stats`
Get leaderboard statistics.

**Response:**
```json
{
  "total_projects": 30,
  "average_score": 82.5,
  "median_score": 85.0,
  "highest_score": 98.5,
  "lowest_score": 65.0
}
```

---

### Voting

#### `POST /api/leaderboard/votes`
Cast a vote for a project.

**Request:**
```json
{
  "project_id": "uuid"
}
```

**Response:**
```json
{
  "voted": true,
  "vote_id": "uuid",
  "remaining_votes": 2
}
```

---

#### `GET /api/leaderboard/votes`
Get vote count for a project.

**Query Params:** `?project_id=uuid`

**Response:**
```json
{
  "project_id": "uuid",
  "vote_count": 150
}
```

---

### Snapshots

#### `POST /api/leaderboard/snapshots`
Freeze leaderboard snapshot.

**Request:**
```json
{
  "name": "Final Rankings",
  "reason": "Competition ended"
}
```

**Response:**
```json
{
  "snapshot_id": "uuid",
  "frozen_at": "2024-01-03T23:59:59Z",
  "rankings": [...]
}
```

---

#### `GET /api/leaderboard/snapshots`
List leaderboard snapshots.

**Response:**
```json
{
  "snapshots": [...],
  "total": 5
}
```

---

### Formulas

#### `PUT /api/leaderboard/admin/formula`
Update ranking formula.

**Request:**
```json
{
  "formula": "0.7 * judging_score + 0.3 * public_votes",
  "description": "70% judge score, 30% public votes"
}
```

**Response:**
```json
{
  "updated": true,
  "formula": "0.7 * judging_score + 0.3 * public_votes"
}
```

---

## Mail Service (`/api/mail`)

**Port:** 3005  
**Swagger:** `http://localhost:3005/docs`

### Send Email

#### `POST /api/mail/send`
Send a single email.

**Request:**
```json
{
  "to": "user@example.com",
  "subject": "Welcome!",
  "body": "Hello!",
  "template_id": "welcome",
  "variables": {
    "name": "John Doe"
  }
}
```

**Response:**
```json
{
  "message_id": "uuid",
  "sent": true
}
```

---

#### `POST /api/mail/broadcast`
Send broadcast email to all users.

**Request:**
```json
{
  "subject": "Important Announcement",
  "template_id": "announcement",
  "variables": {
    "message": "Event starts tomorrow!"
  }
}
```

**Response:**
```json
{
  "broadcast_id": "uuid",
  "recipients": 500,
  "queued": true
}
```

---

### Templates

#### `GET /api/mail/templates`
List all email templates.

**Response:**
```json
{
  "templates": [
    {
      "id": "welcome",
      "name": "Welcome Email",
      "subject": "Welcome to {{hackathon_name}}!",
      "variables": ["name", "hackathon_name"]
    }
  ],
  "total": 8
}
```

---

#### `PUT /api/mail/templates/:id`
Update email template.

**Request:**
```json
{
  "subject": "Updated Subject",
  "body": "Updated body content"
}
```

**Response:**
```json
{
  "updated": true,
  "template": { ... }
}
```

---

#### `POST /api/mail/templates/:id/test`
Send test email with template.

**Request:**
```json
{
  "email": "test@example.com",
  "variables": {
    "name": "Test User"
  }
}
```

**Response:**
```json
{
  "sent": true,
  "message_id": "uuid"
}
```

---

## Notification Service (`/api/notify`)

**Port:** 3006  
**Swagger:** `http://localhost:3006/docs`

### Discord

#### `POST /api/notify/discord/send`
Send Discord message.

**Request:**
```json
{
  "channel_id": "123456789",
  "message": "Hello from OpenHack!",
  "embed": {
    "title": "Announcement",
    "color": 3447003
  }
}
```

**Response:**
```json
{
  "sent": true,
  "message_id": "987654321"
}
```

---

### Slack

#### `POST /api/notify/slack/send`
Send Slack message.

**Request:**
```json
{
  "channel": "#announcements",
  "text": "Hello from OpenHack!",
  "blocks": [...]
}
```

**Response:**
```json
{
  "sent": true,
  "ts": "1234567890.123456"
}
```

---

### Webhooks

#### `POST /api/notify/webhooks`
Register new webhook.

**Request:**
```json
{
  "name": "My Webhook",
  "url": "https://example.com/webhook",
  "events": ["user.registered", "project.submitted"],
  "secret": "hmac-secret"
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "My Webhook",
  "created_at": "2024-01-01T00:00:00Z"
}
```

---

#### `GET /api/notify/webhooks`
List all webhooks.

**Response:**
```json
{
  "webhooks": [...],
  "total": 5
}
```

---

## AI Service (`/api/ai`)

**Port:** 3007  
**Swagger:** `http://localhost:3007/docs`

### Chat

#### `POST /api/ai/chat`
Chat with AI assistant.

**Request:**
```json
{
  "message": "How do I submit my project?",
  "conversation_id": "uuid"
}
```

**Response:**
```json
{
  "response": "To submit your project, go to...",
  "conversation_id": "uuid",
  "sources": ["Knowledge Base Article #123"]
}
```

---

#### `GET /api/ai/conversations`
Get conversation history.

**Response:**
```json
{
  "conversations": [...],
  "total": 10
}
```

---

### Idea Generator

#### `POST /api/ai/ideas`
Generate project ideas.

**Request:**
```json
{
  "theme": "sustainability",
  "technologies": ["react", "python"],
  "skill_level": "intermediate"
}
```

**Response:**
```json
{
  "ideas": [
    {
      "name": "EcoTracker",
      "description": "Track your carbon footprint",
      "technologies": ["React", "Python", "PostgreSQL"],
      "difficulty": 3
    }
  ]
}
```

---

### Team Matcher

#### `POST /api/ai/teams/suggest`
Get teammate suggestions.

**Request:**
```json
{
  "user_id": "uuid",
  "skills": ["frontend", "design"],
  "looking_for": ["backend", "devops"]
}
```

**Response:**
```json
{
  "suggestions": [
    {
      "user_id": "uuid",
      "name": "Jane Doe",
      "skills": ["backend", "devops"],
      "match_score": 95
    }
  ]
}
```

---

### Code Review

#### `POST /api/ai/review`
Get AI code review.

**Request:**
```json
{
  "code": "def hello():\n    print('world')",
  "language": "python"
}
```

**Response:**
```json
{
  "feedback": "Good function! Consider adding docstrings...",
  "issues": [],
  "suggestions": ["Add type hints", "Add unit tests"]
}
```

---

### Knowledge Base

#### `POST /api/ai/knowledge`
Upload knowledge base document.

**Request:** `multipart/form-data`

**Fields:**
- `file`: PDF/TXT/MD file
- `title`: Document title
- `category`: rules/faq/schedule
- `tags`: Comma-separated tags

**Response:**
```json
{
  "knowledge_id": "uuid",
  "uploaded": true,
  "chunks_created": 15
}
```

---

#### `DELETE /api/ai/knowledge/:id`
Delete knowledge base document.

**Response:**
```json
{
  "deleted": true
}
```

---

## Analytics Service (`/api/analytics`)

**Port:** 3008  
**Swagger:** `http://localhost:3008/docs`

### Metrics

#### `GET /api/analytics/metrics`
Get current metrics.

**Response:**
```json
{
  "total_registrations": 500,
  "total_teams": 120,
  "total_projects": 80,
  "total_submissions": 75,
  "active_users": 350,
  "last_updated": "2024-01-03T12:00:00Z"
}
```

---

#### `GET /api/analytics/metrics/over-time`
Get metrics over time.

**Query Params:** `?metric=registrations&start=2024-01-01&end=2024-01-03`

**Response:**
```json
{
  "metrics": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "value": 100
    },
    {
      "timestamp": "2024-01-02T00:00:00Z",
      "value": 300
    }
  ]
}
```

---

### Charts

#### `GET /api/analytics/charts/registrations`
Get registration chart data.

**Response:**
```json
{
  "labels": ["Day 1", "Day 2", "Day 3"],
  "datasets": [
    {
      "label": "Registrations",
      "data": [100, 300, 500]
    }
  ]
}
```

---

#### `GET /api/analytics/charts/submissions`
Get submission chart data.

**Response:**
```json
{
  "labels": ["Hour 1", "Hour 2", "Hour 3"],
  "datasets": [
    {
      "label": "Submissions",
      "data": [10, 25, 40]
    }
  ]
}
```

---

### Reports

#### `GET /api/analytics/reports`
Generate analytics report.

**Query Params:** `?type=daily&date=2024-01-03`

**Response:**
```json
{
  "report_id": "uuid",
  "type": "daily",
  "date": "2024-01-03",
  "metrics": {...},
  "charts": {...}
}
```

---

#### `GET /api/analytics/export`
Export analytics data.

**Query Params:** `?format=csv&metric=registrations`

**Response:** CSV file download

---

## Sponsors Service (`/api/sponsors`)

**Port:** 3009  
**Swagger:** `http://localhost:3009/docs`

### Booths

#### `GET /api/sponsors/booths`
List all sponsor booths.

**Response:**
```json
{
  "booths": [
    {
      "id": "uuid",
      "sponsor_id": "uuid",
      "name": "Company Booth",
      "description": "Visit our booth!",
      "tier": "gold",
      "published": true
    }
  ],
  "total": 10
}
```

---

#### `POST /api/sponsors/booths`
Create sponsor booth.

**Request:**
```json
{
  "name": "Company Booth",
  "description": "Visit our booth!",
  "tier": "gold"
}
```

**Response:**
```json
{
  "booth_id": "uuid",
  "created": true
}
```

---

#### `PUT /api/sponsors/booths/:id/publish`
Publish/unpublish booth.

**Response:**
```json
{
  "published": true,
  "booth_id": "uuid"
}
```

---

### Prizes

#### `GET /api/sponsors/prizes`
List all prizes.

**Query Params:** `?sponsor_id=uuid&status=available`

**Response:**
```json
{
  "prizes": [
    {
      "id": "uuid",
      "name": "Best Overall",
      "description": "Grand prize",
      "value": 1000,
      "currency": "USD",
      "awarded": false
    }
  ],
  "total": 15
}
```

---

#### `POST /api/sponsors/prizes`
Create prize.

**Request:**
```json
{
  "name": "Best Overall",
  "description": "Grand prize",
  "value": 1000,
  "currency": "USD"
}
```

**Response:**
```json
{
  "prize_id": "uuid",
  "created": true
}
```

---

#### `POST /api/sponsors/prizes/:id/winner`
Select prize winner.

**Request:**
```json
{
  "project_id": "uuid"
}
```

**Response:**
```json
{
  "winner_selected": true,
  "project_id": "uuid"
}
```

---

### Submissions

#### `GET /api/sponsors/submissions`
List prize submissions.

**Query Params:** `?prize_id=uuid&status=pending`

**Response:**
```json
{
  "submissions": [
    {
      "id": "uuid",
      "project_id": "uuid",
      "prize_id": "uuid",
      "status": "pending"
    }
  ],
  "total": 20
}
```

---

#### `POST /api/sponsors/submissions`
Submit project for prize.

**Request:**
```json
{
  "prize_id": "uuid",
  "project_id": "uuid"
}
```

**Response:**
```json
{
  "submission_id": "uuid",
  "submitted": true
}
```

---

#### `PUT /api/sponsors/submissions/:id/approve`
Approve submission.

**Response:**
```json
{
  "approved": true,
  "submission_id": "uuid"
}
```

---

#### `PUT /api/sponsors/submissions/:id/reject`
Reject submission.

**Request:**
```json
{
  "reason": "Does not meet criteria"
}
```

**Response:**
```json
{
  "rejected": true,
  "submission_id": "uuid"
}
```

---

## Media Service (`/api/media`)

**Port:** 3010  
**Swagger:** `http://localhost:3010/docs`

### Upload

#### `POST /api/media/upload`
Upload file.

**Request:** `multipart/form-data`

**Fields:**
- `file`: File to upload
- `folder`: Optional folder name

**Response:**
```json
{
  "file_id": "uuid",
  "url": "https://cdn.example.com/file.pdf",
  "size": 1024,
  "content_type": "application/pdf",
  "storage_provider": "minio"
}
```

---

#### `GET /api/media/:file_id`
Download file.

**Response:** File stream

---

#### `DELETE /api/media/:file_id`
Delete file.

**Response:**
```json
{
  "deleted": true
}
```

---

#### `GET /api/media/:file_id/url`
Get signed URL.

**Query Params:** `?expires_in=3600`

**Response:**
```json
{
  "signed_url": "https://...",
  "expires_at": "2024-01-03T13:00:00Z"
}
```

---

#### `POST /api/media/files`
Create file metadata record.

**Request:**
```json
{
  "filename": "file.pdf",
  "content_type": "application/pdf",
  "size": 1024,
  "folder": "documents",
  "metadata": {
    "custom_field": "value"
  }
}
```

**Response:**
```json
{
  "file_id": "uuid",
  "url": "https://...",
  "created_at": "2024-01-01T00:00:00Z"
}
```

---

#### `PUT /api/media/files/:file_id`
Update file metadata.

**Request:**
```json
{
  "filename": "updated.pdf",
  "metadata": {
    "custom_field": "updated"
  }
}
```

**Response:**
```json
{
  "updated": true,
  "file": { ... }
}
```

---

## Health Check

#### `GET /health`
Check API gateway health.

**Response:**
```json
{
  "status": "healthy",
  "services": {
    "auth": "healthy",
    "core": "healthy",
    "judging": "healthy",
    "leaderboard": "healthy",
    "mail": "healthy",
    "notify": "healthy",
    "ai": "healthy",
    "analytics": "healthy",
    "sponsors": "healthy",
    "media": "healthy"
  }
}
```

---

## Rate Limits

| Service | Requests/Minute |
|---------|-----------------|
| Auth | 100 |
| Core | 300 |
| Judging | 100 |
| Leaderboard | 300 |
| Mail | 50 |
| Notify | 50 |
| AI | 30 |
| Analytics | 100 |
| Sponsors | 50 |
| Media | 50 |

---

## Authentication

All authenticated endpoints require JWT token in `Authorization` header:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Token expiry: 15 minutes (use refresh token to get new access token)

---

## Error Responses

### 400 Bad Request
```json
{
  "error": "Bad Request",
  "message": "Invalid input",
  "details": [...]
}
```

### 401 Unauthorized
```json
{
  "error": "Unauthorized",
  "message": "Invalid or expired token"
}
```

### 403 Forbidden
```json
{
  "error": "Forbidden",
  "message": "Insufficient permissions"
}
```

### 404 Not Found
```json
{
  "error": "Not Found",
  "message": "Resource not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal Server Error",
  "message": "An unexpected error occurred"
}
```

---

**Last Updated:** May 13, 2026  
**Total Endpoints:** 190+  
**Services:** 11
