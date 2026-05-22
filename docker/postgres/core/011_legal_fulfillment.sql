-- Core Service - Legal and Prize Fulfillment
-- Creates hackathon rules and prize fulfillment tracking tables

CREATE TABLE IF NOT EXISTS core.hackathon_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    rules_text TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1,
    published BOOLEAN DEFAULT FALSE,
    published_at TIMESTAMP,
    created_by UUID,  -- User ID of organizer who created
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(event_id, version)
);

CREATE INDEX IF NOT EXISTS idx_rules_event ON core.hackathon_rules(event_id);
CREATE INDEX IF NOT EXISTS idx_rules_published ON core.hackathon_rules(published);

CREATE TABLE IF NOT EXISTS core.prize_fulfillment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prize_id UUID NOT NULL,  -- Reference to sponsor.prizes
    sponsor_id UUID REFERENCES sponsor.booths(id),  -- Denormalized for easier queries
    winner_project_id UUID NOT NULL,  -- Reference to core.projects
    winner_user_id UUID NOT NULL,  -- Reference to auth.users (team lead)
    status VARCHAR(50) DEFAULT 'pending',  -- pending, claimed, shipped, fulfilled, declined
    shipping_address JSONB,  -- {name, street, city, state, zip, country, phone}
    claimed_at TIMESTAMP,
    shipped_at TIMESTAMP,
    tracking_number VARCHAR(255),
    carrier VARCHAR(100),
    estimated_delivery DATE,
    delivered_at TIMESTAMP,
    notes TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fulfillment_prize ON core.prize_fulfillment(prize_id);
CREATE INDEX IF NOT EXISTS idx_fulfillment_project ON core.prize_fulfillment(winner_project_id);
CREATE INDEX IF NOT EXISTS idx_fulfillment_user ON core.prize_fulfillment(winner_user_id);
CREATE INDEX IF NOT EXISTS idx_fulfillment_status ON core.prize_fulfillment(status);

-- Auto-update updated_at
CREATE OR REPLACE FUNCTION core.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_hackathon_rules_updated_at ON core.hackathon_rules;
CREATE TRIGGER update_hackathon_rules_updated_at
    BEFORE UPDATE ON core.hackathon_rules
    FOR EACH ROW
    EXECUTE FUNCTION core.update_updated_at_column();

DROP TRIGGER IF EXISTS update_prize_fulfillment_updated_at ON core.prize_fulfillment;
CREATE TRIGGER update_prize_fulfillment_updated_at
    BEFORE UPDATE ON core.prize_fulfillment
    FOR EACH ROW
    EXECUTE FUNCTION core.update_updated_at_column();

COMMENT ON TABLE core.hackathon_rules IS 'Hackathon rules and terms with versioning';
COMMENT ON TABLE core.prize_fulfillment IS 'Prize fulfillment tracking from award to delivery';

COMMENT ON COLUMN core.hackathon_rules.event_id IS 'Event this ruleset belongs to';
COMMENT ON COLUMN core.hackathon_rules.version IS 'Version number for rules (auto-incremented)';
COMMENT ON COLUMN core.hackathon_rules.published IS 'Whether rules are visible to participants';

COMMENT ON COLUMN core.prize_fulfillment.prize_id IS 'Reference to sponsor.prizes';
COMMENT ON COLUMN core.prize_fulfillment.sponsor_id IS 'Denormalized sponsor reference for easier queries';
COMMENT ON COLUMN core.prize_fulfillment.winner_project_id IS 'Winning project from core.projects';
COMMENT ON COLUMN core.prize_fulfillment.winner_user_id IS 'Team lead/user to receive prize from auth.users';
COMMENT ON COLUMN core.prize_fulfillment.status IS 'pending, claimed, shipped, fulfilled, or declined';
COMMENT ON COLUMN core.prize_fulfillment.shipping_address IS 'JSON with name, street, city, state, zip, country, phone';
COMMENT ON COLUMN core.prize_fulfillment.tracking_number IS 'Carrier tracking number for shipped prizes';
COMMENT ON COLUMN core.prize_fulfillment.notes IS 'Internal notes about fulfillment issues or special handling';
