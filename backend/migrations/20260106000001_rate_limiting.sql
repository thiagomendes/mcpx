-- Add rate limiting column to servers table
-- NULL = no rate limit (default, unlimited)
-- Integer = max requests per minute per server

ALTER TABLE servers ADD COLUMN rate_limit_per_minute INTEGER DEFAULT NULL;

-- Add index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_servers_rate_limit ON servers(id) WHERE rate_limit_per_minute IS NOT NULL;
