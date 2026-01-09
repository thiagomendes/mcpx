-- Personal Access Tokens (PATs) for gateway authentication
-- Users can create long-lived tokens for CLI/automation use

CREATE TABLE personal_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    token_hash VARCHAR(64) NOT NULL UNIQUE,  -- SHA-256 hash of the token
    token_prefix VARCHAR(12) NOT NULL,       -- First 8 chars for display/identification
    scopes JSONB DEFAULT '[]',               -- Optional: restrict token to specific scopes
    last_used_at TIMESTAMPTZ,                -- Updated on each use
    expires_at TIMESTAMPTZ,                  -- NULL = never expires
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast lookup by user
CREATE INDEX idx_pat_user ON personal_access_tokens(user_id);

-- Index for fast lookup by token hash (primary auth path)
CREATE INDEX idx_pat_hash ON personal_access_tokens(token_hash);
