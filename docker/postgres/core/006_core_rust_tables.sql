-- Core Rust Service Tables
-- Adds missing tables and columns for the core-rust service

-- Create hackathon_config table (singleton)
CREATE TABLE IF NOT EXISTS core.hackathon_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255),
    tagline TEXT,
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    timezone VARCHAR(50) DEFAULT 'UTC',
    logo_url TEXT,
    theme_colors JSONB DEFAULT '{}'::jsonb,
    social_links JSONB DEFAULT '{}'::jsonb,
    registration_open BOOLEAN DEFAULT TRUE
);

INSERT INTO core.hackathon_config (id, name, tagline)
SELECT '00000000-0000-0000-0000-000000000001', 'OpenHack', 'Build something amazing'
WHERE NOT EXISTS (SELECT 1 FROM core.hackathon_config LIMIT 1);

-- Add columns to existing teams table
ALTER TABLE core.teams ADD COLUMN IF NOT EXISTS join_code VARCHAR(8) UNIQUE;
ALTER TABLE core.teams ADD COLUMN IF NOT EXISTS max_size INT DEFAULT 4;

-- Add columns to existing projects table
ALTER TABLE core.projects ADD COLUMN IF NOT EXISTS category VARCHAR(100);
ALTER TABLE core.projects ADD COLUMN IF NOT EXISTS video_url TEXT;
ALTER TABLE core.projects ADD COLUMN IF NOT EXISTS tags TEXT[];
ALTER TABLE core.projects ADD COLUMN IF NOT EXISTS submission_number INT;
ALTER TABLE core.projects ADD COLUMN IF NOT EXISTS submitted_at TIMESTAMP;

-- Create events table
CREATE TABLE IF NOT EXISTS core.events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    location VARCHAR(255),
    type VARCHAR(50),
    capacity INT,
    speaker_name VARCHAR(255),
    speaker_bio TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_events_type ON core.events(type);
CREATE INDEX IF NOT EXISTS idx_events_start_time ON core.events(start_time);

-- Create event_rsvps table
CREATE TABLE IF NOT EXISTS core.event_rsvps (
    event_id UUID NOT NULL REFERENCES core.events(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    rsvped_at TIMESTAMP DEFAULT NOW(),
    attended BOOLEAN DEFAULT FALSE,
    checked_in BOOLEAN DEFAULT FALSE,
    PRIMARY KEY(event_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_event_rsvps_event ON core.event_rsvps(event_id);
CREATE INDEX IF NOT EXISTS idx_event_rsvps_user ON core.event_rsvps(user_id);

-- Create team_invites table
CREATE TABLE IF NOT EXISTS core.team_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES core.teams(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    invited_by UUID NOT NULL,
    expires_at TIMESTAMP,
    accepted BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_team_invites_team ON core.team_invites(team_id);
CREATE INDEX IF NOT EXISTS idx_team_invites_email ON core.team_invites(email);

-- Indexes for new columns
CREATE INDEX IF NOT EXISTS idx_teams_join_code ON core.teams(join_code) WHERE join_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_projects_category ON core.projects(category) WHERE category IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_projects_tags ON core.projects USING GIN(tags) WHERE tags IS NOT NULL;
