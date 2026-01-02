-- Migration: Update setting descriptions for better clarity
-- The original descriptions were too vague, especially for jwt_expiry and m2m_token

-- Security settings - be specific about what login/token means
UPDATE org_settings 
SET description = 'How many days you stay logged into the MCPX Dashboard (GitHub/OAuth login). After this, you will need to sign in again.'
WHERE setting_key = 'jwt_expiry_days';

UPDATE org_settings 
SET description = 'How long Service Account tokens (for API automation) remain valid. Used by external systems calling MCPX APIs.'
WHERE setting_key = 'm2m_token_expiry_hours';

-- Limit settings - clarify what each resource is
UPDATE org_settings 
SET description = 'Maximum number of MCP servers (like weather-mcp, filesystem-mcp) that can be registered in this organization.'
WHERE setting_key = 'max_servers_per_org';

UPDATE org_settings 
SET description = 'Maximum number of gateways (proxy endpoints that expose MCP servers to clients) per organization.'
WHERE setting_key = 'max_gateways_per_org';

UPDATE org_settings 
SET description = 'Maximum Personal Access Tokens each user can create. PATs are used for CLI and API access.'
WHERE setting_key = 'max_pats_per_user';

UPDATE org_settings 
SET description = 'Maximum Service Accounts per organization. Used for machine-to-machine (M2M) API automation.'
WHERE setting_key = 'max_service_accounts_per_org';
