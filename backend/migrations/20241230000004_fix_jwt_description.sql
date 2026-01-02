-- Migration: Fix jwt_expiry description to be provider-agnostic
-- MCPX supports multiple OAuth providers (GitHub, Microsoft, Google, etc.)

UPDATE org_settings 
SET description = 'How many days you stay logged into the MCPX Dashboard after signing in. After this period, you will need to sign in again.'
WHERE setting_key = 'jwt_expiry_days';
