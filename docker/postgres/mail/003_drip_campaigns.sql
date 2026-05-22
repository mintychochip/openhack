-- Mail Service - Drip Campaigns
-- Creates drip campaign system with automated email scheduling

CREATE TABLE IF NOT EXISTS mail.drip_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_event VARCHAR(100) NOT NULL,  -- e.g., "user.registered", "team.formed", "project.submitted"
    status VARCHAR(50) DEFAULT 'draft',  -- draft, active, paused, archived
    created_by UUID,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS mail.drip_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES mail.drip_campaigns(id) ON DELETE CASCADE,
    step_order INT NOT NULL,
    delay_hours INT NOT NULL DEFAULT 0,  -- Hours after trigger or previous step
    template_id UUID REFERENCES mail.templates(id),
    subject VARCHAR(500) NOT NULL,
    body_html TEXT,
    body_text TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(campaign_id, step_order)
);

CREATE TABLE IF NOT EXISTS mail.drip_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES mail.drip_campaigns(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    trigger_event VARCHAR(100),
    trigger_data JSONB,  -- Context data for personalization
    current_step INT DEFAULT 0,
    status VARCHAR(50) DEFAULT 'active',  -- active, completed, unsubscribed, bounced
    enrolled_at TIMESTAMP DEFAULT NOW(),
    next_step_at TIMESTAMP,
    completed_at TIMESTAMP,
    UNIQUE(campaign_id, user_id)
);

CREATE TABLE IF NOT EXISTS mail.drip_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    enrollment_id UUID NOT NULL REFERENCES mail.drip_enrollments(id) ON DELETE CASCADE,
    step_id UUID REFERENCES mail.drip_steps(id),
    status VARCHAR(50) DEFAULT 'pending',  -- pending, sent, failed, skipped
    sent_at TIMESTAMP,
    error_message TEXT,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_campaigns_trigger ON mail.drip_campaigns(trigger_event);
CREATE INDEX IF NOT EXISTS idx_campaigns_status ON mail.drip_campaigns(status);
CREATE INDEX IF NOT EXISTS idx_steps_campaign ON mail.drip_steps(campaign_id);
CREATE INDEX IF NOT EXISTS idx_enrollments_user ON mail.drip_enrollments(user_id);
CREATE INDEX IF NOT EXISTS idx_enrollments_campaign ON mail.drip_enrollments(campaign_id);
CREATE INDEX IF NOT EXISTS idx_enrollments_next_step ON mail.drip_enrollments(next_step_at);
CREATE INDEX IF NOT EXISTS idx_enrollments_status ON mail.drip_enrollments(status);
CREATE INDEX IF NOT EXISTS idx_logs_enrollment ON mail.drip_logs(enrollment_id);
CREATE INDEX IF NOT EXISTS idx_logs_status ON mail.drip_logs(status);

-- Auto-update updated_at
CREATE OR REPLACE FUNCTION mail.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_drip_campaigns_updated_at ON mail.drip_campaigns;
CREATE TRIGGER update_drip_campaigns_updated_at
    BEFORE UPDATE ON mail.drip_campaigns
    FOR EACH ROW
    EXECUTE FUNCTION mail.update_updated_at_column();

COMMENT ON TABLE mail.drip_campaigns IS 'Email drip campaign definitions';
COMMENT ON TABLE mail.drip_steps IS 'Individual steps in a drip campaign with timing and templates';
COMMENT ON TABLE mail.drip_enrollments IS 'User enrollments in drip campaigns with progress tracking';
COMMENT ON TABLE mail.drip_logs IS 'Audit log of sent/skipped/failed drip emails';

COMMENT ON COLUMN mail.drip_campaigns.trigger_event IS 'Event that triggers enrollment (e.g., user.registered)';
COMMENT ON COLUMN mail.drip_steps.delay_hours IS 'Hours to wait after trigger or previous step before sending';
COMMENT ON COLUMN mail.drip_enrollments.current_step IS 'Current step number (0 = not started)';
COMMENT ON COLUMN mail.drip_enrollments.next_step_at IS 'When the next step should be processed';
COMMENT ON COLUMN mail.drip_enrollments.trigger_data IS 'JSON context for personalization (user name, team, etc.)';
COMMENT ON COLUMN mail.drip_logs.status IS 'pending, sent, failed, or skipped (user unsubscribed)';
