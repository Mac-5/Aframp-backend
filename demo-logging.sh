#!/bin/bash
# Demo script to show request logging in action

echo "🎬 DEMO: Request Logging"
echo ""
echo "Starting server in background..."
RUST_LOG=info ./target/debug/Aframp-Backend > /tmp/demo-server.log 2>&1 &
SERVER_PID=$!

# Wait for server to start
sleep 3

echo "✅ Server started (PID: $SERVER_PID)"
echo ""
echo "Making test requests..."
echo ""

# Test 1: Successful request
echo "1️⃣  Testing successful request (GET /)"
curl -s http://127.0.0.1:8000/ > /dev/null
sleep 0.5

# Test 2: Health check
echo "2️⃣  Testing health check (GET /health)"
curl -s http://127.0.0.1:8000/health > /dev/null
sleep 0.5

# Test 3: 404 error
echo "3️⃣  Testing 404 error (GET /nonexistent)"
curl -s http://127.0.0.1:8000/nonexistent > /dev/null
sleep 0.5

# Test 4: Request with query parameters
echo "4️⃣  Testing request with query params (GET /health?test=123&foo=bar)"
curl -s "http://127.0.0.1:8000/health?test=123&foo=bar" > /dev/null
sleep 0.5

# Test 5: Request with custom user agent
echo "5️⃣  Testing request with custom user agent"
curl -s -H "User-Agent: MyCustomApp/1.0" http://127.0.0.1:8000/ > /dev/null
sleep 0.5

# Test 6: Request with custom request ID
echo "6️⃣  Testing request with custom request ID"
curl -s -H "X-Request-ID: demo-request-123" http://127.0.0.1:8000/health/live > /dev/null
sleep 1

echo ""
echo "📊 Showing logs..."
echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

# Show the logs with nice formatting
tail -100 /tmp/demo-server.log | grep -E "(📥|✅|⚠️|❌|🐌)" | while IFS= read -r line; do
    # Extract just the important parts
    if [[ $line == *"📥 Incoming request"* ]]; then
        echo "📥 INCOMING REQUEST"
        echo "$line" | grep -oP 'request_id: \K[^,]+' | sed 's/^/   Request ID: /'
        echo "$line" | grep -oP 'method: \K[^,]+' | sed 's/^/   Method: /'
        echo "$line" | grep -oP 'path: \K[^,]+' | sed 's/^/   Path: /'
        echo "$line" | grep -oP 'query: \K[^,]+' | sed 's/^/   Query: /'
        echo "$line" | grep -oP 'user_agent: \K[^,]+' | sed 's/^/   User-Agent: /'
        echo ""
    elif [[ $line == *"✅"* ]]; then
        echo "✅ SUCCESS"
        echo "$line" | grep -oP 'status: \K[^,]+' | sed 's/^/   Status: /'
        echo "$line" | grep -oP 'duration_ms: \K[^,]+' | sed 's/^/   Duration: /' | sed 's/$/ ms/'
        echo ""
    elif [[ $line == *"⚠️"* ]]; then
        echo "⚠️  CLIENT ERROR"
        echo "$line" | grep -oP 'status: \K[^,]+' | sed 's/^/   Status: /'
        echo "$line" | grep -oP 'duration_ms: \K[^,]+' | sed 's/^/   Duration: /' | sed 's/$/ ms/'
        echo ""
    elif [[ $line == *"🐌"* ]]; then
        echo "🐌 SLOW REQUEST"
        echo "$line" | grep -oP 'status: \K[^,]+' | sed 's/^/   Status: /'
        echo "$line" | grep -oP 'duration_ms: \K[^,]+' | sed 's/^/   Duration: /' | sed 's/$/ ms/'
        echo ""
    fi
done

echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📝 Full logs available at: /tmp/demo-server.log"
echo ""
echo "🛑 Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo "✅ Demo complete!"
echo ""
echo "💡 To see raw logs:"
echo "   cat /tmp/demo-server.log | grep -E '(📥|✅|⚠️|❌|🐌)'"
