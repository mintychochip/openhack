-- Hackathon Phases Table
-- Manages time-based phases for hackathons (registration, hacking, submission, judging, awards)

CREATE TABLE IF NOT EXISTS core.hackathon_phases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hackathon_id UUID NOT NULL REFERENCES core.hackathon_config(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('registration', 'hacking', 'submission', 'judging', 'awards')),
    description TEXT,
    opens_at TIMESTAMP NOT NULL,
    closes_at TIMESTAMP NOT NULL,
    is_active BOOLEAN DEFAULT FALSE,
    config JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    
    CONSTRAINT valid_phase_times CHECK (closes_at > opens_at)
);

-- Indexes for performance
CREATE INDEX idx_phases_hackathon ON core.hackathon_phases(hackathon_id);
CREATE INDEX idx_phases_type ON core.hackathon_phases(type);
CREATE INDEX idx_phases_opens_at ON core.hackathon_phases(opens_at);
CREATE INDEX idx_phases_closes_at ON core.hackathon_phases(closes_at);
CREATE INDEX idx_phases_is_active ON core.hackathon_phases(is_active);
CREATE INDEX idx_phases_active_hackathon ON core.hackathon_phases(hackathon_id, is_active) WHERE is_active = true;

-- Add comment
COMMENT ON TABLE core.hackathon_phases IS 'Time-based phases for hackathon lifecycle management';
COMMENT ON COLUMN core.hackathon_phases.type IS 'Phase type: registration, hacking, submission, judging, awards';
COMMENT ON COLUMN core.hackathon_phases.is_active IS 'Whether this phase is currently active (auto-managed by scheduler)';
COMMENT ON COLUMN core.hackathon_phases.config IS 'Phase-specific configuration (e.g., max submissions, judge assignments)';

-- Insert default phases for existing hackathon (if needed)
-- INSERT INTO core.hackathon_phases (hackathon_id, name, type, opens_at, closes_at, is_active)
-- SELECT 
--     id,
--     'Registration',
--     'registration',
--     start_time,
--     start_time + INTERVAL '7 days',
--     FALSE
-- FROM core.hackathon_config
-- WHERE NOT EXISTS (
--     SELECT 1 FROM core.hackathon_phases WHERE hackathon_id = core.hackathon_config.id
-- );
