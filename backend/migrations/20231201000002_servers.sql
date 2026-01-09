-- Servers table
CREATE TABLE IF NOT EXISTS servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    transport VARCHAR(50) NOT NULL DEFAULT 'streamable-http',
    enabled BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, name)
);

CREATE INDEX IF NOT EXISTS idx_servers_user_id ON servers(user_id);

CREATE TRIGGER update_servers_updated_at 
    BEFORE UPDATE ON servers
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();
