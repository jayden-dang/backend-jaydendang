# RPC API Test Guide

This document describes how to test the newly implemented RPC endpoint.

## Overview

The RPC endpoint is available at `/api/rpc` and follows the JSON-RPC 2.0 specification.

## Available Methods

The following RPC methods are available for the User entity (all prefixed with `user.`):

1. `user.create_user` - Create a new user
2. `user.get_user` - Get a user by ID
3. `user.list_users` - List all users (with optional filters)
4. `user.update_user` - Update a user
5. `user.delete_user` - Delete a user

## Testing

### Quick Test

Run the provided test script:

```bash
./test_rpc.sh
```

### Manual Testing

#### 1. Get User
```bash
curl -X POST http://localhost:8080/api/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.get_user",
    "params": {
      "id": 1
    },
    "id": 1
  }'
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "data": {
      "id": 1,
      "wallet_address": "0x1234567890abcdef",
      "nonce": "test-nonce"
    }
  }
}
```

#### 2. List Users
```bash
curl -X POST http://localhost:8080/api/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.list_users",
    "params": {},
    "id": 2
  }'
```

#### 3. Create User
```bash
curl -X POST http://localhost:8080/api/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.create_user",
    "params": {
      "data": {
        "wallet_address": "0x1234567890abcdef"
      }
    },
    "id": 3
  }'
```

#### 4. Update User
```bash
curl -X POST http://localhost:8080/api/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.update_user",
    "params": {
      "id": 1,
      "data": {
        "wallet_address": "0xnewaddress"
      }
    },
    "id": 4
  }'
```

#### 5. Delete User
```bash
curl -X POST http://localhost:8080/api/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.delete_user",
    "params": {
      "id": 1
    },
    "id": 5
  }'
```

## Implementation Details

The RPC implementation is located in:

1. **Core RPC functionality**: `/crates/core/jd_core/src/base/rpc/`
   - `rpc_params.rs` - RPC parameter types
   - `rpc_result.rs` - RPC result types
   - `macros_utils.rs` - Macros for generating RPC functions
   - `prelude.rs` - Common imports for RPC modules

2. **User RPC module**: `/crates/gateways/api_gateway/src/users/user_rpc.rs`
   - Contains the User RPC implementation with mock data
   - Simple handler function that routes based on method name

3. **RPC routes**: `/crates/gateways/api_gateway/src/routes_rpc.rs`
   - Simple RPC handler that routes requests to appropriate modules
   - Implements JSON-RPC 2.0 response format

## Next Steps

To use this RPC implementation with real data:

1. Replace the mock `UserBmc` implementation in `user_rpc.rs` with actual database operations
2. Add proper error handling
3. Implement authentication/authorization as needed
4. Add more entities by creating similar RPC modules

## Benefits for AI Integration

This RPC implementation makes it easier for AI to interact with your API because:

1. **Standardized Interface**: All CRUD operations follow the same pattern
2. **Single Endpoint**: All operations go through `/api/rpc`
3. **Typed Parameters**: Clear parameter structures for each operation
4. **Consistent Responses**: All responses follow the JSON-RPC format
5. **Easy Extension**: New entities can be added by following the same pattern