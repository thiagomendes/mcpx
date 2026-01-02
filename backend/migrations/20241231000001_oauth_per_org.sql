-- Change oauth_tokens from per-user to per-org
-- This allows all org members to share the same OAuth token

-- Add org_id column
ALTER TABLE oauth_tokens ADD COLUMN org_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Populate org_id from servers table (servers have org_id)
UPDATE oauth_tokens ot
SET org_id = s.org_id
FROM servers s
WHERE ot.server_id = s.id;

-- Make org_id NOT NULL after population
ALTER TABLE oauth_tokens ALTER COLUMN org_id SET NOT NULL;

-- Drop old unique constraint and user_id foreign key
ALTER TABLE oauth_tokens DROP CONSTRAINT oauth_tokens_server_id_user_id_key;
ALTER TABLE oauth_tokens DROP CONSTRAINT oauth_tokens_user_id_fkey;

-- Drop user_id column (no longer needed - tokens are per-org now)
ALTER TABLE oauth_tokens DROP COLUMN user_id;

-- Add new unique constraint (one token per server per org)
ALTER TABLE oauth_tokens ADD CONSTRAINT oauth_tokens_server_id_org_id_key UNIQUE(server_id, org_id);

-- Drop old index and add new one
DROP INDEX IF EXISTS idx_oauth_tokens_user_id;
CREATE INDEX idx_oauth_tokens_org_id ON oauth_tokens(org_id);
