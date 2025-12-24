-- Expand auth_type column to support longer auth type names like 'oauth_client_credentials'
ALTER TABLE servers ALTER COLUMN auth_type TYPE VARCHAR(50);
