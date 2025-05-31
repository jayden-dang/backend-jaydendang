# Unified Authentication & Authorization System

## Overview

This document describes the comprehensive authentication and authorization system implemented for the JD Blog platform. The system supports multiple authentication providers (Email/Password, Google OAuth2, GitHub OAuth, and Wallet-based authentication) with role-based access control (RBAC).

## Features

- **Multiple Authentication Providers**: Email/Password, Google OAuth2, GitHub OAuth, Sui Wallet
- **Unified User Identity**: Single user account can have multiple authentication methods
- **Role-Based Access Control**: 5-tier role system (Normal, Member, VIP, Moderator, Admin)
- **Permission System**: Granular permissions for resources and actions
- **Session Management**: JWT-based authentication with session tracking
- **Profile Management**: Comprehensive user profiles with privacy controls
- **Real-time Middleware**: Authorization middleware for route protection

## Database Schema

### Core Tables

1. **unified_auth.users** - Core user identity
2. **unified_auth.user_auth_providers** - Multiple auth methods per user
3. **unified_auth.user_profiles** - Extended profile information
4. **unified_auth.permissions** - System permissions
5. **unified_auth.role_permissions** - Role-based permission assignments
6. **unified_auth.user_sessions** - Active session tracking
7. **unified_auth.auth_nonces** - Wallet signature verification

### User Roles & Permissions

#### Roles (Hierarchical)
- **Normal** (Level 0): Basic read access
- **Member** (Level 1): Can create content
- **VIP** (Level 2): Premium features + content publishing
- **Moderator** (Level 3): Content moderation powers
- **Admin** (Level 4): Full system access

#### Permission Categories
- **User Management**: `users.read.own`, `users.write.own`, `users.moderate`, etc.
- **Content Management**: `content.read`, `content.write.own`, `content.moderate`, etc.
- **Comments**: `comments.read`, `comments.write`, `comments.moderate`, etc.
- **System**: `system.admin`, `system.analytics`, `system.logs`

## Authentication Flows

### 1. Email/Password Registration & Login

```rust
// Registration
let auth_service = UnifiedAuthService::new(oauth_client);
let result = auth_service.register_with_email(
    "user@example.com".to_string(),
    "username".to_string(),
    "secure_password".to_string(),
    Some("Display Name".to_string()),
).await?;

// Login
let result = auth_service.login_with_email(
    "user@example.com".to_string(),
    "secure_password".to_string(),
).await?;
```

### 2. OAuth2 Authentication (Google/GitHub)

```rust
// Initiate OAuth flow
let auth_url = auth_service.initiate_oauth_login(
    AuthProviderType::Google,
    Some("https://myapp.com/dashboard".to_string()),
).await?;

// Handle callback
let result = auth_service.complete_oauth_login(
    AuthProviderType::Google,
    code,
    state,
).await?;
```

### 3. Wallet-based Authentication

```rust
// Login with wallet signature
let result = auth_service.login_with_wallet(
    wallet_address,
    public_key,
    signature,
    nonce,
).await?;
```

## Authorization Middleware

### Route Protection

```rust
use api_gateway::middleware::mw_auth_rbac::*;

// Protect routes with authentication
let protected_routes = Router::new()
    .route("/profile", get(get_profile))
    .layer(middleware::from_fn_with_state(ctx.clone(), mw_require_auth));

// Protect with specific roles
let admin_routes = Router::new()
    .route("/admin/users", get(list_all_users))
    .layer(middleware::from_fn(require_admin!()));

// Protect with specific permissions
let content_routes = Router::new()
    .route("/content/create", post(create_content))
    .layer(middleware::from_fn(require_permission("content.write.own")));
```

### Extracting Auth Context

```rust
use api_gateway::middleware::mw_auth_rbac::{AuthContext, AuthContextExt};

async fn protected_handler(req: Request) -> impl IntoResponse {
    let auth = req.require_auth_context()?;

    // Check permissions
    if auth.has_permission("users.moderate") {
        // Allow moderation actions
    }

    // Check role
    if auth.has_role(UserRole::Admin) {
        // Admin-only functionality
    }

    // Access user info
    let user_id = auth.user_id;
    let username = &auth.username;
}
```

## User Profile System

### Profile Creation & Management

```rust
use user_service::domain::{UnifiedUser, UserProfile, SubscriptionTier};

// Create user with profile
let user_create = UnifiedUserForCreate::with_oauth_data(
    username,
    email,
    display_name,
    avatar_url,
);

// Update profile
let profile_update = UserProfileForUpdate {
    first_name: Some("John".to_string()),
    last_name: Some("Doe".to_string()),
    bio: Some("Software developer passionate about Rust".to_string()),
    experience_level: Some(ExperienceLevel::Advanced),
    subscription_tier: Some(SubscriptionTier::Premium),
    // ... other fields
};
```

### Privacy Controls

```rust
// Check if profile is public
if user.is_profile_public() {
    let public_profile = user.to_public_profile();
    // Show public profile
}

// Get appropriate profile representation
let summary = user.to_summary(); // Safe for lists
let full_profile = user.profile; // Only for authorized users
```

## Multi-Provider Authentication

### Adding Providers to Existing Users

```rust
// User can link multiple auth providers
let result = auth_service.add_auth_provider_to_user(
    user_id,
    AuthProviderType::Github,
).await?;

// List user's auth providers
let providers = auth_service.get_user_providers(user_id).await?;

// Remove a provider (if not the last one)
auth_service.remove_auth_provider(user_id, provider_id).await?;
```

### Provider Management

```rust
// Check what providers a user has
for provider in user_providers {
    match provider.provider_type {
        AuthProviderType::Email => println!("Email: {}", provider.provider_email.unwrap()),
        AuthProviderType::Google => println!("Google: {}", provider.provider_user_id),
        AuthProviderType::Github => println!("GitHub: {}", provider.provider_user_id),
        AuthProviderType::Wallet => println!("Wallet: {}", provider.wallet_address.unwrap()),
    }
}
```

## Configuration
### Environment Variables

```bash
# OAuth Configuration
GOOGLE_OAUTH_CLIENT_ID=your_google_client_id
GOOGLE_OAUTH_CLIENT_SECRET=your_google_client_secret
GOOGLE_OAUTH_REDIRECT_URI=https://yourapp.com/auth/google/callback

GITHUB_OAUTH_CLIENT_ID=your_github_client_id
GITHUB_OAUTH_CLIENT_SECRET=your_github_client_secret
GITHUB_OAUTH_REDIRECT_URI=https://yourapp.com/auth/github/callback

# JWT Configuration
JWT_SECRET=your_jwt_secret
JWT_EXPIRATION=3600  # 1 hour

# Database
DATABASE_URL=postgresql://user:pass@localhost/jdblog
```

### OAuth Client Setup

```rust
use auth_service::infrastructure::oauth::*;

let mut oauth_client = OAuthClient::new();

// Add Google OAuth
let google_provider = GoogleOAuthProvider::new(
    env::var("GOOGLE_OAUTH_CLIENT_ID")?,
    env::var("GOOGLE_OAUTH_CLIENT_SECRET")?,
    env::var("GOOGLE_OAUTH_REDIRECT_URI")?,
);
oauth_client.add_provider(Box::new(google_provider));

// Add GitHub OAuth
let github_provider = GitHubOAuthProvider::new(
    env::var("GITHUB_OAUTH_CLIENT_ID")?,
    env::var("GITHUB_OAUTH_CLIENT_SECRET")?,
    env::var("GITHUB_OAUTH_REDIRECT_URI")?,
);
oauth_client.add_provider(Box::new(github_provider));
```

## Migration Guide

### Upgrading from Existing System

1. **Run Database Migration**:
   ```bash
   # Apply the new unified auth schema
   psql -d jdblog -f sql/0005_unified_auth_rbac.sql
   ```

2. **Migrate Existing Users**:
   ```sql
   -- Migrate existing profile.users to unified_auth.users
   -- Migrate existing auth.users to unified_auth.user_auth_providers
   -- See migration scripts in sql/ directory
   ```

3. **Update Application Code**:
   - Replace old auth middleware with new RBAC middleware
   - Update user repositories to use unified schema
   - Update route handlers to use new AuthContext

## Security Considerations

### Password Security
- Passwords are hashed using bcrypt/argon2
- Minimum password strength requirements enforced
- Rate limiting on login attempts

### OAuth Security
- State parameter prevents CSRF attacks
- Token encryption for stored OAuth tokens
- Automatic token refresh handling

### Wallet Security
- Cryptographic signature verification
- Nonce-based replay attack prevention
- Address format validation

### Session Security
- JWT tokens with expiration
- Session tracking and revocation
- Device/IP tracking for suspicious activity

## API Examples

### Authentication Endpoints

```
POST /auth/register
POST /auth/login
POST /auth/logout
GET  /auth/me

GET  /auth/oauth/{provider}/login
GET  /auth/oauth/{provider}/callback
POST /auth/wallet/nonce
POST /auth/wallet/verify

POST /auth/providers/add
DELETE /auth/providers/{provider_id}
GET  /auth/providers
```

### User Profile Endpoints

```
GET    /users/me/profile
PUT    /users/me/profile
GET    /users/{user_id}/profile  # Public profiles only
GET    /users/me/settings
PUT    /users/me/settings
```

### Admin Endpoints

```
GET    /admin/users          # Requires admin role
POST   /admin/users/moderate # Requires moderator role
GET    /admin/analytics      # Requires system.analytics permission
```

## Testing

### Unit Tests
```bash
cargo test --package auth_service
cargo test --package user_service
cargo test --package api_gateway
```

### Integration Tests
```bash
# Test complete auth flows
cargo test --test integration_auth_flows

# Test RBAC middleware
cargo test --test middleware_rbac
```

## Deployment Notes

1. **Database**: Ensure PostgreSQL 12+ with UUID extension
2. **Environment**: Set all required environment variables
3. **SSL**: Use HTTPS in production for OAuth redirects
4. **Monitoring**: Set up logging for auth events and failed attempts
5. **Backup**: Regular database backups for user data

## Troubleshooting

### Common Issues

1. **OAuth Callback Errors**: Check redirect URI configuration
2. **JWT Validation Failures**: Verify JWT secret and expiration settings
3. **Permission Denied**: Check user role and permission assignments
4. **Database Connection**: Verify connection string and credentials

### Debug Commands

```sql
-- Check user's permissions
SELECT * FROM unified_auth.get_user_permissions('user-uuid-here');

-- Check user's auth providers
SELECT * FROM unified_auth.user_auth_providers WHERE user_id = 'user-uuid-here';

-- Clean up expired sessions/nonces
SELECT unified_auth.cleanup_expired_sessions();
SELECT unified_auth.cleanup_expired_nonces();
```

This unified authentication system provides a robust, scalable foundation for user management with modern security practices and flexible authentication options.
