# Auth Service

A Rust-based authentication service for Sui blockchain wallet signature verification. This service provides JWT-based authentication using wallet signatures instead of traditional username/password authentication.

## Features

- 🔐 **Wallet-based Authentication**: Users authenticate using their Sui wallet signatures
- 🎫 **JWT Token Management**: Access and refresh token generation and validation
- 🔄 **Nonce-based Security**: Cryptographically secure nonces prevent replay attacks
- 📊 **Redis Integration**: Fast nonce storage and retrieval
- 🗄️ **PostgreSQL Support**: Persistent user data storage
- 🛡️ **Signature Verification**: Sui blockchain signature validation using fastcrypto
- 🚀 **Axum Integration**: Ready-to-use HTTP handlers and middleware

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client App    │    │   Auth Service  │    │   Database      │
│                 │    │                 │    │                 │
│ 1. Request      │───▶│ Generate Nonce  │───▶│ Store Nonce     │
│    Nonce        │    │                 │    │ (Redis)         │
│                 │    │                 │    │                 │
│ 2. Sign Message │    │                 │    │                 │
│    with Wallet  │    │                 │    │                 │
│                 │    │                 │    │                 │
│ 3. Send         │───▶│ Verify          │───▶│ Get/Create User │
│    Signature    │    │ Signature       │    │ (PostgreSQL)    │
│                 │    │                 │    │                 │
│ 4. Receive      │◀───│ Issue JWT       │    │                 │
│    JWT Tokens   │    │ Tokens          │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## API Endpoints

### 1. Generate Nonce
**POST** `/api/auth/nonce`

Generate a cryptographically secure nonce for wallet signing.

**Request:**
```json
{
  "address": "0x123...abc"
}
```

**Response:**
```json
{
  "nonce": "64-character-hex-string",
  "message": "Sign this message to authenticate with Commandos HKT: {nonce}"
}
```

### 2. Verify Signature
**POST** `/api/auth/verify`

Verify wallet signature and issue JWT tokens.

**Request:**
```json
{
  "address": "0x123...abc",
  "signature": "base64-encoded-signature",
  "publicKey": "base64-encoded-public-key"
}
```

**Response:**
```json
{
  "success": true,
  "user": {
    "address": "0x123...abc",
    "publicKey": "base64-encoded-public-key",
    "createdAt": 1703123456789,
    "lastLogin": 1703123456789,
    "loginCount": 5
  },
  "tokens": {
    "accessToken": "jwt-access-token",
    "refreshToken": "jwt-refresh-token"
  }
}
```

### 3. Refresh Token
**POST** `/api/auth/refresh`

Refresh access token using refresh token.

**Request:**
```json
{
  "refreshToken": "jwt-refresh-token"
}
```

**Response:**
```json
{
  "accessToken": "new-jwt-access-token"
}
```

### 4. Get Current User
**GET** `/api/auth/me`

Get current authenticated user information (requires valid access token).

**Headers:**
```
Authorization: Bearer {access-token}
```

**Response:**
```json
{
  "address": "0x123...abc",
  "publicKey": "base64-encoded-public-key",
  "createdAt": 1703123456789,
  "lastLogin": 1703123456789,
  "loginCount": 5
}
```

## Usage

### 1. Add to Cargo.toml

```toml
[dependencies]
auth_service = { path = "path/to/auth_service" }
```

### 2. Setup Dependencies

```rust
use auth_service::{
    application::AuthService,
    infrastructure::{RedisNonceRepository, PostgresUserRepository, SuiSignatureVerifier},
};
use sqlx::PgPool;
use std::sync::Arc;

// Database connection
let pool = PgPool::connect(&database_url).await?;

// Redis connection
let nonce_repo = Arc::new(RedisNonceRepository::new(&redis_url)?);

// User repository
let user_repo = Arc::new(PostgresUserRepository::new(pool));

// Signature verifier
let signature_verifier = Arc::new(SuiSignatureVerifier::new());

// Create auth service
let auth_service = Arc::new(AuthService::new(
    nonce_repo,
    user_repo,
    signature_verifier,
    jwt_secret,
));
```

### 3. Setup Axum Routes

```rust
use axum::{middleware, routing::{get, post}, Router, Extension};
use auth_service::application::{
    generate_nonce, verify_signature, refresh_token, 
    get_current_user, auth_middleware
};

let app = Router::new()
    // Public routes
    .route("/api/auth/nonce", post(generate_nonce))
    .route("/api/auth/verify", post(verify_signature))
    .route("/api/auth/refresh", post(refresh_token))
    // Protected routes
    .route("/api/auth/me", get(get_current_user))
    .route_layer(middleware::from_fn(auth_middleware))
    // Add auth service
    .layer(Extension(auth_service));
```

### 4. Environment Variables

```env
DATABASE_URL=postgresql://user:pass@localhost/dbname
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-super-secret-jwt-key-change-in-production
```

## Database Setup

Run the SQL migration to create the authentication schema:

```sql
-- Run sql/0004_auth.sql
psql -d your_database -f sql/0004_auth.sql
```

## Security Features

### Nonce Security
- Cryptographically secure 32-byte random nonces
- 5-minute expiration time
- One-time use only (deleted after verification)
- Stored in Redis for fast access

### JWT Security
- HS256 algorithm with configurable secret
- Access tokens: 1 hour expiration
- Refresh tokens: 7 days expiration
- Token type validation (access vs refresh)

### Signature Verification
- Uses Sui's fastcrypto library
- Secp256k1 signature verification
- Base64 encoded signatures and public keys
- Message format validation

## Error Handling

The service provides comprehensive error handling with specific error codes:

- `NONCE_NOT_FOUND`: Nonce doesn't exist
- `NONCE_EXPIRED`: Nonce has expired
- `INVALID_SIGNATURE`: Signature verification failed
- `INVALID_TOKEN`: JWT token is invalid
- `TOKEN_EXPIRED`: JWT token has expired
- `MISSING_AUTH_HEADER`: Authorization header missing
- `RATE_LIMIT_EXCEEDED`: Too many requests
- `INVALID_ADDRESS`: Invalid Sui address format

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with coverage
cargo tarpaulin --out html
```

## Performance Considerations

- **Redis**: Used for nonce storage to minimize database load
- **Connection Pooling**: PostgreSQL connection pooling via sqlx
- **Async/Await**: Fully async implementation for high concurrency
- **JWT**: Stateless authentication reduces database queries

## Rate Limiting

Recommended rate limits:
- Nonce generation: 5 requests/minute per address
- Signature verification: 3 attempts/5 minutes per address
- Token refresh: 10 requests/hour per user

## Monitoring

Key metrics to monitor:
- Nonce generation rate
- Signature verification success/failure rate
- Token generation and validation rate
- Database connection pool usage
- Redis connection health

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License. 