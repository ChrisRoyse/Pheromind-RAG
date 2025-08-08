#!/bin/bash
set -e

echo "🚀 Building Embed-Search MCP Server"

# Step 1: Build Rust core
echo "Building Rust core..."
cargo build --release

# Step 2: Build native bridge
echo "Building native bridge..."
if [ -d "mcp-bridge" ]; then
    cd mcp-bridge
    cargo build --release
    cd ..
else
    echo "⚠️ mcp-bridge directory not found, skipping..."
fi

# Step 3: Setup Node project
echo "Setting up Node.js project..."
if [ -d "mcp-server" ]; then
    cd mcp-server
    npm install
    
    # Step 4: Build TypeScript
    echo "Building TypeScript..."
    npm run build
    
    cd ..
else
    echo "⚠️ mcp-server directory not found, skipping..."
fi

# Step 5: Run tests
echo "Running tests..."
cargo test watcher

echo "✅ Build complete!"
echo "To register with Claude: claude mcp add embed-search ./mcp-server/dist/index.js"