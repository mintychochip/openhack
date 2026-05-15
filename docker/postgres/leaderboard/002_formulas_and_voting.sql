-- Leaderboard Service Schema Migration - Phase 2
-- Adds: ranking formulas, voting controls, frozen leaderboards, and critical fixes

-- =====================================================
-- 1. RANKING FORMULAS
-- =====================================================

CREATE TABLE IF NOT EXISTS leaderboard.ranking_formulas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    formula TEXT NOT NULL,  -- e.g., "0.7 * judge_score + 0.3 * public_votes"
    variables TEXT[],  -- e.g., ["judge_score", "public_votes"]
    is_active BOOLEAN DEFAULT FALSE,
    is_preset BOOLEAN DEFAULT FALSE,  -- True for system presets
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_ranking_formulas_active ON leaderboard.ranking_formulas(is_active);

-- Insert preset formulas
INSERT INTO leaderboard.ranking_formulas (name, description, formula, variables, is_preset, is_active) VALUES
    ('Judge Only', 'Pure expert judging - no public votes', 'judge_score', ARRAY['judge_score'], TRUE, FALSE),
    ('Hybrid 70/30', 'Balanced: 70% judge score, 30% public votes', '0.7 * judge_score + 0.3 * public_votes', ARRAY['judge_score', 'public_votes'], TRUE, TRUE),
    ('Hybrid 50/50', 'Community-heavy: equal judge and public weight', '0.5 * judge_score + 0.5 * public_votes', ARRAY['judge_score', 'public_votes'], TRUE, FALSE),
    ('Vote Only', 'Popularity contest - pure public voting', 'public_votes', ARRAY['public_votes'], TRUE, FALSE),
    ('Logarithmic', 'Judge score with diminishing vote returns', 'judge_score * (1.0 + ln(public_votes + 1.0) / 10.0)', ARRAY['judge_score', 'public_votes'], TRUE, FALSE)
ON CONFLICT DO NOTHING;

-- =====================================================
-- 2. VOTING CONFIGURATION
-- =====================================================

CREATE TABLE IF NOT EXISTS leaderboard.voting_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phase_id UUID,  -- NULL = global config
    opens_at TIMESTAMP,
    closes_at TIMESTAMP,
    max_votes_per_user INT DEFAULT 10,
    max_votes_per_minute INT DEFAULT 5,
    rate_limit_window_seconds INT DEFAULT 60,
    vote_weight_default DECIMAL(3,2) DEFAULT 1.0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_voting_config_phase ON leaderboard.voting_config(phase_id);
CREATE INDEX idx_voting_config_active ON leaderboard.voting_config(is_active);

-- Insert default global config
INSERT INTO leaderboard.voting_config (opens_at, closes_at, max_votes_per_user, max_votes_per_minute, is_active) VALUES
    (NOW(), NOW() + INTERVAL '30 days', 10, 5, TRUE)
ON CONFLICT DO NOTHING;

-- =====================================================
-- 3. VOTE WEIGHTS BY ROLE
-- =====================================================

CREATE TABLE IF NOT EXISTS leaderboard.vote_weights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voter_role VARCHAR(50) NOT NULL UNIQUE,  -- participant, judge, sponsor, admin, vip
    weight DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Insert default weights
INSERT INTO leaderboard.vote_weights (voter_role, weight, description) VALUES
    ('participant', 1.0, 'Standard participant vote'),
    ('judge', 1.5, 'Judge vote carries extra weight'),
    ('sponsor', 2.0, 'Sponsor vote has 2x weight'),
    ('admin', 1.0, 'Admin vote (standard weight)'),
    ('vip', 3.0, 'VIP/special guest vote has 3x weight')
ON CONFLICT DO NOTHING;

-- =====================================================
-- 4. FROZEN LEADERBOARD + SNAPSHOTS
-- =====================================================

ALTER TABLE leaderboard.ranks 
    ADD COLUMN IF NOT EXISTS is_frozen BOOLEAN DEFAULT FALSE;

ALTER TABLE leaderboard.ranks 
    ADD COLUMN IF NOT EXISTS frozen_at TIMESTAMP;

ALTER TABLE leaderboard.ranks 
    ADD COLUMN IF NOT EXISTS phase_id UUID;

ALTER TABLE leaderboard.ranks 
    ADD COLUMN IF NOT EXISTS combined_score DECIMAL(10,4);

ALTER TABLE leaderboard.ranks 
    ADD COLUMN IF NOT EXISTS formula_id UUID REFERENCES leaderboard.ranking_formulas(id);

CREATE INDEX idx_ranks_frozen ON leaderboard.ranks(is_frozen);
CREATE INDEX idx_ranks_phase ON leaderboard.ranks(phase_id);

-- Leaderboard snapshots table
CREATE TABLE IF NOT EXISTS leaderboard.snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    phase_id UUID,
    formula_id UUID REFERENCES leaderboard.ranking_formulas(id),
    snapshot_data JSONB NOT NULL,  -- Full leaderboard state at snapshot time
    team_count INT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_snapshots_phase ON leaderboard.snapshots(phase_id);
CREATE INDEX idx_snapshots_created ON leaderboard.snapshots(created_at);

-- =====================================================
-- 5. VOTE MODERATION
-- =====================================================

ALTER TABLE leaderboard.votes 
    ADD COLUMN IF NOT EXISTS is_valid BOOLEAN DEFAULT TRUE;

ALTER TABLE leaderboard.votes 
    ADD COLUMN IF NOT EXISTS moderated_at TIMESTAMP;

ALTER TABLE leaderboard.votes 
    ADD COLUMN IF NOT EXISTS moderated_by UUID;

ALTER TABLE leaderboard.votes 
    ADD COLUMN IF NOT EXISTS moderation_reason TEXT;

ALTER TABLE leaderboard.votes 
    ADD COLUMN IF NOT EXISTS weight_applied DECIMAL(3,2) DEFAULT 1.0;

CREATE INDEX idx_votes_valid ON leaderboard.votes(is_valid);
CREATE INDEX idx_votes_token_hash ON leaderboard.votes(voter_token_hash);

-- =====================================================
-- 6. CRITICAL SCHEMA FIXES
-- =====================================================

-- Fix token hash indexing (was missing)
CREATE INDEX IF NOT EXISTS idx_votes_token_hash_valid 
    ON leaderboard.votes(voter_token_hash) 
    WHERE is_valid = TRUE;

-- Add vote weight to ranks calculation
ALTER TABLE leaderboard.ranks 
    ADD COLUMN IF NOT EXISTS weighted_votes DECIMAL(10,2) DEFAULT 0;

-- Add tiebreaker column (for deterministic ordering)
ALTER TABLE leaderboard.ranks 
    ADD COLUMN IF NOT EXISTS tiebreaker_score DECIMAL(10,4);

-- =====================================================
-- 7. MISSING INDEXES FOR PERFORMANCE
-- =====================================================

CREATE INDEX IF NOT EXISTS idx_score_history_time 
    ON leaderboard.score_history(team_id, recorded_at DESC);

CREATE INDEX IF NOT EXISTS idx_ranks_combined_score 
    ON leaderboard.ranks(combined_score DESC) 
    WHERE is_frozen = FALSE;

-- GIN index for snapshot JSONB
CREATE INDEX IF NOT EXISTS idx_snapshots_data 
    ON leaderboard.snapshots USING GIN (snapshot_data);

-- =====================================================
-- 8. HELPER FUNCTIONS
-- =====================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION leaderboard.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for ranking_formulas table
DROP TRIGGER IF EXISTS update_formulas_updated_at ON leaderboard.ranking_formulas;
CREATE TRIGGER update_formulas_updated_at
    BEFORE UPDATE ON leaderboard.ranking_formulas
    FOR EACH ROW
    EXECUTE FUNCTION leaderboard.update_updated_at_column();

-- Trigger for voting_config table
DROP TRIGGER IF EXISTS update_voting_config_updated_at ON leaderboard.voting_config;
CREATE TRIGGER update_voting_config_updated_at
    BEFORE UPDATE ON leaderboard.voting_config
    FOR EACH ROW
    EXECUTE FUNCTION leaderboard.update_updated_at_column();

-- Function to check if voting is currently open
CREATE OR REPLACE FUNCTION leaderboard.is_voting_open()
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM leaderboard.voting_config
        WHERE is_active = TRUE
          AND (opens_at IS NULL OR opens_at <= NOW())
          AND (closes_at IS NULL OR closes_at > NOW())
    );
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get vote weight for a role
CREATE OR REPLACE FUNCTION leaderboard.get_vote_weight(role_name VARCHAR)
RETURNS DECIMAL(3,2) AS $$
BEGIN
    RETURN COALESCE(
        (SELECT weight FROM leaderboard.vote_weights WHERE voter_role = role_name),
        (SELECT vote_weight_default FROM leaderboard.voting_config WHERE is_active = TRUE LIMIT 1),
        1.0
    );
END;
$$ LANGUAGE plpgsql STABLE;

-- =====================================================
-- 9. SEED DATA
-- =====================================================

-- Default phase reference (matches judging.phases default)
-- Note: This is a cross-schema reference, handled in application code

COMMENT ON TABLE leaderboard.ranking_formulas IS 'Configurable ranking formulas for leaderboard computation';
COMMENT ON TABLE leaderboard.voting_config IS 'Voting time windows and rate limits';
COMMENT ON TABLE leaderboard.vote_weights IS 'Vote weight multipliers by user role';
COMMENT ON TABLE leaderboard.snapshots IS 'Frozen leaderboard snapshots for historical reference';
COMMENT ON COLUMN leaderboard.ranking_formulas.formula IS 'Mathematical expression using variables like judge_score, public_votes';
COMMENT ON COLUMN leaderboard.voting_config.opens_at IS 'When voting opens (NULL = always open)';
COMMENT ON COLUMN leaderboard.voting_config.closes_at IS 'When voting closes (NULL = never closes)';
COMMENT ON COLUMN leaderboard.votes.is_valid IS 'False if vote was flagged as fraudulent/invalid';
COMMENT ON COLUMN leaderboard.votes.weight_applied IS 'Actual vote weight applied (from voter role)';
COMMENT ON COLUMN leaderboard.ranks.combined_score IS 'Result of ranking formula computation';
COMMENT ON COLUMN leaderboard.ranks.is_frozen IS 'True when rankings are locked (phase ended)';
COMMENT ON COLUMN leaderboard.ranks.tiebreaker_score IS 'Secondary sort key for deterministic ordering';
