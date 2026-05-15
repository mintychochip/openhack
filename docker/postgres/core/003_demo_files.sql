-- Core Service Migration: Add demo file support
-- Adds file upload capability for project demos

-- Add demo_file_id to projects table
ALTER TABLE core.projects 
ADD COLUMN IF NOT EXISTS demo_file_id UUID,
ADD COLUMN IF NOT EXISTS demo_file_url TEXT;

-- Add index for demo file lookups
CREATE INDEX IF NOT EXISTS idx_projects_demo_file ON core.projects(demo_file_id) 
WHERE demo_file_id IS NOT NULL;

-- Add demo upload endpoint permission (document only, no DB changes needed)
COMMENT ON COLUMN core.projects.demo_file_id IS 'Uploaded demo file ID (references media.files)';
COMMENT ON COLUMN core.projects.demo_file_url IS 'Signed URL for demo file access';
