-- Credentials table for API keys and Bearer tokens
CREATE TABLE credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    credential_type VARCHAR(50) NOT NULL,  -- 'api_key', 'bearer', 'oauth_client_secret'
    encrypted_value TEXT NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger for updated_at
CREATE TRIGGER update_credentials_updated_at
    BEFORE UPDATE ON credentials
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Index for server lookup
CREATE INDEX idx_credentials_server_id ON credentials(server_id);
