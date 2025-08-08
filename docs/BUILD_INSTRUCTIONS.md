# WORKING BUILD INSTRUCTIONS

## PHASE 1: IMMEDIATE STABILIZATION BUILD

### Prerequisites
- Rust 1.70+ 
- Git
- 8GB+ RAM recommended
- Windows 10/11, macOS, or Linux

### Quick Start (Minimal Working System)

```bash
# 1. Clone and enter directory
git clone [repository-url]
cd embed-search

# 2. Build with minimal features only
cargo build --features minimal --release

# 3. Run basic tests (should pass)
cargo test --features minimal --lib

# 4. Create simple test
echo "fn hello() { println!(\"Hello, world!\"); }" > test.rs
cargo run --features minimal --bin simple_search test.rs "hello"
```

**Expected Result**: Basic text search working, all core tests passing.

---

## PHASE 2: TEXT SEARCH BUILD

### Enable Tantivy Full-Text Search

```bash
# 1. Build with text search features
cargo build --features text-search --release

# 2. Test text search functionality
cargo test --features text-search text_search

# 3. Index a directory
cargo run --features text-search --bin index_directory ./src

# 4. Search indexed content
cargo run --features text-search --bin search_index "function"
```

**Expected Result**: Fast full-text search across large codebases.

---

## PHASE 3: SYMBOL SEARCH BUILD

### Enable Code Symbol Extraction

```bash
# 1. Build with symbol search
cargo build --features symbol-search --release

# 2. Test symbol extraction
cargo test --features symbol-search symbol

# 3. Search for functions/classes
cargo run --features symbol-search --bin search_symbols "main"
```

**Expected Result**: Find function definitions, class declarations across languages.

---

## PHASE 4: FULL SYSTEM BUILD

### Enable All Features (Optional)

```bash
# 1. Build complete system (requires 16GB RAM)
cargo build --features full-system --release

# 2. Download ML models (optional)
./scripts/download_models.sh

# 3. Test semantic search
cargo test --features full-system semantic
```

**Expected Result**: ML-enhanced semantic search with vector similarity.

---

## BUILD CONFIGURATIONS

### Development Builds

```bash
# Fast iteration (minimal features)
cargo build --features minimal

# Text search development
cargo build --features text-search

# Debug builds with all features
cargo build --features full-system
```

### Production Builds

```bash
# Optimized minimal build
cargo build --features minimal --release

# Production text search
cargo build --features text-search --release --target-dir ./target/production

# Full system production (requires model files)
cargo build --features full-system --release
```

### Test Configurations

```bash
# Core functionality only
cargo test --features minimal --lib

# Text search tests
cargo test --features text-search

# Integration tests (all features)
cargo test --features full-system --test integration_test
```

---

## TROUBLESHOOTING BUILD ISSUES

### Issue: Config initialization errors

```
Error: Configuration not initialized. Call Config::init() first.
```

**Solution**: Update code to use `Config::init_safe()`:

```rust
// OLD (fails):
Config::init()?;

// NEW (works with defaults):
Config::init_safe()?;
```

### Issue: Tantivy compilation errors

```
error[E0432]: unresolved import `tantivy::IndexSettings`
```

**Solution**: Update to new Tantivy v0.24 API:

```rust
// OLD:
use tantivy::IndexSettings;

// NEW:
use tantivy::{Index, schema::*};
```

### Issue: Tree-sitter linking errors

```
error: linking with `cc` failed: exit status: 1
```

**Solutions**:
- **Windows**: Install Visual Studio Build Tools
- **macOS**: Install Xcode command line tools: `xcode-select --install`
- **Linux**: Install build essentials: `sudo apt install build-essential`

### Issue: Memory errors during ML builds

```
error: linking failed due to insufficient memory
```

**Solutions**:
- Increase system swap space
- Build with fewer parallel jobs: `cargo build -j 1`
- Use minimal features: `cargo build --features text-search`

### Issue: Test failures

```
test result: FAILED. 69 passed; 6 failed
```

**Solution**: Use isolated test runner:

```bash
# Run tests in sequence to avoid conflicts
cargo test --features minimal -- --test-threads=1

# Run specific test groups
cargo test --features minimal config
cargo test --features minimal bm25
```

---

## PERFORMANCE TUNING

### Compile Time Optimization

**File**: `.cargo/config.toml`

```toml
[build]
# Optimize for faster builds during development
rustflags = ["-C", "target-cpu=native"]

[profile.dev]
opt-level = 1
debug = true
incremental = true

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
```

### Memory Usage Control

**Environment Variables**:

```bash
# Limit memory usage during builds
export CARGO_BUILD_JOBS=2
export RUSTFLAGS="-C link-arg=-Wl,--no-keep-memory"

# Development builds (faster)
export EMBED_SEARCH_MODE="development"

# Production builds (optimized)
export EMBED_SEARCH_MODE="production"
```

### Disk Space Management

**Clean commands**:

```bash
# Clean build artifacts
cargo clean

# Remove model files (saves ~500MB)
rm -rf models/*.gguf

# Clean test data
rm -rf target/debug/test_data
```

---

## VERIFICATION CHECKLIST

### Phase 1 Complete (Minimal System)
- [ ] `cargo build --features minimal` succeeds
- [ ] `cargo test --features minimal --lib` passes all tests
- [ ] Basic text search returns results
- [ ] Configuration system works with defaults
- [ ] Memory usage < 1GB during build

### Phase 2 Complete (Text Search)
- [ ] `cargo build --features text-search` succeeds
- [ ] Tantivy indexing works
- [ ] Full-text search returns ranked results
- [ ] Search performance < 1 second on 10K files
- [ ] Fuzzy search handles typos

### Phase 3 Complete (Symbol Search)  
- [ ] `cargo build --features symbol-search` succeeds
- [ ] Symbol extraction works for Rust/Python/JavaScript
- [ ] Function/class definitions found correctly
- [ ] Code-aware search improves relevance
- [ ] Cross-reference navigation works

### Phase 4 Complete (Full System)
- [ ] `cargo build --features full-system` succeeds
- [ ] ML models load correctly (optional)
- [ ] Semantic search improves results (optional)
- [ ] Vector similarity search works (optional)
- [ ] Full system stable under load

---

## COMMON BUILD COMMANDS REFERENCE

```bash
# Development workflow
cargo check --features minimal                    # Fast syntax check
cargo build --features text-search               # Incremental build
cargo test --features minimal --lib              # Core tests
cargo clippy --features text-search             # Linting

# Production workflow
cargo build --features text-search --release    # Optimized build
cargo test --features full-system --release     # Full test suite
cargo bench --features full-system              # Performance benchmarks

# Debugging workflow
RUST_LOG=debug cargo test --features minimal config  # Debug specific tests
RUST_BACKTRACE=1 cargo test --features minimal       # Stack traces on panic
cargo build --features minimal --verbose             # Verbose build output

# Deployment workflow
cargo build --features text-search --release --target-dir ./deploy
strip ./deploy/release/embed-search                           # Reduce binary size
./deploy/release/embed-search --version                      # Verify build
```

---

## NEXT STEPS

1. **Start with Phase 1**: Get minimal system working first
2. **Validate thoroughly**: Ensure each phase is stable before advancing
3. **Profile performance**: Use `cargo bench` to measure improvements
4. **Deploy incrementally**: Roll out features gradually to production

**Success Criteria**: Each phase should be independently deployable and provide value to users.