-- Fix metrics and audit logs user_id constraint
-- Since we moved to org-based multi-tenancy, user_id (actor) can be null because authentication is now handled via Org context
-- and some requests might be system-generated or purely Org-scoped without a specific user context available at proxy time yet.

ALTER TABLE request_metrics ALTER COLUMN user_id DROP NOT NULL;
ALTER TABLE audit_logs ALTER COLUMN user_id DROP NOT NULL;
