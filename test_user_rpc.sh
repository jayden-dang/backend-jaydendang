#!/bin/bash

# Test script for User RPC API
BASE_URL="http://localhost:8080/api/rpc"

echo "Testing User RPC API Integration"
echo "================================="

# Test 1: Create a new user
echo ""
echo "1. Testing create_user..."
curl -X POST "$BASE_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.create_user",
    "params": {
      "data": {
        "email": "test@example.com",
        "username": "testuser123",
        "password_hash": "hashedpassword123",
        "first_name": "Test",
        "last_name": "User"
      }
    },
    "id": 1
  }' | jq '.'

echo ""
echo "2. Testing get_user_by_username..."
curl -X POST "$BASE_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.get_user_by_username",
    "params": {
      "username": "testuser123"
    },
    "id": 2
  }' | jq '.'

echo ""
echo "3. Testing get_user_by_email..."
curl -X POST "$BASE_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.get_user_by_email",
    "params": {
      "email": "test@example.com"
    },
    "id": 3
  }' | jq '.'

echo ""
echo "4. Testing get_user_by_filter..."
curl -X POST "$BASE_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.get_user_by_filter",
    "params": {
      "filter": {
        "username": "testuser123",
        "email": null,
        "is_active": null
      }
    },
    "id": 4
  }' | jq '.'

echo ""
echo "5. Testing get_user_by_active_status..."
curl -X POST "$BASE_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "user.get_user_by_active_status",
    "params": {
      "is_active": true
    },
    "id": 5
  }' | jq '.'

echo ""
echo "Test completed!"