-- Extended User Profile Fields
-- Adds bio, skills, interests, and other hackathon-specific fields

ALTER TABLE auth.users
ADD COLUMN IF NOT EXISTS bio TEXT,
ADD COLUMN IF NOT EXISTS skills JSONB DEFAULT '[]'::jsonb,
ADD COLUMN IF NOT EXISTS interests JSONB DEFAULT '[]'::jsonb,
ADD COLUMN IF NOT EXISTS experience_level VARCHAR(50),
ADD COLUMN IF NOT EXISTS organization VARCHAR(255),
ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) DEFAULT 'UTC',
ADD COLUMN IF NOT EXISTS dietary_restrictions TEXT,
ADD COLUMN IF NOT EXISTS tshirt_size VARCHAR(20),
ADD COLUMN IF NOT EXISTS phone VARCHAR(20),
ADD COLUMN IF NOT EXISTS emergency_contact JSONB;

-- Indexes for profile search
CREATE INDEX IF NOT EXISTS idx_users_skills ON auth.users USING GIN (skills);
CREATE INDEX IF NOT EXISTS idx_users_interests ON auth.users USING GIN (interests);
CREATE INDEX IF NOT EXISTS idx_users_experience_level ON auth.users (experience_level);
CREATE INDEX IF NOT EXISTS idx_users_organization ON auth.users (organization);

-- Comments
COMMENT ON COLUMN auth.users.skills IS 'Array of skill tags (e.g., ["Python", "React", "ML"])';
COMMENT ON COLUMN auth.users.interests IS 'Array of interest tags (e.g., ["fintech", "health", "sustainability"])';
COMMENT ON COLUMN auth.users.experience_level IS 'Experience level: beginner, intermediate, advanced, expert';
COMMENT ON COLUMN auth.users.organization IS 'University, company, or organization affiliation';
COMMENT ON COLUMN auth.users.timezone IS 'User timezone for scheduling (e.g., "America/New_York")';
COMMENT ON COLUMN auth.users.dietary_restrictions IS 'Dietary restrictions for catering';
COMMENT ON COLUMN auth.users.tshirt_size IS 'T-shirt size: XS, S, M, L, XL, XXL';
COMMENT ON COLUMN auth.users.emergency_contact IS 'JSON object: {name, phone, relationship}';
