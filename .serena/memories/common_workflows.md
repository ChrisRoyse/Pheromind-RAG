# Common Development Workflows

## Adding a New Search Backend

### 1. Create Implementation Module
```
1. find_file "*.rs" "src/search"  # See existing patterns
2. Create new file: src/search/new_backend.rs
3. get_symbols_overview "src/search/bm25.rs"  # Study pattern
4. Implement SearchAdapter trait
```

### 2. Wire Into Unified Searcher
```
1. find_symbol "UnifiedSearcher" depth=1
2. Add new field to struct
3. find_symbol "UnifiedSearcher/new" include_body=true
4. Add initialization in constructor
5. find_symbol "search_documents" include_body=true
6. Integrate into search flow
```

### 3. Add Feature Flag (if needed)
```
1. Edit Cargo.toml [features] section
2. search_for_pattern "#\[cfg\(feature" for examples
3. Add conditional compilation
```

## Implementing a New CLI Command

### 1. Define Command
```
1. find_symbol "Commands" relative_path="src/main.rs"
2. Add new variant to enum
3. Add command-line args structure
```

### 2. Implement Handler
```
1. search_for_pattern "async fn.*_command" relative_path="src/main.rs"
2. Create new async function following pattern
3. find_symbol "main" include_body=true relative_path="src/main.rs"
4. Add match arm for new command
```

### 3. Test Command
```
1. Create test in src/main.rs #[cfg(test)] section
2. Run: cargo test command_name
```

## Adding ML Embedding Support

### 1. Check Current Implementation
```
1. get_symbols_overview "src/embedding/nomic.rs"
2. find_symbol "NomicEmbedding" depth=1
3. Check model loading pattern
```

### 2. Add New Model
```
1. Create new file in src/embedding/
2. Implement embedding trait
3. Add to src/embedding/mod.rs exports
4. Update config for model selection
```

### 3. Integrate with Storage
```
1. find_symbol "store_embeddings" 
2. Ensure compatibility with vector dimensions
3. Update src/storage/lancedb.rs if needed
```

## Debugging Search Quality Issues

### 1. Enable Debug Logging
```bash
RUST_LOG=embed_search=debug cargo run -- search "query"
```

### 2. Check Components
```
1. Test BM25 alone: disable ML in config
2. Test ML alone: disable BM25 in config  
3. Check fusion scores: find_symbol "FusionScorer"
4. Verify chunking: find_symbol "chunk_file"
```

### 3. Inspect Cache
```
1. Check .embed/cache/ for cached embeddings
2. Clear cache if stale: rm -rf .embed/cache/
3. find_symbol "EmbeddingCache" for logic
```

## Optimizing Performance

### 1. Profile Current Performance
```bash
cargo build --release
cargo bench
```

### 2. Identify Bottlenecks
```
1. search_for_pattern "measure_time|elapsed"
2. find_symbol "benchmark" substring_matching=true
3. Check src/observability/ for metrics
```

### 3. Common Optimizations
```
- Enable rayon parallelism
- Increase cache sizes
- Batch database operations
- Use release builds
```

## Setting Up New Development Environment

### 1. Initial Setup
```bash
# Clone and build
git clone <repo>
cd embed
cargo build --features "full-system"
```

### 2. Initialize Indices
```bash
cargo run -- index --directory .
cargo run -- stats
```

### 3. Test Search
```bash
cargo run -- search "test query"
```

## Migrating Between Storage Backends

### 1. Export Current Data
```
1. find_symbol "export" substring_matching=true
2. Run export command/function
3. Backup .embed/ directory
```

### 2. Switch Backend
```
1. Update Cargo.toml features
2. Update config.toml storage settings
3. Clear old storage: cargo run -- clear
```

### 3. Import Data
```
1. Run reindex: cargo run -- index
2. Verify: cargo run -- stats
```

## Running Full Test Suite

### Quick Validation
```bash
# Format and lint
cargo fmt -- --check
cargo clippy --all-features

# Core tests
cargo test --features "core"

# Full tests
cargo test --all-features
```

### Performance Validation
```bash
cargo bench
cargo test --release -- --ignored  # Performance tests
```

## Contributing Changes

### 1. Before Changes
```
1. Update from main: git pull
2. Create feature branch: git checkout -b feature
3. Run tests: cargo test
```

### 2. Make Changes
```
1. Follow code style (cargo fmt)
2. Add tests for new functionality
3. Update documentation
4. Check feature flags
```

### 3. Before Commit
```
1. Run full test suite
2. Check cargo clippy warnings
3. Update CHANGELOG if applicable
4. Verify no secrets in code
```