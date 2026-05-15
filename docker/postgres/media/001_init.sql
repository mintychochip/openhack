-- Media Service Schema Initialization
-- Creates media schema and files metadata table

CREATE SCHEMA IF NOT EXISTS media;

CREATE TABLE IF NOT EXISTS media.files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('avatar', 'demo', 'attachment', 'booth', 'other')),
    uploader_id UUID NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP
);

CREATE INDEX idx_files_uploader_id ON media.files(uploader_id);
CREATE INDEX idx_files_category ON media.files(category);
CREATE INDEX idx_files_created_at ON media.files(created_at);
CREATE INDEX idx_files_deleted_at ON media.files(deleted_at) WHERE deleted_at IS NOT NULL;

CREATE OR REPLACE FUNCTION media.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_media_files_updated_at
    BEFORE UPDATE ON media.files
    FOR EACH ROW
    EXECUTE FUNCTION media.update_updated_at_column();
