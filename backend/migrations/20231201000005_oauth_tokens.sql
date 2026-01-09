-- OAuth tokens for server authentication
-- Stores access/refresh tokens obtained from upstream OAuth providers

CREATE TABLE oauth_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Encrypted tokens (AES-256-GCM)
    access_token_encrypted TEXT NOT NULL,
    refresh_token_encrypted TEXT,
    
    -- Token metadata
    token_type VARCHAR(50) NOT NULL DEFAULT 'Bearer',
    expires_at TIMESTAMPTZ,
    scope TEXT,
    
    -- PKCE state (stored temporarily during auth flow)
    pkce_code_verifier TEXT,
    oauth_state TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- One token per server per user
    UNIQUE(server_id, user_id)
);

-- Trigger for updated_at
CREATE TRIGGER update_oauth_tokens_updated_at
    BEFORE UPDATE ON oauth_tokens
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Indexes
CREATE INDEX idx_oauth_tokens_server_id ON oauth_tokens(server_id);
CREATE INDEX idx_oauth_tokens_user_id ON oauth_tokens(user_id);
CREATE INDEX idx_oauth_tokens_expires_at ON oauth_tokens(expires_at);
