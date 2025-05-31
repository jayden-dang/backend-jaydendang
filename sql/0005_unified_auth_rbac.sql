-- ===================================================================================================
-- UNIFIED AUTHENTICATION & ROLE-BASED ACCESS CONTROL SYSTEM
-- Supports: Wallet, OAuth2 (Google, GitHub), Email/Password authentication
-- ===================================================================================================

-- Create unified auth schema
DROP SCHEMA IF EXISTS "unified_auth" CASCADE;
CREATE SCHEMA "unified_auth";

-- ===================================================================================================
-- 1. USER ROLES ENUM & PERMISSIONS
-- ===================================================================================================
CREATE TYPE user_role AS ENUM ('normal', 'member', 'vip', 'moderator', 'admin');
CREATE TYPE auth_provider AS ENUM ('email', 'google', 'github', 'wallet');
CREATE TYPE provider_status AS ENUM ('active', 'suspended', 'revoked');

-- ===================================================================================================
-- 2. CORE USERS TABLE - Unified User Identity
-- ===================================================================================================
CREATE TABLE unified_auth.users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Core identity
    email VARCHAR(255) UNIQUE,  -- Primary email (can be null for wallet-only users)
    username VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(150),
    
    -- User status and roles
    role user_role DEFAULT 'normal',
    is_active BOOLEAN DEFAULT true,
    is_email_verified BOOLEAN DEFAULT false,
    is_profile_complete BOOLEAN DEFAULT false,
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMPTZ,
    login_count INTEGER DEFAULT 0,
    
    -- Soft delete
    deleted_at TIMESTAMPTZ
);

-- ===================================================================================================
-- 3. AUTHENTICATION PROVIDERS - Multiple auth methods per user
-- ===================================================================================================
CREATE TABLE unified_auth.user_auth_providers (
    provider_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES unified_auth.users(user_id) ON DELETE CASCADE,
    
    -- Provider details
    provider_type auth_provider NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL, -- External ID from provider
    provider_email VARCHAR(255), -- Email from provider (may differ from primary)
    
    -- Authentication data
    password_hash VARCHAR(255), -- For email auth
    wallet_address VARCHAR(66), -- For wallet auth (Sui format)
    public_key TEXT, -- For wallet auth
    oauth_access_token TEXT, -- For OAuth providers (encrypted)
    oauth_refresh_token TEXT, -- For OAuth providers (encrypted)
    oauth_token_expires_at TIMESTAMPTZ, -- For OAuth providers
    
    -- Provider metadata
    provider_metadata JSONB, -- Additional provider-specific data
    status provider_status DEFAULT 'active',
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    UNIQUE(provider_type, provider_user_id),
    UNIQUE(wallet_address) WHERE wallet_address IS NOT NULL
);

-- ===================================================================================================
-- 4. USER PROFILES - Extended user information
-- ===================================================================================================
CREATE TABLE unified_auth.user_profiles (
    profile_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE NOT NULL REFERENCES unified_auth.users(user_id) ON DELETE CASCADE,
    
    -- Personal information
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    bio TEXT,
    avatar_url VARCHAR(500),
    
    -- Demographics
    birth_year INTEGER,
    gender user_gender,
    occupation VARCHAR(100),
    education_level education_level,
    experience_level experience_level,
    
    -- Location & preferences
    timezone VARCHAR(50),
    country_code CHAR(2),
    language_preference VARCHAR(10) DEFAULT 'en',
    
    -- Privacy settings
    profile_visibility profile_visibility DEFAULT 'public',
    show_activity BOOLEAN DEFAULT true,
    show_email BOOLEAN DEFAULT false,
    
    -- Subscription & account
    subscription_tier subscription_tier DEFAULT 'free',
    registration_source registration_source DEFAULT 'organic',
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ===================================================================================================
-- 5. ROLE PERMISSIONS SYSTEM
-- ===================================================================================================
CREATE TABLE unified_auth.permissions (
    permission_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    permission_name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    resource VARCHAR(100) NOT NULL, -- e.g., 'users', 'posts', 'comments'
    action VARCHAR(50) NOT NULL,    -- e.g., 'read', 'write', 'delete', 'moderate'
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE unified_auth.role_permissions (
    role user_role NOT NULL,
    permission_id UUID NOT NULL REFERENCES unified_auth.permissions(permission_id),
    PRIMARY KEY (role, permission_id)
);

-- ===================================================================================================
-- 6. SESSION MANAGEMENT
-- ===================================================================================================
CREATE TABLE unified_auth.user_sessions (
    session_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES unified_auth.users(user_id) ON DELETE CASCADE,
    provider_id UUID REFERENCES unified_auth.user_auth_providers(provider_id),
    
    -- Session data
    jwt_token_id VARCHAR(255) UNIQUE NOT NULL, -- JTI claim
    device_info JSONB,
    ip_address INET,
    user_agent TEXT,
    
    -- Session lifecycle
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_revoked BOOLEAN DEFAULT false,
    revoked_at TIMESTAMPTZ,
    revoked_reason VARCHAR(100)
);

-- ===================================================================================================
-- 7. NONCES TABLE - For wallet signature verification
-- ===================================================================================================
CREATE TABLE unified_auth.auth_nonces (
    nonce_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    nonce_value VARCHAR(64) UNIQUE NOT NULL,
    wallet_address VARCHAR(66) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    is_used BOOLEAN DEFAULT false
);

-- ===================================================================================================
-- 8. INDEXES FOR PERFORMANCE
-- ===================================================================================================

-- Users table indexes
CREATE UNIQUE INDEX idx_users_email ON unified_auth.users(email) WHERE email IS NOT NULL;
CREATE UNIQUE INDEX idx_users_username ON unified_auth.users(username);
CREATE INDEX idx_users_role ON unified_auth.users(role);
CREATE INDEX idx_users_active ON unified_auth.users(is_active) WHERE is_active = true;
CREATE INDEX idx_users_created_at ON unified_auth.users(created_at);
CREATE INDEX idx_users_last_login ON unified_auth.users(last_login);

-- Auth providers indexes
CREATE INDEX idx_auth_providers_user_id ON unified_auth.user_auth_providers(user_id);
CREATE INDEX idx_auth_providers_type ON unified_auth.user_auth_providers(provider_type);
CREATE INDEX idx_auth_providers_status ON unified_auth.user_auth_providers(status);
CREATE UNIQUE INDEX idx_auth_providers_wallet ON unified_auth.user_auth_providers(wallet_address) 
    WHERE wallet_address IS NOT NULL;
CREATE INDEX idx_auth_providers_email ON unified_auth.user_auth_providers(provider_email);

-- Sessions indexes
CREATE INDEX idx_sessions_user_id ON unified_auth.user_sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON unified_auth.user_sessions(expires_at);
CREATE INDEX idx_sessions_active ON unified_auth.user_sessions(is_revoked, expires_at) 
    WHERE is_revoked = false;

-- Nonces indexes
CREATE INDEX idx_nonces_wallet_address ON unified_auth.auth_nonces(wallet_address);
CREATE INDEX idx_nonces_expires_at ON unified_auth.auth_nonces(expires_at);
CREATE INDEX idx_nonces_unused ON unified_auth.auth_nonces(is_used, expires_at) 
    WHERE is_used = false;

-- ===================================================================================================
-- 9. DEFAULT PERMISSIONS SETUP
-- ===================================================================================================

-- Insert default permissions
INSERT INTO unified_auth.permissions (permission_name, description, resource, action) VALUES
    -- User management
    ('users.read.own', 'Read own user profile', 'users', 'read'),
    ('users.read.all', 'Read all user profiles', 'users', 'read'),
    ('users.write.own', 'Update own user profile', 'users', 'write'),
    ('users.write.all', 'Update any user profile', 'users', 'write'),
    ('users.delete.own', 'Delete own user account', 'users', 'delete'),
    ('users.delete.all', 'Delete any user account', 'users', 'delete'),
    ('users.moderate', 'Moderate user accounts', 'users', 'moderate'),
    
    -- Content management
    ('content.read', 'Read content', 'content', 'read'),
    ('content.write.own', 'Create and edit own content', 'content', 'write'),
    ('content.write.all', 'Create and edit any content', 'content', 'write'),
    ('content.delete.own', 'Delete own content', 'content', 'delete'),
    ('content.delete.all', 'Delete any content', 'content', 'delete'),
    ('content.moderate', 'Moderate content', 'content', 'moderate'),
    ('content.publish', 'Publish content', 'content', 'publish'),
    
    -- Comments
    ('comments.read', 'Read comments', 'comments', 'read'),
    ('comments.write', 'Write comments', 'comments', 'write'),
    ('comments.delete.own', 'Delete own comments', 'comments', 'delete'),
    ('comments.delete.all', 'Delete any comments', 'comments', 'delete'),
    ('comments.moderate', 'Moderate comments', 'comments', 'moderate'),
    
    -- System admin
    ('system.admin', 'Full system administration', 'system', 'admin'),
    ('system.analytics', 'Access system analytics', 'system', 'analytics'),
    ('system.logs', 'Access system logs', 'system', 'logs');

-- Assign permissions to roles
INSERT INTO unified_auth.role_permissions (role, permission_id)
SELECT 'normal', permission_id FROM unified_auth.permissions 
WHERE permission_name IN ('users.read.own', 'users.write.own', 'content.read', 'comments.read', 'comments.write', 'comments.delete.own');

INSERT INTO unified_auth.role_permissions (role, permission_id)
SELECT 'member', permission_id FROM unified_auth.permissions 
WHERE permission_name IN ('users.read.own', 'users.write.own', 'content.read', 'content.write.own', 'content.delete.own', 'comments.read', 'comments.write', 'comments.delete.own');

INSERT INTO unified_auth.role_permissions (role, permission_id)
SELECT 'vip', permission_id FROM unified_auth.permissions 
WHERE permission_name IN ('users.read.own', 'users.write.own', 'content.read', 'content.write.own', 'content.delete.own', 'content.publish', 'comments.read', 'comments.write', 'comments.delete.own');

INSERT INTO unified_auth.role_permissions (role, permission_id)
SELECT 'moderator', permission_id FROM unified_auth.permissions 
WHERE permission_name IN ('users.read.all', 'users.write.own', 'users.moderate', 'content.read', 'content.write.own', 'content.delete.own', 'content.moderate', 'content.publish', 'comments.read', 'comments.write', 'comments.delete.all', 'comments.moderate');

INSERT INTO unified_auth.role_permissions (role, permission_id)
SELECT 'admin', permission_id FROM unified_auth.permissions;

-- ===================================================================================================
-- 10. UTILITY FUNCTIONS
-- ===================================================================================================

-- Function to cleanup expired nonces
CREATE OR REPLACE FUNCTION unified_auth.cleanup_expired_nonces()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM unified_auth.auth_nonces 
    WHERE expires_at < CURRENT_TIMESTAMP OR (is_used = true AND created_at < CURRENT_TIMESTAMP - INTERVAL '1 hour');
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup expired sessions
CREATE OR REPLACE FUNCTION unified_auth.cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM unified_auth.user_sessions 
    WHERE expires_at < CURRENT_TIMESTAMP OR is_revoked = true;
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to get user permissions
CREATE OR REPLACE FUNCTION unified_auth.get_user_permissions(p_user_id UUID)
RETURNS TABLE(permission_name VARCHAR, resource VARCHAR, action VARCHAR) AS $$
BEGIN
    RETURN QUERY
    SELECT p.permission_name, p.resource, p.action
    FROM unified_auth.users u
    JOIN unified_auth.role_permissions rp ON u.role = rp.role
    JOIN unified_auth.permissions p ON rp.permission_id = p.permission_id
    WHERE u.user_id = p_user_id AND u.is_active = true;
END;
$$ LANGUAGE plpgsql;

-- Function to check user permission
CREATE OR REPLACE FUNCTION unified_auth.has_permission(p_user_id UUID, p_permission_name VARCHAR)
RETURNS BOOLEAN AS $$
DECLARE
    has_perm BOOLEAN := false;
BEGIN
    SELECT EXISTS(
        SELECT 1 FROM unified_auth.get_user_permissions(p_user_id) 
        WHERE permission_name = p_permission_name
    ) INTO has_perm;
    
    RETURN has_perm;
END;
$$ LANGUAGE plpgsql;

-- Update timestamp trigger
CREATE OR REPLACE FUNCTION unified_auth.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply update triggers
CREATE TRIGGER trigger_users_updated_at BEFORE UPDATE ON unified_auth.users
    FOR EACH ROW EXECUTE FUNCTION unified_auth.update_updated_at_column();

CREATE TRIGGER trigger_user_auth_providers_updated_at BEFORE UPDATE ON unified_auth.user_auth_providers
    FOR EACH ROW EXECUTE FUNCTION unified_auth.update_updated_at_column();

CREATE TRIGGER trigger_user_profiles_updated_at BEFORE UPDATE ON unified_auth.user_profiles
    FOR EACH ROW EXECUTE FUNCTION unified_auth.update_updated_at_column();

-- ===================================================================================================
-- 11. COMMENTS FOR DOCUMENTATION
-- ===================================================================================================
COMMENT ON SCHEMA unified_auth IS 'Unified authentication system supporting multiple providers and RBAC';
COMMENT ON TABLE unified_auth.users IS 'Core user identity table';
COMMENT ON TABLE unified_auth.user_auth_providers IS 'Multiple authentication methods per user';
COMMENT ON TABLE unified_auth.user_profiles IS 'Extended user profile information';
COMMENT ON TABLE unified_auth.permissions IS 'System permissions';
COMMENT ON TABLE unified_auth.role_permissions IS 'Role-based permission assignments';
COMMENT ON TABLE unified_auth.user_sessions IS 'Active user sessions with JWT tracking';
COMMENT ON TABLE unified_auth.auth_nonces IS 'Nonces for wallet signature verification';