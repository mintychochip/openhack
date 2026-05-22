-- Project Showcase/Gallery
-- Migration 007

-- Add featured flag and showcase ordering to projects
ALTER TABLE core.projects
ADD COLUMN IF NOT EXISTS featured BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS showcase_order INTEGER DEFAULT 0;

-- Create index for featured projects ordering
CREATE INDEX IF NOT EXISTS idx_projects_featured_order 
ON core.projects(featured DESC, showcase_order ASC) 
WHERE featured = TRUE;

-- Create favorites table
CREATE TABLE IF NOT EXISTS core.project_favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES core.projects(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, project_id)
);

-- Indexes for favorites
CREATE INDEX IF NOT EXISTS idx_favorites_project ON core.project_favorites(project_id);
CREATE INDEX IF NOT EXISTS idx_favorites_user ON core.project_favorites(user_id);
CREATE INDEX IF NOT EXISTS idx_favorites_created ON core.project_favorites(created_at DESC);

-- Add favorite count cache to projects (denormalized for performance)
ALTER TABLE core.projects
ADD COLUMN IF NOT EXISTS favorite_count INTEGER DEFAULT 0;

-- Trigger to update favorite_count
CREATE OR REPLACE FUNCTION core.update_favorite_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE core.projects SET favorite_count = favorite_count + 1 WHERE id = NEW.project_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE core.projects SET favorite_count = favorite_count - 1 WHERE id = OLD.project_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_favorite_count ON core.project_favorites;
CREATE TRIGGER trg_favorite_count
    AFTER INSERT OR DELETE ON core.project_favorites
    FOR EACH ROW EXECUTE FUNCTION core.update_favorite_count();

-- Comments
COMMENT ON COLUMN core.projects.featured IS 'Whether project is featured in showcase';
COMMENT ON COLUMN core.projects.showcase_order IS 'Order in showcase (lower = first)';
COMMENT ON COLUMN core.projects.favorite_count IS 'Cached count of favorites (denormalized)';
COMMENT ON TABLE core.project_favorites IS 'User favorites/bookmarks for projects';
