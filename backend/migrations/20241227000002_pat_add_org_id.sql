-- Add org_id to personal_access_tokens
-- Tokens are now scoped per user+org, not just per user

-- Add column (nullable first for existing tokens)
ALTER TABLE personal_access_tokens ADD COLUMN org_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Delete any existing tokens (they need to be recreated with org context)
DELETE FROM personal_access_tokens WHERE org_id IS NULL;

-- Make column NOT NULL
ALTER TABLE personal_access_tokens ALTER COLUMN org_id SET NOT NULL;

-- Add index for listing tokens by user+org
CREATE INDEX idx_pat_user_org ON personal_access_tokens(user_id, org_id);
