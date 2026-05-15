-- Auth Service: Add columns required by auth-rust service
-- These columns are referenced by the Rust rewrite but missing from the original schema.

-- Rename 'username' to 'name' (the Rust service uses 'name' everywhere)
ALTER TABLE auth.users RENAME COLUMN username TO name;

-- Add MFA columns
ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS mfa_secret TEXT,
ADD COLUMN IF NOT EXISTS sms_mfa_enabled BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS sms_phone_number VARCHAR(20);

-- Add roles column (text array for RBAC)
ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS roles TEXT[] DEFAULT ARRAY['participant']::text[];

-- Add OAuth identifier columns
ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS github_username VARCHAR(255),
ADD COLUMN IF NOT EXISTS discord_id VARCHAR(255);

-- Add GIN index on roles for role-based queries
CREATE INDEX IF NOT EXISTS idx_users_roles ON auth.users USING GIN (roles);

-- Fix auth.sessions: add revoked_at and device_info, rename for Rust compat
ALTER TABLE auth.sessions
ADD COLUMN IF NOT EXISTS refresh_token_hash VARCHAR(255),
ADD COLUMN IF NOT EXISTS device_info JSONB,
ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMP;

-- Populate refresh_token_hash from existing token_hash if empty
UPDATE auth.sessions SET refresh_token_hash = token_hash WHERE refresh_token_hash IS NULL;

-- Make token_hash nullable (Rust service uses refresh_token_hash instead)
ALTER TABLE auth.sessions ALTER COLUMN token_hash DROP NOT NULL;

-- Drop old unique constraint on token_hash and recreate on refresh_token_hash
DROP INDEX IF EXISTS idx_sessions_token;
CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_refresh_token ON auth.sessions (refresh_token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_revoked ON auth.sessions (revoked_at) WHERE revoked_at IS NULL;

-- Fix auth.oauth_accounts: add updated_at, add provider_user_id (alias for provider_account_id)
ALTER TABLE auth.oauth_accounts
ADD COLUMN IF NOT EXISTS provider_user_id VARCHAR(255),
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- Populate provider_user_id from provider_account_id
UPDATE auth.oauth_accounts SET provider_user_id = provider_account_id WHERE provider_user_id IS NULL;

-- Add unique constraint on (provider, provider_user_id) for Rust queries
CREATE UNIQUE INDEX IF NOT EXISTS idx_oauth_provider_user ON auth.oauth_accounts (provider, provider_user_id);

-- Fix auth.password_reset_tokens: add token (plaintext) and used_at columns
-- Rust service stores/queries the plaintext token, not just the hash
ALTER TABLE auth.password_reset_tokens
ADD COLUMN IF NOT EXISTS token VARCHAR(255),
ADD COLUMN IF NOT EXISTS used_at TIMESTAMP;

-- Add index on token for lookup by plaintext
CREATE INDEX IF NOT EXISTS idx_reset_tokens_token ON auth.password_reset_tokens (token) WHERE token IS NOT NULL;
