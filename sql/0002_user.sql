-- Create ENUMs for stable, business-critical categories
CREATE TYPE experience_level_enum AS ENUM ('beginner', 'intermediate', 'advanced', 'expert');
CREATE TYPE subscription_tier_enum AS ENUM ('free', 'premium', 'enterprise', 'lifetime');
CREATE TYPE education_level_enum AS ENUM ('high_school', 'bachelor', 'master', 'phd', 'bootcamp', 'self_taught', 'other');
CREATE TYPE device_type_enum AS ENUM ('mobile', 'tablet', 'desktop');
CREATE TYPE user_gender_enum AS ENUM ('male', 'female', 'non_binary', 'prefer_not_to_say', 'other');
CREATE TYPE registration_source_enum AS ENUM ('organic', 'google', 'facebook', 'twitter', 'referral', 'paid_ad', 'blog', 'youtube', 'email', 'other');

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
    is_active BOOLEAN DEFAULT true,
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
    gender user_gender_enum,
    occupation VARCHAR(100),
    education_level education_level_enum,
    experience_level experience_level_enum,

    -- Location & preferences
    timezone VARCHAR(50),
    country_code CHAR(2),
    language_preference VARCHAR(10) DEFAULT 'en',

    -- Profile metadata
    avatar_url VARCHAR(500),
    bio TEXT,

    -- Privacy settings
    profile_visibility VARCHAR(20) DEFAULT 'public', -- public, private, friends
    show_progress BOOLEAN DEFAULT true,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_profiles_user_id ON profile.user_profiles(user_id);
CREATE INDEX idx_profiles_experience ON profile.user_profiles(experience_level);
CREATE INDEX idx_profiles_education ON profile.user_profiles(education_level);

-- ===================================================================================================
-- 3. USER SUBSCRIPTIONS - Billing & Subscription Management
-- Principle: Business Logic Separation - Billing is complex domain
-- ===================================================================================================
CREATE TABLE profile.user_subscriptions (
    subscription_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE REFERENCES profile.users(user_id) ON DELETE CASCADE,

    -- Current subscription
    tier subscription_tier_enum DEFAULT 'free',
    status VARCHAR(20) DEFAULT 'active', -- active, cancelled, expired, suspended

    -- Billing cycle
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    auto_renew BOOLEAN DEFAULT true,

    -- Pricing info (for analytics)
    price_paid DECIMAL(10,2),
    currency_code CHAR(3) DEFAULT 'USD',

    -- Payment method reference (don't store sensitive data)
    payment_method_id VARCHAR(100), -- Reference to payment processor

    -- Billing history tracking
    billing_address JSONB,

    -- Subscription metadata
    promo_code VARCHAR(50),
    referral_credit DECIMAL(10,2) DEFAULT 0,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_subscriptions_user_id ON profile.user_subscriptions(user_id);
CREATE INDEX idx_subscriptions_tier_status ON profile.user_subscriptions(tier, status);
CREATE INDEX idx_subscriptions_end_date ON profile.user_subscriptions(end_date) WHERE status = 'active';

-- ===================================================================================================
-- 4. USER ACQUISITION - Marketing & Registration Data
-- Principle: Marketing Analytics Separation - Different access patterns
-- ===================================================================================================
CREATE TABLE profile.user_acquisition (
    acquisition_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE REFERENCES profile.users(user_id) ON DELETE CASCADE,

    -- Registration source tracking
    registration_source registration_source_enum,
    referral_code VARCHAR(50),

    -- UTM parameters (marketing attribution)
    utm_source VARCHAR(100),
    utm_medium VARCHAR(100),
    utm_campaign VARCHAR(100),
    utm_content VARCHAR(100),
    utm_term VARCHAR(100),

    -- Technical registration data
    registration_ip INET,
    registration_user_agent TEXT,

    -- Referral system
    referred_by_user_id UUID REFERENCES profile.users(user_id),
    referral_reward_given BOOLEAN DEFAULT false,

    -- Landing page analytics
    landing_page VARCHAR(500),
    entry_point VARCHAR(200), -- signup_button, trial_popup, etc.

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_acquisition_user_id ON profile.user_acquisition(user_id);
CREATE INDEX idx_acquisition_source ON profile.user_acquisition(registration_source);
CREATE INDEX idx_acquisition_campaign ON profile.user_acquisition(utm_campaign, utm_source);
CREATE INDEX idx_acquisition_referrer ON profile.user_acquisition(referred_by_user_id);

-- ===================================================================================================
-- 5. USER DEVICE ANALYTICS - Technical & Behavioral Data
-- Principle: Analytics Separation - High volume, different query patterns
-- ===================================================================================================
CREATE TABLE profile.user_devices (
    device_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES profile.users(user_id) ON DELETE CASCADE,

    -- Device fingerprinting
    device_type device_type_enum,
    browser VARCHAR(50),
    browser_version VARCHAR(20),
    operating_system VARCHAR(50),
    os_version VARCHAR(20),
    screen_resolution VARCHAR(20),

    -- Device preferences
    is_primary_device BOOLEAN DEFAULT false,
    device_name VARCHAR(100), -- User-defined name

    -- Security tracking
    first_seen_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_trusted BOOLEAN DEFAULT false,

    -- Usage analytics
    session_count INTEGER DEFAULT 0,
    total_time_spent INTEGER DEFAULT 0, -- minutes

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_devices_user_id ON profile.user_devices(user_id);
CREATE INDEX idx_devices_type ON profile.user_devices(device_type);
CREATE INDEX idx_devices_primary ON profile.user_devices(user_id, is_primary_device) WHERE is_primary_device = true;
CREATE INDEX idx_devices_last_seen ON profile.user_devices(last_seen_at);

-- ===================================================================================================
-- 6. USER ENGAGEMENT METRICS - Aggregated Analytics
-- Principle: Performance Optimization - Pre-computed metrics for dashboards
-- ===================================================================================================
CREATE TABLE profile.user_engagement_metrics (
    metric_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE REFERENCES profile.users(user_id) ON DELETE CASCADE,

    -- Login patterns
    total_logins INTEGER DEFAULT 0,
    login_streak_current INTEGER DEFAULT 0,
    login_streak_longest INTEGER DEFAULT 0,
    last_login_at TIMESTAMPTZ,

    -- Time-based engagement
    total_time_spent INTEGER DEFAULT 0, -- minutes
    avg_session_duration INTEGER DEFAULT 0, -- minutes
    sessions_count INTEGER DEFAULT 0,

    -- Content engagement
    courses_enrolled INTEGER DEFAULT 0,
    courses_completed INTEGER DEFAULT 0,
    lessons_completed INTEGER DEFAULT 0,

    -- Social engagement
    posts_created INTEGER DEFAULT 0,
    comments_made INTEGER DEFAULT 0,
    likes_given INTEGER DEFAULT 0,

    -- Learning velocity
    avg_completion_rate DECIMAL(5,2) DEFAULT 0,
    learning_velocity_score DECIMAL(5,2) DEFAULT 0, -- Custom metric

    -- Computed daily (via scheduled job)
    last_computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_engagement_user_id ON profile.user_engagement_metrics(user_id);
CREATE INDEX idx_engagement_login_streak ON profile.user_engagement_metrics(login_streak_current);
CREATE INDEX idx_engagement_completion_rate ON profile.user_engagement_metrics(avg_completion_rate);

-- ===================================================================================================
-- 7. USER PREFERENCES - Settings & Personalization
-- Principle: Configuration Management - User-controlled settings
-- ===================================================================================================
CREATE TABLE profile.user_preferences (
    preference_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE REFERENCES profile.users(user_id) ON DELETE CASCADE,

    -- Learning preferences
    preferred_difficulty VARCHAR(20),
    preferred_content_types VARCHAR(100)[], -- ['video', 'text', 'interactive']
    preferred_lesson_duration INTEGER, -- minutes

    -- Notification preferences
    email_notifications BOOLEAN DEFAULT true,
    push_notifications BOOLEAN DEFAULT true,
    marketing_emails BOOLEAN DEFAULT false,
    course_reminders BOOLEAN DEFAULT true,
    achievement_notifications BOOLEAN DEFAULT true,

    -- UI/UX preferences
    theme VARCHAR(20) DEFAULT 'light', -- light, dark, auto
    autoplay_videos BOOLEAN DEFAULT true,
    subtitles_enabled BOOLEAN DEFAULT false,
    playback_speed DECIMAL(3,2) DEFAULT 1.0,

    -- Privacy preferences
    profile_public BOOLEAN DEFAULT true,
    show_progress BOOLEAN DEFAULT true,
    allow_friend_requests BOOLEAN DEFAULT true,

    -- Learning schedule
    study_reminder_time TIME,
    preferred_study_days INTEGER[], -- [1,2,3,4,5] for Mon-Fri
    weekly_study_goal INTEGER, -- hours

    -- Consent management (GDPR/CCPA)
    analytics_consent BOOLEAN DEFAULT true,
    marketing_consent BOOLEAN DEFAULT false,
    data_processing_consent BOOLEAN DEFAULT true,
    consent_date TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_preferences_user_id ON profile.user_preferences(user_id);
CREATE INDEX idx_preferences_notifications ON profile.user_preferences(email_notifications, push_notifications);

-- ===================================================================================================
-- COMPREHENSIVE VIEW - Joining normalized data efficiently
-- ===================================================================================================
CREATE VIEW profile.user_complete_profile AS
SELECT
    -- Core user data
    u.user_id,
    u.email,
    u.username,
    u.first_name,
    u.last_name,
    u.is_active,
    u.email_verified,
    u.created_at as registered_at,

    -- Profile information
    p.birth_year,
    p.gender,
    p.occupation,
    p.education_level,
    p.experience_level,
    p.country_code,
    p.language_preference,

    -- Subscription status
    s.tier as subscription_tier,
    s.status as subscription_status,
    s.end_date as subscription_end_date,

    -- Acquisition data
    a.registration_source,
    a.utm_campaign,
    a.utm_source,
    a.utm_medium,

    -- Engagement metrics
    e.total_logins,
    e.login_streak_current,
    e.last_login_at,
    e.total_time_spent,
    e.courses_enrolled,
    e.courses_completed,
    e.avg_completion_rate,

    -- Primary device
    d.device_type as primary_device_type,
    d.browser as primary_browser

FROM profile.users u
LEFT JOIN profile.user_profiles p ON u.user_id = p.user_id
LEFT JOIN profile.user_subscriptions s ON u.user_id = s.user_id
LEFT JOIN profile.user_acquisition a ON u.user_id = a.user_id
LEFT JOIN profile.user_engagement_metrics e ON u.user_id = e.user_id
LEFT JOIN profile.user_devices d ON u.user_id = d.user_id AND d.is_primary_device = true;

-- ===================================================================================================
-- MAINTENANCE FUNCTIONS - Keeping data fresh and clean
-- ===================================================================================================

-- Function to update engagement metrics (run daily)
CREATE OR REPLACE FUNCTION profile.update_user_engagement_metrics(target_user_id UUID DEFAULT NULL)
RETURNS void AS $$
BEGIN
    INSERT INTO profile.user_engagement_metrics (
        user_id, total_logins, sessions_count, total_time_spent,
        courses_enrolled, courses_completed, last_login_at, last_computed_at
    )
    SELECT
        u.user_id,
        COALESCE(session_stats.login_count, 0),
        COALESCE(session_stats.session_count, 0),
        COALESCE(session_stats.total_duration, 0),
        COALESCE(enrollment_stats.enrolled_count, 0),
        COALESCE(enrollment_stats.completed_count, 0),
        session_stats.last_login,
        CURRENT_TIMESTAMP
    FROM profile.users u
    LEFT JOIN (
        SELECT
            user_id,
            COUNT(DISTINCT DATE(session_start)) as login_count,
            COUNT(*) as session_count,
            SUM(session_duration) / 60 as total_duration,
            MAX(session_start) as last_login
        FROM profile.user_sessions
        WHERE (target_user_id IS NULL OR user_id = target_user_id)
        GROUP BY user_id
    ) session_stats ON u.user_id = session_stats.user_id
    LEFT JOIN (
        SELECT
            user_id,
            COUNT(*) as enrolled_count,
            COUNT(CASE WHEN completed_at IS NOT NULL THEN 1 END) as completed_count
        FROM profile.user_enrollments
        WHERE (target_user_id IS NULL OR user_id = target_user_id)
        GROUP BY user_id
    ) enrollment_stats ON u.user_id = enrollment_stats.user_id
    WHERE (target_user_id IS NULL OR u.user_id = target_user_id)
    ON CONFLICT (user_id) DO UPDATE SET
        total_logins = EXCLUDED.total_logins,
        sessions_count = EXCLUDED.sessions_count,
        total_time_spent = EXCLUDED.total_time_spent,
        courses_enrolled = EXCLUDED.courses_enrolled,
        courses_completed = EXCLUDED.courses_completed,
        last_login_at = EXCLUDED.last_login_at,
        last_computed_at = EXCLUDED.last_computed_at,
        updated_at = CURRENT_TIMESTAMP;
END;
$$ LANGUAGE plpgsql;

-- Function to sync device information
CREATE OR REPLACE FUNCTION profile.sync_user_device(
    p_user_id UUID,
    p_device_type device_type_enum,
    p_browser VARCHAR(50),
    p_os VARCHAR(50),
    p_screen_resolution VARCHAR(20)
)
RETURNS UUID AS $$
DECLARE
    device_uuid UUID;
BEGIN
    -- Try to find existing device
    SELECT device_id INTO device_uuid
    FROM profile.user_devices
    WHERE user_id = p_user_id
      AND device_type = p_device_type
      AND browser = p_browser
      AND operating_system = p_os;

    IF device_uuid IS NULL THEN
        -- Create new device record
        INSERT INTO profile.user_devices (
            user_id, device_type, browser, operating_system,
            screen_resolution, is_primary_device
        ) VALUES (
            p_user_id, p_device_type, p_browser, p_os, p_screen_resolution,
            NOT EXISTS(SELECT 1 FROM profile.user_devices WHERE user_id = p_user_id)
        ) RETURNING device_id INTO device_uuid;
    ELSE
        -- Update last seen
        UPDATE profile.user_devices
        SET
            last_seen_at = CURRENT_TIMESTAMP,
            session_count = session_count + 1,
            updated_at = CURRENT_TIMESTAMP
        WHERE device_id = device_uuid;
    END IF;

    RETURN device_uuid;
END;
$$ LANGUAGE plpgsql;
