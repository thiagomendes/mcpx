-- Organizations and Multi-Provider Auth
-- This migration creates the org-based multi-tenant structure

-- ============================================
-- ORGANIZATIONS
-- ============================================

CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,  -- URL-safe identifier for proxy URLs
    is_personal BOOLEAN NOT NULL DEFAULT false,  -- Personal workspaces can't be deleted
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orgs_slug ON organizations(slug);

CREATE TRIGGER update_organizations_updated_at 
    BEFORE UPDATE ON organizations
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- ORG MEMBERS (many-to-many with roles)
-- ============================================

CREATE TABLE org_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'member',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, user_id),
    CONSTRAINT chk_role CHECK (role IN ('owner', 'admin', 'member'))
);

CREATE INDEX idx_org_members_org ON org_members(org_id);
CREATE INDEX idx_org_members_user ON org_members(user_id);

-- ============================================
-- USER IDENTITIES (multi-provider auth)
-- ============================================

CREATE TABLE user_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,  -- 'google', 'github', 'microsoft'
    provider_user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, provider_user_id)
);

CREATE INDEX idx_identities_user ON user_identities(user_id);
CREATE INDEX idx_identities_provider ON user_identities(provider, provider_user_id);

-- ============================================
-- MODIFY USERS TABLE
-- ============================================

-- Remove google_id (now in user_identities)
ALTER TABLE users DROP COLUMN IF EXISTS google_id;
DROP INDEX IF EXISTS idx_users_google_id;

-- ============================================
-- MODIFY SERVERS TABLE
-- ============================================

-- Add org_id column
ALTER TABLE servers ADD COLUMN org_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Drop old constraints and indexes
ALTER TABLE servers DROP CONSTRAINT IF EXISTS servers_user_id_name_key;
DROP INDEX IF EXISTS idx_servers_user_id;

-- Drop user_id column
ALTER TABLE servers DROP COLUMN user_id;

-- Add new constraint and index
ALTER TABLE servers ADD CONSTRAINT servers_org_id_name_key UNIQUE(org_id, name);
CREATE INDEX idx_servers_org_id ON servers(org_id);

-- Make org_id NOT NULL
ALTER TABLE servers ALTER COLUMN org_id SET NOT NULL;

-- ============================================
-- MODIFY GATEWAYS TABLE
-- ============================================

-- Add org_id column
ALTER TABLE gateways ADD COLUMN org_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Drop old constraints and indexes
ALTER TABLE gateways DROP CONSTRAINT IF EXISTS gateways_user_id_name_key;
ALTER TABLE gateways DROP CONSTRAINT IF EXISTS gateways_user_id_slug_key;
DROP INDEX IF EXISTS idx_gateways_user_id;
DROP INDEX IF EXISTS idx_gateways_slug;

-- Drop user_id column
ALTER TABLE gateways DROP COLUMN user_id;

-- Add new constraints and indexes
ALTER TABLE gateways ADD CONSTRAINT gateways_org_id_name_key UNIQUE(org_id, name);
ALTER TABLE gateways ADD CONSTRAINT gateways_org_id_slug_key UNIQUE(org_id, slug);
CREATE INDEX idx_gateways_org_id ON gateways(org_id);
CREATE INDEX idx_gateways_org_slug ON gateways(org_id, slug);

-- Make org_id NOT NULL
ALTER TABLE gateways ALTER COLUMN org_id SET NOT NULL;

-- ============================================
-- MODIFY GATEWAY_SESSIONS TABLE
-- ============================================

-- Add org_id (who owns the gateway) - keep user_id as actor
ALTER TABLE gateway_sessions ADD COLUMN org_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- We keep user_id here as "actor" - the user who initiated the session
-- Drop the user_id FK but keep the column for tracking
ALTER TABLE gateway_sessions DROP CONSTRAINT IF EXISTS gateway_sessions_user_id_fkey;

-- Make org_id NOT NULL after data migration would happen (not needed for fresh DB)
ALTER TABLE gateway_sessions ALTER COLUMN org_id SET NOT NULL;

-- ============================================
-- MODIFY REQUEST_METRICS TABLE
-- ============================================

-- Note: This is a TimescaleDB hypertable, we need to be careful
-- Add org_id column
ALTER TABLE request_metrics ADD COLUMN org_id UUID;

-- Keep user_id as actor who made the request
-- Update index to use org_id for common queries
DROP INDEX IF EXISTS idx_metrics_user_time;
DROP INDEX IF EXISTS idx_metrics_user_target;
CREATE INDEX idx_metrics_org_time ON request_metrics (org_id, time DESC);
CREATE INDEX idx_metrics_org_target ON request_metrics (org_id, target_type, target_id, time DESC);

-- ============================================
-- MODIFY AUDIT_LOGS TABLE
-- ============================================

-- Note: This is a TimescaleDB hypertable
-- Add org_id column, keep user_id as actor
ALTER TABLE audit_logs ADD COLUMN org_id UUID;

-- Update indexes
DROP INDEX IF EXISTS idx_audit_user_time;
CREATE INDEX idx_audit_org_time ON audit_logs (org_id, time DESC);
CREATE INDEX idx_audit_actor_time ON audit_logs (user_id, time DESC);

-- ============================================
-- MODIFY ALERT_RULES TABLE
-- ============================================

-- Add org_id column
ALTER TABLE alert_rules ADD COLUMN org_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Drop old index
DROP INDEX IF EXISTS idx_alert_rules_user;

-- Drop user_id column
ALTER TABLE alert_rules DROP COLUMN user_id;

-- Add new index
CREATE INDEX idx_alert_rules_org ON alert_rules(org_id);

-- Make org_id NOT NULL
ALTER TABLE alert_rules ALTER COLUMN org_id SET NOT NULL;

-- ============================================
-- COMMENTS
-- ============================================

COMMENT ON TABLE organizations IS 'Multi-tenant organizations. Each org owns servers, gateways, and other resources.';
COMMENT ON TABLE org_members IS 'Organization membership with roles (owner, admin, member).';
COMMENT ON TABLE user_identities IS 'OAuth provider identities linked to users. Enables multi-provider auth with email-based merging.';
COMMENT ON COLUMN organizations.slug IS 'URL-safe identifier used in proxy URLs: /mcp/{slug}/{server}';
COMMENT ON COLUMN organizations.is_personal IS 'Personal workspaces are auto-created for each user and cannot be deleted.';
