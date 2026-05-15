-- AI Service Database Schema
-- pgvector extension and AI-related tables

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create AI schema
CREATE SCHEMA IF NOT EXISTS ai;

-- Knowledge base table for RAG
CREATE TABLE ai.knowledge (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    category VARCHAR(50),  -- rules, faq, resources, sponsor_api
    tags TEXT[],
    embedding vector(1536),  -- OpenAI embeddings dimension
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Conversations table
CREATE TABLE ai.conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,  -- References auth.users (optional)
    context JSONB,  -- {user_role: str, team_id: UUID, project_id: UUID}
    created_at TIMESTAMP DEFAULT NOW(),
    last_message_at TIMESTAMP DEFAULT NOW()
);

-- Messages table
CREATE TABLE ai.messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID REFERENCES ai.conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL,  -- user, assistant
    content TEXT NOT NULL,
    sources JSONB,  -- [{title: str, url: str, id: UUID}]
    created_at TIMESTAMP DEFAULT NOW()
);

-- Code reviews table (beta feature)
CREATE TABLE ai.code_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID,  -- References core.projects (optional)
    feedback JSONB,  -- [{file: str, line: int, severity: str, message: str, suggestion: str}]
    score INT CHECK (score >= 0 AND score <= 100),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_knowledge_embedding ON ai.knowledge USING ivfflat (embedding vector_cosine_ops);
CREATE INDEX idx_knowledge_category ON ai.knowledge(category);
CREATE INDEX idx_knowledge_tags ON ai.knowledge USING GIN (tags);
CREATE INDEX idx_conversations_user ON ai.conversations(user_id);
CREATE INDEX idx_conversations_last_message ON ai.conversations(last_message_at DESC);
CREATE INDEX idx_messages_conversation ON ai.messages(conversation_id);
CREATE INDEX idx_code_reviews_project ON ai.code_reviews(project_id);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION ai.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add trigger for knowledge table
CREATE TRIGGER update_knowledge_updated_at
    BEFORE UPDATE ON ai.knowledge
    FOR EACH ROW
    EXECUTE FUNCTION ai.update_updated_at_column();

-- Update last_message_at on conversation when message is added
CREATE OR REPLACE FUNCTION ai.update_conversation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE ai.conversations
    SET last_message_at = NEW.created_at
    WHERE id = NEW.conversation_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_conversation_on_message
    AFTER INSERT ON ai.messages
    FOR EACH ROW
    EXECUTE FUNCTION ai.update_conversation_timestamp();
