-- Add tool_name column to request_metrics for tracking specific tools called via tools/call
ALTER TABLE request_metrics ADD COLUMN IF NOT EXISTS tool_name VARCHAR(255);

-- Create index for tool_name queries
CREATE INDEX IF NOT EXISTS idx_metrics_tool_name ON request_metrics (user_id, tool_name) WHERE tool_name IS NOT NULL;
