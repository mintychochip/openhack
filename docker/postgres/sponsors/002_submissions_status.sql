-- Add sponsor_id column to booths for identity-driven routes
ALTER TABLE sponsor.booths ADD COLUMN IF NOT EXISTS sponsor_id VARCHAR(255);
CREATE INDEX IF NOT EXISTS idx_booths_sponsor_id ON sponsor.booths(sponsor_id);

-- Add status column to submissions
ALTER TABLE sponsor.submissions ADD COLUMN IF NOT EXISTS status VARCHAR(50) DEFAULT 'pending';

-- Comments for documentation
COMMENT ON COLUMN sponsor.booths.sponsor_id IS 'Authenticated user ID from JWT, used as sponsor identifier for identity-driven routes';
COMMENT ON COLUMN sponsor.submissions.status IS 'Submission status: pending, approved, rejected, or awarded';
