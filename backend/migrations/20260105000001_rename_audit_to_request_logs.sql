-- This migration was replaced by 20260105000003
-- TimescaleDB doesn't support renaming hypertables directly
-- Instead, we keep audit_logs for MCP request logs and create admin_audit_logs for admin actions

-- No changes needed - audit_logs continues to store MCP request logs
-- The backend code has been updated to use audit_logs for request logs
-- and admin_audit_logs for admin action audit trail
