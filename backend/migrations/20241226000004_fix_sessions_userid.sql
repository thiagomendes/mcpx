-- Fix gateway_sessions user_id constraint
-- Similar to metrics/audit, user_id might not be available for org-level proxying or M2M interactions.
-- We rely on org_id for ownership.

ALTER TABLE gateway_sessions ALTER COLUMN user_id DROP NOT NULL;
