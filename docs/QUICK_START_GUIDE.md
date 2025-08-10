# QUICK START: Working MCP Server in 30 Minutes

**Goal:** Get a functional MCP server running for Claude Code integration  
**Time Required:** 30 minutes maximum  
**Skill Level:** Intermediate developer  

## THE PROBLEM
The MCP server is 90% complete but fails at startup due to configuration dependency:
```
Failed to create MCP server: Configuration error: Configuration not initialized. Call Config::init() first.
```

## THE SOLUTION (3 Steps)

### Step 1: Fix Config Dependency (5 minutes)
**Edit:** `src/bin/mcp_server.rs` 

**Find line ~71:**
```rust
if let Err(e) = Config::init() {
    log::warn!("Config initialization failed, using defaults: {}", e);
}
```

**Replace with:**
```rust
std::env::set_var("EMBED_LOG_LEVEL", "info");
if let Err(e) = Config::init() {
    log::warn!("Config initialization failed, using defaults: {}", e);
    // Continue anyway - MCP has its own config system
}
```

### Step 2: Create MCP Config (10 minutes)
**Create:** `C:\code\embed\.embed\mcp-config.toml`

```toml
server_name = "embed-search-mcp"
server_version = "0.1.0" 
server_description = "Functional MCP server"

[transport]
type = "Stdio"
buffer_size = 8192
line_buffering = true

[tools]
enable_search = true
enable_index = true
enable_status = true
enable_clear = true
enable_orchestrated_search = false
max_results_per_call = 50
default_search_timeout_ms = 10000
max_concurrent_operations = 5

[performance]
max_concurrent_requests = 25
request_timeout_ms = 30000
max_request_size_bytes = 524288
max_response_size_bytes = 2097152
enable_metrics = false
metrics_interval_secs = 300

[security]
enable_request_validation = true
max_query_length = 500
allowed_file_extensions = ["rs", "py", "js", "ts", "json", "md", "txt"]
blocked_file_patterns = ["\.git/.*", ".*\.log$"]
enable_path_protection = true
max_indexing_depth = 10

mcp_log_level = "info"
enable_request_logging = false
enable_performance_logging = false
```

### Step 3: Test & Verify (15 minutes)
**Build:**
```bash
cargo build --bin mcp_server
```

**Test Ping:**
```bash
echo '{"jsonrpc":"2.0","method":"ping","id":1}' | target/debug/mcp_server.exe .
```
**Expected:** `{"jsonrpc":"2.0","result":{"status":"ok","timestamp":...},"id":1}`

**Test Search:**
```bash  
echo '{"jsonrpc":"2.0","method":"search","params":{"query":"test","max_results":5},"id":2}' | target/debug/mcp_server.exe .
```
**Expected:** Search results JSON response

**Test Capabilities:**
```bash
echo '{"jsonrpc":"2.0","method":"capabilities","id":3}' | target/debug/mcp_server.exe .
```
**Expected:** Server capabilities JSON response

## CLAUDE CODE INTEGRATION

### Add to Claude Code MCP Config
```json
{
  "mcpServers": {
    "embed-search": {
      "command": "C:\\code\\embed\\target\\debug\\mcp_server.exe",
      "args": ["."],
      "env": {
        "EMBED_LOG_LEVEL": "info"
      }
    }
  }
}
```

### Restart Claude Code
After adding the server configuration, restart Claude Code to load the MCP server.

## TROUBLESHOOTING

### Server Won't Start
- Check that `.embed/mcp-config.toml` exists
- Verify file paths are correct
- Check that Config::init() change was made

### No Response from Server  
- Test with simple ping first
- Check stderr output for error messages
- Verify JSON format is correct (no trailing commas)

### Claude Code Can't Connect
- Check server executable path
- Verify server responds to stdio input
- Look at Claude Code MCP server logs

## WHAT YOU GET

✅ **Working MCP Server** - Responds to JSON-RPC requests  
✅ **Search Functionality** - BM25 + hash-based embedding search  
✅ **File Indexing** - Index project files for search  
✅ **Status Monitoring** - Server health and metrics  
✅ **Claude Code Compatible** - Follows MCP protocol specification  

## NEXT STEPS

After basic functionality works:
1. **Add Optional Features** - ML embeddings, vector database, etc.
2. **Performance Tuning** - Caching, parallel processing optimization
3. **Advanced Tools** - File watching, real-time indexing  
4. **Production Hardening** - Error handling, monitoring, security

The foundation is solid - you're just completing the last configuration mile!