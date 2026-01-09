-- Add auth_type to servers table
ALTER TABLE servers ADD COLUMN auth_type VARCHAR(20) NOT NULL DEFAULT 'none';
ALTER TABLE servers ADD COLUMN oauth_client_id TEXT;
ALTER TABLE servers ADD COLUMN oauth_authorization_url TEXT;
ALTER TABLE servers ADD COLUMN oauth_token_url TEXT;
ALTER TABLE servers ADD COLUMN oauth_scopes TEXT;
ALTER TABLE servers ADD COLUMN oauth_use_pkce BOOLEAN DEFAULT true;

-- Index for finding servers by auth type
CREATE INDEX idx_servers_auth_type ON servers(auth_type);

COMMENT ON COLUMN servers.auth_type IS 'Authentication type: none, api_key, bearer, oauth';
COMMENT ON COLUMN servers.oauth_use_pkce IS 'Use PKCE for OAuth 2.1 (default true)';
