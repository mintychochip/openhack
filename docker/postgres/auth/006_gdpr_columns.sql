-- GDPR compliance: Add deletion tracking columns
-- Migration 006

ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS deletion_requested_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS deletion_scheduled_at TIMESTAMPTZ;

-- Add index for cleanup job
CREATE INDEX IF NOT EXISTS idx_users_deletion_scheduled 
ON auth.users(deletion_scheduled_at) 
WHERE deletion_scheduled_at IS NOT NULL;

-- Add comment
COMMENT ON COLUMN auth.users.deletion_requested_at IS 'Timestamp when user requested account deletion (GDPR right to be forgotten)';
COMMENT ON COLUMN auth.users.deletion_scheduled_at IS 'Timestamp when account will be anonymized (30 days after request)';
