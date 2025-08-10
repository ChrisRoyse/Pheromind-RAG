# PHASE 1: CORE MVP - 10-MINUTE TASK BREAKDOWN
## From Broken Codebase to Working MVP

**Timeline**: 1-2 weeks (30 tasks × 10 minutes = 5 hours focused work)  
**Goal**: Transform broken codebase into functional MVP with MCP integration  
**Success Criteria**: Compiles cleanly, runs MCP server, basic functionality operational  

---

## WEEK 1: FOUNDATION FIXES (Tasks 1-15)

### Task 1: Fix Missing Dependencies in Cargo.toml
**Time**: 10 minutes  
**Prerequisites**: None  
**Action**: Add missing `tempfile` and `toml` dependencies to Cargo.toml  
**Acceptance Criteria**: `cargo check` runs without "unresolved crate" errors  
**Files Modified**: `C:\code\embed\Cargo.toml`  
**Commands**: 
```toml
# Add to [dependencies] section:
tempfile = "3.8"
toml = "0.8"
```
```bash
cargo check --message-format=short
```

### Task 2: Add Missing Feature Flags to Cargo.toml  
**Time**: 10 minutes  
**Prerequisites**: Task 1  
**Action**: Add `vectordb`, `tree-sitter`, and `ml` features to Cargo.toml  
**Acceptance Criteria**: No more "unexpected cfg condition" warnings  
**Files Modified**: `C:\code\embed\Cargo.toml`  
**Commands**: 
```toml
[features]
default = []
vectordb = []
tree-sitter = []  
ml = []
```

### Task 3: Create Minimal config.toml for Testing
**Time**: 10 minutes  
**Prerequisites**: Task 2  
**Action**: Create a minimal config.toml in project root to fix Config::init() dependency  
**Acceptance Criteria**: Config::init() no longer fails with "No configuration file found"  
**Files Modified**: `C:\code\embed\config.toml`  
**Commands**: Create config file with minimal required fields

### Task 4: Fix search_adapter Module Import Errors
**Time**: 10 minutes  
**Prerequisites**: Task 3  
**Action**: Create missing search_adapter.rs module or fix import paths  
**Acceptance Criteria**: No more "could not find search_adapter in search" errors  
**Files Modified**: `C:\code\embed\src\search\search_adapter.rs` or relevant imports  
**Commands**: `cargo check src/bin/test_project_scoping.rs`

### Task 5: Fix TantivySearcher Import Errors  
**Time**: 10 minutes  
**Prerequisites**: Task 4  
**Action**: Replace TantivySearcher references with existing NativeSearcher  
**Acceptance Criteria**: `tantivy_migrator.rs` compiles without import errors  
**Files Modified**: `C:\code\embed\src\bin\tantivy_migrator.rs`  
**Commands**: `cargo check src/bin/tantivy_migrator.rs`

### Task 6: Validate MinimalEmbedder Compilation
**Time**: 10 minutes  
**Prerequisites**: Task 5  
**Action**: Ensure 44-line MinimalEmbedder compiles and runs  
**Acceptance Criteria**: MinimalEmbedder generates 768-dimensional embeddings  
**Files Modified**: None (already working)  
**Commands**: 
```bash
cargo test minimal_embedder
cargo run -- --help
```

### Task 7: Test Basic CLI Functionality
**Time**: 10 minutes  
**Prerequisites**: Task 6  
**Action**: Verify basic CLI commands work (config, validate-config)  
**Acceptance Criteria**: CLI shows help and validates config without errors  
**Files Modified**: None  
**Commands**:
```bash
cargo run -- --help
cargo run -- config
cargo run -- validate-config
```

### Task 8: Fix Conditional Compilation Guards
**Time**: 10 minutes  
**Prerequisites**: Task 7  
**Action**: Remove or fix #[cfg] guards that reference non-existent features  
**Acceptance Criteria**: No more "unexpected cfg condition" warnings in core modules  
**Files Modified**: `src\storage\simple_vectordb.rs`, `src\main.rs`  
**Commands**: `cargo check --all-targets`

### Task 9: Create Test Directory Structure
**Time**: 10 minutes  
**Prerequisites**: Task 8  
**Action**: Create proper test directory structure for integration tests  
**Acceptance Criteria**: Tests directory is organized and accessible  
**Files Modified**: `tests/integration/`, `tests/unit/`  
**Commands**: 
```bash
mkdir -p tests/integration tests/unit
mv tests/*.rs tests/integration/
```

### Task 10: Fix Binary Target Compilation Errors  
**Time**: 10 minutes  
**Prerequisites**: Task 9  
**Action**: Comment out or fix broken binary targets that prevent compilation  
**Acceptance Criteria**: `cargo build` completes without binary target errors  
**Files Modified**: `C:\code\embed\Cargo.toml` or problematic bin files  
**Commands**: `cargo build --bins`

### Task 11: Test Index Command Basic Functionality
**Time**: 10 minutes  
**Prerequisites**: Task 10  
**Action**: Test indexing a single file with minimal embedder  
**Acceptance Criteria**: Can index a .rs file without crashes  
**Files Modified**: None  
**Commands**: 
```bash  
cargo run -- index src/main.rs
cargo run -- stats
```

### Task 12: Test Search Command Basic Functionality  
**Time**: 10 minutes  
**Prerequisites**: Task 11  
**Action**: Test searching for a simple query  
**Acceptance Criteria**: Search returns results without crashing  
**Files Modified**: None  
**Commands**:
```bash
cargo run -- search "main"
cargo run -- search "config" 
```

### Task 13: Validate Core Dependencies Work
**Time**: 10 minutes  
**Prerequisites**: Task 12  
**Action**: Test that serde, tokio, clap work correctly  
**Acceptance Criteria**: All core async operations function properly  
**Files Modified**: None  
**Commands**: 
```bash
cargo test --lib config
cargo test --lib search
```

### Task 14: Fix Memory Safety Issues
**Time**: 10 minutes  
**Prerequisites**: Task 13  
**Action**: Run basic memory safety checks and fix obvious issues  
**Acceptance Criteria**: No memory leaks in basic operations  
**Files Modified**: As needed  
**Commands**: Monitor memory during indexing

### Task 15: Create Minimal Integration Test
**Time**: 10 minutes  
**Prerequisites**: Task 14  
**Action**: Create one integration test that indexes and searches  
**Acceptance Criteria**: Integration test passes end-to-end  
**Files Modified**: `tests/integration/basic_functionality.rs`  
**Commands**: `cargo test --test basic_functionality`

---

## WEEK 2: MCP INTEGRATION (Tasks 16-30)

### Task 16: Fix MCP Server Basic Compilation  
**Time**: 10 minutes  
**Prerequisites**: Task 15  
**Action**: Fix MCP server compilation errors  
**Acceptance Criteria**: `mcp_server.rs` compiles without errors  
**Files Modified**: `src/bin/mcp_server.rs`, imports  
**Commands**: `cargo check --bin mcp_server`

### Task 17: Test MCP Protocol Basic Structure
**Time**: 10 minutes  
**Prerequisites**: Task 16  
**Action**: Verify MCP protocol types and structures compile  
**Acceptance Criteria**: MCP types serialize/deserialize properly  
**Files Modified**: `src/mcp/protocol.rs`, `src/mcp/types.rs`  
**Commands**: `cargo test mcp::protocol`

### Task 18: Fix MCP Transport Layer
**Time**: 10 minutes  
**Prerequisites**: Task 17  
**Action**: Ensure stdio transport compiles and handles basic I/O  
**Acceptance Criteria**: MCP server can read/write JSON messages  
**Files Modified**: `src/mcp/transport/stdio.rs`  
**Commands**: `cargo test mcp::transport`

### Task 19: Implement Basic MCP Tool - Status
**Time**: 10 minutes  
**Prerequisites**: Task 18  
**Action**: Get status tool working (simplest MCP tool)  
**Acceptance Criteria**: Status tool returns system information via MCP  
**Files Modified**: `src/mcp/tools/status.rs`  
**Commands**: Test MCP status command

### Task 20: Implement Basic MCP Tool - Clear
**Time**: 10 minutes  
**Prerequisites**: Task 19  
**Action**: Get clear tool working (clears index)  
**Acceptance Criteria**: Clear tool works via MCP protocol  
**Files Modified**: `src/mcp/tools/clear.rs`  
**Commands**: Test MCP clear command

### Task 21: Implement Basic MCP Tool - Index
**Time**: 10 minutes  
**Prerequisites**: Task 20  
**Action**: Get index tool working via MCP  
**Acceptance Criteria**: Can index files through MCP interface  
**Files Modified**: `src/mcp/tools/index.rs`  
**Commands**: Test MCP index command

### Task 22: Implement Basic MCP Tool - Search  
**Time**: 10 minutes  
**Prerequisites**: Task 21  
**Action**: Get search tool working via MCP  
**Acceptance Criteria**: Can search through MCP interface  
**Files Modified**: `src/mcp/tools/search.rs`  
**Commands**: Test MCP search command

### Task 23: Test MCP Server Standalone Operation
**Time**: 10 minutes  
**Prerequisites**: Task 22  
**Action**: Run MCP server in standalone mode and test basic tools  
**Acceptance Criteria**: MCP server responds to tool calls correctly  
**Files Modified**: None  
**Commands**: 
```bash
cargo run --bin mcp_server
# Test with manual JSON messages
```

### Task 24: Create MCP Integration Test
**Time**: 10 minutes  
**Prerequisites**: Task 23  
**Action**: Create integration test for MCP server functionality  
**Acceptance Criteria**: MCP integration test passes  
**Files Modified**: `tests/integration/mcp_integration.rs`  
**Commands**: `cargo test --test mcp_integration`

### Task 25: Fix MCP Configuration Loading
**Time**: 10 minutes  
**Prerequisites**: Task 24  
**Action**: Ensure MCP server loads configuration properly  
**Acceptance Criteria**: MCP server uses same config as CLI  
**Files Modified**: `src/mcp/server.rs`, config loading  
**Commands**: Test MCP server with custom config

### Task 26: Test MCP Error Handling
**Time**: 10 minutes  
**Prerequisites**: Task 25  
**Action**: Verify MCP server handles errors gracefully  
**Acceptance Criteria**: MCP server returns proper error responses  
**Files Modified**: Error handling in MCP tools  
**Commands**: Test invalid MCP requests

### Task 27: Validate MCP Server Performance  
**Time**: 10 minutes  
**Prerequisites**: Task 26  
**Action**: Test MCP server performance with larger operations  
**Acceptance Criteria**: MCP server handles reasonable loads without crashing  
**Files Modified**: None  
**Commands**: Test indexing vectortest/ via MCP

### Task 28: Create MCP Documentation
**Time**: 10 minutes  
**Prerequisites**: Task 27  
**Action**: Document MCP server setup and usage  
**Acceptance Criteria**: Clear instructions for using MCP server  
**Files Modified**: `docs/MCP_USAGE.md`  
**Commands**: Validate documentation against actual usage

### Task 29: Test End-to-End MCP Workflow
**Time**: 10 minutes  
**Prerequisites**: Task 28  
**Action**: Complete workflow: start MCP server → index → search → get results  
**Acceptance Criteria**: Full MCP workflow works without manual intervention  
**Files Modified**: None  
**Commands**: Full end-to-end MCP test

### Task 30: Performance Validation and Cleanup
**Time**: 10 minutes  
**Prerequisites**: Task 29  
**Action**: Run final performance tests and clean up any remaining issues  
**Acceptance Criteria**: System performs reliably under normal usage patterns  
**Files Modified**: Performance optimizations as needed  
**Commands**: 
```bash
cargo test --release
cargo run --release -- test
```

---

## SUCCESS CRITERIA VALIDATION

**✅ Compilation Success**: `cargo build --release` succeeds  
**✅ Core CLI Operational**: Index, search, stats, clear commands functional  
**✅ MCP Server Running**: Responds to basic MCP tool calls  
**✅ Integration Tests Pass**: Basic functionality verified  
**✅ MinimalEmbedder Active**: 768-dimensional vectors generated  
**✅ Configuration Loading**: Loads config from TOML files  
**✅ Memory Safe Operation**: No crashes during normal operations

---

## CRITICAL PATH ANALYSIS

**Must Complete First Week**: Tasks 1→2→3→6→7 (foundation)  
**Parallel Opportunities**: Tasks 9-15 can run concurrently after Task 8  
**High Risk Items**: 
- Task 3 (Config::init dependency) - **CRITICAL** 
- Tasks 16-23 (MCP integration) - **HIGH COMPLEXITY**

**Estimated Total Time**: 5 hours focused work (30 × 10 minutes)  
**Real-world Timeline**: 1-2 weeks with testing and validation