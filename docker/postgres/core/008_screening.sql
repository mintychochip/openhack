-- Submission Screening
-- Migration 008

-- Add screening fields to projects
ALTER TABLE core.projects
ADD COLUMN IF NOT EXISTS screening_status TEXT DEFAULT 'pending' CHECK (screening_status IN ('pending', 'approved', 'rejected', 'needs_info')),
ADD COLUMN IF NOT EXISTS screened_by UUID REFERENCES auth.users(id),
ADD COLUMN IF NOT EXISTS screened_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS screening_notes TEXT;

-- Index for screening queue
CREATE INDEX IF NOT EXISTS idx_projects_screening_status 
ON core.projects(screening_status, created_at DESC) 
WHERE screening_status = 'pending';

-- Add submission requirements to hackathon config
ALTER TABLE core.hackathon_config
ADD COLUMN IF NOT EXISTS require_repo_url BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS require_demo_url BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS require_description_min_length INTEGER DEFAULT 100;

-- Comments
COMMENT ON COLUMN core.projects.screening_status IS 'Screening status: pending, approved, rejected, needs_info';
COMMENT ON COLUMN core.projects.screened_by IS 'Admin/judge who performed screening';
COMMENT ON COLUMN core.projects.screened_at IS 'When screening was performed';
COMMENT ON COLUMN core.projects.screening_notes IS 'Notes from screening (visible to judges/admins)';
COMMENT ON COLUMN core.hackathon_config.require_repo_url IS 'Whether repo URL is required for submission';
COMMENT ON COLUMN core.hackathon_config.require_demo_url IS 'Whether demo URL is required for submission';
COMMENT ON COLUMN core.hackathon_config.require_description_min_length IS 'Minimum description length for submission';
