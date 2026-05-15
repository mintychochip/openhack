-- Media Service: Add columns required by media-rust service
-- The original table uses file_name/file_path/file_size/uploader_id/category
-- The Rust service uses original_filename/storage_path/size/storage_provider/uploaded_by/metadata

-- Add columns the Rust service expects
ALTER TABLE media.files
ADD COLUMN IF NOT EXISTS original_filename TEXT,
ADD COLUMN IF NOT EXISTS storage_path TEXT,
ADD COLUMN IF NOT EXISTS size BIGINT,
ADD COLUMN IF NOT EXISTS storage_provider VARCHAR(50) DEFAULT 'local',
ADD COLUMN IF NOT EXISTS uploaded_by UUID,
ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}'::jsonb;

-- Populate new columns from existing data where possible
UPDATE media.files SET
    original_filename = file_name,
    storage_path = file_path,
    size = file_size,
    uploaded_by = uploader_id
WHERE original_filename IS NULL;

-- Add index on storage_provider
CREATE INDEX IF NOT EXISTS idx_files_storage_provider ON media.files (storage_provider);
CREATE INDEX IF NOT EXISTS idx_files_uploaded_by ON media.files (uploaded_by) WHERE uploaded_by IS NOT NULL;

-- Add media schema grant (init.sql may not have it)
GRANT ALL ON SCHEMA media TO openhack;
GRANT ALL ON ALL TABLES IN SCHEMA media TO openhack;
