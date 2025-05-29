-- ===================================================================================================
-- SPONSORED TRANSACTIONS - Gas Station Tracking
-- ===================================================================================================

CREATE TABLE IF NOT EXISTS public.sponsored_transactions (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(66) NOT NULL, -- Sui addresses are 66 chars with 0x prefix
    gas_budget BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes for common queries
    CONSTRAINT valid_gas_budget CHECK (gas_budget > 0),
    CONSTRAINT valid_address_format CHECK (user_address ~ '^0x[a-fA-F0-9]{64}$')
);

-- Indexes for performance
CREATE INDEX idx_sponsored_transactions_user_address ON public.sponsored_transactions(user_address);
CREATE INDEX idx_sponsored_transactions_timestamp ON public.sponsored_transactions(timestamp);
CREATE INDEX idx_sponsored_transactions_user_timestamp ON public.sponsored_transactions(user_address, timestamp);

-- Comments for documentation
COMMENT ON TABLE public.sponsored_transactions IS 'Track sponsored transactions for gas station analytics and rate limiting';
COMMENT ON COLUMN public.sponsored_transactions.user_address IS 'Sui address of the user whose transaction was sponsored';
COMMENT ON COLUMN public.sponsored_transactions.gas_budget IS 'Gas budget for the sponsored transaction in MIST';
COMMENT ON COLUMN public.sponsored_transactions.timestamp IS 'When the transaction was sponsored'; 