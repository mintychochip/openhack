-- Account Lockout + Email Verification Migration
-- Adds brute-force protection and email verification fields to auth.users

ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER DEFAULT 0 NOT NULL,
ADD COLUMN IF NOT EXISTS locked_until TIMESTAMP NULL,
ADD COLUMN IF NOT EXISTS email_verification_token UUID NULL,
ADD COLUMN IF NOT EXISTS email_verification_token_expires_at TIMESTAMP NULL;

CREATE INDEX IF NOT EXISTS idx_users_locked_until ON auth.users(locked_until);
CREATE INDEX IF NOT EXISTS idx_users_email_verification_token ON auth.users(email_verification_token);

COMMENT ON COLUMN auth.users.failed_login_attempts IS 'Count of consecutive failed login attempts. Reset to 0 on successful login.';
COMMENT ON COLUMN auth.users.locked_until IS 'Timestamp when account lockout expires. NULL means not locked.';
COMMENT ON COLUMN auth.users.email_verification_token IS 'UUID token for email verification. NULL means verified or no token.';
COMMENT ON COLUMN auth.users.email_verification_token_expires_at IS 'Timestamp when verification token expires. NULL means no token.';
