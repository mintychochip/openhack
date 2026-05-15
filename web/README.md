# OpenHack Web Frontend

Next.js 14 + TypeScript + Tailwind CSS frontend for OpenHack platform.

## Quick Start

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start
```

## Configuration

Create `.env.local`:

```bash
NEXT_PUBLIC_API_URL=http://localhost:8000
```

## Project Structure

```
web/
├── src/
│   ├── app/                    # Next.js App Router
│   │   ├── dashboard/          # Authenticated dashboard
│   │   ├── login/              # Login page
│   │   ├── register/           # Registration page
│   │   ├── layout.tsx          # Root layout
│   │   └── page.tsx            # Home page
│   ├── components/
│   │   └── ui/                 # shadcn/ui components
│   └── lib/
│       ├── api.ts              # API client
│       └── utils.ts            # Utility functions
├── package.json
├── tsconfig.json
├── tailwind.config.ts
└── next.config.js
```

## Features

- ✅ Authentication (login, register)
- ✅ Dashboard with stats
- ✅ Responsive design
- ✅ Dark mode support
- ✅ TypeScript for type safety
- ✅ Tailwind CSS for styling
- ✅ shadcn/ui components

## API Integration

The API client (`src/lib/api.ts`) handles:
- JWT authentication
- Token storage (localStorage)
- Automatic token refresh
- Error handling

All backend services are proxied through Next.js rewrites in `next.config.js`.

## Pages to Build

### Auth Pages
- [x] Login
- [x] Register
- [ ] Forgot Password
- [ ] Reset Password
- [ ] MFA Setup/Verify

### Participant Portal
- [ ] Dashboard
- [ ] Teams (create, join, manage)
- [ ] Projects (create, submit, upload demo)
- [ ] Events (RSVP)
- [ ] Leaderboard (view, vote)

### Judge Interface
- [ ] Dashboard
- [ ] Assignments
- [ ] Scoring
- [ ] Rubrics

### Admin Dashboard
- [ ] User Management
- [ ] Team Management
- [ ] Project Management
- [ ] Event Management
- [ ] Judging Configuration
- [ ] Leaderboard Settings

### Sponsor Portal
- [ ] Booth Management
- [ ] Prize Management
- [ ] Winner Selection
