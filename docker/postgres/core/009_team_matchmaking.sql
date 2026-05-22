-- Team Formation/Matchmaking
-- Migration 009

-- Add team-seeking fields to users (stored in auth.users for simplicity)
ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS looking_for_team BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS team_preferences JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS availability TEXT,
ADD COLUMN IF NOT EXISTS timezone TEXT;

-- Index for finding team-seekers
CREATE INDEX IF NOT EXISTS idx_users_looking_for_team 
ON auth.users(looking_for_team) 
WHERE looking_for_team = TRUE;

-- GIN index for JSONB preferences (skill matching)
CREATE INDEX IF NOT EXISTS idx_users_team_preferences 
ON auth.users USING GIN(team_preferences);

-- Add skills column to users if not exists (for matching)
ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS skills TEXT[] DEFAULT '{}';

-- Index for skill search
CREATE INDEX IF NOT EXISTS idx_users_skills ON auth.users USING GIN(skills);

-- Add seeking fields to teams
ALTER TABLE core.teams
ADD COLUMN IF NOT EXISTS looking_for_members BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS seeking_skills TEXT[] DEFAULT '{}',
ADD COLUMN IF NOT EXISTS team_description TEXT;

-- Index for teams seeking members
CREATE INDEX IF NOT EXISTS idx_teams_looking_for_members 
ON core.teams(looking_for_members) 
WHERE looking_for_members = TRUE;

-- Index for skill-based team search
CREATE INDEX IF NOT EXISTS idx_teams_seeking_skills ON core.teams USING GIN(seeking_skills);

-- Comments
COMMENT ON COLUMN auth.users.looking_for_team IS 'User is actively looking for a team';
COMMENT ON COLUMN auth.users.team_preferences IS 'JSON: {desired_skills: [], min_team_size, max_team_size, preferences}';
COMMENT ON COLUMN auth.users.availability IS 'Time commitment: full-time, part-time, weekends, etc.';
COMMENT ON COLUMN auth.users.timezone IS 'User timezone for team matching';
COMMENT ON COLUMN auth.users.skills IS 'User skills/tags for matching';
COMMENT ON COLUMN core.teams.looking_for_members IS 'Team is actively recruiting';
COMMENT ON COLUMN core.teams.seeking_skills IS 'Skills the team is looking for';
COMMENT ON COLUMN core.teams.team_description IS 'Team pitch/description';
