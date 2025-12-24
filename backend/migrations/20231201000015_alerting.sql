-- Alerting System Tables
-- Stores alert rules and their trigger history

-- Alert Rules table
CREATE TABLE IF NOT EXISTS alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    alert_type VARCHAR(20) NOT NULL,      -- 'threshold', 'spike', 'no_data'
    metric VARCHAR(50) NOT NULL,           -- 'error_rate', 'avg_latency', 'request_count', 'p95_latency'
    scope_type VARCHAR(20) NOT NULL,       -- 'all', 'server', 'gateway'
    scope_id UUID,                          -- NULL if scope_type='all'
    operator VARCHAR(10) NOT NULL,         -- 'above', 'below'
    threshold FLOAT NOT NULL,
    duration_minutes INT NOT NULL DEFAULT 5,
    notify_dashboard BOOLEAN NOT NULL DEFAULT true,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_alert_type CHECK (alert_type IN ('threshold', 'spike', 'no_data')),
    CONSTRAINT chk_metric CHECK (metric IN ('error_rate', 'avg_latency', 'request_count', 'p95_latency')),
    CONSTRAINT chk_scope_type CHECK (scope_type IN ('all', 'server', 'gateway')),
    CONSTRAINT chk_operator CHECK (operator IN ('above', 'below'))
);

-- Alert History table - tracks triggered and resolved alerts
CREATE TABLE IF NOT EXISTS alert_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    trigger_value FLOAT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'triggered',
    
    CONSTRAINT chk_status CHECK (status IN ('triggered', 'resolved', 'acknowledged'))
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_alert_rules_user ON alert_rules(user_id);
CREATE INDEX IF NOT EXISTS idx_alert_rules_enabled ON alert_rules(enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_alert_history_rule ON alert_history(rule_id, triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_history_active ON alert_history(status) WHERE status IN ('triggered', 'acknowledged');

-- Trigger to update updated_at on alert_rules
CREATE OR REPLACE FUNCTION update_alert_rules_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_alert_rules_updated_at ON alert_rules;
CREATE TRIGGER trigger_alert_rules_updated_at
    BEFORE UPDATE ON alert_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_alert_rules_updated_at();

-- Comments for documentation
COMMENT ON TABLE alert_rules IS 'User-defined alert rules for monitoring metrics';
COMMENT ON TABLE alert_history IS 'History of triggered alerts with their resolution status';
COMMENT ON COLUMN alert_rules.alert_type IS 'threshold: metric crosses value, spike: sudden change, no_data: no requests';
COMMENT ON COLUMN alert_rules.scope_type IS 'all: monitor all targets, server: specific server, gateway: specific gateway';
