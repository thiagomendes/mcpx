-- Migration: org_settings table (generic key-value configuration)
-- This table stores configurable settings per organization

CREATE TABLE IF NOT EXISTS org_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    setting_key VARCHAR(100) NOT NULL,
    setting_value TEXT NOT NULL,
    setting_type VARCHAR(20) NOT NULL DEFAULT 'string', -- string, integer, boolean, json
    description TEXT, -- Human-readable description
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT unique_org_setting UNIQUE (org_id, setting_key)
);

CREATE INDEX IF NOT EXISTS idx_org_settings_org_id ON org_settings(org_id);
CREATE INDEX IF NOT EXISTS idx_org_settings_key ON org_settings(setting_key);

-- Comment on table
COMMENT ON TABLE org_settings IS 'Per-organization configuration settings (key-value)';
COMMENT ON COLUMN org_settings.setting_key IS 'Unique setting identifier within org';
COMMENT ON COLUMN org_settings.setting_value IS 'Setting value (parsed based on setting_type)';
COMMENT ON COLUMN org_settings.setting_type IS 'Data type: string, integer, boolean, json';

-- Insert default settings for all existing organizations
INSERT INTO org_settings (org_id, setting_key, setting_value, setting_type, description)
SELECT 
    o.id,
    s.key,
    s.value,
    s.type,
    s.description
FROM organizations o
CROSS JOIN (VALUES
    -- Retention Settings (days)
    ('retention_metrics_days', '7', 'integer', 'How long request metrics are kept. Used for dashboard charts and analytics.'),
    ('retention_request_logs_days', '7', 'integer', 'How long detailed request logs are kept. Contains request/response for debugging.'),
    ('retention_audit_logs_days', '90', 'integer', 'How long user action audit trail is kept. For compliance and security.'),
    ('retention_audit_events_days', '90', 'integer', 'How long compliance events are kept. For security audits and investigations.'),
    
    -- System Intervals (seconds)
    ('health_check_interval_seconds', '300', 'integer', 'How often MCPX checks if upstream MCP servers are healthy.'),
    ('alert_eval_interval_seconds', '60', 'integer', 'How often alert rules are evaluated against metrics.'),
    
    -- Token/Session Settings
    ('jwt_expiry_days', '30', 'integer', 'How long user login sessions last before requiring re-authentication.'),
    ('m2m_token_expiry_hours', '1', 'integer', 'How long Service Account (M2M) tokens are valid.'),
    ('session_cache_hours', '1', 'integer', 'How long proxy sessions are cached in the database.'),
    
    -- Limits
    ('max_servers_per_org', '100', 'integer', 'Maximum number of MCP servers allowed per organization.'),
    ('max_gateways_per_org', '50', 'integer', 'Maximum number of gateways allowed per organization.'),
    ('max_pats_per_user', '10', 'integer', 'Maximum Personal Access Tokens per user.'),
    ('max_service_accounts_per_org', '20', 'integer', 'Maximum service accounts per organization.')
) AS s(key, value, type, description)
ON CONFLICT (org_id, setting_key) DO NOTHING;
