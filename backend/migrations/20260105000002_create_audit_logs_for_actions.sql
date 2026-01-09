-- Create new admin_audit_logs table for admin/user actions
-- This tracks WHO did WHAT, WHEN for security and compliance
-- Note: We keep audit_logs for MCP request logs (legacy name due to TimescaleDB hypertable limitation)

CREATE TABLE IF NOT EXISTS admin_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Action info
    action VARCHAR(50) NOT NULL,           -- 'server.create', 'gateway.update', 'user.login', etc.
    resource_type VARCHAR(50) NOT NULL,    -- 'server', 'gateway', 'user', 'org'
    resource_id UUID,                       -- ID of affected resource (if applicable)
    resource_name VARCHAR(255),             -- Human-readable name
    
    -- Additional context
    details JSONB,                          -- Additional action-specific data
    ip_address VARCHAR(45),                 -- Client IP (IPv4 or IPv6)
    user_agent TEXT,                        -- Browser/client info
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_admin_audit_logs_org_time ON admin_audit_logs (org_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_admin_audit_logs_user_time ON admin_audit_logs (user_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_admin_audit_logs_action ON admin_audit_logs (action, time DESC);
CREATE INDEX IF NOT EXISTS idx_admin_audit_logs_resource ON admin_audit_logs (resource_type, resource_id, time DESC);

-- Comment for documentation
COMMENT ON TABLE admin_audit_logs IS 'Security audit trail for admin and user actions. Tracks who did what and when.';

-- Note: This table is NOT a hypertable since it stores discrete events, not high-frequency time-series data.
-- Retention can be managed via application logic or a scheduled job based on org settings.
