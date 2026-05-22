-- Migration: Add font_config to hackathon_config for typography theming
-- Author: OpenHack AI Agent
-- Date: 2026-05-22

ALTER TABLE core.hackathon_config
    ADD COLUMN IF NOT EXISTS font_config JSONB DEFAULT '{}'::jsonb;
