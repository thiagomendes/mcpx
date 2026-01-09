-- Service Accounts for M2M OAuth Client Credentials
-- These are "virtual users" for automated systems, CI/CD pipelines, etc.

CREATE TABLE IF NOT EXISTS service_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    client_id VARCHAR(64) NOT NULL UNIQUE,        -- mcpx_sa_<random>
    client_secret_hash VARCHAR(64) NOT NULL,      -- SHA-256 hash
    scopes JSONB DEFAULT '["mcp:tool:execute"]',  -- Default scopes
    enabled BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_service_accounts_org ON service_accounts(org_id);
CREATE INDEX IF NOT EXISTS idx_service_accounts_client_id ON service_accounts(client_id);
