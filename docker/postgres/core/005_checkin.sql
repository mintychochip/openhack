-- Event Check-in System with QR Codes
-- Adds QR code-based check-in for events

-- Add checked_in column to event_rsvps
ALTER TABLE core.event_rsvps
ADD COLUMN IF NOT EXISTS checked_in BOOLEAN DEFAULT FALSE;

-- Create index for checked_in
CREATE INDEX IF NOT EXISTS idx_event_rsvps_checked_in ON core.event_rsvps(checked_in);

-- Create event_checkins table for QR code management
CREATE TABLE IF NOT EXISTS core.event_checkins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES core.events(id) ON DELETE CASCADE,
    user_id UUID,
    qr_code VARCHAR(64) UNIQUE NOT NULL,
    checked_in BOOLEAN DEFAULT FALSE NOT NULL,
    checked_in_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_event_checkins_event ON core.event_checkins(event_id);
CREATE INDEX idx_event_checkins_user ON core.event_checkins(user_id);
CREATE INDEX idx_event_checkins_qr_code ON core.event_checkins(qr_code);
CREATE INDEX idx_event_checkins_checked_in ON core.event_checkins(checked_in);
CREATE INDEX idx_event_checkins_checked_in_at ON core.event_checkins(checked_in_at);

-- Comments
COMMENT ON TABLE core.event_checkins IS 'QR code-based event check-in records';
COMMENT ON COLUMN core.event_checkins.qr_code IS 'Unique QR code string (generated per event)';
COMMENT ON COLUMN core.event_checkins.checked_in IS 'Whether QR code has been scanned';
COMMENT ON COLUMN core.event_checkins.user_id IS 'User who scanned the QR code (NULL until scanned)';
COMMENT ON COLUMN core.event_checkins.checked_in_at IS 'Timestamp when QR code was scanned';

-- Add comment to event_rsvps.checked_in
COMMENT ON COLUMN core.event_rsvps.checked_in IS 'Whether user checked in via QR code';
