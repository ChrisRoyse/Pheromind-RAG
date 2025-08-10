# PHASE 1: CORE MVP - FOUNDATION ESTABLISHMENT
## Building the Working Foundation

**Timeline**: 1-2 weeks  
**Priority**: Critical path - all subsequent phases depend on this  
**Goal**: Transform broken codebase into compilable, testable, runnable MVP  

---

## PHASE 1 OBJECTIVES

### PRIMARY GOAL: WORKING SYSTEM
- ✅ **Compiles cleanly** with zero errors and minimal warnings
- ✅ **Runs successfully** as MCP server process
- ✅ **Responds to basic MCP commands** (ping, status)
- ✅ **Provides hash-based embeddings** via 44-line MinimalEmbedder
- ✅ **Demonstrates end-to-end functionality** with simple search

### SUCCESS CRITERIA (ALL MUST BE MET)
1. `cargo build --release` completes without errors
2. MCP server starts and accepts JSON-RPC connections
3. Claude Code can connect and execute basic operations
4. MinimalEmbedder generates deterministic 768-dim vectors
5. Basic file indexing and search functionality operational
6. All core components have passing unit tests

---

## CURRENT STATE ANALYSIS

### ✅ WORKING COMPONENTS (Keep As-Is)
- **MinimalEmbedder**: 44-line hash-based embedding (perfect)
- **MCP Server Binary**: Compiles and starts (needs config fix)
- **Protocol Handler**: JSON-RPC 2.0 compliant (production ready)
- **Error Handling**: thiserror-based system (robust)
- **Bounded Cache**: LRU cache with parking_lot (performant)

### ❌ BROKEN COMPONENTS (Fix Required)
- **Configuration System**: Missing required config file
- **Dependency Management**: 9 compilation errors from missing deps
- **Feature Flags**: 249 warnings from undefined features
- **Test Suite**: Cannot run due to missing tempfile dependency

### 🔧 NEEDS CLEANUP
- **Unified Searcher**: Works but over-engineered (simplify)
- **Tool Registry**: Complete but has unused features (prune)
- **Storage System**: Safe implementation exists but incomplete

---

## IMPLEMENTATION ROADMAP

### WEEK 1: FOUNDATION FIXES

#### Day 1-2: Dependency Resolution
**Immediate Actions:**
```toml
# Add to Cargo.toml [dependencies]
tempfile = "3.0"        # Required for tests
toml = "0.8"           # Required for config parsing

# Remove or comment out all feature-gated code:
# #[cfg(feature = "vectordb")] -> comment out
# #[cfg(feature = "tantivy")] -> comment out  
# #[cfg(feature = "ml")] -> comment out
```

**Expected Outcome**: Clean compilation with zero errors

#### Day 3: Configuration Bootstrap
**Create minimal config:**
```toml
# config/minimal.toml
[transport]
type = "stdio"

[search]
backend = "bm25"
max_results = 50

[cache] 
max_size = 1000
```

**Integration Point**: `Config::init()` called at server startup

#### Day 4-5: Basic Functionality Validation
**Test Core Operations:**
1. Server startup and shutdown
2. Ping/status responses
3. MinimalEmbedder vector generation
4. Basic file indexing
5. Simple text search

### WEEK 2: MVP COMPLETION

#### Day 6-8: MCP Integration Testing
**Validate with Claude Code:**
- Configure MCP server in Claude settings
- Test basic embedding requests
- Verify search functionality
- Debug any protocol issues

#### Day 9-10: Test Suite Establishment
**Create Basic Tests:**
```rust
#[test]
fn test_minimal_embedder_deterministic() {
    let embedder = MinimalEmbedder::new(768);
    let text = "test embedding";
    let embedding1 = embedder.embed(text);
    let embedding2 = embedder.embed(text);
    assert_eq!(embedding1, embedding2);
}

#[test] 
fn test_mcp_server_startup() {
    let server = McpServer::new(config);
    assert!(server.start().is_ok());
}
```

---

## TECHNICAL IMPLEMENTATION DETAILS

### 1. DEPENDENCY CLEANUP STRATEGY

**Remove Immediately:**
```toml
# Comment out or remove these dependencies
# claude-flow = "^2.0.0-alpha.88"  
# lancedb = "*"
# tantivy = "*"
# tree-sitter = "*"
```

**Keep Essential Only:**
```toml
[dependencies]
# Core runtime
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0" 
thiserror = "1.0"
tokio = { version = "1.43", features = ["rt", "rt-multi-thread", "io-util", "io-std"] }

# Performance  
parking_lot = "0.12"
lru = "0.12"
rustc-hash = "1.1"

# File operations
walkdir = "2.4"
regex = "1.11"

# Testing
tempfile = "3.0"
```

### 2. CONFIGURATION SYSTEM DESIGN

**Minimal Config Schema:**
```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub transport: TransportConfig,
    pub search: SearchConfig,
    pub cache: CacheConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Deserialize)]
pub struct TransportConfig {
    pub type: String,  // "stdio"
    pub timeout_ms: u64,
}
```

**Default Configuration Path:**
- Primary: `config/embed-search.toml`
- Fallback: Built-in defaults for all settings
- Environment: Override via `EMBED_SEARCH_CONFIG_PATH`

### 3. MCP PROTOCOL VALIDATION

**Required JSON-RPC Methods:**
```json
// Initialization
{"jsonrpc": "2.0", "method": "initialize", "params": {...}, "id": 1}

// Core Operations  
{"jsonrpc": "2.0", "method": "search", "params": {"query": "...", "limit": 10}, "id": 2}
{"jsonrpc": "2.0", "method": "embed", "params": {"text": "..."}, "id": 3}
{"jsonrpc": "2.0", "method": "ping", "id": 4}
```

**Response Format Validation:**
```json
// Success Response
{"jsonrpc": "2.0", "result": {...}, "id": 1}

// Error Response
{"jsonrpc": "2.0", "error": {"code": -32600, "message": "..."}, "id": 1}
```

### 4. TESTING FRAMEWORK SETUP

**Unit Test Structure:**
```
tests/
├── unit/
│   ├── embedding_tests.rs      # MinimalEmbedder validation
│   ├── protocol_tests.rs       # JSON-RPC compliance  
│   ├── search_tests.rs         # BM25 and text search
│   └── config_tests.rs         # Configuration validation
├── integration/
│   ├── mcp_server_tests.rs     # Full server testing
│   └── claude_code_tests.rs    # End-to-end integration
└── common/
    └── test_helpers.rs         # Shared testing utilities
```

**Test Execution Commands:**
```bash
cargo test --lib                    # Unit tests
cargo test --test integration       # Integration tests  
cargo test -- --nocapture          # With debug output
```

---

## RISK MITIGATION PLAN

### HIGH RISK ITEMS

#### 1. Configuration Initialization Failure
**Risk**: Server won't start due to config loading errors  
**Mitigation**: Create comprehensive config validation with clear error messages  
**Fallback**: Built-in default configuration that always works

#### 2. MCP Protocol Compatibility Issues  
**Risk**: Claude Code integration fails due to protocol mismatches  
**Mitigation**: Implement exact JSON-RPC 2.0 specification compliance  
**Testing**: Use real Claude Code instance for validation

#### 3. Performance Regression in MinimalEmbedder
**Risk**: Hash-based embedding performance degrades during integration  
**Mitigation**: Benchmark before and after all changes  
**Target**: Maintain <1ms embedding generation time

#### 4. Test Infrastructure Complexity
**Risk**: Test setup becomes too complex, slowing development  
**Mitigation**: Keep tests simple and focused, avoid over-engineering  
**Rule**: Each test should be <50 lines and test single functionality

### MEDIUM RISK ITEMS

- Dependency version conflicts (mitigate with lockfile)
- Build time increases (monitor and optimize)
- Memory usage growth (profile continuously)

---

## QUALITY GATES CHECKLIST

**Phase 1 CANNOT advance to Phase 2 until ALL items checked:**

### ✅ Compilation Success
- [ ] `cargo build --release` completes without errors
- [ ] `cargo build --release --all-targets` succeeds
- [ ] No `#[cfg(feature = "...")]` warnings for missing features
- [ ] Clippy lints pass with `cargo clippy -- -D warnings`

### ✅ Functionality Validation  
- [ ] MCP server binary starts successfully
- [ ] Server responds to ping with valid JSON-RPC response
- [ ] MinimalEmbedder generates 768-dimensional normalized vectors
- [ ] Basic search returns relevant results
- [ ] Server shuts down gracefully on SIGTERM/SIGINT

### ✅ Integration Testing
- [ ] Claude Code MCP connection established
- [ ] Basic embed operation works through Claude Code
- [ ] Simple search query returns expected results
- [ ] Error conditions handled gracefully

### ✅ Test Coverage
- [ ] All core functions have unit tests
- [ ] Unit tests pass with `cargo test`
- [ ] Integration tests demonstrate end-to-end functionality
- [ ] Performance benchmarks establish baseline metrics

### ✅ Documentation
- [ ] README updated with working build instructions
- [ ] Configuration options documented
- [ ] MCP integration guide created
- [ ] Troubleshooting guide for common issues

---

## DELIVERABLES

### Code Deliverables
1. **Working MCP server binary** (`target/release/mcp_server`)
2. **Minimal configuration file** (`config/embed-search.toml`)
3. **Complete test suite** (unit + integration tests)
4. **Updated dependency manifest** (`Cargo.toml` with only required deps)

### Documentation Deliverables
1. **Phase 1 completion report** (this document updated with results)
2. **Configuration reference** (all settings explained)
3. **Integration guide** (how to use with Claude Code)
4. **Performance baseline report** (benchmark results)

### Validation Deliverables
1. **All quality gate items completed** (checklist above)
2. **End-to-end demo video** (Claude Code -> MCP server -> results)
3. **Test coverage report** (coverage percentage + critical paths)
4. **Performance benchmark results** (embedding speed, search latency)

---

## NEXT STEPS AFTER PHASE 1

Upon successful completion of all Phase 1 quality gates:

1. **Conduct Phase 1 Review** - Technical demonstration and acceptance
2. **Update Project Status** - Communicate completion to stakeholders  
3. **Begin Phase 2 Planning** - Review `PHASE_2_INTEGRATION_LAYER.md`
4. **Archive Phase 1 Branch** - Tag and merge to main development branch
5. **Celebrate Success** - Acknowledge working foundation achievement

---

## TROUBLESHOOTING GUIDE

### Common Issues and Solutions

**Issue**: `cargo build` fails with missing dependencies
**Solution**: Add `tempfile = "3.0"` and `toml = "0.8"` to Cargo.toml

**Issue**: MCP server won't start  
**Solution**: Create `config/embed-search.toml` with minimal configuration

**Issue**: Claude Code can't connect
**Solution**: Verify MCP server config in Claude settings, check stdio transport

**Issue**: Tests fail with "file not found"
**Solution**: Use `tempfile` crate for test file creation, clean up properly

**Issue**: Performance regression in embeddings
**Solution**: Profile with `cargo flamegraph`, check for additional allocations

---

**Phase 1 establishes the foundation that makes everything else possible. Quality and completeness here determines the success of the entire project.**