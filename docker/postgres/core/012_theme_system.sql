-- Theme System Migration
-- Adds DaisyUI theme support and custom CSS to hackathon_config

ALTER TABLE core.hackathon_config
ADD COLUMN IF NOT EXISTS custom_css TEXT DEFAULT '',
ADD COLUMN IF NOT EXISTS daisyui_theme_preset VARCHAR(50) DEFAULT 'light',
ADD COLUMN IF NOT EXISTS daisyui_custom_theme JSONB DEFAULT '{}'::jsonb;

COMMENT ON COLUMN core.hackathon_config.custom_css IS 'Custom CSS injected into every page';
COMMENT ON COLUMN core.hackathon_config.daisyui_theme_preset IS 'Active DaisyUI theme preset name (light, dark, cupcake, etc.)';
COMMENT ON COLUMN core.hackathon_config.daisyui_custom_theme IS 'Custom OKLCH color overrides for semantic colors';
