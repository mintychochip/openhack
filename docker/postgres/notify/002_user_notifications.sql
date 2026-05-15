-- Notify Service: Add user_notifications table required by notify-rust service

CREATE TABLE IF NOT EXISTS notify.user_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    type VARCHAR(100) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    data JSONB,
    action_url TEXT,
    is_read BOOLEAN DEFAULT FALSE,
    read_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_notifications_user_id ON notify.user_notifications (user_id);
CREATE INDEX IF NOT EXISTS idx_user_notifications_is_read ON notify.user_notifications (is_read) WHERE is_read = FALSE;
CREATE INDEX IF NOT EXISTS idx_user_notifications_type ON notify.user_notifications (type);
CREATE INDEX IF NOT EXISTS idx_user_notifications_created_at ON notify.user_notifications (created_at DESC);
