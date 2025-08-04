#!/usr/bin/env python3
"""
Debug MCP server setup
"""

import asyncio
import sys
from mcp import server, types
from mcp.server.stdio import stdio_server

# Create a minimal server for testing
app = server.Server("debug-server")

@app.list_tools()
async def list_tools():
    return [
        types.Tool(
            name="test_tool",
            description="A test tool",
            inputSchema={
                "type": "object",
                "properties": {}
            }
        )
    ]

@app.call_tool()
async def call_tool(name: str, arguments: dict):
    return [types.TextContent(type="text", text=f"Called tool: {name}")]

async def main():
    try:
        print("Starting debug MCP server...", file=sys.stderr)
        async with stdio_server() as streams:
            await app.run(
                streams[0], 
                streams[1],
                server.InitializationOptions(
                    server_name="debug-server",
                    server_version="1.0.0",
                    capabilities=types.ServerCapabilities()
                )
            )
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(main())