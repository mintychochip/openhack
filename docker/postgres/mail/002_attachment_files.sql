-- Mail Service Migration: Add attachment file support
-- Adds file upload capability for email attachments

-- Add attachment_file_ids to messages table (array of UUIDs)
ALTER TABLE mail.email_queue
ADD COLUMN IF NOT EXISTS attachment_file_ids UUID[],
ADD COLUMN IF NOT EXISTS attachment_urls TEXT[];

CREATE INDEX IF NOT EXISTS idx_email_queue_attachments
ON mail.email_queue USING GIN (attachment_file_ids);

COMMENT ON COLUMN mail.email_queue.attachment_file_ids IS 'Array of uploaded attachment file IDs (references media.files)';
COMMENT ON COLUMN mail.email_queue.attachment_urls IS 'Array of signed URLs for attachment access';
