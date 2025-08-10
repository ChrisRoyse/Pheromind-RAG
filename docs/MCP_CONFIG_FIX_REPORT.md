# MCP Configuration Fix Report

## Problem Identified

The embed-search MCP server was not showing up in Claude Code's `/mcp` list despite being added to the configuration file.

## Root Cause Analysis

### Configuration Structure Issue

The initial configuration script incorrectly added `embed-search` OUTSIDE the `mcpServers` object:

```json
{
  "mcpServers": {
    "ruv-swarm": { ... },
    "playwright": { ... }
  },
  "embed-search": {  // ❌ WRONG: Outside mcpServers
    "type": "stdio",
    "command": "...",
    "args": []
  }
}
```

### Correct Structure

The configuration should be INSIDE the `mcpServers` object:

```json
{
  "mcpServers": {
    "ruv-swarm": { ... },
    "playwright": { ... },
    "embed-search": {  // ✅ CORRECT: Inside mcpServers
      "type": "stdio",
      "command": "...",
      "args": []
    }
  }
}
```

## Claude Code Configuration Hierarchy

Claude Code uses a complex configuration structure with multiple levels:

### 1. Global MCP Servers (Root Level)
Located at the root of `~/.claude.json`:
- Example: `neo4j-cypher`, `neo4j-memory` (lines 25-48)
- These are legacy/special servers

### 2. Global mcpServers Object
Located at `~/.claude.json` → `mcpServers` (line 1448):
- This is where globally available MCP servers should be placed
- Examples: `ruv-swarm`, `playwright`, and now `embed-search`

### 3. Project-Specific mcpServers
Located at `~/.claude.json` → `projects` → `{project-path}` → `mcpServers`:
- These are MCP servers specific to individual projects
- Multiple projects can have their own MCP server configurations

## The Fix

### Step 1: Removed Incorrect Configuration
Removed `embed-search` from the root level (line 1471 originally)

### Step 2: Added to Correct Location
Added `embed-search` inside the global `mcpServers` object (now at line 1470)

### Step 3: Verified Structure
Ensured proper JSON structure with all braces correctly placed

## Final Configuration

The embed-search MCP server is now correctly configured at:
```
~/.claude.json → mcpServers → embed-search
```

With the following configuration:
```json
"embed-search": {
  "type": "stdio",
  "command": "C:\\\\code\\\\embed\\\\target\\\\release\\\\embed-search-mcp.exe",
  "args": []
}
```

## Verification Steps

1. **Check JSON validity:**
   ```bash
   python -m json.tool < ~/.claude.json
   ```
   ✅ No errors

2. **Verify placement:**
   ```bash
   grep -n "embed-search" ~/.claude.json
   ```
   ✅ Shows lines 1470 and 1472 (inside mcpServers)

3. **Test MCP server:**
   ```bash
   echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | C:/code/embed/target/release/embed-search-mcp.exe
   ```
   ✅ Returns 5 tools

## Action Required

**RESTART CLAUDE CODE** - The configuration changes will only take effect after restarting Claude Code.

After restart, verify with:
- `/mcp` - Should show embed-search as connected
- `/mcp tools` - Should list the 5 embed-search tools

## Lessons Learned

1. **Configuration Structure Matters**: Claude Code expects MCP servers in the `mcpServers` object, not at root level
2. **Multiple Configuration Levels**: Be aware of global vs project-specific configurations
3. **JSON Structure Validation**: Always validate JSON structure after modifications
4. **Test Before Deployment**: Test MCP servers manually before adding to configuration

## Scripts Created

1. `scripts/configure_claude_code.py` - Initial (incorrect) configuration script
2. `scripts/fix_mcp_config.py` - Corrected configuration script that properly places MCP servers

## Backup Files

- `~/.claude.json.backup` - First backup (from incorrect script)
- `~/.claude.json.backup2` - Second backup (before fix)

Both backups are available if rollback is needed.