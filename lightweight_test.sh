#!/bin/bash
# Lightweight test script that bypasses heavy compilation

echo "🚀 Running lightweight accuracy test..."
echo "======================================"

# Build only the necessary binary in release mode
echo "📦 Building test binary..."
cargo build --bin test_accuracy --release 2>/dev/null

if [ $? -ne 0 ]; then
    echo "❌ Build failed. Running with cargo run instead..."
    cargo run --bin test_accuracy --release
else
    echo "✅ Build complete. Running test..."
    ./target/release/test_accuracy
fi