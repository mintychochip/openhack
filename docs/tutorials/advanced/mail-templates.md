# Custom Mail Templates

**Purpose:** Customize email templates for user notifications, announcements, and automated emails.

---

## Template System Overview

OpenHack uses **Jinja2** templating engine for email templates.

**Features:**
- Variable substitution (`{{name}}`)
- Conditional logic (`{% if %}`)
- Loops (`{% for %}`)
- Template inheritance (`{% extends %}`)
- Filters (`{{ name|upper }}`)

---

## Pre-built Templates

### 1. Welcome Email

**Template ID:** `welcome`  
**Trigger:** User registration  
**Variables:** `name`, `email`, `hackathon_name`, `hackathon_dates`

**Default Template:**
```jinja2
{% extends "base.html" %}

{% block subject %}Welcome to {{ hackathon_name }}, {{ name }}!{% endblock %}

{% block content %}
<h1>Welcome to {{ hackathon_name }}!</h1>

<p>Hi {{ name }},</p>

<p>We're excited to have you join us for {{ hackathon_name }}!</p>

<p><strong>Event Details:</strong></p>
<ul>
  <li><strong>Dates:</strong> {{ hackathon_dates }}</li>
  <li><strong>Location:</strong> Virtual & In-Person</li>
  <li><strong>Registration:</strong> Confirmed</li>
</ul>

<h2>Next Steps:</h2>
<ol>
  <li><a href="{{ dashboard_url }}">Complete your profile</a></li>
  <li><a href="{{ teams_url }}">Create or join a team</a></li>
  <li><a href="{{ events_url }}">RSVP to events</a></li>
</ol>

<p>If you have any questions, reply to this email or join our <a href="{{ discord_url }}">Discord server</a>.</p>

<p>See you at the hackathon!</p>

<p>Best regards,<br>
The {{ hackathon_name }} Team</p>
{% endblock %}
```

---

### 2. Password Reset

**Template ID:** `password_reset`  
**Trigger:** User requests password reset  
**Variables:** `name`, `reset_link`, `expiry_hours`

**Default Template:**
```jinja2
{% extends "base.html" %}

{% block subject %}Reset Your Password{% endblock %}

{% block content %}
<h1>Password Reset Request</h1>

<p>Hi {{ name }},</p>

<p>We received a request to reset your password. Click the link below to reset it:</p>

<p><a href="{{ reset_link }}" style="background-color: #3B82F6; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">Reset Password</a></p>

<p><strong>This link expires in {{ expiry_hours }} hours.</strong></p>

<p>If you didn't request this, you can safely ignore this email.</p>

<p>Best regards,<br>
The Team</p>
{% endblock %}
```

---

### 3. Team Invitation

**Template ID:** `team_invite`  
**Trigger:** User invited to join team  
**Variables:** `invitee_name`, `inviter_name`, `team_name`, `accept_link`, `decline_link`

**Default Template:**
```jinja2
{% extends "base.html" %}

{% block subject %}You're invited to join {{ team_name }}!{% endblock %}

{% block content %}
<h1>Team Invitation</h1>

<p>Hi {{ invitee_name }},</p>

<p><strong>{{ inviter_name }}</strong> has invited you to join their team: <strong>{{ team_name }}</strong></p>

<div style="text-align: center; margin: 30px;">
  <a href="{{ accept_link }}" style="background-color: #10B981; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; margin: 10px;">Accept Invitation</a>
  
  <a href="{{ decline_link }}" style="background-color: #EF4444; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; margin: 10px;">Decline</a>
</div>

<p>The invitation expires in 48 hours.</p>

<p>Good luck!</p>

<p>Best regards,<br>
The Team</p>
{% endblock %}
```

---

### 4. Project Submission Confirmation

**Template ID:** `project_submission`  
**Trigger:** Project submitted  
**Variables:** `user_name`, `project_name`, `team_name`, `submission_url`

**Default Template:**
```jinja2
{% extends "base.html" %}

{% block subject %}Project Submitted: {{ project_name }}{% endblock %}

{% block content %}
<h1>Project Submitted!</h1>

<p>Hi {{ user_name }},</p>

<p>Your project <strong>{{ project_name }}</strong> has been successfully submitted!</p>

<p><strong>Team:</strong> {{ team_name }}</p>
<p><strong>Submission Time:</strong> {{ submission_time }}</p>

<p>You can view your submission here: <a href="{{ submission_url }}">{{ submission_url }}</a></p>

<h2>What's Next?</h2>
<ul>
  <li>Judging will begin on {{ judging_start_date }}</li>
  <li>Public voting opens on {{ voting_start_date }}</li>
  <li>Winners announced on {{ announcement_date }}</li>
</ul>

<p>Good luck!</p>

<p>Best regards,<br>
The Team</p>
{% endblock %}
```

---

### 5. Judge Assignment

**Template ID:** `judge_assignment`  
**Trigger:** Judge assigned to projects  
**Variables:** `judge_name`, `assignment_count`, `dashboard_url`

**Default Template:**
```jinja2
{% extends "base.html" %}

{% block subject %}You have {{ assignment_count }} projects to judge{% endblock %}

{% block content %}
<h1>Judge Assignment</h1>

<p>Hi {{ judge_name }},</p>

<p>You've been assigned <strong>{{ assignment_count }} projects</strong> to judge.</p>

<p><a href="{{ dashboard_url }}" style="background-color: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px;">Go to Judging Dashboard</a></p>

<h2>Judging Guidelines:</h2>
<ul>
  <li>Review each project thoroughly</li>
  <li>Use the provided rubric for scoring</li>
  <li>Provide constructive feedback</li>
  <li>Complete scoring by {{ deadline }}</li>
</ul>

<p>Thank you for volunteering as a judge!</p>

<p>Best regards,<br>
The Judging Team</p>
{% endblock %}
```

---

### 6. Announcement (Broadcast)

**Template ID:** `announcement`  
**Trigger:** Admin sends announcement  
**Variables:** `subject`, `message`, `sender_name`

**Default Template:**
```jinja2
{% extends "base.html" %}

{% block subject %}{{ subject }}{% endblock %}

{% block content %}
<h1>Announcement</h1>

<p>Hi there,</p>

{{ message|safe }}

<p>Best regards,<br>
{{ sender_name }}</p>

<hr>
<p style="font-size: 12px; color: #666;">
  You received this email because you're registered for the hackathon.
  <a href="{{ unsubscribe_url }}">Unsubscribe</a>
</p>
{% endblock %}
```

---

## Editing Templates

### Via Admin Dashboard (Recommended)

**1. Access Template Editor**

Navigate to: `https://hackathon.example.com/dashboard/admin/mail`

**2. Select Template**

Choose from the list of templates.

**3. Edit Content**

- Update subject line
- Modify HTML body
- Insert variables using the variable inserter

**4. Preview**

Click "Preview" to see how it renders with sample data.

**5. Test Send**

Enter your email and click "Send Test".

**6. Save**

Click "Save Template" to apply changes.

---

### Via API

**Get Template:**
```bash
curl http://localhost:8000/api/mail/admin/templates/welcome \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN"
```

**Update Template:**
```bash
curl -X PUT http://localhost:8000/api/mail/admin/templates/welcome \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "Welcome to {{hackathon_name}}!",
    "body": "<h1>Welcome!</h1><p>Hi {{name}}, welcome to {{hackathon_name}}!</p>"
  }'
```

**List All Templates:**
```bash
curl http://localhost:8000/api/mail/admin/templates \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN"
```

---

## Variable Reference

### Global Variables

Available in all templates:

| Variable | Description | Example |
|----------|-------------|---------|
| `hackathon_name` | Hackathon name | "HackMIT 2024" |
| `hackathon_dates` | Event dates | "September 14-16, 2024" |
| `hackathon_url` | Hackathon website | "https://hackmit.org" |
| `support_email` | Support email | "support@hackmit.org" |
| `discord_url` | Discord invite | "https://discord.gg/hackmit" |

### User Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `name` | User's full name | "Jane Doe" |
| `email` | User's email | "jane@example.com" |
| `username` | Username | "janedoe" |

### Context-Specific Variables

**Team Templates:**
| Variable | Description |
|----------|-------------|
| `team_name` | Team name |
| `team_members` | List of team members |
| `inviter_name` | Person who sent invite |

**Project Templates:**
| Variable | Description |
|----------|-------------|
| `project_name` | Project title |
| `project_description` | Project description |
| `submission_time` | When project was submitted |

**Judging Templates:**
| Variable | Description |
|----------|-------------|
| `assignment_count` | Number of assigned projects |
| `deadline` | Judging deadline |
| `rubric_name` | Name of judging rubric |

---

## Jinja2 Syntax Guide

### Variable Substitution

```jinja2
{{ name }}           → Jane Doe
{{ name|upper }}     → JANE DOE
{{ name|lower }}     → jane doe
{{ count|default(0) }} → 0 (if count is undefined)
```

### Conditional Logic

```jinja2
{% if is_admin %}
  <p>You have admin access.</p>
{% elif is_judge %}
  <p>You have judge access.</p>
{% else %}
  <p>You have participant access.</p>
{% endif %}
```

### Loops

```jinja2
<ul>
{% for member in team_members %}
  <li>{{ member.name }} - {{ member.role }}</li>
{% endfor %}
</ul>
```

### Filters

```jinja2
{{ name|length }}           → 8
{{ email|upper }}           → JANE@EXAMPLE.COM
{{ date|date("Y-m-d") }}    → 2024-09-14
{{ description|truncate(50) }} → "This is a long description..."
```

---

## Best Practices

### 1. Use Template Inheritance

**base.html:**
```jinja2
<!DOCTYPE html>
<html>
<head>
  <style>
    body { font-family: Arial, sans-serif; }
    .button { background-color: #3B82F6; color: white; }
  </style>
</head>
<body>
  {% block content %}{% endblock %}
</body>
</html>
```

**welcome.html:**
```jinja2
{% extends "base.html" %}

{% block content %}
  <h1>Welcome!</h1>
  <p>Hi {{ name }}!</p>
{% endblock %}
```

---

### 2. Keep Templates Responsive

```html
<table role="presentation" style="width: 100%; max-width: 600px;">
  <tr>
    <td style="padding: 20px;">
      <!-- Content here -->
    </td>
  </tr>
</table>
```

---

### 3. Test Across Email Clients

- Use email testing tools (Litmus, Email on Acid)
- Test in Gmail, Outlook, Apple Mail
- Check mobile rendering
- Verify dark mode compatibility

---

### 4. Avoid Spam Triggers

- Don't use all caps in subject
- Avoid excessive punctuation (!!!)
- Include unsubscribe link
- Don't use spammy words (FREE, $$$)

---

## Testing Templates

### Local Testing

**1. Use Mailhog for development**
```yaml
# docker-compose.yml
mailhog:
  image: mailhog/mailhog
  ports:
    - "1025:1025"  # SMTP
    - "8025:8025"  # Web UI
```

**2. Send test email**
```bash
curl -X POST http://localhost:8000/api/mail/admin/templates/welcome/test \
  -H "Authorization: Bearer TOKEN" \
  -d '{"email": "test@example.com"}'
```

**3. View in Mailhog**
Visit http://localhost:8025

---

### Production Testing

**1. Send to multiple test accounts**
- Gmail
- Outlook
- Yahoo
- Corporate email

**2. Check deliverability**
- Use tools like Mail-Tester.com
- Check SPF/DKIM records
- Monitor spam complaints

---

**Last Updated:** May 13, 2026  
**See Also:** [Getting Started](../getting-started.md), [Admin Dashboard](../../PROJECT.md#admin-dashboard)
