-- Create ENUMs for stable, business-critical categories
CREATE TYPE experience_level AS ENUM ('beginner', 'intermediate', 'advanced', 'expert');
CREATE TYPE subscription_tier AS ENUM ('free', 'premium', 'enterprise', 'lifetime');
CREATE TYPE education_level AS ENUM ('high_school', 'bachelor', 'master', 'phd', 'bootcamp', 'self_taught', 'other');
CREATE TYPE device_type AS ENUM ('mobile', 'tablet', 'desktop');
CREATE TYPE profile_visibility AS ENUM ('public', 'private', 'friends');
CREATE TYPE user_gender AS ENUM ('male', 'female', 'non_binary', 'prefer_not_to_say', 'other');
CREATE TYPE registration_source AS ENUM ('organic', 'google', 'facebook', 'twitter', 'referral', 'paid_ad', 'blog', 'youtube', 'email', 'other');
CREATE TYPE account_status AS ENUM ('active', 'inactive', 'suspended', 'pending_verification', 'locked', 'marked_for_deletion');

-- Create profile schema
CREATE SCHEMA IF NOT EXISTS "profile";

-- ===================================================================================================
-- 1. CORE USERS TABLE - Authentication & Basic Info Only
-- Principle: Single Responsibility - Authentication concerns only
-- ===================================================================================================
CREATE TABLE profile.users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Authentication data (hot data - accessed every request)
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,

    -- Basic identity (frequently accessed)
    first_name VARCHAR(100),
    last_name VARCHAR(100),

    -- Account status (security-critical)
    is_active BOOLEAN DEFAULT false,
    email_verified BOOLEAN DEFAULT false,

    -- Timestamps (audit trail)
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Optimized indexes for authentication
CREATE UNIQUE INDEX idx_users_email ON profile.users(email);
CREATE UNIQUE INDEX idx_users_username ON profile.users(username);
CREATE INDEX idx_users_active ON profile.users(is_active) WHERE is_active = true;

-- ===================================================================================================
-- 2. USER PROFILES - Demographics & Personal Info
-- Principle: Separation of Concerns - Profile management separate from auth
-- ===================================================================================================
CREATE TABLE profile.user_profiles (
    profile_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE REFERENCES profile.users(user_id) ON DELETE CASCADE,

    -- Demographics (for personalization)
    birth_year INTEGER,
    gender user_gender,
    occupation VARCHAR(100),
    education_level education_level,
    experience_level experience_level,
    account_status account_status DEFAULT 'active',

    -- Location & preferences
    timezone VARCHAR(50),
    country_code CHAR(2),
    language_preference VARCHAR(10) DEFAULT 'en',

    -- Profile metadata
    avatar_url VARCHAR(500),
    bio TEXT,

    -- Privacy settings
    profile_visibility profile_visibility DEFAULT 'public',
    show_progress BOOLEAN DEFAULT true,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_profiles_user_id ON profile.user_profiles(user_id);
CREATE INDEX idx_profiles_experience ON profile.user_profiles(experience_level);
CREATE INDEX idx_profiles_education ON profile.user_profiles(education_level);
