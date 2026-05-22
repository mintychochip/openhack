-- Analytics Service - Certificate Generation
-- Creates certificate tracking with verification codes

CREATE TABLE IF NOT EXISTS analytics.certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    event_id UUID NOT NULL,
    certificate_type VARCHAR(50) NOT NULL,  -- participant, winner, volunteer, mentor, judge
    issued_at TIMESTAMP DEFAULT NOW(),
    verification_code VARCHAR(32) UNIQUE NOT NULL,
    revoked_at TIMESTAMP,
    pdf_file_id UUID,  -- Reference to media service file (optional)
    metadata JSONB,  -- Additional data: {team_name, project_name, placement, hackathon_name}
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_certificates_user ON analytics.certificates(user_id);
CREATE INDEX IF NOT EXISTS idx_certificates_event ON analytics.certificates(event_id);
CREATE INDEX IF NOT EXISTS idx_certificates_verification ON analytics.certificates(verification_code);
CREATE INDEX IF NOT EXISTS idx_certificates_type ON analytics.certificates(certificate_type);

-- Auto-update updated_at
CREATE OR REPLACE FUNCTION analytics.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_certificates_updated_at ON analytics.certificates;
CREATE TRIGGER update_certificates_updated_at
    BEFORE UPDATE ON analytics.certificates
    FOR EACH ROW
    EXECUTE FUNCTION analytics.update_updated_at_column();

COMMENT ON TABLE analytics.certificates IS 'Certificate issuance tracking with verification codes';
COMMENT ON COLUMN analytics.certificates.certificate_type IS 'participant, winner, volunteer, mentor, or judge';
COMMENT ON COLUMN analytics.certificates.verification_code IS 'Public verification code (32 char random string)';
COMMENT ON COLUMN analytics.certificates.metadata IS 'JSON with team_name, project_name, placement, hackathon_name, etc.';
COMMENT ON COLUMN analytics.certificates.pdf_file_id IS 'Optional reference to generated PDF in media service';
