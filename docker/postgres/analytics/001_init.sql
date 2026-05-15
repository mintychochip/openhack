-- Analytics Service Database Initialization
-- Creates schema and tables for event tracking, metrics, and reports

CREATE SCHEMA IF NOT EXISTS analytics;

-- Events table for tracking all platform events
CREATE TABLE IF NOT EXISTS analytics.events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB,
    user_id UUID,
    team_id UUID,
    project_id UUID,
    occurred_at TIMESTAMP DEFAULT NOW()
);

-- Metrics table for aggregated metrics
CREATE TABLE IF NOT EXISTS analytics.metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL,
    dimensions JSONB,
    recorded_at TIMESTAMP DEFAULT NOW()
);

-- Reports table for generated reports
CREATE TABLE IF NOT EXISTS analytics.reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_type VARCHAR(50) NOT NULL,
    generated_at TIMESTAMP DEFAULT NOW(),
    data JSONB
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_events_type ON analytics.events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_occurred ON analytics.events(occurred_at);
CREATE INDEX IF NOT EXISTS idx_events_user ON analytics.events(user_id);
CREATE INDEX IF NOT EXISTS idx_events_team ON analytics.events(team_id);
CREATE INDEX IF NOT EXISTS idx_events_project ON analytics.events(project_id);
CREATE INDEX IF NOT EXISTS idx_metrics_name ON analytics.metrics(metric_name);
CREATE INDEX IF NOT EXISTS idx_metrics_recorded ON analytics.metrics(recorded_at);
CREATE INDEX IF NOT EXISTS idx_reports_type ON analytics.reports(report_type);
