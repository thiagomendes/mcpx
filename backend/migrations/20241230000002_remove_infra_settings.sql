-- Migration: Remove infrastructure-level settings from org_settings
-- These settings are now controlled via ENV vars, not per-org configuration
--
-- Retention settings -> ENV vars (RETENTION_*_DAYS)
-- Interval settings -> ENV vars (HEALTH_CHECK_INTERVAL_SECONDS, ALERT_EVAL_INTERVAL_SECONDS, SESSION_CACHE_HOURS)

-- Remove retention settings (now in .env as RETENTION_* vars)
DELETE FROM org_settings WHERE setting_key IN (
    'retention_metrics_days',
    'retention_request_logs_days', 
    'retention_audit_logs_days',
    'retention_audit_events_days'
);

-- Remove interval/session settings (now in .env as *_INTERVAL_SECONDS, SESSION_CACHE_HOURS vars)
DELETE FROM org_settings WHERE setting_key IN (
    'health_check_interval_seconds',
    'alert_eval_interval_seconds',
    'session_cache_hours'
);

-- Remaining per-org settings:
-- - jwt_expiry_days (token expiry)
-- - m2m_token_expiry_hours (M2M token expiry)
-- - max_servers_per_org (limit)
-- - max_gateways_per_org (limit)
-- - max_pats_per_user (limit)
-- - max_service_accounts_per_org (limit)
