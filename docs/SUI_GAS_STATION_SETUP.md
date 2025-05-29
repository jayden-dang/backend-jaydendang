# Sui Gas Station Setup Guide

## Overview
Gas Station cho phép sponsor các giao dịch cho người dùng bằng cách trả phí gas từ một ví sponsor.

## Yêu cầu

### 1. Tạo Sponsor Wallet
Bạn cần một ví Sui có SUI tokens để trả phí gas:

```bash
# Cài đặt Sui CLI
curl -fLJO https://github.com/MystenLabs/sui/releases/download/sui-v1.15.0/sui-ubuntu-x86_64.tgz
tar -xzf sui-ubuntu-x86_64.tgz
sudo mv sui /usr/local/bin/

# Tạo ví mới hoặc import private key
sui client new-address ed25519  # Tạo địa chỉ mới
# hoặc
sui client import <private-key> ed25519  # Import private key có sẵn

# Lấy địa chỉ active
sui client active-address

# Export private key (cần cho sponsor transaction)
sui keytool export <address> ed25519
```

### 2. Nạp SUI vào Sponsor Wallet

**Testnet:**
```bash
# Request faucet
sui client faucet

# Kiểm tra balance
sui client gas
```

**Mainnet:**
- Chuyển SUI từ ví khác
- Mua SUI từ exchange

### 3. Cấu hình Environment Variables

Thêm vào file `.env` hoặc environment:

```bash
# Sui Network Configuration
SUI_ENV=testnet  # hoặc mainnet, devnet, local

# Gas Station Configuration  
SUI_SPONSOR_ADDRESS=0x1234567890abcdef1234567890abcdef12345678  # Địa chỉ sponsor wallet
SUI_SPONSOR_PRIVATE_KEY=0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890  # Private key (32 bytes)
SUI_MAX_GAS_BUDGET=1000000000  # 1 SUI = 1,000,000,000 MIST (optional, default: 1 SUI)
```

## Cách lấy Private Key

1. **Từ Sui CLI:**
```bash
# List keys
sui keytool list

# Export private key for specific address
sui keytool export <address> ed25519

# Copy the private key (starts with 0x)
```

2. **Security Best Practices:**
- ✅ Sử dụng testnet cho development
- ✅ Store private key trong environment variables
- ✅ Không commit private key vào git
- ✅ Sử dụng separate wallet cho sponsor
- ❌ Không hardcode private key trong code

## Gas Station Capabilities

### Level 1: Read-Only (Chỉ có Address)
```bash
SUI_SPONSOR_ADDRESS=0x...
# Không có SUI_SPONSOR_PRIVATE_KEY
```
**Available endpoints:**
- ✅ `/gas-pool-status` - Xem gas pool stats
- ✅ `/user/{address}/stats` - User statistics  
- ❌ `/sponsor` - Sponsor transaction (limited)
- ✅ `/refresh-gas-pool` - Refresh gas objects

### Level 2: Full Sponsoring (Có Address + Private Key)
```bash
SUI_SPONSOR_ADDRESS=0x...
SUI_SPONSOR_PRIVATE_KEY=0x...
```
**Available endpoints:**
- ✅ Tất cả Level 1 endpoints
- ✅ `/sponsor` - Full transaction sponsoring
- ✅ Transaction signing và submission
- ✅ Real gas object management

## API Endpoints

### Gas Station Status
```bash
GET /api/v1/sui/gas-pool-status
```

### Debug Configuration
```bash
GET /api/v1/sui/debug
```

### Test Gas Station Setup
```bash
GET /api/v1/sui/test-gas-station
```

### Sponsor Transaction (FULL IMPLEMENTATION)
```bash
POST /api/v1/sui/sponsor
Content-Type: application/json

{
  "user_address": "0x...",
  "transaction_data": {
    "sender": "0x...",
    "tx_bytes": [1, 2, 3, ...],
    "gas_data": {
      "budget": "10000000",
      "price": "1000"
    }
  },
  "user_signature": []
}
```

## Troubleshooting

### Lỗi "Sponsor keystore not available"
- Kiểm tra `SUI_SPONSOR_PRIVATE_KEY` có được set chưa
- Verify private key format (32 bytes hex với hoặc không có 0x prefix)
- Check private key matches sponsor address

### Lỗi "Private key address doesn't match sponsor address"
- Export private key từ đúng address
- Double-check sponsor address trong config

### Lỗi "Failed to parse transaction data"
- Transaction bytes phải là valid BCS-encoded TransactionData
- Check transaction format từ client

### Private Key Format Examples
```bash
# Valid formats:
SUI_SPONSOR_PRIVATE_KEY=0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890
SUI_SPONSOR_PRIVATE_KEY=abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890

# Invalid formats:
SUI_SPONSOR_PRIVATE_KEY=0xabcdef  # Too short
SUI_SPONSOR_PRIVATE_KEY=not-hex   # Not hex
```

## Development vs Production

### Development (Testnet)
```bash
SUI_ENV=testnet
SUI_SPONSOR_ADDRESS=0x...  # Testnet address
SUI_SPONSOR_PRIVATE_KEY=0x...  # Testnet private key
SUI_MAX_GAS_BUDGET=100000000  # 0.1 SUI
```

### Production (Mainnet)  
```bash
SUI_ENV=mainnet
SUI_SPONSOR_ADDRESS=0x...  # Mainnet address với nhiều SUI
SUI_SPONSOR_PRIVATE_KEY=0x...  # Mainnet private key (SECURE!)
SUI_MAX_GAS_BUDGET=1000000000  # 1 SUI
```

## Security Checklist

- [ ] Private key stored securely (environment variables)
- [ ] Separate sponsor wallet (không phải main wallet)
- [ ] Rate limiting enabled (10 requests/minute per user)
- [ ] Gas budget limits configured
- [ ] Monitoring setup cho gas pool
- [ ] Backup private key securely
- [ ] Regular balance checks
- [ ] Testnet testing trước khi deploy production 