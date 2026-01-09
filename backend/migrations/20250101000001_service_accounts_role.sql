-- Add role column to service_accounts for RBAC support
-- Allows M2M tokens to have admin or member roles

-- Add column if not exists
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'service_accounts' AND column_name = 'role') THEN
        ALTER TABLE service_accounts ADD COLUMN role VARCHAR(20) NOT NULL DEFAULT 'member';
    END IF;
END $$;

-- Add constraint if not exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints 
                   WHERE constraint_name = 'chk_sa_role' AND table_name = 'service_accounts') THEN
        ALTER TABLE service_accounts ADD CONSTRAINT chk_sa_role CHECK (role IN ('admin', 'member'));
    END IF;
END $$;

-- Create index if not exists
CREATE INDEX IF NOT EXISTS idx_service_accounts_role ON service_accounts(org_id, role);
