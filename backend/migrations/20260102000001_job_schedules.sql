-- Job Queue Schema
-- Enables distributed job execution across multiple worker replicas

CREATE TABLE IF NOT EXISTS job_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(50) NOT NULL UNIQUE,
    interval_seconds INTEGER NOT NULL DEFAULT 60,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    locked_by VARCHAR(100),
    locked_at TIMESTAMPTZ,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for efficient job claiming
CREATE INDEX IF NOT EXISTS idx_job_schedules_claimable 
ON job_schedules(next_run_at) 
WHERE enabled = true AND locked_by IS NULL;

-- Seed default jobs
INSERT INTO job_schedules (job_type, interval_seconds, next_run_at, enabled) VALUES
    ('health_check', 300, NOW(), true),
    ('alert_eval', 60, NOW(), true)
ON CONFLICT (job_type) DO NOTHING;

COMMENT ON TABLE job_schedules IS 'Scheduled background jobs with distributed locking via SKIP LOCKED';
COMMENT ON COLUMN job_schedules.locked_by IS 'Worker instance ID that currently holds the lock';
COMMENT ON COLUMN job_schedules.locked_at IS 'When the lock was acquired - jobs locked >5min are considered stuck';
