-- Sponsor Booths Service Database Schema
-- Creates sponsor.* schema with booths, prizes, and submissions tables

-- Create schema
CREATE SCHEMA IF NOT EXISTS sponsor;

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Booths table - stores sponsor page configurations
CREATE TABLE IF NOT EXISTS sponsor.booths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sponsor_name VARCHAR(255) NOT NULL,
    tagline VARCHAR(500),
    description TEXT,
    logo_url TEXT,
    banner_url TEXT,
    website_url TEXT,
    careers_url TEXT,
    api_docs_url TEXT,
    technologies TEXT[],
    contact_email VARCHAR(255),
    discord_channel VARCHAR(255),
    theme_colors JSONB,
    published BOOLEAN DEFAULT FALSE,
    view_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Prizes table - stores prize definitions for booths
CREATE TABLE IF NOT EXISTS sponsor.prizes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booth_id UUID NOT NULL REFERENCES sponsor.booths(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    value_usd DECIMAL(10,2),
    criteria TEXT,
    winner_project_id UUID,
    announced BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Submissions table - tracks project submissions for prizes
CREATE TABLE IF NOT EXISTS sponsor.submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prize_id UUID REFERENCES sponsor.prizes(id) ON DELETE CASCADE,
    project_id UUID NOT NULL,
    submitted_at TIMESTAMP DEFAULT NOW(),
    CONSTRAINT unique_prize_project UNIQUE (prize_id, project_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_booths_published ON sponsor.booths(published);
CREATE INDEX IF NOT EXISTS idx_prizes_booth ON sponsor.prizes(booth_id);
CREATE INDEX IF NOT EXISTS idx_submissions_prize ON sponsor.submissions(prize_id);

-- Comments for documentation
COMMENT ON TABLE sponsor.booths IS 'Sponsor booth configurations for hackathon presence';
COMMENT ON TABLE sponsor.prizes IS 'Prizes offered by sponsors for hackathon participants';
COMMENT ON TABLE sponsor.submissions IS 'Project submissions for sponsor prizes';

COMMENT ON COLUMN sponsor.booths.theme_colors IS 'JSON object with color customization (e.g., {"primary": "#FF0000"})';
COMMENT ON COLUMN sponsor.booths.technologies IS 'Array of technology tags (e.g., ["react", "nodejs", "mongodb"])';
COMMENT ON COLUMN sponsor.prizes.value_usd IS 'Prize value in US dollars';
COMMENT ON COLUMN sponsor.prizes.winner_project_id IS 'UUID of winning project (from projects service)';
