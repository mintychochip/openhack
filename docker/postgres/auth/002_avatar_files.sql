-- Auth Service Migration: Add avatar file support
-- Adds file upload capability for user avatars

-- Add avatar_file_id to users table
ALTER TABLE auth.users 
ADD COLUMN IF NOT EXISTS avatar_file_id UUID,
ADD COLUMN IF NOT EXISTS avatar_url TEXT;

-- Add index for avatar lookups
CREATE INDEX IF NOT EXISTS idx_users_avatar ON auth.users(avatar_file_id) 
WHERE avatar_file_id IS NOT NULL;

-- Add avatar upload endpoint permission (document only, no DB changes needed)
COMMENT ON COLUMN auth.users.avatar_file_id IS 'Uploaded avatar file ID (references media.files)';
COMMENT ON COLUMN auth.users.avatar_url IS 'Signed URL for avatar file access';
