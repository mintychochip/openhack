-- Core Service Additional Migrations
-- This file is loaded after init.sql

-- Hackathon phases table (used by the phase scheduler)
CREATE TABLE IF NOT EXISTS core.hackathon_phases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hackathon_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('registration', 'hacking', 'submission', 'judging', 'awards')),
    description TEXT,
    opens_at TIMESTAMP,
    closes_at TIMESTAMP,
    is_active BOOLEAN DEFAULT FALSE,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_hackathon_phases_hackathon_id ON core.hackathon_phases (hackathon_id);
CREATE INDEX IF NOT EXISTS idx_hackathon_phases_active ON core.hackathon_phases (is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_hackathon_phases_opens ON core.hackathon_phases (opens_at) WHERE opens_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_hackathon_phases_closes ON core.hackathon_phases (closes_at) WHERE closes_at IS NOT NULL;

CREATE TRIGGER update_hackathon_phases_updated_at
    BEFORE UPDATE ON core.hackathon_phases
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
