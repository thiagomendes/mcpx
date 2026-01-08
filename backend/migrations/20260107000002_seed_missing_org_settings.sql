-- Migration: Seed default settings for organizations that don't have them
-- This fixes orgs created before settings initialization was added

INSERT INTO org_settings (org_id, setting_key, setting_value, setting_type, description)
SELECT 
    o.id,
    s.key,
    s.value,
    s.type,
    s.description
FROM organizations o
CROSS JOIN (VALUES
    ('jwt_expiry_days', '30', 'integer', 'How long user login sessions last before requiring re-authentication.'),
    ('m2m_token_expiry_hours', '1', 'integer', 'How long Service Account (M2M) tokens are valid.'),
    ('max_servers_per_org', '100', 'integer', 'Maximum number of MCP servers allowed per organization.'),
    ('max_gateways_per_org', '50', 'integer', 'Maximum number of gateways allowed per organization.'),
    ('max_pats_per_user', '10', 'integer', 'Maximum Personal Access Tokens per user.'),
    ('max_service_accounts_per_org', '20', 'integer', 'Maximum service accounts per organization.')
) AS s(key, value, type, description)
ON CONFLICT (org_id, setting_key) DO NOTHING;
