-- Sponsor Service - Booth Interactivity
-- Creates visitor tracking, Q&A, and polls for sponsor booths

CREATE TABLE IF NOT EXISTS sponsor.booth_visitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booth_id UUID NOT NULL REFERENCES sponsor.booths(id) ON DELETE CASCADE,
    user_id UUID,  -- NULL for anonymous visitors
    visited_at TIMESTAMP DEFAULT NOW(),
    duration_seconds INT,  -- Time spent in booth
    source VARCHAR(50),  -- How they arrived: browse, search, recommendation, qr_code
    metadata JSONB,  -- Additional tracking data
    UNIQUE(booth_id, user_id, visited_at)
);

CREATE TABLE IF NOT EXISTS sponsor.booth_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booth_id UUID NOT NULL REFERENCES sponsor.booths(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    message TEXT NOT NULL,
    is_sponsor_reply BOOLEAN DEFAULT FALSE,
    parent_message_id UUID REFERENCES sponsor.booth_messages(id),
    created_at TIMESTAMP DEFAULT NOW(),
    is_read BOOLEAN DEFAULT FALSE,
    read_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sponsor.booth_polls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booth_id UUID NOT NULL REFERENCES sponsor.booths(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    description TEXT,
    options JSONB NOT NULL,  -- [{text: "Option A", color: "#FF0000"}, ...]
    allow_multiple BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    ends_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    created_by UUID  -- Sponsor user ID
);

CREATE TABLE IF NOT EXISTS sponsor.booth_poll_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID NOT NULL REFERENCES sponsor.booth_polls(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    option_indices INT[] NOT NULL,  -- Array of selected option indices
    responded_at TIMESTAMP DEFAULT NOW(),
    metadata JSONB,
    UNIQUE(poll_id, user_id)
);

CREATE TABLE IF NOT EXISTS sponsor.booth_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booth_id UUID NOT NULL REFERENCES sponsor.booths(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    resource_type VARCHAR(50) NOT NULL,  -- document, video, link, demo, sandbox
    url TEXT NOT NULL,
    download_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_visitors_booth ON sponsor.booth_visitors(booth_id);
CREATE INDEX IF NOT EXISTS idx_visitors_user ON sponsor.booth_visitors(user_id);
CREATE INDEX IF NOT EXISTS idx_visitors_visited_at ON sponsor.booth_visitors(visited_at);
CREATE INDEX IF NOT EXISTS idx_messages_booth ON sponsor.booth_messages(booth_id);
CREATE INDEX IF NOT EXISTS idx_messages_user ON sponsor.booth_messages(user_id);
CREATE INDEX IF NOT EXISTS idx_polls_booth ON sponsor.booth_polls(booth_id);
CREATE INDEX IF NOT EXISTS idx_polls_active ON sponsor.booth_polls(is_active);
CREATE INDEX IF NOT EXISTS idx_poll_responses_poll ON sponsor.booth_poll_responses(poll_id);
CREATE INDEX IF NOT EXISTS idx_poll_responses_user ON sponsor.booth_poll_responses(user_id);
CREATE INDEX IF NOT EXISTS idx_resources_booth ON sponsor.booth_resources(booth_id);

-- Auto-increment view_count on booth visit
CREATE OR REPLACE FUNCTION sponsor.increment_booth_view_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE sponsor.booths SET view_count = view_count + 1 WHERE id = NEW.booth_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS increment_booth_view_count ON sponsor.booth_visitors;
CREATE TRIGGER increment_booth_view_count
    AFTER INSERT ON sponsor.booth_visitors
    FOR EACH ROW
    EXECUTE FUNCTION sponsor.increment_booth_view_count();

-- Auto-set read_at when is_read becomes true
CREATE OR REPLACE FUNCTION sponsor.mark_message_read()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_read = TRUE AND OLD.is_read = FALSE THEN
        NEW.read_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS mark_message_read ON sponsor.booth_messages;
CREATE TRIGGER mark_message_read
    BEFORE UPDATE ON sponsor.booth_messages
    FOR EACH ROW
    EXECUTE FUNCTION sponsor.mark_message_read();

COMMENT ON TABLE sponsor.booth_visitors IS 'Tracks booth visits for analytics and engagement';
COMMENT ON TABLE sponsor.booth_messages IS 'Q&A messages between attendees and sponsors';
COMMENT ON TABLE sponsor.booth_polls IS 'Interactive polls for booth engagement';
COMMENT ON TABLE sponsor.booth_poll_responses IS 'User responses to booth polls';
COMMENT ON TABLE sponsor.booth_resources IS 'Downloadable resources shared by sponsors';

COMMENT ON COLUMN sponsor.booth_visitors.duration_seconds IS 'Time spent in booth (updated on leave)';
COMMENT ON COLUMN sponsor.booth_visitors.source IS 'How visitor found the booth';
COMMENT ON COLUMN sponsor.booth_messages.is_sponsor_reply IS 'True if message is from sponsor';
COMMENT ON COLUMN sponsor.booth_messages.parent_message_id IS 'For threaded conversations';
COMMENT ON COLUMN sponsor.booth_polls.options IS 'JSON array of {text, color, image_url} objects';
COMMENT ON COLUMN sponsor.booth_resources.resource_type IS 'document, video, link, demo, or sandbox';
