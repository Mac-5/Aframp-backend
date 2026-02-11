# Request Logging Guide

## Overview

The Aframp backend now logs **every HTTP request** with comprehensive details including:
- ✅ Request ID (for tracing)
- ✅ HTTP method (GET, POST, etc.)
- ✅ Request path
- ✅ Query parameters
- ✅ Client IP address
- ✅ User agent
- ✅ Response status code
- ✅ Request duration in milliseconds
- ✅ Success/error indicators with emojis

## Log Format

### Incoming Request
```
📥 Incoming request
  request_id: 51c44e69-6205-4e07-ba6d-4964eac11f44
  method: GET
  path: /health
  query: test=value&foo=bar
  client_ip: 192.168.1.100
  user_agent: curl/8.5.0
```

### Successful Response (2xx)
```
✅ Request completed successfully
  request_id: 51c44e69-6205-4e07-ba6d-4964eac11f44
  method: GET
  path: /health
  query: test=value&foo=bar
  client_ip: 192.168.1.100
  status: 200
  duration_ms: 45
```

### Client Error (4xx)
```
⚠️  Request completed with client error
  request_id: 05d1aaaa-e89d-45ea-b214-ca77aaba3c46
  method: GET
  path: /nonexistent
  query: 
  client_ip: 192.168.1.100
  status: 404
  duration_ms: 2
```

### Server Error (5xx)
```
❌ Request failed with server error
  request_id: abc123-def456-ghi789
  method: POST
  path: /api/transaction
  query: 
  client_ip: 192.168.1.100
  status: 500
  duration_ms: 150
```

### Slow Request (>200ms)
```
🐌 Slow request completed
  request_id: c1300ea2-1f12-4e24-9a40-0778bdaccd03
  method: GET
  path: /health
  query: test=value&foo=bar
  client_ip: 192.168.1.100
  status: 200
  duration_ms: 1113
```

## Log Levels

The middleware uses different log levels based on the response:

| Status | Log Level | Emoji | Description |
|--------|-----------|-------|-------------|
| 2xx (Success) | INFO | ✅ | Request completed successfully |
| 4xx (Client Error) | WARN | ⚠️ | Client error (bad request, not found, etc.) |
| 5xx (Server Error) | ERROR | ❌ | Server error (internal error, service unavailable) |
| Slow (>200ms) | WARN | 🐌 | Request took longer than 200ms |

## Request ID

Every request gets a unique UUID that can be used to:
- Trace requests across logs
- Correlate requests with responses
- Debug issues
- Track request flow through the system

The request ID is:
1. Generated automatically if not provided
2. Can be provided by the client via `X-Request-ID` header
3. Included in all log entries for that request
4. Propagated to downstream services

## Client IP Detection

The middleware attempts to detect the client IP in this order:

1. **X-Forwarded-For header** - Used when behind a proxy/load balancer
2. **X-Real-IP header** - Alternative proxy header
3. **Direct connection** - Falls back to "unknown" if not available

Example with proxy:
```bash
curl -H "X-Forwarded-For: 192.168.1.100" http://127.0.0.1:8000/
```

## Query Parameters

All query parameters are logged for debugging:

```bash
# Request
curl "http://127.0.0.1:8000/health?test=value&foo=bar"

# Log shows
query: test=value&foo=bar
```

## User Agent

The User-Agent header is logged to identify clients:

```bash
# Request
curl -H "User-Agent: MyApp/1.0" http://127.0.0.1:8000/

# Log shows
user_agent: MyApp/1.0
```

## Performance Monitoring

### Slow Request Detection

Requests taking longer than **200ms** are automatically flagged:

```
🐌 Slow request completed
  duration_ms: 1113
```

This helps identify:
- Performance bottlenecks
- Database query issues
- External API delays
- Network problems

### Duration Tracking

Every request logs its duration in milliseconds:
- Fast requests: < 50ms
- Normal requests: 50-200ms
- Slow requests: > 200ms (logged as WARNING)

## Example Log Output

Here's a real example from the server:

```
2026-02-09T16:35:47.916107Z  INFO Aframp_Backend::middleware::logging: 📥 Incoming request
  request_id: 51c44e69-6205-4e07-ba6d-4964eac11f44
  method: GET
  path: /health
  query: 
  client_ip: unknown
  user_agent: curl/8.5.0

2026-02-09T16:35:48.144016Z  WARN Aframp_Backend::middleware::logging: 🐌 Slow request completed
  request_id: 51c44e69-6205-4e07-ba6d-4964eac11f44
  method: GET
  path: /health
  query: 
  client_ip: unknown
  status: 200
  duration_ms: 227
```

## Viewing Logs

### Real-time Logs

```bash
# Run the server and see logs in real-time
cargo run

# Or with specific log level
RUST_LOG=info cargo run
```

### Filtering Logs

```bash
# Only show request logs
cargo run 2>&1 | grep "middleware::logging"

# Only show errors
cargo run 2>&1 | grep "ERROR"

# Only show slow requests
cargo run 2>&1 | grep "🐌"

# Only show client errors (4xx)
cargo run 2>&1 | grep "⚠️"

# Only show server errors (5xx)
cargo run 2>&1 | grep "❌"
```

### Log to File

```bash
# Save all logs to a file
cargo run > server.log 2>&1

# View logs in real-time
tail -f server.log

# Search logs
grep "request_id: abc123" server.log
```

## Log Levels Configuration

Set the `RUST_LOG` environment variable in `.env`:

```env
# Very verbose (includes debug logs)
RUST_LOG=debug

# Normal (default) - includes info, warn, error
RUST_LOG=info

# Only warnings and errors
RUST_LOG=warn

# Only errors
RUST_LOG=error

# Module-specific logging
RUST_LOG=aframp_backend::middleware::logging=debug,info
```

## Structured Logging

All logs use structured fields that can be parsed by log aggregation tools:

```json
{
  "timestamp": "2026-02-09T16:35:47.916107Z",
  "level": "INFO",
  "target": "Aframp_Backend::middleware::logging",
  "message": "📥 Incoming request",
  "fields": {
    "request_id": "51c44e69-6205-4e07-ba6d-4964eac11f44",
    "method": "GET",
    "path": "/health",
    "query": "",
    "client_ip": "unknown",
    "user_agent": "curl/8.5.0"
  }
}
```

## Integration with Monitoring Tools

The structured logs can be integrated with:

### ELK Stack (Elasticsearch, Logstash, Kibana)
```bash
# Forward logs to Logstash
cargo run 2>&1 | logstash -f logstash.conf
```

### Datadog
```bash
# Use Datadog agent to collect logs
# Configure in datadog.yaml
```

### CloudWatch
```bash
# Use AWS CloudWatch agent
# Configure in cloudwatch-config.json
```

### Grafana Loki
```bash
# Use Promtail to ship logs to Loki
# Configure in promtail.yaml
```

## Debugging with Request IDs

### Trace a Specific Request

```bash
# Make a request with a custom request ID
curl -H "X-Request-ID: my-test-123" http://127.0.0.1:8000/health

# Search logs for that request
grep "my-test-123" server.log
```

### Find All Requests from a Client

```bash
# Search by client IP
grep "client_ip: 192.168.1.100" server.log

# Search by user agent
grep "user_agent: MyApp" server.log
```

### Find Slow Requests

```bash
# Find all slow requests
grep "🐌" server.log

# Find requests slower than 500ms
grep "duration_ms" server.log | awk '$NF > 500'
```

### Find Errors

```bash
# Find all client errors (4xx)
grep "⚠️" server.log

# Find all server errors (5xx)
grep "❌" server.log

# Find specific status codes
grep "status: 404" server.log
```

## Best Practices

### 1. Always Include Request ID
When reporting issues, include the request ID:
```
Issue: Health check failed
Request ID: 51c44e69-6205-4e07-ba6d-4964eac11f44
```

### 2. Monitor Slow Requests
Set up alerts for slow requests:
```bash
# Alert if more than 10 slow requests in 5 minutes
grep "🐌" server.log | tail -100 | wc -l
```

### 3. Track Error Rates
Monitor 4xx and 5xx error rates:
```bash
# Count errors in last 1000 requests
tail -1000 server.log | grep -E "(⚠️|❌)" | wc -l
```

### 4. Use Log Rotation
Prevent log files from growing too large:
```bash
# Install logrotate
sudo apt install logrotate

# Configure in /etc/logrotate.d/aframp
/var/log/aframp/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
}
```

## Security Considerations

### Sensitive Data

The logging middleware automatically:
- ✅ Does NOT log request bodies (may contain passwords, tokens)
- ✅ Does NOT log response bodies (may contain sensitive data)
- ✅ Does NOT log authentication headers
- ✅ Logs query parameters (be careful with sensitive data in URLs)

### PII (Personally Identifiable Information)

Be aware that logs may contain:
- Client IP addresses
- User agents (may reveal device info)
- Query parameters (may contain user data)

For production:
- Consider anonymizing IP addresses
- Implement log retention policies
- Restrict log access to authorized personnel
- Comply with GDPR/privacy regulations

## Troubleshooting

### Logs Not Showing

1. Check RUST_LOG environment variable:
```bash
echo $RUST_LOG
# Should be: info, debug, or trace
```

2. Set it explicitly:
```bash
RUST_LOG=info cargo run
```

3. Check if logs are going to stderr:
```bash
cargo run 2>&1 | grep "Incoming"
```

### Too Many Logs

Reduce log verbosity:
```bash
# Only show warnings and errors
RUST_LOG=warn cargo run

# Disable request logging (not recommended)
RUST_LOG=aframp_backend::middleware::logging=off,info cargo run
```

### Missing Request Details

If some fields show "unknown":
- **client_ip**: Normal for local development (127.0.0.1)
- **user_agent**: Client didn't send User-Agent header
- **query**: No query parameters in the request

## Summary

Your backend now logs **every single request** with:
- 📥 Incoming request notification
- ✅ Success indicator (2xx)
- ⚠️ Client error indicator (4xx)
- ❌ Server error indicator (5xx)
- 🐌 Slow request warning (>200ms)
- 🔍 Full request details (method, path, query, IP, user agent)
- ⏱️ Performance metrics (duration in ms)
- 🆔 Unique request ID for tracing

All logs are structured, searchable, and ready for production monitoring!
