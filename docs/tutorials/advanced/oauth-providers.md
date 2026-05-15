# Custom OAuth Providers

**Purpose:** Add or configure OAuth authentication providers (GitHub, Google, Discord, etc.)

---

## Configuring Built-in Providers

### GitHub OAuth

**1. Register GitHub Application**

1. Go to https://github.com/settings/developers
2. Click "New OAuth App"
3. Fill in:
   - **Application name:** OpenHack
   - **Homepage URL:** https://hackathon.example.com
   - **Authorization callback URL:** https://hackathon.example.com/api/auth/oauth/github/callback
4. Copy **Client ID** and generate **Client Secret**

**2. Configure OpenHack**

Edit `.env`:
```bash
GITHUB_ENABLED=true
GITHUB_CLIENT_ID=your-client-id-here
GITHUB_CLIENT_SECRET=your-client-secret-here
```

**3. Restart Auth Service**
```bash
docker compose restart auth-svc
```

**4. Test**
Visit http://localhost:3000/login and click "Login with GitHub"

---

### Google OAuth

**1. Create Google Cloud Project**

1. Go to https://console.cloud.google.com
2. Create new project or select existing
3. Enable "Google+ API"

**2. Create OAuth Credentials**

1. Go to "APIs & Services" → "Credentials"
2. Click "Create Credentials" → "OAuth client ID"
3. Application type: **Web application**
4. Authorized redirect URIs:
   ```
   https://hackathon.example.com/api/auth/oauth/google/callback
   ```
5. Copy **Client ID** and **Client Secret**

**3. Configure OpenHack**

Edit `.env`:
```bash
GOOGLE_ENABLED=true
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-client-secret-here
```

**4. Restart Auth Service**
```bash
docker compose restart auth-svc
```

---

### Discord OAuth

**1. Create Discord Application**

1. Go to https://discord.com/developers/applications
2. Click "New Application"
3. Go to "OAuth2" section
4. Add redirect:
   ```
   https://hackathon.example.com/api/auth/oauth/discord/callback
   ```
5. Copy **Client ID** and **Client Secret**

**2. Configure OpenHack**

Edit `.env`:
```bash
DISCORD_ENABLED=true
DISCORD_CLIENT_ID=your-client-id-here
DISCORD_CLIENT_SECRET=your-client-secret-here
```

**3. Restart Auth Service**
```bash
docker compose restart auth-svc
```

---

## Adding a New OAuth Provider

### Example: Adding GitLab OAuth

**1. Register GitLab Application**

1. Go to https://gitlab.com/-/profile/applications
2. Click "New application"
3. Set redirect URI:
   ```
   https://hackathon.example.com/api/auth/oauth/gitlab/callback
   ```
4. Select scopes: `read_user`, `email`
5. Copy **Application ID** and **Secret**

---

**2. Add Provider to Auth Service**

Edit `services/auth/src/routes/oauth.ts`:
```typescript
import { fastifyOauth } from '@fastify/oauth2';
import gitlabProvider from '@fastify/oauth2/providers/gitlab';

fastify.register(fastifyOauth, {
  name: 'gitlab',
  credentials: {
    client: {
      id: process.env.GITLAB_CLIENT_ID!,
      secret: process.env.GITLAB_CLIENT_SECRET!,
    },
    auth: gitlabProvider,
  },
  callbackUri: '/api/auth/oauth/gitlab/callback',
});

fastify.get('/api/auth/oauth/gitlab', async (request, reply) => {
  const redirectUrl = fastify.gitlab.generateAuthorizationUrl();
  reply.redirect(redirectUrl);
});

fastify.get('/api/auth/oauth/gitlab/callback', async (request, reply) => {
  const token = await fastify.gitlab.getAccessTokenFromRequestCode(request.query.code);
  
  // Get user info from GitLab
  const userInfo = await fastify.gitlab.getUserInfo(token.token.access_token);
  
  // Create or update user in database
  const user = await authService.findOrCreateUser({
    email: userInfo.email,
    name: userInfo.name,
    avatar_url: userInfo.avatar_url,
    provider: 'gitlab',
    provider_id: userInfo.id,
  });
  
  // Generate JWT tokens
  const tokens = authService.generateTokens(user);
  
  reply.send(tokens);
});
```

---

**3. Add Environment Variables**

Edit `.env.example`:
```bash
GITLAB_ENABLED=false
GITLAB_CLIENT_ID=
GITLAB_CLIENT_SECRET=
```

Edit `services/auth/.env.example`:
```bash
GITLAB_CLIENT_ID=your-gitlab-client-id
GITLAB_CLIENT_SECRET=your-gitlab-client-secret
```

---

**4. Update Frontend**

Edit `web/src/app/login/page.tsx`:
```typescript
<div className="oauth-buttons">
  {config.githubEnabled && (
    <Button onClick={() => signIn('github')}>
      Login with GitHub
    </Button>
  )}
  {config.googleEnabled && (
    <Button onClick={() => signIn('google')}>
      Login with Google
    </Button>
  )}
  {config.gitlabEnabled && (
    <Button onClick={() => signIn('gitlab')}>
      Login with GitLab
    </Button>
  )}
</div>
```

Edit `web/src/lib/config.ts`:
```typescript
export const config = {
  githubEnabled: process.env.NEXT_PUBLIC_GITHUB_ENABLED === 'true',
  googleEnabled: process.env.NEXT_PUBLIC_GOOGLE_ENABLED === 'true',
  gitlabEnabled: process.env.NEXT_PUBLIC_GITLAB_ENABLED === 'true',
  // ...
};
```

---

**5. Add Tests**

Create `services/auth/src/__tests__/oauth-gitlab.test.ts`:
```typescript
import { describe, it, expect, vi } from 'vitest';

describe('GitLab OAuth', () => {
  it('should redirect to GitLab authorization page', async () => {
    const response = await request(app)
      .get('/api/auth/oauth/gitlab')
      .expect(302);
    
    expect(response.headers.location).toContain('gitlab.com/oauth/authorize');
  });

  it('should handle callback and create user', async () => {
    // Mock GitLab user info
    vi.spyOn(fastify.gitlab, 'getAccessTokenFromRequestCode')
      .resolves({ token: { access_token: 'mock-token' } });
    
    vi.spyOn(fastify.gitlab, 'getUserInfo')
      .resolves({
        id: 123,
        email: 'test@gitlab.com',
        name: 'Test User',
        avatar_url: 'https://gitlab.com/avatar.png',
      });

    const response = await request(app)
      .get('/api/auth/oauth/gitlab/callback?code=mock-code')
      .expect(200);

    expect(response.body).toHaveProperty('access_token');
    expect(response.body).toHaveProperty('refresh_token');
  });
});
```

---

**6. Update Documentation**

Add GitLab to `docs/tutorials/advanced/oauth-providers.md` (this file) with setup instructions.

---

## OAuth Configuration Reference

### Environment Variables

| Variable | Provider | Required | Description |
|----------|----------|----------|-------------|
| `GITHUB_ENABLED` | GitHub | No | Enable GitHub OAuth |
| `GITHUB_CLIENT_ID` | GitHub | Yes* | GitHub application client ID |
| `GITHUB_CLIENT_SECRET` | GitHub | Yes* | GitHub application secret |
| `GOOGLE_ENABLED` | Google | No | Enable Google OAuth |
| `GOOGLE_CLIENT_ID` | Google | Yes* | Google OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | Google | Yes* | Google OAuth secret |
| `DISCORD_ENABLED` | Discord | No | Enable Discord OAuth |
| `DISCORD_CLIENT_ID` | Discord | Yes* | Discord application client ID |
| `DISCORD_CLIENT_SECRET` | Discord | Yes* | Discord application secret |

*Required if provider is enabled

---

### Callback URLs

| Provider | Callback URL |
|----------|--------------|
| GitHub | `/api/auth/oauth/github/callback` |
| Google | `/api/auth/oauth/google/callback` |
| Discord | `/api/auth/oauth/discord/callback` |
| GitLab | `/api/auth/oauth/gitlab/callback` |
| Custom | `/api/auth/oauth/:provider/callback` |

---

### Scopes

**Recommended scopes per provider:**

**GitHub:**
- `read:user` - Read user profile
- `user:email` - Read email addresses

**Google:**
- `openid` - OpenID Connect
- `email` - Email address
- `profile` - Basic profile info

**Discord:**
- `identify` - Basic user info
- `email` - Email address

**GitLab:**
- `read_user` - Read user profile
- `email` - Email address

---

## Testing OAuth

### Local Testing

**1. Use ngrok for localhost tunneling**
```bash
# Install ngrok
npm install -g ngrok

# Start tunnel
ngrok http 3000

# Use the ngrok URL as your OAuth callback
# https://abc123.ngrok.io/api/auth/oauth/github/callback
```

**2. Test each provider**
```bash
# GitHub
curl -v http://localhost:3000/api/auth/oauth/github

# Google
curl -v http://localhost:3000/api/auth/oauth/google

# Discord
curl -v http://localhost:3000/api/auth/oauth/discord
```

---

### Production Testing

**1. Verify redirect URLs**
- Check OAuth provider dashboard for correct callback URL
- Test with production domain

**2. Test user creation**
- Register new user via OAuth
- Verify user created in database
- Check email association

**3. Test existing user login**
- Login with existing OAuth account
- Verify tokens generated
- Check session created

---

## Troubleshooting

### Common Issues

**"Redirect URI mismatch"**
- Verify callback URL in OAuth provider settings matches exactly
- Check for trailing slashes
- Ensure http vs https is correct

**"Invalid client_id"**
- Double-check client ID in `.env`
- Restart auth service after changes
- Verify environment variable is loaded

**"User email not found"**
- Some providers don't return email by default
- Request additional scopes
- Handle case where email is not provided

**"Token exchange failed"**
- Check client secret is correct
- Verify OAuth code hasn't expired (typically 10 min)
- Check network connectivity to provider

---

## Security Best Practices

1. **Use HTTPS in production** - Never use OAuth over HTTP
2. **Store secrets securely** - Use environment variables or secret manager
3. **Validate state parameter** - Prevent CSRF attacks
4. **Implement PKCE** - For public clients (mobile/SPA)
5. **Rate limit OAuth endpoints** - Prevent brute force
6. **Log OAuth failures** - Monitor for attacks
7. **Rotate secrets regularly** - Especially if compromised

---

**Last Updated:** May 13, 2026  
**See Also:** [Getting Started](../getting-started.md), [Deployment Guide](../../DEPLOYMENT.md)
