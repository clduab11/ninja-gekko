-- V003: Create trade execution tables for autonomous trading
-- This migration creates tables for tracking individual trade executions and a comprehensive audit log.

CREATE TABLE IF NOT EXISTS trade_executions (
    id UUID PRIMARY KEY,
    bot_id VARCHAR(50) NOT NULL,
    exchange VARCHAR(50) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(10) NOT NULL CHECK (side IN ('Buy', 'Sell')),
    order_type VARCHAR(20) NOT NULL CHECK (order_type IN ('Market', 'Limit', 'Stop', 'StopLimit')),
    quantity DECIMAL(20, 8) NOT NULL,
    price DECIMAL(20, 8),
    status VARCHAR(20) NOT NULL CHECK (status IN ('Pending', 'Open', 'Filled', 'PartiallyFilled', 'Cancelled', 'Rejected', 'Failed')),
    external_order_id VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID REFERENCES trade_executions(id),
    event_type VARCHAR(50) NOT NULL, -- ORDER_PLACED, FILL, ERROR, SAFETY_HALT, EMERGENCY_STOP
    message TEXT,
    metadata JSONB,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('Info', 'Warn', 'Error', 'Critical')),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_trade_executions_bot_id ON trade_executions(bot_id);
CREATE INDEX IF NOT EXISTS idx_trade_executions_exchange ON trade_executions(exchange);
CREATE INDEX IF NOT EXISTS idx_trade_executions_status ON trade_executions(status);
CREATE INDEX IF NOT EXISTS idx_trade_executions_created_at ON trade_executions(created_at);

CREATE INDEX IF NOT EXISTS idx_audit_logs_execution_id ON audit_logs(execution_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);

-- Trigger for updating updated_at
CREATE TRIGGER update_trade_executions_updated_at BEFORE UPDATE ON trade_executions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
