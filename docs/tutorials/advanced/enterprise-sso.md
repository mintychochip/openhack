# Enterprise SSO Configuration Guide

## Overview

OpenHack supports three enterprise SSO providers in addition to standard OAuth (GitHub, Google, Discord, etc.):

- **Auth0** - OIDC-compatible identity platform
- **AWS Cognito** - Amazon's user directory service  
- **SAML 2.0** - Standard SSO protocol for enterprise IdPs

## Quick Setup

### 1. Choose Your Provider

Edit `services/auth/.env` and set:

```bash
# For Auth0
AUTH_PROVIDER=auth0

# For AWS Cognito
AUTH_PROVIDER=cognito

# For SAML
AUTH_PROVIDER=saml

# For internal auth (default)
AUTH_PROVIDER=internal
```

### 2. Configure Provider Credentials

#### Auth0

```bash
AUTH0_DOMAIN=your-domain.auth0.com
AUTH0_CLIENT_ID=your-client-id
AUTH0_CLIENT_SECRET=your-client-secret
```

**Auth0 Dashboard Setup:**
1. Create Application (Single Page or Regular Web App)
2. Set Callback URL: `https://your-domain.com/api/auth/sso/auth0/callback`
3. Enable OIDC Conformant flag in Advanced settings
4. Copy Domain, Client ID, and Client Secret to `.env`

#### AWS Cognito

```bash
COGNITO_REGION=us-east-1
COGNITO_USER_POOL_ID=us-east-1_abc123
COGNITO_CLIENT_ID=your-client-id
COGNITO_CLIENT_SECRET=your-client-secret
```

**Cognito Console Setup:**
1. Create User Pool
2. Create App Client (disable SRP for OAuth flow)
3. Set Callback URL: `https://your-domain.com/api/auth/sso/cognito/callback`
4. Copy Region, User Pool ID, Client ID, and Secret to `.env`

#### SAML 2.0

```bash
SAML_ENTRY_POINT=https://idp.example.com/saml
SAML_ISSUER=https://your-domain.com
SAML_CALLBACK_URL=https://your-domain.com/api/auth/sso/saml/callback
SAML_CERT=path/to/cert.pem
SAML_PRIVATE_KEY=path/to/key.pem
```

**IdP Setup:**
1. Get SP Metadata: `GET /api/auth/sso/saml/metadata`
2. Configure IdP with metadata XML
3. Set ACS URL: `https://your-domain.com/api/auth/sso/saml/callback`
4. Set Entity ID: `https://your-domain.com`
5. Map attributes: `email`, `name` (required), `firstName`, `lastName` (optional)
6. Download IdP metadata URL and certificate
7. Copy to `.env`

### 3. Restart Auth Service

```bash
cd services/auth
npm install  # Installs new SSO dependencies
npm run build
npm start
```

Check logs for:
```
🔐 Enterprise SSO: auth0  # or cognito, saml
✅ Auth0 SSO enabled     # confirms provider loaded
```

## Frontend Integration

### Login Button

Add SSO login button to your login page:

```tsx
// Auth0
<a href="/api/auth/sso/auth0">Login with Auth0</a>

// Cognito
<a href="/api/auth/sso/cognito">Login with AWS Cognito</a>

// SAML
<a href="/api/auth/sso/saml">Login with SSO</a>
```

### Callback Handling

All providers redirect to:
```
/frontend-url/auth/callback?access_token=xxx&refresh_token=xxx
```

Same as standard OAuth - no frontend changes needed!

## User Flow

1. User clicks SSO login button
2. Redirected to IdP (Auth0/Cognito/SAML)
3. User authenticates with IdP
4. IdP redirects back with authorization code
5. Auth service exchanges code for tokens
6. Creates/updates user in database
7. Generates JWT tokens
8. Redirects to frontend with tokens

## User Account Linking

- If email already exists, SSO account is **linked** automatically
- If OAuth account exists, user is **logged in** automatically
- New users are **created** with verified email

## Testing

### Auth0
```bash
curl "http://localhost:3001/api/auth/sso/auth0"
# Should redirect to Auth0 login
```

### Cognito
```bash
curl "http://localhost:3001/api/auth/sso/cognito"
# Should redirect to Cognito login
```

### SAML
```bash
# Get SP metadata
curl "http://localhost:3001/api/auth/sso/saml/metadata"

# Test login (requires IdP)
curl "http://localhost:3001/api/auth/sso/saml"
```

## Troubleshooting

### "Provider not configured"
- Check `AUTH_PROVIDER` matches your config
- Verify all required env vars are set
- Check auth service logs for startup errors

### "Invalid callback URL"
- Ensure callback URL in IdP matches exactly
- Include `https://` prefix
- Match domain and port exactly

### "Email attribute not found" (SAML)
- Check IdP attribute mapping
- Common names: `email`, `mail`, `emailAddress`
- Some IdPs use long URN format

### Dependencies not installed
```bash
cd services/auth
npm install
# Should install: @fastify/oauth2, aws-jwt-verify, jwks-rsa, samlify
```

## Security Notes

- All SSO emails are marked as **verified**
- JWT tokens use same security as internal auth
- State parameter prevents CSRF attacks
- Sessions stored in Redis with expiry
- Refresh tokens hashed before storage

## Migration from Internal Auth

Existing users can link SSO accounts:
1. Login with email/password
2. Go to Account Settings
3. Click "Link SSO Account"
4. Authenticate with IdP
5. Future logins work with either method

## Supported SAML Attributes

| Attribute | Required | Common Names |
|-----------|----------|--------------|
| Email | Yes | `email`, `mail`, `emailAddress` |
| Name | No | `name`, `displayName`, `cn` |
| First Name | No | `firstName`, `givenName` |
| Last Name | No | `lastName`, `sn`, `familyName` |

## Rate Limiting

SSO endpoints use same rate limits as other auth routes:
- 100 requests per 15 minutes per IP
- Configurable via `RATE_LIMIT_MAX` and `RATE_LIMIT_WINDOW`

## Monitoring

Check Grafana dashboards:
- **Auth Overview** - SSO login success/failure rates
- **Business Metrics** - New users by provider
- **Services** - Auth service health

Alerts configured for:
- SSO failure rate > 10%
- Auth service downtime
- Token generation failures
