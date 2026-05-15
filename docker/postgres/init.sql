-- ================================
-- OpenHack Database Initialization
-- ================================

-- Create schemas for each service
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS core;
CREATE SCHEMA IF NOT EXISTS mail;
CREATE SCHEMA IF NOT EXISTS judging;
CREATE SCHEMA IF NOT EXISTS leaderboard;
CREATE SCHEMA IF NOT EXISTS ai;
CREATE SCHEMA IF NOT EXISTS analytics;
CREATE SCHEMA IF NOT EXISTS sponsor;
CREATE SCHEMA IF NOT EXISTS media;
CREATE SCHEMA IF NOT EXISTS notify;
CREATE SCHEMA IF NOT EXISTS discord_bot;

-- Grant permissions
GRANT ALL ON SCHEMA auth TO openhack;
GRANT ALL ON SCHEMA core TO openhack;
GRANT ALL ON SCHEMA mail TO openhack;
GRANT ALL ON SCHEMA judging TO openhack;
GRANT ALL ON SCHEMA leaderboard TO openhack;
GRANT ALL ON SCHEMA ai TO openhack;
GRANT ALL ON SCHEMA analytics TO openhack;
GRANT ALL ON SCHEMA sponsor TO openhack;
GRANT ALL ON SCHEMA media TO openhack;
GRANT ALL ON SCHEMA notify TO openhack;
GRANT ALL ON SCHEMA discord_bot TO openhack;

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ================================
-- Auth Service Migrations
-- ================================

-- Users table
CREATE TABLE IF NOT EXISTS auth.users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255),
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login_at TIMESTAMP,
    status VARCHAR(50) DEFAULT 'active'
);

CREATE INDEX IF NOT EXISTS idx_users_email ON auth.users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON auth.users(username);
CREATE INDEX IF NOT EXISTS idx_users_status ON auth.users(status);

-- OAuth accounts table
CREATE TABLE IF NOT EXISTS auth.oauth_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_account_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    expires_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, provider_account_id)
);

CREATE INDEX IF NOT EXISTS idx_oauth_user_id ON auth.oauth_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_provider ON auth.oauth_accounts(provider);

-- Sessions table
CREATE TABLE IF NOT EXISTS auth.sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    user_agent TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON auth.sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON auth.sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON auth.sessions(expires_at);

-- Password reset tokens
CREATE TABLE IF NOT EXISTS auth.password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_reset_tokens_user ON auth.password_reset_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_reset_tokens_expires ON auth.password_reset_tokens(expires_at);

-- ================================
-- Core Service Migrations
-- ================================

-- Teams table
CREATE TABLE IF NOT EXISTS core.teams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) DEFAULT 'active'
);

CREATE INDEX IF NOT EXISTS idx_teams_name ON core.teams(name);
CREATE INDEX IF NOT EXISTS idx_teams_status ON core.teams(status);

-- Team members
CREATE TABLE IF NOT EXISTS core.team_members (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    team_id UUID NOT NULL REFERENCES core.teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member',
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(team_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_team_members_team ON core.team_members(team_id);
CREATE INDEX IF NOT EXISTS idx_team_members_user ON core.team_members(user_id);

-- Projects table
CREATE TABLE IF NOT EXISTS core.projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    team_id UUID NOT NULL REFERENCES core.teams(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    repository_url VARCHAR(500),
    demo_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) DEFAULT 'draft'
);

CREATE INDEX IF NOT EXISTS idx_projects_team ON core.projects(team_id);
CREATE INDEX IF NOT EXISTS idx_projects_status ON core.projects(status);

-- Challenges table
CREATE TABLE IF NOT EXISTS core.challenges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    sponsor VARCHAR(255),
    prize_pool DECIMAL(10, 2),
    start_date TIMESTAMP,
    end_date TIMESTAMP,
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_challenges_status ON core.challenges(status);
CREATE INDEX IF NOT EXISTS idx_challenges_dates ON core.challenges(start_date, end_date);

-- Challenge participants
CREATE TABLE IF NOT EXISTS core.challenge_participants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    challenge_id UUID NOT NULL REFERENCES core.challenges(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES core.teams(id) ON DELETE CASCADE,
    registered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(challenge_id, team_id)
);

CREATE INDEX IF NOT EXISTS idx_participants_challenge ON core.challenge_participants(challenge_id);
CREATE INDEX IF NOT EXISTS idx_participants_team ON core.challenge_participants(team_id);

-- ================================
-- Mail Service Migrations
-- ================================

-- Email templates
CREATE TABLE IF NOT EXISTS mail.email_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    subject VARCHAR(500) NOT NULL,
    body_html TEXT NOT NULL,
    body_text TEXT,
    variables JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_templates_name ON mail.email_templates(name);

-- Email queue
CREATE TABLE IF NOT EXISTS mail.email_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    to_address VARCHAR(255) NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    subject VARCHAR(500) NOT NULL,
    body_html TEXT,
    body_text TEXT,
    status VARCHAR(50) DEFAULT 'pending',
    priority INTEGER DEFAULT 0,
    scheduled_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    sent_at TIMESTAMP,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_email_queue_status ON mail.email_queue(status);
CREATE INDEX IF NOT EXISTS idx_email_queue_scheduled ON mail.email_queue(scheduled_at);
CREATE INDEX IF NOT EXISTS idx_email_queue_priority ON mail.email_queue(priority, scheduled_at);

-- Email logs
CREATE TABLE IF NOT EXISTS mail.email_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    queue_id UUID REFERENCES mail.email_queue(id),
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_email_logs_queue ON mail.email_logs(queue_id);
CREATE INDEX IF NOT EXISTS idx_email_logs_event ON mail.email_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_email_logs_created ON mail.email_logs(created_at);

-- ================================
-- Housekeeping Functions (needed before subdirectory migrations)
-- ================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Run auth Rust compatibility migrations
\i /docker-entrypoint-initdb.d/auth/001_init.sql
\i /docker-entrypoint-initdb.d/auth/002_avatar_files.sql
\i /docker-entrypoint-initdb.d/auth/003_extended_profiles.sql
\i /docker-entrypoint-initdb.d/auth/004_rust_service_compat.sql

-- Run ai migrations
\i /docker-entrypoint-initdb.d/ai/001_init.sql

-- Run core additional migrations
\i /docker-entrypoint-initdb.d/core/001_init.sql
\i /docker-entrypoint-initdb.d/core/003_demo_files.sql
\i /docker-entrypoint-initdb.d/core/004_phases.sql
\i /docker-entrypoint-initdb.d/core/006_core_rust_tables.sql
\i /docker-entrypoint-initdb.d/core/005_checkin.sql

-- Run judging migrations
\i /docker-entrypoint-initdb.d/judging/001_init.sql
\i /docker-entrypoint-initdb.d/judging/002_phases_and_normalization.sql

-- Run leaderboard migrations
\i /docker-entrypoint-initdb.d/leaderboard/001_init.sql
\i /docker-entrypoint-initdb.d/leaderboard/002_formulas_and_voting.sql

-- Run notify migrations
\i /docker-entrypoint-initdb.d/notify/001_init.sql
\i /docker-entrypoint-initdb.d/notify/002_user_notifications.sql

-- Run analytics migrations
\i /docker-entrypoint-initdb.d/analytics/001_init.sql

-- Run sponsors migrations
\i /docker-entrypoint-initdb.d/sponsors/001_init.sql
\i /docker-entrypoint-initdb.d/sponsors/002_submissions_status.sql

-- Run media migrations
\i /docker-entrypoint-initdb.d/media/001_init.sql
\i /docker-entrypoint-initdb.d/media/002_rust_service_compat.sql

-- Run mail migrations
\i /docker-entrypoint-initdb.d/mail/001_init.sql
\i /docker-entrypoint-initdb.d/mail/002_attachment_files.sql

-- Run discord-bot migrations
\i /docker-entrypoint-initdb.d/discord-bot/init.sql

-- ================================
-- Initial Data
-- ================================

-- Insert default email templates
INSERT INTO mail.email_templates (name, subject, body_html, body_text, variables) VALUES
    ('welcome', 'Welcome to OpenHack!', '
        <html>
            <body>
                <h1>Welcome to OpenHack, {{username}}!</h1>
                <p>We are excited to have you on board.</p>
                <p>Get started by creating your first team and joining a challenge.</p>
            </body>
        </html>
    ', 'Welcome to OpenHack, {{username}}!', '["username"]'::jsonb),
    ('verify_email', 'Verify Your Email', '
        <html>
            <body>
                <h1>Verify Your Email</h1>
                <p>Click the link below to verify your email:</p>
                <a href="{{verification_link}}">Verify Email</a>
            </body>
        </html>
    ', 'Verify your email: {{verification_link}}', '["verification_link"]'::jsonb),
    ('password_reset', 'Password Reset Request', '
        <html>
            <body>
                <h1>Password Reset</h1>
                <p>Click the link below to reset your password:</p>
                <a href="{{reset_link}}">Reset Password</a>
                <p>This link expires in 1 hour.</p>
            </body>
        </html>
    ', 'Reset your password: {{reset_link}}', '["reset_link"]'::jsonb)
ON CONFLICT (name) DO NOTHING;

-- ================================
-- Housekeeping Functions
-- ================================

-- Apply updated_at trigger to relevant tables
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON auth.users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_teams_updated_at
    BEFORE UPDATE ON core.teams
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_projects_updated_at
    BEFORE UPDATE ON core.projects
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_challenges_updated_at
    BEFORE UPDATE ON core.challenges
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_email_templates_updated_at
    BEFORE UPDATE ON mail.email_templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to clean up expired sessions
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS void AS $$
BEGIN
    DELETE FROM auth.sessions WHERE expires_at < CURRENT_TIMESTAMP;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up expired password reset tokens
CREATE OR REPLACE FUNCTION cleanup_expired_reset_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM auth.password_reset_tokens WHERE expires_at < CURRENT_TIMESTAMP OR used = TRUE;
END;
$$ LANGUAGE plpgsql;

-- Run performance indexes (after all schemas and tables are created)
\i /docker-entrypoint-initdb.d/indexes/002_performance_indexes.sql
