-- Update continuous aggregate refresh policy for near real-time dashboard
-- Change from 1 hour to 5 minutes for more responsive metrics

-- Remove existing policy for hourly aggregate
SELECT remove_continuous_aggregate_policy('request_metrics_hourly', if_exists => true);

-- Add new policy with 5 minute refresh interval
SELECT add_continuous_aggregate_policy('request_metrics_hourly',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '5 minutes');
