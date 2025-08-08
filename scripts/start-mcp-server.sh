#!/bin/bash
# Startup script for MCP server with proper memory configuration
# This prevents V8 heap allocation errors when loading embedding models

echo "Starting MCP server with optimized memory settings..."

# Set Node.js memory limit to 4GB (adjust as needed)
export NODE_OPTIONS="--max-old-space-size=4096"

# Start the MCP server
npx claude-flow@alpha mcp start

echo "MCP server stopped."