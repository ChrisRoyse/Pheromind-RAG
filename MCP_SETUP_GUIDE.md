# MCP Server Setup Guide - IMPORTANT TRUTH

## The Reality About MCP Servers

**IMPORTANT UNDERSTANDING**: MCP servers are NOT like traditional command-line tools that are available globally on your system. They work differently:

1. **MCP servers are registered with specific AI clients** (like Claude Desktop)
2. **They are NOT available as global system commands**
3. **Each AI client maintains its own MCP server registry**
4. **The servers run as child processes of the AI client when needed**

## How MCP Actually Works

### Architecture
```
Claude Desktop
    ├── Reads: claude_desktop_config.json
    ├── Discovers registered MCP servers
    ├── Spawns server processes on-demand
    └── Communicates via JSON-RPC over STDIO
```

### Key Points
- MCP servers communicate via **STDIO** (standard input/output) using JSON-RPC
- They are **NOT REST APIs** or system services
- They are **spawned as needed** by the AI client
- They **terminate** when the client disconnects

## Your embed-search MCP Server Status

### ✅ What's Working
1. **Binary Built**: `C:\code\embed\target\release\embed-search-mcp.exe`
2. **Registered in Claude**: Added to `claude_desktop_config.json`
3. **5 Tools Available**:
   - `embed_search` - Hybrid search
   - `embed_index` - Index files
   - `embed_extract_symbols` - Extract code symbols
   - `embed_status` - System status
   - `embed_clear` - Clear indexed data

### ❌ What Doesn't Work (And Never Will)
- Running `embed-search` as a global command from any directory
- Using the MCP server outside of Claude Desktop
- Accessing the tools via REST API or web interface

## How to Use Your MCP Server

### In Claude Desktop

After restarting Claude Desktop, you should see the embed-search tools available. You can:

1. **Index your code**:
   ```
   Use the embed_index tool to index files in a directory
   ```

2. **Search your code**:
   ```
   Use the embed_search tool with queries like "calculate fibonacci"
   ```

3. **Extract symbols**:
   ```
   Use the embed_extract_symbols tool on code snippets
   ```

### Testing the MCP Server Manually

You can test the server works correctly:

```bash
# Test with initialize request
echo {"jsonrpc":"2.0","method":"initialize","id":1,"params":{}} | C:\code\embed\target\release\embed-search-mcp.exe

# Test listing tools
echo {"jsonrpc":"2.0","method":"tools/list","id":1} | C:\code\embed\target\release\embed-search-mcp.exe
```

## Making It "Global" - The Workarounds

While you can't make MCP servers truly global, here are some alternatives:

### Option 1: Create a CLI Wrapper (Recommended)

Create a separate CLI tool that doesn't use MCP:

```rust
// src/bin/embed-cli.rs
use clap::Parser;
use embed_search::HybridSearch;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    Search { query: String },
    Index { path: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    // Implement CLI logic here
}
```

Then add to PATH:
```bash
# Add to system PATH
setx PATH "%PATH%;C:\code\embed\target\release"
```

### Option 2: Create Batch Scripts

Create wrapper scripts in a directory that's in your PATH:

```batch
@echo off
C:\code\embed\target\release\embed-search.exe %*
```

### Option 3: Use as a Library

Import and use in other Rust projects:

```toml
[dependencies]
embed-search = { path = "C:/code/embed" }
```

## Configuration Location

Your MCP server is configured in:
```
C:\Users\hotra\AppData\Roaming\Claude\claude_desktop_config.json
```

Entry added:
```json
"embed-search": {
  "command": "C:\\code\\embed\\target\\release\\embed-search-mcp.exe",
  "args": [],
  "env": {
    "RUST_LOG": "info"
  }
}
```

## Troubleshooting

### MCP Server Not Showing in Claude
1. **Restart Claude Desktop** (required after config changes)
2. Check for JSON syntax errors in config
3. Verify the exe path is correct
4. Check Claude logs at: `%APPDATA%\Claude\logs`

### Testing Server Health
```powershell
# PowerShell test
'{"jsonrpc":"2.0","method":"initialize","id":1,"params":{}}' | C:\code\embed\target\release\embed-search-mcp.exe
```

### Common Issues
- **"Tool not found"**: Server not registered properly
- **"Server crashed"**: Check RUST_LOG output
- **"No response"**: Server might be writing to stdout incorrectly

## The Truth About "Global" Access

**MCP servers are fundamentally designed to be client-specific, not system-wide tools.**

This is by design because:
1. **Security**: Servers run with client permissions
2. **Isolation**: Each client manages its own servers
3. **Protocol**: STDIO communication requires parent process
4. **Lifecycle**: Servers start/stop with client sessions

## What You CAN Do

### ✅ Use in Claude Desktop
- All 5 search technologies work
- Tools are available in any Claude conversation
- Can index and search any accessible directory

### ✅ Create Traditional CLI
- Build a separate `embed-cli` binary
- Add to system PATH
- Use from any terminal

### ✅ Use as Rust Library
- Import in other projects
- Full programmatic access
- No MCP protocol overhead

### ❌ What You CANNOT Do
- Use MCP tools outside Claude
- Access MCP server as REST API
- Run MCP commands in terminal
- Share MCP server between different AI clients

## Summary

Your embed-search MCP server is **correctly configured and working** for its intended purpose - providing search capabilities to Claude Desktop. It will never be a "global" command because that's not how MCP works.

If you need global command-line access, create a separate CLI tool using the same underlying library but without the MCP protocol layer.

---

**Remember**: MCP = Model Context Protocol = AI Client Integration ≠ Global CLI Tool