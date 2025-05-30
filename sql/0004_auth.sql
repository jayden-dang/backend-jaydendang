-- ===================================================================================================
-- AUTHENTICATION SCHEMA
-- ===================================================================================================

-- Create auth schema
CREATE SCHEMA IF NOT EXISTS "auth";

-- ===================================================================================================
-- 1. AUTH USERS TABLE - Wallet-based Authentication
-- ===================================================================================================
CREATE TABLE auth.users (
    address VARCHAR(66) PRIMARY KEY,  -- Sui address (0x + 64 hex chars)
    public_key TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    login_count INTEGER DEFAULT 1
);

-- Indexes for performance
CREATE INDEX idx_auth_users_last_login ON auth.users(last_login);
CREATE INDEX idx_auth_users_created_at ON auth.users(created_at);

-- ===================================================================================================
-- 2. NONCES TABLE - For signature verification (optional if using Redis)
-- ===================================================================================================
CREATE TABLE auth.nonces (
    address VARCHAR(66) PRIMARY KEY,
    nonce VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL
);

-- Index for cleanup
CREATE INDEX idx_auth_nonces_expires_at ON auth.nonces(expires_at);

-- ===================================================================================================
-- 3. CLEANUP FUNCTION FOR EXPIRED NONCES
-- ===================================================================================================
CREATE OR REPLACE FUNCTION auth.cleanup_expired_nonces()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM auth.nonces WHERE expires_at < CURRENT_TIMESTAMP;
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ===================================================================================================
-- 4. COMMENTS FOR DOCUMENTATION
-- ===================================================================================================
COMMENT ON SCHEMA auth IS 'Authentication system for wallet-based login';
COMMENT ON TABLE auth.users IS 'Authenticated users with wallet addresses';
COMMENT ON TABLE auth.nonces IS 'Temporary nonces for signature verification';
COMMENT ON COLUMN auth.users.address IS 'Sui wallet address (0x + 64 hex characters)';
COMMENT ON COLUMN auth.users.public_key IS 'Base64 encoded public key';
COMMENT ON COLUMN auth.nonces.nonce IS '64-character hex string for signature verification';
COMMENT ON FUNCTION auth.cleanup_expired_nonces() IS 'Removes expired nonces from the database'; 