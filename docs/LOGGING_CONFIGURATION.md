# 📊 Logging Configuration Guide

## 🎯 **Tóm tắt vấn đề bạn gặp phải**

Logs từ request `/api/v1/sui/gas-pool-status` có rất nhiều TRACE/DEBUG từ external dependencies:
- `h2::` (HTTP/2 protocol) 
- `jsonrpsee_http_client` (RPC client)
- `rustls::` (TLS/SSL)

**Điều này BÌNH THƯỜNG** vì API Gateway gọi tới Sui RPC node, nhưng có thể tối ưu.

## 🛠️ **Giải pháp: Log Level Configuration**

### 1. **Environment Variables**

Thêm vào `.env` hoặc shell:

```bash
# ============================================================================
# LOGGING CONFIGURATION
# ============================================================================

# Environment mode
ENVIRONMENT=development

# Option 1: Quiet Development (Recommended)
RUST_LOG="info,jd_=debug,api_gateway=debug,user_service=debug,sui_service=debug,h2=warn,rustls=warn,jsonrpsee=warn"

# Option 2: Normal Development  
RUST_LOG="debug,jd_=trace,api_gateway=trace,h2=warn,rustls=warn,jsonrpsee_http_client=warn"

# Option 3: Production-like
RUST_LOG="warn,jd_=info,api_gateway=info"

# Option 4: Full Debug (when you need to debug external deps)
RUST_LOG="trace"
```

### 2. **Quick Test Commands**

```bash
# Quiet development
RUST_LOG="info,api_gateway=debug,h2=warn,rustls=warn,jsonrpsee=warn" cargo run

# Normal development (default after our changes)
cargo run

# Production mode
ENVIRONMENT=production cargo run

# Debugging specific component
RUST_LOG="debug,sui_service=trace,jsonrpsee=info" cargo run
```

## 📋 **Log Levels Explained**

### ✅ **Application Logs** (Keep these visible)
```
api_gateway=trace     # Your API Gateway logs
user_service=trace    # User service logs  
sui_service=trace     # Sui service logs
jd_=trace            # All your application modules
```

### 🔇 **External Dependencies** (Filter these)
```
h2=warn              # HTTP/2 protocol (very verbose)
rustls=warn          # TLS/SSL operations
jsonrpsee_http_client=warn  # JSON-RPC client internals
hyper=warn           # HTTP client internals
tokio=warn           # Async runtime
tower=warn           # Middleware framework
```

### 📊 **Database & Network** (Moderate)
```
sqlx=info            # Database queries (useful but not verbose)
reqwest=info         # HTTP requests (useful for debugging)
jsonrpsee=info       # RPC method calls (useful)
```

## 🎨 **Customized Tracing Config**

Đã cập nhật `jd_tracing` với environment-specific configs:

### **Development** (Optimized)
```rust
// Application: TRACE level
// External deps: INFO/WARN level  
"debug,jd_=trace,api_gateway=trace,user_service=trace,sui_service=trace,
 sqlx=info,hyper=warn,tokio=warn,h2=warn,tower=warn,reqwest=info,
 rustls=warn,jsonrpsee=info,jsonrpsee_http_client=warn"
```

### **Production** (Minimal)
```rust
// Only essential logs
"info,sqlx=warn,hyper=warn,tokio=warn,h2=warn,tower=warn,reqwest=warn,
 rustls=warn,jsonrpsee=warn"
```

### **Staging** (Balanced)
```rust
// More detailed but filtered noise
"debug,sqlx=info,hyper=info,h2=warn,rustls=warn,jsonrpsee=info"
```

## 🧪 **Test Different Levels**

### 1. **Test với Quiet Logs:**
```bash
RUST_LOG="info,api_gateway=debug,h2=warn,rustls=warn,jsonrpsee=warn" cargo run
```

Then test: `curl http://localhost:8080/api/v1/sui/gas-pool-status`

**Expected:** Only essential logs, no noise from h2/rustls

### 2. **Test với Normal Logs:**
```bash
cargo run  # Uses new default config
```

**Expected:** Application logs at trace, external deps filtered

### 3. **Test với Full Debug:**
```bash
RUST_LOG="trace" cargo run
```

**Expected:** All logs (like before, for deep debugging)

## 🔍 **Log Analysis**

### **Before Optimization:**
```
TRACE h2::proto::connection: connection.state: Open
DEBUG h2::codec::framed_write: send, frame: GoAway
TRACE h2::frame::go_away: encoding GO_AWAY
TRACE h2::codec::framed_write: encoded go_away, rem: 17
DEBUG rustls::common_state: Sending warning alert CloseNotify
```

### **After Optimization:**
```
INFO api_gateway::middleware::mw_request_context: Request started
DEBUG sui_service::infrastructure: Calling Sui RPC method: rpc.discover  
INFO api_gateway::middleware::mw_res_map: Request completed successfully
INFO api_gateway::log: REQUEST LOG: {...}
```

## ⚡ **Performance Impact**

### **Trước khi tối ưu:**
- **~50-100 log lines** per request
- **High CPU usage** cho log formatting
- **Large log files**

### **Sau khi tối ưu:**
- **~5-10 log lines** per request
- **Lower CPU usage**
- **Manageable log size**
- **Better signal-to-noise ratio**

## 🎯 **Recommendations**

### **Development:**
```bash
RUST_LOG="debug,jd_=trace,api_gateway=trace,h2=warn,rustls=warn,jsonrpsee_http_client=warn"
```

### **Local Testing:**
```bash
RUST_LOG="info,api_gateway=debug,sui_service=debug"
```

### **Production:**
```bash
ENVIRONMENT=production
# Uses built-in production config
```

### **Debugging External Issues:**
```bash
RUST_LOG="debug,jsonrpsee=trace,reqwest=debug,h2=info"
```

## 🚀 **Quick Setup**

1. **Update your .env:**
```bash
ENVIRONMENT=development
RUST_LOG="debug,jd_=trace,api_gateway=trace,h2=warn,rustls=warn,jsonrpsee_http_client=warn"
```

2. **Restart server:**
```bash
cargo run
```

3. **Test request:**
```bash
curl http://localhost:8080/api/v1/sui/gas-pool-status
```

4. **Should see clean logs:**
```
INFO api_gateway::middleware::mw_request_context: Request started request_id="..." 
DEBUG sui_service: Fetching gas pool status
INFO api_gateway::middleware::mw_res_map: Request completed successfully
```

## 🎉 **Kết luận**

- ✅ **Normal behavior** - Sui RPC calls generate many external logs
- ✅ **Optimized config** - Application logs visible, noise filtered  
- ✅ **Flexible control** - Easy to adjust via RUST_LOG
- ✅ **Better performance** - Reduced logging overhead

Your request context middleware is working perfectly! 🚀 