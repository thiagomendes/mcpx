-- Add status column to servers table
-- Status values: 'active', 'pending_auth', 'disabled', 'error'
ALTER TABLE servers ADD COLUMN IF NOT EXISTS status VARCHAR(20) DEFAULT 'active';

-- Update existing oauth_auto servers without tokens to pending_auth
UPDATE servers 
SET status = 'pending_auth' 
WHERE auth_type = 'oauth_auto' 
  AND id NOT IN (SELECT server_id FROM oauth_tokens);

-- Update existing oauth_auto servers with tokens to active  
UPDATE servers 
SET status = 'active' 
WHERE auth_type = 'oauth_auto' 
  AND id IN (SELECT server_id FROM oauth_tokens);
