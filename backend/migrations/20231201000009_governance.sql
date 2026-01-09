-- Tool Governance Configuration Table
-- Controls which tools are exposed from each MCP server

CREATE TABLE IF NOT EXISTS governance_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL UNIQUE REFERENCES servers(id) ON DELETE CASCADE,
    allowed_tools JSONB DEFAULT '[]',   -- Whitelist: ONLY these tools exposed (empty = all)
    denied_tools JSONB DEFAULT '[]',    -- Blacklist: these tools hidden
    tool_prefix VARCHAR(50) DEFAULT '', -- Prefix added to tool names (e.g., "gh" → "gh_create_issue")
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraint: cannot have both whitelist and blacklist
    CONSTRAINT check_mutually_exclusive 
        CHECK (
            (jsonb_array_length(allowed_tools) = 0) OR 
            (jsonb_array_length(denied_tools) = 0)
        )
);

-- Index for fast lookup by server
CREATE INDEX IF NOT EXISTS idx_governance_configs_server_id ON governance_configs(server_id);
