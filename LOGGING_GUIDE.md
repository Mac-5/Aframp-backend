# Comprehensive Logging Guide

## Overview

Your Aframp backend now has comprehensive logging for every process and operation. This guide explains what's logged and how to use it.

---

## Logging Features

### ✅ What's Logged

1. **Application Startup**
   - Version and environment
   - Configuration loading
   - Database pool initialization
   - Redis cache initialization
   - Stellar client initialization
   - Health checks
   - Route setup
   - Server binding and listening

2. **HTTP Requests**
   - Request ID (UUID)
   - HTTP method and path
   - Response status code
   - Response time (milliseconds)
   - Slow request warnings (>200ms)

3. **Health Checks**
   - Overall health status
   - Individual component health (DB, Cache, Stellar)
   - Response times for each component
   - Readiness probe results
   - Liveness probe results

4. **Stellar Operations**
   - Account lookups
   - Account existence checks
   - Balance queries
   - Error conditions

5. **Graceful Shutdown**
   - Shutdown signal received
   - Shutdown completion

---

## Log Levels

### Available Levels (from most to least verbose)
- `TRACE` - Very detailed, for debugging
- `DEBUG` - Detailed information for development
- `INFO` - General informational messages (default)
- `WARN` - Warning messages
- `ERROR` - Error messages

### Setting Log Level

```bash
# Environment variable
export RUST_LOG=debug

# Or in .env file
RUST_LOG=debug

# Module-specific logging
RUST_LOG=aframp=debug,sqlx=warn,hyper=warn
```

---

## Log Formats

### Development (Pretty Format)
```
2026-02-09T18:30:00.123Z  INFO aframp: 🚀 Starting Aframp backend service
    at src/main.rs:55
    in main
    with version: "0.1.0", environment: "development"
```

### Production (JSON Format)
```json
{
  "timestamp": "2026-02-09T18:30:00.123Z",
  "level": "INFO",
  "target": "aframp",
  "message": "🚀 Starting Aframp backend service",
  "version": "0.1.0",
  "environment": "production"
}
```

### Switching Formats

```bash
# Set in environment
export LOG_FORMAT=json  # or "plain"
export ENVIRONMENT=production

# Or in .env
LOG_FORMAT=json
ENVIRONMENT=production
```

---

## Example Log Output

### Startup Sequence

```
🚀 Starting Aframp backend service
  version: "0.1.0"
  environment: "development"

Server configuration loaded
  host: "127.0.0.1"
  port: "8000"

📊 Initializing database connection pool...
✅ Database connection pool initialized
  max_connections: 20

🔄 Initializing Redis cache connection pool...
✅ Cache connection pool initialized
  redis_url: "redis://127.0.0.1:6379"

⭐ Initializing Stellar client...
Stellar configuration loaded
  network: Testnet
  timeout_secs: 15
  max_retries: 3

✅ Stellar client initialized successfully

🏥 Performing Stellar health check...
✅ Stellar Horizon is healthy
  response_time_ms: 150

🧪 Demo: Testing Stellar functionality
✅ Test account exists
  address: "GCJRI..."

✅ Successfully fetched account details
  account_id: "GCJRI..."
  sequence: 12345
  balances: 2

Account balance
  balance: "10000.0000000"
  asset_type: "native"

🏥 Initializing health checker...
✅ Health checker initialized

🛣️  Setting up application routes...
✅ Routes configured

🚀 Server listening on http://127.0.0.1:8000
  address: "127.0.0.1:8000"

📡 Available endpoints:
  - GET  /
  - GET  /health
  - GET  /health/ready
  - GET  /health/live
  - GET  /api/stellar/account/{address}

✅ Server is ready to accept connections
```

### HTTP Request Logs

```
📍 Root endpoint accessed
  request_id: "550e8400-e29b-41d4-a716-446655440000"
  method: "GET"
  path: "/"
  status: 200
  duration_ms: 2

🏥 Health check requested
  request_id: "550e8400-e29b-41d4-a716-446655440001"

Database health check: OK (5ms)
Cache health check: OK (2ms)
Stellar health check: OK (150ms)

✅ Health check passed
  method: "GET"
  path: "/health"
  status: 200
  duration_ms: 157

🔍 Stellar account lookup requested
  address: "GCJRI5CIWK5IU67Q6DGA7QW52JDKRO7JEAHQKFNDUJUPEZGURDBX3LDX"
  request_id: "550e8400-e29b-41d4-a716-446655440002"

✅ Account exists, fetching details
  address: "GCJRI..."

✅ Account details fetched successfully
  address: "GCJRI..."
  balances: 2

  method: "GET"
  path: "/api/stellar/account/GCJRI..."
  status: 200
  duration_ms: 245
```

### Error Logs

```
❌ Failed to initialize database pool: connection refused
  error: "Connection refused (os error 111)"

❌ Health check failed - service unhealthy
  request_id: "550e8400-e29b-41d4-a716-446655440003"
  method: "GET"
  path: "/health"
  status: 503
  duration_ms: 5002

❌ Failed to fetch account details
  address: "INVALID_ADDRESS"
  error: "Invalid address format"
```

### Shutdown Logs

```
Shutdown signal received, starting graceful shutdown
👋 Server shutdown complete
```

---

## Monitoring & Debugging

### View Logs in Real-Time

```bash
# Start server with debug logging
RUST_LOG=debug cargo run --features database

# Filter specific modules
RUST_LOG=aframp=debug,sqlx=warn cargo run --features database

# JSON format for production
LOG_FORMAT=json ENVIRONMENT=production cargo run --release --features database
```

### Search Logs

```bash
# Find all errors
cargo run --features database 2>&1 | grep "ERROR"

# Find slow requests
cargo run --features database 2>&1 | grep "WARN.*slow"

# Find specific request ID
cargo run --features database 2>&1 | grep "550e8400-e29b-41d4-a716-446655440000"

# Find health check failures
cargo run --features database 2>&1 | grep "Health check failed"
```

### Production Log Management

```bash
# Redirect logs to file
cargo run --release --features database > /var/log/aframp/app.log 2>&1

# Rotate logs with logrotate
# Create /etc/logrotate.d/aframp:
/var/log/aframp/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 aframp aframp
}
```

---

## Request Tracing

### Request ID Tracking

Every HTTP request gets a unique UUID that's:
- Generated automatically
- Included in all logs for that request
- Propagated through the entire request lifecycle
- Returned in the `X-Request-ID` response header

### Example Request Flow

```
1. Request arrives
   request_id: "550e8400-e29b-41d4-a716-446655440000"
   method: "GET"
   path: "/api/stellar/account/GCJRI..."

2. Handler processes
   🔍 Stellar account lookup requested
   address: "GCJRI..."
   request_id: "550e8400-e29b-41d4-a716-446655440000"

3. External call
   ✅ Account exists, fetching details
   request_id: "550e8400-e29b-41d4-a716-446655440000"

4. Response sent
   status: 200
   duration_ms: 245
   request_id: "550e8400-e29b-41d4-a716-446655440000"
```

---

## Performance Monitoring

### Slow Request Detection

Requests taking longer than 200ms are automatically logged as warnings:

```
WARN Request took longer than 200ms
  request_id: "550e8400-e29b-41d4-a716-446655440000"
  method: "GET"
  path: "/api/stellar/account/GCJRI..."
  duration_ms: 1250
```

### Health Check Response Times

```
Database health check: OK (5ms)
Cache health check: OK (2ms)
Stellar health check: OK (150ms)
```

---

## Troubleshooting

### Common Issues

#### 1. No Logs Appearing
```bash
# Check RUST_LOG is set
echo $RUST_LOG

# Set it if not
export RUST_LOG=info

# Or use default
RUST_LOG=info cargo run --features database
```

#### 2. Too Many Logs
```bash
# Reduce verbosity
export RUST_LOG=warn

# Or filter specific modules
export RUST_LOG=aframp=info,sqlx=error,hyper=error
```

#### 3. Can't Find Specific Request
```bash
# Use request ID from response header
curl -v http://localhost:8000/health

# Look for X-Request-ID in response
# Then grep logs for that ID
cargo run --features database 2>&1 | grep "550e8400-e29b-41d4-a716-446655440000"
```

---

## Best Practices

### Development
1. Use `RUST_LOG=debug` for detailed information
2. Use pretty format (`LOG_FORMAT=plain`)
3. Monitor logs in real-time
4. Use request IDs to trace issues

### Production
1. Use `RUST_LOG=info` or `RUST_LOG=warn`
2. Use JSON format (`LOG_FORMAT=json`)
3. Send logs to centralized logging (ELK, Splunk, etc.)
4. Set up alerts for errors
5. Monitor slow requests
6. Rotate logs regularly

### Security
1. Never log sensitive data (passwords, tokens, keys)
2. Use the `mask_wallet_address()` function for addresses
3. Use the `redact_sensitive_data()` function for JSON
4. Review logs before sharing

---

## Integration with Monitoring Tools

### Prometheus Metrics (Future Enhancement)
```rust
// Add metrics endpoint
.route("/metrics", get(metrics))
```

### ELK Stack
```bash
# Filebeat configuration for JSON logs
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/aframp/*.log
  json.keys_under_root: true
  json.add_error_key: true
```

### Grafana Loki
```yaml
# Promtail configuration
clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: aframp
    static_configs:
      - targets:
          - localhost
        labels:
          job: aframp
          __path__: /var/log/aframp/*.log
```

---

## Summary

Your application now logs:
- ✅ Every startup step with emojis for easy scanning
- ✅ Every HTTP request with timing and status
- ✅ Every health check with component details
- ✅ Every Stellar operation with results
- ✅ Every error with context
- ✅ Graceful shutdown events

All logs include:
- Timestamps
- Log levels
- Request IDs (for HTTP requests)
- Structured fields for easy parsing
- Emojis for quick visual scanning

**Next Steps:**
1. Run the server and observe the logs
2. Make some requests and see the request tracing
3. Set up log rotation for production
4. Consider adding metrics endpoint
5. Integrate with your monitoring stack
