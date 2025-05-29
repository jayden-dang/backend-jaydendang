# 🛠️ Makefile.toml Usage Guide

## 🎯 **Tổng quan**
File `Makefile.toml` đã được cập nhật để tích hợp với `jd_tracing` config mới, cung cấp logging tối ưu cho từng environment và use case.

## 🚀 **Quick Start Commands**

### **Development Mode**
```bash
# Recommended: Trace-level application logs, filtered external noise  
cargo make dev

# Quiet mode: Essential logs only
cargo make dev-quiet

# Info level: Balanced logging
cargo make dev-info

# Full debug: All logs including external dependencies
cargo make dev-debug
```

### **Environment-Specific**
```bash
# Staging environment
cargo make staging

# Production environment 
cargo make production

# Testing environment
cargo make test-env
```

### **Specialized Debug**
```bash
# Debug database operations
cargo make debug-db

# Debug Sui RPC calls
cargo make debug-sui

# Debug authentication & middleware
cargo make debug-auth
```

### **Complete Setup**
```bash
# Start all services + development mode
cargo make run-all

# Start all services + quiet logging
cargo make run-quiet

# Start all services + staging
cargo make run-staging

# Start all services + production
cargo make run-production
```

## 📊 **Logging Levels Comparison**

| Task | Application | SQLx | External Deps | Use Case |
|------|-------------|------|---------------|----------|
| `dev` | **TRACE** | INFO | WARN | Full development |
| `dev-quiet` | **DEBUG** | WARN | WARN | Clean development |
| `dev-info` | **INFO** | INFO | WARN | Balanced development |
| `dev-debug` | **TRACE** | TRACE | TRACE | Deep debugging |
| `staging` | **DEBUG** | INFO | WARN/INFO | Staging environment |
| `production` | **INFO** | WARN | WARN | Production environment |
| `test-env` | **ERROR** | - | - | Testing only |

## 🔍 **Specialized Debug Tasks Explained**

### 1. **Database Debug (`debug-db`)**
```bash
cargo make debug-db
```

**Logs visible:**
- `jd_storage=trace` - All database operations
- `sqlx=debug` - SQL query details
- `sea_query=debug` - Query building
- External dependencies filtered

**Use when:** Database queries are slow or failing

### 2. **Sui RPC Debug (`debug-sui`)**
```bash
cargo make debug-sui
```

**Logs visible:**
- `sui_service=trace` - All Sui service operations
- `jsonrpsee=debug` - RPC call details
- `reqwest=debug` - HTTP request details
- `fastcrypto=info` - Cryptographic operations
- `h2=info` - HTTP/2 details (for RPC debugging)

**Use when:** Sui RPC calls are failing or slow

### 3. **Authentication Debug (`debug-auth`)**
```bash
cargo make debug-auth
```

**Logs visible:**
- `api_gateway=trace` - All gateway operations
- `api_gateway::middleware=trace` - Middleware details
- `user_service=debug` - User operations
- `axum=info` - Web framework details
- `tower=info` - Middleware stack

**Use when:** Authentication or authorization issues

## 🎨 **Log Output Examples**

### **Development Mode (`cargo make dev`)**
```
INFO  api_gateway::middleware::mw_request_context: Request started request_id="abc123"
DEBUG sui_service::infrastructure: Calling Sui RPC method: rpc.discover
TRACE api_gateway::middleware::mw_auth: Checking authentication
INFO  api_gateway::middleware::mw_res_map: Request completed successfully
```

### **Quiet Mode (`cargo make dev-quiet`)**
```
INFO  api_gateway::middleware::mw_request_context: Request started request_id="abc123"
DEBUG api_gateway::users: Processing user request
INFO  api_gateway::middleware::mw_res_map: Request completed successfully
```

### **Database Debug (`cargo make debug-db`)**
```
INFO  api_gateway::middleware::mw_request_context: Request started request_id="abc123"
TRACE jd_storage::pool: Getting connection from pool
DEBUG sqlx::query: SELECT id, email FROM users WHERE id = $1 [duration=2.34ms rows=1]
TRACE sea_query: Building query: SELECT "users"."id", "users"."email" FROM "users"
INFO  api_gateway::middleware::mw_res_map: Request completed successfully
```

### **Sui Debug (`cargo make debug-sui`)**
```
INFO  api_gateway::middleware::mw_request_context: Request started request_id="abc123"
TRACE sui_service::infrastructure: Creating Sui client for network: mainnet
DEBUG jsonrpsee: Calling method: sui_getGasObjects params=[...]
DEBUG reqwest: POST https://fullnode.mainnet.sui.io:443 [200 OK] [1.2s]
INFO  sui_service::gas_pool: Found 5 gas objects, total balance: 1000000000
```

## ⚡ **Performance Impact**

### **Before Optimization (old config):**
```
TRACE h2::proto::connection: connection.state: Open
DEBUG h2::codec::framed_write: send, frame: GoAway  
TRACE h2::frame::go_away: encoding GO_AWAY
TRACE h2::codec::framed_write: encoded go_away, rem: 17
DEBUG rustls::common_state: Sending warning alert CloseNotify
```
**Result:** ~50-100 log lines per request

### **After Optimization (new config):**
```
INFO  api_gateway::middleware::mw_request_context: Request started
DEBUG sui_service: Fetching gas pool status
INFO  api_gateway::middleware::mw_res_map: Request completed successfully
```
**Result:** ~5-10 log lines per request

## 🛡️ **Environment Security**

### **Development:** Full visibility
- All application logs at TRACE level
- External dependencies filtered but still informative
- Perfect for debugging

### **Staging:** Balanced approach
- Application logs at DEBUG level
- Some external dependency details
- Good for integration testing

### **Production:** Minimal exposure
- Only INFO level and above
- External dependencies at WARN level
- No sensitive debugging information

### **Testing:** Silent operation
- Only ERROR level for applications
- Minimal noise for test output

## 🔧 **Customization**

### **Custom RUST_LOG Override**
```bash
# Override any task's logging
RUST_LOG="trace,h2=error" cargo make dev

# Environment-specific override
ENVIRONMENT=production RUST_LOG="debug" cargo make dev
```

### **Adding New Debug Tasks**
```toml
[tasks.debug-new-feature]
description = "Debug new feature"
install_crate = "cargo-watch"
cwd = "./crates/gateways/web_server"
command = "cargo"
args = ["watch", "-x", "clippy", "-x", "run"]

[tasks.debug-new-feature.env]
RUST_LOG = "info,jd_=debug,new_feature=trace"
ENVIRONMENT = "development"
```

## 📋 **Best Practices**

### **Daily Development:**
```bash
cargo make dev  # Start with optimized logging
```

### **Debugging Issues:**
```bash
# Database issues
cargo make debug-db

# Sui/RPC issues  
cargo make debug-sui

# Auth/middleware issues
cargo make debug-auth

# Unknown issues
cargo make dev-debug  # Full trace
```

### **Testing:**
```bash
cargo make test-env  # Minimal logs
```

### **Staging Deployment:**
```bash
cargo make run-staging  # Balanced logging
```

### **Production Deployment:**
```bash
cargo make run-production  # Minimal, secure logging
```

## 🚀 **Migration từ Config Cũ**

### **Trước đây:**
```bash
cargo make dev  # Quá nhiều noise từ external deps
```

### **Bây giờ:**
```bash
cargo make dev        # Optimized logging
cargo make dev-quiet  # Nếu vẫn thấy nhiều
cargo make dev-debug  # Khi cần full trace
```

## 🎉 **Kết luận**

Makefile.toml mới cung cấp:

- ✅ **Optimized logging** cho từng use case
- ✅ **Environment-specific** configurations  
- ✅ **Specialized debug** tasks
- ✅ **Performance improvements** (ít log noise hơn)
- ✅ **Flexible control** qua RUST_LOG override
- ✅ **Backward compatibility** với legacy tasks

**Recommended workflow:**
1. `cargo make dev` cho daily development
2. `cargo make debug-*` khi có specific issues
3. `cargo make dev-debug` khi cần full trace
4. `cargo make run-staging/production` cho deployment

Your logging is now **optimized, organized, and powerful**! 🚀 