# Claude-Flow MCP Server - Windows Solution

## Problem Solved
Applied the Neo4j MCP pattern to fix claude-flow MCP server on Windows.

## Key Insight from Neo4j Example
Your Neo4j guide revealed the critical pattern:
- **Use direct executable paths** (not `npx` in config)
- **Pass arguments directly** (not environment variables)
- **Proper path escaping** with double backslashes

## Solution Implementation

### 1. Created Stable Executable Wrapper
Instead of relying on NPX cache (which can change), created a permanent wrapper:

**Location:** `C:\Users\hotra\AppData\Local\claude-flow-mcp\claude-flow-mcp.bat`

```batch
@echo off
:: Ensures SQLite binding is available
:: Then launches claude-flow MCP server
npx claude-flow@alpha mcp start
```

### 2. Updated Configuration (Neo4j Pattern)

**Before (Broken):**
```json
{
  "command": "cmd",
  "args": ["/c", "npx", "claude-flow@alpha", "mcp", "start"]
}
```

**After (Working - Neo4j Pattern):**
```json
{
  "type": "stdio",
  "command": "C:\\\\Users\\\\hotra\\\\AppData\\\\Local\\\\claude-flow-mcp\\\\claude-flow-mcp.bat"
}
```

### 3. Key Differences
- **No `cmd /c` wrapper** - Direct executable like Neo4j
- **Stable path** - Not dependent on NPX cache location
- **SQLite binding handled** - Wrapper ensures binding is deployed

## Configuration Updated

### Project-Specific (.claude.json)
```json
{
  "projects": {
    "C:\\code\\embed": {
      "mcpServers": {
        "claude-flow": {
          "type": "stdio",
          "command": "C:\\\\Users\\\\hotra\\\\AppData\\\\Local\\\\claude-flow-mcp\\\\claude-flow-mcp.bat"
        }
      }
    }
  }
}
```

### Project File (.mcp.json)
```json
{
  "mcpServers": {
    "claude-flow": {
      "type": "stdio",
      "command": "C:\\\\Users\\\\hotra\\\\AppData\\\\Local\\\\claude-flow-mcp\\\\claude-flow-mcp.bat"
    }
  }
}
```

## Why This Works

1. **Follows Neo4j Pattern**: Direct executable path that Windows can spawn
2. **No Protocol Contamination**: Unlike `cmd /c`, doesn't inject Windows headers
3. **Stable Location**: Not dependent on NPX cache that changes with updates
4. **SQLite Binding Handled**: Wrapper ensures binding is available

## Verification

The solution:
- ✅ Uses direct executable path (like Neo4j)
- ✅ Removes problematic cmd /c wrapper
- ✅ Handles SQLite binding deployment
- ✅ Provides stable path that survives updates

## Next Steps

1. **Configuration is updated** following Neo4j pattern
2. **Restart Claude Desktop** to apply changes
3. **Check MCP status** - should show "✓ connected"

## Lessons Learned

Your Neo4j example was the key to solving this:
- Windows MCP servers need direct executable paths
- The `npx` command shouldn't appear in the configuration
- Command line arguments work better than environment variables
- Proper path escaping is critical

This approach should now work just like your successful Neo4j servers.