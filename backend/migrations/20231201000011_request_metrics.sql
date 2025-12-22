-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Request metrics hypertable
CREATE TABLE request_metrics (
    time TIMESTAMPTZ NOT NULL,
    user_id UUID NOT NULL,
    target_type VARCHAR(10) NOT NULL,  -- 'server' or 'gateway'
    target_id UUID NOT NULL,
    target_name VARCHAR(255) NOT NULL,
    method VARCHAR(50),
    latency_ms INT,
    success BOOLEAN NOT NULL DEFAULT true
);

-- Convert to hypertable (partitioned by time)
SELECT create_hypertable('request_metrics', 'time');

-- Indexes for common queries
CREATE INDEX idx_metrics_user_time ON request_metrics (user_id, time DESC);
CREATE INDEX idx_metrics_target_time ON request_metrics (target_id, time DESC);
CREATE INDEX idx_metrics_user_target ON request_metrics (user_id, target_type, target_id, time DESC);

-- Retention policy: 90 days for raw data
SELECT add_retention_policy('request_metrics', INTERVAL '90 days');

-- Continuous aggregate: hourly stats (auto-updated)
CREATE MATERIALIZED VIEW request_metrics_hourly
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 hour', time) AS bucket,
    user_id,
    target_type,
    target_id,
    target_name,
    COUNT(*) AS total,
    COUNT(*) FILTER (WHERE success) AS success_count,
    AVG(latency_ms)::INT AS avg_latency_ms
FROM request_metrics
GROUP BY bucket, user_id, target_type, target_id, target_name
WITH NO DATA;

-- Refresh policy for hourly aggregates (every hour)
SELECT add_continuous_aggregate_policy('request_metrics_hourly',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour');

-- Daily aggregate for long-term analytics
CREATE MATERIALIZED VIEW request_metrics_daily
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 day', time) AS bucket,
    user_id,
    target_type,
    target_id,
    target_name,
    COUNT(*) AS total,
    COUNT(*) FILTER (WHERE success) AS success_count,
    AVG(latency_ms)::INT AS avg_latency_ms
FROM request_metrics
GROUP BY bucket, user_id, target_type, target_id, target_name
WITH NO DATA;

-- Refresh policy for daily aggregates (every day)
SELECT add_continuous_aggregate_policy('request_metrics_daily',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day');

-- ============================================
-- REQUEST LOGS (Schema only - future use)
-- ============================================
CREATE TABLE request_logs (
    time TIMESTAMPTZ NOT NULL,
    user_id UUID NOT NULL,
    server_id UUID,
    server_name VARCHAR(255),
    level VARCHAR(10) NOT NULL DEFAULT 'info',  -- 'debug', 'info', 'warn', 'error'
    method VARCHAR(50),
    message TEXT,
    metadata JSONB DEFAULT '{}'
);

-- Convert to hypertable
SELECT create_hypertable('request_logs', 'time');

-- Indexes
CREATE INDEX idx_logs_user_time ON request_logs (user_id, time DESC);
CREATE INDEX idx_logs_level ON request_logs (level, time DESC);

-- Retention: 30 days
SELECT add_retention_policy('request_logs', INTERVAL '30 days');

-- ============================================
-- AUDIT EVENTS (Schema only - future use)
-- ============================================
CREATE TABLE audit_events (
    time TIMESTAMPTZ NOT NULL,
    user_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,           -- 'create', 'update', 'delete', 'login', 'logout'
    resource_type VARCHAR(50),             -- 'server', 'gateway', 'credential', 'governance'
    resource_id UUID,
    resource_name VARCHAR(255),
    details JSONB DEFAULT '{}'
);

-- Convert to hypertable
SELECT create_hypertable('audit_events', 'time');

-- Indexes
CREATE INDEX idx_audit_user_time ON audit_events (user_id, time DESC);
CREATE INDEX idx_audit_action ON audit_events (action, time DESC);
CREATE INDEX idx_audit_resource ON audit_events (resource_type, resource_id, time DESC);

-- Retention: 365 days
SELECT add_retention_policy('audit_events', INTERVAL '365 days');
