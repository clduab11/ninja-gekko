CREATE TABLE IF NOT EXISTS schema_migrations (
    version BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    checksum VARCHAR(64) NOT NULL,
    applied_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    execution_time_ms BIGINT,
    rolled_back_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS schema_migration_locks (
    lock_id VARCHAR(255) PRIMARY KEY,
    locked_by VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL
);
