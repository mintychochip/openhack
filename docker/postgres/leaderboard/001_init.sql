CREATE SCHEMA IF NOT EXISTS leaderboard;

CREATE TABLE leaderboard.ranks (
    team_id UUID PRIMARY KEY,
    total_score DECIMAL(10,2) DEFAULT 0,
    public_votes INT DEFAULT 0,
    rank INT,
    previous_rank INT,
    last_updated TIMESTAMP DEFAULT NOW()
);

CREATE TABLE leaderboard.score_history (
    id BIGSERIAL PRIMARY KEY,
    team_id UUID NOT NULL,
    score DECIMAL(10,2),
    rank INT,
    recorded_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE leaderboard.votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    voter_token_hash VARCHAR(255) NOT NULL,
    voter_ip VARCHAR(45),
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(voter_token_hash, project_id)
);

CREATE INDEX idx_ranks_score ON leaderboard.ranks(total_score DESC);
CREATE INDEX idx_votes_project ON leaderboard.votes(project_id);
CREATE INDEX idx_score_history_team ON leaderboard.score_history(team_id);
