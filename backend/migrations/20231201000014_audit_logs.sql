-- Audit Logs Hypertable
-- Stores detailed request/response data for debugging and compliance
-- Separate from request_metrics which stores only aggregate data

-- Create audit_logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID DEFAULT gen_random_uuid(),
    time TIMESTAMPTZ NOT NULL,
    user_id UUID NOT NULL,
    
    -- Target info
    target_type VARCHAR(10) NOT NULL,  -- 'server' or 'gateway'
    target_id UUID NOT NULL,
    target_name VARCHAR(255) NOT NULL,
    
    -- Request details
    method VARCHAR(50),                 -- 'initialize', 'tools/list', 'tools/call', etc.
    tool_name VARCHAR(255),             -- Actual tool name for tools/call
    request_body JSONB,                 -- Full request params (sanitized)
    
    -- Response details
    response_body JSONB,                -- Response result or error (truncated if large)
    status_code INT,                    -- HTTP status code
    error_message TEXT,                 -- Error message if failed
    
    -- Performance
    latency_ms INT,
    success BOOLEAN NOT NULL DEFAULT true,
    
    -- Metadata
    client_ip VARCHAR(45),              -- IPv4 or IPv6
    user_agent TEXT,
    session_id VARCHAR(255),            -- MCP session ID
    
    PRIMARY KEY (id, time)
);

-- Convert to hypertable (partitioned by time, 1 day chunks)
SELECT create_hypertable('audit_logs', 'time', 
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_audit_user_time ON audit_logs (user_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_target ON audit_logs (target_type, target_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_method ON audit_logs (method, time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_tool ON audit_logs (tool_name, time DESC) WHERE tool_name IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_success ON audit_logs (success, time DESC) WHERE NOT success;

-- Retention policy: 30 days by default (can be changed)
-- To change: SELECT remove_retention_policy('audit_logs');
--            SELECT add_retention_policy('audit_logs', INTERVAL '60 days');
SELECT add_retention_policy('audit_logs', INTERVAL '30 days', if_not_exists => TRUE);

-- Compression policy: compress chunks older than 7 days (saves ~90% disk space)
ALTER TABLE audit_logs SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'user_id, target_type, target_id',
    timescaledb.compress_orderby = 'time DESC'
);

SELECT add_compression_policy('audit_logs', INTERVAL '7 days', if_not_exists => TRUE);

-- Comment for documentation
COMMENT ON TABLE audit_logs IS 'Detailed MCP request/response logs for debugging and compliance. Auto-compressed after 7 days, auto-deleted after 30 days.';
