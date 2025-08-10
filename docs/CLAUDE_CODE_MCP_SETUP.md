# Claude Code MCP Server Setup Guide

## Overview

This guide documents the correct process for setting up the embed-search MCP server with **Claude Code** (not Claude Desktop) on Windows.

## Key Differences: Claude Code vs Claude Desktop

| Aspect | Claude Code | Claude Desktop |
|--------|-------------|----------------|
| Config Location | `~/.claude.json` | `%APPDATA%\Claude\claude_desktop_config.json` |
| Config Structure | MCP servers at root level | MCP servers in `mcpServers` object |
| Usage | Command-line tool | GUI application |

## Prerequisites

1. **Built MCP Server Binary**
   ```bash
   cargo build --release --bin embed-search-mcp
   ```
   Location: `C:\code\embed\target\release\embed-search-mcp.exe`

2. **Claude Code Installed**
   - Verify installation: `claude --version`
   - Config file exists: `~/.claude.json`

## Configuration Steps

### Automatic Setup (Recommended)

Run the provided configuration script:

```bash
python scripts/configure_claude_code.py
```

This script will:
1. Locate your Claude Code configuration
2. Add the embed-search MCP server
3. Create a backup of your original config
4. Verify the configuration

### Manual Setup

1. **Open Claude Code Configuration**
   ```bash
   notepad C:\Users\%USERNAME%\.claude.json
   ```

2. **Add MCP Server Configuration**
   
   Add this entry at the root level of the JSON (NOT in an mcpServers object):
   
   ```json
   {
     "embed-search": {
       "type": "stdio",
       "command": "C:\\\\code\\\\embed\\\\target\\\\release\\\\embed-search-mcp.exe",
       "args": []
     },
     // ... other existing configurations
   }
   ```

   **Important Notes:**
   - Use quadruple backslashes (`\\\\`) for Windows paths in JSON
   - Place at root level, same as other servers like `neo4j-cypher`
   - No `mcpServers` wrapper object needed

## Verification

### 1. Test MCP Server Manually

```bash
echo {"jsonrpc":"2.0","method":"tools/list","id":1} | C:\code\embed\target\release\embed-search-mcp.exe
```

Expected output: JSON response with 5 tools listed

### 2. Restart Claude Code

**IMPORTANT**: You must restart Claude Code for configuration changes to take effect.

### 3. Check MCP Connection in Claude Code

After restart, in any Claude Code conversation:

```
/mcp
```

You should see:
```
embed-search
Status: √ connected
Tools: 5 tools
```

### 4. List Available Tools

```
/mcp tools
```

Should show:
- `embed_search` - Hybrid code search
- `embed_index` - Index files
- `embed_extract_symbols` - Extract AST symbols
- `embed_status` - System status
- `embed_clear` - Clear indexed data

## Available MCP Tools

### 1. embed_search
Search through indexed code using hybrid semantic and text search.

**Parameters:**
- `query` (required): Search query text
- `search_type`: "hybrid" | "semantic" | "text" | "symbol" (default: "hybrid")
- `limit`: Max results 1-50 (default: 10)

### 2. embed_index
Index files in a directory for searching.

**Parameters:**
- `path` (required): Directory path to index
- `file_extensions`: Array of extensions (default: ["rs", "py", "js", "ts"])
- `max_file_size`: Max file size in bytes (default: 100000)

### 3. embed_extract_symbols
Extract code symbols from source code.

**Parameters:**
- `code` (required): Source code to analyze
- `file_extension` (required): "rs" | "py" | "js" | "ts"

### 4. embed_status
Get system status and health information.

### 5. embed_clear
Clear all indexed data.

**Parameters:**
- `confirm`: Boolean confirmation (default: false)

## Usage Examples

In Claude Code conversations, you can now use the MCP tools:

```
Use the embed_index tool to index all Rust files in C:/myproject

Use the embed_search tool to find all functions that handle authentication

Use the embed_extract_symbols tool to analyze the main.rs file
```

## Troubleshooting

### MCP Server Not Connecting

1. **Check executable path exists:**
   ```bash
   ls C:\code\embed\target\release\embed-search-mcp.exe
   ```

2. **Verify JSON syntax:**
   ```bash
   python -m json.tool < ~/.claude.json
   ```

3. **Check for duplicate entries:**
   Ensure "embed-search" doesn't appear twice in the config

4. **Review logs:**
   Check Claude Code logs for connection errors

### Common Issues

| Issue | Solution |
|-------|----------|
| "MCP error -32000: Connection closed" | Wrong executable path or missing file |
| "spawn ENOENT" | Path escaping issue - use quadruple backslashes |
| No tools showing | Restart Claude Code after config change |
| JSON parse error | Check for trailing commas or syntax errors |

## Configuration Location Reference

The correct configuration structure for Claude Code:

```json
{
  "numStartups": 62,
  "neo4j-cypher": {
    "type": "stdio",
    "command": "C:\\\\path\\\\to\\\\exe",
    "args": []
  },
  "embed-search": {
    "type": "stdio",
    "command": "C:\\\\code\\\\embed\\\\target\\\\release\\\\embed-search-mcp.exe",
    "args": []
  },
  "projects": {
    // Project-specific settings
  }
}
```

**Note**: MCP servers are configured at the root level, NOT inside an `mcpServers` object.

## Summary

✅ **Configuration Complete**

The embed-search MCP server is now:
1. Configured in Claude Code (not Claude Desktop)
2. Using the correct Windows path format
3. Placed at the root level of `~/.claude.json`
4. Ready to use after Claude Code restart

Remember to restart Claude Code and use `/mcp` to verify the connection!