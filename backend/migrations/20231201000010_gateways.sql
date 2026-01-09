-- Virtual Gateways Tables
-- Aggregate multiple MCP servers into a single endpoint

CREATE TABLE gateways (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name),
    UNIQUE(user_id, slug)
);

CREATE INDEX idx_gateways_user_id ON gateways(user_id);
CREATE INDEX idx_gateways_slug ON gateways(user_id, slug);

CREATE TABLE gateway_servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gateway_id UUID NOT NULL REFERENCES gateways(id) ON DELETE CASCADE,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(gateway_id, server_id)
);

CREATE INDEX idx_gateway_servers_gateway_id ON gateway_servers(gateway_id);

CREATE TABLE gateway_sessions (
    id VARCHAR(64) PRIMARY KEY,
    gateway_id UUID NOT NULL REFERENCES gateways(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    server_sessions JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_gateway_sessions_expires ON gateway_sessions(expires_at);
CREATE INDEX idx_gateway_sessions_gateway ON gateway_sessions(gateway_id);
