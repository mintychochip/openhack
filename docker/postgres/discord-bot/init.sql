-- Discord Bot Service Schema
-- Initializes the discord_bot schema and all required tables.

CREATE SCHEMA IF NOT EXISTS discord_bot;

CREATE TABLE discord_bot.channel_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    guild_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(event_type, channel_id)
);

CREATE TABLE discord_bot.user_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    discord_user_id TEXT NOT NULL UNIQUE,
    openhack_user_id UUID NOT NULL,
    linked_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE discord_bot.interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    discord_user_id TEXT NOT NULL,
    interaction_type TEXT NOT NULL,
    command_name TEXT,
    channel_id TEXT,
    guild_id TEXT,
    payload JSONB,
    response_status TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE discord_bot.link_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    discord_user_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    claimed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_interactions_discord_user ON discord_bot.interactions (discord_user_id);
CREATE INDEX idx_interactions_created_at ON discord_bot.interactions (created_at DESC);
CREATE INDEX idx_link_tokens_token ON discord_bot.link_tokens (token) WHERE NOT claimed;
CREATE INDEX idx_user_links_openhack ON discord_bot.user_links (openhack_user_id);
