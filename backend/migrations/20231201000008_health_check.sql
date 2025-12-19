-- Migration: Server Health Check System
-- New status values: pending_auth, pending_health, healthy, unhealthy

-- Update status column to new values
-- Map old values to new ones:
-- 'active' -> 'healthy' (if has tokens) or 'pending_health' (if no auth needed)
-- 'pending_auth' -> 'pending_auth' (unchanged)
-- 'disabled' -> 'unhealthy'

-- First, update servers with tokens (oauth that worked) to healthy
UPDATE servers 
SET status = 'healthy' 
WHERE status = 'active' 
  AND auth_type = 'oauth_auto'
  AND id IN (SELECT server_id FROM oauth_tokens);

-- Update servers without auth requirement to pending_health (will become healthy after 1st check)
UPDATE servers 
SET status = 'pending_health' 
WHERE status = 'active' 
  AND (auth_type IS NULL OR auth_type = 'none');

-- Keep pending_auth as is

-- Add health check tracking columns to servers table
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_health_check TIMESTAMP WITH TIME ZONE;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS health_error TEXT;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS health_check_interval_seconds INTEGER DEFAULT 300;
