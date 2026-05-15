-- OpenHack Judging Service: Initial Schema
-- Creates all judging.* tables and indexes.

CREATE SCHEMA IF NOT EXISTS judging;

CREATE TABLE judging.rubrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    criteria JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE judging.assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    judge_id UUID NOT NULL,
    project_id UUID NOT NULL,
    rubric_id UUID REFERENCES judging.rubrics(id),
    status VARCHAR(50) DEFAULT 'pending',
    priority INT DEFAULT 0,
    assigned_at TIMESTAMP DEFAULT NOW(),
    completed_at TIMESTAMP,
    UNIQUE(judge_id, project_id)
);

CREATE TABLE judging.scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID REFERENCES judging.assignments(id),
    scores JSONB NOT NULL,
    total_score DECIMAL(5,2),
    comment TEXT,
    feedback TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_assignments_judge ON judging.assignments(judge_id);
CREATE INDEX idx_assignments_project ON judging.assignments(project_id);
CREATE INDEX idx_assignments_status ON judging.assignments(status);
