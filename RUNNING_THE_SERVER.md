# Running the Aframp Backend Server

## Quick Start

### Option 1: Using cargo run (Recommended for Development)
```bash
cargo run
```

### Option 2: Using the run script
```bash
./run-server.sh
```

### Option 3: Build and run separately
```bash
# Build the project
cargo build

# Run the binary
./target/debug/Aframp-Backend
```

## What You'll See

When the server starts successfully, you'll see a prominent banner like this:

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║          🚀 AFRAMP BACKEND SERVER IS RUNNING 🚀             ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  🌐 Server Address:  http://127.0.0.1:8000                    ║
║  📡 Port:            8000                                  ║
║  🏠 Host:            127.0.0.1                            ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  📡 AVAILABLE ENDPOINTS:                                     ║
║                                                              ║
║  GET  /                          - Root endpoint            ║
║  GET  /health                    - Health check             ║
║  GET  /health/ready              - Readiness probe          ║
║  GET  /health/live               - Liveness probe           ║
║  GET  /api/stellar/account/{address} - Stellar account    ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  💡 Try it out:                                              ║
║     curl http://127.0.0.1:8000                                ║
║     curl http://127.0.0.1:8000/health                        ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

## Testing the Endpoints

### Root Endpoint
```bash
curl http://127.0.0.1:8000/
# Response: Welcome to Aframp Backend API
```

### Health Check
```bash
curl http://127.0.0.1:8000/health
# Returns JSON with health status of all components
```

### Readiness Probe
```bash
curl http://127.0.0.1:8000/health/ready
# Checks if service is ready to accept traffic
```

### Liveness Probe
```bash
curl http://127.0.0.1:8000/health/live
# Checks if service is alive
```

### Stellar Account Lookup
```bash
curl http://127.0.0.1:8000/api/stellar/account/GCJRI5CIWK5IU67Q6DGA7QW52JDKRO7JEAHQKFNDUJUPEZGURDBX3LDX
# Returns account information if it exists
```

## Configuration

The server reads configuration from the `.env` file:

```env
HOST=127.0.0.1
PORT=8000
DATABASE_URL=postgresql:///aframp
REDIS_URL=redis://127.0.0.1:6379
RUST_LOG=info
```

### Changing the Port

To run on a different port, edit `.env`:

```env
PORT=3000
```

Then restart the server.

## Logging

The server logs every request with:
- Request ID (for tracing)
- HTTP method and path
- Response status code
- Duration in milliseconds
- Warnings for slow requests (>200ms)

Example log output:
```
2026-02-09T15:53:42.123456Z  INFO aframp_backend: 📍 Root endpoint accessed
2026-02-09T15:53:42.234567Z  INFO aframp_backend: ✅ Health check passed
```

### Adjusting Log Level

Set `RUST_LOG` in `.env`:

```env
RUST_LOG=debug   # Very verbose
RUST_LOG=info    # Normal (default)
RUST_LOG=warn    # Only warnings and errors
RUST_LOG=error   # Only errors
```

## Troubleshooting

### Port Already in Use

If you see an error like "Address already in use":

```bash
# Find what's using port 8000
lsof -i :8000

# Kill the process
kill -9 <PID>

# Or change the port in .env
```

### Database Connection Error

Make sure PostgreSQL is running and the database exists:

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check if database exists
psql -d aframp -c "SELECT 1;"
```

### Redis Connection Error

Make sure Redis is running:

```bash
# Check if Redis is running
redis-cli ping
# Should return: PONG

# Start Redis if not running
sudo systemctl start redis
```

## Stopping the Server

Press `Ctrl+C` to gracefully shutdown the server.

You'll see:
```
Shutdown signal received, starting graceful shutdown
👋 Server shutdown complete
```

## Development Tips

### Auto-reload on File Changes

Install cargo-watch:
```bash
cargo install cargo-watch
```

Run with auto-reload:
```bash
cargo watch -x run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Building for Production

```bash
# Build optimized binary
cargo build --release

# Run production binary
./target/release/Aframp-Backend
```

## Environment-Specific Configurations

### Development
```bash
cargo run
```

### Test Environment
```bash
# Use test database
DATABASE_URL=postgresql:///aframp_test cargo run
```

### Production
See `PRODUCTION_DEPLOYMENT.md` for production setup instructions.
