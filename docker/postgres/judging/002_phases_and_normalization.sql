-- Judging Service Schema Migration - Phase 2
-- Adds: phases, rubric versioning, score normalization, and critical fixes

-- =====================================================
-- 1. JUDGING PHASES SYSTEM
-- =====================================================

CREATE TABLE IF NOT EXISTS judging.phases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL,  -- preliminary, semifinal, final, custom
    rubric_id UUID REFERENCES judging.rubrics(id),
    opens_at TIMESTAMP,
    closes_at TIMESTAMP,
    advancement_count INT,  -- Number of teams that advance to next phase
    parent_phase_id UUID REFERENCES judging.phases(id),
    status VARCHAR(50) DEFAULT 'draft',  -- draft, open, closed, finalized
    scoring_method VARCHAR(50) DEFAULT 'raw',  -- raw, normalized_zscore, normalized_minmax
    normalization_config JSONB,  -- {min_scores_required: 3, method: z_score}
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS judging.phase_advancements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_phase_id UUID REFERENCES judging.phases(id),
    to_phase_id UUID REFERENCES judging.phases(id),
    team_id UUID NOT NULL,
    project_id UUID NOT NULL,
    advanced_at TIMESTAMP DEFAULT NOW(),
    rank_in_phase INT,
    UNIQUE(from_phase_id, to_phase_id, team_id)
);

CREATE INDEX idx_phases_status ON judging.phases(status);
CREATE INDEX idx_phases_type ON judging.phases(type);
CREATE INDEX idx_advancements_team ON judging.phase_advancements(team_id);

-- =====================================================
-- 2. RUBRIC VERSIONING
-- =====================================================

CREATE TABLE IF NOT EXISTS judging.rubric_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rubric_id UUID NOT NULL REFERENCES judging.rubrics(id) ON DELETE CASCADE,
    version INT NOT NULL,
    criteria JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(rubric_id, version)
);

CREATE INDEX idx_rubric_versions_rubric ON judging.rubric_versions(rubric_id);

-- =====================================================
-- 3. JUDGE STATISTICS (for normalization)
-- =====================================================

CREATE TABLE IF NOT EXISTS judging.judge_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    judge_id UUID NOT NULL,
    phase_id UUID REFERENCES judging.phases(id),
    mean_score DECIMAL(10,4),
    std_dev DECIMAL(10,4),
    total_scores_count INT DEFAULT 0,
    min_score DECIMAL(10,4),
    max_score DECIMAL(10,4),
    computed_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(judge_id, phase_id)
);

CREATE INDEX idx_judge_stats_phase ON judging.judge_stats(phase_id);

-- =====================================================
-- 4. CRITICAL SCHEMA FIXES
-- =====================================================

-- Fix total_score overflow (DECIMAL(5,2) -> DECIMAL(10,4))
ALTER TABLE judging.scores 
    ALTER COLUMN total_score TYPE DECIMAL(10,4);

-- Add phase_id to assignments
ALTER TABLE judging.assignments 
    ADD COLUMN IF NOT EXISTS phase_id UUID REFERENCES judging.phases(id);

-- Add phase_id to scores (for denormalized queries)
ALTER TABLE judging.scores 
    ADD COLUMN IF NOT EXISTS phase_id UUID REFERENCES judging.phases(id);

-- Add started_at to track judging duration
ALTER TABLE judging.assignments 
    ADD COLUMN IF NOT EXISTS started_at TIMESTAMP;

-- Add status constraint
ALTER TABLE judging.assignments 
    DROP CONSTRAINT IF EXISTS valid_status;

ALTER TABLE judging.assignments 
    ADD CONSTRAINT valid_status 
    CHECK (status IN ('pending', 'in_progress', 'completed', 'recused'));

-- Add recusal support
ALTER TABLE judging.assignments 
    ADD COLUMN IF NOT EXISTS recused_at TIMESTAMP;

ALTER TABLE judging.assignments 
    ADD COLUMN IF NOT EXISTS recusal_reason VARCHAR(255);

-- =====================================================
-- 5. MISSING INDEXES FOR PERFORMANCE
-- =====================================================

CREATE INDEX IF NOT EXISTS idx_scores_assignment ON judging.scores(assignment_id);
CREATE INDEX IF NOT EXISTS idx_assignments_phase ON judging.assignments(phase_id);
CREATE INDEX IF NOT EXISTS idx_assignments_rubric ON judging.assignments(rubric_id);
CREATE INDEX IF NOT EXISTS idx_assignments_status_phase ON judging.assignments(status, phase_id);

-- GIN index for scores JSONB (query by criterion name)
CREATE INDEX IF NOT EXISTS idx_scores_data ON judging.scores USING GIN (scores);

-- =====================================================
-- 6. HELPER FUNCTIONS
-- =====================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION judging.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for phases table
DROP TRIGGER IF EXISTS update_phases_updated_at ON judging.phases;
CREATE TRIGGER update_phases_updated_at
    BEFORE UPDATE ON judging.phases
    FOR EACH ROW
    EXECUTE FUNCTION judging.update_updated_at_column();

-- Trigger for rubric_versions (auto-increment version)
CREATE OR REPLACE FUNCTION judging.increment_rubric_version()
RETURNS TRIGGER AS $$
BEGIN
    SELECT COALESCE(MAX(version), 0) + 1 INTO NEW.version
    FROM judging.rubric_versions
    WHERE rubric_id = NEW.rubric_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS increment_rubric_version ON judging.rubric_versions;
CREATE TRIGGER increment_rubric_version
    BEFORE INSERT ON judging.rubric_versions
    FOR EACH ROW
    EXECUTE FUNCTION judging.increment_rubric_version();

-- =====================================================
-- 7. SEED DATA (optional, for testing)
-- =====================================================

-- Default phase for backward compatibility
INSERT INTO judging.phases (id, name, type, status, opens_at, closes_at)
VALUES (
    '00000000-0000-0000-0000-000000000001'::UUID,
    'Main Judging',
    'custom',
    'open',
    NOW(),
    NOW() + INTERVAL '30 days'
) ON CONFLICT DO NOTHING;

-- Update existing assignments to use default phase
UPDATE judging.assignments 
SET phase_id = '00000000-0000-0000-0000-000000000001'::UUID
WHERE phase_id IS NULL;

COMMENT ON TABLE judging.phases IS 'Judging phases for multi-round competitions (preliminary, semifinal, final)';
COMMENT ON TABLE judging.rubric_versions IS 'Immutable rubric snapshots for score consistency';
COMMENT ON TABLE judging.judge_stats IS 'Judge scoring statistics for normalization';
COMMENT ON COLUMN judging.phases.type IS 'preliminary, semifinal, final, or custom';
COMMENT ON COLUMN judging.phases.status IS 'draft, open (accepting scores), closed (no more scores), finalized (results locked)';
COMMENT ON COLUMN judging.phases.scoring_method IS 'raw (no normalization), normalized_zscore, normalized_minmax';
COMMENT ON COLUMN judging.phases.advancement_count IS 'Number of top teams to auto-advance to next phase';
COMMENT ON COLUMN judging.assignments.phase_id IS 'Phase this assignment belongs to';
COMMENT ON COLUMN judging.scores.phase_id IS 'Denormalized phase reference for faster queries';
COMMENT ON COLUMN judging.assignments.started_at IS 'When judge started evaluating (first score submission)';
COMMENT ON COLUMN judging.assignments.recused_at IS 'When judge recused themselves (conflict of interest)';
