@echo off
REM Startup script for MCP server with proper memory configuration
REM This prevents V8 heap allocation errors when loading embedding models

echo Starting MCP server with optimized memory settings...

REM Set Node.js memory limit to 4GB (adjust as needed)
set NODE_OPTIONS=--max-old-space-size=4096

REM Start the MCP server
npx claude-flow@alpha mcp start

echo MCP server stopped.