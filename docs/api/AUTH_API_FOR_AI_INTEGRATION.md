# Auth Service API Documentation for AI Frontend Integration

## Base Information
- **Base URL**: `http://localhost:8080` (development)
- **API Version**: v1
- **Authentication**: JWT Bearer Token (for protected routes)
- **Content-Type**: `application/json`

## Authentication Flow Overview

The authentication system uses Sui blockchain wallet signatures instead of traditional username/password:

1. **Generate Nonce** → Get a unique message to sign
2. **Sign Message** → User signs with their Sui wallet
3. **Verify Signature** → Server validates and returns JWT tokens
4. **Use Access Token** → Include in Authorization header for protected routes
5. **Refresh Token** → Get new access token when expired

---

## API Endpoints

### 1. Generate Nonce

**Purpose**: Generate a cryptographically secure nonce for wallet signature authentication.

**Endpoint**: `POST /api/v1/auth/nonce`

**Headers**:
```json
{
  "Content-Type": "application/json"
}
```

**Request Body**:
```json
{
  "address": "string" // Sui wallet address (66 chars: 0x + 64 hex)
}
```

**Request Example**:
```json
{
  "address": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
}
```

**Response (200 OK)**:
```json
{
  "nonce": "string", // 64-character hex string
  "message": "string" // Full message to sign with wallet
}
```

**Response Example**:
```json
{
  "nonce": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
  "message": "Please sign this nonce with your wallet"
}
```

**Error Responses**:
- `400 Bad Request`: Invalid address format
- `500 Internal Server Error`: Server error

---

### 2. Verify Signature

**Purpose**: Verify wallet signature and authenticate user, returning JWT tokens.

**Endpoint**: `POST /api/v1/auth/verify`

**Headers**:
```json
{
  "Content-Type": "application/json"
}
```

**Request Body**:
```json
{
  "address": "string", // Same address used for nonce
  "signature": "string", // Wallet signature of the message
  "public_key": "string" // Wallet public key for verification
}
```

**Request Example**:
```json
{
  "address": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "signature": "base64_encoded_signature_from_wallet",
  "public_key": "base64_encoded_public_key_from_wallet"
}
```

**Response (200 OK)**:
```json
{
  "success": true,
  "user": {
    "address": "string",
    "public_key": "string",
    "created_at": "string", // ISO 8601 timestamp
    "last_login": "string", // ISO 8601 timestamp
    "login_count": "number"
  },
  "tokens": {
    "access_token": "string", // JWT access token (15 min expiry)
    "refresh_token": "string" // JWT refresh token (7 days expiry)
  }
}
```

**Response Example**:
```json
{
  "success": true,
  "user": {
    "address": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "public_key": "base64_encoded_public_key",
    "created_at": "2025-01-15T10:30:00Z",
    "last_login": "2025-01-15T10:30:00Z",
    "login_count": 1
  },
  "tokens": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

**Error Responses**:
- `400 Bad Request`: Invalid request data or validation errors
- `401 Unauthorized`: Invalid signature or authentication failed
- `404 Not Found`: Nonce not found or expired
- `500 Internal Server Error`: Server error

---

### 3. Refresh Token

**Purpose**: Get a new access token using refresh token.

**Endpoint**: `POST /api/v1/auth/refresh`

**Headers**:
```json
{
  "Content-Type": "application/json"
}
```

**Request Body**:
```json
{
  "refresh_token": "string" // Valid refresh token from login
}
```

**Request Example**:
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response (200 OK)**:
```json
{
  "access_token": "string" // New JWT access token
}
```

**Response Example**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error Responses**:
- `401 Unauthorized`: Invalid or expired refresh token
- `500 Internal Server Error`: Server error

---

### 4. Get Current User (Protected)

**Purpose**: Get authenticated user information.

**Endpoint**: `GET /api/v1/auth/me`

**Headers**:
```json
{
  "Content-Type": "application/json",
  "Authorization": "Bearer {access_token}"
}
```

**Request Body**: None

**Response (200 OK)**:
```json
{
  "user": {
    "address": "string",
    "public_key": "string", 
    "created_at": "string", // ISO 8601 timestamp
    "last_login": "string", // ISO 8601 timestamp
    "login_count": "number"
  }
}
```

**Response Example**:
```json
{
  "user": {
    "address": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "public_key": "base64_encoded_public_key",
    "created_at": "2025-01-15T10:30:00Z",
    "last_login": "2025-01-15T10:45:00Z", 
    "login_count": 5
  }
}
```

**Error Responses**:
- `401 Unauthorized`: Missing, invalid, or expired access token
- `500 Internal Server Error`: Server error

---

## Frontend Integration Guide

### Required Dependencies

For JavaScript/TypeScript frontend:
```json
{
  "@suiet/wallet-kit": "^0.2.x", // For Sui wallet integration
  "axios": "^1.6.x", // For HTTP requests  
  "@noble/ed25519": "^2.0.x" // For signature verification (optional)
}
```

### Step-by-Step Integration

#### 1. Install Sui Wallet Integration
```bash
npm install @suiet/wallet-kit
```

#### 2. Wallet Connection Component
```javascript
import { ConnectButton, useWallet } from '@suiet/wallet-kit';

function WalletAuth() {
  const { connected, account, signMessage } = useWallet();
  
  if (!connected) {
    return <ConnectButton />;
  }
  
  return <AuthFlow account={account} signMessage={signMessage} />;
}
```

#### 3. Authentication Flow Implementation
```javascript
async function authenticateWithWallet(account, signMessage) {
  try {
    // Step 1: Get nonce
    const nonceResponse = await fetch('/api/v1/auth/nonce', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ address: account.address })
    });
    
    const { nonce, message } = await nonceResponse.json();
    
    // Step 2: Sign message with wallet
    const signatureResult = await signMessage({
      message: new TextEncoder().encode(message)
    });
    
    // Step 3: Verify signature
    const verifyResponse = await fetch('/api/v1/auth/verify', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        address: account.address,
        signature: signatureResult.signature,
        public_key: account.publicKey
      })
    });
    
    const authResult = await verifyResponse.json();
    
    if (authResult.success) {
      // Store tokens securely
      localStorage.setItem('access_token', authResult.tokens.access_token);
      localStorage.setItem('refresh_token', authResult.tokens.refresh_token);
      return authResult.user;
    }
    
  } catch (error) {
    console.error('Authentication failed:', error);
    throw error;
  }
}
```

#### 4. API Client with Auto-Refresh
```javascript
class AuthApiClient {
  constructor(baseURL = 'http://localhost:8080') {
    this.baseURL = baseURL;
    this.accessToken = localStorage.getItem('access_token');
    this.refreshToken = localStorage.getItem('refresh_token');
  }
  
  async makeRequest(endpoint, options = {}) {
    const url = `${this.baseURL}${endpoint}`;
    const headers = {
      'Content-Type': 'application/json',
      ...options.headers
    };
    
    if (this.accessToken) {
      headers.Authorization = `Bearer ${this.accessToken}`;
    }
    
    let response = await fetch(url, { ...options, headers });
    
    // Auto-refresh on 401
    if (response.status === 401 && this.refreshToken) {
      const refreshed = await this.refreshAccessToken();
      if (refreshed) {
        headers.Authorization = `Bearer ${this.accessToken}`;
        response = await fetch(url, { ...options, headers });
      }
    }
    
    return response;
  }
  
  async refreshAccessToken() {
    try {
      const response = await fetch(`${this.baseURL}/api/v1/auth/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refresh_token: this.refreshToken })
      });
      
      if (response.ok) {
        const { access_token } = await response.json();
        this.accessToken = access_token;
        localStorage.setItem('access_token', access_token);
        return true;
      }
    } catch (error) {
      console.error('Token refresh failed:', error);
    }
    
    // Refresh failed, clear tokens
    this.logout();
    return false;
  }
  
  logout() {
    this.accessToken = null;
    this.refreshToken = null;
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
  }
  
  async getCurrentUser() {
    const response = await this.makeRequest('/api/v1/auth/me');
    return response.json();
  }
}
```

#### 5. React Hook Example
```javascript
import { useState, useEffect } from 'react';

function useAuth() {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const apiClient = new AuthApiClient();
  
  useEffect(() => {
    checkAuth();
  }, []);
  
  async function checkAuth() {
    try {
      if (apiClient.accessToken) {
        const userData = await apiClient.getCurrentUser();
        setUser(userData.user);
      }
    } catch (error) {
      console.error('Auth check failed:', error);
    } finally {
      setLoading(false);
    }
  }
  
  async function login(account, signMessage) {
    setLoading(true);
    try {
      const userData = await authenticateWithWallet(account, signMessage);
      setUser(userData);
      return userData;
    } finally {
      setLoading(false);
    }
  }
  
  function logout() {
    apiClient.logout();
    setUser(null);
  }
  
  return { user, loading, login, logout, isAuthenticated: !!user };
}
```

---

## Error Handling

### Standard Error Response Format
```json
{
  "error": "string", // Error type
  "message": "string", // Human-readable error message
  "code": "string" // Optional error code
}
```

### Common Error Scenarios

1. **Invalid Address Format**:
   ```json
   {
     "error": "Bad Request",
     "message": "Address must be 66 characters (0x + 64 hex)",
     "code": "INVALID_ADDRESS"
   }
   ```

2. **Nonce Expired**:
   ```json
   {
     "error": "Not Found", 
     "message": "Nonce not found or expired",
     "code": "NONCE_EXPIRED"
   }
   ```

3. **Invalid Signature**:
   ```json
   {
     "error": "Unauthorized",
     "message": "Invalid signature",
     "code": "INVALID_SIGNATURE"
   }
   ```

4. **Token Expired**:
   ```json
   {
     "error": "Unauthorized",
     "message": "Access token expired",
     "code": "TOKEN_EXPIRED"
   }
   ```

---

## Security Considerations

1. **Nonce Expiration**: Nonces expire after 5 minutes
2. **Token Expiration**: 
   - Access tokens: 15 minutes
   - Refresh tokens: 7 days
3. **HTTPS Required**: Use HTTPS in production
4. **Secure Storage**: Store tokens securely (avoid localStorage in production)
5. **CORS Configuration**: Server supports specified origins only

---

## Testing Tips

1. **Use Mock Data**: For development, test with mock signatures
2. **Error Simulation**: Test all error scenarios
3. **Token Refresh**: Test automatic token refresh flow
4. **Wallet Integration**: Test with actual Sui wallet in development
5. **Network Errors**: Handle network connectivity issues

---

## Environment Variables

Frontend should support these environment variables:
```javascript
const API_BASE_URL = process.env.REACT_APP_API_BASE_URL || 'http://localhost:8080';
const ENABLE_AUTH_DEBUG = process.env.REACT_APP_AUTH_DEBUG === 'true';
```

Server requires:
```bash
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-super-secret-jwt-key-change-in-production
``` 