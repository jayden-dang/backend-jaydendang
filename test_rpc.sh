#!/bin/bash

# Test RPC endpoint with various methods

# 1. Test get_user
echo "Testing user.get_user..."
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

echo -e "\n\n"

# 2. Test list_users
echo "Testing user.list_users..."
curl -X POST http://localhost:8080/api/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.list_users",
    "params": {},
    "id": 2
  }'

echo -e "\n\n"

# 3. Test create_user
echo "Testing user.create_user..."
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

echo -e "\n\n"

# 4. Test update_user
echo "Testing user.update_user..."
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

echo -e "\n\n"

# 5. Test delete_user
echo "Testing user.delete_user..."
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

echo -e "\n"