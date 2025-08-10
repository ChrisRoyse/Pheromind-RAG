# MCP IMPLEMENTATION PLAN: Working Server in Claude Code

**Target:** Functional MCP server integrated with Claude Code  
**Timeline:** 2.5-3.5 hours total implementation time  
**Current Status:** Infrastructure 90% complete, configuration fixes needed  

## PHASE 1: CORE MVP (1-2 hours)
*Goal: Working MCP server that responds to basic requests*

### Step 1.1: Fix Configuration System (30 minutes)
**Problem:** Config::init() dependency causing startup failure

**Solution Options:**
```rust
// OPTION A: Quick Fix (Recommended)
// In src/bin/mcp_server.rs, modify main() function:
std::env::set_var("EMBED_LOG_LEVEL", "info");
if let Err(e) = Config::init() {
    // Use MCP config directly if base config fails
    log::warn!("Base config failed: {}, proceeding with MCP-only config", e);
}

// OPTION B: Bypass Base Config  
// Modify McpServer::with_project_path() to not require Config::init()
```

**Files to modify:**
- `src/bin/mcp_server.rs` (line 71-73)
- `src/mcp/server.rs` (lines 63-77) 
- `src/mcp/config.rs` (add fallback configuration)

### Step 1.2: Create Working Configuration (15 minutes)
**Location:** `C:\code\embed\.embed\mcp-config.toml`

**Content:** (Use provided minimal config from research)

### Step 1.3: Basic Functionality Test (30 minutes)
```bash
# Test sequence:
1. cargo build --bin mcp_server
2. echo '{"jsonrpc":"2.0","method":"ping","id":1}' | target/debug/mcp_server.exe .
3. echo '{"jsonrpc":"2.0","method":"capabilities","id":2}' | target/debug/mcp_server.exe .
4. echo '{"jsonrpc":"2.0","method":"search","params":{"query":"test"},"id":3}' | target/debug/mcp_server.exe .
```

**Expected Results:**
- Ping: {"jsonrpc":"2.0","result":{"status":"ok",...},"id":1}
- Capabilities: Server features list
- Search: Search results or empty results array

### Step 1.4: Clean Up Warnings (15 minutes)
**Problem:** 100+ feature flag warnings make debugging difficult

**Solution:**
```toml
# Add to Cargo.toml [features] section:
[features]
default = []
vectordb = []
tantivy = []
tree-sitter = []
ml = []
```

**Acceptance Criteria:**
✅ MCP server starts without errors  
✅ Responds to ping requests  
✅ Can perform basic search operations  
✅ Clean build (no warnings)  

---

## PHASE 2: CLAUDE CODE INTEGRATION (1-2 hours)
*Goal: MCP server working within Claude Code interface*

### Step 2.1: Research Claude Code MCP Setup (30 minutes)
**Investigate:**
- Claude Code MCP server configuration format
- Required server capabilities/methods
- Transport protocol specifics
- Authentication/security requirements

### Step 2.2: Configure Claude Code Integration (30 minutes)
**Typical MCP Server Configuration:**
```json
{
  "mcpServers": {
    "embed-search": {
      "command": "C:\\path\\to\\target\\debug\\mcp_server.exe",
      "args": ["."],
      "env": {
        "EMBED_LOG_LEVEL": "info"
      }
    }
  }
}
```

### Step 2.3: Integration Testing (45 minutes)
**Test Sequence:**
1. Add server to Claude Code configuration
2. Restart Claude Code
3. Verify server appears in tools list  
4. Test search functionality from Claude Code
5. Validate request/response format compatibility

### Step 2.4: Debug and Fix Issues (15 minutes)
**Common Issues:**
- Path resolution problems
- JSON format incompatibilities
- Timeout configurations
- Error response handling

**Acceptance Criteria:**
✅ Claude Code recognizes MCP server  
✅ Search tool appears in Claude Code interface  
✅ Search operations return valid results  
✅ Error handling works correctly  

---

## PHASE 3: ENHANCEMENT & HARDENING (2-4 hours)
*Goal: Production-ready MCP server with advanced features*

### Step 3.1: Restore Optional Features (2 hours)
**Add back conditionally:**
- Tantivy full-text search (if needed)
- Tree-sitter symbol extraction (if needed)  
- Vector database support (if needed)
- ML embeddings (if requested)

### Step 3.2: Performance Optimization (1 hour)
**Enhancements:**
- Request batching and queuing
- Search result caching
- Parallel search execution verification
- Memory usage optimization

### Step 3.3: Error Resilience (1 hour)
**Improvements:**
- Graceful degradation on errors
- Recovery from corrupted index
- Better error messages
- Health check monitoring

**Acceptance Criteria:**
✅ Advanced search features available  
✅ Handles high request volume  
✅ Recovers from error conditions  
✅ Production deployment ready  

---

## CRITICAL SUCCESS FACTORS

### 1. Configuration Management
**The #1 blocker is configuration initialization.** The MCP server has sophisticated config requirements that must be satisfied before startup.

**Mitigation:**
- Provide tested configuration templates
- Implement graceful fallbacks
- Clear error messages with solutions

### 2. Protocol Compatibility  
**Claude Code likely expects specific MCP behavior.** The current implementation follows JSON-RPC 2.0 but may need Claude-specific adaptations.

**Mitigation:**  
- Test with actual Claude Code instance
- Document any required modifications
- Maintain JSON-RPC 2.0 compliance

### 3. Performance Under Load
**Search operations may be slow without proper optimization.** BM25 + hash embeddings should be fast, but file I/O could be a bottleneck.

**Mitigation:**
- Profile search operations
- Implement caching strategies  
- Add request timeout handling

---

## IMMEDIATE NEXT STEPS

### Day 1 (Today - 30 minutes)
1. **Fix Config::init() dependency** in mcp_server.rs main()
2. **Create .embed/mcp-config.toml** with minimal settings
3. **Test basic ping operation** to verify server starts

### Day 1 (Today - 60 minutes)  
4. **Test search functionality** with sample queries
5. **Clean up feature flag warnings** 
6. **Document working server setup**

### Day 2 (Tomorrow - 90 minutes)
7. **Research Claude Code MCP integration**
8. **Configure Claude Code to use MCP server**
9. **Test end-to-end functionality**

---

## SUCCESS METRICS

### Minimum Viable Success
- [ ] MCP server starts without errors
- [ ] Responds to JSON-RPC requests  
- [ ] Performs basic search operations
- [ ] Works with Claude Code (even if limited)

### Complete Success  
- [ ] All MCP tools available in Claude Code
- [ ] Fast, accurate search results
- [ ] Handles production workload
- [ ] Easy deployment and configuration

### Stretch Success
- [ ] Advanced search features (ML, vector DB)
- [ ] Real-time file watching and indexing
- [ ] Performance metrics and monitoring
- [ ] Multi-repository support

---

**BOTTOM LINE:** This is a "last mile" problem, not an architectural rebuild. The hardest parts (JSON-RPC protocol, stdio transport, search engine) are already implemented and tested. Focus on configuration and integration, not reimplementation.