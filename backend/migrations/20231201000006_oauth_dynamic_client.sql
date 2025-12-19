-- Add dynamic_client_id column to oauth_tokens for storing the generated client_id
ALTER TABLE oauth_tokens ADD COLUMN IF NOT EXISTS dynamic_client_id TEXT;
