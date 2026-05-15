CREATE SCHEMA IF NOT EXISTS notify;

CREATE TABLE notify.announcements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    audience VARCHAR(50) NOT NULL,
    channels TEXT[],
    status VARCHAR(50) DEFAULT 'draft',
    scheduled_at TIMESTAMP,
    sent_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE notify.webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    events TEXT[],
    secret VARCHAR(255),
    active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE notify.webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID REFERENCES notify.webhooks(id) ON DELETE CASCADE,
    event_type VARCHAR(100),
    payload JSONB,
    status VARCHAR(50),
    response_code INT,
    response_body TEXT,
    attempts INT DEFAULT 0,
    next_retry_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_announcements_status ON notify.announcements(status);
CREATE INDEX idx_announcements_created_at ON notify.announcements(created_at DESC);
CREATE INDEX idx_webhooks_active ON notify.webhooks(active);
CREATE INDEX idx_webhook_deliveries_webhook_id ON notify.webhook_deliveries(webhook_id);
CREATE INDEX idx_webhook_deliveries_status ON notify.webhook_deliveries(status);
CREATE INDEX idx_webhook_deliveries_next_retry ON notify.webhook_deliveries(next_retry_at) WHERE status = 'pending';
