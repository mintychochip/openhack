-- Global Search Support (PostgreSQL full-text search)
-- Migration 010

-- Add tsvector columns for full-text search on projects
ALTER TABLE core.projects
ADD COLUMN IF NOT EXISTS search_vector TSVECTOR;

-- Populate search vector
UPDATE core.projects SET search_vector = 
    setweight(to_tsvector('english', COALESCE(title, '')), 'A') ||
    setweight(to_tsvector('english', COALESCE(description, '')), 'B') ||
    setweight(to_tsvector('simple', COALESCE(array_to_string(tags, ' '), '')), 'C')
WHERE search_vector IS NULL;

-- Index for project search
CREATE INDEX IF NOT EXISTS idx_projects_search_vector 
ON core.projects USING GIN(search_vector);

-- Trigger to update search vector on project changes
CREATE OR REPLACE FUNCTION core.projects_search_vector_update()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.description, '')), 'B') ||
        setweight(to_tsvector('simple', COALESCE(array_to_string(NEW.tags, ' '), '')), 'C');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_projects_search_vector ON core.projects;
CREATE TRIGGER trg_projects_search_vector
    BEFORE INSERT OR UPDATE ON core.projects
    FOR EACH ROW EXECUTE FUNCTION core.projects_search_vector_update();

-- Add tsvector for teams
ALTER TABLE core.teams
ADD COLUMN IF NOT EXISTS search_vector TSVECTOR;

UPDATE core.teams SET search_vector = 
    setweight(to_tsvector('english', COALESCE(name, '')), 'A') ||
    setweight(to_tsvector('english', COALESCE(description, '')), 'B')
WHERE search_vector IS NULL;

CREATE INDEX IF NOT EXISTS idx_teams_search_vector 
ON core.teams USING GIN(search_vector);

CREATE OR REPLACE FUNCTION core.teams_search_vector_update()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.name, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.description, '')), 'B');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_teams_search_vector ON core.teams;
CREATE TRIGGER trg_teams_search_vector
    BEFORE INSERT OR UPDATE ON core.teams
    FOR EACH ROW EXECUTE FUNCTION core.teams_search_vector_update();

-- Add tsvector for users (for skill/name search)
ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS search_vector TSVECTOR;

UPDATE auth.users SET search_vector = 
    setweight(to_tsvector('english', COALESCE(name, '')), 'A') ||
    setweight(to_tsvector('english', COALESCE(email, '')), 'B') ||
    setweight(to_tsvector('simple', COALESCE(array_to_string(skills, ' '), '')), 'C')
WHERE search_vector IS NULL;

CREATE INDEX IF NOT EXISTS idx_users_search_vector 
ON auth.users USING GIN(search_vector);

CREATE OR REPLACE FUNCTION auth.users_search_vector_update()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.name, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.email, '')), 'B') ||
        setweight(to_tsvector('simple', COALESCE(array_to_string(NEW.skills, ' '), '')), 'C');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_users_search_vector ON auth.users;
CREATE TRIGGER trg_users_search_vector
    BEFORE INSERT OR UPDATE ON auth.users
    FOR EACH ROW EXECUTE FUNCTION auth.users_search_vector_update();

COMMENT ON COLUMN core.projects.search_vector IS 'Full-text search vector for title, description, tags';
COMMENT ON COLUMN core.teams.search_vector IS 'Full-text search vector for name, description';
COMMENT ON COLUMN auth.users.search_vector IS 'Full-text search vector for name, email, skills';
