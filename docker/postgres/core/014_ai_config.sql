-- Add AI provider configuration columns to hackathon_config
ALTER TABLE core.hackathon_config
    ADD COLUMN IF NOT EXISTS ai_provider VARCHAR(50) DEFAULT 'openai',
    ADD COLUMN IF NOT EXISTS ai_enabled BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS openai_api_key TEXT,
    ADD COLUMN IF NOT EXISTS openai_base_url VARCHAR(500) DEFAULT 'https://api.openai.com/v1',
    ADD COLUMN IF NOT EXISTS openai_model VARCHAR(100) DEFAULT 'gpt-4-turbo',
    ADD COLUMN IF NOT EXISTS anthropic_api_key TEXT,
    ADD COLUMN IF NOT EXISTS anthropic_model VARCHAR(100) DEFAULT 'claude-3-5-sonnet-20241022';
