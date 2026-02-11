#!/bin/bash
# Run the Aframp backend server with proper logging

echo "🔨 Building the server..."
cargo build --quiet

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Starting server..."
    echo ""
    ./target/debug/Aframp-Backend
else
    echo "❌ Build failed!"
    exit 1
fi
