-- Performance indexes for OpenHack database
-- Additional composite and text-search indexes beyond those in per-service init scripts

-- =============================================================================
-- AUTH SERVICE: Additional composite indexes
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_sessions_user_active
ON auth.sessions (user_id, expires_at);

-- =============================================================================
-- CORE SERVICE: Additional composite indexes
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_projects_team_status
ON core.projects (team_id, status);

CREATE INDEX IF NOT EXISTS idx_team_members_user_team
ON core.team_members (user_id, team_id);

-- =============================================================================
-- MAIL SERVICE: Additional composite indexes
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_email_queue_unsent
ON mail.email_queue (status, scheduled_at)
WHERE status = 'pending';

-- =============================================================================
-- JUDGING SERVICE: Additional composite indexes
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_assignments_judge_pending
ON judging.assignments (judge_id, status)
WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_scores_assignment_id
ON judging.scores (assignment_id);

-- =============================================================================
-- LEADERBOARD SERVICE: Additional composite indexes
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_votes_project_time
ON leaderboard.votes (project_id, created_at);

-- =============================================================================
-- NOTIFY SERVICE: Additional composite indexes
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_webhooks_active_only
ON notify.webhooks (id)
WHERE active = TRUE;

-- =============================================================================
-- SPONSORS SERVICE: Additional composite indexes
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_prizes_announced
ON sponsor.prizes (announced)
WHERE announced = TRUE;

CREATE INDEX IF NOT EXISTS idx_submissions_project
ON sponsor.submissions (project_id);

CREATE INDEX IF NOT EXISTS idx_booths_published_only
ON sponsor.booths (id)
WHERE published = TRUE;

-- =============================================================================
-- MEDIA SERVICE: Additional composite indexes
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_files_owner_folder
ON media.files (uploader_id, category);

-- =============================================================================
-- TEXT SEARCH INDEXES (core.projects, core.teams, auth.users)
-- =============================================================================

ALTER TABLE core.projects ADD COLUMN IF NOT EXISTS search_vector tsvector;
CREATE INDEX IF NOT EXISTS idx_projects_search ON core.projects USING GIN (search_vector);

CREATE OR REPLACE FUNCTION update_projects_search_vector() RETURNS trigger AS $$
BEGIN
  NEW.search_vector := to_tsvector('english', COALESCE(NEW.name, '') || ' ' || COALESCE(NEW.description, ''));
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_search_vector ON core.projects;
CREATE TRIGGER update_search_vector
  BEFORE INSERT OR UPDATE ON core.projects
  FOR EACH ROW
  EXECUTE FUNCTION update_projects_search_vector();

ALTER TABLE core.teams ADD COLUMN IF NOT EXISTS search_vector tsvector;
CREATE INDEX IF NOT EXISTS idx_teams_search ON core.teams USING GIN (search_vector);

CREATE OR REPLACE FUNCTION update_teams_search_vector() RETURNS trigger AS $$
BEGIN
  NEW.search_vector := to_tsvector('english', COALESCE(NEW.name, '') || ' ' || COALESCE(NEW.description, ''));
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_teams_search_vector ON core.teams;
CREATE TRIGGER update_teams_search_vector
  BEFORE INSERT OR UPDATE ON core.teams
  FOR EACH ROW
  EXECUTE FUNCTION update_teams_search_vector();

-- =============================================================================
-- VACUUM AND ANALYZE (refresh query planner statistics)
-- =============================================================================

ANALYZE auth.users;
ANALYZE auth.sessions;
ANALYZE auth.oauth_accounts;
ANALYZE auth.password_reset_tokens;
ANALYZE core.teams;
ANALYZE core.team_members;
ANALYZE core.projects;
ANALYZE core.challenges;
ANALYZE core.challenge_participants;
ANALYZE mail.email_templates;
ANALYZE mail.email_queue;
ANALYZE mail.email_logs;
ANALYZE judging.rubrics;
ANALYZE judging.assignments;
ANALYZE judging.scores;
ANALYZE leaderboard.ranks;
ANALYZE leaderboard.votes;
ANALYZE leaderboard.score_history;
ANALYZE notify.webhooks;
ANALYZE notify.webhook_deliveries;
ANALYZE notify.announcements;
ANALYZE ai.knowledge;
ANALYZE ai.conversations;
ANALYZE ai.messages;
ANALYZE sponsor.booths;
ANALYZE sponsor.prizes;
ANALYZE sponsor.submissions;
ANALYZE media.files;
